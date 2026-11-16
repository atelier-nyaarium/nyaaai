import { z } from "@nyaaai/contract/shape";
import { originOf } from "@nyaaai/contract/url";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { urlCases } from "../caseSources/url";

export const url = caseGroup("url", {
	kinds: {
		/** Null: not an origin this primitive reads. */
		origin: kind(
			z.object({ urls: z.array(z.string()) }),
			z.object({ origins: z.array(z.string().nullable()) }),
			(input) => ({
				origins: input.urls.map((held) => originOf(held) ?? null),
			}),
		),
	},
	sources: casesFor("url", urlCases),
});
