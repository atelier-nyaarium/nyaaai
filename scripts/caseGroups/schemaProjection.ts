import { cleanSchema, nullableOptionals, strictSchema, withoutNulls } from "@nyaaai/agentic/schemaProjection";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { schemaProjectionCases } from "../caseSources/schemaProjection";

export const schemaProjection = caseGroup("schemaProjection", {
	kinds: {
		/** `value` matters to `withoutNulls` alone. A refusal leaves `output` null. */
		project: kind(
			z.object({
				projection: z.enum(["clean", "nullable", "withoutNulls", "strict"]),
				schema: z.record(z.string(), z.unknown()),
				value: z.unknown(),
			}),
			z.object({ output: z.unknown(), refusal: z.string().nullable() }),
			(input) => {
				try {
					switch (input.projection) {
						case "clean":
							return { output: cleanSchema(input.schema), refusal: null };
						case "nullable":
							return { output: nullableOptionals(input.schema), refusal: null };
						case "withoutNulls":
							return { output: withoutNulls(input.schema, input.value), refusal: null };
						case "strict":
							return { output: strictSchema(input.schema), refusal: null };
					}
				} catch (error) {
					if (!(error instanceof Error)) throw error;
					return { output: null, refusal: error.message };
				}
			},
		),
	},
	sources: casesFor("schemaProjection", schemaProjectionCases),
});
