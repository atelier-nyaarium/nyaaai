import { z } from "@nyaaai/contract/shape";
import { originOf, parseUrl } from "@nyaaai/contract/url";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { urlCases } from "../caseSources/url";

const UrlParts = () =>
	z.object({
		scheme: z.string(),
		username: z.string(),
		password: z.string(),
		host: z.string(),
		port: z.string(),
		origin: z.string(),
		path: z.string(),
		query: z.string(),
		fragment: z.string(),
	});

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
		/** Null: not a URL this primitive reads. */
		parts: kind(
			z.object({ urls: z.array(z.string()) }),
			z.object({ parts: z.array(UrlParts().nullable()) }),
			(input) => ({ parts: input.urls.map((held) => parseUrl(held) ?? null) }),
		),
	},
	sources: casesFor("url", urlCases),
});
