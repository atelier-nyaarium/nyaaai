// Builders and conformance fixtures for the native-family codec cases. Values are made up.

import { DONE, data, FIXTURES, type Fixture, type Json } from "./codecFixtures";

////////////////////////////////
//  Constants

export const GPT = "gpt-6-luna";
export const HAIKU = "claude-haiku-5-5";
export const FLASH = "gemini-3.8-flash";

/** A scripted instant: 2027-01-15T08:00:00Z. */
export const NOW = 1_800_000_000_000;

const FULL_CHOICE = ["auto", "none", "required", "tool"];

const OVERLOADED = "[overloaded] the server is overloaded; retry shortly";
const READ_ARGS = '{"path":"a"}';
const CALL = { kind: "call", index: 0, wireId: "call_a", name: "read", argumentsText: READ_ARGS };

/** What OpenCode reports, cache writes summed from its token block. */
const BILLED = {
	input: 100,
	output: 10,
	cached: 60,
	cacheWrite: 7,
	reasoning: 4,
	cost: 0.002,
	billingSource: "go",
};

////////////////////////////////
//  Functions & Helpers

type Over = { efforts?: string[]; api?: Json; nowMs?: number; outputLimit?: number | undefined; bare?: boolean };

function named(family: string, prefix: string, api: Json, efforts: string[], over: Over): Json {
	return {
		family,
		dialect: "opencode",
		prefix,
		...(over.bare === true
			? {}
			: {
					facts: {
						api: { family, ...api, ...over.api },
						efforts: over.efforts ?? efforts,
						...(over.outputLimit === undefined ? {} : { outputLimit: over.outputLimit }),
					},
				}),
		...(over.nowMs === undefined ? {} : { nowMs: over.nowMs }),
	};
}

/** The Responses codec behind the plan's host. */
export const responses = (over: Over = {}): Json =>
	named(
		"responses",
		"/openai/v1",
		{ structured: true, choice: FULL_CHOICE, strict: true },
		["none", "low", "high"],
		over,
	);

/** The Messages codec behind the plan's host. */
export const messages = (over: Over = {}): Json =>
	named(
		"messages",
		"/anthropic/v1",
		{ structured: true, choice: FULL_CHOICE, strict: true, toggle: true },
		["low", "high"],
		{ outputLimit: 128_000, ...over },
	);

/** The generateContent codec behind the plan's host. */
export const gemini = (over: Over = {}): Json =>
	named(
		"generate-content",
		"/google/v1beta",
		{ structured: true, choice: FULL_CHOICE, strict: false },
		["low", "medium", "high"],
		over,
	);

/** A Responses or Messages event: the `event:` line names the type too. */
export function event(type: string, fields: Json = {}): Json {
	return { kind: "data", event: type, data: JSON.stringify({ type, ...fields }) };
}

/** An event whose data holds no type; the `event:` line names it. */
export function eventNamed(type: string, fields: Json = {}): Json {
	return { kind: "data", event: type, data: JSON.stringify(fields) };
}

/** A generateContent chunk. */
export function chunk(parts: unknown[], over: Json = {}, top: Json = {}): Json {
	return data({ candidates: [{ content: { role: "model", parts }, ...over }], responseId: "resp_g", ...top });
}

////////////////////////////////
//  Responses fixture

const RESPONSES_USAGE = {
	input_tokens: 100,
	input_tokens_details: { cached_tokens: 60 },
	output_tokens: 10,
	output_tokens_details: { reasoning_tokens: 4 },
	cost: 0.002,
	opencode: { billing_source: "go", tokens: { cache_write_5m: 5, cache_write_1h: 2 } },
};

export const SUMMARIZED = {
	type: "reasoning",
	id: "rs_1",
	summary: [{ type: "summary_text", text: "Thinking." }],
	encrypted_content: "enc_1",
};
export const SEALED = { type: "reasoning", id: "rs_2", summary: [], encrypted_content: "enc_2" };
export const COMMENTARY = {
	type: "message",
	id: "msg_1",
	role: "assistant",
	phase: "commentary",
	content: [{ type: "output_text", text: "Reading." }],
};
export const FUNCTION_CALL = {
	type: "function_call",
	id: "fc_1",
	call_id: "call_a",
	name: "read",
	arguments: READ_ARGS,
};

const RESPONSES_STREAM = [
	event("response.created", { response: { id: "resp_1", status: "in_progress" } }),
	event("response.output_item.added", { item: { ...SUMMARIZED, summary: [], encrypted_content: null } }),
	event("response.reasoning_summary_text.delta", { item_id: "rs_1", summary_index: 0, delta: "Think" }),
	event("response.reasoning_summary_text.delta", { item_id: "rs_1", summary_index: 0, delta: "ing." }),
	event("response.output_item.done", { item: SUMMARIZED }),
	event("response.output_item.added", { item: { ...COMMENTARY, content: [] } }),
	event("response.output_text.delta", { item_id: "msg_1", delta: "Reading." }),
	event("response.output_item.done", { item: COMMENTARY }),
	event("response.output_item.added", { item: { ...SEALED, encrypted_content: null } }),
	event("response.output_item.done", { item: SEALED }),
	event("response.output_item.added", { item: { ...FUNCTION_CALL, arguments: "" } }),
	event("response.function_call_arguments.delta", { item_id: "fc_1", delta: '{"path":' }),
	event("response.function_call_arguments.delta", { item_id: "fc_1", delta: '"a"}' }),
	event("response.function_call_arguments.done", { item_id: "fc_1", arguments: READ_ARGS }),
	event("response.output_item.done", { item: FUNCTION_CALL }),
];

export const RESPONSES_FIXTURE: Fixture = {
	name: "responses",
	codec: responses({ efforts: ["low", "high"] }),
	model: GPT,
	foreign: "gpt-5.5",
	stream: [
		...RESPONSES_STREAM,
		event("response.completed", { response: { id: "resp_1", status: "completed", usage: RESPONSES_USAGE } }),
	],
	reply: {
		id: "resp_1",
		status: "completed",
		output: [SUMMARIZED, COMMENTARY, SEALED, FUNCTION_CALL],
		usage: RESPONSES_USAGE,
	},
	result: {
		parts: [
			{
				kind: "thought",
				type: "reasoning",
				segments: [{ kind: "summary", text: "Thinking." }],
				opaque: "enc_1",
				id: "rs_1",
			},
			{ kind: "text", text: "Reading.", phase: "commentary" },
			{ kind: "thought", type: "reasoning", opaque: "enc_2", id: "rs_2" },
			CALL,
		],
		finish: "tool_calls",
		usage: BILLED,
		serving: { generationId: "resp_1" },
	},
	cut: RESPONSES_STREAM,
	failing: [
		RESPONSES_STREAM[0] ?? DONE,
		event("response.output_text.delta", { item_id: "msg_1", delta: "Hi" }),
		event("response.failed", {
			response: {
				id: "resp_1",
				status: "failed",
				error: { code: "server_error", message: OVERLOADED },
				usage: { input_tokens: 4, output_tokens: 2, cost: 0.1 },
			},
		}),
	],
	failures: [
		[401, { error: { message: "Invalid API key.", type: "invalid_request_error" } }, "keyRejected"],
		[429, { type: "error", error: { type: "GoUsageLimitError", message: "Usage limit reached." } }, "allowance"],
		[400, { error: { message: "Model is unavailable.", type: "invalid_request_error" } }, "notFound"],
	],
};

////////////////////////////////
//  Messages fixture

const MESSAGE_START = event("message_start", {
	message: {
		id: "msg_1",
		type: "message",
		role: "assistant",
		content: [],
		model: HAIKU,
		usage: { input_tokens: 40, cache_read_input_tokens: 60, output_tokens: 1 },
	},
});

export const blockDelta = (index: number, delta: Json): Json => event("content_block_delta", { index, delta });

const MESSAGES_STREAM = [
	MESSAGE_START,
	event("content_block_start", { index: 0, content_block: { type: "thinking", thinking: "", signature: "" } }),
	blockDelta(0, { type: "thinking_delta", thinking: "Think" }),
	event("ping"),
	blockDelta(0, { type: "thinking_delta", thinking: "ing." }),
	blockDelta(0, { type: "signature_delta", signature: "sig_1" }),
	event("content_block_stop", { index: 0 }),
	event("content_block_start", { index: 1, content_block: { type: "text", text: "" } }),
	blockDelta(1, { type: "text_delta", text: "Reading." }),
	event("content_block_stop", { index: 1 }),
	event("content_block_start", { index: 2, content_block: { type: "redacted_thinking", data: "red_2" } }),
	event("content_block_stop", { index: 2 }),
	event("content_block_start", {
		index: 3,
		content_block: { type: "tool_use", id: "call_a", name: "read", input: {} },
	}),
	blockDelta(3, { type: "input_json_delta", partial_json: '{"path":' }),
	blockDelta(3, { type: "input_json_delta", partial_json: '"a"}' }),
	event("content_block_stop", { index: 3 }),
];

const MESSAGES_BILLING = {
	cost: 0.002,
	opencode: { billing_source: "go", tokens: { reasoning: 4, cache_write_5m: 5, cache_write_1h: 2 } },
};

export const MESSAGES_FIXTURE: Fixture = {
	name: "messages",
	codec: messages(),
	model: HAIKU,
	foreign: "qwen3.8-flash",
	stream: [
		...MESSAGES_STREAM,
		event("message_delta", {
			delta: { stop_reason: "tool_use", stop_sequence: null },
			usage: { output_tokens: 10, ...MESSAGES_BILLING },
		}),
		event("message_stop"),
	],
	reply: {
		id: "msg_1",
		type: "message",
		role: "assistant",
		model: HAIKU,
		content: [
			{ type: "thinking", thinking: "Thinking.", signature: "sig_1" },
			{ type: "text", text: "Reading." },
			{ type: "redacted_thinking", data: "red_2" },
			{ type: "tool_use", id: "call_a", name: "read", input: { path: "a" } },
		],
		stop_reason: "tool_use",
		usage: { input_tokens: 40, cache_read_input_tokens: 60, output_tokens: 10, ...MESSAGES_BILLING },
	},
	result: {
		parts: [
			{ kind: "thought", type: "thinking", segments: [{ kind: "content", text: "Thinking." }], opaque: "sig_1" },
			{ kind: "text", text: "Reading." },
			{ kind: "thought", type: "redacted_thinking", opaque: "red_2" },
			CALL,
		],
		finish: "tool_calls",
		usage: BILLED,
		serving: { generationId: "msg_1" },
	},
	cut: MESSAGES_STREAM,
	failing: [
		MESSAGE_START,
		event("content_block_start", { index: 0, content_block: { type: "text", text: "" } }),
		blockDelta(0, { type: "text_delta", text: "Hi" }),
		event("error", { error: { type: "overloaded_error", message: "Overloaded" } }),
	],
	failures: [
		[401, { type: "error", error: { type: "authentication_error", message: "invalid x-api-key" } }, "keyRejected"],
		[429, { type: "error", error: { type: "GoUsageLimitError", message: "Usage limit reached." } }, "allowance"],
		[529, { type: "error", error: { type: "overloaded_error", message: "Overloaded" } }, "unavailable"],
	],
};

////////////////////////////////
//  Gemini fixture

const PROMPT_USAGE = { promptTokenCount: 100, cachedContentTokenCount: 60 };

const GEMINI_USAGE = {
	...PROMPT_USAGE,
	candidatesTokenCount: 6,
	thoughtsTokenCount: 4,
	cost: 0.002,
	opencode: { billing_source: "go", tokens: { cache_write_5m: 5, cache_write_1h: 2 } },
};

export const SIGNED_CALL = {
	functionCall: { id: "call_a", name: "read", args: { path: "a" } },
	thoughtSignature: "sig_g",
};

const GEMINI_STREAM = [
	data({
		candidates: [{ content: { role: "model", parts: [{ text: "Think", thought: true }] } }],
		usageMetadata: PROMPT_USAGE,
		responseId: "resp_g",
	}),
	chunk([{ text: "ing.", thought: true }]),
	chunk([{ text: "Reading." }]),
];

export const GEMINI_FIXTURE: Fixture = {
	name: "gemini",
	codec: gemini(),
	model: FLASH,
	foreign: "gemini-4-argon",
	stream: [
		...GEMINI_STREAM,
		data({
			candidates: [{ content: { role: "model", parts: [SIGNED_CALL] }, finishReason: "STOP" }],
			usageMetadata: GEMINI_USAGE,
			responseId: "resp_g",
		}),
	],
	reply: {
		candidates: [
			{
				content: {
					role: "model",
					parts: [{ text: "Thinking.", thought: true }, { text: "Reading." }, SIGNED_CALL],
				},
				finishReason: "STOP",
			},
		],
		usageMetadata: GEMINI_USAGE,
		responseId: "resp_g",
	},
	result: {
		parts: [
			{ kind: "thought", type: "thought", segments: [{ kind: "content", text: "Thinking." }] },
			{ kind: "text", text: "Reading." },
			{ kind: "thought", type: "thought", opaque: "sig_g", anchor: "call:0" },
			CALL,
		],
		finish: "tool_calls",
		usage: BILLED,
		serving: { generationId: "resp_g" },
	},
	cut: GEMINI_STREAM,
	failing: [GEMINI_STREAM[0] ?? DONE, data({ error: { code: 503, message: OVERLOADED, status: "UNAVAILABLE" } })],
	failures: [
		[401, { error: { code: 401, message: "API key not valid.", status: "UNAUTHENTICATED" } }, "keyRejected"],
		[429, { type: "error", error: { type: "GoUsageLimitError", message: "Usage limit reached." } }, "allowance"],
		[400, { error: { code: 400, message: "Model is unavailable.", status: "INVALID_ARGUMENT" } }, "notFound"],
	],
};

export const NATIVE_FIXTURES: Fixture[] = [RESPONSES_FIXTURE, MESSAGES_FIXTURE, GEMINI_FIXTURE];

/** Every fixture the conformance cases run. */
export const ALL_FIXTURES: Fixture[] = [...FIXTURES, ...NATIVE_FIXTURES];
