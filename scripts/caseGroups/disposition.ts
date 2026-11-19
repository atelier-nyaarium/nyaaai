import { MODELS } from "@nyaaai/agentic/models/curated";
import { unknownView } from "@nyaaai/agentic/models/view";
import { dispositionOf, SKIP_REASONS, skipReasonOf } from "@nyaaai/agentic/provider/disposition";
import { z } from "@nyaaai/contract/shape";
import { Disposition, SkipReason } from "@nyaaai/contract/shapes/provider";
import { Failure } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { dispositionCases } from "../caseSources/disposition";

export const disposition = caseGroup("disposition", {
	kinds: {
		/** `status`: the status of the view the failure met; null for none. */
		of: kind(
			z.object({
				rows: z.array(
					z.object({
						failure: Failure,
						status: z.enum(["ok", "retired", "noZdrHost", "noTools", "unknown"]).nullable(),
					}),
				),
			}),
			z.object({ dispositions: z.array(Disposition) }),
			(input) => ({
				dispositions: input.rows.map((row) => {
					const first = MODELS[0];
					if (first === undefined) throw new Error("The curated list is empty.");
					const view = row.status === null ? undefined : { ...unknownView(first), status: row.status };
					return dispositionOf(row.failure, view);
				}),
			}),
		),
		/** `all`: the reasons a run can skip a route for, in order. */
		skipReason: kind(
			z.object({ failures: z.array(Failure) }),
			z.object({ reasons: z.array(SkipReason), all: z.array(SkipReason) }),
			(input) => ({ reasons: input.failures.map(skipReasonOf), all: [...SKIP_REASONS] }),
		),
	},
	sources: casesFor("disposition", dispositionCases),
});
