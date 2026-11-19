// Entry health: key status, bills, pool cooldowns and gates, read through ports.

import type { EntryKeys } from "@nyaaai/contract/entryKeys";
import type { Ids } from "@nyaaai/contract/id";
import { canonicalJson } from "@nyaaai/contract/json";
import type {
	Admission,
	Blocked,
	EntryHealth,
	GateOutcome,
	GateSettled,
	GateWaiting,
	HealthChange,
	PoolEvent,
	PoolVerdict,
	Ticket,
} from "@nyaaai/contract/shapes/health";
import type { Billed, KeyState, ProviderEntry, ProviderKindName, SkipReason } from "@nyaaai/contract/shapes/provider";
import type { Failure, NormalizedUsage } from "@nyaaai/contract/shapes/step";
import type { Scheduler } from "@nyaaai/contract/time";
import type { EntryRegistry, Era, Incarnation, RegistryEvent } from "../provider/incarnation";
import type { KindDeps, ProviderKind } from "../provider/kind";
import type { Resolved } from "../provider/resolver";
import { type GateState, ProviderGate } from "./gate";
import { KEY_CHECK_MS, KeyReader } from "./keyReader";
import { CLEAR, step, verdict } from "./pool";

////////////////////////////////
//  Interfaces & Types

/** What the lifecycle calls: per-route admission, cooldowns and skips. */
export interface Gate {
	/** Waits through route pauses; `retired` once its incarnation ended. */
	admit(
		route: Resolved,
		signal: AbortSignal,
		onWait?: (waiting: GateWaiting) => void,
		again?: boolean,
	): Promise<Admission>;
	/** Never waits; none once its incarnation ended. */
	tryAdmit(route: Resolved): Ticket | null;
	settle(ticket: Ticket, outcome: GateOutcome, retryAfterMs?: number): GateSettled;
	/** Balance-cooldown exemption. */
	deferred(route: Resolved, keep?: ReadonlySet<string>): SkipReason | undefined;
	/** Last-route refusal. */
	blocked(route: Resolved, keep?: ReadonlySet<string>): Blocked | undefined;
	/** True when the report was dropped: its era ended. */
	failed(route: Resolved, failure: Failure, onward: boolean, ticket?: Ticket): boolean;
	/** Balance replies join `run`; true when the report was dropped as `failed` says. */
	served(route: Resolved, usage: NormalizedUsage | undefined, run?: Set<string>, ticket?: Ticket): boolean;
}

/** One reply's billing. */
export interface Billing {
	source: string;
	/** Weighted USD. */
	allowance?: number;
	cost?: number;
}

export interface HealthDeps {
	/** Gate pauses, rechecks and the time of every pool event. */
	scheduler: Scheduler;
	/** Asked once, for the gate's name. */
	ids: Ids;
	/** Shared with the resolver. */
	entries: EntryRegistry;
	/** The store's keys; health offers `entryKeys` over them. */
	keys: Pick<EntryKeys, "get">;
	kinds: Partial<Record<ProviderKindName, ProviderKind>>;
	/** What a kind's check stands on. */
	kindDeps(kind: ProviderKindName): KindDeps;
	pauseMs?: number;
	maxPauses?: number;
}

type Part = HealthChange["changed"][number];

////////////////////////////////
//  Constants

const PAUSE_MS = 60_000;

const RECHECK_MS = 60_000;

/** Declaration order of the shape's parts. */
const PARTS: readonly Part[] = ["key", "billed", "pool", "waiting"];

////////////////////////////////
//  Functions & Helpers

const none = (): KeyState => ({ kind: "none" });

function keyOf(route: Pick<Resolved, "entry" | "key">): string {
	return `${route.entry.id}\n${route.key}`;
}

const ticketKey = (ticket: Ticket): string => canonicalJson(ticket);

function total(sum: number | undefined, more: number | undefined): number | undefined {
	return sum === undefined && more === undefined ? undefined : (sum ?? 0) + (more ?? 0);
}

function billedWith(was: Billed | undefined, reply: Billing): Billed {
	const allowance = total(was?.allowance, reply.allowance);
	const cost = total(was?.cost, reply.cost);
	return {
		billing: reply.source,
		...(allowance === undefined ? {} : { allowance }),
		...(cost === undefined ? {} : { cost }),
	};
}

////////////////////////////////
//  Classes

/** Entry health and routing. */
export class ProviderHealth implements Gate {
	/** Store keys, unless health holds the entry out. */
	readonly entryKeys: EntryKeys;
	private readonly gate: ProviderGate;
	/** Readers dropped while a check ran. */
	private retired: KeyReader[] = [];
	private readonly listeners = new Set<(change: HealthChange) => void>();
	private readonly unsubscribe: (() => void)[] = [];
	/** Depth of the running changes whose telling waits for their end. */
	private batching = 0;
	private readonly pending = new Map<string, Set<Part>>();
	/** Changes made and not yet told, oldest first. */
	private readonly outbox: HealthChange[] = [];
	private delivering = false;
	private disposed = false;

	/** Defers initial key checks. */
	constructor(private readonly d: HealthDeps) {
		this.gate = new ProviderGate({
			scheduler: d.scheduler,
			ids: d.ids,
			...(d.pauseMs === undefined ? {} : { pauseMs: d.pauseMs }),
			...(d.maxPauses === undefined ? {} : { maxPauses: d.maxPauses }),
			onTicket: (ticket) => this.admitted(ticket),
		});
		this.entryKeys = {
			get: async (id) => {
				try {
					const incarnation = d.entries.incarnation(id);
					if (incarnation === undefined) return { kind: "none" };
					const standing = this.standing(incarnation);
					if (standing !== undefined) return { kind: standing };
					const answer = await d.keys.get(id);
					// A URL kind needs no key, so an unreadable store is no key.
					return answer.kind === "unready" && this.keyless(incarnation.entry) ? { kind: "none" } : answer;
				} catch {
					return { kind: "unready" };
				}
			},
			onChange: (listener) => d.entries.onKeyChange(listener),
		};
		this.unsubscribe.push(
			d.entries.watch((events) => this.observe(events)),
			this.gate.onChange((owner) => this.touch(owner, "waiting")),
		);
	}

	/** Cached key state. */
	status(id: string): KeyState {
		return { ...(this.eraOf(id)?.status?.state ?? none()) };
	}

	/** Listed replies create bills. */
	bill(id: string): Billed | undefined {
		const held = this.eraOf(id)?.bill;
		return held === undefined ? undefined : { ...held };
	}

	/** Everything health tracks of one entry. */
	state(id: string): EntryHealth {
		const era = this.eraOf(id);
		const prefix = `${id}\n`;
		const billed = this.bill(id);
		return {
			entry: id,
			key: this.status(id),
			...(billed === undefined ? {} : { billed }),
			pool: structuredClone(era?.pool ?? CLEAR),
			waiting:
				era === undefined
					? []
					: this.gate
							.holding(era.gates)
							.filter((model) => model.startsWith(prefix))
							.map((model) => model.slice(prefix.length)),
		};
	}

	/** One model's gate as data; none until it is first used. */
	gateState(route: Pick<Resolved, "entry" | "key" | "incarnation">): GateState | undefined {
		const incarnation = this.owned(route);
		return incarnation.live ? this.gate.stateOf(incarnation.era.gates, keyOf(route)) : undefined;
	}

	/** Accumulates reply billing; with no ticket, the route's incarnation must live. */
	billed(id: string, reply: Billing, ticket?: Ticket, incarnation?: Incarnation): void {
		if (incarnation !== undefined) this.owned({ incarnation });
		let era: Era | undefined;
		if (ticket !== undefined)
			era = incarnation === undefined ? this.d.entries.era(ticket.era) : this.eraFor(incarnation, ticket);
		else if (incarnation !== undefined) era = incarnation.live ? incarnation.era : undefined;
		else era = this.eraOf(id);
		if (era === undefined) return;
		const was = era.bill;
		const next = billedWith(was, reply);
		if (was?.billing === next.billing && was.allowance === next.allowance && was.cost === next.cost) return;
		era.bill = next;
		this.touch(era.incarnation.id, "billed");
	}

	/** Replaces pending checks. */
	check(id: string): void {
		const incarnation = this.d.entries.incarnation(id);
		if (this.disposed || incarnation === undefined) return;
		const { era } = incarnation;
		this.batch(() => {
			void this.statusOf(era).reader.read();
		});
	}

	checkUnread(): void {
		for (const { id, era } of this.d.entries.incarnations()) if (era.status === undefined) this.check(id);
	}

	/** Settles when no key check is running. */
	async idle(): Promise<void> {
		for (;;) {
			const readers = this.d.entries.incarnations().flatMap(({ era }) => (era.status ? [era.status.reader] : []));
			const busy = [...readers, ...this.retired].filter((reader) => reader.busy());
			if (busy.length === 0) {
				this.retired = [];
				return;
			}
			await Promise.all(busy.map((reader) => reader.idle()));
		}
	}

	/** Told each change once; returns unsubscribe. */
	onChange(listener: (change: HealthChange) => void): () => void {
		this.listeners.add(listener);
		return () => {
			this.listeners.delete(listener);
		};
	}

	/** A key change answers the waiters at once; each asks again and the new gate is open. */
	admit(
		route: Resolved,
		signal: AbortSignal,
		onWait?: (waiting: GateWaiting) => void,
		again = false,
	): Promise<Admission> {
		const incarnation = this.owned(route);
		if (!incarnation.live) return Promise.resolve({ kind: "retired" });
		const asked = this.batch(() => this.gate.admit(incarnation.era.gates, keyOf(route), signal, onWait, again));
		// A key change moves the waiter to the new era.
		return asked.then((admission) =>
			admission.kind === "retired" && incarnation.live ? this.admit(route, signal, onWait, again) : admission,
		);
	}

	tryAdmit(route: Resolved): Ticket | null {
		const incarnation = this.owned(route);
		if (!incarnation.live) return null;
		return this.batch(() => this.gate.tryAdmit(incarnation.era.gates, keyOf(route)));
	}

	settle(ticket: Ticket, outcome: GateOutcome, retryAfterMs?: number): GateSettled {
		const era = this.d.entries.era(ticket.era);
		if (era === undefined || this.disposed) return "done";
		return this.batch(() => {
			if (era.probes.delete(ticketKey(ticket))) this.apply(era, { kind: "settled", ticket });
			return this.gate.settle(era.gates, ticket, outcome, retryAfterMs);
		});
	}

	deferred(route: Resolved, keep?: ReadonlySet<string>): SkipReason | undefined {
		const incarnation = this.owned(route);
		if (this.rejected(incarnation.era)) return "keyRejected";
		const pool = this.verdict(incarnation, keep);
		if (pool !== undefined) return pool.skip;
		return this.gate.ready(incarnation.era.gates, keyOf(route)) ? undefined : "paused";
	}

	blocked(route: Resolved, keep?: ReadonlySet<string>): Blocked | undefined {
		const incarnation = this.owned(route);
		if (this.rejected(incarnation.era)) return { kind: "keyRejected" };
		return this.verdict(incarnation, keep)?.blocked;
	}

	/** Balance replies extend `run`. */
	served(route: Resolved, usage: NormalizedUsage | undefined, run?: Set<string>, ticket?: Ticket): boolean {
		const incarnation = this.owned(route);
		const source = usage?.billingSource;
		if (source === undefined) return false;
		const era = this.eraFor(incarnation, ticket);
		if (era === undefined) return true;
		// Settings may change mid-request.
		const { entry } = incarnation;
		this.apply(era, {
			kind: "billed",
			source,
			...(entry.plan === undefined ? {} : { plan: entry.plan }),
			extraUsage: entry.extraUsage === true,
		});
		if (source === "balance") run?.add(entry.id);
		return false;
	}

	failed(route: Resolved, failure: Failure, onward: boolean, ticket?: Ticket): boolean {
		const era = this.eraFor(this.owned(route), ticket);
		if (era === undefined) return true;
		const { entry } = route;
		this.batch(() => {
			switch (failure.category) {
				case "keyRejected":
					this.mark(era, { kind: "rejected" });
					return;
				// One pool per entry.
				case "allowance":
				case "outOfCredit":
					this.apply(era, {
						kind: "refused",
						reason: failure.category === "allowance" ? "allowance" : "credit",
						...(failure.resetsAt === undefined ? {} : { resetsAt: failure.resetsAt }),
						...(failure.retryAfterMs === undefined ? {} : { retryAfterMs: failure.retryAfterMs }),
					});
					return;
				case "network":
					if (this.byUrl(entry)) this.mark(era, { kind: "unreachable" });
					else if (onward) this.gate.cool(era.gates, keyOf(route), failure.retryAfterMs);
					return;
				// Settlement owns overload pauses.
				case "overloaded":
				case "keyMissing":
				case "refused":
				case "badRequest":
				case "replayRejected":
					return;
				default:
					if (onward) this.gate.cool(era.gates, keyOf(route), failure.retryAfterMs);
			}
		});
		return false;
	}

	dispose(): void {
		this.disposed = true;
		for (const off of this.unsubscribe) off();
		for (const { era } of this.d.entries.incarnations()) {
			era.status?.reader.dispose();
			era.status = undefined;
			era.bill = undefined;
			era.pool = CLEAR;
			era.probes.clear();
		}
		this.listeners.clear();
		this.gate.dispose();
	}

	/** Applies registry changes in order. */
	private observe(events: RegistryEvent[]): void {
		this.batch(() => {
			for (const event of events) {
				switch (event.kind) {
					case "retired":
						this.retire(event.incarnation.era);
						break;
					case "rekeyed":
						this.retire(event.retired);
						this.check(event.incarnation.id);
						break;
					case "added":
						this.check(event.incarnation.id);
						break;
					case "extraUsage": {
						const { era } = event.incarnation;
						const lastBill = era.bill?.billing;
						this.apply(era, {
							kind: "extraUsage",
							on: event.on,
							...(lastBill === undefined ? {} : { lastBill }),
						});
						break;
					}
				}
			}
		});
	}

	/** Ends what a retired era held. */
	private retire(era: Era): void {
		const id = era.incarnation.id;
		this.batch(() => {
			const { status } = era;
			if (status !== undefined) {
				status.reader.dispose();
				if (status.reader.busy()) this.retired.push(status.reader);
				era.status = undefined;
				if (status.state.kind !== "none") this.touch(id, "key");
			}
			if (era.bill !== undefined) this.touch(id, "billed");
			if (era.pool.kind !== "clear") this.touch(id, "pool");
			this.gate.retire(era.gates);
		});
	}

	/** Unhealthy entries are held out; asking rechecks an unreachable server. */
	private standing(incarnation: Incarnation): "refused" | "unready" | undefined {
		const { era, entry } = incarnation;
		const status = era.status?.state.kind ?? "none";
		if (status === "rejected") return "refused";
		if (!this.byUrl(entry)) return undefined;
		if (status === "unreachable") this.recheck(era);
		// Out until the recheck passes.
		return status === "unreachable" || (status === "checking" && this.rechecking(era)) ? "unready" : undefined;
	}

	/** A later turn may find it back. */
	private recheck(era: Era): void {
		const now = this.d.scheduler.nowMs();
		if (era.rechecked !== undefined && now - era.rechecked < RECHECK_MS) return;
		era.rechecked = now;
		this.check(era.incarnation.id);
	}

	private rechecking(era: Era): boolean {
		return era.rechecked !== undefined && this.d.scheduler.nowMs() - era.rechecked < KEY_CHECK_MS;
	}

	/** The only pool writer. */
	private apply(era: Era, event: PoolEvent): void {
		const was = era.pool;
		era.pool = step(was, event, this.d.scheduler.nowMs(), this.d.pauseMs ?? PAUSE_MS);
		if (canonicalJson(era.pool) !== canonicalJson(was)) this.touch(era.incarnation.id, "pool");
	}

	private admitted(ticket: Ticket): void {
		const era = this.d.entries.era(ticket.era);
		if (era === undefined) return;
		this.batch(() => {
			era.probes.add(ticketKey(ticket));
			this.apply(era, { kind: "admitted", ticket });
		});
	}

	/** The route's incarnation, which must be of this registry. */
	private owned(route: Pick<Resolved, "incarnation">): Incarnation {
		if (route.incarnation.registry !== this.d.entries) throw new Error("The route belongs to another registry.");
		return route.incarnation;
	}

	/** The ticket's era if it is the incarnation's; with no ticket, the live one. None drops it. */
	private eraFor(incarnation: Incarnation, ticket: Ticket | undefined): Era | undefined {
		if (ticket !== undefined) {
			const era = this.d.entries.era(ticket.era);
			return era?.incarnation === incarnation ? era : undefined;
		}
		return incarnation.live ? incarnation.era : undefined;
	}

	/** Extra usage is read live. */
	private verdict(incarnation: Incarnation, keep: ReadonlySet<string> | undefined): PoolVerdict | undefined {
		const exempt = incarnation.entry.extraUsage === true || keep?.has(incarnation.id) === true;
		return verdict(incarnation.era.pool, exempt, this.d.scheduler.nowMs());
	}

	/** Inference stands until recheck. */
	private mark(era: Era, state: KeyState): void {
		const status = this.statusOf(era);
		if (status.state.kind === state.kind) return;
		status.state = state;
		this.touch(era.incarnation.id, "key");
	}

	private statusOf(era: Era): NonNullable<Era["status"]> {
		if (era.status !== undefined) return era.status;
		const fresh: NonNullable<Era["status"]> = {
			state: none(),
			reader: new KeyReader(
				this.d.scheduler,
				(signal) => this.keyCheck(era, signal),
				(state) => {
					fresh.state = state;
					this.touch(era.incarnation.id, "key");
				},
			),
		};
		era.status = fresh;
		return fresh;
	}

	/** Ends without effect once its era retired. */
	private async keyCheck(era: Era, signal: AbortSignal): Promise<KeyState | undefined> {
		const { entry } = era.incarnation;
		const kind = this.d.kinds[entry.kind];
		if (kind === undefined) return none();
		const stale = () => signal.aborted || era.retired;
		const answer = await this.d.keys.get(entry.id);
		if (stale()) return undefined;
		let state: KeyState;
		if (answer.kind === "refused") state = { kind: "rejected" };
		else if (answer.kind === "unready" && (kind.field === "key" || kind.optionalKey === true))
			state = { kind: "unreachable" };
		else {
			const secret = answer.kind === "key" ? answer.key : undefined;
			state =
				kind.field === "key" && secret === undefined
					? none()
					: await kind.check(entry, secret, this.d.kindDeps(entry.kind), signal);
		}
		return stale() ? undefined : state;
	}

	/** Runs a change whose parts are told together, once it ends. */
	private batch<T>(run: () => T): T {
		this.batching += 1;
		try {
			return run();
		} finally {
			this.batching -= 1;
			if (this.batching === 0) this.flush();
		}
	}

	private touch(id: string, part: Part): void {
		let parts = this.pending.get(id);
		if (parts === undefined) {
			parts = new Set();
			this.pending.set(id, parts);
		}
		parts.add(part);
		if (this.batching === 0) this.flush();
	}

	/** Delivers in order made; a listener's change queues behind. */
	private flush(): void {
		const held = [...this.pending];
		this.pending.clear();
		if (this.disposed) return;
		for (const [entry, parts] of held)
			this.outbox.push({ entry, changed: PARTS.filter((part) => parts.has(part)) });
		if (this.delivering) return;
		this.delivering = true;
		try {
			for (let change = this.outbox.shift(); change !== undefined; change = this.outbox.shift()) {
				for (const listener of [...this.listeners]) {
					try {
						listener(change);
					} catch {
						// A listener cannot change health.
					}
				}
			}
		} finally {
			this.delivering = false;
		}
	}

	/** The current era. */
	private eraOf(id: string): Era | undefined {
		return this.d.entries.incarnation(id)?.era;
	}

	private rejected(era: Era): boolean {
		return era.status?.state.kind === "rejected";
	}

	/** A URL kind that takes no key at all. */
	private keyless(entry: ProviderEntry): boolean {
		const kind = this.d.kinds[entry.kind];
		return kind?.field === "url" && kind.optionalKey !== true;
	}

	private byUrl(entry: Pick<ProviderEntry, "kind">): boolean {
		return this.d.kinds[entry.kind]?.field === "url";
	}
}
