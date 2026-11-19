// One request over its candidate routes: admission, retries, fall-through, metering and a trace.

import { numberText } from "@nyaaai/contract/number";
import type {
	Admission,
	Blocked,
	ErrorStatus,
	GateOutcome,
	GateSettled,
	GateWaiting,
	Notice,
	Ticket,
} from "@nyaaai/contract/shapes/health";
import type { TraceEntry } from "@nyaaai/contract/shapes/lifecycle";
import type { NoticeKind, Passed, Skip, SkipReason } from "@nyaaai/contract/shapes/provider";
import type { Failure, NormalizedUsage, StepResult } from "@nyaaai/contract/shapes/step";
import type { CallTag, MeteredCall } from "@nyaaai/contract/shapes/usage";
import { deadline, type Scheduler } from "@nyaaai/contract/time";
import { failureOf } from "../failure";
import type { Gate } from "../health/health";
import { told } from "../kinds/join";
import { raced } from "../models/view";
import { overloadedText, unavailableText } from "../noticeText";
import { skipReasonOf } from "../provider/disposition";
import type { Candidates, Resolved, Routes } from "../provider/resolver";
import type { FailureBilled } from "../step";
import { stepCost, wantsLongCache } from "../usage/cost";
import type { Meter } from "../usage/meter";
import { RetryPolicy } from "./retry";

////////////////////////////////
//  Interfaces & Types

export type Ended = { kind: "stopped" } | { kind: "notice"; notice: Notice };

export type Attempt<T> = { ok: true; value: T; route: Resolved } | { ok: false; outcome: Ended; failure?: Failure };

export interface Observer {
	/** False after streamed output. */
	fresh?(): boolean;
	onRetry?(attempt: number, waitMs: number, status: ErrorStatus): void;
	onWait?(waiting: GateWaiting | null): void;
	onSkip?(skip: Skip): void;
	/** Told one entry per effect, in order. With none given no entry is built. */
	onTrace?(entry: TraceEntry): void;
}

/** What a reply adds to the meter's call; a field may be undefined. */
export interface Metered {
	usage?: NormalizedUsage | undefined;
	provider?: string | undefined;
	generationId?: string | undefined;
	finishReason?: string | undefined;
	failure?: string | undefined;
	cancelled?: boolean | undefined;
}

export interface Call<T> {
	tag: CallTag;
	signal: AbortSignal;
	/** Which ended the signal, when it ended: the caller's deadline or a cancel. */
	ended?(): "deadline" | "caller" | undefined;
	/** Two nonblocking tries. */
	live?: boolean;
	send(route: Resolved, firstToken: () => void): Promise<T>;
	read(result: T): Metered;
	/** Usage billed before failure. */
	billed?(): FailureBilled | undefined;
}

export interface LifecycleDeps {
	gate?: Gate;
	meter?: Meter;
	retry?: RetryPolicy;
	/** The time and the waits of the retries. */
	scheduler: Scheduler;
}

type Billing = { usage?: NormalizedUsage };

type Refusal = { skip: SkipReason } | { blocked: Blocked };

interface RouteDeps {
	attempt<T extends Billing>(
		candidates: Candidates,
		passed: readonly Passed[],
		keep: Set<string> | undefined,
		call: Call<T>,
		o: Observer,
	): Promise<Attempt<T>>;
	refusal(route: Resolved, last: boolean, keep: ReadonlySet<string> | undefined): Refusal | undefined;
}

/** Fall-through failure. */
type Onward = { onward: Failure };

/** Deferred by ZDR. */
type Kept = { kept: true };

type Gated = Admission | { kind: "go"; ticket: undefined };

////////////////////////////////
//  Constants

export const STOPPED: Ended = { kind: "stopped" };

const LIVE_TRIES = 2;

/** The most a limits read may take. */
const LIMITS_MS = 5000;

////////////////////////////////
//  Functions & Helpers

/** Adds route context. */
export function notice(kind: NoticeKind, failure?: Failure, route?: Resolved): Ended {
	return {
		kind: "notice",
		notice: {
			kind,
			...(failure === undefined ? {} : { status: failure.status, message: failure.message }),
			...(route === undefined ? {} : { entry: route.entry.label }),
		},
	};
}

function overloaded(route: Resolved): Ended {
	const model = route.view.row.label;
	return { kind: "notice", notice: { kind: "providerOverloaded", model, message: overloadedText(model) } };
}

/** The failure a gate's give-up stands for, so the route falls through as an overload does. */
function gaveUpOn(route: Resolved): Failure {
	return { category: "overloaded", status: "overloaded", message: overloadedText(route.view.row.label) };
}

/** Tells the sink one entry; nothing is built without a sink, and a throwing sink changes nothing. */
function trace(o: Observer, entry: () => TraceEntry): void {
	const sink = o.onTrace;
	if (sink === undefined) return;
	told(() => sink(entry()));
}

/** Distinguish a deadline from a cancel. */
function abandoned(call: Call<unknown>): Pick<MeteredCall, "failure" | "cancelled"> {
	return call.ended?.() === "deadline" ? { failure: "timedOut" } : { cancelled: true };
}

function statusText(status: ErrorStatus): string {
	return typeof status === "number" ? numberText(status) : status;
}

function reasonOf(error: unknown, route: Resolved): string {
	const failure = failureOf(error);
	if (failure === undefined) return "error";
	if (failure.category === "overloaded") return "overloaded";
	const disposition = route.kind.failure(failure, route.view);
	return disposition.act === "retry" ? statusText(failure.status) : disposition.notice;
}

/** The fields that hold a value. */
function defined<T extends object>(fields: T): { [K in keyof T]?: Exclude<T[K], undefined> } {
	return Object.fromEntries(Object.entries(fields).filter(([, value]) => value !== undefined)) as {
		[K in keyof T]?: Exclude<T[K], undefined>;
	};
}

export function meteredOf(result: StepResult): Metered {
	return {
		usage: result.usage,
		provider: result.serving.host,
		generationId: result.serving.generationId,
		finishReason: result.finish,
	};
}

////////////////////////////////
//  Classes

/** Runs candidates in order. */
export class RequestLifecycle {
	private readonly retry: RetryPolicy;

	constructor(private readonly d: LifecycleDeps) {
		this.retry = d.retry ?? new RetryPolicy();
	}

	run<T extends Billing>(candidates: Candidates, call: Call<T>, o: Observer = {}): Promise<Attempt<T>> {
		return this.attempt(candidates, [], undefined, call, o);
	}

	/** Creates a stateful route. */
	begin(): RunRoute {
		return new RunRoute({
			attempt: (candidates, passed, keep, call, o) => this.attempt(candidates, passed, keep, call, o),
			refusal: (route, last, keep) => this.refusal(route, last, keep),
		});
	}

	/** Entries that serve a step join `keep`; the run's end is the last entry told. */
	private async attempt<T extends Billing>(
		candidates: Candidates,
		passed: readonly Passed[],
		keep: Set<string> | undefined,
		call: Call<T>,
		o: Observer,
	): Promise<Attempt<T>> {
		const result = await this.walk(candidates, passed, keep, call, o);
		trace(o, (): TraceEntry => {
			if (result.ok)
				return {
					kind: "served",
					entry: result.route.entry.id,
					label: result.route.entry.label,
					model: result.route.key,
				};
			return result.outcome.kind === "stopped"
				? { kind: "stopped" }
				: { kind: "notice", notice: result.outcome.notice };
		});
		return result;
	}

	private async walk<T extends Billing>(
		candidates: Candidates,
		passed: readonly Passed[],
		keep: Set<string> | undefined,
		call: Call<T>,
		o: Observer,
	): Promise<Attempt<T>> {
		const skip = (route: Resolved, reason: SkipReason, cause: "deferred" | "fellThrough") => {
			trace(o, () => ({ kind: "skip", cause, entry: route.entry.id, label: route.entry.label, reason }));
			o.onSkip?.({ entry: route.entry.id, label: route.entry.label, reason });
		};
		const ahead = (at: number) => {
			for (const { before, ...each } of passed) {
				if (before !== at) continue;
				trace(o, () => ({ kind: "skip", cause: "passed", ...each }));
				o.onSkip?.(each);
			}
		};
		for (const [at, route] of candidates.slice(0, -1).entries()) {
			ahead(at);
			const refused = this.refusal(route, false, keep);
			if (refused !== undefined && "skip" in refused) {
				skip(route, refused.skip, "deferred");
				continue;
			}
			const tried = await this.tries(route, call, o, keep, true);
			if ("kept" in tried) continue;
			if (!("onward" in tried)) return tried;
			this.failed(route, tried.onward, true, o);
			skip(route, skipReasonOf(tried.onward), "fellThrough");
		}
		ahead(candidates.length - 1);
		const last = candidates[candidates.length - 1] ?? candidates[0];
		const refused = this.refusal(last, true, keep);
		if (refused !== undefined && "blocked" in refused)
			return { ok: false, outcome: { kind: "notice", notice: { ...refused.blocked, entry: last.entry.label } } };
		const tried = await this.tries(last, call, o, keep, false);
		return "kept" in tried ? { ok: false, outcome: notice("noZdrHost") } : tried;
	}

	/** Earlier defer; last blocks. */
	private refusal(route: Resolved, last: boolean, keep: ReadonlySet<string> | undefined): Refusal | undefined {
		const { gate } = this.d;
		if (gate === undefined) return undefined;
		if (last) {
			const blocked = gate.blocked(route, keep);
			return blocked === undefined ? undefined : { blocked };
		}
		const skip = gate.deferred(route, keep);
		return skip === undefined ? undefined : { skip };
	}

	private tries<T extends Billing>(
		route: Resolved,
		call: Call<T>,
		o: Observer,
		run: Set<string> | undefined,
		onward: true,
	): Promise<Attempt<T> | Onward | Kept>;
	private tries<T extends Billing>(
		route: Resolved,
		call: Call<T>,
		o: Observer,
		run: Set<string> | undefined,
		onward: false,
	): Promise<Attempt<T> | Kept>;
	private async tries<T extends Billing>(
		route: Resolved,
		call: Call<T>,
		o: Observer,
		run: Set<string> | undefined,
		onward: boolean,
	): Promise<Attempt<T> | Onward | Kept> {
		// Backoff consumes budget.
		let waited = 0;
		let again = false;
		for (let attempt = 1; ; ) {
			const readmitted = again;
			const admission =
				call.live === true ? this.admitLive(route, call.signal, o) : await this.admit(route, call, o, again);
			again = false;
			if (admission.kind === "stopped") return { ok: false, outcome: STOPPED };
			if (admission.kind === "gaveUp") {
				// A give-up ends a route that is not the last, not the request.
				if (onward) return { onward: gaveUpOn(route) };
				return { ok: false, outcome: overloaded(route) };
			}
			const { ticket } = admission;
			// Recheck ZDR on admission.
			if (route.zdrOnly && route.view.dataKept === true) {
				trace(o, () => ({ kind: "kept", entry: route.entry.id, model: route.key }));
				this.settle(route, ticket, "cancelled", undefined, o);
				return { kept: true };
			}
			trace(o, () => ({
				kind: "attempt",
				entry: route.entry.id,
				model: route.key,
				attempt,
				live: call.live === true,
				again: readmitted,
			}));
			try {
				const value = await this.metered(route, call, o);
				this.settle(route, ticket, "ok", undefined, o);
				this.served(route, value.usage, run, "ok", o);
				return { ok: true, value, route };
			} catch (error) {
				if (call.signal.aborted) {
					this.settle(route, ticket, "cancelled", undefined, o);
					return { ok: false, outcome: STOPPED };
				}
				const failure = failureOf(error);
				if (failure === undefined) {
					this.settle(route, ticket, "failed", undefined, o);
					trace(o, () => ({
						kind: "threw",
						entry: route.entry.id,
						message: error instanceof Error ? error.message : "The send threw a value that is no error.",
					}));
					throw error;
				}
				this.served(route, call.billed?.()?.usage ?? failure.billed?.usage, run, "failed", o);
				// Empty replies stay fresh.
				const fresh = failure.category === "empty" || o.fresh === undefined || o.fresh();
				const failedAt = (disposition: ReturnType<typeof route.kind.failure> | undefined) =>
					trace(o, () => ({
						kind: "failure",
						entry: route.entry.id,
						model: route.key,
						failure,
						...(disposition === undefined ? {} : { disposition }),
						fresh,
						onward,
					}));
				if (failure.category === "overloaded") {
					const settled = this.settle(route, ticket, "overloaded", failure.retryAfterMs, o);
					failedAt(undefined);
					if (onward) return { onward: failure };
					if (call.live === true || settled === undefined || settled === "gaveUp")
						return { ok: false, outcome: overloaded(route), failure };
					if (!fresh) return { ok: false, outcome: notice("streamFailed", failure), failure };
					again = true;
					continue;
				}
				this.settle(route, ticket, "failed", undefined, o);
				const disposition = route.kind.failure(failure, route.view);
				failedAt(disposition);
				if (!fresh) {
					if (onward && disposition.act !== "stop") return { onward: failure };
					return this.ended(route, failure, notice("streamFailed", failure), o);
				}
				if (disposition.act !== "retry") {
					if (onward && disposition.act === "fallThrough") return { onward: failure };
					return this.ended(
						route,
						failure,
						await this.noticeFor(disposition.notice, failure, route, call.signal, o),
						o,
					);
				}
				const waitMs =
					call.live === true && attempt >= LIVE_TRIES ? undefined : this.retry.wait(failure, attempt, waited);
				if (waitMs === undefined) {
					if (onward) return { onward: failure };
					return this.ended(route, failure, notice("retriesExhausted", failure), o);
				}
				trace(o, () => ({
					kind: "retry",
					entry: route.entry.id,
					attempt,
					waitMs,
					why: failure.retryAfterMs === undefined ? "backoff" : "retryAfter",
					status: failure.status,
				}));
				o.onRetry?.(attempt, waitMs, failure.status);
				attempt++;
				waited += waitMs;
				try {
					await this.d.scheduler.sleep(waitMs, call.signal);
				} catch (sleepError) {
					if (call.signal.aborted) return { ok: false, outcome: STOPPED };
					throw sleepError;
				}
			}
		}
	}

	private ended(route: Resolved, failure: Failure, outcome: Ended, o: Observer): Attempt<never> {
		this.failed(route, failure, false, o);
		return { ok: false, outcome, failure };
	}

	/** Tells the gate a route failed. */
	private failed(route: Resolved, failure: Failure, onward: boolean, o: Observer): void {
		const { gate } = this.d;
		if (gate === undefined) return;
		gate.failed(route, failure, onward);
		trace(o, () => ({ kind: "gateFailed", entry: route.entry.id, failure, onward }));
	}

	/** Tells the gate a reply's billing. */
	private served(
		route: Resolved,
		usage: NormalizedUsage | undefined,
		run: Set<string> | undefined,
		reply: "ok" | "failed",
		o: Observer,
	): void {
		const { gate } = this.d;
		if (gate === undefined) return;
		gate.served(route, usage, run);
		trace(o, () => ({
			kind: "billing",
			entry: route.entry.id,
			reply,
			...(usage === undefined ? {} : { usage }),
		}));
	}

	private async metered<T>(route: Resolved, call: Call<T>, o: Observer): Promise<T> {
		const { meter, scheduler } = this.d;
		if (meter === undefined) return call.send(route, () => {});
		const { view } = route;
		const at = scheduler.nowMs();
		let first: number | undefined;
		const record = (of: () => Metered) => {
			try {
				const fields = of();
				const metered: MeteredCall = {
					tag: call.tag,
					model: route.key,
					entry: route.entry.id,
					label: route.entry.label,
					at,
					ms: scheduler.nowMs() - at,
					...(first === undefined ? {} : { firstTokenMs: first }),
					cost: stepCost(fields.usage, view, wantsLongCache(call.tag, view)),
					...defined(fields),
				};
				trace(o, () => ({ kind: "metered", call: metered }));
				meter.called(metered);
			} catch {
				// Metering cannot alter outcomes.
			}
		};
		let result: T;
		try {
			result = await call.send(route, () => {
				first ??= scheduler.nowMs() - at;
			});
		} catch (error) {
			record(() => {
				const spent = call.billed?.();
				const owed = failureOf(error)?.billed;
				return {
					usage: spent?.usage ?? owed?.usage,
					provider: spent?.host ?? owed?.host,
					generationId: spent?.generationId ?? owed?.generationId,
					...(call.signal.aborted ? abandoned(call) : { failure: reasonOf(error, route) }),
				};
			});
			throw error;
		}
		record(() => call.read(result));
		return result;
	}

	/** Live calls never wait. */
	private admitLive(route: Resolved, signal: AbortSignal, o: Observer): Gated {
		if (signal.aborted) return { kind: "stopped" };
		const gate = this.d.gate;
		if (gate === undefined) return { kind: "go", ticket: undefined };
		const ticket = gate.tryAdmit(route);
		const admission: Admission = ticket === null ? { kind: "gaveUp" } : { kind: "go", ticket };
		trace(o, () => ({ kind: "admitted", entry: route.entry.id, admission }));
		return admission;
	}

	private async admit(route: Resolved, call: Call<unknown>, o: Observer, again: boolean): Promise<Gated> {
		const { gate } = this.d;
		const { signal } = call;
		if (signal.aborted) return { kind: "stopped" };
		if (gate === undefined) return { kind: "go", ticket: undefined };
		let held = false;
		const onWait = (waiting: GateWaiting) => {
			held = true;
			trace(o, () => ({ kind: "held", entry: route.entry.id, waiting }));
			o.onWait?.(waiting);
		};
		const admission = await gate.admit(route, signal, onWait, again);
		trace(o, () => ({ kind: "admitted", entry: route.entry.id, admission }));
		if (held) {
			trace(o, () => ({ kind: "released", entry: route.entry.id }));
			o.onWait?.(null);
		}
		if (admission.kind !== "go" || !signal.aborted) return admission;
		this.settle(route, admission.ticket, "cancelled", undefined, o);
		return { kind: "stopped" };
	}

	private settle(
		route: Resolved,
		ticket: Ticket | undefined,
		outcome: GateOutcome,
		retryAfterMs: number | undefined,
		o: Observer,
	): GateSettled | undefined {
		const { gate } = this.d;
		if (ticket === undefined || gate === undefined) return undefined;
		const answer = gate.settle(ticket, outcome, retryAfterMs);
		trace(o, () => ({
			kind: "settled",
			entry: route.entry.id,
			outcome,
			...(retryAfterMs === undefined ? {} : { retryAfterMs }),
			answer,
		}));
		return answer;
	}

	/** The kind's limits, none when the read fails or outlives its bound. */
	private async limitsOf(route: Resolved, signal: AbortSignal) {
		if (route.kind.limits === undefined) return undefined;
		const bound = deadline(this.d.scheduler, LIMITS_MS, signal);
		try {
			return await raced(route.kind.limits(route.transport, bound.signal), bound.signal);
		} catch {
			return undefined;
		} finally {
			bound.clear();
		}
	}

	private async noticeFor(
		kind: NoticeKind,
		failure: Failure,
		route: Resolved,
		signal: AbortSignal,
		o: Observer,
	): Promise<Ended> {
		const ended = notice(kind, failure, route);
		if (ended.kind !== "notice") return ended;
		if (kind === "refused" && failure.reasons !== undefined) ended.notice.reasons = failure.reasons;
		if (kind === "modelUnavailable") {
			ended.notice.model = route.view.row.label;
			ended.notice.message = unavailableText(ended.notice.model, ended.notice.entry);
		}
		if (kind === "spendLimit") {
			const limits = await this.limitsOf(route, signal);
			if (route.kind.limits !== undefined)
				trace(o, () => ({
					kind: "limits",
					entry: route.entry.id,
					read: limits !== undefined,
					...(limits?.limit === undefined ? {} : { limit: limits.limit }),
					...(limits?.reset === undefined ? {} : { reset: limits.reset }),
				}));
			if (limits?.limit !== undefined) ended.notice.limit = limits.limit;
			if (limits?.reset !== undefined) ended.notice.reset = limits.reset;
			const waited =
				failure.retryAfterMs === undefined ? undefined : this.d.scheduler.nowMs() + failure.retryAfterMs;
			const resetsAt = failure.category === "allowance" ? (failure.resetsAt ?? waited) : undefined;
			if (resetsAt !== undefined) ended.notice.resetsAt = resetsAt;
		}
		return ended;
	}
}

/** Tracks per-run route state: `keep` holds every entry that served a step of the run, whatever it billed. */
export class RunRoute {
	private readonly keep = new Set<string>();
	private readonly skipped = new Set<string>();

	constructor(private readonly d: RouteDeps) {}

	async run<T extends Billing>(routes: Routes, call: Call<T>, o: Observer = {}): Promise<Attempt<T>> {
		const result = await this.d.attempt(routes.candidates, routes.passed, this.keep, call, {
			...o,
			onSkip: (skip) => {
				if (this.skipped.has(skip.entry)) return;
				this.skipped.add(skip.entry);
				o.onSkip?.(skip);
			},
		});
		if (result.ok) this.keep.add(result.route.entry.id);
		return result;
	}

	/** Eligible routes, or all. */
	sizing(candidates: Candidates): Candidates {
		const last = candidates.length - 1;
		const [first, ...rest] = candidates.filter(
			(route, at) => this.d.refusal(route, at === last, this.keep) === undefined,
		);
		return first === undefined ? candidates : [first, ...rest];
	}
}
