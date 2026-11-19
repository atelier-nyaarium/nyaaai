// Kinds for the scripted ports and the deadline.

import type { KeyAnswer } from "@nyaaai/contract/entryKeys";
import { z } from "@nyaaai/contract/shape";
import type { ModelRow, StoredRegistry, TestRecord } from "@nyaaai/contract/shapes/model";
import type { ProviderEntry } from "@nyaaai/contract/shapes/provider";
import { type Deadline, deadline } from "@nyaaai/contract/time";
import { kind } from "@nyaaai/testkit/caseGroup";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedEntryKeys, scriptedModelRegistry, scriptedProviderList } from "@nyaaai/testkit/scriptedPorts";

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
};
