// The key source port.

////////////////////////////////
//  Interfaces & Types

/** A key; none; refused by its server; or not yet read. */
export type KeyAnswer = { kind: "key"; key: string } | { kind: "none" } | { kind: "refused" } | { kind: "unready" };

/** Keys by entry id. `get` never rejects: a store that cannot be read answers `unready`. */
export interface EntryKeys {
	get: (id: string) => Promise<KeyAnswer>;
	/** Told the id of an entry whose key changed; the result unsubscribes. */
	onChange: (listener: (id: string) => void) => () => void;
}
