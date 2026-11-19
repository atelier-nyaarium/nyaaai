// Catalog cases on a scripted network and the timer wheel. Every value is made up.
//
// A script lists exchanges (what the network answers, in the order a catalog asks)
// and steps (calls, time passing, the registry's rows changing).

import type { CaseSource } from "../caseAdapter";
import { failing, HOUR, type Json, NOW, refused, reply } from "./catalogKit";

////////////////////////////////
//  Fixtures

const LUNA = "openai/gpt-6-luna";
const OPUS = "anthropic/claude-opus-5.5";
const LARGE = "mistralai/mistral-large-2512";

const PARAMS = ["tools", "reasoning", "response_format", "structured_outputs"];

const MODELS_BODY: Json = {
	data: [
		{
			id: LUNA,
			context_length: 1_000_000,
			pricing: { prompt: "0.00000012", completion: "0.0000006" },
			reasoning: { supported_efforts: ["high", "low"] },
		},
		{ id: OPUS, pricing: { prompt: "0.000005", completion: "0.000025" } },
		{ id: LARGE },
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
		{ model_id: LARGE, tag: "gamma", supported_parameters: ["tools"] },
	],
};

const MODELS = reply(MODELS_BODY);
const ZDR = reply(ZDR_BODY);

const PAIR = [MODELS, ZDR];

/** Both of a refresh fail. */
const FAILING_PAIR = [failing(), failing()];

const ROWS = [LUNA, OPUS, LARGE];

const views = (extra: Record<string, unknown> = {}) => ({ kind: "call", call: { kind: "views", ...extra } });

const call = (body: Record<string, unknown>) => ({ kind: "call", call: body });

const advance = (ms: number) => ({ kind: "advance", ms });

const run = (name: string, input: Record<string, unknown>): CaseSource => ({
	kind: "catalog.openrouterRun",
	name,
	input: { startMs: NOW, rows: ROWS, ...input },
});

const oc = (name: string, input: Record<string, unknown>): CaseSource => ({
	kind: "catalog.opencodeRun",
	name,
	input: { startMs: NOW, rows: ["openai/gpt-6-luna", "deepseek/deepseek-v4.1-flash"], plan: "go", ...input },
});

/** A plan's list. */
const LIST: Json = {
	data: [
		{
			id: "gpt-6-luna",
			name: "Luna",
			capabilities: { tools: { supported: true }, reasoning: { effort: ["low", "high"] } },
			limit: { context: 1_000_000, output: 100_000 },
			cost: { text: { input: { price: 0.1 }, output: { price: 0.5 } } },
			api: { responses: {} },
		},
		{
			id: "deepseek-v4.1-flash",
			capabilities: { tools: { supported: true } },
			api: { "chat-completions": {} },
		},
	],
};

const LISTED = reply(LIST);

////////////////////////////////
//  Cases

export const catalogRunCases: CaseSource[] = [
	run("run-shares-one-fetch-and-keeps-it-12-hours", {
		exchanges: [...PAIR, ...PAIR],
		steps: [
			{ kind: "together", calls: [{ kind: "views" }, { kind: "view", key: LUNA }] },
			advance(11 * HOUR),
			views(),
			advance(HOUR + 1),
			views(),
		],
	}),
	run("run-ttl-override", {
		ttlMs: 1000,
		exchanges: [...PAIR, ...PAIR],
		steps: [views(), advance(999), views(), advance(1), views()],
	}),
	run("run-removal-inside-the-cache-reads-retired-without-a-fetch", {
		exchanges: PAIR.map((held, at) =>
			at === 0
				? reply({ data: [{ id: LUNA, expiration_date: "2026-09-25T13:00:00Z" }, { id: OPUS }, { id: LARGE }] })
				: held,
		),
		steps: [
			{ kind: "call", call: { kind: "view", key: LUNA } },
			advance(HOUR - 1),
			{ kind: "call", call: { kind: "view", key: LUNA } },
			advance(1),
			{ kind: "call", call: { kind: "view", key: LUNA } },
		],
	}),
	run("run-new-rows-rejoin-without-a-fetch", {
		exchanges: PAIR,
		steps: [views(), views(), { kind: "rows", rows: [LUNA] }, views(), views()],
	}),
	run("run-registry-tells-the-same-rows-rejoins", {
		exchanges: PAIR,
		steps: [views(), { kind: "rows", rows: ROWS }, views(), views()],
	}),
	run("run-each-use-keeps-its-own-join", {
		exchanges: PAIR,
		steps: [
			views({ use: "chat" }),
			views({ use: "analysis" }),
			views({ use: "chat" }),
			views({ use: "analysis" }),
			views({ use: "compaction" }),
			views({ use: "chat" }),
		],
	}),
	run("run-a-different-flag-rejoins", {
		exchanges: PAIR,
		steps: [
			views({ zdrOnly: true }),
			views({ zdrOnly: true }),
			views({ zdrOnly: false }),
			views({ zdrOnly: false }),
			views({ zdrOnly: true }),
			views(),
		],
	}),
	run("run-refetched-data-rejoins-and-the-same-data-does-not", {
		exchanges: [...PAIR, ...PAIR],
		steps: [views(), advance(13 * HOUR), views(), views()],
	}),
	run("run-failed-refresh-keeps-the-last-good-views", {
		exchanges: [...FAILING_PAIR, ...PAIR, ...FAILING_PAIR],
		steps: [
			{ kind: "call", call: { kind: "view", key: LUNA } },
			advance(HOUR),
			{ kind: "call", call: { kind: "view", key: LUNA } },
			advance(13 * HOUR),
			{ kind: "call", call: { kind: "view", key: LUNA } },
		],
	}),
	run("run-first-failure-leaves-every-row-unknown", {
		exchanges: FAILING_PAIR,
		steps: [views(), views()],
	}),
	run("run-failure-waits-sixty-seconds-before-a-new-try", {
		exchanges: [...FAILING_PAIR, ...PAIR],
		steps: [views(), advance(30_000), views(), advance(29_999), views(), advance(1), views()],
	}),
	run("run-one-failed-read-fails-the-refresh", {
		exchanges: [MODELS, failing()],
		steps: [views(), advance(60_000), views()],
	}),
	run("run-a-body-that-is-no-list-fails-the-refresh", {
		exchanges: [MODELS, reply({ error: "x" }), reply({ data: {} }), ZDR, reply([]), ZDR],
		steps: [views(), advance(60_000), views(), advance(60_000), views()],
	}),
	run("run-a-reply-that-is-no-json-fails-the-refresh", {
		exchanges: [{ status: 200, text: "<html>" }, ZDR],
		steps: [views()],
	}),
	run("run-a-refused-connection-fails-the-refresh", {
		exchanges: [refused(), refused()],
		steps: [views()],
	}),
	run("run-explicit-retry-reads-again-after-a-failure", {
		exchanges: [
			...PAIR,
			failing(),
			failing(),
			reply({ data: [...(MODELS_BODY as { data: Json[] }).data, { id: "lab/new" }] }),
			ZDR,
		],
		steps: [
			call({ kind: "listing" }),
			advance(13 * HOUR),
			call({ kind: "listing" }),
			call({ kind: "listing", retry: true }),
		],
	}),
	run("run-explicit-retry-inside-the-window-reads-again", {
		exchanges: [...FAILING_PAIR, ...PAIR],
		steps: [call({ kind: "listing" }), call({ kind: "listing" }), call({ kind: "listing", retry: true })],
	}),
	run("run-a-slow-fetch-ends-at-the-deadline", {
		exchanges: [{ ...MODELS, headMs: 31_000 }, ...PAIR],
		steps: [views(), advance(60_000), views()],
	}),
	run("run-a-fetch-just-inside-the-deadline-lands", {
		exchanges: [{ ...MODELS, headMs: 29_999 }, ZDR],
		steps: [views()],
	}),
	run("run-the-deadline-of-one-read-covers-both", {
		exchanges: [
			{ ...MODELS, headMs: 20_000 },
			{ ...ZDR, headMs: 20_000 },
		],
		steps: [views()],
	}),
	run("run-a-callers-abort-leaves-the-refresh-running", {
		exchanges: [{ ...MODELS, headMs: 10 }, ZDR],
		steps: [views({ abortAfterMs: 5 }), views(), views()],
	}),
	run("run-a-caller-aborted-before-the-call-still-starts-the-refresh", {
		exchanges: PAIR,
		steps: [views({ abortedBefore: true }), views()],
	}),
	run("run-one-caller-aborts-the-other-gets-the-views", {
		exchanges: [{ ...MODELS, headMs: 10 }, ZDR],
		steps: [{ kind: "together", calls: [{ kind: "views", abortAfterMs: 3 }, { kind: "views" }] }],
	}),
	run("run-abort-of-a-cached-read", {
		exchanges: PAIR,
		steps: [views(), views({ abortedBefore: true }), call({ kind: "entry", id: LUNA, abortedBefore: true })],
	}),
	run("run-refreshed-is-told-after-each-success", {
		exchanges: [...PAIR, ...FAILING_PAIR, ...PAIR],
		steps: [views(), advance(13 * HOUR), views(), advance(60_000), views()],
	}),
	run("run-entry-view-for-and-listing", {
		exchanges: PAIR,
		steps: [
			call({ kind: "entry", id: LUNA }),
			call({ kind: "entry", id: "lab/none" }),
			call({ kind: "viewFor", row: LUNA }),
			call({ kind: "viewFor", row: "openrouter:lab/made-up" }),
			call({ kind: "viewFor", row: LARGE, use: "analysis" }),
			call({ kind: "listing" }),
			call({ kind: "view", key: "lab/none" }),
		],
	}),
	run("run-view-for-without-data-is-unknown", {
		exchanges: FAILING_PAIR,
		steps: [call({ kind: "viewFor", row: LUNA }), call({ kind: "entry", id: LUNA }), call({ kind: "listing" })],
	}),

	// OpenCode.
	oc("run-opencode-shares-one-fetch-and-keeps-it-12-hours", {
		exchanges: [LISTED, LISTED],
		steps: [
			{ kind: "together", calls: [{ kind: "views" }, { kind: "models" }] },
			advance(11 * HOUR),
			views(),
			advance(HOUR + 1),
			views(),
		],
	}),
	oc("run-opencode-ttl-override", {
		ttlMs: 1000,
		exchanges: [LISTED, LISTED],
		steps: [views(), advance(999), views(), advance(1), views()],
	}),
	oc("run-opencode-reads-the-rows-at-every-call", {
		exchanges: [LISTED],
		steps: [views(), views(), { kind: "rows", rows: ["openai/gpt-6-luna"] }, views(), views({ zdrOnly: false })],
	}),
	oc("run-opencode-first-failure-keeps-retention", {
		plan: "payg",
		rows: ["openai/gpt-6-luna", "deepseek/deepseek-v4.1-flash", "minimax/minimax-m3"],
		exchanges: [failing()],
		steps: [views({ zdrOnly: true }), views({ zdrOnly: false })],
	}),
	oc("run-opencode-failed-refresh-keeps-the-last-list", {
		exchanges: [failing(), LISTED, failing()],
		steps: [views(), advance(HOUR), views(), advance(13 * HOUR), views()],
	}),
	oc("run-opencode-failure-waits-sixty-seconds", {
		exchanges: [failing(), LISTED],
		steps: [views(), advance(59_999), views(), advance(1), views()],
	}),
	oc("run-opencode-fresh-retries-inside-the-window", {
		exchanges: [failing(), LISTED],
		steps: [call({ kind: "models" }), call({ kind: "models" }), call({ kind: "models", fresh: true })],
	}),
	oc("run-opencode-a-body-that-is-no-list-fails", {
		exchanges: [reply({ error: "x" }), reply([]), reply({ data: null }), LISTED],
		steps: [views(), advance(60_000), views(), advance(60_000), views(), advance(60_000), views()],
	}),
	oc("run-opencode-a-slow-fetch-ends-at-the-deadline", {
		exchanges: [{ ...LISTED, headMs: 30_000 }, LISTED],
		steps: [views(), advance(60_000), views()],
	}),
	oc("run-opencode-a-fetch-just-inside-the-deadline-lands", {
		exchanges: [{ ...LISTED, headMs: 29_999 }],
		steps: [views()],
	}),
	oc("run-opencode-a-callers-abort-leaves-the-refresh-running", {
		exchanges: [{ ...LISTED, headMs: 10 }],
		steps: [views({ abortAfterMs: 5 }), views(), call({ kind: "models", abortedBefore: true })],
	}),
	oc("run-opencode-refreshed-is-told-after-each-success", {
		exchanges: [LISTED, failing(), LISTED],
		steps: [views(), advance(13 * HOUR), views(), advance(60_000), views()],
	}),
	oc("run-opencode-view-for", {
		rows: ["openai/gpt-6-luna"],
		exchanges: [LISTED],
		steps: [
			call({ kind: "viewFor", row: "openai/gpt-6-luna" }),
			call({ kind: "viewFor", row: "opencode:gone" }),
			call({ kind: "viewFor", row: "opencode:deepseek-v4.1-flash" }),
			call({ kind: "viewFor", row: "mistralai/mistral-large-2512" }),
		],
	}),
	oc("run-opencode-view-for-without-a-list-is-unknown", {
		exchanges: [failing()],
		steps: [call({ kind: "viewFor", row: "openai/gpt-6-luna" })],
	}),

	// Both reads settle before a refresh fails.
	run("run-a-deadline-that-ends-the-first-read-ends-the-second-before-its-request", {
		exchanges: [{ ...MODELS, headMs: 40_000 }, ...PAIR],
		steps: [views(), advance(60_000), views()],
	}),
	run("run-a-failed-first-read-waits-for-the-slow-second-one", {
		exchanges: [failing(), { ...ZDR, headMs: 20_000 }, ...PAIR],
		steps: [views(), advance(59_999), views(), advance(1), views()],
	}),
	run("run-a-failed-first-read-and-a-second-that-hits-the-deadline", {
		exchanges: [failing(), { ...ZDR, headMs: 40_000 }, ...PAIR],
		steps: [views(), advance(60_000), views()],
	}),
	run("run-a-failed-second-read-beside-a-good-first-one", {
		exchanges: [MODELS, failing(), ...PAIR],
		steps: [views(), advance(60_000), views()],
	}),
	run("run-both-reads-fail", {
		exchanges: [failing(404), failing(500), ...PAIR],
		steps: [views(), advance(60_000), views()],
	}),
	run("run-a-read-that-does-not-parse-fails-after-both-settle", {
		exchanges: [reply({ data: "x" }), ZDR, ...PAIR],
		steps: [views(), advance(60_000), views()],
	}),
	// New data is stored before anyone is told.
	run("run-refreshed-reads-the-new-data", {
		exchanges: [...PAIR, ...PAIR],
		onRefreshed: "reads",
		steps: [views(), advance(13 * HOUR), views()],
	}),
	run("run-refreshed-that-throws-changes-nothing", {
		exchanges: [...PAIR, ...PAIR],
		onRefreshed: "readsAndThrows",
		steps: [views(), views(), advance(13 * HOUR), views()],
	}),
	oc("run-opencode-refreshed-reads-the-new-data", {
		exchanges: [LISTED, LISTED],
		onRefreshed: "reads",
		steps: [views(), advance(13 * HOUR), views()],
	}),
	oc("run-opencode-refreshed-that-throws-changes-nothing", {
		exchanges: [LISTED, LISTED],
		onRefreshed: "readsAndThrows",
		steps: [views(), views(), advance(13 * HOUR), views()],
	}),
];
