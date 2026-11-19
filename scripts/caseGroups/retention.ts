import { isZdr, retentionOf } from "@nyaaai/agentic/kinds/opencode/retention";
import { z } from "@nyaaai/contract/shape";
import { ProviderPlan } from "@nyaaai/contract/shapes/provider";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { retentionCases } from "../caseSources/retention";

export const retention = caseGroup("retention", {
	kinds: {
		/** Null: the plan's table says nothing of the id. */
		of: kind(
			z.object({ rows: z.array(z.object({ plan: ProviderPlan, id: z.string() })) }),
			z.object({
				rows: z.array(z.object({ zdr: z.boolean(), until: z.string().exactOptional() }).nullable()),
			}),
			(input) => ({ rows: input.rows.map((row) => retentionOf(row.plan, row.id) ?? null) }),
		),
		/** `now` is epoch milliseconds. */
		isZdr: kind(
			z.object({ rows: z.array(z.object({ plan: ProviderPlan, id: z.string(), now: z.int() })) }),
			z.object({ values: z.array(z.boolean()) }),
			(input) => ({ values: input.rows.map((row) => isZdr(row.plan, row.id, row.now)) }),
		),
	},
	sources: casesFor("retention", retentionCases),
});
