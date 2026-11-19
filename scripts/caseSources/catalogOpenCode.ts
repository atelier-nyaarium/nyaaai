// OpenCode catalog cases: the decoder, the join and the plan lists. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { cut, DAY, type Json, NOW, setIn, wrongFor } from "./catalogKit";

////////////////////////////////
//  Fixtures

const API: Json = {
	compatibility: { reasoning_field: "reasoning_content" },
	capabilities: { tools: { choice: ["auto", "none"], strict: true } },
};

const MODEL: Json = {
	id: "model-a",
	name: "Model A",
	description: "A model for cases.",
	modalities: { input: ["text", "image/png"], output: ["text"] },
	capabilities: {
		tools: { supported: true },
		reasoning: { supported: true, effort: ["low", "high"], toggle: true },
		structured_output: true,
		cache: { implicit: false, explicit: { read: 0.1 } },
	},
	limit: { context: 200_000, input: 190_000, output: 64_000 },
	cost: { text: { input: { price: 1.5 }, output: { price: 6 } } },
	api: { "chat-completions": API, messages: API },
};

type Holds = "text" | "number" | "boolean" | "object" | "list";

interface Site {
	path: string[];
	holds: Holds;
}

/** Every field that falls back, as the old schema had it. */
const SITES: Site[] = [
	{ path: ["name"], holds: "text" },
	{ path: ["description"], holds: "text" },
	{ path: ["modalities"], holds: "object" },
	{ path: ["modalities", "input"], holds: "list" },
	{ path: ["modalities", "output"], holds: "list" },
	{ path: ["capabilities"], holds: "object" },
	{ path: ["capabilities", "tools"], holds: "object" },
	{ path: ["capabilities", "tools", "supported"], holds: "boolean" },
	{ path: ["capabilities", "reasoning"], holds: "object" },
	{ path: ["capabilities", "reasoning", "supported"], holds: "boolean" },
	{ path: ["capabilities", "reasoning", "effort"], holds: "list" },
	{ path: ["capabilities", "reasoning", "toggle"], holds: "boolean" },
	{ path: ["capabilities", "structured_output"], holds: "boolean" },
	{ path: ["capabilities", "cache"], holds: "object" },
	{ path: ["capabilities", "cache", "implicit"], holds: "boolean" },
	{ path: ["limit"], holds: "object" },
	{ path: ["limit", "context"], holds: "number" },
	{ path: ["limit", "input"], holds: "number" },
	{ path: ["limit", "output"], holds: "number" },
	{ path: ["cost"], holds: "object" },
	{ path: ["cost", "text"], holds: "object" },
	{ path: ["api"], holds: "object" },
	{ path: ["api", "chat-completions", "compatibility"], holds: "object" },
	{ path: ["api", "chat-completions", "compatibility", "reasoning_field"], holds: "text" },
	{ path: ["api", "chat-completions", "capabilities"], holds: "object" },
	{ path: ["api", "chat-completions", "capabilities", "tools"], holds: "object" },
	{ path: ["api", "chat-completions", "capabilities", "tools", "choice"], holds: "list" },
	{ path: ["api", "chat-completions", "capabilities", "tools", "strict"], holds: "boolean" },
];

////////////////////////////////
//  Functions & Helpers

const decode = (name: string, body: unknown): CaseSource => ({
	kind: "catalog.opencodeModels",
	name,
	input: { body },
});

const data = (...items: Json[]): Json => ({ data: items });

function siteCases(): CaseSource[] {
	const cases: CaseSource[] = [];
	for (const site of SITES) {
		const name = `models-${site.path.join(".")}`;
		cases.push(decode(`${name}-omitted`, data(cut(MODEL, site.path))));
		cases.push(decode(`${name}-null`, data(setIn(MODEL, site.path, null))));
		cases.push(decode(`${name}-wrong-type`, data(setIn(MODEL, site.path, wrongFor(site.holds)))));
		if (site.holds === "list") cases.push(decode(`${name}-wrong-item`, data(setIn(MODEL, site.path, ["a", 3]))));
	}
	return cases;
}

const joinCase = (name: string, input: unknown): CaseSource => ({ kind: "catalog.opencodeJoin", name, input });

/** A model of a plan's list, with the facts a join reads. */
export function listed(id: string, extra: Record<string, unknown> = {}): Json {
	return {
		id,
		name: id,
		capabilities: { tools: { supported: true }, reasoning: { supported: true, effort: ["low", "high"] } },
		limit: { context: 100_000, output: 32_000 },
		cost: { text: { input: { price: 0.5 }, output: { price: 2 } } },
		api: { "chat-completions": {} },
		...extra,
	} as Json;
}

export const LIST_GO: Json = data(
	listed("gpt-6-luna", {
		limit: { context: 1_050_000, input: 922_000, output: 128_000 },
		cost: { text: { input: { price: 0.1 }, output: { price: 0.5 } } },
		capabilities: {
			tools: { supported: true },
			reasoning: { supported: true, effort: ["none", "low", "medium", "high", "xhigh", "max"] },
			structured_output: true,
		},
		api: {
			responses: { capabilities: { tools: { choice: ["auto", "none", "required", "tool"] } } },
		},
	}),
	listed("deepseek-v4.1-flash", {
		limit: { context: 1_000_000, output: 384_000 },
		cost: { text: { input: { price: 0.15 }, output: { price: 0.6 } } },
		capabilities: {
			tools: { supported: true },
			reasoning: { supported: true, effort: ["low", "high", "max"], toggle: true },
			structured_output: true,
		},
		api: {
			"chat-completions": {
				compatibility: { reasoning_field: "reasoning_content" },
				capabilities: { tools: { choice: ["auto", "none", "required", "tool"] } },
			},
		},
	}),
	listed("minimax-m3", {
		limit: { context: 1_000_000, output: 64_000 },
		capabilities: { tools: { supported: true } },
		api: { messages: { capabilities: { tools: { choice: ["auto", "none"], strict: true } } } },
	}),
	listed("grok-4.7", {
		limit: { context: 500_000, output: 500_000 },
		cost: { text: { input: { price: 2 }, output: { price: 6 } } },
		capabilities: {
			tools: { supported: true },
			reasoning: { supported: true, effort: ["low", "medium", "high", "xhigh"] },
			cache: { implicit: true },
		},
		api: { responses: {} },
	}),
	listed("claude-haiku-5-5", {
		limit: { context: 1_000_000, output: 128_000 },
		capabilities: {
			tools: { supported: true },
			reasoning: { supported: true, effort: ["low", "medium", "high", "xhigh", "max"] },
			cache: { explicit: { read: 0.1 } },
		},
		api: { messages: {} },
	}),
	{ id: "gemini-3.8-flash", capabilities: { tools: { supported: true } }, api: { "generate-content": {} } },
);

const ROWS_BOUND = [
	"openai/gpt-6-luna",
	"deepseek/deepseek-v4.1-flash",
	"minimax/minimax-m3",
	"x-ai/grok-4.7",
	"anthropic/claude-haiku-5.5",
	"google/gemini-3.8-flash",
	"mistralai/mistral-large-2512",
];

const pass = (use: string, zdrOnly: boolean) => ({ use, zdrOnly });

////////////////////////////////
//  Cases

export const catalogOpenCodeCases: CaseSource[] = [
	decode("models-full", data(MODEL)),
	decode("models-minimal", data({ id: "bare" })),
	...siteCases(),
	// The text prices need both numbers.
	decode("models-cost.text.input-missing", data(setIn(MODEL, ["cost", "text"], { output: { price: 6 } }))),
	decode("models-cost.text.input-price-missing", data(setIn(MODEL, ["cost", "text", "input"], {}))),
	decode("models-cost.text.input-price-wrong-type", data(setIn(MODEL, ["cost", "text", "input", "price"], "1.5"))),
	decode("models-cost.text.input-wrong-type", data(setIn(MODEL, ["cost", "text", "input"], 1.5))),
	decode("models-cost.text.output-price-wrong-type", data(setIn(MODEL, ["cost", "text", "output", "price"], null))),
	decode("models-cost.text.output-missing", data(setIn(MODEL, ["cost", "text"], { input: { price: 1.5 } }))),
	decode(
		"models-cost.text.free-price-zero",
		data(setIn(MODEL, ["cost", "text"], { input: { price: 0 }, output: { price: 0 } })),
	),
	// A cache is explicit when the listing holds an object or `true` for it.
	decode("models-cache.explicit-omitted", data(setIn(MODEL, ["capabilities", "cache"], { implicit: false }))),
	decode(
		"models-cache.explicit-null",
		data(setIn(MODEL, ["capabilities", "cache"], { implicit: false, explicit: null })),
	),
	decode("models-cache.explicit-true", data(setIn(MODEL, ["capabilities", "cache"], { explicit: true }))),
	decode("models-cache.explicit-empty-object", data(setIn(MODEL, ["capabilities", "cache"], { explicit: {} }))),
	decode("models-cache.explicit-list", data(setIn(MODEL, ["capabilities", "cache"], { explicit: [] }))),
	decode("models-cache.explicit-empty-text", data(setIn(MODEL, ["capabilities", "cache"], { explicit: "" }))),
	decode(
		"models-cache.explicit-false-beside-implicit-false",
		data(setIn(MODEL, ["capabilities", "cache"], { implicit: false, explicit: false })),
	),
	decode("models-cache.explicit-false", data(setIn(MODEL, ["capabilities", "cache"], { explicit: false }))),
	decode("models-cache.explicit-zero", data(setIn(MODEL, ["capabilities", "cache"], { explicit: 0 }))),
	decode("models-cache.explicit-text", data(setIn(MODEL, ["capabilities", "cache"], { explicit: "yes" }))),
	decode("models-cache.empty", data(setIn(MODEL, ["capabilities", "cache"], {}))),
	// The API record: a family that is not an object falls back to no facts.
	decode("models-api.family-null", data(setIn(MODEL, ["api"], { "chat-completions": null }))),
	decode("models-api.family-text", data(setIn(MODEL, ["api"], { responses: "x" }))),
	decode("models-api.family-list", data(setIn(MODEL, ["api"], { messages: [API] }))),
	decode("models-api.family-number", data(setIn(MODEL, ["api"], { "generate-content": 3 }))),
	decode("models-api.family-empty", data(setIn(MODEL, ["api"], { responses: {} }))),
	decode(
		"models-api.families-in-the-order-of-the-four",
		data(
			setIn(MODEL, ["api"], {
				"generate-content": { capabilities: { tools: { strict: true } } },
				messages: {},
				responses: {},
				"chat-completions": {},
			}),
		),
	),
	decode("models-api.unknown-family-ignored", data(setIn(MODEL, ["api"], { embeddings: API, other: "x" }))),
	decode("models-api.empty-record", data(setIn(MODEL, ["api"], {}))),
	decode("models-api.list", data(setIn(MODEL, ["api"], [API]))),
	decode("models-api.proto-key-ignored", data(setIn(MODEL, ["api"], JSON.parse('{"__proto__":{},"responses":{}}')))),
	// The item and the body.
	decode("models-id-missing", data(cut(MODEL, ["id"]))),
	decode("models-id-null", data(setIn(MODEL, ["id"], null))),
	decode("models-id-number", data(setIn(MODEL, ["id"], 7))),
	decode("models-item-null", data(null, MODEL)),
	decode("models-item-text", data("model-a", MODEL)),
	decode("models-item-list", data([MODEL], MODEL)),
	decode("models-bad-items-between-good", data({ id: "one" }, { unexpected: true }, { id: "two" })),
	decode(
		"models-same-id-twice-later-replaces-in-place",
		data({ id: "one", name: "First" }, { id: "two" }, { id: "one", name: "Second" }),
	),
	decode("models-unknown-keys-dropped", data({ ...(MODEL as object), extra: 1, canonical_id: "lab/model-a" })),
	decode("models-empty-data", data()),
	decode("models-body-null", null),
	decode("models-body-text", "data"),
	decode("models-body-list", [MODEL]),
	decode("models-body-without-data", { models: [MODEL] }),
	decode("models-data-null", { data: null }),
	decode("models-data-object", { data: MODEL }),

	// The join of a plan's list to rows.
	joinCase("join-go-base-prices-context-efforts-family-and-cache", {
		rows: ROWS_BOUND,
		plan: "go",
		models: LIST_GO,
		nowMs: NOW,
		passes: [pass("chat", false), pass("analysis", false)],
	}),
	joinCase("join-go-zdr-only-refuses-routes-that-keep-data", {
		rows: ROWS_BOUND,
		plan: "go",
		models: LIST_GO,
		nowMs: NOW,
		passes: [pass("chat", true)],
	}),
	joinCase("join-payg-zdr-only", {
		rows: ROWS_BOUND,
		plan: "payg",
		models: LIST_GO,
		nowMs: NOW,
		passes: [pass("chat", true), pass("chat", false)],
	}),
	joinCase("join-deepseek-zdr-ends-after-its-day", {
		rows: ["deepseek/deepseek-v4.1-flash"],
		plan: "go",
		models: LIST_GO,
		nowMs: NOW + 66 * DAY,
		passes: [pass("chat", true)],
	}),
	joinCase("join-deepseek-zdr-on-its-last-instant", {
		rows: ["deepseek/deepseek-v4.1-flash"],
		plan: "go",
		models: LIST_GO,
		nowMs: Date.UTC(2026, 11, 1) - 1,
		passes: [pass("chat", true)],
	}),
	joinCase("join-deepseek-zdr-after-its-last-instant", {
		rows: ["deepseek/deepseek-v4.1-flash"],
		plan: "go",
		models: LIST_GO,
		nowMs: Date.UTC(2026, 11, 1),
		passes: [pass("chat", true)],
	}),
	joinCase("join-never-loaded-keeps-retention", {
		rows: ROWS_BOUND,
		plan: "go",
		models: null,
		nowMs: NOW,
		passes: [pass("chat", true), pass("chat", false)],
	}),
	joinCase("join-never-loaded-payg", {
		rows: ROWS_BOUND,
		plan: "payg",
		models: { error: "down" },
		nowMs: NOW,
		passes: [pass("chat", true), pass("chat", false)],
	}),
	joinCase("join-row-without-binding-left-out", {
		rows: ["mistralai/mistral-large-2512", "google/gemma-4-26b-a4b-it"],
		plan: "go",
		models: LIST_GO,
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-model-not-listed-leaves-row-out", {
		rows: ["openai/gpt-6.1-sol", "openai/gpt-6-luna"],
		plan: "go",
		models: LIST_GO,
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-model-without-api-leaves-row-out", {
		rows: ["openai/gpt-6-luna"],
		plan: "go",
		models: data(listed("gpt-6-luna", { api: {} })),
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-model-without-tools-leaves-row-out", {
		rows: ["openai/gpt-6-luna"],
		plan: "go",
		models: data(listed("gpt-6-luna", { capabilities: { tools: { supported: false } } })),
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-row-without-reasoning-offers-no-efforts", {
		rows: ["minimax/minimax-m3"],
		plan: "payg",
		models: data(listed("minimax-m3")),
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-input-limit-wins-over-context", {
		rows: ["openai/gpt-6-luna"],
		plan: "go",
		models: data(listed("gpt-6-luna", { limit: { context: 10, input: 5, output: 3 } })),
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-no-limits-and-no-cost", {
		rows: ["openai/gpt-6-luna"],
		plan: "go",
		models: data(listed("gpt-6-luna", { limit: null, cost: null })),
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-api-facts-structured-choice-strict-and-thought-field", {
		rows: ["deepseek/deepseek-v4.1-flash", "minimax/minimax-m3"],
		plan: "payg",
		models: data(
			listed("deepseek-v4.1-flash", {
				capabilities: {
					tools: { supported: true },
					reasoning: { supported: true, effort: ["low"], toggle: true },
					structured_output: true,
				},
				api: {
					"chat-completions": {
						compatibility: { reasoning_field: "reasoning_content" },
						capabilities: { tools: { choice: ["auto", "required"], strict: true } },
					},
				},
			}),
			listed("minimax-m3", {
				capabilities: { tools: { supported: true }, reasoning: { toggle: false } },
				api: { "chat-completions": { capabilities: { tools: {} } } },
			}),
		),
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-first-listed-family-in-the-order-of-the-four", {
		rows: ["openai/gpt-6-luna"],
		plan: "go",
		models: data(listed("gpt-6-luna", { api: { "generate-content": {}, messages: {}, responses: {} } })),
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-cache-by-family-when-unlisted", {
		rows: ["openai/gpt-6-luna", "anthropic/claude-haiku-5.5"],
		plan: "go",
		models: data(
			listed("gpt-6-luna", { api: { messages: {} } }),
			listed("claude-haiku-5-5", { api: { responses: {} } }),
		),
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-cache-listed-explicit-without-implicit", {
		rows: ["openai/gpt-6-luna"],
		plan: "go",
		models: data(
			listed("gpt-6-luna", {
				capabilities: { tools: { supported: true }, cache: { explicit: { read: 1 } } },
			}),
		),
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-cache-listed-explicit-with-implicit", {
		rows: ["openai/gpt-6-luna"],
		plan: "go",
		models: data(
			listed("gpt-6-luna", {
				capabilities: { tools: { supported: true }, cache: { implicit: true, explicit: { read: 1 } } },
			}),
		),
		nowMs: NOW,
		passes: [pass("chat", false)],
	}),
	joinCase("join-made-up-hosted-row", {
		rows: ["opencode:glm-5.3-flash", "opencode:grok-4.7", "opencode:gone"],
		plan: "go",
		models: data(listed("glm-5.3-flash"), listed("grok-4.7")),
		nowMs: NOW,
		passes: [pass("chat", true), pass("chat", false)],
	}),
];
