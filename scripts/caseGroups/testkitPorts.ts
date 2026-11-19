// Kinds for the scripted ports and the deadline.

import type { KeyAnswer } from "@nyaaai/contract/entryKeys";
import { z } from "@nyaaai/contract/shape";
import type { ModelRow, StoredRegistry, TestRecord } from "@nyaaai/contract/shapes/model";
import type { ProviderEntry } from "@nyaaai/contract/shapes/provider";
import { type MeteredCall, UnfilledRecord } from "@nyaaai/contract/shapes/usage";
import { type Clock, type Deadline, dayKey, deadline, steppedClock } from "@nyaaai/contract/time";
import { kind } from "@nyaaai/testkit/caseGroup";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import {
	scriptedEntryKeys,
	scriptedModelRegistry,
	scriptedProviderList,
	scriptedUsageSink,
} from "@nyaaai/testkit/scriptedPorts";

////////////////////////////////
//  Functions & Helpers

/** A row of fixed values. */
const rowOf = (key: string): ModelRow => ({
	key,
	bindings: { openrouter: { id: `vendor/${key}` } },
	family: key,
	label: key,
	reasoning: false,
	schema: false,
	cache: "implicit",
	verified: "2026-01-01",
	verifiedPrice: { prompt: 1, completion: 2 },
	identity: { kind: "curated", key },
});

/** A record of fixed values. */
const recordOf = (label: string): TestRecord => ({
	label,
	date: "2026-01-01",
	cost: 0.01,
	checks: { toolPlus: "pass", recovery: "pass", reasoning: "n/a", schema: "n/a" },
	reasoning: false,
	cache: "implicit",
	verifiedPrice: { prompt: 1, completion: 2 },
});

const entryOf = (id: string): ProviderEntry => ({ id, kind: "openrouter", label: id });

/** A call of fixed values. */
const callOf = (model: string, at: number, generationId: string | undefined): MeteredCall => ({
	tag: { feature: "chat" },
	model,
	at,
	ms: 1,
	cost: { usd: 0, estimated: false },
	...(generationId === undefined ? {} : { generationId }),
});

const settle = async () => {
	for (let turn = 0; turn < 4; turn++) await Promise.resolve();
};

////////////////////////////////
//  Kinds

const Heard = z.object({ name: z.string(), id: z.string() });

export const portKinds = {
	/** Listeners are told in subscription order; `get` counts reads. */
	entryKeys: kind(
		z.object({
			steps: z.array(
				z.object({
					op: z.enum(["listen", "unlisten", "set", "get"]),
					name: z.string().exactOptional(),
					id: z.string().exactOptional(),
					answer: z.enum(["key", "none", "refused", "unready"]).exactOptional(),
					key: z.string().exactOptional(),
					/** A listener that, once told, stops the named one. */
					unlistens: z.string().exactOptional(),
				}),
			),
		}),
		z.object({
			steps: z.array(
				z.object({
					heard: z.array(Heard),
					got: z.object({ kind: z.string(), key: z.string().exactOptional() }).exactOptional(),
					reads: z.int(),
				}),
			),
		}),
		async (input) => {
			const keys = scriptedEntryKeys();
			const stops = new Map<string, () => void>();
			let heard: Array<{ name: string; id: string }> = [];
			const steps = [];
			for (const step of input.steps) {
				const name = step.name ?? "";
				const id = step.id ?? "";
				let got: { kind: string; key?: string } | undefined;
				if (step.op === "listen") {
					stops.set(
						name,
						keys.onChange((changed) => {
							heard.push({ name, id: changed });
							if (step.unlistens !== undefined) stops.get(step.unlistens)?.();
						}),
					);
				} else if (step.op === "unlisten") {
					stops.get(name)?.();
				} else if (step.op === "set") {
					const answer: KeyAnswer =
						step.answer === "key" ? { kind: "key", key: step.key ?? "" } : { kind: step.answer ?? "none" };
					keys.set(id, answer);
				} else {
					const answer = await keys.get(id);
					got = answer.kind === "key" ? { kind: "key", key: answer.key } : { kind: answer.kind };
				}
				steps.push({ heard, ...(got === undefined ? {} : { got }), reads: keys.reads() });
				heard = [];
			}
			return { steps };
		},
	),
	providerList: kind(
		z.object({
			steps: z.array(
				z.object({
					op: z.enum(["listen", "unlisten", "set", "read"]),
					name: z.string().exactOptional(),
					ids: z.array(z.string()).exactOptional(),
					unlistens: z.string().exactOptional(),
				}),
			),
		}),
		z.object({
			steps: z.array(
				z.object({
					heard: z.array(z.string()),
					ids: z.array(z.string()).exactOptional(),
					reads: z.int(),
				}),
			),
		}),
		(input) => {
			const list = scriptedProviderList();
			const stops = new Map<string, () => void>();
			let heard: string[] = [];
			const steps = [];
			for (const step of input.steps) {
				const name = step.name ?? "";
				let ids: string[] | undefined;
				if (step.op === "listen")
					stops.set(
						name,
						list.onChange(() => {
							heard.push(name);
							if (step.unlistens !== undefined) stops.get(step.unlistens)?.();
						}),
					);
				else if (step.op === "unlisten") stops.get(name)?.();
				else if (step.op === "set") list.set((step.ids ?? []).map(entryOf));
				else ids = list.entries().map((entry) => entry.id);
				steps.push({ heard, ...(ids === undefined ? {} : { ids }), reads: list.reads() });
				heard = [];
			}
			return { steps };
		},
	),
	/** Rows and hidden keys are set by key; a record is written with a label. */
	modelRegistry: kind(
		z.object({
			steps: z.array(
				z.object({
					op: z.enum(["listen", "unlisten", "setRows", "setHidden", "read", "record", "refuse", "allow"]),
					name: z.string().exactOptional(),
					keys: z.array(z.string()).exactOptional(),
					entry: z.string().exactOptional(),
					id: z.string().exactOptional(),
					label: z.string().exactOptional(),
					why: z.string().exactOptional(),
					unlistens: z.string().exactOptional(),
				}),
			),
		}),
		z.object({
			steps: z.array(
				z.object({
					heard: z.array(z.string()),
					rows: z.array(z.string()).exactOptional(),
					hidden: z.array(z.string()).exactOptional(),
					/** `entry/id=label`, in stored order. */
					tested: z.array(z.string()).exactOptional(),
					/** Null: written. */
					outcome: z.string().nullable().exactOptional(),
					reads: z.int(),
					written: z.int(),
				}),
			),
		}),
		async (input) => {
			const registry = scriptedModelRegistry();
			const stops = new Map<string, () => void>();
			let heard: string[] = [];
			const steps = [];
			for (const step of input.steps) {
				const name = step.name ?? "";
				let seen: { rows: string[]; hidden: string[]; tested: string[] } | undefined;
				let outcome: string | null | undefined;
				if (step.op === "listen")
					stops.set(
						name,
						registry.onChange(() => {
							heard.push(name);
							if (step.unlistens !== undefined) stops.get(step.unlistens)?.();
						}),
					);
				else if (step.op === "unlisten") stops.get(name)?.();
				else if (step.op === "setRows") registry.setRows((step.keys ?? []).map(rowOf));
				else if (step.op === "setHidden") {
					const stored: StoredRegistry = { ...registry.stored(), hidden: step.keys ?? [] };
					registry.setStored(stored);
				} else if (step.op === "refuse") registry.refuse(step.why ?? "");
				else if (step.op === "allow") registry.refuse(null);
				else if (step.op === "record") {
					outcome = await registry.record(step.entry ?? "", step.id ?? "", recordOf(step.label ?? ""));
					await settle();
				} else {
					const stored = registry.stored();
					seen = {
						rows: registry.rows().map((row) => row.key),
						hidden: [...stored.hidden],
						tested: Object.entries(stored.tested).flatMap(([entry, records]) =>
							Object.entries(records).map(([id, record]) => `${entry}/${id}=${record.label}`),
						),
					};
				}
				steps.push({
					heard,
					...(seen ?? {}),
					...(outcome === undefined ? {} : { outcome }),
					reads: registry.reads(),
					written: registry.written().length,
				});
				heard = [];
			}
			return { steps };
		},
	),
	/** Deadlines and callers on one wheel; a caller aborts at its own `abortAt`. */
	deadline: kind(
		z.object({
			startMs: z.int(),
			steps: z.array(
				z.object({
					op: z.enum(["caller", "abortCaller", "abortAt", "deadline", "clear", "advance"]),
					id: z.string().exactOptional(),
					ms: z.int().exactOptional(),
					/** The caller a deadline follows. */
					caller: z.string().exactOptional(),
				}),
			),
		}),
		z.object({
			steps: z.array(
				z.object({
					nowMs: z.int(),
					waiting: z.array(z.string()),
					deadlines: z.array(
						z.object({ id: z.string(), aborted: z.boolean(), ended: z.string().nullable() }),
					),
				}),
			),
		}),
		(input) => {
			const scheduler = fakeScheduler(new TimerWheel(input.startMs));
			const callers = new Map<string, AbortController>();
			const held: Array<{ id: string; deadline: Deadline }> = [];
			const steps = [];
			for (const step of input.steps) {
				const id = step.id ?? "";
				const ms = step.ms ?? 0;
				if (step.op === "caller") callers.set(id, new AbortController());
				else if (step.op === "abortCaller") callers.get(id)?.abort();
				else if (step.op === "abortAt") scheduler.schedule(ms, () => callers.get(id)?.abort());
				else if (step.op === "deadline") {
					const caller = callers.get(step.caller ?? "");
					held.push({ id, deadline: deadline(scheduler, ms, caller?.signal) });
				} else if (step.op === "clear") {
					for (const each of held) if (each.id === id) each.deadline.clear();
				} else scheduler.wheel.advance(ms);
				steps.push({
					nowMs: scheduler.nowMs(),
					waiting: scheduler.wheel.waiting(),
					deadlines: held.map((each) => ({
						id: each.id,
						aborted: each.deadline.signal.aborted,
						ended: each.deadline.ended() ?? null,
					})),
				});
			}
			return { steps };
		},
	),
	/** Ids from a list, none on demand; what was given is kept in order. */
	usageSink: kind(
		z.object({
			steps: z.array(
				z.object({
					op: z.enum(["ids", "none", "record", "edit", "fill", "markFilled", "rows", "filled", "unfilled"]),
					ids: z.array(z.string()).exactOptional(),
					on: z.boolean().exactOptional(),
					model: z.string().exactOptional(),
					generationId: z.string().exactOptional(),
					id: z.string().exactOptional(),
					at: z.int().exactOptional(),
					provider: z.string().exactOptional(),
					rows: z.array(UnfilledRecord).exactOptional(),
				}),
			),
		}),
		z.object({
			steps: z.array(
				z.object({
					/** Null: not kept. */
					answer: z.string().nullable().exactOptional(),
					filled: z.boolean().exactOptional(),
					unfilled: z.array(UnfilledRecord).exactOptional(),
					calls: z.object({ record: z.int(), fill: z.int(), filled: z.int(), unfilled: z.int() }),
					/** `model@at`. */
					recorded: z.array(z.string()),
					/** `id/provider`. */
					fills: z.array(z.string()),
					/** `id@at`. */
					filledAsks: z.array(z.string()),
					unfilledAsks: z.array(z.int()),
				}),
			),
		}),
		async (input) => {
			const sink = scriptedUsageSink();
			/** The call last handed over, which a step may edit. */
			let last: MeteredCall | undefined;
			const steps = [];
			for (const step of input.steps) {
				let answer: string | null | undefined;
				let filled: boolean | undefined;
				let unfilled: UnfilledRecord[] | undefined;
				if (step.op === "ids") sink.setIds(step.ids ?? []);
				else if (step.op === "none") sink.answerNone(step.on ?? false);
				else if (step.op === "record") {
					last = callOf(step.model ?? "", step.at ?? 0, step.generationId);
					answer = sink.record(last) ?? null;
				} else if (step.op === "edit") {
					if (last !== undefined) last.model = step.model ?? "";
				} else if (step.op === "fill") sink.fill(step.id ?? "", { provider: step.provider ?? "" });
				else if (step.op === "markFilled") sink.setFilled(step.ids ?? []);
				else if (step.op === "rows") sink.setUnfilled(step.rows ?? []);
				else if (step.op === "filled") filled = await sink.filled(step.id ?? "", step.at ?? 0);
				else unfilled = await sink.unfilled(step.at ?? 0);
				steps.push({
					...(answer === undefined ? {} : { answer }),
					...(filled === undefined ? {} : { filled }),
					...(unfilled === undefined ? {} : { unfilled }),
					calls: sink.calls(),
					recorded: sink.recorded().map((call) => `${call.model}@${call.at}`),
					fills: sink.fills().map((fill) => `${fill.id}/${fill.stats.provider ?? ""}`),
					filledAsks: sink.filledAsks().map((ask) => `${ask.id}@${ask.at}`),
					unfilledAsks: sink.unfilledAsks(),
				});
			}
			return { steps };
		},
	),
	/** A scripted clock's offsets and the local-day rule they feed. */
	timeOffset: kind(
		z.object({
			clock: z.enum(["stepped", "wheel", "scheduler"]),
			startMs: z.int(),
			/** The test's zone: the latest `fromMs` at or before an instant, a later entry on a tie. */
			offsets: z.array(z.object({ fromMs: z.int(), minutes: z.int() })),
			asks: z.array(z.int()),
		}),
		z.object({
			rows: z.array(
				z.object({
					atMs: z.int(),
					minutes: z.int(),
					/** `dayKey(atMs, minutes)`: the local day. */
					day: z.string().nullable(),
				}),
			),
		}),
		(input) => {
			const wheel = new TimerWheel(input.startMs, input.offsets);
			const clock: Clock =
				input.clock === "stepped"
					? steppedClock(input.startMs, 1, input.offsets)
					: input.clock === "wheel"
						? wheel
						: fakeScheduler(wheel);
			return {
				rows: input.asks.map((atMs) => {
					const minutes = clock.utcOffsetMinutes(atMs);
					return { atMs, minutes, day: dayKey(atMs, minutes) ?? null };
				}),
			};
		},
	),
	/** A scheduler's offsets set at construction and later, while its time moves. */
	timeOffsetRun: kind(
		z.object({
			startMs: z.int(),
			offsets: z.array(z.object({ fromMs: z.int(), minutes: z.int() })),
			steps: z.array(
				z.object({
					op: z.enum(["set", "advance", "ask"]),
					offsets: z.array(z.object({ fromMs: z.int(), minutes: z.int() })).exactOptional(),
					ms: z.int().exactOptional(),
					atMs: z.int().exactOptional(),
				}),
			),
		}),
		z.object({
			steps: z.array(
				z.object({
					/** The scheduler's own time; no offset moves it. */
					nowMs: z.int(),
					minutes: z.int().exactOptional(),
				}),
			),
		}),
		(input) => {
			const scheduler = fakeScheduler(new TimerWheel(input.startMs, input.offsets));
			const steps = [];
			for (const step of input.steps) {
				let minutes: number | undefined;
				if (step.op === "set") scheduler.wheel.setOffsets(step.offsets ?? []);
				else if (step.op === "advance") scheduler.wheel.advance(step.ms ?? 0);
				else minutes = scheduler.utcOffsetMinutes(step.atMs ?? 0);
				steps.push({ nowMs: scheduler.nowMs(), ...(minutes === undefined ? {} : { minutes }) });
			}
			return { steps };
		},
	),
};
