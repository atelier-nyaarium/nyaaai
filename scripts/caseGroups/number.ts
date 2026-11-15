import { fixedText, integerOfText, numberOfText, numberText, roundHalfUp } from "@nyaaai/contract/number";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { numberCases } from "../caseSources/number";

const values = () => z.object({ values: z.array(z.number()) });
const texts = () => z.object({ texts: z.array(z.string()) });

export const number = caseGroup("number", {
	kinds: {
		text: kind(values(), texts(), (input) => ({ texts: input.values.map(numberText) })),
		// Keeps negative zero.
		round: kind(values(), texts(), (input) => ({
			texts: input.values.map((value) =>
				Object.is(roundHalfUp(value), -0) ? "-0" : numberText(roundHalfUp(value)),
			),
		})),
		fixed: kind(
			z.object({ values: z.array(z.number()), digits: z.array(z.int().min(0)) }),
			z.object({ rows: z.array(z.array(z.string())) }),
			(input) => ({ rows: input.values.map((value) => input.digits.map((digits) => fixedText(value, digits))) }),
		),
		ofText: kind(texts(), z.object({ values: z.array(z.number().nullable()) }), (input) => ({
			values: input.texts.map((text) => numberOfText(text) ?? null),
		})),
		integerOfText: kind(texts(), z.object({ values: z.array(z.int().nullable()) }), (input) => ({
			values: input.texts.map((text) => integerOfText(text) ?? null),
		})),
	},
	sources: casesFor("number", numberCases),
});
