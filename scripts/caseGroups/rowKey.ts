import { parseRowKey, rowKey, servedIdOf, shownId, shownKey } from "@nyaaai/agentic/models/rowKey";
import { z } from "@nyaaai/contract/shape";
import { RowIdentity } from "@nyaaai/contract/shapes/model";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { rowKeyCases } from "../caseSources/rowKey";

export const rowKeyGroup = caseGroup("rowKey", {
	kinds: {
		/** `back`: the key read again; null when it does not read. */
		build: kind(
			z.object({ identities: z.array(RowIdentity) }),
			z.object({ keys: z.array(z.string()), back: z.array(RowIdentity.nullable()) }),
			(input) => {
				const keys = input.identities.map(rowKey);
				return { keys, back: keys.map((key) => parseRowKey(key) ?? null) };
			},
		),
		/** Null: the value names no row. */
		parse: kind(
			z.object({ values: z.array(z.unknown()) }),
			z.object({ identities: z.array(RowIdentity.nullable()) }),
			(input) => ({ identities: input.values.map((value) => parseRowKey(value) ?? null) }),
		),
		/** `ids`: Browse ids of identities. `keyed`: of stored keys, an unreadable one as stored. */
		shown: kind(
			z.object({ identities: z.array(RowIdentity), keys: z.array(z.string()) }),
			z.object({ ids: z.array(z.string()), keyed: z.array(z.string()) }),
			(input) => ({ ids: input.identities.map(shownId), keyed: input.keys.map(shownKey) }),
		),
		/** Null for a curated identity, which no kind serves under its own id. */
		servedId: kind(
			z.object({ identities: z.array(RowIdentity) }),
			z.object({ ids: z.array(z.string().nullable()) }),
			(input) => ({
				ids: input.identities.map((identity) => (identity.kind === "curated" ? null : servedIdOf(identity))),
			}),
		),
	},
	sources: casesFor("rowKey", rowKeyCases),
});
