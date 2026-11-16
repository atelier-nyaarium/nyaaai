// Codec cases: the Messages codec.

import type { CaseSource } from "../caseAdapter";
import { comment, DONE, data, type Json } from "./codecFixtures";
import { completes, encodes, fails, streams } from "./codecKit";
import { blockDelta, event, eventNamed, HAIKU, messages, NOW } from "./codecNativeFixtures";

////////////////////////////////
//  Fixtures

const FINISH_TOOL = { name: "finish", description: "Finish.", parameters: { type: "object" } };

const TITLE_SCHEMA = {
	name: "title",
	json: { type: "object", properties: { title: { type: "string" } }, required: ["title"] },
};

const SCRAMBLED = {
	name: "scrambled",
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

const asks = (name: string, codec: Json, over: Json = {}, mode: "stream" | "complete" = "complete"): CaseSource =>
	encodes(name, codec, { model: HAIKU, ...over }, mode);

const SIGNED = {
	kind: "thought",
	type: "thinking",
	segments: [{ kind: "content", text: "Plan." }],
	opaque: "sig",
};

const UNSIGNED = { kind: "thought", type: "thinking", segments: [{ kind: "content", text: "?" }] };

const REDACTED = { kind: "thought", type: "redacted_thinking", opaque: "red" };

const stepOf = (parts: Json[], model: string | null = HAIKU): Json => ({
	role: "step",
	...(model === null ? {} : { producer: { model } }),
	parts,
});

/** A restored read's step, thinking first. */
function readStep(model: string | null, thoughts: Json[] = [SIGNED], wireId = "run-1/restored/0"): Json {
	return stepOf(
		[
			...thoughts,
			{ kind: "call", index: 0, wireId, name: "read", arguments: '{"file":"a"}' },
			{ kind: "result", index: 0, output: "missing", isError: true },
		],
		model,
	);
}

const start = (usage: unknown = { input_tokens: 4, output_tokens: 1 }, id: unknown = "msg_1"): Json =>
	event("message_start", { message: { id, usage } });

const textBlock = (index = 0, initial = ""): Json =>
	event("content_block_start", { index, content_block: { type: "text", text: initial } });

const stop = (reason: unknown, usage?: unknown): Json =>
	event("message_delta", { delta: { stop_reason: reason }, usage });

const STOP = event("message_stop");

/** A short stream ending on a stop reason. */
const stopping = (reason: unknown): Json[] => [start(), textBlock(0, "Hi"), stop(reason, { output_tokens: 2 }), STOP];

const toolBlock = (index: number, id: string, name: string): Json =>
	event("content_block_start", { index, content_block: { type: "tool_use", id, name, input: {} } });

const BILLING = {
	cost: 0.002,
	opencode: {
		billing_source: "go",
		list_cost: 0.01,
		allowance: 0.5,
		tokens: { reasoning: 4, cache_write_5m: 5, cache_write_1h: 2 },
	},
};

////////////////////////////////
//  Requests

const requests: CaseSource[] = [
	asks(
		"messages-request-full",
		messages(),
		{
			system: ["Sections.", "Rules."],
			summary: "Earlier.",
			history: [{ role: "user", text: "Read it." }, readStep(HAIKU)],
			state: "Live.",
			effort: "high",
			tools: [FINISH_TOOL],
			toolChoice: { name: "finish" },
			schema: TITLE_SCHEMA,
			cache: { kind: "explicit", ttl: "1h" },
		},
		"stream",
	),
	asks("messages-request-complete-path", messages(), {}, "complete"),
	asks("messages-request-no-facts", messages({ bare: true }), {
		effort: "high",
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
		schema: TITLE_SCHEMA,
	}),
	asks("messages-request-no-system", messages(), { system: [], history: [] }),
	asks("messages-request-empty-system-blocks-are-dropped", messages(), { system: ["", "One.", ""], summary: "" }),
	asks("messages-request-tools-empty-list", messages(), { tools: [] }),
	// The token ceiling: the request's, the model's, then the default.
	asks("messages-ceiling-from-the-request", messages(), { maxTokens: 900 }),
	asks("messages-ceiling-from-the-model", messages(), {}),
	asks("messages-ceiling-default", messages({ outputLimit: undefined, bare: false }), {}),
	asks("messages-ceiling-default-without-facts", messages({ bare: true }), {}),
	asks("messages-ceiling-request-wins-over-the-model", messages({ outputLimit: 8000 }), { maxTokens: 64_000 }),
	// Forcing follows the catalog.
	asks("messages-choice-required-is-any", messages({ api: { choice: ["auto", "required"] } }), {
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
	}),
	asks("messages-choice-tool-wins-over-required", messages({ api: { choice: ["required", "tool"] } }), {
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
	}),
	asks("messages-choice-auto-only-sends-no-choice", messages({ api: { choice: ["auto"] } }), {
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
	}),
	asks("messages-choice-not-asked", messages(), { tools: [FINISH_TOOL] }),
	// Schemas follow the catalog.
	asks("messages-schema-enforced", messages(), { schema: TITLE_SCHEMA }),
	asks("messages-schema-enforced-with-effort", messages(), { schema: TITLE_SCHEMA, effort: "low" }),
	asks("messages-schema-in-prompt", messages({ api: { structured: false } }), { schema: TITLE_SCHEMA }),
	asks("messages-schema-in-prompt-after-summary", messages({ api: { structured: false } }), {
		schema: TITLE_SCHEMA,
		summary: "Earlier.",
	}),
	asks("messages-schema-refused-by-ref", messages(), {
		schema: { name: "tree", json: { $ref: "#/$defs/node", $defs: { node: { type: "object" } } } },
	}),
	asks("messages-schema-refused-optional-field", messages(), {
		schema: { name: "pick", json: { type: "object", properties: { a: { type: "string" } } } },
	}),
	// Thinking and effort follow the model.
	asks("messages-thinking-adaptive-for-claude", messages(), {}),
	asks("messages-thinking-adaptive-for-a-prefixed-claude", messages(), { model: "anthropic/claude-haiku-5.5" }),
	asks("messages-thinking-none-for-another-vendor", messages(), { model: "qwen3.8-flash", effort: "high" }),
	asks("messages-thinking-none-when-no-efforts", messages({ efforts: [] }), { effort: "high" }),
	asks("messages-effort-listed", messages(), { effort: "low" }),
	asks("messages-effort-not-listed", messages(), { effort: "medium" }),
	asks("messages-effort-none-when-not-asked", messages(), {}),
	// Cache breakpoints.
	asks("messages-cache-explicit", messages(), {
		system: ["Sections.", "Rules."],
		summary: "Earlier.",
		cache: { kind: "explicit" },
	}),
	asks("messages-cache-explicit-1h", messages(), {
		system: ["Sections.", "Rules."],
		summary: "Earlier.",
		cache: { kind: "explicit", ttl: "1h" },
	}),
	asks("messages-cache-implicit-marks-nothing", messages(), {
		cache: { kind: "implicit", ttl: "1h" },
		summary: "Earlier.",
	}),
	asks("messages-cache-none-marks-nothing", messages(), { cache: { kind: "none" }, summary: "Earlier." }),
	asks("messages-cache-explicit-no-system-marks-the-summary", messages(), {
		system: [],
		summary: "Earlier.",
		cache: { kind: "explicit" },
	}),
	asks("messages-cache-explicit-no-system-no-summary", messages(), { system: [], cache: { kind: "explicit" } }),
	asks("messages-cache-explicit-marks-the-last-user-text", messages(), {
		history: [
			{ role: "user", text: "One." },
			stepOf([{ kind: "text", text: "Two." }]),
			{ role: "user", text: "Three." },
		],
		cache: { kind: "explicit" },
	}),
	asks("messages-cache-explicit-marks-the-last-tool-result", messages(), {
		history: [{ role: "user", text: "Read." }, readStep(HAIKU)],
		cache: { kind: "explicit" },
	}),
	asks("messages-cache-explicit-marks-before-the-state", messages(), {
		history: [{ role: "user", text: "Read." }],
		state: "Live.",
		cache: { kind: "explicit" },
	}),
	asks("messages-cache-explicit-nothing-from-the-user", messages(), {
		history: [stepOf([{ kind: "text", text: "Hi." }])],
		cache: { kind: "explicit" },
	}),
	asks("messages-cache-explicit-empty-history", messages(), { history: [], cache: { kind: "explicit" } }),
	// Key order.
	asks("messages-key-order-schema-enforced", messages(), { schema: SCRAMBLED }),
	asks("messages-key-order-schema-in-prompt", messages({ api: { structured: false } }), { schema: SCRAMBLED }),
	asks("messages-key-order-tools", messages(), { tools: [SCRAMBLED_TOOL], toolChoice: { name: "scramble" } }),
	asks("messages-key-order-call-input", messages(), {
		history: [
			stepOf([
				{
					kind: "call",
					index: 0,
					wireId: "toolu_a",
					name: "read",
					arguments: '{"zeta":1,"alpha":{"b":1,"a":2},"10":3,"2":4}',
				},
				{ kind: "result", index: 0, output: "x", isError: false },
			]),
		],
	}),
];

////////////////////////////////
//  History and thought replay

const history: CaseSource[] = [
	// Thinking goes back to its own model, or between Claude models.
	asks("messages-replay-same-model", messages(), { history: [readStep(HAIKU)] }),
	asks("messages-replay-between-two-models-of-one-vendor", messages(), {
		model: "claude-opus-5-5",
		history: [readStep(HAIKU)],
	}),
	asks("messages-replay-between-prefixed-claude-names", messages(), {
		model: "anthropic/claude-opus-5.5",
		history: [readStep("anthropic/claude-haiku-5.5")],
	}),
	asks("messages-replay-from-another-vendor", messages(), { history: [readStep("qwen3.8-flash")] }),
	asks("messages-replay-to-another-vendor", messages(), { model: "qwen3.8-flash", history: [readStep(HAIKU)] }),
	asks("messages-replay-same-model-of-another-vendor", messages(), {
		model: "qwen3.8-flash",
		history: [readStep("qwen3.8-flash")],
	}),
	asks("messages-replay-no-producer", messages(), { history: [readStep(null)] }),
	asks("messages-replay-unsigned-thought-drops-the-step-thoughts", messages(), {
		history: [readStep(HAIKU, [SIGNED, UNSIGNED])],
	}),
	asks("messages-replay-only-an-unsigned-thought", messages(), { history: [readStep(HAIKU, [UNSIGNED])] }),
	asks("messages-replay-redacted-thought", messages(), { history: [readStep(HAIKU, [SIGNED, REDACTED])] }),
	asks("messages-replay-redacted-thought-unsigned", messages(), {
		history: [readStep(HAIKU, [{ kind: "thought", type: "redacted_thinking" }])],
	}),
	asks("messages-replay-other-thought-type-drops-the-step-thoughts", messages(), {
		history: [readStep(HAIKU, [{ ...SIGNED, type: "reasoning.text" }])],
	}),
	asks("messages-replay-segments-join-without-a-gap", messages(), {
		history: [
			readStep(HAIKU, [
				{
					kind: "thought",
					type: "thinking",
					opaque: "sig",
					segments: [
						{ kind: "content", text: "One " },
						{ kind: "summary", text: "two " },
						{ kind: "content", text: "three." },
					],
				},
			]),
		],
	}),
	asks("messages-replay-thought-without-segments", messages(), {
		history: [readStep(HAIKU, [{ kind: "thought", type: "thinking", opaque: "sig" }])],
	}),
	asks("messages-replay-a-step-with-no-thoughts", messages(), { history: [stepOf([{ kind: "text", text: "Hi." }])] }),
	asks("messages-replay-dropped-thoughts-keep-text-and-calls", messages(), {
		history: [
			stepOf(
				[
					UNSIGNED,
					{ kind: "text", text: "Reading." },
					{ kind: "call", index: 0, wireId: "toolu_a", name: "read", arguments: "{}" },
					{ kind: "result", index: 0, output: "a", isError: false },
				],
				"qwen3.8-flash",
			),
		],
	}),
	// Calls, results and the messages they make.
	asks("messages-parallel-calls-in-one-assistant-message", messages(), {
		history: [
			{ role: "user", text: "Read." },
			stepOf([
				{ kind: "call", index: 0, wireId: "toolu_a", name: "read", arguments: '{"file":"a"}' },
				{ kind: "call", index: 1, wireId: "toolu_b", name: "read", arguments: '{"file":"b"}' },
				{ kind: "result", index: 0, output: "a", isError: false },
				{ kind: "result", index: 1, output: "b", isError: false },
			]),
		],
	}),
	asks("messages-results-pair-by-index", messages(), {
		history: [
			stepOf([
				{ kind: "call", index: 1, wireId: "toolu_x", name: "read", arguments: "{}" },
				{ kind: "call", index: 0, wireId: "toolu_y", name: "read", arguments: "{}" },
				{ kind: "result", index: 0, output: "second", isError: false },
				{ kind: "result", index: 1, output: "first", isError: true },
			]),
		],
	}),
	asks("messages-result-of-unknown-call-is-dropped", messages(), {
		history: [
			stepOf([
				{ kind: "call", index: 0, wireId: "toolu_a", name: "read", arguments: "{}" },
				{ kind: "result", index: 3, output: "lost", isError: false },
			]),
		],
	}),
	asks("messages-call-without-result", messages(), {
		history: [stepOf([{ kind: "call", index: 0, wireId: "toolu_a", name: "read", arguments: "{}" }])],
	}),
	asks("messages-call-arguments-not-json", messages(), {
		history: [stepOf([{ kind: "call", index: 0, wireId: "toolu_a", name: "read", arguments: "{not json" }])],
	}),
	asks("messages-call-arguments-not-an-object", messages(), {
		history: [
			stepOf([
				{ kind: "call", index: 0, wireId: "toolu_a", name: "read", arguments: "[1,2]" },
				{ kind: "call", index: 1, wireId: "toolu_b", name: "read", arguments: "5" },
				{ kind: "call", index: 2, wireId: "toolu_c", name: "read", arguments: "" },
			]),
		],
	}),
	asks("messages-text-empty-is-dropped", messages(), {
		history: [
			stepOf([
				{ kind: "text", text: "" },
				{ kind: "text", text: "Hi" },
			]),
		],
	}),
	asks("messages-empty-step-is-dropped", messages(), {
		history: [{ role: "user", text: "Hi" }, stepOf([]), { role: "user", text: "Again" }],
	}),
	asks("messages-same-role-messages-join", messages(), {
		history: [
			{ role: "user", text: "One." },
			{ role: "user", text: "Two." },
		],
	}),
	asks("messages-user-text-after-results-joins-the-results", messages(), {
		history: [
			{ role: "user", text: "Read." },
			stepOf([
				{ kind: "call", index: 0, wireId: "toolu_a", name: "read", arguments: "{}" },
				{ kind: "result", index: 0, output: "a", isError: false },
			]),
			{ role: "user", text: "Thanks." },
		],
	}),
	asks("messages-two-steps-in-a-row-join-their-assistant-messages", messages(), {
		history: [stepOf([{ kind: "text", text: "One." }]), stepOf([{ kind: "text", text: "Two." }])],
	}),
	asks("messages-state-joins-the-last-user-message", messages(), {
		history: [
			{ role: "user", text: "Read." },
			stepOf([
				{ kind: "call", index: 0, wireId: "toolu_a", name: "read", arguments: "{}" },
				{ kind: "result", index: 0, output: "a", isError: false },
			]),
		],
		state: "Live.",
	}),
	asks("messages-state-after-an-assistant-message-starts-a-user-message", messages(), {
		history: [{ role: "user", text: "Hi." }, stepOf([{ kind: "text", text: "Hello." }])],
		state: "Live.",
	}),
	asks("messages-state-alone", messages(), { history: [], state: "Live." }),
	// Tool ids keep the characters Messages allows.
	...[
		["slashes", "run-1/restored/0"],
		["plain", "toolu_a-B9"],
		["empty", ""],
		["spaces-and-dots", "a b.c"],
		["accent", "caf\u{e9}"],
		["emoji-is-two-units", "x\u{1f600}y"],
		["only-symbols", "///"],
	].map(([name, wireId]) =>
		asks(`messages-tool-id-${name}`, messages(), {
			history: [
				stepOf([
					{ kind: "call", index: 0, wireId: wireId as string, name: "read", arguments: "{}" },
					{ kind: "result", index: 0, output: "a", isError: false },
				]),
			],
		}),
	),
];

////////////////////////////////
//  Stream decoding

const streamsOf: CaseSource[] = [
	// Stop reasons map to finishes.
	...(
		[
			["end-turn", "end_turn"],
			["max-tokens", "max_tokens"],
			["refusal", "refusal"],
			["pause-turn", "pause_turn"],
			["tool-use", "tool_use"],
			["stop-sequence", "stop_sequence"],
			["context-window", "model_context_window_exceeded"],
			["unknown", "unknown_reason"],
			["object-constructor", "constructor"],
			["object-to-string", "toString"],
			["object-proto", "__proto__"],
		] as const
	).map(([name, reason]) => streams(`messages-stream-stop-reason-${name}`, messages(), ...stopping(reason))),
	streams("messages-stream-stop-reason-null-fails", messages(), ...stopping(null)),
	streams("messages-stream-stop-reason-empty-fails", messages(), ...stopping("")),
	streams("messages-stream-stop-reason-mistyped-fails", messages(), ...stopping(7)),
	streams(
		"messages-stream-stop-reason-absent-fails",
		messages(),
		start(),
		textBlock(0, "Hi"),
		event("message_delta", { delta: {} }),
		STOP,
	),
	streams("messages-stream-message-delta-without-a-delta-fails", messages(), start(), event("message_delta"), STOP),
	streams("messages-stream-stop-before-any-delta-fails", messages(), start(), textBlock(0, "Hi"), STOP),
	streams("messages-stream-no-stop-is-truncated", messages(), start(), textBlock(0, "Hi"), stop("end_turn")),
	streams("messages-stream-no-items-is-truncated", messages()),
	streams(
		"messages-stream-done-marker-before-the-stop",
		messages(),
		start(),
		textBlock(0, "Hi"),
		stop("end_turn"),
		DONE,
	),
	streams(
		"messages-stream-items-after-the-stop-are-never-read",
		messages(),
		...stopping("end_turn"),
		textBlock(1, "late"),
		data("{not json"),
	),
	streams(
		"messages-stream-second-finish-replaces-the-first",
		messages(),
		start(),
		stop("max_tokens"),
		stop("end_turn"),
		STOP,
	),
	// Thinking.
	streams(
		"messages-stream-thinking-deltas-and-signature",
		messages(),
		start(),
		event("content_block_start", { index: 0, content_block: { type: "thinking", thinking: "", signature: "" } }),
		blockDelta(0, { type: "thinking_delta", thinking: "Think" }),
		blockDelta(0, { type: "thinking_delta", thinking: "ing." }),
		blockDelta(0, { type: "signature_delta", signature: "sig_1" }),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-thinking-start-text-is-not-read",
		messages(),
		start(),
		event("content_block_start", {
			index: 0,
			content_block: { type: "thinking", thinking: "Early.", signature: "early" },
		}),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-thinking-blocks-by-index",
		messages(),
		start(),
		blockDelta(0, { type: "thinking_delta", thinking: "A" }),
		blockDelta(2, { type: "thinking_delta", thinking: "B" }),
		blockDelta(0, { type: "thinking_delta", thinking: "C" }),
		blockDelta(2, { type: "signature_delta", signature: "sig_b" }),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-thinking-delta-empty-is-no-event",
		messages(),
		start(),
		blockDelta(0, { type: "thinking_delta", thinking: "" }),
		blockDelta(0, { type: "thinking_delta" }),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-signature-delta-empty",
		messages(),
		start(),
		blockDelta(0, { type: "signature_delta", signature: "" }),
		blockDelta(0, { type: "signature_delta" }),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-redacted-thinking",
		messages(),
		start(),
		event("content_block_start", { index: 0, content_block: { type: "redacted_thinking", data: "red_1" } }),
		event("content_block_start", { index: 1, content_block: { type: "redacted_thinking" } }),
		stop("end_turn"),
		STOP,
	),
	// Text.
	streams(
		"messages-stream-text-start-and-deltas",
		messages(),
		start(),
		textBlock(0, "Hel"),
		blockDelta(0, { type: "text_delta", text: "lo" }),
		blockDelta(0, { type: "text_delta" }),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-text-mistyped",
		messages(),
		start(),
		event("content_block_start", { index: 0, content_block: { type: "text", text: 5 } }),
		blockDelta(0, { type: "text_delta", text: ["x"] }),
		stop("end_turn"),
		STOP,
	),
	// Calls.
	streams(
		"messages-stream-call-by-deltas",
		messages(),
		start(),
		toolBlock(3, "call_a", "read"),
		blockDelta(3, { type: "input_json_delta", partial_json: '{"path":' }),
		blockDelta(3, { type: "input_json_delta", partial_json: '"a"}' }),
		stop("tool_use"),
		STOP,
	),
	streams(
		"messages-stream-calls-index-by-arrival-not-by-block",
		messages(),
		start(),
		textBlock(0, "Hi"),
		toolBlock(5, "call_a", "read"),
		toolBlock(2, "call_b", "write"),
		blockDelta(2, { type: "input_json_delta", partial_json: "{}" }),
		blockDelta(5, { type: "input_json_delta", partial_json: "{}" }),
		stop("tool_use"),
		STOP,
	),
	streams(
		"messages-stream-input-delta-for-an-unknown-block-is-ignored",
		messages(),
		start(),
		toolBlock(1, "call_a", "read"),
		blockDelta(9, { type: "input_json_delta", partial_json: "{}" }),
		blockDelta(1, { type: "input_json_delta" }),
		stop("tool_use"),
		STOP,
	),
	streams(
		"messages-stream-tool-block-fields-mistyped",
		messages(),
		start(),
		event("content_block_start", { index: 0, content_block: { type: "tool_use", id: 1, name: [], input: "x" } }),
		stop("tool_use"),
		STOP,
	),
	streams(
		"messages-stream-block-index-mistyped-is-zero",
		messages(),
		start(),
		event("content_block_delta", { index: "1", delta: { type: "thinking_delta", thinking: "A" } }),
		event("content_block_delta", { delta: { type: "thinking_delta", thinking: "B" } }),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-block-index-fractional",
		messages(),
		start(),
		blockDelta(1.5, { type: "thinking_delta", thinking: "A" }),
		blockDelta(-1, { type: "thinking_delta", thinking: "B" }),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-unknown-delta-type-is-ignored",
		messages(),
		start(),
		blockDelta(0, { type: "citations_delta" }),
		blockDelta(0, {}),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-unknown-block-type-is-ignored",
		messages(),
		start(),
		event("content_block_start", { index: 0, content_block: { type: "server_tool_use", id: "s", name: "search" } }),
		event("content_block_start", { index: 1, content_block: {} }),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-block-and-delta-not-objects",
		messages(),
		start(),
		event("content_block_start", { index: 0, content_block: "text" }),
		event("content_block_start", { index: 0 }),
		event("content_block_delta", { index: 0, delta: [1] }),
		event("content_block_delta", { index: 0 }),
		stop("end_turn"),
		STOP,
	),
	// Usage and serving.
	streams(
		"messages-stream-usage-cumulates-across-events",
		messages(),
		start({ input_tokens: 40, cache_read_input_tokens: 60, output_tokens: 1 }),
		textBlock(0, "Hi"),
		stop("end_turn", { output_tokens: 10 }),
		STOP,
	),
	streams(
		"messages-stream-usage-cache-creation-adds-to-the-input",
		messages(),
		start({ input_tokens: 10, cache_creation_input_tokens: 30, cache_read_input_tokens: 5, output_tokens: 1 }),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-usage-needs-both-counts",
		messages(),
		start({ input_tokens: 4 }),
		textBlock(0, "Hi"),
		stop("end_turn", { output_tokens: 3 }),
		STOP,
	),
	streams(
		"messages-stream-usage-counts-mistyped",
		messages(),
		start({ input_tokens: "4", output_tokens: 1 }),
		stop("end_turn", { output_tokens: "2", cache_read_input_tokens: null }),
		STOP,
	),
	streams(
		"messages-stream-usage-cache-counts-mistyped",
		messages(),
		start({ input_tokens: 4, output_tokens: 1, cache_read_input_tokens: "x", cache_creation_input_tokens: [] }),
		stop("end_turn"),
		STOP,
	),
	streams("messages-stream-usage-empty-delta-repeats-the-usage", messages(), start(), stop("end_turn", {}), STOP),
	streams("messages-stream-usage-not-an-object", messages(), start("x"), stop("end_turn", [1]), STOP),
	streams(
		"messages-stream-usage-with-billing",
		messages(),
		start(),
		stop("end_turn", { output_tokens: 10, ...BILLING }),
		STOP,
	),
	streams(
		"messages-stream-billing-source-unknown-and-negatives-are-dropped",
		messages(),
		start(),
		stop("end_turn", {
			output_tokens: 10,
			cost: -1,
			opencode: { billing_source: "gift", list_cost: -2, allowance: -3 },
		}),
		STOP,
	),
	streams(
		"messages-stream-billing-mistyped",
		messages(),
		start(),
		stop("end_turn", {
			output_tokens: 10,
			cost: "1",
			opencode: { billing_source: 5, list_cost: "x", allowance: [], tokens: "y" },
		}),
		STOP,
	),
	streams("messages-stream-serving-from-the-start", messages(), start(undefined, "msg_9"), stop("end_turn"), STOP),
	streams(
		"messages-stream-serving-id-empty-or-mistyped",
		messages(),
		start(undefined, ""),
		start(undefined, 5),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-serving-repeats-for-each-start",
		messages(),
		start(undefined, "msg_1"),
		start(undefined, "msg_2"),
		stop("end_turn"),
		STOP,
	),
	streams("messages-stream-start-without-a-message", messages(), event("message_start"), stop("end_turn"), STOP),
	// Failures.
	streams(
		"messages-stream-error-overloaded",
		messages(),
		start(),
		textBlock(0, "Hi"),
		event("error", { error: { type: "overloaded_error", message: "Overloaded" } }),
	),
	streams(
		"messages-stream-error-allowance",
		messages(),
		start(),
		event("error", { error: { type: "GoUsageLimitError", message: "Usage limit reached." } }),
	),
	streams(
		"messages-stream-error-api-error",
		messages(),
		start(),
		event("error", { error: { type: "api_error", message: "Internal error." } }),
	),
	streams(
		"messages-stream-error-bills-usage-and-serving",
		messages(),
		start({ input_tokens: 40, output_tokens: 1 }, "msg_1"),
		event("error", { error: { type: "api_error", message: "Down." } }),
	),
	streams("messages-stream-error-before-any-start", messages(), event("error", { error: { message: "Early." } })),
	streams("messages-stream-error-no-error-object", messages(), event("error")),
	streams("messages-stream-error-text-is-the-message", messages(), event("error", { error: "It broke." })),
	streams("messages-stream-error-empty-text-has-no-message", messages(), event("error", { error: "" })),
	streams("messages-stream-error-array-has-no-message", messages(), event("error", { error: ["x"] })),
	streams(
		"messages-stream-error-fields-mistyped",
		messages(),
		event("error", { error: { type: 5, message: ["x"] } }),
	),
	streams(
		"messages-stream-error-blank-message",
		messages(),
		event("error", { error: { type: "api_error", message: "   " } }),
	),
	// Event names and comments.
	streams(
		"messages-stream-comments-and-pings-are-ignored",
		messages(),
		comment("ping"),
		start(),
		event("ping"),
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-the-event-line-names-the-type",
		messages(),
		eventNamed("message_start", { message: { id: "msg_1" } }),
		eventNamed("content_block_start", { index: 0, content_block: { type: "text", text: "Hi" } }),
		eventNamed("message_delta", { delta: { stop_reason: "end_turn" } }),
		eventNamed("message_stop", {}),
	),
	streams(
		"messages-stream-the-data-type-wins-over-the-event-line",
		messages(),
		{ kind: "data", event: "ping", data: JSON.stringify({ type: "message_start", message: { id: "msg_1" } }) },
		stop("end_turn"),
		STOP,
	),
	streams(
		"messages-stream-no-type-anywhere-is-ignored",
		messages(),
		data({ message: { id: "msg_1" } }),
		stop("end_turn"),
		STOP,
	),
	streams("messages-stream-type-mistyped-is-ignored", messages(), data({ type: 5 }), stop("end_turn"), STOP),
	streams(
		"messages-stream-unknown-event-types-are-ignored",
		messages(),
		start(),
		event("content_block_stop", { index: 0 }),
		event("something_new"),
		stop("end_turn"),
		STOP,
	),
	streams("messages-stream-chunk-not-json", messages(), data("{not json")),
	streams("messages-stream-chunk-is-an-array", messages(), data("[1]")),
	streams("messages-stream-chunk-is-null", messages(), data("null")),
	streams("messages-stream-chunk-is-empty-text", messages(), data("")),
];

////////////////////////////////
//  Whole replies

const BODY = (over: Json = {}): Json => ({
	id: "msg_1",
	type: "message",
	role: "assistant",
	content: [{ type: "text", text: "Hello." }],
	stop_reason: "end_turn",
	usage: { input_tokens: 4, output_tokens: 2 },
	...over,
});

const replies: CaseSource[] = [
	completes("messages-reply-text", messages(), BODY()),
	completes(
		"messages-reply-everything",
		messages(),
		BODY({
			content: [
				{ type: "thinking", thinking: "Thinking.", signature: "sig_1" },
				{ type: "text", text: "Reading." },
				{ type: "redacted_thinking", data: "red_2" },
				{ type: "tool_use", id: "call_a", name: "read", input: { path: "a" } },
			],
			stop_reason: "tool_use",
			usage: { input_tokens: 40, cache_read_input_tokens: 60, output_tokens: 10, ...BILLING },
		}),
	),
	completes(
		"messages-reply-thinking-without-signature",
		messages(),
		BODY({ content: [{ type: "thinking", thinking: "Hmm." }] }),
	),
	completes(
		"messages-reply-thinking-without-text",
		messages(),
		BODY({ content: [{ type: "thinking", signature: "sig" }] }),
	),
	completes(
		"messages-reply-thinking-empty",
		messages(),
		BODY({ content: [{ type: "thinking", thinking: "", signature: "" }] }),
	),
	completes("messages-reply-redacted-without-data", messages(), BODY({ content: [{ type: "redacted_thinking" }] })),
	completes(
		"messages-reply-call-input-keeps-its-key-order",
		messages(),
		BODY({
			content: [
				{ type: "tool_use", id: "c", name: "read", input: { zeta: 1, alpha: { b: 1, a: 2 }, "10": 3, "2": 4 } },
			],
			stop_reason: "tool_use",
		}),
	),
	completes(
		"messages-reply-call-input-not-an-object",
		messages(),
		BODY({
			content: [
				{ type: "tool_use", id: "c", name: "read", input: "x" },
				{ type: "tool_use", id: "d", name: "read" },
				{ type: "tool_use", id: "e", name: "read", input: [1] },
			],
			stop_reason: "tool_use",
		}),
	),
	completes(
		"messages-reply-two-calls-by-position",
		messages(),
		BODY({
			content: [
				{ type: "text", text: "Hi" },
				{ type: "tool_use", id: "c", name: "read", input: {} },
				{ type: "tool_use", id: "d", name: "write", input: {} },
			],
			stop_reason: "tool_use",
		}),
	),
	completes("messages-reply-stop-max-tokens", messages(), BODY({ stop_reason: "max_tokens" })),
	completes("messages-reply-stop-refusal", messages(), BODY({ stop_reason: "refusal" })),
	completes("messages-reply-stop-null-fails", messages(), BODY({ stop_reason: null })),
	completes("messages-reply-stop-missing-fails", messages(), { id: "msg_1", content: [] }),
	completes("messages-reply-stop-mistyped-fails", messages(), BODY({ stop_reason: 3 })),
	completes("messages-reply-stop-constructor", messages(), BODY({ stop_reason: "constructor" })),
	completes("messages-reply-content-not-a-list", messages(), BODY({ content: "Hi" })),
	completes(
		"messages-reply-content-items-not-objects",
		messages(),
		BODY({ content: [1, null, "x", [{ type: "text", text: "no" }], { type: "text", text: "yes" }] }),
	),
	completes(
		"messages-reply-unknown-block-types",
		messages(),
		BODY({ content: [{ type: "server_tool_use" }, {}, { type: "text" }] }),
	),
	completes("messages-reply-no-usage", messages(), BODY({ usage: undefined })),
	completes("messages-reply-usage-mistyped", messages(), BODY({ usage: { input_tokens: "4", output_tokens: 2 } })),
	completes(
		"messages-reply-usage-with-billing",
		messages(),
		BODY({ usage: { input_tokens: 4, output_tokens: 2, ...BILLING } }),
	),
	completes("messages-reply-no-id", messages(), BODY({ id: undefined })),
	completes("messages-reply-id-mistyped", messages(), BODY({ id: 5 })),
	completes("messages-reply-error-object", messages(), {
		type: "error",
		error: { type: "overloaded_error", message: "Overloaded" },
	}),
	completes("messages-reply-error-allowance", messages(), {
		type: "error",
		error: { type: "GoUsageLimitError", message: "Usage limit reached." },
	}),
	completes("messages-reply-error-text-is-the-message", messages(), { type: "error", error: "It broke." }),
	completes("messages-reply-error-empty-text", messages(), { type: "error", error: "" }),
	completes("messages-reply-error-array", messages(), { type: "error", error: ["x"] }),
	completes("messages-reply-error-null", messages(), { type: "error", error: null }),
	completes("messages-reply-error-type-without-an-error", messages(), { type: "error" }),
	completes("messages-reply-error-field-without-the-type-is-no-error", messages(), {
		error: { message: "x" },
		content: [],
		stop_reason: "end_turn",
	}),
	completes("messages-reply-is-an-array", messages(), [1]),
	completes("messages-reply-is-text", messages(), "ok"),
	completes("messages-reply-is-null", messages(), null),
	completes("messages-reply-empty-object", messages(), {}),
];

////////////////////////////////
//  HTTP failures

const ERR = (type: string, message: string): Json => ({ type: "error", error: { type, message } });

const failures: CaseSource[] = [
	fails("messages-http-400-bad-request", messages(), 400, ERR("invalid_request_error", "Bad field.")),
	fails(
		"messages-http-400-model-unavailable",
		messages(),
		400,
		ERR("invalid_request_error", "Model is unavailable."),
	),
	fails(
		"messages-http-400-model-unavailable-ignores-case",
		messages(),
		400,
		ERR("x", "The MODEL IS UNAVAILABLE right now"),
	),
	fails("messages-http-400-no-body", messages(), 400, null),
	fails("messages-http-401", messages(), 401, ERR("authentication_error", "invalid x-api-key")),
	fails("messages-http-402", messages(), 402, ERR("x", "Insufficient balance.")),
	fails("messages-http-403", messages(), 403, ERR("permission_error", "Forbidden.")),
	fails("messages-http-404", messages(), 404, ERR("not_found_error", "No such model.")),
	fails("messages-http-408", messages(), 408, ERR("x", "Slow.")),
	fails("messages-http-413", messages(), 413, ERR("request_too_large", "Too large.")),
	fails("messages-http-429-rate-limited", messages(), 429, ERR("rate_limit_error", "Slow down.")),
	fails(
		"messages-http-429-retry-after-header-wins",
		messages({ nowMs: NOW }),
		429,
		{ error: { message: "Slow.", resetsAt: NOW / 1000 + 90 } },
		[["retry-after", "5"]],
	),
	fails("messages-http-429-reset-in-the-future", messages({ nowMs: NOW }), 429, {
		error: { message: "Slow.", resetsAt: NOW / 1000 + 90 },
	}),
	fails("messages-http-429-reset-in-the-past", messages({ nowMs: NOW }), 429, {
		error: { message: "Slow.", resetsAt: NOW / 1000 - 90 },
	}),
	fails("messages-http-429-overloaded-marker", messages(), 429, ERR("x", "[overloaded] busy")),
	fails("messages-http-429-overloaded-type", messages(), 429, ERR("overloaded_error", "Busy.")),
	fails("messages-http-529-overloaded", messages(), 529, ERR("overloaded_error", "Overloaded")),
	fails("messages-http-429-allowance", messages({ nowMs: NOW }), 429, {
		type: "error",
		error: { type: "GoUsageLimitError", message: "Usage limit reached.", resetsAt: NOW / 1000 + 3600 },
	}),
	fails("messages-http-429-allowance-reset-in-the-past", messages({ nowMs: NOW }), 429, {
		type: "error",
		error: { type: "GoUsageLimitError", message: "Usage limit reached.", resetsAt: NOW / 1000 - 3600 },
	}),
	fails("messages-http-429-budget-limit", messages({ nowMs: NOW }), 429, {
		type: "error",
		error: { type: "BudgetLimitError", message: "Budget spent.", resetsAt: "2027-01-15T09:00:00Z" },
	}),
	fails("messages-http-500", messages(), 500, ERR("api_error", "Down.")),
	fails("messages-http-error-text-is-the-message", messages(), 500, { type: "error", error: "Down." }),
	fails("messages-http-message-blank", messages(), 500, { error: { message: "   " } }),
	fails("messages-http-body-array", messages(), 500, [1]),
	fails("messages-http-body-text", messages(), 502, "Bad gateway"),
	fails("messages-http-body-empty-object", messages(), 503, {}),
];

////////////////////////////////
//  Cases

export const messagesCases: CaseSource[] = [...requests, ...history, ...streamsOf, ...replies, ...failures];
