// The one reader of a host's keys, and the guard every key a kind would send passes.

import type { EntryKeys, KeyAnswer } from "@nyaaai/contract/entryKeys";
import type { Span } from "@nyaaai/contract/fetch";
import type { ProviderList } from "@nyaaai/contract/providerList";
import type { EntryFields, KeyState, ProviderEntry, ProviderKindName } from "@nyaaai/contract/shapes/provider";
import { type KeyKinds, keyBindingOf, takesKey } from "./entry";
import { keyMayTravel } from "./host";
import type { EntryRegistry } from "./incarnation";
import type { KindDeps, ProviderKind } from "./kind";

////////////////////////////////
//  Interfaces & Types

/** The port's answer, or a stored key that may not be sent: given for another binding, or over plain http. */
export type KeyRead = KeyAnswer | { kind: "foreign"; boundTo: string } | { kind: "insecure" };

/** Keys by entry id, as the keyring reads them. */
export interface KeyReads {
	get(id: string): Promise<KeyRead>;
	/** Told the id of an entry whose key changed; the result unsubscribes. */
	onChange(listener: (id: string) => void): () => void;
}

type Kinds = Partial<Record<ProviderKindName, ProviderKind>>;

export interface KeyringDeps {
	/** The host's store; nothing else calls its `get`. */
	keys: Pick<EntryKeys, "get">;
	entries: EntryRegistry;
	kinds: Kinds;
	/** Where each kind's requests go. */
	kindDeps(kind: ProviderKindName): KindDeps;
}

/** A host's own reader of configured keys. */
export interface VouchedRead {
	get(id: string): Promise<string | undefined>;
	onChange(listener: (id: string) => void): () => void;
}

////////////////////////////////
//  Functions & Helpers

/** The guard: a key only for its binding, only where it may travel. */
function guarded(kinds: Kinds, fields: EntryFields, key: string, boundTo: string, deps: KindDeps): KeyRead {
	const kind = kinds[fields.kind];
	if (kind === undefined || !takesKey(kind)) return { kind: "none" };
	if (boundTo !== keyBindingOf(fields, kinds)) return { kind: "foreign", boundTo };
	// Missing or unreadable is insecure.
	const address = deps.baseUrl ?? kind.address(fields);
	if (address === undefined || !keyMayTravel(address)) return { kind: "insecure" };
	return { kind: "key", key, boundTo };
}

/** A typed key's check: the guard, then the kind's own. */
export async function checkTyped(
	kinds: Kinds,
	fields: EntryFields,
	secret: string | undefined,
	deps: KindDeps,
	span: Span,
): Promise<KeyState> {
	const kind = kinds[fields.kind];
	if (kind === undefined) return { kind: "none" };
	if (secret === undefined || !takesKey(kind)) return kind.check(fields, undefined, deps, span);
	const binding = keyBindingOf(fields, kinds);
	if (binding === undefined) return { kind: "invalid" };
	const read = guarded(kinds, fields, secret, binding, deps);
	return read.kind === "insecure" ? { kind: "insecure" } : kind.check(fields, secret, deps, span);
}

/** Keys from a host's configuration, each bound to its entry as listed when the read began. */
export function vouchedKeys(read: VouchedRead, list: Pick<ProviderList, "entries">, kinds: KeyKinds): EntryKeys {
	const bindingOf = (id: string): string | undefined => {
		const entry = list.entries().find((each) => each.id === id);
		return entry === undefined ? undefined : keyBindingOf(entry, kinds);
	};
	return {
		get: async (id) => {
			const binding = bindingOf(id);
			if (binding === undefined) return { kind: "none" };
			let key: string | undefined;
			try {
				key = await read.get(id);
			} catch {
				return { kind: "unready" };
			}
			// The entry changed while the read ran.
			if (key === undefined || bindingOf(id) !== binding) return { kind: "none" };
			return { kind: "key", key, boundTo: binding };
		},
		onChange: (listener) => read.onChange(listener),
	};
}

////////////////////////////////
//  Classes

/** The only caller of the store's `get`. */
export class Keyring implements KeyReads {
	constructor(private readonly d: KeyringDeps) {}

	/** The entry's key as its binding and address allow; never rejects. */
	async read(entry: ProviderEntry): Promise<KeyRead> {
		let answer: KeyAnswer;
		try {
			answer = await this.d.keys.get(entry.id);
		} catch {
			return { kind: "unready" };
		}
		if (answer.kind !== "key") return answer;
		return guarded(this.d.kinds, entry, answer.key, answer.boundTo, this.d.kindDeps(entry.kind));
	}

	/** The listed entry's; none when the registry holds no such id. */
	async get(id: string): Promise<KeyRead> {
		const incarnation = this.d.entries.incarnation(id);
		return incarnation === undefined ? { kind: "none" } : this.read(incarnation.entry);
	}

	onChange(listener: (id: string) => void): () => void {
		return this.d.entries.onKeyChange(listener);
	}
}
