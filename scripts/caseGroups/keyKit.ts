// Stored keys as a case plants them, and what a keyring reads.

import { keyBindingOf } from "@nyaaai/agentic/provider/entry";
import { KINDS } from "@nyaaai/agentic/provider/kind";
import type { KeyAnswer } from "@nyaaai/contract/entryKeys";
import { z } from "@nyaaai/contract/shape";
import type { EntryFields } from "@nyaaai/contract/shapes/provider";

////////////////////////////////
//  Shapes

/** What the store answers; a key with no `boundTo` is bound to its entry where the script plants it. */
export const storedKey = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("key"), key: z.string(), boundTo: z.string().exactOptional() }),
		z.object({ kind: z.literal("none") }),
		z.object({ kind: z.literal("refused") }),
		z.object({ kind: z.literal("unready") }),
	]);

export type StoredKey = z.infer<ReturnType<typeof storedKey>>;

/** What the keyring reads: the store's answer, or a key that may not be sent. */
export const readKey = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("key"), key: z.string(), boundTo: z.string() }),
		z.object({ kind: z.literal("none") }),
		z.object({ kind: z.literal("refused") }),
		z.object({ kind: z.literal("unready") }),
		z.object({ kind: z.literal("foreign"), boundTo: z.string() }),
		z.object({ kind: z.literal("insecure") }),
	]);

////////////////////////////////
//  Functions & Helpers

/** The store's answer, a key bound to the entry it is planted for; an entry that binds no key gets its kind name, an unlisted id the empty text. */
export function planted(answer: StoredKey, entry: EntryFields | undefined): KeyAnswer {
	if (answer.kind !== "key") return answer;
	const boundTo = answer.boundTo ?? (entry === undefined ? "" : (keyBindingOf(entry, KINDS) ?? entry.kind));
	return { kind: "key", key: answer.key, boundTo };
}

/** Each answer planted for the entry of its id. */
export function plantedAll(
	answers: Readonly<Record<string, StoredKey>>,
	entries: ReadonlyArray<EntryFields & { id: string }>,
): Record<string, KeyAnswer> {
	return Object.fromEntries(
		Object.entries(answers).map(([id, answer]) => [
			id,
			planted(
				answer,
				entries.find((each) => each.id === id),
			),
		]),
	);
}
