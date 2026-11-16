// Codec cases: the Chat Completions codec and its three dialects.

import type { CaseSource } from "../caseAdapter";
import {
	call,
	comment,
	DEEPSEEK,
	DONE,
	data,
	finish,
	type Json,
	LUNA,
	local,
	NOW,
	OPENCODE_BARE,
	OPENROUTER,
	OPUS,
	opencode,
	reasoned,
	reasoning,
	reasoningContent,
	stepOf,
	text,
} from "./codecFixtures";
import { geminiCases } from "./codecGemini";
import { hostCases } from "./codecHost";
import { completes, encodes, fails, streams } from "./codecKit";
import { messagesCases } from "./codecMessages";
import { ALL_FIXTURES } from "./codecNativeFixtures";
import { responsesCases } from "./codecResponses";

////////////////////////////////
//  Functions & Helpers

const wireCall = (id: string, name: string, args: unknown): Json => ({
	id,
	type: "function",
	function: { name, arguments: args },
});

const TITLE_SCHEMA = {
	name: "title",
	json: { type: "object", properties: { title: { type: "string" } }, required: ["title"] },
};

const FINISH_TOOL = { name: "finish", description: "Finish.", parameters: { type: "object" } };

const DIVIDE = {
	kind: "thought",
	type: "reasoning.text",
	segments: [{ kind: "content", text: "Divide." }],
	index: 0,
};

/** A step that reads a file after a thought. */
function readStep(model: string, thought: Json = DIVIDE, host?: string): Json {
	return {
		role: "step",
		producer: { model, ...(host === undefined ? {} : { host }) },
		parts: [
			thought,
			{ kind: "call", index: 0, wireId: "call_a", name: "read_file", arguments: "{}" },
			{ kind: "result", index: 0, output: "source", isError: false },
		],
	};
}

////////////////////////////////
//  Conformance: each fixture, five runs

const conformance: CaseSource[] = ALL_FIXTURES.flatMap((fixture) => {
	const history = [{ role: "user", text: "Hi" }, stepOf(fixture.result, fixture.model)];
	const at = (suffix: string) => `conformance-${fixture.name}-${suffix}`;
	return [
		streams(at("stream"), fixture.codec, ...fixture.stream),
		completes(at("reply"), fixture.codec, fixture.reply),
		streams(at("cut"), fixture.codec, ...fixture.cut),
		streams(at("garbled"), fixture.codec, data("{not json")),
		streams(at("failing"), fixture.codec, ...fixture.failing),
		...fixture.failures.map(([status, body, category]) =>
			fails(at(`failure-${status}-${category}`), fixture.codec, status, body),
		),
		encodes(at("replay-own"), fixture.codec, { model: fixture.model, history }),
		encodes(at("replay-foreign"), fixture.codec, { model: fixture.foreign, history }),
	];
});

////////////////////////////////
//  Every dialect: stream decoding

const DIALECTS: Array<{ name: string; codec: Json; thought: Json[]; part: Json }> = [
	{
		name: "openrouter",
		codec: OPENROUTER,
		thought: [
			data(reasoned({ type: "reasoning.text", index: 0, text: "Think" })),
			data(reasoned({ type: "reasoning.text", index: 0, text: "ing." })),
		],
		part: DIVIDE,
	},
	{
		name: "opencode",
		codec: opencode({ efforts: ["low", "high", "max"] }),
		thought: [data(reasoningContent("Think")), data(reasoningContent("ing."))],
		part: DIVIDE,
	},
	{
		name: "local",
		codec: local({ replay: "reasoning" }),
		thought: [data(reasoning("Think")), data(reasoning("ing."))],
		part: DIVIDE,
	},
];

const everyDialect: CaseSource[] = DIALECTS.flatMap(({ name, codec, thought }) => {
	const at = (suffix: string) => `${name}-${suffix}`;
	const oneCall = (index: number, id: string, args = "") => data(call(index, { id, name: "read", args }));
	return [
		// Calls join by index: a late id, the first name, empty and malformed arguments.
		streams(
			at("calls-by-index"),
			codec,
			data(call(1, { name: "write", args: '{"pa' })),
			data(call(0, { name: "read" })),
			data(call(1, { id: "call_b", name: "ignored", args: 'th":"a"}' })),
			data(call(2, { id: "call_c", name: "broken", args: "{not json" })),
			data(call(0, { id: "call_a" })),
			data(finish("tool_calls")),
			DONE,
		),
		streams(at("call-index-fraction"), codec, oneCall(0.5, "call_a"), data(finish("tool_calls")), DONE),
		streams(at("call-index-negative"), codec, oneCall(-1, "call_a"), data(finish("tool_calls")), DONE),
		streams(
			at("call-two-ids-one-index"),
			codec,
			oneCall(0, "call_a", "{"),
			oneCall(0, "call_b", "}"),
			data(finish("tool_calls")),
			DONE,
		),
		// Thoughts precede text; role-only chunks are no output; the first finish wins.
		streams(
			at("thought-then-text-first-finish-wins"),
			codec,
			...thought,
			data(text("Hi")),
			data(finish("stop")),
			data(finish("length", { prompt_tokens: 3, completion_tokens: 1 })),
			DONE,
		),
		streams(
			at("role-only-chunk-is-no-output"),
			codec,
			data({ id: "gen-1", choices: [{ delta: { role: "assistant", content: "" } }] }),
			data(finish("stop")),
			DONE,
		),
		streams(
			at("text-after-done-is-ignored"),
			codec,
			data(text("Hi")),
			data(finish("stop")),
			DONE,
			data(text("late")),
		),
		streams(at("done-without-finish"), codec, data(text("Hi")), DONE),
		streams(at("no-done-is-truncated"), codec, data(text("Hi"))),
		streams(at("no-items-is-truncated"), codec),
		streams(at("finish-error"), codec, data(text("Hi")), data(finish("error")), DONE),
		streams(at("chunk-not-json"), codec, data("{not json")),
		// An unbilled error chunk keeps its serving.
		streams(at("error-chunk-unbilled"), codec, data({ id: "gen-1", error: { message: "Failed." } }), DONE),
		streams(at("error-chunk-no-serving"), codec, data({ error: { message: "Failed." } }), DONE),
		// Whole replies.
		completes(at("reply-object-content"), codec, {
			choices: [{ message: { content: { title: "A" } }, finish_reason: "stop" }],
		}),
		completes(at("reply-no-finish"), codec, { choices: [{ message: { content: "A" } }] }),
		// Results pair by index.
		encodes(at("results-pair-by-index"), codec, {
			history: [
				{
					role: "step",
					parts: [
						{
							kind: "call",
							index: 1,
							wireId: "run-1/restored/0",
							name: "read_file",
							arguments: '{"file":"a"}',
						},
						{ kind: "call", index: 0, wireId: "call_b", name: "read_file", arguments: '{"file":"b"}' },
						{ kind: "result", index: 0, output: "b", isError: false },
						{ kind: "result", index: 1, output: "a", isError: false },
					],
				},
			],
		}),
		encodes(at("result-of-unknown-call-is-dropped"), codec, {
			history: [
				{
					role: "step",
					parts: [
						{ kind: "call", index: 0, wireId: "call_a", name: "read_file", arguments: "{}" },
						{ kind: "result", index: 0, output: "a", isError: false },
						{ kind: "result", index: 5, output: "lost", isError: true },
					],
				},
			],
		}),
		// State stays last.
		encodes(at("state-last-implicit"), codec, { state: "Live.", cache: { kind: "implicit", ttl: "1h" } }),
		encodes(at("state-last-explicit"), codec, { state: "Live.", cache: { kind: "explicit" } }),
		encodes(at("state-last-no-cache"), codec, { state: "Live.", cache: { kind: "none" } }),
		// History shapes.
		encodes(at("history-text-only-step"), codec, {
			history: [
				{ role: "user", text: "Hi" },
				{ role: "step", producer: { model: LUNA }, parts: [{ kind: "text", text: "Hello." }] },
				{ role: "user", text: "Again" },
			],
		}),
		encodes(at("history-empty-step"), codec, { history: [{ role: "step", parts: [] }] }),
		encodes(at("history-text-and-call"), codec, {
			history: [
				{
					role: "step",
					parts: [
						{ kind: "text", text: "Reading " },
						{ kind: "text", text: "now.", phase: "final" },
						{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: "{}" },
						{ kind: "result", index: 0, output: "x", isError: false },
					],
				},
			],
		}),
		encodes(at("no-history"), codec, { history: [], system: [] }),
		encodes(at("many-system-blocks"), codec, { system: ["One.", "Two.", "Three."], summary: "Earlier." }),
		encodes(at("complete-mode"), codec, {}, "complete"),
		encodes(at("tools-empty-list"), codec, { tools: [] }),
		encodes(at("tools-offered"), codec, {
			tools: [
				FINISH_TOOL,
				{ name: "read", description: "Read.", parameters: { type: "object", properties: {} } },
			],
		}),
	];
});

////////////////////////////////
//  Cache markers

const CACHE_CODECS: Array<[string, Json]> = [
	["openrouter", OPENROUTER],
	["opencode", opencode()],
	["local", local()],
];

const cacheMarkers: CaseSource[] = CACHE_CODECS.flatMap(([name, codec]) => {
	const marked = (suffix: string, cache: Json) =>
		encodes(`${name}-cache-${suffix}`, codec, {
			system: ["Sections.", "Rules."],
			summary: "Earlier.",
			history: [{ role: "user", text: "Read it." }, readStep(LUNA)],
			state: "Live.",
			cache,
		});
	return [
		marked("explicit", { kind: "explicit" }),
		marked("explicit-1h", { kind: "explicit", ttl: "1h" }),
		marked("implicit", { kind: "implicit" }),
		marked("implicit-1h", { kind: "implicit", ttl: "1h" }),
		marked("none", { kind: "none" }),
		marked("none-1h", { kind: "none", ttl: "1h" }),
		encodes(`${name}-cache-explicit-last-message-is-assistant`, codec, {
			history: [
				{ role: "user", text: "Hi" },
				{ role: "step", parts: [{ kind: "text", text: "Hello." }] },
			],
			cache: { kind: "explicit" },
		}),
		encodes(`${name}-cache-explicit-nothing-to-mark`, codec, {
			system: [],
			history: [{ role: "step", parts: [{ kind: "text", text: "Hello." }] }],
			cache: { kind: "explicit" },
		}),
	];
});

////////////////////////////////
//  Forced tools and schemas

const ASKED = { tools: [FINISH_TOOL], toolChoice: { name: "finish" }, schema: TITLE_SCHEMA };

const FORCED = (suffix: string, codec: Json, over: Json = {}, mode: "stream" | "complete" = "complete") =>
	encodes(suffix, codec, { ...ASKED, ...over }, mode);

const forcedAndSchemas: CaseSource[] = [
	// OpenRouter forces by default and always enforces.
	FORCED("openrouter-forced-tool-and-schema", OPENROUTER),
	encodes("openrouter-forced-tool-only", OPENROUTER, { tools: [FINISH_TOOL], toolChoice: { name: "finish" } }),
	encodes("openrouter-forced-name-without-the-tool", OPENROUTER, { toolChoice: { name: "finish" } }),
	encodes("openrouter-schema-only", OPENROUTER, { schema: TITLE_SCHEMA }),
	encodes("openrouter-schema-refused-by-ref", OPENROUTER, {
		schema: { name: "tree", json: { $ref: "#/$defs/node", $defs: { node: { type: "object" } } } },
	}),
	encodes("openrouter-schema-refused-optional-field", OPENROUTER, {
		schema: { name: "pick", json: { type: "object", properties: { a: { type: "string" } } } },
	}),
	// OpenCode follows the catalog's facts.
	FORCED("opencode-enforced", opencode()),
	FORCED("opencode-required-only", opencode({ api: { choice: ["auto", "required"] } })),
	FORCED("opencode-schema-in-prompt", opencode({ api: { structured: false, choice: ["auto"] } })),
	FORCED("opencode-no-choices-at-all", opencode({ api: { choice: [] } })),
	FORCED("opencode-without-facts", OPENCODE_BARE),
	encodes("opencode-tool-wins-over-required", opencode({ api: { choice: ["required", "tool"] } }), {
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
	}),
	encodes("opencode-schema-refused-when-enforced", opencode(), {
		schema: { name: "pick", json: { type: "object", properties: { a: { type: "string" } } } },
	}),
	encodes("opencode-schema-not-refused-when-prompted", opencode({ api: { structured: false } }), {
		schema: { name: "pick", json: { type: "object", properties: { a: { type: "string" } } } },
	}),
	// A URL server never sends tool_choice: the forced tool becomes the answer.
	FORCED("local-forced-tool-and-schema", local()),
	encodes(
		"local-forced-tool-becomes-the-schema",
		local(),
		{ tools: [FINISH_TOOL], toolChoice: { name: "finish" } },
		"complete",
	),
	encodes("local-offered-tools", local(), { tools: [FINISH_TOOL], maxTokens: 8 }),
	encodes("local-forced-name-without-the-tool", local(), { tools: [FINISH_TOOL], toolChoice: { name: "other" } }),
	encodes("local-schema-enforced-and-shown", local(), {
		schema: {
			name: "Reply",
			json: { type: "object", properties: { mood: { type: "string", description: "One word." } } },
		},
	}),
	// Token limits.
	encodes("openrouter-max-tokens-small", OPENROUTER, { maxTokens: 8 }),
	encodes("opencode-max-tokens-floor", opencode(), { maxTokens: 8 }),
	encodes("opencode-max-tokens-above-floor", opencode(), { maxTokens: 4096 }),
	encodes("opencode-floor-needs-a-max", opencode(), {}),
	encodes("local-max-tokens-small", local(), { maxTokens: 8 }),
	encodes("local-max-tokens-zero", local(), { maxTokens: 0 }),
];

////////////////////////////////
//  Key order inside JSON written into strings

const SCRAMBLED = {
	name: "scramble",
	json: {
		type: "object",
		required: ["zeta", "alpha", "10", "2"],
		properties: {
			zeta: { type: "string", description: "Last letter." },
			alpha: { type: "integer" },
			"10": { type: "boolean" },
			"2": { type: "number" },
		},
		additionalProperties: false,
	},
};

const SCRAMBLED_TOOL = {
	name: "scramble",
	description: "Scramble.",
	parameters: {
		type: "object",
		required: ["zeta", "alpha", "10", "2"],
		properties: {
			zeta: { type: "string" },
			alpha: { type: "integer" },
			"10": { type: "boolean" },
			"2": { type: "number" },
		},
		additionalProperties: false,
	},
};

const keyOrder: CaseSource[] = [
	encodes("openrouter-key-order-schema-and-tools", OPENROUTER, {
		schema: SCRAMBLED,
		tools: [SCRAMBLED_TOOL],
		toolChoice: { name: "scramble" },
	}),
	encodes("opencode-key-order-schema-in-prompt", opencode({ api: { structured: false } }), {
		schema: SCRAMBLED,
		tools: [SCRAMBLED_TOOL],
	}),
	encodes("opencode-key-order-schema-enforced", opencode(), { schema: SCRAMBLED }),
	encodes("local-key-order-schema-in-prompt-and-format", local(), { schema: SCRAMBLED, tools: [SCRAMBLED_TOOL] }),
	encodes("local-key-order-forced-tool-as-schema", local(), {
		tools: [SCRAMBLED_TOOL],
		toolChoice: { name: "scramble" },
	}),
	completes("openrouter-key-order-object-content", OPENROUTER, {
		choices: [{ message: { content: { zeta: 1, alpha: 2, "10": 3, "2": 4 } }, finish_reason: "stop" }],
	}),
	streams(
		"openrouter-key-order-object-content-streamed",
		OPENROUTER,
		data(text({ zeta: 1, alpha: { b: 1, a: 2 }, "10": 3, "2": 4 })),
		data(finish("stop")),
		DONE,
	),
	encodes("openrouter-key-order-request-with-calls", OPENROUTER, {
		history: [
			{
				role: "step",
				parts: [
					{
						kind: "call",
						index: 0,
						wireId: "call_a",
						name: "read",
						arguments: '{"zeta":1,"alpha":2,"10":3}',
					},
					{ kind: "result", index: 0, output: "x", isError: false },
				],
			},
		],
	}),
];

////////////////////////////////
//  OpenRouter

const openrouterCases: CaseSource[] = [
	// Blocks join thought deltas.
	streams(
		"openrouter-thought-blocks-join",
		OPENROUTER,
		data(reasoned({ type: "reasoning.text", index: 0, text: "Think" })),
		data(reasoned({ type: "reasoning.encrypted", index: 0, data: "sealed" })),
		data(reasoned({ type: "reasoning.encrypted", index: 1, data: "abc" })),
		data(reasoned({ type: "reasoning.text", index: 0, text: "ing.", signature: "s1" })),
		data(text("Done.")),
		data(reasoned({ type: "reasoning.encrypted", index: 1, data: "abcdef", id: "r1" })),
		data(reasoned({ type: "reasoning.encrypted", index: 2, data: "next", id: "r1" })),
		data(finish("stop")),
		DONE,
	),
	streams(
		"openrouter-thought-late-index-adopts-loose-tail",
		OPENROUTER,
		data(reasoned({ type: "reasoning.text", text: "A" })),
		data(reasoned({ type: "reasoning.text", text: "B" })),
		data(reasoned({ type: "reasoning.text", index: 4, text: "C" })),
		data(finish("stop")),
		DONE,
	),
	streams(
		"openrouter-plain-reasoning-is-a-loose-thought",
		OPENROUTER,
		data({ id: "gen-1", choices: [{ delta: { reasoning: "Hm" } }] }),
		data({ id: "gen-1", choices: [{ delta: { reasoning: "m." } }] }),
		data(finish("stop")),
		DONE,
	),
	streams(
		"openrouter-details-win-over-plain-reasoning",
		OPENROUTER,
		data({
			id: "gen-1",
			choices: [
				{
					delta: {
						reasoning: "plain",
						reasoning_details: [{ type: "reasoning.text", index: 0, text: "block" }],
					},
				},
			],
		}),
		data(finish("stop")),
		DONE,
	),
	streams(
		"openrouter-empty-details-fall-back-to-plain-reasoning",
		OPENROUTER,
		data({ id: "gen-1", choices: [{ delta: { reasoning: "plain", reasoning_details: [] } }] }),
		data(finish("stop")),
		DONE,
	),
	streams(
		"openrouter-summary-and-text-segments",
		OPENROUTER,
		data(reasoned({ type: "reasoning.summary", index: 0, summary: "Plan.", format: "unknown" })),
		data(reasoned({ type: "reasoning.text", index: 1, text: "Body.", summary: "Short.", id: "t1" })),
		data(finish("stop")),
		DONE,
	),
	streams(
		"openrouter-unknown-detail-type-is-kept",
		OPENROUTER,
		data(reasoned({ type: "reasoning.hologram", index: 0, text: "x", signature: "sig" })),
		data(finish("stop")),
		DONE,
	),
	streams(
		"openrouter-usage-fields",
		OPENROUTER,
		data(text("Hi")),
		data(
			finish("stop", {
				prompt_tokens: 10,
				completion_tokens: 5,
				total_tokens: 15,
				cost: 0.002,
				is_byok: true,
				cost_details: { upstream_inference_cost: 0.001 },
				prompt_tokens_details: { cached_tokens: 4, cache_write_tokens: 2 },
				completion_tokens_details: { reasoning_tokens: 3 },
			}),
		),
		DONE,
	),
	streams(
		"openrouter-usage-null-upstream-cost",
		OPENROUTER,
		data(text("Hi")),
		data(
			finish("stop", { prompt_tokens: 1, completion_tokens: 1, cost_details: { upstream_inference_cost: null } }),
		),
		DONE,
	),
	streams(
		"openrouter-usage-on-its-own-chunk",
		OPENROUTER,
		data(text("Hi")),
		data(finish("stop")),
		data({
			id: "gen-1",
			provider: "Azure",
			choices: [],
			usage: { prompt_tokens: 7, completion_tokens: 3, cost: 0.5 },
		}),
		DONE,
	),
	streams(
		"openrouter-serving-from-later-chunks",
		OPENROUTER,
		data({ choices: [{ delta: { content: "a" } }] }),
		data({ id: "gen-9", choices: [{ delta: { content: "b" } }] }),
		data({ provider: "Fireworks", id: "gen-10", choices: [{ delta: { content: "c" } }] }),
		data(finish("stop")),
		DONE,
	),
	// A failure inside a stream.
	streams(
		"openrouter-in-stream-error-code-and-billing",
		OPENROUTER,
		data(text("Hi")),
		data({
			id: "gen-1",
			provider: "Azure",
			error: { code: 429, message: "Provider returned error", metadata: { provider_name: "Mistral" } },
			usage: { prompt_tokens: 4, completion_tokens: 2, cost: 0.1 },
		}),
		DONE,
	),
	streams(
		"openrouter-in-stream-error-text-code",
		OPENROUTER,
		data({ id: "gen-1", error: { code: "server_error", message: "Boom" } }),
	),
	streams("openrouter-in-stream-error-as-text", OPENROUTER, data({ id: "gen-1", error: "Plain words." })),
	streams(
		"openrouter-in-stream-error-with-reasons",
		OPENROUTER,
		data({ error: { code: 403, message: "Flagged", metadata: { reasons: ["violence", 3, "spam"] } } }),
	),
	// Whole replies.
	completes("openrouter-reply-calls-by-position", OPENROUTER, {
		id: "gen-1",
		provider: "Azure",
		choices: [
			{
				message: {
					content: null,
					tool_calls: [wireCall("call_a", "read", '{"a":1}'), wireCall("call_b", "write", "")],
				},
				finish_reason: "tool_calls",
			},
		],
	}),
	completes("openrouter-reply-reasoning-details", OPENROUTER, {
		id: "gen-1",
		choices: [
			{
				message: {
					content: "Done.",
					reasoning_details: [
						{ type: "reasoning.text", index: 0, text: "Think", signature: "s1" },
						{ type: "reasoning.encrypted", index: 1, data: "sealed", id: "r1" },
					],
				},
				finish_reason: "stop",
			},
		],
	}),
	completes("openrouter-reply-error-body", OPENROUTER, {
		id: "gen-1",
		provider: "Azure",
		error: { code: 402, message: "Insufficient credits" },
		usage: { prompt_tokens: 1, completion_tokens: 0 },
	}),
	completes("openrouter-reply-error-no-billing", OPENROUTER, { error: { message: "Failed." } }),
	// HTTP failures by status, then type.
	...(
		[
			[400, { message: "Bad schema" }, "badRequest"],
			[400, { message: "Reasoning must go to the provider that created it." }, "replayRejected"],
			[400, { message: "REASONING MUST GO TO THE PROVIDER THAT CREATED IT." }, "replayRejected"],
			[401, { message: "No auth" }, "keyRejected"],
			[402, { message: "Insufficient credits" }, "outOfCredit"],
			[403, { message: "Flagged", metadata: { reasons: ["violence"] } }, "refused"],
			[404, { message: "No endpoints found" }, "notFound"],
			[408, undefined, "unavailable"],
			[
				429,
				{ message: "temporarily rate-limited upstream.", metadata: { error_type: "rate_limit_exceeded" } },
				"overloaded",
			],
			[429, { message: "Provider returned error", metadata: { provider_name: "Mistral" } }, "overloaded"],
			[429, { message: "Slowed UPSTREAM again" }, "overloaded"],
			[429, { message: "Rate limit exceeded: free-models-per-min." }, "rateLimited"],
			[429, { message: "Rate limited", metadata: { provider_name: 7 } }, "rateLimited"],
			[503, { message: "No endpoints available upstream." }, "unavailable"],
			[500, undefined, "unavailable"],
			[418, { message: "Teapot" }, "badRequest"],
			[301, { message: "Moved" }, "badRequest"],
		] as Array<[number, Json | undefined, string]>
	).map(([status, error, category]) =>
		fails(
			`openrouter-http-${status}-${category}${
				error === undefined
					? "-no-error"
					: `-${
							error["message"] === undefined
								? "bare"
								: String(error["message"])
										.slice(0, 12)
										.replaceAll(/[^A-Za-z0-9]+/g, "")
						}`
			}`,
			OPENROUTER,
			status,
			error === undefined ? "xxxxxxxxxx" : { error },
		),
	),
	fails("openrouter-http-reasons-keep-text-only", OPENROUTER, 403, {
		error: { message: "Flagged", metadata: { reasons: ["violence", 3] } },
	}),
	fails("openrouter-http-reasons-not-a-list", OPENROUTER, 403, {
		error: { message: "Flagged", metadata: { reasons: "violence" } },
	}),
	fails("openrouter-http-no-body", OPENROUTER, 500, null),
	fails("openrouter-http-blank-message", OPENROUTER, 500, { error: { message: "  " } }),
	fails("openrouter-http-error-as-text", OPENROUTER, 502, { error: "Bad gateway." }),
	fails("openrouter-http-error-as-empty-text", OPENROUTER, 502, { error: "" }),
	fails("openrouter-http-error-null", OPENROUTER, 502, { error: null }),
	fails("openrouter-http-error-number", OPENROUTER, 502, { error: 5 }),
	fails("openrouter-http-error-true", OPENROUTER, 502, { error: true }),
	fails("openrouter-http-error-array", OPENROUTER, 502, { error: [] }),
	fails("openrouter-http-error-zero", OPENROUTER, 502, { error: 0 }),
	fails("openrouter-http-body-array", OPENROUTER, 502, [{ error: { message: "x" } }]),
	fails("openrouter-http-body-text", OPENROUTER, 502, "Bad gateway."),
	fails("openrouter-http-message-not-text", OPENROUTER, 502, { error: { message: 5 } }),
	fails("openrouter-http-metadata-not-an-object", OPENROUTER, 429, { error: { message: "x", metadata: "upstream" } }),
	fails("openrouter-http-headers-ignored", OPENROUTER, 429, { error: { message: "Slow." } }, [["retry-after", "30"]]),
	// In-stream errors decode by their own code.
	...(
		[
			[401, "keyRejected"],
			[402, "outOfCredit"],
			[403, "refused"],
			[404, "notFound"],
			[408, "unavailable"],
			[500, "unavailable"],
			[400, "badRequest"],
		] as Array<[number, string]>
	).map(([code, category]) =>
		streams(
			`openrouter-in-stream-code-${code}-${category}`,
			OPENROUTER,
			data({ id: "gen-1", error: { code, message: "Failed." } }),
		),
	),
	// Thought replay: the same model, the blocks that came.
	encodes("openrouter-replay-blocks-as-they-came", OPENROUTER, {
		history: [
			{ role: "user", text: "Hi" },
			{
				role: "step",
				producer: { model: LUNA },
				parts: [
					{
						kind: "thought",
						type: "reasoning.text",
						segments: [{ kind: "content", text: "Thinking." }],
						opaque: "s1",
						index: 0,
					},
					{ kind: "thought", type: "reasoning.encrypted", opaque: "abcdef", id: "r1", index: 1 },
					{ kind: "text", text: "Done." },
				],
			},
		],
	}),
	encodes("openrouter-replay-foreign-model-drops-blocks", OPENROUTER, {
		model: OPUS,
		history: [
			{
				role: "step",
				producer: { model: LUNA },
				parts: [
					{ kind: "thought", type: "reasoning.encrypted", opaque: "abcdef", index: 1 },
					{ kind: "text", text: "Done." },
				],
			},
		],
	}),
	encodes("openrouter-replay-step-without-producer", OPENROUTER, {
		history: [
			{
				role: "step",
				parts: [{ kind: "thought", type: "reasoning.encrypted", opaque: "abcdef" }],
			},
		],
	}),
	encodes("openrouter-replay-missing-signature", OPENROUTER, {
		history: [
			{
				role: "step",
				producer: { model: LUNA },
				parts: [
					{
						kind: "thought",
						type: "reasoning.text",
						segments: [{ kind: "content", text: "No seal." }],
						index: 0,
					},
				],
			},
		],
	}),
	encodes("openrouter-replay-signature-without-text", OPENROUTER, {
		history: [
			{
				role: "step",
				producer: { model: LUNA },
				parts: [{ kind: "thought", type: "reasoning.text", opaque: "only-sig", index: 0 }],
			},
		],
	}),
	encodes("openrouter-replay-summary-and-content-segments", OPENROUTER, {
		history: [
			{
				role: "step",
				producer: { model: LUNA },
				parts: [
					{
						kind: "thought",
						type: "reasoning.summary",
						segments: [
							{ kind: "summary", text: "Plan " },
							{ kind: "content", text: "Body " },
							{ kind: "summary", text: "more." },
							{ kind: "content", text: "end." },
						],
						format: "anthropic-claude-v1",
						id: "t9",
					},
				],
			},
		],
	}),
	encodes("openrouter-replay-anchor-is-ignored", OPENROUTER, {
		history: [
			{
				role: "step",
				producer: { model: LUNA },
				parts: [{ kind: "thought", type: "reasoning.encrypted", opaque: "x", anchor: "call:0" }],
			},
		],
	}),
	encodes("openrouter-replay-loose-reasoning-is-not-replayed", OPENROUTER, {
		history: [
			{
				role: "step",
				producer: { model: LUNA },
				parts: [
					{ kind: "thought", type: "reasoning", segments: [{ kind: "content", text: "Hm." }] },
					{ kind: "text", text: "Done." },
				],
			},
		],
	}),
	// Routing.
	encodes(
		"openrouter-request-routing-tools-schema-limits",
		OPENROUTER,
		{
			zdrOnly: false,
			effort: "high",
			tools: [FINISH_TOOL],
			toolChoice: { name: "finish" },
			schema: TITLE_SCHEMA,
			maxTokens: 8,
		},
		"complete",
	),
	encodes("openrouter-request-zdr-only", OPENROUTER, { zdrOnly: true }),
	encodes("openrouter-request-no-effort", OPENROUTER, { zdrOnly: false }),
	encodes("openrouter-request-effort-is-never-gated", OPENROUTER, { effort: "max" }),
	encodes("openrouter-request-session", OPENROUTER, { session: "other-7" }),
	encodes("openrouter-affinity-pins-the-host", OPENROUTER, {
		history: [
			{ role: "user", text: "Hi" },
			readStep(LUNA, { kind: "thought", type: "reasoning.encrypted", opaque: "x", index: 0 }, "Azure"),
		],
	}),
	encodes("openrouter-affinity-newest-step-wins", OPENROUTER, {
		history: [
			readStep(LUNA, { kind: "thought", type: "reasoning.encrypted", opaque: "x", index: 0 }, "Azure"),
			readStep(LUNA, { kind: "thought", type: "reasoning.encrypted", opaque: "y", index: 0 }, "Fireworks"),
		],
	}),
	encodes("openrouter-affinity-skips-steps-without-blocks", OPENROUTER, {
		history: [
			readStep(LUNA, { kind: "thought", type: "reasoning.encrypted", opaque: "x", index: 0 }, "Azure"),
			readStep(
				LUNA,
				{ kind: "thought", type: "reasoning", segments: [{ kind: "content", text: "loose" }] },
				"Fireworks",
			),
		],
	}),
	encodes("openrouter-affinity-needs-a-recorded-host", OPENROUTER, {
		history: [readStep(LUNA, { kind: "thought", type: "reasoning.encrypted", opaque: "x", index: 0 })],
	}),
	encodes("openrouter-affinity-ignores-other-models", OPENROUTER, {
		history: [readStep(OPUS, { kind: "thought", type: "reasoning.encrypted", opaque: "x", index: 0 }, "Azure")],
	}),
	encodes("openrouter-affinity-with-zdr", OPENROUTER, {
		zdrOnly: false,
		history: [readStep(LUNA, { kind: "thought", type: "reasoning.encrypted", opaque: "x", index: 0 }, "Azure")],
	}),
];

////////////////////////////////
//  OpenCode

const resetIso = new Date(NOW + 3_600_000).toISOString();
const resetHttp = new Date(NOW + 3_600_000).toUTCString();

const opencodeFailure = (
	name: string,
	status: number,
	error: Json,
	headers: string[][] = [],
	nowMs = NOW,
): CaseSource => fails(name, opencode({ nowMs }), status, { type: "error", error }, headers);

const opencodeCases: CaseSource[] = [
	encodes("opencode-request-header-usage-efforts-floor", opencode(), {
		model: DEEPSEEK,
		effort: "high",
		maxTokens: 8,
	}),
	encodes("opencode-request-unlisted-effort-complete", opencode(), { model: DEEPSEEK, effort: "medium" }, "complete"),
	encodes("opencode-request-no-efforts-listed", opencode({ efforts: [] }), { model: DEEPSEEK, effort: "high" }),
	encodes("opencode-request-no-facts", OPENCODE_BARE, { model: DEEPSEEK, effort: "high" }),
	encodes("opencode-request-session-header", opencode(), { session: "root-9" }),
	// Reasoning goes back to its own family.
	...(
		[
			[
				"own-family",
				DEEPSEEK,
				readStep("deepseek-v4-pro", {
					kind: "thought",
					type: "reasoning_content",
					segments: [{ kind: "content", text: "Read it." }],
				}),
			],
			[
				"vendored-name",
				DEEPSEEK,
				readStep("deepseek/deepseek-v4.1-flash", {
					kind: "thought",
					type: "reasoning_content",
					segments: [{ kind: "content", text: "Read it." }],
				}),
			],
			[
				"foreign-tool-turn-carries-it-empty",
				DEEPSEEK,
				readStep(LUNA, {
					kind: "thought",
					type: "reasoning_content",
					segments: [{ kind: "content", text: "Read it." }],
				}),
			],
			["blocks-of-another-kind-carry-it-empty", DEEPSEEK, readStep(DEEPSEEK)],
			[
				"no-thought-no-call-sends-nothing",
				DEEPSEEK,
				{ role: "step", producer: { model: LUNA }, parts: [{ kind: "text", text: "Hi" }] },
			],
			[
				"step-without-producer",
				DEEPSEEK,
				{
					role: "step",
					parts: [
						{ kind: "thought", type: "reasoning_content", segments: [{ kind: "content", text: "x" }] },
						{ kind: "call", index: 0, wireId: "c", name: "n", arguments: "{}" },
					],
				},
			],
			[
				"model-without-a-dash",
				"deepseek",
				readStep("deepseek", {
					kind: "thought",
					type: "reasoning_content",
					segments: [{ kind: "content", text: "Same." }],
				}),
			],
			[
				"two-segments-join",
				DEEPSEEK,
				readStep(DEEPSEEK, {
					kind: "thought",
					type: "reasoning_content",
					segments: [
						{ kind: "content", text: "A" },
						{ kind: "content", text: "B" },
					],
				}),
			],
			[
				"empty-text-with-a-call",
				DEEPSEEK,
				readStep(DEEPSEEK, {
					kind: "thought",
					type: "reasoning_content",
					segments: [{ kind: "content", text: "" }],
				}),
			],
		] as Array<[string, string, Json]>
	).map(([name, model, step]) => encodes(`opencode-replay-${name}`, opencode(), { model, history: [step] })),
	encodes("opencode-replay-off-without-the-fact", opencode({ api: { reasoning: undefined } }), {
		model: DEEPSEEK,
		history: [
			readStep(DEEPSEEK, {
				kind: "thought",
				type: "reasoning_content",
				segments: [{ kind: "content", text: "Read it." }],
			}),
		],
	}),
	encodes("opencode-replay-off-with-another-field", opencode({ api: { reasoning: "reasoning" } }), {
		model: DEEPSEEK,
		history: [
			readStep(DEEPSEEK, {
				kind: "thought",
				type: "reasoning_content",
				segments: [{ kind: "content", text: "Read it." }],
			}),
		],
	}),
	encodes("opencode-replay-off-without-facts", OPENCODE_BARE, {
		model: DEEPSEEK,
		history: [
			readStep(DEEPSEEK, {
				kind: "thought",
				type: "reasoning_content",
				segments: [{ kind: "content", text: "Read it." }],
			}),
		],
	}),
	encodes("opencode-replay-anchor-is-ignored", opencode(), {
		model: DEEPSEEK,
		history: [
			readStep(DEEPSEEK, {
				kind: "thought",
				type: "reasoning_content",
				segments: [{ kind: "content", text: "Read it." }],
				anchor: "call:0",
			}),
		],
	}),
	// Reading thoughts.
	streams(
		"opencode-reasoning-content-reads",
		opencode(),
		data(reasoningContent("Read ")),
		data(reasoningContent("it.")),
		data(call(0, { id: "call_a", name: "read_file", args: "{}" })),
		data(finish("tool_calls")),
		DONE,
	),
	streams(
		"opencode-plain-reasoning-field-is-not-read",
		opencode(),
		data(reasoning("hidden")),
		data(text("Hi")),
		data(finish("stop")),
		DONE,
	),
	streams(
		"opencode-empty-reasoning-content-is-no-thought",
		opencode(),
		data(reasoningContent("")),
		data(text("Hi")),
		data(finish("stop")),
		DONE,
	),
	// Usage.
	streams(
		"opencode-usage-charge-source-list-cost-allowance",
		opencode(),
		data(text("Hi")),
		data(
			finish("stop", {
				prompt_tokens: 100,
				completion_tokens: 10,
				prompt_tokens_details: { cached_tokens: 60 },
				completion_tokens_details: { reasoning_tokens: 4 },
				cost: 0,
				opencode: { billing_source: "go", list_cost: 0.0012, allowance: 0.0048 },
			}),
		),
		DONE,
	),
	streams(
		"opencode-usage-unknown-source-and-negative-amounts-drop",
		opencode(),
		data(text("Hi")),
		data(
			finish("stop", {
				prompt_tokens: 100,
				completion_tokens: 10,
				prompt_tokens_details: { cached_tokens: 60 },
				completion_tokens_details: { reasoning_tokens: 4 },
				cost: -0.1,
				opencode: { billing_source: "promo", list_cost: 0.0012, allowance: -1 },
			}),
		),
		DONE,
	),
	streams(
		"opencode-usage-from-the-token-block",
		opencode(),
		data(text("Hi")),
		data(
			finish("stop", {
				prompt_tokens: 50,
				completion_tokens: 5,
				opencode: {
					billing_source: "byok",
					tokens: { cache_read: 30, reasoning: 2, cache_write_5m: 3, cache_write_1h: 4 },
				},
			}),
		),
		DONE,
	),
	streams(
		"opencode-usage-own-counts-win-over-the-block",
		opencode(),
		data(text("Hi")),
		data(
			finish("stop", {
				prompt_tokens: 50,
				completion_tokens: 5,
				prompt_tokens_details: { cached_tokens: 10, cache_write_tokens: 1 },
				completion_tokens_details: { reasoning_tokens: 1 },
				opencode: { tokens: { cache_read: 30, reasoning: 2, cache_write_5m: 3 } },
			}),
		),
		DONE,
	),
	streams(
		"opencode-usage-one-cache-write-size",
		opencode(),
		data(text("Hi")),
		data(finish("stop", { prompt_tokens: 5, completion_tokens: 1, opencode: { tokens: { cache_write_1h: 4 } } })),
		DONE,
	),
	streams(
		"opencode-usage-every-source",
		opencode(),
		data(text("a")),
		data({
			id: "g",
			choices: [{ delta: { content: "" } }],
			usage: { prompt_tokens: 1, completion_tokens: 1, opencode: { billing_source: "free" } },
		}),
		data({
			id: "g",
			choices: [{ delta: { content: "" } }],
			usage: { prompt_tokens: 1, completion_tokens: 1, opencode: { billing_source: "balance" } },
		}),
		data({
			id: "g",
			choices: [{ delta: { content: "" } }],
			usage: { prompt_tokens: 1, completion_tokens: 1, opencode: { billing_source: "go-plus" } },
		}),
		data(finish("stop")),
		DONE,
	),
	streams("opencode-serving-only-the-id", opencode(), data(text("Hi")), data(finish("stop")), DONE),
	// Failures.
	opencodeFailure("opencode-http-401-rejected-key", 401, { type: "AuthError", message: "Invalid API key." }),
	opencodeFailure("opencode-http-402-no-funds", 402, {
		type: "CreditLimitExceeded",
		message: "Insufficient account funds",
	}),
	opencodeFailure("opencode-http-429-go-pool-resets-at-iso", 429, {
		type: "GoUsageLimitError",
		message: "Usage limit reached.",
		resetsAt: resetIso,
	}),
	opencodeFailure(
		"opencode-http-429-budget-ignores-retry-after",
		429,
		{ type: "BudgetLimitError", message: "Budget reached.", resetsAt: NOW + 3_600_000 },
		[["retry-after", "30"]],
	),
	opencodeFailure("opencode-http-429-allowance-seconds-stamp", 429, {
		type: "GoUsageLimitError",
		message: "Limit.",
		resetsAt: (NOW + 7_200_000) / 1000,
	}),
	opencodeFailure("opencode-http-429-allowance-http-date", 429, {
		type: "GoUsageLimitError",
		message: "Limit.",
		resetsAt: resetHttp,
	}),
	opencodeFailure("opencode-http-429-allowance-no-stamp", 429, { type: "GoUsageLimitError", message: "Limit." }),
	opencodeFailure("opencode-http-429-allowance-bad-stamp", 429, {
		type: "GoUsageLimitError",
		message: "Limit.",
		resetsAt: "tomorrow-ish",
	}),
	opencodeFailure("opencode-http-429-allowance-stamp-in-the-past", 429, {
		type: "GoUsageLimitError",
		message: "Limit.",
		resetsAt: NOW - 1000,
	}),
	opencodeFailure("opencode-http-429-rate-limit-until-reset", 429, {
		type: "rate_limit",
		message: "Slow down.",
		resetsAt: NOW + 3_600_000,
	}),
	opencodeFailure("opencode-http-429-rate-limit-reset-iso", 429, {
		type: "rate_limit",
		message: "Slow down.",
		resetsAt: resetIso,
	}),
	opencodeFailure("opencode-http-429-rate-limit-reset-seconds", 429, {
		type: "rate_limit",
		message: "Slow down.",
		resetsAt: (NOW + 90_000) / 1000,
	}),
	opencodeFailure("opencode-http-429-rate-limit-reset-past-is-zero", 429, {
		type: "rate_limit",
		message: "Slow down.",
		resetsAt: NOW - 5000,
	}),
	opencodeFailure(
		"opencode-http-429-rate-limit-retry-after-header-wins",
		429,
		{ type: "rate_limit", message: "Slow down.", resetsAt: NOW + 3_600_000 },
		[["Retry-After", "2"]],
	),
	opencodeFailure("opencode-http-429-rate-limit-no-reset", 429, { type: "rate_limit", message: "Slow down." }),
	// Reset stamps the contract's time primitives read differently from the platform's.
	opencodeFailure("opencode-http-429-allowance-stamp-as-a-loose-date", 429, {
		type: "GoUsageLimitError",
		message: "Limit.",
		resetsAt: "Jan 15 2027 09:00:00 GMT",
	}),
	opencodeFailure("opencode-http-429-allowance-stamp-date-only", 429, {
		type: "GoUsageLimitError",
		message: "Limit.",
		resetsAt: "2027-01-16",
	}),
	opencodeFailure("opencode-http-429-allowance-stamp-in-fractional-seconds", 429, {
		type: "GoUsageLimitError",
		message: "Limit.",
		resetsAt: 1_800_000_090.0004,
	}),
	opencodeFailure("opencode-http-429-allowance-stamp-far-negative", 429, {
		type: "GoUsageLimitError",
		message: "Limit.",
		resetsAt: -200_000_000_000,
	}),
	opencodeFailure("opencode-http-429-rate-limit-reset-in-fractional-seconds", 429, {
		type: "rate_limit",
		message: "Slow down.",
		resetsAt: 1_800_000_090.0004,
	}),
	opencodeFailure("opencode-http-429-rate-limit-reset-padded-text", 429, {
		type: "rate_limit",
		message: "Slow down.",
		resetsAt: ` ${resetIso} `,
	}),
	opencodeFailure(
		"opencode-http-429-rate-limit-reset-from-a-later-clock",
		429,
		{ type: "rate_limit", message: "Slow down.", resetsAt: NOW + 3_600_000 },
		[],
		NOW + 1_800_000,
	),
	opencodeFailure("opencode-http-429-overloaded-marker", 429, {
		type: "server_error",
		message: "[overloaded] the server is overloaded; retry shortly",
	}),
	opencodeFailure("opencode-http-429-overloaded-type", 429, { type: "overloaded_error", message: "Busy." }),
	opencodeFailure("opencode-http-500-overloaded-marker-is-unavailable", 500, {
		type: "server_error",
		message: "[overloaded] again",
	}),
	opencodeFailure("opencode-http-400-model-unavailable", 400, {
		type: "invalid_request_error",
		message: "Model is unavailable.",
	}),
	opencodeFailure("opencode-http-400-model-unavailable-upper-case", 400, {
		message: "MODEL IS UNAVAILABLE right now",
	}),
	opencodeFailure("opencode-http-400-no-session", 400, { type: "MissingSessionID", message: "Session id required." }),
	opencodeFailure("opencode-http-403-refused", 403, { message: "Forbidden." }),
	opencodeFailure("opencode-http-404-not-found", 404, { message: "No such model." }),
	opencodeFailure("opencode-http-408-unavailable", 408, { message: "Timed out." }),
	opencodeFailure("opencode-http-502-unavailable", 502, { message: "Bad gateway." }),
	opencodeFailure("opencode-http-418-bad-request", 418, { message: "Teapot." }),
	opencodeFailure("opencode-http-limit-type-wins-over-status", 500, { type: "GoUsageLimitError", message: "Limit." }),
	opencodeFailure("opencode-http-blank-message-keeps-status-text", 500, { message: "  " }),
	fails("opencode-http-blank-message-and-error-as-text", opencode(), 502, { error: "" }),
	fails("opencode-http-no-body", opencode(), 500, null),
	fails("opencode-http-error-before-a-model-401", OPENCODE_BARE, 401, { error: { message: "No." } }),
	fails("opencode-http-error-before-a-model-400-unavailable", OPENCODE_BARE, 400, {
		error: { message: "Model is unavailable." },
	}),
	// In-stream errors.
	streams(
		"opencode-in-stream-limit-by-type",
		opencode({ nowMs: NOW }),
		data({
			id: "gen-1",
			error: { type: "GoUsageLimitError", message: "Usage limit reached.", resetsAt: resetIso },
		}),
		DONE,
	),
	streams(
		"opencode-in-stream-overloaded",
		opencode(),
		data(text("Hi")),
		data({
			id: "gen-1",
			error: { type: "server_error", message: "[overloaded] shortly" },
			usage: { prompt_tokens: 4, completion_tokens: 2, cost: 0.1 },
		}),
		DONE,
	),
	streams(
		"opencode-in-stream-plain-error-is-a-stream-failure",
		opencode(),
		data({ id: "gen-1", error: { type: "server_error", message: "Boom." } }),
	),
	streams(
		"opencode-in-stream-numeric-code",
		opencode(),
		data({ id: "gen-1", error: { code: 503, message: "Down." } }),
	),
	streams(
		"opencode-in-stream-error-with-billing-and-source",
		opencode(),
		data({
			id: "gen-2",
			error: { message: "Cut." },
			usage: {
				prompt_tokens: 10,
				completion_tokens: 3,
				cost: 0.01,
				opencode: { billing_source: "go", allowance: 0.02 },
			},
		}),
	),
	completes("opencode-reply-reasoning-content", opencode(), {
		id: "gen-1",
		choices: [{ message: { content: "Done.", reasoning_content: "Why." }, finish_reason: "stop" }],
		usage: { prompt_tokens: 5, completion_tokens: 2, opencode: { billing_source: "go" } },
	}),
	completes("opencode-reply-error-body", opencode(), {
		id: "gen-1",
		error: { type: "GoUsageLimitError", message: "Limit." },
	}),
];

////////////////////////////////
//  URL servers

const localCases: CaseSource[] = [
	encodes("local-request-effort-listed-none", local(), { effort: "none" }),
	encodes("local-request-effort-listed-high", local(), { effort: "high" }),
	encodes("local-request-effort-not-listed", local(), { effort: "xhigh" }),
	encodes("local-request-no-effort-list", local({}, []), { effort: "high" }),
	encodes(
		"local-request-no-facts",
		{ family: "chat-completions", dialect: "local", local: { host: "Lab", label: "Ollama" } },
		{ effort: "high" },
	),
	encodes("local-request-complete-has-no-stream-options", local(), {}, "complete"),
	encodes("local-request-default-path", local(), {}),
	encodes("local-request-custom-path", local({ chatPath: "/chat/completions" }), {}),
	encodes("local-request-no-session-or-routing", local(), { session: "root-3", zdrOnly: false }),
	// Thoughts: read from either field, replayed only when the dialect says so.
	streams(
		"local-reads-either-reasoning-field",
		local(),
		data({ id: "c", choices: [{ delta: { reasoning_content: "Hmm." } }] }),
		data({ id: "c", choices: [{ delta: { reasoning: "More." } }] }),
		data(finish("stop", { prompt_tokens: 3, completion_tokens: 1 })),
		DONE,
	),
	streams(
		"local-reasoning-content-wins-over-reasoning",
		local(),
		data({ id: "c", choices: [{ delta: { reasoning_content: "A", reasoning: "B" } }] }),
		data(finish("stop")),
		DONE,
	),
	streams(
		"local-empty-reasoning-content-hides-reasoning",
		local(),
		data({ id: "c", choices: [{ delta: { reasoning_content: "", reasoning: "B" } }] }),
		data(finish("stop")),
		DONE,
	),
	...(
		[
			["never-replays-without-a-field", local(), LUNA],
			["as-reasoning", local({ replay: "reasoning" }), LUNA],
			["as-reasoning-content", local({ replay: "reasoning_content" }), LUNA],
			["not-for-another-model", local({ replay: "reasoning" }), OPUS],
		] as Array<[string, Json, string]>
	).map(([name, codec, model]) =>
		encodes(`local-replay-${name}`, codec, {
			model,
			history: [
				{
					role: "step",
					producer: { model: LUNA },
					parts: [
						{ kind: "thought", type: "reasoning", segments: [{ kind: "content", text: "Hmm." }] },
						{ kind: "text", text: "Hi" },
					],
				},
			],
		}),
	),
	encodes("local-replay-anchor-is-ignored", local({ replay: "reasoning" }), {
		history: [
			{
				role: "step",
				producer: { model: LUNA },
				parts: [
					{
						kind: "thought",
						type: "reasoning",
						segments: [{ kind: "content", text: "Hmm." }],
						anchor: "call:0",
					},
					{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: "{}" },
					{ kind: "result", index: 0, output: "x", isError: false },
				],
			},
		],
	}),
	encodes("local-replay-empty-text-sends-nothing", local({ replay: "reasoning" }), {
		history: [
			{
				role: "step",
				producer: { model: LUNA },
				parts: [
					{ kind: "thought", type: "reasoning", segments: [{ kind: "content", text: "" }] },
					{ kind: "text", text: "Hi" },
				],
			},
		],
	}),
	encodes("local-replay-other-thought-types-stay-out", local({ replay: "reasoning" }), {
		history: [
			{
				role: "step",
				producer: { model: LUNA },
				parts: [
					{ kind: "thought", type: "reasoning_content", segments: [{ kind: "content", text: "No." }] },
					{
						kind: "thought",
						type: "reasoning.text",
						segments: [{ kind: "content", text: "No." }],
						opaque: "s",
					},
					{ kind: "text", text: "Hi" },
				],
			},
		],
	}),
	encodes("local-replay-two-thoughts-join", local({ replay: "reasoning_content" }), {
		history: [
			{
				role: "step",
				producer: { model: LUNA },
				parts: [
					{ kind: "thought", type: "reasoning", segments: [{ kind: "content", text: "A" }] },
					{ kind: "thought", type: "reasoning", segments: [{ kind: "content", text: "B" }] },
				],
			},
		],
	}),
	// Usage and serving.
	streams(
		"local-usage-free-model-costs-nothing",
		local({ free: true }),
		data(text("Hi")),
		data(finish("stop", { prompt_tokens: 100, completion_tokens: 10, cost: 5 })),
		DONE,
	),
	streams(
		"local-usage-paid-model-reports-no-cost",
		local({ free: false }),
		data(text("Hi")),
		data(finish("stop", { prompt_tokens: 100, completion_tokens: 10, cost: 5 })),
		DONE,
	),
	streams(
		"local-serving-names-the-host",
		local({ host: "Box" }),
		data({ choices: [{ delta: { content: "a" } }] }),
		data(finish("stop")),
		DONE,
	),
	streams("local-serving-host-with-id", local({ host: "Box" }), data(text("a")), data(finish("stop")), DONE),
	completes("local-reply-names-the-host", local(), {
		id: "gen-1",
		choices: [{ message: { content: "Hi", reasoning: "Because." }, finish_reason: "stop" }],
		usage: { prompt_tokens: 1, completion_tokens: 1 },
	}),
	completes("local-reply-without-an-id-still-names-the-host", local(), {
		choices: [{ message: { content: "Hi" }, finish_reason: "stop" }],
	}),
	// Failures.
	streams(
		"local-in-stream-error-names-the-server",
		local(),
		data(text("Hi")),
		data({ id: "gen-1", error: { type: "api_error", message: "the model failed to generate a response" } }),
		DONE,
	),
	streams("local-in-stream-error-blank-message", local(), data({ error: { message: " " } })),
	streams("local-in-stream-error-with-code", local(), data({ error: { code: 503, message: "Down." } })),
	streams("local-in-stream-error-with-code-404", local(), data({ error: { code: 404, message: "Gone." } })),
	...[404, 400, 401, 402, 403, 408, 429, 500, 503, 418].flatMap((status) => [
		fails(`local-http-${status}-open`, local(), status, { error: { message: "Said." } }),
		fails(`local-http-${status}-keyed`, local({ optionalKey: true, label: "Box" }), status, {
			error: { message: "Said." },
		}),
	]),
	fails("local-http-no-message", local(), 500, { error: {} }),
	fails("local-http-no-body", local({ label: "LM Studio" }), 500, null),
	fails("local-http-error-as-text", local(), 503, {
		error: "server busy, please try again.  maximum pending requests exceeded",
	}),
	fails("local-http-model-missing", local(), 404, {
		error: { type: "not_found_error", message: 'model "qwen" not found, try pulling it first' },
	}),
	fails("local-http-no-thinking-support", local(), 400, {
		error: { type: "invalid_request_error", message: '"qwen" does not support thinking' },
	}),
];

////////////////////////////////
//  Loose provider JSON

const LOOSE_CODECS: Array<[string, Json]> = [
	["openrouter", OPENROUTER],
	["opencode", opencode()],
	["local", local({ replay: "reasoning" })],
];

const oneChunk = (name: string, codec: Json, chunk: unknown, ...rest: Json[]): CaseSource =>
	streams(name, codec, typeof chunk === "string" ? data(chunk) : data(chunk as Json), ...rest);

const looseStreams: CaseSource[] = LOOSE_CODECS.flatMap(([name, codec]) => {
	const at = (suffix: string) => `${name}-loose-${suffix}`;
	const withFinish = (chunk: unknown, label: string) => oneChunk(at(label), codec, chunk, data(finish("stop")), DONE);
	return [
		// Chunks that are not objects.
		oneChunk(at("chunk-empty-text"), codec, ""),
		oneChunk(at("chunk-array"), codec, "[1,2]"),
		oneChunk(at("chunk-number"), codec, "5"),
		oneChunk(at("chunk-string"), codec, '"text"'),
		oneChunk(at("chunk-null"), codec, "null"),
		oneChunk(at("chunk-truncated-json"), codec, '{"choices":[{"delta":{"content":"a'),
		oneChunk(
			at("chunk-surrogate-escape"),
			codec,
			'{"choices":[{"delta":{"content":"a\\ud800b"}}]}',
			data(finish("stop")),
			DONE,
		),
		oneChunk(at("chunk-empty-object"), codec, {}, data(finish("stop")), DONE),
		// choices.
		withFinish({ choices: "no" }, "choices-text"),
		withFinish({ choices: { 0: { delta: { content: "a" } } } }, "choices-object"),
		withFinish({ choices: null }, "choices-null"),
		withFinish({ choices: [null, 5, "x", [], { delta: { content: "kept" } }] }, "choices-junk-entries"),
		withFinish({ choices: [{ delta: { content: "a" } }, { delta: { content: "b" } }] }, "choices-two-entries"),
		withFinish({ choices: [{}] }, "choice-empty"),
		withFinish({ choices: [{ delta: null }] }, "delta-null"),
		withFinish({ choices: [{ delta: "text" }] }, "delta-text"),
		withFinish({ choices: [{ delta: [] }] }, "delta-array"),
		withFinish({ choices: [{ delta: { role: 5, content: "a" } }] }, "role-mistyped"),
		// content.
		withFinish({ choices: [{ delta: { content: null } }] }, "content-null"),
		withFinish({ choices: [{ delta: { content: 5 } }] }, "content-number"),
		withFinish({ choices: [{ delta: { content: true } }] }, "content-boolean"),
		withFinish(
			{ choices: [{ delta: { content: ["a", "  ", { text: "b" }, { content: [{ text: "c" }] }, null, 7] } }] },
			"content-segments",
		),
		withFinish(
			{ choices: [{ delta: { content: { content: [{ text: "x" }, "y"] } } }] },
			"content-object-with-segments",
		),
		withFinish({ choices: [{ delta: { content: { content: "plain" } } }] }, "content-object-with-text-content"),
		withFinish({ choices: [{ delta: { content: { value: 1, deep: { k: [1] } } } }] }, "content-object"),
		withFinish({ choices: [{ delta: { content: [] } }] }, "content-empty-array"),
		withFinish({ choices: [{ delta: { content: [[1]] } }] }, "content-nested-array"),
		withFinish(
			{ choices: [{ delta: { content: [{ text: 5, content: "no" }] } }] },
			"content-segment-with-mistyped-text",
		),
		withFinish({ choices: [{ delta: { content: "  " } }] }, "content-blank-text"),
		// tool_calls.
		withFinish({ choices: [{ delta: { tool_calls: "x" } }] }, "calls-text"),
		withFinish({ choices: [{ delta: { tool_calls: {} } }] }, "calls-object"),
		withFinish({ choices: [{ delta: { tool_calls: [null, 4, "x"] } }] }, "calls-junk-entries"),
		withFinish({ choices: [{ delta: { tool_calls: [{ index: 0 }] } }] }, "call-index-only"),
		withFinish(
			{ choices: [{ delta: { tool_calls: [{ id: "c", function: { name: "n" } }] } }] },
			"call-without-index",
		),
		withFinish({ choices: [{ delta: { tool_calls: [{ index: "0", id: "c" }] } }] }, "call-index-text"),
		withFinish({ choices: [{ delta: { tool_calls: [{ index: null, id: "c" }] } }] }, "call-index-null"),
		withFinish({ choices: [{ delta: { tool_calls: [{ index: 2.5, id: "c" }] } }] }, "call-index-fraction"),
		withFinish(
			{ choices: [{ delta: { tool_calls: [{ index: 0, id: 7, function: { name: 8, arguments: 9 } }] } }] },
			"call-fields-numbers",
		),
		withFinish(
			{ choices: [{ delta: { tool_calls: [{ index: 0, id: null, function: null }] } }] },
			"call-fields-null",
		),
		withFinish(
			{ choices: [{ delta: { tool_calls: [{ index: 0, id: "c", function: "text" }] } }] },
			"call-function-text",
		),
		withFinish(
			{
				choices: [
					{ delta: { tool_calls: [{ index: 0, id: "c", function: { name: "n", arguments: { a: 1 } } }] } },
				],
			},
			"call-arguments-object",
		),
		withFinish(
			{
				choices: [
					{
						delta: {
							tool_calls: [
								{
									index: 0,
									id: "c",
									type: "other",
									extra: 1,
									function: { name: "n", arguments: "{}", more: 2 },
								},
							],
						},
					},
				],
			},
			"call-unknown-fields",
		),
		withFinish(
			{
				choices: [
					{
						delta: {
							tool_calls: [
								{ index: 0, id: "c", function: { name: "n", arguments: "{" } },
								{ index: 1, id: "d", function: { name: "m", arguments: "[1" } },
							],
						},
					},
				],
			},
			"calls-two-in-a-chunk",
		),
		withFinish(
			{ choices: [{ delta: { tool_calls: [{ index: 0, id: "c", function: { name: "n", arguments: "" } }] } }] },
			"call-empty-arguments",
		),
		withFinish(
			{ choices: [{ delta: { tool_calls: [{ index: 0, id: "c", function: { name: "n", arguments: "  " } }] } }] },
			"call-blank-arguments",
		),
		withFinish(
			{
				choices: [
					{ delta: { tool_calls: [{ index: 0, id: "c", function: { name: "n", arguments: "[1,2]" } }] } },
				],
			},
			"call-arguments-array-text",
		),
		// finish_reason.
		oneChunk(at("finish-null"), codec, { choices: [{ delta: { content: "a" }, finish_reason: null }] }, DONE),
		oneChunk(at("finish-number"), codec, { choices: [{ delta: { content: "a" }, finish_reason: 5 }] }, DONE),
		oneChunk(at("finish-empty-text"), codec, { choices: [{ delta: { content: "a" }, finish_reason: "" }] }, DONE),
		oneChunk(
			at("finish-unknown-name"),
			codec,
			{ choices: [{ delta: { content: "a" }, finish_reason: "function_call" }] },
			DONE,
		),
		oneChunk(at("finish-each-known-name"), codec, { choices: [{ finish_reason: "length" }] }, DONE),
		oneChunk(at("finish-content-filter"), codec, { choices: [{ finish_reason: "content_filter" }] }, DONE),
		oneChunk(at("finish-in-a-choice-without-delta"), codec, { choices: [{ finish_reason: "stop" }] }, DONE),
		oneChunk(at("finish-object"), codec, { choices: [{ finish_reason: { a: 1 } }] }, DONE),
		// Thoughts.
		withFinish({ choices: [{ delta: { reasoning: 5, reasoning_content: 6 } }] }, "reasoning-numbers"),
		withFinish({ choices: [{ delta: { reasoning: null, reasoning_content: null } }] }, "reasoning-null"),
		withFinish({ choices: [{ delta: { reasoning: "", reasoning_content: "" } }] }, "reasoning-empty"),
		withFinish({ choices: [{ delta: { reasoning_details: "x" } }] }, "details-text"),
		withFinish({ choices: [{ delta: { reasoning_details: {} } }] }, "details-object"),
		withFinish({ choices: [{ delta: { reasoning_details: [null, 3, "x", {}] } }] }, "details-junk-entries"),
		withFinish({ choices: [{ delta: { reasoning_details: [{ text: "no type" }] } }] }, "details-without-type"),
		withFinish({ choices: [{ delta: { reasoning_details: [{ type: 5, text: "x" }] } }] }, "details-type-number"),
		withFinish(
			{
				choices: [
					{
						delta: {
							reasoning_details: [
								{
									type: "reasoning.text",
									text: null,
									summary: null,
									signature: null,
									id: null,
									index: null,
									format: null,
								},
							],
						},
					},
				],
			},
			"details-null-fields",
		),
		withFinish(
			{
				choices: [
					{
						delta: {
							reasoning_details: [
								{
									type: "reasoning.text",
									text: 5,
									summary: 6,
									signature: 7,
									id: 8,
									index: "1",
									format: 9,
								},
							],
						},
					},
				],
			},
			"details-mistyped-fields",
		),
		withFinish(
			{ choices: [{ delta: { reasoning_details: [{ type: "reasoning.encrypted", data: 5, signature: "s" }] } }] },
			"details-encrypted-mistyped-data",
		),
		withFinish(
			{
				choices: [
					{
						delta: {
							reasoning_details: [{ type: "reasoning.text", text: "x", extra: { a: 1 }, unknown: true }],
						},
					},
				],
			},
			"details-unknown-fields",
		),
		withFinish(
			{ choices: [{ delta: { reasoning_details: [{ type: "reasoning.text", index: 1.5, text: "x" }] } }] },
			"details-fractional-index",
		),
		// usage.
		oneChunk(at("usage-null"), codec, { choices: [], usage: null }, data(finish("stop")), DONE),
		oneChunk(at("usage-text"), codec, { choices: [], usage: "x" }, data(finish("stop")), DONE),
		oneChunk(at("usage-array"), codec, { choices: [], usage: [1] }, data(finish("stop")), DONE),
		oneChunk(at("usage-empty"), codec, { choices: [], usage: {} }, data(finish("stop")), DONE),
		oneChunk(
			at("usage-mistyped-counts"),
			codec,
			{ choices: [], usage: { prompt_tokens: "5", completion_tokens: null, cost: "0.1" } },
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("usage-fractional-counts"),
			codec,
			{ choices: [], usage: { prompt_tokens: 5.5, completion_tokens: 2.25 } },
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("usage-negative-counts"),
			codec,
			{ choices: [], usage: { prompt_tokens: -5, completion_tokens: -2, cost: -1 } },
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("usage-detail-objects-mistyped"),
			codec,
			{
				choices: [],
				usage: {
					prompt_tokens: 1,
					completion_tokens: 1,
					prompt_tokens_details: "x",
					completion_tokens_details: [1],
					cost_details: 5,
					opencode: "x",
					is_byok: "yes",
				},
			},
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("usage-detail-fields-mistyped"),
			codec,
			{
				choices: [],
				usage: {
					prompt_tokens: 1,
					completion_tokens: 1,
					prompt_tokens_details: { cached_tokens: "4", cache_write_tokens: null },
					completion_tokens_details: { reasoning_tokens: {} },
					cost_details: { upstream_inference_cost: "0.1" },
					opencode: { billing_source: 5, list_cost: "x", allowance: "y", tokens: [1] },
				},
			},
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("usage-unknown-fields"),
			codec,
			{ choices: [], usage: { prompt_tokens: 2, completion_tokens: 1, brand_new: { a: 1 } } },
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("usage-token-block-mistyped"),
			codec,
			{
				choices: [],
				usage: {
					prompt_tokens: 2,
					completion_tokens: 1,
					opencode: { tokens: { cache_read: "9", cache_write_5m: null, cache_write_1h: 2, reasoning: [] } },
				},
			},
			data(finish("stop")),
			DONE,
		),
		// error.
		oneChunk(
			at("error-null-is-no-error"),
			codec,
			{ error: null, choices: [{ delta: { content: "a" } }] },
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("error-number-is-an-error"),
			codec,
			{ error: 5, choices: [{ delta: { content: "a" } }] },
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("error-true-is-an-error"),
			codec,
			{ error: true, choices: [{ delta: { content: "a" } }] },
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("error-array-is-an-error"),
			codec,
			{ error: [], choices: [{ delta: { content: "a" } }] },
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("error-false-is-no-error"),
			codec,
			{ error: false, choices: [{ delta: { content: "a" } }] },
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("error-zero-is-no-error"),
			codec,
			{ error: 0, choices: [{ delta: { content: "a" } }] },
			data(finish("stop")),
			DONE,
		),
		oneChunk(
			at("error-empty-text-is-no-error"),
			codec,
			{ error: "", choices: [{ delta: { content: "a" } }] },
			data(finish("stop")),
			DONE,
		),
		oneChunk(at("error-empty-object"), codec, { error: {} }),
		oneChunk(at("error-text"), codec, { error: "Plain words." }),
		oneChunk(at("error-fields-mistyped"), codec, {
			error: { code: [], type: 5, message: 6, metadata: "x", resetsAt: {} },
		}),
		oneChunk(at("error-code-text"), codec, { error: { code: "429", message: "Slow." } }),
		oneChunk(at("error-code-fraction"), codec, { error: { code: 429.5, message: "Slow." } }),
		oneChunk(at("error-metadata-reasons-mixed"), codec, {
			error: { code: 403, message: "No.", metadata: { reasons: ["a", 1, null, "b"], provider_name: 7 } },
		}),
		oneChunk(at("error-unknown-fields"), codec, { error: { message: "No.", weird: { a: 1 } }, other: 1 }),
		oneChunk(at("error-with-output-in-the-same-chunk"), codec, {
			error: { message: "No." },
			choices: [{ delta: { content: "late" } }],
		}),
		// serving.
		withFinish({ id: 5, provider: 6, choices: [{ delta: { content: "a" } }] }, "serving-mistyped"),
		withFinish({ id: "", provider: "", choices: [{ delta: { content: "a" } }] }, "serving-empty-texts"),
		withFinish({ id: null, provider: null, choices: [{ delta: { content: "a" } }] }, "serving-null"),
		// Unknown fields and unknown event names.
		withFinish(
			{
				object: "chat.completion.chunk",
				model: "m",
				created: 5,
				system_fingerprint: "x",
				choices: [{ index: 3, logprobs: null, delta: { content: "a", refusal: null, function_call: null } }],
			},
			"unknown-fields",
		),
	];
});

const looseItems: CaseSource[] = LOOSE_CODECS.flatMap(([name, codec]) => [
	streams(
		`${name}-loose-comments-are-skipped`,
		codec,
		comment(" PROCESSING"),
		data(text("Hi")),
		comment(""),
		data(finish("stop")),
		DONE,
	),
	streams(
		`${name}-loose-event-names-are-ignored`,
		codec,
		{ kind: "data", data: JSON.stringify(text("Hi")), event: "message" },
		{ kind: "data", data: JSON.stringify(finish("stop")), event: "something.else" },
		DONE,
	),
	streams(`${name}-loose-comment-after-done`, codec, data(text("Hi")), data(finish("stop")), DONE, comment("late")),
	streams(`${name}-loose-two-dones`, codec, data(text("Hi")), data(finish("stop")), DONE, DONE),
	streams(`${name}-loose-done-first`, codec, DONE),
	streams(`${name}-loose-only-comments`, codec, comment("a"), comment("b")),
]);

const looseReplies: CaseSource[] = LOOSE_CODECS.flatMap(([name, codec]) => {
	const at = (suffix: string) => `${name}-loose-reply-${suffix}`;
	const reply = (label: string, body: unknown) => completes(at(label), codec, body);
	const message = (message: unknown, finishReason: unknown = "stop") => ({
		choices: [{ message, finish_reason: finishReason }],
	});
	return [
		reply("null", null),
		reply("text", "x"),
		reply("number", 5),
		reply("boolean", true),
		reply("array", []),
		reply("array-of-replies", [message({ content: "a" })]),
		reply("empty-object", {}),
		reply("no-choices", { id: "gen-1" }),
		reply("choices-empty", { choices: [] }),
		reply("choices-text", { choices: "x" }),
		reply("choices-null", { choices: null }),
		reply("choice-null", { choices: [null] }),
		reply("choice-empty", { choices: [{}] }),
		reply("finish-only", { choices: [{ finish_reason: "stop" }] }),
		reply("finish-empty-text", message({ content: "a" }, "")),
		reply("finish-null", message({ content: "a" }, null)),
		reply("finish-number", message({ content: "a" }, 5)),
		reply("finish-unknown-name", message({ content: "a" }, "function_call")),
		reply("message-null", { choices: [{ message: null, finish_reason: "stop" }] }),
		reply("message-text", { choices: [{ message: "x", finish_reason: "stop" }] }),
		reply("message-only-first-choice", {
			choices: [
				{ message: { content: "first" }, finish_reason: "stop" },
				{ message: { content: "second" }, finish_reason: "stop" },
			],
		}),
		reply("content-null", message({ content: null })),
		reply("content-missing", message({})),
		reply("content-number", message({ content: 5 })),
		reply(
			"content-segments",
			message({
				content: [
					{ type: "text", text: "a" },
					{ type: "text", text: "b" },
				],
			}),
		),
		reply(
			"content-nested-segments",
			message({ content: { content: [{ text: "a" }, { content: [{ text: "b" }] }] } }),
		),
		reply("tool-calls-text", message({ content: "a", tool_calls: "x" })),
		reply("tool-calls-junk-entries", message({ content: "a", tool_calls: [null, 5, "x"] })),
		reply("tool-call-without-function", message({ content: "a", tool_calls: [{ id: "c", type: "function" }] })),
		reply("tool-call-function-text", message({ content: "a", tool_calls: [{ id: "c", function: "text" }] })),
		reply(
			"tool-call-fields-mistyped",
			message({ content: "a", tool_calls: [{ id: 5, function: { name: 6, arguments: 7 } }] }),
		),
		reply(
			"tool-call-fields-null",
			message({ content: "a", tool_calls: [{ id: null, function: { name: null, arguments: null } }] }),
		),
		reply(
			"tool-call-without-id-gets-one",
			message({ tool_calls: [{ function: { name: "read", arguments: "{}" } }] }, "tool_calls"),
		),
		reply(
			"tool-call-arguments-object",
			message({ tool_calls: [{ id: "c", function: { name: "read", arguments: { a: 1 } } }] }, "tool_calls"),
		),
		reply(
			"tool-call-arguments-malformed",
			message({ tool_calls: [{ id: "c", function: { name: "read", arguments: "{not json" } }] }, "tool_calls"),
		),
		reply(
			"tool-call-arguments-empty",
			message({ tool_calls: [{ id: "c", function: { name: "read", arguments: "" } }] }, "tool_calls"),
		),
		reply(
			"tool-calls-two",
			message({ tool_calls: [wireCall("c1", "a", "{}"), wireCall("c2", "b", '{"x":1}')] }, "tool_calls"),
		),
		reply(
			"tool-calls-same-id-twice",
			message({ tool_calls: [wireCall("c1", "a", "{}"), wireCall("c1", "b", "{}")] }, "tool_calls"),
		),
		reply("reasoning-fields", message({ content: "a", reasoning: "R", reasoning_content: "RC" })),
		reply(
			"reasoning-details-junk",
			message({
				content: "a",
				reasoning_details: [null, { type: "reasoning.text", text: "T", signature: "s" }, { text: "no type" }],
			}),
		),
		reply("usage-null", { ...message({ content: "a" }), usage: null }),
		reply("usage-empty", { ...message({ content: "a" }), usage: {} }),
		reply("usage-mistyped", { ...message({ content: "a" }), usage: { prompt_tokens: "1", completion_tokens: [] } }),
		reply("usage-without-choices", { usage: { prompt_tokens: 1, completion_tokens: 1 } }),
		reply("error-null-is-no-error", { error: null, ...message({ content: "a" }) }),
		reply("error-false-is-no-error", { error: false, ...message({ content: "a" }) }),
		reply("error-zero-is-no-error", { error: 0, ...message({ content: "a" }) }),
		reply("error-empty-text-is-no-error", { error: "", ...message({ content: "a" }) }),
		reply("error-true-is-an-error", { error: true, ...message({ content: "a" }) }),
		reply("error-number-is-an-error", { error: 5, ...message({ content: "a" }) }),
		reply("error-array-is-an-error", { error: [], ...message({ content: "a" }) }),
		reply("error-empty-object", { error: {} }),
		reply("error-text", { error: "Plain words." }),
		reply("error-with-choices", { error: { message: "No." }, ...message({ content: "late" }) }),
		reply("error-fields-mistyped", { error: { code: [], type: 5, message: 6 } }),
		reply("serving-mistyped", { id: 5, provider: 6, ...message({ content: "a" }) }),
		reply("serving-only", { id: "gen-3", provider: "Azure", choices: [{ finish_reason: "stop" }] }),
		reply("unknown-fields", {
			object: "chat.completion",
			created: 5,
			model: "m",
			system_fingerprint: "x",
			...message({ content: "a", refusal: null }),
		}),
	];
});

////////////////////////////////
//  Failure bodies the same in every dialect

const looseFailures: CaseSource[] = LOOSE_CODECS.flatMap(([name, codec]) => [
	fails(`${name}-loose-http-body-null`, codec, 500, null),
	fails(`${name}-loose-http-body-number`, codec, 500, 5),
	fails(`${name}-loose-http-body-text`, codec, 500, "plain"),
	fails(`${name}-loose-http-body-array`, codec, 500, [1]),
	fails(`${name}-loose-http-body-empty-object`, codec, 500, {}),
	fails(`${name}-loose-http-error-object-mistyped`, codec, 400, {
		error: { code: "x", type: 1, message: [], metadata: 3, resetsAt: true },
	}),
	fails(`${name}-loose-http-error-nested-error`, codec, 400, { error: { error: { message: "inner" } } }),
	fails(`${name}-loose-http-message-with-newlines`, codec, 400, { error: { message: "line one\nline two" } }),
	fails(`${name}-loose-http-unknown-fields`, codec, 400, {
		error: { message: "No.", param: "x", extra: { a: 1 } },
		request_id: "r",
	}),
	fails(`${name}-loose-http-headers-repeat`, codec, 429, { error: { message: "Slow." } }, [
		["x-a", "1"],
		["X-A", "2"],
	]),
]);

////////////////////////////////
//  Whole-reply successes the same in every dialect

const replies: CaseSource[] = LOOSE_CODECS.flatMap(([name, codec]) => [
	completes(`${name}-reply-text`, codec, {
		id: "gen-1",
		choices: [{ message: { role: "assistant", content: "Hello." }, finish_reason: "stop" }],
		usage: { prompt_tokens: 4, completion_tokens: 2 },
	}),
	completes(`${name}-reply-calls-with-text`, codec, {
		id: "gen-1",
		choices: [
			{
				message: { content: "Reading.", tool_calls: [wireCall("call_a", "read", '{"path":"a"}')] },
				finish_reason: "tool_calls",
			},
		],
	}),
	completes(`${name}-reply-length`, codec, { choices: [{ message: { content: "Cut" }, finish_reason: "length" }] }),
	completes(`${name}-reply-content-filter`, codec, { choices: [{ message: {}, finish_reason: "content_filter" }] }),
	completes(`${name}-reply-no-output-stop`, codec, {
		choices: [{ message: { content: "" }, finish_reason: "stop" }],
	}),
]);

////////////////////////////////
//  Cases

export const codecCases: CaseSource[] = [
	...conformance,
	...everyDialect,
	...cacheMarkers,
	...forcedAndSchemas,
	...keyOrder,
	...openrouterCases,
	...opencodeCases,
	...localCases,
	...looseStreams,
	...looseItems,
	...looseReplies,
	...looseFailures,
	...replies,
	...responsesCases,
	...messagesCases,
	...geminiCases,
	...hostCases,
];
