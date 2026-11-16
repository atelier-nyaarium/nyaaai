// Scripted ports: values set by the test.

import type { EntryKeys, KeyAnswer } from "@nyaaai/contract/entryKeys";
import type { ModelRegistry } from "@nyaaai/contract/modelRegistry";
import type { ProviderList } from "@nyaaai/contract/providerList";
import type { ModelRow, StoredRegistry, TestRecord } from "@nyaaai/contract/shapes/model";
import type { ProviderEntry } from "@nyaaai/contract/shapes/provider";

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
