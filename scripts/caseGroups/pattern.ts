import { Pattern, patternProblem } from "@nyaaai/contract/pattern";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { patternCases } from "../caseSources/pattern";

const PatternCaseMatch = z.object({
	start: z.int().min(0),
	end: z.int().min(0),
	text: z.string(),
	captures: z.array(z.string().nullable()),
});

export const pattern = caseGroup("pattern", {
	shapes: { PatternCaseMatch },
	kinds: {
		problems: kind(
			z.object({ patterns: z.array(z.object({ source: z.string(), flags: z.string() })) }),
			z.object({ refused: z.array(z.boolean()) }),
			(input) => ({
				refused: input.patterns.map(({ source, flags }) => patternProblem(source, flags) !== undefined),
			}),
		),
		matches: kind(
			z.object({ source: z.string(), flags: z.string(), texts: z.array(z.string()) }),
			z.object({ results: z.array(z.object({ test: z.boolean(), all: z.array(PatternCaseMatch) })) }),
			(input) => {
				const held = Pattern.compile(input.source, input.flags);
				return { results: input.texts.map((text) => ({ test: held.test(text), all: held.findAll(text) })) };
			},
		),
		find: kind(
			z.object({ source: z.string(), flags: z.string(), text: z.string(), froms: z.array(z.int().min(0)) }),
			z.object({ found: z.array(PatternCaseMatch.nullable()) }),
			(input) => {
				const held = Pattern.compile(input.source, input.flags);
				return { found: input.froms.map((from) => held.find(input.text, from) ?? null) };
			},
		),
		edits: kind(
			z.object({ source: z.string(), flags: z.string(), template: z.string(), texts: z.array(z.string()) }),
			z.object({ results: z.array(z.object({ replaced: z.string(), pieces: z.array(z.string()) })) }),
			(input) => {
				const held = Pattern.compile(input.source, input.flags);
				return {
					results: input.texts.map((text) => ({
						replaced: held.replaceAll(text, input.template),
						pieces: held.split(text),
					})),
				};
			},
		),
	},
	sources: casesFor("pattern", patternCases),
});
