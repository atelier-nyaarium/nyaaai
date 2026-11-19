// OpenRouter decoder cases. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { cut, type Json, setIn, wrongFor } from "./catalogKit";

////////////////////////////////
//  Fixtures

const PRICING: Json = {
	prompt: "0.000001",
	completion: "0.000002",
	input_cache_read: "0.0000001",
	input_cache_write: "0.00000125",
	input_cache_write_1h: "0.000002",
	overrides: [
		{
			min_prompt_tokens: 200_000,
			prompt: "0.000002",
			completion: "0.000004",
			input_cache_read: "0.0000002",
			input_cache_write: "0.0000025",
			input_cache_write_1h: "0.000004",
		},
	],
};

const MODEL: Json = {
	id: "lab/model-a",
	name: "Model A",
	created: 1_790_000_000,
	context_length: 128_000,
	expiration_date: "2027-01-15",
	pricing: PRICING,
	reasoning: { supported_efforts: ["low", "high"] },
	supported_parameters: ["tools", "reasoning"],
};

const ENDPOINT: Json = {
	model_id: "lab/model-a",
	tag: "host-a",
	provider_name: "Host A",
	context_length: 128_000,
	max_prompt_tokens: 100_000,
	supported_parameters: ["tools", "reasoning"],
	pricing: PRICING,
};

type Site = {
	/** The path of the field in the item. */
	path: string[];
	/** The kind of value the field holds. */
	holds: "text" | "number" | "object" | "list";
};

/** Every field that falls back, as the old schema had it. */
const MODEL_SITES: Site[] = [
	{ path: ["name"], holds: "text" },
	{ path: ["created"], holds: "number" },
	{ path: ["context_length"], holds: "number" },
	{ path: ["expiration_date"], holds: "text" },
	{ path: ["pricing"], holds: "object" },
	{ path: ["pricing", "overrides"], holds: "list" },
	{ path: ["reasoning"], holds: "object" },
	{ path: ["reasoning", "supported_efforts"], holds: "list" },
	{ path: ["supported_parameters"], holds: "list" },
];

const ENDPOINT_SITES: Site[] = [
	{ path: ["tag"], holds: "text" },
	{ path: ["provider_name"], holds: "text" },
	{ path: ["context_length"], holds: "number" },
	{ path: ["max_prompt_tokens"], holds: "number" },
	{ path: ["supported_parameters"], holds: "list" },
	{ path: ["pricing"], holds: "object" },
	{ path: ["pricing", "overrides"], holds: "list" },
];

////////////////////////////////
//  Functions & Helpers

const modelCase = (name: string, body: unknown): CaseSource => ({
	kind: "catalog.openrouterModels",
	name,
	input: { body },
});

const endpointCase = (name: string, body: unknown): CaseSource => ({
	kind: "catalog.openrouterEndpoints",
	name,
	input: { body },
});

const data = (...items: unknown[]) => ({ data: items });

/** One case per way a site can be wrong: omitted, null, a wrong type, and for a list a wrong item. */
function siteCases(
	make: (name: string, body: unknown) => CaseSource,
	item: Json,
	sites: Site[],
	prefix: string,
): CaseSource[] {
	const cases: CaseSource[] = [];
	for (const site of sites) {
		const name = `${prefix}-${site.path.join(".")}`;
		cases.push(make(`${name}-omitted`, data(cut(item, site.path))));
		cases.push(make(`${name}-null`, data(setIn(item, site.path, null))));
		cases.push(make(`${name}-wrong-type`, data(setIn(item, site.path, wrongFor(site.holds)))));
		if (site.holds === "list") {
			const held = site.path[site.path.length - 1];
			const wrongItem = held === "overrides" ? [{ min_prompt_tokens: "200000", prompt: "0.000002" }] : ["a", 3];
			cases.push(make(`${name}-wrong-item`, data(setIn(item, site.path, wrongItem))));
		}
	}
	return cases;
}

////////////////////////////////
//  Cases

export const catalogDecodeCases: CaseSource[] = [
	// Models.
	modelCase("models-full", data(MODEL)),
	modelCase("models-minimal", data({ id: "lab/bare" })),
	...siteCases(modelCase, MODEL, MODEL_SITES, "models"),
	modelCase(
		"models-pricing-overrides-item-lacks-minimum",
		data(setIn(MODEL, ["pricing", "overrides"], [{ prompt: "0.000002" }])),
	),
	modelCase(
		"models-pricing-overrides-item-not-object",
		data(setIn(MODEL, ["pricing", "overrides"], [{ min_prompt_tokens: 1 }, "x"])),
	),
	modelCase(
		"models-pricing-overrides-two-kept-in-order",
		data(
			setIn(
				MODEL,
				["pricing", "overrides"],
				[
					{ min_prompt_tokens: 300_000, prompt: 0.000003 },
					{ min_prompt_tokens: 100_000, prompt: 0.0000015 },
				],
			),
		),
	),
	modelCase("models-pricing-empty-object", data(setIn(MODEL, ["pricing"], {}))),
	modelCase(
		"models-pricing-any-json-passes-through",
		data(
			setIn(MODEL, ["pricing"], {
				prompt: 0.000001,
				completion: { nested: [1, 2] },
				input_cache_read: null,
				input_cache_write: true,
			}),
		),
	),
	modelCase("models-reasoning-empty-object", data(setIn(MODEL, ["reasoning"], {}))),
	modelCase("models-reasoning-wrong-type-list", data(setIn(MODEL, ["reasoning"], ["low"]))),
	modelCase("models-id-missing", data(cut(MODEL, ["id"]))),
	modelCase("models-id-null", data(setIn(MODEL, ["id"], null))),
	modelCase("models-id-number", data(setIn(MODEL, ["id"], 7))),
	modelCase("models-id-empty-text", data(setIn(MODEL, ["id"], ""))),
	modelCase("models-item-null", data(null, MODEL)),
	modelCase("models-item-text", data("lab/model-a", MODEL)),
	modelCase("models-item-list", data([MODEL], MODEL)),
	modelCase("models-bad-items-between-good", data({ id: "lab/one" }, { unexpected: true }, { id: "lab/two" })),
	modelCase(
		"models-same-id-twice-later-replaces-in-place",
		data({ id: "lab/one", name: "First" }, { id: "lab/two" }, { id: "lab/one", name: "Second" }),
	),
	modelCase("models-unknown-keys-dropped", data({ ...(MODEL as object), extra: 1, "": 2 })),
	modelCase("models-empty-data", data()),
	modelCase("models-body-null", null),
	modelCase("models-body-text", "data"),
	modelCase("models-body-list", [MODEL]),
	modelCase("models-body-number", 7),
	modelCase("models-body-without-data", { models: [MODEL] }),
	modelCase("models-data-null", { data: null }),
	modelCase("models-data-object", { data: { id: "lab/one" } }),
	modelCase("models-data-text", { data: "lab/one" }),
	modelCase("models-proto-key-ignored", JSON.parse('{"data":[{"id":"lab/one","__proto__":{"name":"x"}}]}')),

	// Endpoints.
	endpointCase("endpoints-full", data(ENDPOINT)),
	endpointCase("endpoints-minimal", data({ model_id: "lab/bare" })),
	...siteCases(endpointCase, ENDPOINT, ENDPOINT_SITES, "endpoints"),
	endpointCase("endpoints-model-id-missing", data(cut(ENDPOINT, ["model_id"]))),
	endpointCase("endpoints-model-id-null", data(setIn(ENDPOINT, ["model_id"], null))),
	endpointCase("endpoints-model-id-number", data(setIn(ENDPOINT, ["model_id"], 7))),
	endpointCase("endpoints-item-null", data(null, ENDPOINT)),
	endpointCase("endpoints-item-list", data([ENDPOINT], ENDPOINT)),
	endpointCase(
		"endpoints-grouped-by-model-in-first-seen-order",
		data(
			{ model_id: "lab/b", tag: "one" },
			{ model_id: "lab/a", tag: "two" },
			{ model_id: "lab/b", tag: "three" },
			{ unexpected: true },
			{ model_id: "lab/a", tag: "four" },
		),
	),
	endpointCase("endpoints-empty-data", data()),
	endpointCase("endpoints-body-null", null),
	endpointCase("endpoints-body-text", "data"),
	endpointCase("endpoints-body-list", [ENDPOINT]),
	endpointCase("endpoints-body-without-data", { endpoints: [ENDPOINT] }),
	endpointCase("endpoints-data-null", { data: null }),
	endpointCase("endpoints-data-object", { data: ENDPOINT }),
];
