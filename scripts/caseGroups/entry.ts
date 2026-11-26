import {
	boundFor,
	boundKey,
	isEntryId,
	keyBindingOf,
	parseEntries,
	parseEntry,
	readBoundKey,
	sameOrigin,
	secretKey,
} from "@nyaaai/agentic/provider/entry";
import { keyMayTravel } from "@nyaaai/agentic/provider/host";
import { KINDS, type ProviderKind } from "@nyaaai/agentic/provider/kind";
import { z } from "@nyaaai/contract/shape";
import { EntryFields, ProviderEntry, ProviderKindName } from "@nyaaai/contract/shapes/provider";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { entryCases } from "../caseSources/entry";

export const entry = caseGroup("entry", {
	kinds: {
		/** Null: the value is not an entry. */
		parse: kind(z.object({ value: z.unknown() }), z.object({ entry: ProviderEntry.nullable() }), (input) => ({
			entry: parseEntry(input.value) ?? null,
		})),
		/** What a settings value holds: the entries that read, in order. */
		parseList: kind(z.object({ value: z.unknown() }), z.object({ entries: z.array(ProviderEntry) }), (input) => ({
			entries: parseEntries(input.value),
		})),
		/** Null stands for an absent URL. */
		sameOrigin: kind(
			z.object({
				rows: z.array(z.object({ left: z.string().nullable(), right: z.string().nullable() })),
			}),
			z.object({ same: z.array(z.boolean()) }),
			(input) => ({ same: input.rows.map((row) => sameOrigin(row.left ?? undefined, row.right ?? undefined)) }),
		),
		/** Whether each value is an entry id. */
		id: kind(z.object({ values: z.array(z.unknown()) }), z.object({ valid: z.array(z.boolean()) }), (input) => ({
			valid: input.values.map(isEntryId),
		})),
		secretKey: kind(
			z.object({ ids: z.array(z.string()), namespace: z.string() }),
			z.object({ keys: z.array(z.string()) }),
			(input) => ({ keys: input.ids.map((id) => secretKey(id, input.namespace)) }),
		),
		/** What a key typed for each entry is for, over `KINDS`, and whom that names; null binds none. */
		binding: kind(
			z.object({ rows: z.array(EntryFields) }),
			z.object({ rows: z.array(z.object({ binding: z.string().nullable(), for: z.string().nullable() })) }),
			(input) => ({
				rows: input.rows.map((row) => {
					const binding = keyBindingOf(row, KINDS) ?? null;
					return { binding, for: binding === null ? null : boundFor(binding, KINDS) };
				}),
			}),
		),
		/** Whom each stored binding names, over a table of the kinds given. */
		keyFor: kind(
			z.object({ bindings: z.array(z.string()), kinds: z.array(ProviderKindName) }),
			z.object({ for: z.array(z.string()) }),
			(input) => {
				const table: Partial<Record<z.infer<typeof ProviderKindName>, ProviderKind>> = {};
				for (const name of input.kinds) {
					const held = KINDS[name];
					if (held !== undefined) table[name] = held;
				}
				return { for: input.bindings.map((binding) => boundFor(binding, table)) };
			},
		),
		/** The text a host stores for a key; null for fields that bind no key. */
		boundKey: kind(
			z.object({ rows: z.array(z.object({ fields: EntryFields, key: z.string() })) }),
			z.object({ texts: z.array(z.string().nullable()) }),
			(input) => ({ texts: input.rows.map((row) => boundKey(row.fields, row.key, KINDS) ?? null) }),
		),
		/** What a stored text reads as. */
		readBoundKey: kind(
			z.object({ texts: z.array(z.string()) }),
			z.object({
				answers: z.array(
					z.discriminatedUnion("kind", [
						z.object({ kind: z.literal("key"), key: z.string(), boundTo: z.string() }),
						z.object({ kind: z.literal("none") }),
					]),
				),
			}),
			(input) => ({ answers: input.texts.map(readBoundKey) }),
		),
		/** Whether a key may go to each URL. */
		travel: kind(z.object({ urls: z.array(z.string()) }), z.object({ travels: z.array(z.boolean()) }), (input) => ({
			travels: input.urls.map(keyMayTravel),
		})),
	},
	sources: casesFor("entry", entryCases),
});
