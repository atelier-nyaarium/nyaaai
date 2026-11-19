import { failed } from "@nyaaai/agentic/failure";
import { type Gate, ProviderHealth } from "@nyaaai/agentic/health/health";
import {
	type Attempt,
	type Call,
	meteredOf,
	type Observer,
	RequestLifecycle,
	type RunRoute,
} from "@nyaaai/agentic/lifecycle/lifecycle";
import { RetryPolicy } from "@nyaaai/agentic/lifecycle/retry";
import { parseRowKey } from "@nyaaai/agentic/models/rowKey";
import { CHAT } from "@nyaaai/agentic/models/uses";
import { type EntryCatalog, KINDS, type ProviderKind } from "@nyaaai/agentic/provider/kind";
import { ModelResolver, type Resolved, type Routes } from "@nyaaai/agentic/provider/resolver";
import { replyCalls, replyText, StepAssembler } from "@nyaaai/agentic/step";
import { providerMeter } from "@nyaaai/agentic/usage/meter";
import type { KeyAnswer } from "@nyaaai/contract/entryKeys";
import { countedIds } from "@nyaaai/contract/id";
import { sequenceRandom } from "@nyaaai/contract/random";
import { z } from "@nyaaai/contract/shape";
import { EntryHealth, GateWaiting, Notice, RetryOptions } from "@nyaaai/contract/shapes/health";
import { TraceEntry } from "@nyaaai/contract/shapes/lifecycle";
import { type ModelRow, ModelUse, type ModelView, Price, Unresolved } from "@nyaaai/contract/shapes/model";
import { Passed, ProviderEntry, ProviderKindName, SkipReason } from "@nyaaai/contract/shapes/provider";
import { type CanonicalRequest, Failure, HeaderList, StepEvent, StepResult } from "@nyaaai/contract/shapes/step";
import { CallTag, MeteredCall } from "@nyaaai/contract/shapes/usage";
import { trimText } from "@nyaaai/contract/text";
import { deadline } from "@nyaaai/contract/time";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { type ScriptedExchange, scriptedFetch } from "@nyaaai/testkit/scriptedFetch";
import {
	scriptedEntryKeys,
	scriptedModelRegistry,
	scriptedProviderList,
	scriptedUsageSink,
} from "@nyaaai/testkit/scriptedPorts";
import { lifecycleCases } from "../caseSources/lifecycle";
import { rowOfKey } from "./catalogKit";
import { localRowOf } from "./localKit";

////////////////////////////////
//  Shapes

/** What `EntryKeys.get` answers. */
const LifeKeyAnswer = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("key"), key: z.string() }),
	z.object({ kind: z.literal("none") }),
	z.object({ kind: z.literal("refused") }),
	z.object({ kind: z.literal("unready") }),
]);

/** One request's reply: head, body chunks, ending. */
const LifeReply = z.object({
	status: z.int(),
	headers: HeaderList,
	chunks: z.array(z.string()),
	/** Wheel time spent before the head. */
	headMs: z.int().min(0).exactOptional(),
	/** Wheel time spent before each chunk. */
	chunkMs: z.int().min(0).exactOptional(),
	/** `closed` by default. */
	end: z.enum(["closed", "dropped", "refused"]).exactOptional(),
	error: z.string().exactOptional(),
});

/** What a catalog lists of a row; an unnamed row is `ok`. */
const LifeListing = z.object({
	/** The catalog has no view of the row. */
	unlisted: z.literal(true).exactOptional(),
	/** Only data-keeping hosts serve it: `noZdrHost` with the switch on, `dataKept` off. */
	keeps: z.literal(true).exactOptional(),
	price: z.union([Price, z.literal("free"), z.literal("unknown")]).exactOptional(),
	cacheWrite1h: z.literal(true).exactOptional(),
});

/** A request, named so a later step can abort it. */
const LifeSend = z.object({
	kind: z.literal("send"),
	call: z.string(),
	/** The model key, resolved for the use when no plan is named. */
	model: z.string(),
	use: ModelUse.exactOptional(),
	/** A plan an earlier step resolved. */
	plan: z.string().exactOptional(),
	/** A run an earlier step began; the call goes through its `RunRoute`. */
	run: z.string().exactOptional(),
	mode: z.enum(["stream", "complete"]).exactOptional(),
	live: z.boolean().exactOptional(),
	tag: CallTag,
	/** A deadline of the call's own, from the call's start. */
	deadlineMs: z.int().min(0).exactOptional(),
	/** The send throws this as a plain error before the network. */
	throws: z.string().exactOptional(),
	/** The caller's abort, after this much wheel time. */
	abortAfterMs: z.int().min(0).exactOptional(),
	abortedBefore: z.boolean().exactOptional(),
});

/** A change of the entries or their keys. */
const LifeChange = z.discriminatedUnion("kind", [
	/** The list changes, and tells. */
	z.object({ kind: z.literal("list"), entries: z.array(ProviderEntry) }),
	/** The store's key changes, and tells. */
	z.object({ kind: z.literal("key"), id: z.string(), answer: LifeKeyAnswer }),
]);

const LifeStep = z.discriminatedUnion("kind", [
	LifeSend,
	/** A change made `ms` on, so it can land inside a request. */
	z.object({ kind: z.literal("later"), ms: z.int().min(0), change: LifeChange }),
	/** Resolves a model and holds the routes under a name. */
	z.object({ kind: z.literal("plan"), name: z.string(), model: z.string(), use: ModelUse.exactOptional() }),
	/** Begins a `RunRoute`. */
	z.object({ kind: z.literal("begin"), run: z.string() }),
	/** `RunRoute.sizing` over a plan's candidates. */
	z.object({ kind: z.literal("sizing"), run: z.string(), plan: z.string() }),
	z.object({ kind: z.literal("advance"), ms: z.int().min(0) }),
	/** The caller aborts a call. */
	z.object({ kind: z.literal("abort"), call: z.string() }),
	/** The list changes, and tells. */
	z.object({ kind: z.literal("list"), entries: z.array(ProviderEntry) }),
	/** The store's key changes, and tells. */
	z.object({ kind: z.literal("key"), id: z.string(), answer: LifeKeyAnswer }),
	/** The zero-retention switch. */
	z.object({ kind: z.literal("zdr"), on: z.boolean() }),
	/** A key check of an entry. */
	z.object({ kind: z.literal("check"), id: z.string() }),
	/** A reply billed to an entry outside any request. */
	z.object({ kind: z.literal("billed"), id: z.string(), source: z.string(), cost: z.number().exactOptional() }),
]);

const LifeAnswer = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("none") }),
	z.object({
		kind: z.literal("plan"),
		unresolved: Unresolved.nullable(),
		/** Entry ids in priority order. */
		candidates: z.array(z.string()),
		passed: z.array(Passed),
	}),
	/** The entry ids the run keeps. */
	z.object({ kind: z.literal("sizing"), entries: z.array(z.string()) }),
]);

/** How a call ended. */
const LifeResult = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("served"), entry: z.string(), reply: StepResult }),
	z.object({ kind: z.literal("stopped") }),
	z.object({ kind: z.literal("notice"), notice: Notice, failure: Failure.nullable() }),
	z.object({ kind: z.literal("thrown"), message: z.string() }),
	/** Still parked when the script ended. */
	z.object({ kind: z.literal("open") }),
]);

/** What an `Observer` callback was told. */
const LifeTold = z.discriminatedUnion("kind", [
	z.object({
		kind: z.literal("retry"),
		attempt: z.int(),
		waitMs: z.int(),
		status: z.union([z.number(), z.string()]),
	}),
	z.object({ kind: z.literal("wait"), waiting: GateWaiting.nullable() }),
	z.object({ kind: z.literal("skip"), entry: z.string(), label: z.string(), reason: SkipReason }),
]);

const LifeCall = z.object({
	call: z.string(),
	result: LifeResult,
	/** Every entry told, in order; null when no sink was given. */
	trace: z.array(TraceEntry).nullable(),
	/** What the callbacks were told, in order. */
	told: z.array(LifeTold),
	/** The events each send's stream yielded, by attempt. */
	attempts: z.array(z.object({ entry: z.string(), events: z.array(StepEvent) })),
});

const LifeSeen = z.object({
	url: z.string(),
	authorization: z.string().nullable(),
	accept: z.string().nullable(),
});

////////////////////////////////
//  Constants

const CALLER_REASON = "caller aborted";

const REQUEST: CanonicalRequest = {
	model: "m",
	system: [],
	history: [],
	cache: { kind: "none" },
	session: "session-1",
	zdrOnly: true,
};

const UTF8 = new TextEncoder();

/** Key checks always pass. */
const checked = (named: ProviderKind): ProviderKind => ({ ...named, check: async () => ({ kind: "ok" }) });

////////////////////////////////
//  Functions & Helpers

function rowFor(key: string): ModelRow {
	return parseRowKey(key)?.kind === "local" ? localRowOf(key) : rowOfKey(key);
}

function exchangeOf(held: z.infer<typeof LifeReply>): ScriptedExchange {
	return {
		status: held.status,
		headers: held.headers,
		chunks: held.chunks.map((text) => ({ bytes: UTF8.encode(text), delayMs: held.chunkMs ?? 0 })),
		...(held.headMs === undefined ? {} : { headMs: held.headMs }),
		...(held.end === undefined ? {} : { end: held.end }),
		...(held.error === undefined ? {} : { error: held.error }),
	};
}

const messageOf = (error: unknown): string =>
	error instanceof Error ? error.message : "The call threw a value that is no error.";

/** One call the script made. */
interface Slot {
	name: string;
	controller: AbortController;
	signal: AbortSignal;
	/** `held` waits in a gate's queue; `sleeping` waits on the scheduler. */
	state: "running" | "sleeping" | "held" | "settled";
	/** The gate model key while held. */
	model: string | undefined;
	route: Resolved | undefined;
	trace: Array<z.infer<typeof TraceEntry>>;
	told: Array<z.infer<typeof LifeTold>>;
	attempts: Array<{ entry: string; events: Array<z.infer<typeof StepEvent>> }>;
	result: z.infer<typeof LifeResult>;
	promise: Promise<void>;
}

////////////////////////////////
//  Group

export const lifecycle = caseGroup("lifecycle", {
	shapes: {
		LifeKeyAnswer,
		LifeReply,
		LifeListing,
		LifeChange,
		LifeStep,
		LifeAnswer,
		LifeResult,
		LifeTold,
		LifeCall,
		LifeSeen,
	},
	kinds: {
		/** A request lifecycle over real health and gate; each step runs calls until ended or parked. */
		run: kind(
			z.object({
				startMs: z.int(),
				pauseMs: z.int().min(0).exactOptional(),
				maxPauses: z.int().min(0).exactOptional(),
				retry: RetryOptions.exactOptional(),
				/** The draws of the retry policy, cycled; one 1 when absent. */
				draws: z.array(z.number()).exactOptional(),
				entries: z.array(ProviderEntry),
				/** The store's answers by entry id; an entry with none answers none. */
				keys: z.record(z.string(), LifeKeyAnswer),
				/** The registry's rows, by key. */
				rows: z.array(z.string()),
				kinds: z.array(ProviderKindName),
				/** What each entry's catalog lists, by entry id, then row key. */
				listings: z.record(z.string(), z.record(z.string(), LifeListing)),
				zdrOnly: z.boolean(),
				/** Usage sink ids in order; none after they run out. */
				usageIds: z.array(z.string()).exactOptional(),
				/** What the network answers, in the order requests reach it. */
				exchanges: z.array(LifeReply),
				/** False gives the lifecycle no trace sink. */
				trace: z.boolean().exactOptional(),
				steps: z.array(LifeStep).min(1),
			}),
			z.object({
				steps: z.array(
					z.object({
						/** The wheel's time when the step ended. */
						nowMs: z.int(),
						answer: LifeAnswer,
						/** Calls that ended during the step. */
						ended: z.array(z.string()),
						/** Calls still parked when it ended. */
						parked: z.array(z.string()),
						/** Every entry the script has named, in the order first named. */
						entries: z.array(EntryHealth),
						/** Requests the network has seen so far. */
						requests: z.int(),
					}),
				),
				calls: z.array(LifeCall),
				requests: z.array(LifeSeen),
				/** What the meter gave the sink, in order. */
				recorded: z.array(MeteredCall),
				/** The stats hook's calls. */
				stats: z.array(z.object({ id: z.string(), generationId: z.string(), call: MeteredCall })),
				/** Timers still waiting at the end. */
				waiting: z.array(z.string()),
			}),
			async (input) => {
				const slots = new Map<string, Slot>();
				const bySignal = new Map<AbortSignal, Slot>();
				let wake: Array<() => void> = [];
				const changed = () => new Promise<void>((resolve) => wake.push(resolve));
				const notify = () => {
					const waiting = wake;
					wake = [];
					for (const resolve of waiting) resolve();
				};

				const wheel = new TimerWheel(input.startMs);
				const scheduler = fakeScheduler(wheel, {
					slept: (signal) => {
						const slot = signal === undefined ? undefined : bySignal.get(signal);
						if (slot === undefined) return;
						slot.state = "sleeping";
						notify();
					},
					woke: (signal) => {
						const slot = signal === undefined ? undefined : bySignal.get(signal);
						if (slot?.state === "sleeping") slot.state = "running";
					},
				});
				const played = scriptedFetch(input.exchanges.map(exchangeOf), wheel);

				const scriptedKeys = scriptedEntryKeys(input.keys as Record<string, KeyAnswer>);
				const list = scriptedProviderList(input.entries);
				const registry = scriptedModelRegistry(input.rows.map(rowFor));
				const usage = scriptedUsageSink();
				usage.setIds(input.usageIds ?? []);
				let zdr = input.zdrOnly;

				const deps = { fetch: played.fetch, scheduler, log: () => {}, userAgent: "cases/1.0" };
				const checks: Partial<Record<z.infer<typeof ProviderKindName>, ProviderKind>> = {};
				const stub = (name: z.infer<typeof ProviderKindName>): ProviderKind => {
					const real = KINDS[name];
					if (real === undefined) throw new Error(`No ${name} kind.`);
					return {
						...real,
						catalog: (entry): EntryCatalog => ({
							views: async (_use, zdrOnly) =>
								registry.rows().flatMap((row): Array<z.infer<typeof ModelView>> => {
									const bound = row.bindings[name] !== undefined;
									const elsewhere = row.identity.kind === "local" && row.identity.entry !== entry.id;
									if (!bound || elsewhere) return [];
									const held = input.listings[entry.id]?.[row.key];
									if (held?.unlisted === true) return [];
									const kept =
										held?.keeps === true
											? zdrOnly
												? { status: "noZdrHost" as const }
												: { dataKept: true as const }
											: {};
									return [
										{
											row,
											status: "ok",
											efforts: [],
											hosts: [],
											...(held?.price === undefined ? {} : { price: held.price }),
											...(held?.cacheWrite1h === undefined
												? {}
												: { cacheWrite1h: true as const }),
											...kept,
										},
									];
								}),
							idle: async () => {},
						}),
					};
				};
				const kinds: Partial<Record<z.infer<typeof ProviderKindName>, ProviderKind>> = {};
				for (const name of input.kinds) {
					kinds[name] = stub(name);
					checks[name] = checked(KINDS[name] ?? stub(name));
				}

				const health = new ProviderHealth({
					scheduler,
					ids: countedIds("gate-"),
					list,
					keys: scriptedKeys,
					kinds: checks,
					kindDeps: () => deps,
					...(input.pauseMs === undefined ? {} : { pauseMs: input.pauseMs }),
					...(input.maxPauses === undefined ? {} : { maxPauses: input.maxPauses }),
				});
				const resolver = new ModelResolver({
					list,
					keys: health.entryKeys,
					registry,
					kinds,
					kindDeps: () => deps,
					zdrOnly: () => zdr,
				});

				// Queues front first, to see what a change released.
				const queues = new Map<string, { route: Resolved; slots: Slot[] }>();
				const unqueue = (slot: Slot) => {
					for (const held of queues.values()) held.slots = held.slots.filter((each) => each !== slot);
				};
				const gate: Gate = {
					admit: (route, signal, onWait, again) => {
						const slot = bySignal.get(signal);
						if (slot === undefined) return health.admit(route, signal, onWait, again);
						unqueue(slot);
						const model = `${route.entry.id}\n${route.key}`;
						let joined = false;
						const admission = health.admit(
							route,
							signal,
							(waiting) => {
								if (!joined) {
									joined = true;
									slot.state = "held";
									slot.model = model;
									const held = queues.get(model) ?? { route, slots: [] };
									queues.set(model, held);
									if (again === true) held.slots.unshift(slot);
									else held.slots.push(slot);
								}
								onWait?.(waiting);
							},
							again,
						);
						if (joined) notify();
						return admission;
					},
					tryAdmit: (route) => health.tryAdmit(route),
					settle: (ticket, outcome, retryAfterMs) => health.settle(ticket, outcome, retryAfterMs),
					deferred: (route, keep) => health.deferred(route, keep),
					blocked: (route, keep) => health.blocked(route, keep),
					failed: (route, failure, onward, ticket) => health.failed(route, failure, onward, ticket),
					served: (route, billed, run, ticket) => health.served(route, billed, run, ticket),
				};

				const stats: Array<{ id: string; generationId: string; call: z.infer<typeof MeteredCall> }> = [];
				const meter = providerMeter({
					usage,
					health,
					stats: (id, generationId, call) => stats.push({ id, generationId, call: structuredClone(call) }),
				});
				const lifecycle = new RequestLifecycle({
					gate,
					meter,
					retry: new RetryPolicy(input.retry ?? {}, sequenceRandom(input.draws ?? [1])),
					scheduler,
				});

				const plans = new Map<string, Routes>();
				const runs = new Map<string, RunRoute>();
				const named: string[] = [];
				const name = (id: string) => {
					if (!named.includes(id)) named.push(id);
				};
				for (const entry of input.entries) name(entry.id);

				/** Calls the gate or the wheel let go of can run again. */
				const refresh = () => {
					for (const [model, held] of queues) {
						for (const slot of held.slots.filter((each) => each.signal.aborted)) {
							slot.state = "running";
						}
						held.slots = held.slots.filter((each) => !each.signal.aborted);
						const queued = health.gateState(held.route)?.queued ?? 0;
						const released = held.slots.length - queued;
						for (const slot of held.slots.splice(0, Math.max(0, released))) slot.state = "running";
						if (held.slots.length === 0) queues.delete(model);
					}
				};

				const running = () => [...slots.values()].some((slot) => slot.state === "running");

				/** Runs until every call has ended or is parked. */
				const quiesce = async () => {
					for (;;) {
						refresh();
						if (!running()) return;
						await changed();
					}
				};

				const plan = async (model: string, use: z.infer<typeof ModelUse> | undefined): Promise<Routes> => {
					const found = await resolver.routes(model, use ?? CHAT);
					if (!found.ok) throw new Error(`No routes for ${model}.`);
					return found;
				};

				const start = async (step: z.infer<typeof LifeSend>) => {
					const routes = step.plan === undefined ? await plan(step.model, step.use) : plans.get(step.plan);
					if (routes === undefined) throw new Error(`No plan ${step.plan}.`);
					const runRoute = step.run === undefined ? undefined : runs.get(step.run);
					if (step.run !== undefined && runRoute === undefined) throw new Error(`No run ${step.run}.`);
					const controller = new AbortController();
					if (step.abortedBefore === true) controller.abort(new Error(CALLER_REASON));
					else if (step.abortAfterMs !== undefined)
						wheel.after(step.abortAfterMs, `abort-${step.call}`, () =>
							controller.abort(new Error(CALLER_REASON)),
						);
					const limit =
						step.deadlineMs === undefined
							? undefined
							: deadline(scheduler, step.deadlineMs, controller.signal);
					const signal = limit === undefined ? controller.signal : limit.signal;
					const slot: Slot = {
						name: step.call,
						controller,
						signal,
						state: "running",
						model: undefined,
						route: undefined,
						trace: [],
						told: [],
						attempts: [],
						result: { kind: "open" },
						promise: Promise.resolve(),
					};
					slots.set(step.call, slot);
					bySignal.set(signal, slot);
					let assembler = new StepAssembler();
					const mode = step.mode ?? "stream";
					const call: Call<StepResult> = {
						tag: step.tag,
						signal,
						...(limit === undefined ? {} : { ended: limit.ended }),
						...(step.live === undefined ? {} : { live: step.live }),
						send: async (route, firstToken) => {
							assembler = new StepAssembler();
							const events: Array<z.infer<typeof StepEvent>> = [];
							slot.attempts.push({ entry: route.entry.id, events });
							name(route.entry.id);
							if (step.throws !== undefined) throw new Error(step.throws);
							const request: CanonicalRequest = {
								...REQUEST,
								model: route.binding.id,
								zdrOnly: route.zdrOnly,
							};
							if (mode === "complete") return route.transport.complete(route.codec, request, signal);
							for await (const event of route.transport.stream(route.codec, request, signal)) {
								events.push(event);
								assembler.add(event);
								if (assembler.outputSeen) firstToken();
							}
							const result = assembler.result();
							if (result.finish === "error") throw failed("stream", "The provider ended the step.");
							const cut = result.finish === "length" || result.finish === "content_filter";
							if (!cut && trimText(replyText(result)).length === 0 && replyCalls(result).length === 0)
								throw failed("empty", "The reply was empty.");
							return result;
						},
						read: meteredOf,
						billed: () => assembler.billed,
					};
					const observer: Observer = {
						fresh: () => !assembler.outputSeen,
						onRetry: (attempt, waitMs, status) =>
							slot.told.push({ kind: "retry", attempt, waitMs, status }),
						onWait: (waiting) => slot.told.push({ kind: "wait", waiting }),
						onSkip: (skip) => slot.told.push({ kind: "skip", ...skip }),
						...(input.trace === false ? {} : { onTrace: (entry) => slot.trace.push(entry) }),
					};
					slot.promise = (async () => {
						try {
							const attempt: Attempt<StepResult> =
								runRoute === undefined
									? await lifecycle.run(routes.candidates, call, observer)
									: await runRoute.run(routes, call, observer);
							slot.result = attempt.ok
								? { kind: "served", entry: attempt.route.entry.id, reply: attempt.value }
								: attempt.outcome.kind === "stopped"
									? { kind: "stopped" }
									: {
											kind: "notice",
											notice: attempt.outcome.notice,
											failure: attempt.failure ?? null,
										};
						} catch (error) {
							slot.result = { kind: "thrown", message: messageOf(error) };
						} finally {
							limit?.clear();
							wheel.cancel(`abort-${step.call}`);
							slot.state = "settled";
							notify();
						}
					})();
				};

				let later = 0;
				const relist = (entries: Array<z.infer<typeof ProviderEntry>>) => {
					for (const entry of entries) name(entry.id);
					list.set(entries);
				};
				const rekey = (id: string, answer: z.infer<typeof LifeKeyAnswer>) => {
					name(id);
					scriptedKeys.set(id, answer);
				};

				const stepOuts: Array<{
					nowMs: number;
					answer: z.infer<typeof LifeAnswer>;
					ended: string[];
					parked: string[];
					entries: Array<z.infer<typeof EntryHealth>>;
					requests: number;
				}> = [];
				for (const step of input.steps) {
					const before = new Set(
						[...slots.values()].filter((slot) => slot.state === "settled").map((slot) => slot.name),
					);
					let answer: z.infer<typeof LifeAnswer> = { kind: "none" };
					if (step.kind === "send") {
						await start(step);
					} else if (step.kind === "plan") {
						const found = await resolver.routes(step.model, step.use ?? CHAT);
						if (found.ok) {
							plans.set(step.name, found);
							answer = {
								kind: "plan",
								unresolved: null,
								candidates: found.candidates.map((each) => each.entry.id),
								passed: [...found.passed],
							};
						} else answer = { kind: "plan", unresolved: found, candidates: [], passed: [] };
					} else if (step.kind === "begin") {
						runs.set(step.run, lifecycle.begin());
					} else if (step.kind === "sizing") {
						const routes = plans.get(step.plan);
						const run = runs.get(step.run);
						if (routes === undefined || run === undefined) throw new Error("No plan or run to size.");
						answer = {
							kind: "sizing",
							entries: run.sizing(routes.candidates).map((each) => each.entry.id),
						};
					} else if (step.kind === "advance") {
						wheel.advance(step.ms);
					} else if (step.kind === "abort") {
						slots.get(step.call)?.controller.abort(new Error(CALLER_REASON));
					} else if (step.kind === "later") {
						later += 1;
						const change = step.change;
						wheel.after(step.ms, `later-${later}`, () =>
							change.kind === "list" ? relist(change.entries) : rekey(change.id, change.answer),
						);
					} else if (step.kind === "list") {
						relist(step.entries);
					} else if (step.kind === "key") {
						rekey(step.id, step.answer);
					} else if (step.kind === "zdr") {
						zdr = step.on;
					} else if (step.kind === "check") {
						name(step.id);
						health.check(step.id);
					} else {
						name(step.id);
						health.billed(
							step.id,
							step.cost === undefined
								? { source: step.source }
								: { source: step.source, cost: step.cost },
						);
					}
					await health.idle();
					await quiesce();
					if (played.waited() > 0)
						throw new Error(`step ${stepOuts.length + 1} gave two calls the network at once`);
					stepOuts.push({
						nowMs: wheel.nowMs(),
						answer,
						ended: [...slots.values()]
							.filter((slot) => slot.state === "settled" && !before.has(slot.name))
							.map((slot) => slot.name),
						parked: [...slots.values()].filter((slot) => slot.state !== "settled").map((slot) => slot.name),
						entries: named.map((id) => health.state(id)),
						requests: played.requests.length,
					});
				}

				const left = [...slots.values()].filter((slot) => slot.state !== "settled").map((slot) => slot.name);
				const calls = [...slots.values()].map((slot) => ({
					call: slot.name,
					result: left.includes(slot.name) ? ({ kind: "open" } as const) : slot.result,
					trace: input.trace === false ? null : structuredClone(slot.trace),
					told: structuredClone(slot.told),
					attempts: structuredClone(slot.attempts),
				}));
				// Parked calls end by the caller's abort.
				for (const slot of slots.values())
					if (slot.state !== "settled") slot.controller.abort(new Error(CALLER_REASON));
				await Promise.all([...slots.values()].map((slot) => slot.promise));
				await health.idle();
				const seen = played.requests.map((sent) => ({
					url: sent.url,
					authorization: sent.headers.find(([header]) => header === "Authorization")?.[1] ?? null,
					accept: sent.headers.find(([header]) => header === "Accept")?.[1] ?? null,
				}));
				resolver.dispose();
				health.dispose();
				const waiting = wheel.waiting();
				return {
					steps: stepOuts,
					calls,
					requests: seen,
					recorded: usage.recorded(),
					stats,
					waiting,
				};
			},
		),
	},
	sources: casesFor("lifecycle", lifecycleCases),
});
