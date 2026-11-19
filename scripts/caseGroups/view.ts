import { MODELS } from "@nyaaai/agentic/models/curated";
import { codecFactsOf, runEffort, tiersOf, unknownView } from "@nyaaai/agentic/models/view";
import { parseJson } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { ModelView, Price, PriceTier } from "@nyaaai/contract/shapes/model";
import { ApiFacts, CodecFacts, Effort } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { viewCases } from "../caseSources/view";

////////////////////////////////
//  Shapes

/** The view parts helpers read; the row is made up. */
const ViewSpec = () =>
	z.object({
		reasoning: z.boolean(),
		status: z.enum(["ok", "retired", "noZdrHost", "noTools", "unknown"]),
		efforts: z.array(Effort),
		outputLimit: z.number().exactOptional(),
		price: z.union([Price, z.literal("free"), z.literal("unknown")]).exactOptional(),
		api: ApiFacts.exactOptional(),
	});

////////////////////////////////
//  Functions & Helpers

function viewOf(spec: z.infer<ReturnType<typeof ViewSpec>>): ModelView {
	return {
		row: {
			key: "k",
			bindings: {},
			family: "k",
			label: "K",
			reasoning: spec.reasoning,
			schema: false,
			cache: "implicit",
			verified: "2026-01-01",
			verifiedPrice: { prompt: 0, completion: 0 },
			identity: { kind: "curated", key: "k" },
		},
		status: spec.status,
		efforts: spec.efforts,
		hosts: [],
		...(spec.outputLimit === undefined ? {} : { outputLimit: spec.outputLimit }),
		...(spec.price === undefined ? {} : { price: spec.price }),
		...(spec.api === undefined ? {} : { api: spec.api }),
	};
}

////////////////////////////////
//  Group

export const view = caseGroup("view", {
	kinds: {
		/** The view of a curated row no catalog lists. */
		unknown: kind(z.object({ keys: z.array(z.string()) }), z.object({ views: z.array(ModelView) }), (input) => ({
			views: input.keys.map((key) => {
				const row = MODELS.find((each) => each.key === key);
				if (row === undefined) throw new Error(`No row ${key}.`);
				return unknownView(row);
			}),
		})),
		/** Null: the view is absent. An unpriced view has no tiers: the list is null too. */
		tiers: kind(
			z.object({
				rows: z.array(
					z.object({
						view: z
							.object({
								price: z.union([Price, z.literal("free"), z.literal("unknown")]).exactOptional(),
							})
							.nullable(),
					}),
				),
			}),
			z.object({ tiers: z.array(z.array(PriceTier).nullable()) }),
			(input) => ({
				tiers: input.rows.map((row) => {
					const tiers = tiersOf(row.view ?? undefined);
					return tiers === undefined ? null : [...tiers];
				}),
			}),
		),
		/** Null: the view is absent. */
		codecFacts: kind(
			z.object({ views: z.array(ViewSpec().nullable()) }),
			z.object({ facts: z.array(CodecFacts.nullable()) }),
			(input) => ({
				facts: input.views.map((spec) => codecFactsOf(spec === null ? undefined : viewOf(spec)) ?? null),
			}),
		),
		/** Each text is a JSON number, so an overflow can be cased; null: no limit is sent. */
		outputLimit: kind(
			z.object({ texts: z.array(z.string()) }),
			z.object({ limits: z.array(z.int().nullable()) }),
			(input) => ({
				limits: input.texts.map((text) => {
					const limit = parseJson(text);
					if (typeof limit !== "number") throw new Error(`${text} is not a number.`);
					const view = viewOf({ reasoning: false, status: "ok", efforts: [], outputLimit: limit });
					return codecFactsOf(view)?.outputLimit ?? null;
				}),
			}),
		),
		/** Null stands for an absent view or an absent wish, and for a result of none. */
		runEffort: kind(
			z.object({ rows: z.array(z.object({ view: ViewSpec().nullable(), wanted: Effort.nullable() })) }),
			z.object({ efforts: z.array(Effort.nullable()) }),
			(input) => ({
				efforts: input.rows.map(
					(row) =>
						runEffort(row.view === null ? undefined : viewOf(row.view), row.wanted ?? undefined) ?? null,
				),
			}),
		),
	},
	sources: casesFor("view", viewCases),
});
