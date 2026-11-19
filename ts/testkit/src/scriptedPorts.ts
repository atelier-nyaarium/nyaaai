// Scripted ports: values set by the test.

import type { EntryKeys, KeyAnswer } from "@nyaaai/contract/entryKeys";
import type { ModelRegistry } from "@nyaaai/contract/modelRegistry";
import type { ProviderList } from "@nyaaai/contract/providerList";
import type { ModelRow, StoredRegistry, TestRecord } from "@nyaaai/contract/shapes/model";
import type { ProviderEntry } from "@nyaaai/contract/shapes/provider";
import type { GenerationStats, MeteredCall, UnfilledRecord } from "@nyaaai/contract/shapes/usage";
import type { UsageSink } from "@nyaaai/contract/usageSink";

////////////////////////////////
//  Interfaces & Types

export interface ScriptedEntryKeys extends EntryKeys {
	/** Tells every listener, in subscription order. */
	set: (id: string, answer: KeyAnswer) => void;
	/** Times `get` was called. */
	reads: () => number;
}

export interface ScriptedProviderList extends ProviderList {
	set: (entries: readonly ProviderEntry[]) => void;
	/** Times `entries` was called. */
	reads: () => number;
}

export interface ScriptedModelRegistry extends ModelRegistry {
	setRows: (rows: readonly ModelRow[]) => void;
	setStored: (stored: StoredRegistry) => void;
	/** Makes `record` answer this text and write nothing; null writes again. */
	refuse: (why: string | null) => void;
	/** Times `rows` or `stored` was called. */
	reads: () => number;
	/** Records written, in order. */
	written: () => Array<{ entry: string; id: string; record: TestRecord }>;
}

export interface ScriptedUsageSink extends UsageSink {
	/** Ids `record` answers, in order; none once they run out. */
	setIds: (ids: readonly string[]) => void;
	/** While true, `record` answers none; it still keeps the call. */
	answerNone: (none: boolean) => void;
	/** Ids that count as filled, plus recorded ids whose call is not before the ask. */
	setFilled: (ids: readonly string[]) => void;
	/** What `unfilled` lists, less rows before its `since` and ids filled likewise. */
	setUnfilled: (rows: readonly UnfilledRecord[]) => void;
	/** Calls `record` was given, in order. */
	recorded: () => MeteredCall[];
	/** Calls `fill` was given, in order. */
	fills: () => Array<{ id: string; stats: GenerationStats }>;
	/** Asks of `filled`, in order. */
	filledAsks: () => Array<{ id: string; at: number }>;
	/** The `since` of each `unfilled`, in order. */
	unfilledAsks: () => number[];
	/** Times each method was called. */
	calls: () => { record: number; fill: number; filled: number; unfilled: number };
}

////////////////////////////////
//  Classes

/** Calls in subscription order; an unsubscribed listener is not called. */
class Listeners<Args extends unknown[]> {
	private held: Array<(...args: Args) => void> = [];

	add(listener: (...args: Args) => void): () => void {
		const own = (...args: Args) => listener(...args);
		this.held.push(own);
		return () => {
			this.held = this.held.filter((each) => each !== own);
		};
	}

	tell(...args: Args): void {
		for (const listener of [...this.held]) if (this.held.includes(listener)) listener(...args);
	}
}

////////////////////////////////
//  Functions & Helpers

export function scriptedEntryKeys(initial: Readonly<Record<string, KeyAnswer>> = {}): ScriptedEntryKeys {
	const answers = new Map(Object.entries(initial));
	const listeners = new Listeners<[string]>();
	let reads = 0;
	return {
		get: async (id) => {
			reads += 1;
			return answers.get(id) ?? { kind: "none" };
		},
		onChange: (listener) => listeners.add(listener),
		set: (id, answer) => {
			answers.set(id, answer);
			listeners.tell(id);
		},
		reads: () => reads,
	};
}

export function scriptedProviderList(initial: readonly ProviderEntry[] = []): ScriptedProviderList {
	let entries = initial;
	const listeners = new Listeners<[]>();
	let reads = 0;
	return {
		entries: () => {
			reads += 1;
			return entries;
		},
		onChange: (listener) => listeners.add(listener),
		set: (next) => {
			entries = next;
			listeners.tell();
		},
		reads: () => reads,
	};
}

export function scriptedModelRegistry(
	initialRows: readonly ModelRow[] = [],
	initialStored: StoredRegistry = { tested: {}, added: {}, hidden: [] },
): ScriptedModelRegistry {
	let rows = initialRows;
	let stored = initialStored;
	let refused: string | null = null;
	let reads = 0;
	const written: Array<{ entry: string; id: string; record: TestRecord }> = [];
	const listeners = new Listeners<[]>();
	return {
		rows: () => {
			reads += 1;
			return rows;
		},
		stored: () => {
			reads += 1;
			return stored;
		},
		record: async (entry, id, record) => {
			if (refused !== null) return refused;
			written.push({ entry, id, record });
			stored = { ...stored, tested: { ...stored.tested, [entry]: { ...stored.tested[entry], [id]: record } } };
			listeners.tell();
			return null;
		},
		onChange: (listener) => listeners.add(listener),
		setRows: (next) => {
			rows = next;
			listeners.tell();
		},
		setStored: (next) => {
			stored = next;
			listeners.tell();
		},
		refuse: (why) => {
			refused = why;
		},
		reads: () => reads,
		written: () => [...written],
	};
}

export function scriptedUsageSink(): ScriptedUsageSink {
	let ids: string[] = [];
	let none = false;
	let rows: UnfilledRecord[] = [];
	/** Ids `fill` or `setFilled` named. */
	const named = new Set<string>();
	/** The call each answered id was recorded with. */
	const kept = new Map<string, MeteredCall>();
	const recorded: MeteredCall[] = [];
	const fills: Array<{ id: string; stats: GenerationStats }> = [];
	const filledAsks: Array<{ id: string; at: number }> = [];
	const unfilledAsks: number[] = [];
	/** A log's answer: a recorded id whose call is not before `at`. */
	const isFilled = (id: string, at: number): boolean => {
		const call = kept.get(id);
		return named.has(id) && call !== undefined && call.at >= at;
	};
	return {
		record: (call) => {
			const copy = structuredClone(call);
			recorded.push(copy);
			const id = none ? undefined : ids.shift();
			if (id !== undefined) kept.set(id, structuredClone(call));
			return id;
		},
		fill: (id, stats) => {
			fills.push({ id, stats: structuredClone(stats) });
			named.add(id);
		},
		filled: async (id, at) => {
			filledAsks.push({ id, at });
			return isFilled(id, at);
		},
		unfilled: async (since) => {
			unfilledAsks.push(since);
			return rows.filter((row) => row.at >= since && !isFilled(row.id, since)).map((row) => ({ ...row }));
		},
		setIds: (next) => {
			ids = [...next];
		},
		answerNone: (next) => {
			none = next;
		},
		setFilled: (next) => {
			for (const id of next) named.add(id);
		},
		setUnfilled: (next) => {
			rows = next.map((row) => ({ ...row }));
		},
		recorded: () => recorded.map((call) => structuredClone(call)),
		fills: () => fills.map((fill) => structuredClone(fill)),
		filledAsks: () => [...filledAsks],
		unfilledAsks: () => [...unfilledAsks],
		calls: () => ({
			record: recorded.length,
			fill: fills.length,
			filled: filledAsks.length,
			unfilled: unfilledAsks.length,
		}),
	};
}
