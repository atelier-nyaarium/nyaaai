import { MODELS } from "@nyaaai/agentic/models/curated";
import { z } from "@nyaaai/contract/shape";
import { ModelRow } from "@nyaaai/contract/shapes/model";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";

export const curated = caseGroup("curated", {
	kinds: {
		/** The whole list, in picker order. */
		rows: kind(z.object({}), z.object({ rows: z.array(ModelRow) }), () => ({ rows: [...MODELS] })),
	},
	sources: casesFor("curated", [{ kind: "curated.rows", name: "rows", input: {} }]),
});
