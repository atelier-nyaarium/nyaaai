// One incarnation per listed entry id, and the key eras under it.

import type { EntryKeys } from "@nyaaai/contract/entryKeys";
import { canonicalJson } from "@nyaaai/contract/json";
import type { ProviderList } from "@nyaaai/contract/providerList";
import type { PoolState } from "@nyaaai/contract/shapes/health";
import type { Billed, KeyState, ProviderEntry } from "@nyaaai/contract/shapes/provider";
import { GateBook } from "../health/gate";
import type { KeyReader } from "../health/keyReader";
import { CLEAR } from "../health/pool";

////////////////////////////////
//  Interfaces & Types

/** A key state and its reader. */
export interface KeyStatus {
	state: KeyState;
	reader: KeyReader;
}

/** A change, told once the registry holds it. */
export type RegistryEvent =
	| { kind: "added"; incarnation: Incarnation }
	| { kind: "retired"; incarnation: Incarnation }
	| { kind: "extraUsage"; incarnation: Incarnation; on: boolean }
	| { kind: "rekeyed"; incarnation: Incarnation; retired: Era };

export interface RegistryDeps {
	/** Priority order; the first row of an id wins. */
	list: ProviderList;
	/** Tells the id of a key change. */
	keys: Pick<EntryKeys, "onChange">;
}

////////////////////////////////
//  Functions & Helpers

/** What an entry is: its kind, plan and URL. */
export function identityOf(entry: ProviderEntry): string {
	return canonicalJson([entry.kind, entry.plan ?? null, entry.url ?? null]);
}

////////////////////////////////
//  Classes

/** One key generation. Health owns its fields. */
export class Era {
	/** Set by the registry. */
	retired = false;
	status: KeyStatus | undefined;
	pool: PoolState = CLEAR;
	bill: Billed | undefined;
	rechecked: number | undefined;
	/** Admitted tickets, by data. */
	readonly probes = new Set<string>();
	readonly gates: GateBook;

	constructor(
		readonly number: number,
		readonly incarnation: Incarnation,
	) {
		this.gates = new GateBook(number, incarnation.id);
	}
}

/** One identity under one id; retired for good. */
export class Incarnation {
	/** Cleared by the registry. */
	live = true;
	/** Replaced by the registry on a list change. */
	entry: ProviderEntry;
	/** Replaced by the registry on a key change. */
	era: Era;

	constructor(
		readonly number: number,
		readonly id: string,
		readonly identity: string,
		entry: ProviderEntry,
		firstEra: number,
		/** The registry that made it. */
		readonly registry: EntryRegistry,
	) {
		this.entry = entry;
		this.era = new Era(firstEra, this);
	}
}

/** Sole subscriber of the list and key changes. */
export class EntryRegistry {
	/** Live, in list order. */
	private order: Incarnation[] = [];
	private readonly eras = new Map<number, Era>();
	private readonly watchers: Array<(events: RegistryEvent[]) => void> = [];
	private readonly keyListeners: Array<(id: string) => void> = [];
	private readonly unsubscribe: Array<() => void>;
	/** Changes to tell, in the order made. */
	private readonly outbox: Array<() => void> = [];
	private delivering = false;
	private incarnationCount = 0;
	private eraCount = 0;
	private disposed = false;

	constructor(private readonly d: RegistryDeps) {
		for (const entry of this.rows().values()) this.order.push(this.make(entry));
		this.unsubscribe = [d.keys.onChange((id) => this.keyChanged(id)), d.list.onChange(() => this.listChanged())];
	}

	/** Live, in list order. */
	incarnations(): Incarnation[] {
		return [...this.order];
	}

	incarnation(id: string): Incarnation | undefined {
		return this.order.find((incarnation) => incarnation.id === id);
	}

	/** A live era. */
	era(number: number): Era | undefined {
		return this.eras.get(number);
	}

	/** Told each change; returns unsubscribe. */
	watch(listener: (events: RegistryEvent[]) => void): () => void {
		this.watchers.push(listener);
		return () => {
			const at = this.watchers.indexOf(listener);
			if (at >= 0) this.watchers.splice(at, 1);
		};
	}

	/** Told each key change; returns unsubscribe. */
	onKeyChange(listener: (id: string) => void): () => void {
		this.keyListeners.push(listener);
		return () => {
			const at = this.keyListeners.indexOf(listener);
			if (at >= 0) this.keyListeners.splice(at, 1);
		};
	}

	/** Retires every incarnation, tells the watchers, then stops listening. */
	dispose(): void {
		if (this.disposed) return;
		this.disposed = true;
		for (const off of this.unsubscribe) off();
		const events: RegistryEvent[] = [];
		for (const incarnation of this.order) {
			this.retire(incarnation);
			events.push({ kind: "retired", incarnation });
		}
		this.order = [];
		this.tell(events);
		this.post(() => {
			this.watchers.length = 0;
			this.keyListeners.length = 0;
		});
	}

	private rows(): Map<string, ProviderEntry> {
		const held = new Map<string, ProviderEntry>();
		for (const entry of this.d.list.entries()) if (!held.has(entry.id)) held.set(entry.id, entry);
		return held;
	}

	private make(entry: ProviderEntry): Incarnation {
		this.incarnationCount += 1;
		this.eraCount += 1;
		const incarnation = new Incarnation(
			this.incarnationCount,
			entry.id,
			identityOf(entry),
			entry,
			this.eraCount,
			this,
		);
		this.eras.set(incarnation.era.number, incarnation.era);
		return incarnation;
	}

	private retire(incarnation: Incarnation): void {
		incarnation.live = false;
		incarnation.era.retired = true;
		this.eras.delete(incarnation.era.number);
	}

	private tell(events: RegistryEvent[]): void {
		if (events.length === 0) return;
		this.post(() => {
			for (const watcher of [...this.watchers]) {
				try {
					watcher(events);
				} catch {
					// A watcher cannot change the registry.
				}
			}
		});
	}

	/** One deliverer; a change made while telling queues behind. */
	private post(deliver: () => void): void {
		this.outbox.push(deliver);
		if (this.delivering) return;
		this.delivering = true;
		try {
			for (let next = this.outbox.shift(); next !== undefined; next = this.outbox.shift()) next();
		} finally {
			this.delivering = false;
		}
	}

	private listChanged(): void {
		if (this.disposed) return;
		const rows = this.rows();
		const events: RegistryEvent[] = [];
		const made = new Map<string, Incarnation>();
		for (const was of this.order) {
			const row = rows.get(was.id);
			if (row === undefined) {
				this.retire(was);
				events.push({ kind: "retired", incarnation: was });
			} else if (identityOf(row) !== was.identity) {
				this.retire(was);
				events.push({ kind: "retired", incarnation: was });
				const next = this.make(row);
				made.set(was.id, next);
				events.push({ kind: "added", incarnation: next });
			} else {
				const flipped = (row.extraUsage === true) !== (was.entry.extraUsage === true);
				was.entry = row;
				made.set(was.id, was);
				if (flipped) events.push({ kind: "extraUsage", incarnation: was, on: row.extraUsage === true });
			}
		}
		const next: Incarnation[] = [];
		for (const [id, row] of rows) {
			let held = made.get(id);
			if (held === undefined) {
				held = this.make(row);
				events.push({ kind: "added", incarnation: held });
			}
			next.push(held);
		}
		this.order = next;
		this.tell(events);
	}

	private keyChanged(id: string): void {
		if (this.disposed) return;
		const incarnation = this.incarnation(id);
		if (incarnation !== undefined) {
			const was = incarnation.era;
			was.retired = true;
			this.eras.delete(was.number);
			this.eraCount += 1;
			incarnation.era = new Era(this.eraCount, incarnation);
			this.eras.set(incarnation.era.number, incarnation.era);
			this.tell([{ kind: "rekeyed", incarnation, retired: was }]);
		}
		this.post(() => {
			for (const listener of [...this.keyListeners]) {
				try {
					listener(id);
				} catch {
					// A listener cannot change the registry.
				}
			}
		});
	}
}
