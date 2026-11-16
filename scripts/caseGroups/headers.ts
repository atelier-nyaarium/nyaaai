import { headerOf } from "@nyaaai/contract/headers";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { headerCases } from "../caseSources/headers";

export const headers = caseGroup("headers", {
	kinds: {
		/** Null: no header of that name. */
		of: kind(
			z.object({ headers: z.array(z.tuple([z.string(), z.string()])), name: z.string() }),
			z.object({ value: z.string().nullable() }),
			(input) => ({ value: headerOf(input.headers, input.name) ?? null }),
		),
	},
	sources: casesFor("headers", headerCases),
});
