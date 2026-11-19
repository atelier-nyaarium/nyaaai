import { price, reportedCost, stepCost, wantsLongCache } from "@nyaaai/agentic/usage/cost";
import { z } from "@nyaaai/contract/shape";
import { Price } from "@nyaaai/contract/shapes/model";
import { NormalizedUsage } from "@nyaaai/contract/shapes/step";
import { CallTag, Cost } from "@nyaaai/contract/shapes/usage";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { costCases } from "../caseSources/cost";

export const cost = caseGroup("cost", {
	kinds: {
		/**
		 * `price` is the model's listing: absent when the model is not listed, else a price, `free` or
		 * `unknown`. `longCache` is the call's ask for one-hour cache writes.
		 */
		step: kind(
			z.object({
				rows: z.array(
					z.object({
						usage: NormalizedUsage.exactOptional(),
						price: z.union([Price, z.literal("free"), z.literal("unknown")]).exactOptional(),
						longCache: z.boolean(),
					}),
				),
			}),
			z.object({ costs: z.array(Cost) }),
			(input) => ({
				costs: input.rows.map((row) =>
					stepCost(row.usage, row.price === undefined ? undefined : { price: row.price }, row.longCache),
				),
			}),
		),
		/** Null: not a usable amount. */
		price: kind(
			z.object({ values: z.array(z.unknown()) }),
			z.object({ prices: z.array(z.number().nullable()) }),
			(input) => ({ prices: input.values.map((value) => price(value) ?? null) }),
		),
		/** Null: the reply reported no cost. */
		reported: kind(
			z.object({
				rows: z.array(
					z.object({
						cost: z.number().exactOptional(),
						byok: z.boolean().exactOptional(),
						upstreamCost: z.number().exactOptional(),
					}),
				),
			}),
			z.object({ values: z.array(z.number().nullable()) }),
			(input) => ({ values: input.rows.map((row) => reportedCost(row) ?? null) }),
		),
		/** `view`: null for no view, else what the listing says of one-hour cache writes. */
		longCache: kind(
			z.object({
				rows: z.array(
					z.object({
						tag: CallTag,
						view: z.object({ cacheWrite1h: z.literal(true).exactOptional() }).nullable(),
					}),
				),
			}),
			z.object({ values: z.array(z.boolean()) }),
			(input) => ({ values: input.rows.map((row) => wantsLongCache(row.tag, row.view ?? undefined)) }),
		),
	},
	sources: casesFor("cost", costCases),
});
