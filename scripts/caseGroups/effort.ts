import { EFFORTS, resolveEffort } from "@nyaaai/agentic/effort";
import { z } from "@nyaaai/contract/shape";
import { Effort } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { effortCases } from "../caseSources/effort";

export const effort = caseGroup("effort", {
	kinds: {
		ladder: kind(z.object({}), z.object({ efforts: z.array(Effort) }), () => ({ efforts: [...EFFORTS] })),
		/** One result per wanted name; null when nothing fits. */
		resolve: kind(
			z.object({ offered: z.array(z.string()), wanted: z.array(z.string()) }),
			z.object({ values: z.array(z.string().nullable()) }),
			(input) => ({ values: input.wanted.map((wanted) => resolveEffort(input.offered, wanted) ?? null) }),
		),
	},
	sources: casesFor("effort", effortCases),
});
