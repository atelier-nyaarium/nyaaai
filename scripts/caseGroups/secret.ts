import { leaksSpelling, scrubSpellings, secretSpellings } from "@nyaaai/contract/secret";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { secretCases } from "../caseSources/secret";

export const secret = caseGroup("secret", {
	kinds: {
		/** The spellings of a secret, in order; `folded` lists the positions compared without ASCII case. */
		spellings: kind(
			z.object({ secret: z.string() }),
			z.object({ spellings: z.array(z.object({ text: z.string(), folded: z.array(z.int()) })) }),
			(input) => ({ spellings: secretSpellings(input.secret) }),
		),
		/** `text` with every spelling replaced in one pass, and whether any occurs in it. */
		scrub: kind(
			z.object({ secret: z.string(), text: z.string() }),
			z.object({ scrubbed: z.string(), leaks: z.boolean() }),
			(input) => {
				const spellings = secretSpellings(input.secret);
				return { scrubbed: scrubSpellings(input.text, spellings), leaks: leaksSpelling(input.text, spellings) };
			},
		),
	},
	sources: casesFor("secret", secretCases),
});
