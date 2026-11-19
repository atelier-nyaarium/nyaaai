// A kind's route: its transport, catalog, browser and tester as the kind wires them. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { HOUR, type Json, NOW, reply } from "./catalogKit";
import { LIST_GO } from "./catalogOpenCode";

////////////////////////////////
//  Fixtures

const LUNA = "openai/gpt-6-luna";
const OPUS = "anthropic/claude-opus-5.5";

const PARAMS = ["tools", "reasoning", "response_format", "structured_outputs"];

const MODELS_BODY: Json = {
	data: [
		{
			id: LUNA,
			name: "Luna",
			context_length: 1_000_000,
			pricing: { prompt: "0.00000012", completion: "0.0000006" },
			reasoning: { supported_efforts: ["high", "low"] },
		},
		{ id: OPUS, pricing: { prompt: "0.000005", completion: "0.000025" } },
	],
};

const ZDR_BODY: Json = {
	data: [
		{
			model_id: LUNA,
			tag: "alpha",
			supported_parameters: PARAMS,
			pricing: { prompt: "0.00000012", completion: "0.0000006" },
		},
		{ model_id: OPUS, tag: "beta", supported_parameters: PARAMS },
	],
};

const DETAIL_BODY: Json = {
	data: {
		endpoints: [
			{
				provider_name: "Alpha",
				tag: "alpha",
				context_length: 131_000,
				pricing: { prompt: "0.00000012", completion: "0.0000006" },
			},
		],
	},
};

const PAIR = [reply(MODELS_BODY), reply(ZDR_BODY)];

////////////////////////////////
//  Functions & Helpers

const route = (name: string, kind: string, input: Record<string, unknown>): CaseSource => ({
	kind: "kind.route",
	name,
	input: { kind, entry: { label: "Zen" }, startMs: NOW, exchanges: [], rows: [LUNA, OPUS], ...input },
});

const views = (extra: Record<string, unknown> = {}) => ({ kind: "views", ...extra });
const list = (extra: Record<string, unknown> = {}) => ({ kind: "list", ...extra });
const detail = (id: string) => ({ kind: "detail", id });
const listed = (id: string) => ({ kind: "listed", id });
const view = (row: string, use?: string) => ({ kind: "view", row, ...(use === undefined ? {} : { use }) });
const advance = (ms: number) => ({ kind: "advance", ms });

////////////////////////////////
//  Cases

export const kindRouteCases: CaseSource[] = [
	// OpenRouter.
	route("openrouter-views-join-the-two-lists", "openrouter", { exchanges: PAIR, steps: [views()] }),
	route("openrouter-views-per-use-and-zero-retention", "openrouter", {
		exchanges: PAIR,
		steps: [views({ use: "compaction" }), views({ zdrOnly: false }), views({ use: "analysis", zdrOnly: false })],
	}),
	route("openrouter-the-browser-and-the-catalog-share-one-fetch", "openrouter", {
		exchanges: PAIR,
		steps: [views(), list(), advance(12 * HOUR - 1), list(), advance(1), list()],
	}),
	route("openrouter-detail-reads-the-endpoints-of-a-model", "openrouter", {
		exchanges: [...PAIR, reply(DETAIL_BODY)],
		steps: [detail(LUNA)],
	}),
	route("openrouter-the-tester-names-a-model-and-views-a-row", "openrouter", {
		exchanges: PAIR,
		steps: [listed(LUNA), view(LUNA), listed("lab/none")],
	}),
	route("openrouter-a-failing-catalog-leaves-views-unknown", "openrouter", {
		exchanges: [
			{ status: 500, json: {} },
			{ status: 500, json: {} },
		],
		steps: [views(), list()],
	}),
	route("openrouter-the-removal-day-is-read-from-the-scheduler", "openrouter", {
		exchanges: [
			reply({ data: [{ id: LUNA, expiration_date: "2026-09-26" }] }),
			reply(ZDR_BODY),
			reply({ data: [{ id: LUNA, expiration_date: "2026-09-26" }] }),
			reply(ZDR_BODY),
		],
		steps: [list(), advance(86_400_000), list({ fresh: true })],
	}),

	// OpenCode.
	route("opencode-views-by-plan-go", "opencode", {
		entry: { label: "Zen", plan: "go" },
		exchanges: [reply(LIST_GO)],
		steps: [views(), views({ zdrOnly: false })],
	}),
	route("opencode-views-by-plan-payg", "opencode", {
		entry: { label: "Zen", plan: "payg" },
		exchanges: [reply(LIST_GO)],
		steps: [views(), views({ zdrOnly: false })],
	}),
	route("opencode-no-plan-is-go", "opencode", { exchanges: [reply(LIST_GO)], steps: [views(), list()] }),
	route("opencode-the-browser-carries-the-plan", "opencode", {
		entry: { label: "Zen", plan: "payg" },
		exchanges: [reply(LIST_GO)],
		steps: [list(), detail("gpt-6-luna"), detail("none")],
	}),
	route("opencode-the-tester-names-a-model-and-views-a-row", "opencode", {
		exchanges: [reply(LIST_GO)],
		steps: [listed("gpt-6-luna"), view(LUNA), listed("none")],
	}),
	route("opencode-a-failing-list-leaves-views-unknown", "opencode", {
		exchanges: [{ status: 500, json: {} }],
		steps: [views(), list()],
	}),
	route("opencode-the-list-is-kept-twelve-hours", "opencode", {
		exchanges: [reply(LIST_GO), reply(LIST_GO)],
		steps: [list(), advance(12 * HOUR - 1), list(), advance(1), list()],
	}),
];
