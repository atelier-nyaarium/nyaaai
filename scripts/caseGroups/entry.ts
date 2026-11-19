import { isEntryId, parseEntries, parseEntry, sameOrigin, secretKey } from "@nyaaai/agentic/provider/entry";
import { z } from "@nyaaai/contract/shape";
import { ProviderEntry } from "@nyaaai/contract/shapes/provider";
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
	},
	sources: casesFor("entry", entryCases),
});
