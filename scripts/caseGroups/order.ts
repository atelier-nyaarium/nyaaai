import { sortedTexts } from "@nyaaai/contract/order";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { canonicalCases } from "../caseSources/json";

const texts = () => z.object({ texts: z.array(z.string()) });

export const order = caseGroup("order", {
	kinds: { sortedTexts: kind(texts(), texts(), (input) => ({ texts: sortedTexts(input.texts) })) },
	sources: casesFor("order", canonicalCases),
});
