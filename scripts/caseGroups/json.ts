import { canonicalJson, parseJson, stringifyJson } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { canonicalCases, jsonCases } from "../caseSources/json";

const texts = () => z.object({ texts: z.array(z.string()) });

export const json = caseGroup("json", {
	kinds: {
		roundTrip: kind(
			texts(),
			z.object({ results: z.array(z.object({ ok: z.boolean(), text: z.string().exactOptional() })) }),
			(input) => ({
				results: input.texts.map((text) => {
					const value = parseJson(text);
					return value === undefined ? { ok: false } : { ok: true, text: stringifyJson(value) };
				}),
			}),
		),
		canonical: kind(texts(), z.object({ texts: z.array(z.string().nullable()) }), (input) => ({
			texts: input.texts.map((text) => {
				const value = parseJson(text);
				return value === undefined ? null : canonicalJson(value);
			}),
		})),
	},
	sources: casesFor("json", jsonCases, canonicalCases),
});
