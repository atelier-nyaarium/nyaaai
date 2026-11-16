// Codec cases: the plan's host, seen through each native family.

import type { CaseSource } from "../caseAdapter";
import type { Json } from "./codecFixtures";
import { completes, fails } from "./codecKit";
import { gemini, messages, NOW, responses } from "./codecNativeFixtures";

////////////////////////////////
//  Fixtures

type Family = { name: string; codec: Json; reply: (usage: Json) => Json };

/** A reply holding ten input tokens and five output tokens, plus the usage fragment. */
const FAMILIES: Family[] = [
	{
		name: "responses",
		codec: responses(),
		reply: (usage) => ({
			id: "resp_1",
			status: "completed",
			output: [],
			usage: { input_tokens: 10, output_tokens: 5, ...usage },
		}),
	},
	{
		name: "messages",
		codec: messages(),
		reply: (usage) => ({
			id: "msg_1",
			content: [],
			stop_reason: "end_turn",
			usage: { input_tokens: 10, output_tokens: 5, ...usage },
		}),
	},
	{
		name: "gemini",
		codec: gemini(),
		reply: (usage) => ({
			candidates: [{ finishReason: "STOP" }],
			responseId: "resp_g",
			usageMetadata: { promptTokenCount: 10, candidatesTokenCount: 5, ...usage },
		}),
	},
];

const block = (over: Json): Json => ({ opencode: over });

const USAGES: Array<[string, Json]> = [
	["charge", { cost: 0.002 }],
	["charge-zero", { cost: 0 }],
	["charge-negative-is-dropped", { cost: -0.5 }],
	["charge-mistyped-is-dropped", { cost: "0.5" }],
	["source-free", block({ billing_source: "free" })],
	["source-balance", block({ billing_source: "balance" })],
	["source-go", block({ billing_source: "go" })],
	["source-go-plus", block({ billing_source: "go-plus" })],
	["source-byok", block({ billing_source: "byok" })],
	["source-unknown-is-dropped", block({ billing_source: "gift" })],
	["source-case-matters", block({ billing_source: "Go" })],
	["source-mistyped-is-dropped", block({ billing_source: 1 })],
	["list-cost", block({ list_cost: 0.01 })],
	["list-cost-negative-is-kept", block({ list_cost: -0.01 })],
	["list-cost-mistyped-is-dropped", block({ list_cost: "0.01" })],
	["allowance", block({ allowance: 0.25 })],
	["allowance-zero", block({ allowance: 0 })],
	["allowance-negative-is-dropped", block({ allowance: -0.25 })],
	["allowance-mistyped-is-dropped", block({ allowance: [1] })],
	["everything", { cost: 0.002, ...block({ billing_source: "go", list_cost: 0.01, allowance: 0.25 }) }],
	[
		"tokens-fill-what-the-api-did-not-say",
		block({ tokens: { cache_read: 50, reasoning: 3, cache_write_5m: 4, cache_write_1h: 6 } }),
	],
	["tokens-cache-write-one-size", block({ tokens: { cache_write_5m: 4 } })],
	["tokens-cache-write-other-size", block({ tokens: { cache_write_1h: 6 } })],
	[
		"tokens-mistyped-are-dropped",
		block({ tokens: { cache_read: "50", reasoning: null, cache_write_5m: [1], cache_write_1h: {} } }),
	],
	["tokens-not-an-object", block({ tokens: 5 })],
	["block-not-an-object", { opencode: "go" }],
	["block-is-a-list", { opencode: [{ billing_source: "go" }] }],
	["block-empty", block({})],
];

////////////////////////////////
//  Usage through every family

const usages: CaseSource[] = FAMILIES.flatMap((family) =>
	USAGES.map(([name, usage]) => completes(`host-${family.name}-usage-${name}`, family.codec, family.reply(usage))),
);

/** The API's own counts win over the token block. */
const OWN_COUNTS: Record<string, Json> = {
	responses: {
		input_tokens_details: { cached_tokens: 11, cache_write_tokens: 12 },
		output_tokens_details: { reasoning_tokens: 13 },
	},
	messages: { cache_read_input_tokens: 11, cache_creation_input_tokens: 12 },
	gemini: { cachedContentTokenCount: 11, thoughtsTokenCount: 13 },
};

const own: CaseSource[] = FAMILIES.map((family) =>
	completes(
		`host-${family.name}-usage-own-counts-win-over-the-token-block`,
		family.codec,
		family.reply({
			...OWN_COUNTS[family.name],
			...block({ tokens: { cache_read: 50, reasoning: 3, cache_write_5m: 4, cache_write_1h: 6 } }),
		}),
	),
);

////////////////////////////////
//  Limits and reset stamps

const BODY = (resetsAt: unknown, type = "GoUsageLimitError"): Json => ({
	type: "error",
	error: { type, message: "Limit reached.", resetsAt },
});

const SECONDS = NOW / 1000;

const STAMPS: Array<[string, unknown]> = [
	["seconds-ahead", SECONDS + 90],
	["seconds-behind", SECONDS - 90],
	["seconds-now", SECONDS],
	["milliseconds-ahead", NOW + 90_000],
	["milliseconds-behind", NOW - 90_000],
	["fractional-seconds", SECONDS + 90.0004],
	["iso-ahead", "2027-01-15T08:01:30Z"],
	["iso-behind", "2027-01-15T07:58:30Z"],
	["iso-with-an-offset", "2027-01-15T09:01:30+01:00"],
	["http-date-ahead", "Fri, 15 Jan 2027 08:01:30 GMT"],
	["text-seconds", String(SECONDS + 90)],
	["padded-text", " 2027-01-15T08:01:30Z "],
	["date-only", "2027-01-16"],
	["zero", 0],
	["negative", -5],
	["garbage-text", "soon"],
	["empty-text", ""],
	["boolean", true],
	["null", null],
];

const limits: CaseSource[] = [
	...STAMPS.flatMap(([name, stamp]) => [
		fails(`host-responses-429-allowance-reset-${name}`, responses({ nowMs: NOW }), 429, BODY(stamp)),
		fails(`host-responses-429-rate-limit-reset-${name}`, responses({ nowMs: NOW }), 429, {
			error: { message: "Slow.", resetsAt: stamp },
		}),
	]),
	fails("host-responses-429-allowance-reset-with-the-default-clock", responses(), 429, BODY(90)),
	fails("host-responses-429-rate-limit-reset-with-the-default-clock", responses(), 429, {
		error: { message: "Slow.", resetsAt: 90 },
	}),
	fails(
		"host-messages-429-allowance-reset-moves-with-the-clock",
		messages({ nowMs: NOW + 60_000 }),
		429,
		BODY(SECONDS + 90),
	),
	fails(
		"host-gemini-429-allowance-reset-moves-with-the-clock",
		gemini({ nowMs: NOW + 60_000 }),
		429,
		BODY(SECONDS + 90),
	),
	fails(
		"host-responses-429-retry-after-header-in-any-case",
		responses({ nowMs: NOW }),
		429,
		{ error: { message: "Slow.", resetsAt: SECONDS + 90 } },
		[["RETRY-AFTER", "7"]],
	),
	fails(
		"host-responses-429-other-header-does-not-count",
		responses({ nowMs: NOW }),
		429,
		{ error: { message: "Slow.", resetsAt: SECONDS + 90 } },
		[["x-retry", "7"]],
	),
	fails("host-responses-500-with-a-reset-is-not-a-wait", responses({ nowMs: NOW }), 500, {
		error: { message: "Down.", resetsAt: SECONDS + 90 },
	}),
	fails("host-responses-403-limit-type-is-still-an-allowance", responses({ nowMs: NOW }), 403, BODY(SECONDS + 90)),
	fails("host-responses-429-overload-wins-over-the-limit-type", responses({ nowMs: NOW }), 429, {
		error: { type: "BudgetLimitError", message: "[overloaded] and spent", resetsAt: SECONDS + 90 },
	}),
	fails("host-responses-503-overload-marker-is-ignored-and-the-limit-type-wins", responses(), 503, {
		error: { type: "GoUsageLimitError", message: "[overloaded] x" },
	}),
];

////////////////////////////////
//  Cases

export const hostCases: CaseSource[] = [...usages, ...own, ...limits];
