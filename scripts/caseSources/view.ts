// Model view cases.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Helpers

const LADDER = ["none", "minimal", "low", "medium", "high", "xhigh", "max"];

const STATUSES = ["ok", "retired", "noZdrHost", "noTools", "unknown"];

interface Spec {
	reasoning: boolean;
	status: string;
	efforts: string[];
	outputLimit?: number;
	price?: unknown;
	api?: unknown;
}

const spec = (over: Partial<Spec> = {}): Spec => ({ reasoning: true, status: "ok", efforts: [], ...over });

const API = {
	family: "responses",
	structured: true,
	choice: ["auto", "required"],
	strict: true,
	reasoning: "reasoning_content",
	toggle: true,
};

const TIERS = [
	{ minPromptTokens: 0, prompt: 1, completion: 2 },
	{ minPromptTokens: 200000, prompt: 2, completion: 4, cacheRead: 0.5, cacheWrite: 2.5, cacheWrite1h: 4 },
];

const PRICE = { prompt: [0.5, 1], completion: [1, 2], tiers: TIERS };

const efforts = (name: string, rows: Array<{ view: Spec | null; wanted: string | null }>): CaseSource => ({
	kind: "view.runEffort",
	name,
	input: { rows },
});

/** A view offering these efforts, asked for every rung. */
const sweep = (name: string, view: Spec): CaseSource =>
	efforts(
		name,
		[...LADDER, null].map((wanted) => ({ view, wanted })),
	);

////////////////////////////////
//  Cases

export const viewCases: CaseSource[] = [
	{ kind: "view.unknown", name: "reasoning-row", input: { keys: ["openai/gpt-6-luna"] } },
	{ kind: "view.unknown", name: "plain-row", input: { keys: ["google/gemma-4-26b-a4b-it", "moonshotai/kimi-k3"] } },

	{
		kind: "view.tiers",
		name: "absent-and-unpriced",
		input: { rows: [{ view: null }, { view: {} }, { view: { price: "free" } }, { view: { price: "unknown" } }] },
	},
	{
		kind: "view.tiers",
		name: "priced",
		input: {
			rows: [
				{ view: { price: { prompt: [0, 0], completion: [0, 0], tiers: [] } } },
				{ view: { price: PRICE } },
				{ view: { price: { ...PRICE, tiers: [TIERS[1]] } } },
			],
		},
	},

	{
		kind: "view.codecFacts",
		name: "absent",
		input: { views: [null] },
	},
	{
		kind: "view.codecFacts",
		name: "no-api-reads-as-none",
		input: { views: [spec(), spec({ efforts: ["low", "high"] }), spec({ status: "unknown", reasoning: false })] },
	},
	{
		kind: "view.codecFacts",
		name: "api-kept",
		input: {
			views: [
				spec({ api: API }),
				spec({ api: { family: "messages", structured: false, choice: [], strict: false } }),
			],
		},
	},
	{
		kind: "view.codecFacts",
		name: "output-limit",
		input: {
			views: [
				spec({ outputLimit: 0 }),
				spec({ outputLimit: 8192 }),
				spec({ outputLimit: 128000, efforts: ["medium"] }),
			],
		},
	},
	{
		kind: "view.codecFacts",
		name: "price-free-only",
		input: {
			views: [
				spec({ price: "free" }),
				spec({ price: "unknown" }),
				spec({ price: PRICE }),
				spec({ price: "free", outputLimit: 100, api: API }),
			],
		},
	},
	{
		kind: "view.codecFacts",
		name: "statuses-do-not-matter",
		input: { views: STATUSES.map((status) => spec({ status, efforts: ["low"], outputLimit: 10 })) },
	},

	efforts("absent-view", [
		{ view: null, wanted: "xhigh" },
		{ view: null, wanted: null },
	]),
	efforts("absent-wish", [
		{ view: spec({ efforts: ["low", "high", "xhigh"] }), wanted: null },
		{ view: spec({ status: "unknown" }), wanted: null },
	]),
	sweep("offers-low-high-xhigh", spec({ efforts: ["low", "high", "xhigh"] })),
	sweep("offers-whole-ladder", spec({ efforts: LADDER })),
	sweep("offers-nothing", spec()),
	sweep("offers-only-max", spec({ efforts: ["max"] })),
	sweep("offers-low-high", spec({ status: "retired", efforts: ["low", "high"] })),
	sweep("offers-medium", spec({ efforts: ["medium"], reasoning: false })),
	...STATUSES.map((status) =>
		sweep(`status-${status}-reasoning`, spec({ status, efforts: ["low", "high"], reasoning: true })),
	),
	...STATUSES.map((status) =>
		sweep(`status-${status}-plain`, spec({ status, efforts: ["low", "high"], reasoning: false })),
	),
	sweep("unknown-reasoning-passes-the-wish", spec({ status: "unknown", reasoning: true })),
	sweep("unknown-plain-sends-none", spec({ status: "unknown", reasoning: false })),
	{
		kind: "view.outputLimit",
		name: "limits",
		input: {
			texts: [
				"128000",
				"8192",
				"4096.7",
				"1.5",
				"1",
				"0.999",
				"0",
				"-0",
				"-5",
				"-0.5",
				"1e30",
				"9007199254740991",
				"9007199254740993",
				"1e999",
				"-1e999",
			],
		},
	},
	{ kind: "view.outputLimit", name: "fraction", input: { texts: ["4096.7"] } },
	{ kind: "view.outputLimit", name: "overflow", input: { texts: ["1e999"] } },
];
