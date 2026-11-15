import { jsString, parseJson } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { canonicalCases } from "../caseSources/json";

export const js = caseGroup("js", {
	kinds: {
		string: kind(z.object({ texts: z.array(z.string()) }), z.object({ strings: z.array(z.string()) }), (input) => ({
			strings: input.texts.map((text) => {
				const value = parseJson(text);
				if (value === undefined) throw new Error("a value text is not JSON");
				return jsString(value);
			}),
		})),
	},
	sources: casesFor("js", canonicalCases),
});
