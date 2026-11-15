import { base64Decode, base64Encode } from "@nyaaai/contract/base64";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { base64Cases } from "../caseSources/json";

export const base64 = caseGroup("base64", {
	kinds: {
		/** Null marks a refusal. */
		decode: kind(
			z.object({ texts: z.array(z.string()) }),
			z.object({ texts: z.array(z.string().nullable()) }),
			(input) => ({
				texts: input.texts.map((text) => {
					const bytes = base64Decode(text);
					return bytes === undefined ? null : base64Encode(bytes);
				}),
			}),
		),
	},
	sources: casesFor("base64", base64Cases),
});
