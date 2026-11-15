import { ISSUE_MESSAGES, type IssueCode } from "@nyaaai/contract/issues";
import { parseJson } from "@nyaaai/contract/json";
import type { Json } from "@nyaaai/contract/schemaExport";
import { z } from "@nyaaai/contract/shape";
import { PathPart } from "@nyaaai/contract/shapes/frames";
import { type Issue, normalizeSchema, schemaProblems, validate } from "@nyaaai/contract/strictSchema";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { strictSchemaCases } from "../caseSources/strictSchema";

const StrictSchemaIssue = z.object({
	path: z.array(PathPart),
	code: z.enum(Object.keys(ISSUE_MESSAGES) as [IssueCode, ...IssueCode[]]),
	message: z.string(),
});
const schemas = () => z.object({ schemas: z.array(z.unknown()) });
const issues = () => z.object({ issues: z.array(z.array(StrictSchemaIssue)) });

function issuesOf(schema: unknown, text: string): Issue[] {
	const value = parseJson(text);
	if (value === undefined) throw new Error("a value text is not JSON");
	return validate(schema as Json, value as Json);
}

export const strictSchema = caseGroup("strictSchema", {
	shapes: { StrictSchemaIssue },
	kinds: {
		normalize: kind(schemas(), schemas(), (input) => ({
			schemas: input.schemas.map((schema) => normalizeSchema(schema as Json)),
		})),
		problems: kind(schemas(), z.object({ problems: z.array(z.array(StrictSchemaIssue)) }), (input) => ({
			problems: input.schemas.map((schema) => schemaProblems(normalizeSchema(schema as Json))),
		})),
		/** Model JSON text. */
		validate: kind(z.object({ schema: z.unknown(), texts: z.array(z.string()) }), issues(), (input) => ({
			issues: input.texts.map((text) => issuesOf(input.schema, text)),
		})),
		/** Repeated item JSON. */
		validateList: kind(
			z.object({ schema: z.unknown(), item: z.string(), counts: z.array(z.int().min(0)) }),
			issues(),
			(input) => ({
				issues: input.counts.map((count) =>
					issuesOf(input.schema, `[${Array.from({ length: count }, () => input.item).join(",")}]`),
				),
			}),
		),
	},
	sources: casesFor("strictSchema", strictSchemaCases),
});
