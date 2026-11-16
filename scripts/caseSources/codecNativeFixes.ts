// Codec cases: the corrections to the native families.

import type { CaseSource } from "../caseAdapter";
import { data, type Json } from "./codecFixtures";
import { completes, encodes, streams } from "./codecKit";
import { chunk, event, gemini, HAIKU, messages, responses } from "./codecNativeFixtures";

////////////////////////////////
//  Fixtures

/** A data item whose JSON text is written by hand, for numbers JSON.stringify would change. */
const raw = (text: string, name?: string): Json => ({
	kind: "data",
	...(name === undefined ? {} : { event: name }),
	data: text,
});

const STOP = event("message_stop");
const start = (usage: unknown, id = "msg_1"): Json => event("message_start", { message: { id, usage } });
const stop = (usage?: unknown, reason = "end_turn"): Json =>
	event("message_delta", { delta: { stop_reason: reason }, usage });

const completed = (extra: Json = {}): Json =>
	event("response.completed", { response: { id: "resp_1", status: "completed", ...extra } });
const added = (item: Json, over: Json = {}): Json => event("response.output_item.added", { item, ...over });
const done = (item: Json, over: Json = {}): Json => event("response.output_item.done", { item, ...over });
const call = (over: Json = {}): Json => ({ type: "function_call", name: "read", ...over });
const message = (text: string, over: Json = {}): Json => ({
	type: "message",
	role: "assistant",
	content: [{ type: "output_text", text }],
	...over,
});

const BODY = (over: Json = {}): Json => ({ id: "resp_1", status: "completed", output: [], ...over });

const finished = (parts: unknown[] = [{ text: "Hi" }], top: Json = {}): Json =>
	chunk(parts, { finishReason: "STOP" }, top);

const step = (parts: Json[], model: string | null = HAIKU): Json => ({
	role: "step",
	...(model === null ? {} : { producer: { model } }),
	parts,
});

const read = (wireId: string, index = 0): Json[] => [
	{ kind: "call", index, wireId, name: "read", arguments: "{}" },
	{ kind: "result", index, output: `out ${wireId}`, isError: false },
];

////////////////////////////////
//  Messages usage keeps what it knew

const KEPT = start({ input_tokens: 5, cache_read_input_tokens: 1000, output_tokens: 1 });

const messagesUsage: CaseSource[] = [
	streams(
		"messages-stream-usage-null-count-after-a-valid-one-changes-nothing",
		messages(),
		KEPT,
		stop({ output_tokens: 20, cache_read_input_tokens: null }),
		STOP,
	),
	streams(
		"messages-stream-usage-null-input-after-a-valid-one-changes-nothing",
		messages(),
		KEPT,
		stop({ input_tokens: null, output_tokens: 20 }),
		STOP,
	),
	streams(
		"messages-stream-usage-negative-after-a-valid-one-changes-nothing",
		messages(),
		KEPT,
		stop({ output_tokens: -3, cache_read_input_tokens: -1 }),
		STOP,
	),
	streams(
		"messages-stream-usage-text-after-a-valid-one-changes-nothing",
		messages(),
		KEPT,
		stop({ output_tokens: "20", cache_read_input_tokens: [1] }),
		STOP,
	),
	streams(
		"messages-stream-usage-billing-block-replaces-the-earlier-one",
		messages(),
		start({ input_tokens: 5, output_tokens: 1, opencode: { billing_source: "go" } }),
		stop({ output_tokens: 2, opencode: { billing_source: "byok" } }),
		STOP,
	),
	streams(
		"messages-stream-usage-negative-cost-after-a-valid-one-changes-nothing",
		messages(),
		start({ input_tokens: 5, output_tokens: 1, cost: 0.5 }),
		stop({ output_tokens: 2, cost: -1 }),
		STOP,
	),
	streams(
		"messages-stream-usage-later-valid-values-replace",
		messages(),
		KEPT,
		stop({ output_tokens: 20, cache_read_input_tokens: 7 }),
		STOP,
	),
];

////////////////////////////////
//  Responses items without an id

const FUNCTION = (callId: string, over: Json = {}): Json =>
	call({ call_id: callId, arguments: `{"x":"${callId}"}`, ...over });

const responsesPlaces: CaseSource[] = [
	completes(
		"responses-reply-calls-without-ids-stay-apart",
		responses(),
		BODY({ output: [FUNCTION("a"), FUNCTION("b")] }),
	),
	completes(
		"responses-reply-calls-with-neither-id-stay-apart",
		responses(),
		BODY({ output: [call({ arguments: "{}" }), call({ arguments: '{"y":1}' })] }),
	),
	completes(
		"responses-reply-call-keyed-by-its-id-before-its-call-id",
		responses(),
		BODY({ output: [FUNCTION("a", { id: "fc_1" }), FUNCTION("b", { id: "fc_2" })] }),
	),
	completes(
		"responses-reply-messages-without-ids-keep-every-text",
		responses(),
		BODY({ output: [message("one"), message("two", { phase: "final_answer" })] }),
	),
	completes(
		"responses-reply-messages-with-empty-ids-keep-every-text",
		responses(),
		BODY({ output: [message("one", { id: "" }), message("two", { id: "" })] }),
	),
	completes(
		"responses-reply-reasoning-without-ids-stays-apart",
		responses(),
		BODY({
			output: [
				{ type: "reasoning", summary: [{ type: "summary_text", text: "One." }], encrypted_content: "e1" },
				{ type: "reasoning", summary: [{ type: "summary_text", text: "Two." }], encrypted_content: "e2" },
			],
		}),
	),
	completes(
		"responses-reply-same-id-twice-is-one-message",
		responses(),
		BODY({ output: [message("one", { id: "m" }), message("two", { id: "m" })] }),
	),
	streams(
		"responses-stream-calls-without-ids-by-output-index",
		responses(),
		added(call({ call_id: "a", arguments: "" }), { output_index: 0 }),
		added(call({ call_id: "b", arguments: "" }), { output_index: 1 }),
		event("response.function_call_arguments.delta", { output_index: 1, delta: "{}" }),
		event("response.function_call_arguments.delta", { output_index: 0, delta: '{"a":1}' }),
		event("response.function_call_arguments.done", { output_index: 0, arguments: '{"a":1}' }),
		done(FUNCTION("a"), { output_index: 0 }),
		completed(),
	),
	streams(
		"responses-stream-calls-by-call-id-and-place",
		responses(),
		added(call({ call_id: "a", arguments: "" }), { output_index: 0 }),
		event("response.function_call_arguments.delta", { output_index: 0, delta: "{}" }),
		done(call({ call_id: "a", arguments: "{}" })),
		completed(),
	),
	streams(
		"responses-stream-messages-without-ids-by-output-index",
		responses(),
		added(message("", { phase: "commentary" }), { output_index: 0 }),
		added(message("", { phase: "final_answer" }), { output_index: 1 }),
		event("response.output_text.delta", { output_index: 1, delta: "B" }),
		event("response.output_text.delta", { output_index: 0, delta: "A" }),
		done(message("A"), { output_index: 0 }),
		done(message("B2"), { output_index: 1 }),
		completed(),
	),
	streams(
		"responses-stream-messages-without-any-key-each-show-their-text",
		responses(),
		done(message("one")),
		done(message("two")),
		completed(),
	),
	streams(
		"responses-stream-reasoning-without-ids-by-output-index",
		responses(),
		event("response.reasoning_summary_text.delta", { output_index: 0, summary_index: 0, delta: "A" }),
		event("response.reasoning_summary_text.delta", { output_index: 1, summary_index: 0, delta: "B" }),
		event("response.reasoning_summary_text.delta", { output_index: 0, summary_index: 1, delta: "C" }),
		done({ type: "reasoning", encrypted_content: "e0" }, { output_index: 0 }),
		completed(),
	),
	streams(
		"responses-stream-an-id-wins-over-the-place",
		responses(),
		added(call({ id: "fc_1", call_id: "a", arguments: "" }), { output_index: 5 }),
		event("response.function_call_arguments.delta", { item_id: "fc_1", output_index: 9, delta: "{}" }),
		completed(),
	),
	streams(
		"responses-stream-empty-id-falls-back-to-the-place",
		responses(),
		added(call({ id: "", call_id: "a", arguments: "" }), { output_index: 2 }),
		event("response.function_call_arguments.delta", { item_id: "", output_index: 2, delta: "{}" }),
		completed(),
	),
	streams(
		"responses-stream-output-index-mistyped-is-no-place",
		responses(),
		added(call({ call_id: "a", arguments: "" }), { output_index: "0" }),
		event("response.function_call_arguments.delta", { output_index: "0", delta: "{}" }),
		completed(),
	),
];

////////////////////////////////
//  Responses arguments that contradict

const responsesArguments: CaseSource[] = [
	streams(
		"responses-stream-final-arguments-in-an-item-contradict-the-deltas",
		responses(),
		added(call({ id: "fc_1", call_id: "a", arguments: "" })),
		event("response.function_call_arguments.delta", { item_id: "fc_1", delta: '{"a":' }),
		done(call({ id: "fc_1", call_id: "a", arguments: '{"b":2}' })),
		completed(),
	),
	streams(
		"responses-stream-final-arguments-longer-than-the-deltas-extend-them",
		responses(),
		added(call({ id: "fc_1", call_id: "a", arguments: "" })),
		event("response.function_call_arguments.delta", { item_id: "fc_1", delta: '{"a":' }),
		event("response.function_call_arguments.done", { item_id: "fc_1", arguments: '{"a":1}' }),
		completed(),
	),
	streams(
		"responses-stream-final-arguments-shorter-than-the-deltas-contradict-them",
		responses(),
		added(call({ id: "fc_1", call_id: "a", arguments: "" })),
		event("response.function_call_arguments.delta", { item_id: "fc_1", delta: '{"a":1}' }),
		event("response.function_call_arguments.done", { item_id: "fc_1", arguments: '{"a"' }),
		completed(),
	),
	streams(
		"responses-stream-final-arguments-empty-are-ignored",
		responses(),
		added(call({ id: "fc_1", call_id: "a", arguments: "" })),
		event("response.function_call_arguments.delta", { item_id: "fc_1", delta: '{"a":1}' }),
		event("response.function_call_arguments.done", { item_id: "fc_1", arguments: "" }),
		done(call({ id: "fc_1", call_id: "a", arguments: "" })),
		completed(),
	),
	streams(
		"responses-stream-final-arguments-mistyped-are-ignored",
		responses(),
		added(call({ id: "fc_1", call_id: "a", arguments: "" })),
		event("response.function_call_arguments.delta", { item_id: "fc_1", delta: '{"a":1}' }),
		event("response.function_call_arguments.done", { item_id: "fc_1", arguments: { a: 1 } }),
		completed(),
	),
	streams(
		"responses-stream-text-done-after-deltas-keeps-the-deltas",
		responses(),
		event("response.output_text.delta", { item_id: "msg_1", delta: "Hel" }),
		done(message("Hello world", { id: "msg_1" })),
		completed(),
	),
	completes(
		"responses-reply-arguments-need-no-deltas",
		responses(),
		BODY({ output: [call({ id: "fc_1", call_id: "a", arguments: '{"b":2}' })] }),
	),
];

////////////////////////////////
//  Messages tool ids stay distinct

const ids = (...wireIds: string[]): Json => ({
	history: [{ role: "user", text: "Go." }, step(wireIds.flatMap((wireId, index) => read(wireId, index)))],
});

const messagesIds: CaseSource[] = [
	encodes("messages-tool-ids-differing-in-punctuation-stay-distinct", messages(), {
		model: HAIKU,
		...ids("call.1", "call:1"),
	}),
	encodes("messages-tool-ids-three-way-clash-counts-up", messages(), {
		model: HAIKU,
		...ids("call.1", "call:1", "call 1"),
	}),
	encodes("messages-tool-id-that-needs-no-change-keeps-its-id", messages(), {
		model: HAIKU,
		...ids("call.1", "call_1"),
	}),
	encodes("messages-tool-id-suffix-avoids-an-id-in-use", messages(), {
		model: HAIKU,
		...ids("call.1", "call:1", "call_1_2"),
	}),
	encodes("messages-tool-ids-empty-and-call-stay-distinct", messages(), { model: HAIKU, ...ids("", "call") }),
	encodes("messages-tool-ids-emoji-and-symbols-stay-distinct", messages(), {
		model: HAIKU,
		...ids("\u{1f600}", "__", ".."),
	}),
	encodes("messages-tool-id-repeated-across-steps-maps-the-same", messages(), {
		model: HAIKU,
		history: [
			{ role: "user", text: "Go." },
			step(read("run/1")),
			{ role: "user", text: "Again." },
			step(read("run/1")),
			step(read("run:1")),
		],
	}),
	encodes("messages-tool-id-results-name-the-block-id", messages(), { model: HAIKU, ...ids("a.b", "a:b") }),
];

////////////////////////////////
//  An error is an error whatever else the body lacks

const errors: CaseSource[] = [
	completes("messages-reply-error-without-the-type-fails-as-an-error", messages(), {
		error: { type: "overloaded_error", message: "busy" },
	}),
	completes("messages-reply-error-text-without-the-type", messages(), { error: "busy" }),
	completes("messages-reply-error-true-without-the-type", messages(), {
		error: true,
		content: [],
		stop_reason: "end_turn",
	}),
	completes("messages-reply-error-null-without-the-type-is-no-error", messages(), {
		error: null,
		content: [{ type: "text", text: "Hi" }],
		stop_reason: "end_turn",
	}),
	completes("messages-reply-error-zero-without-the-type-is-no-error", messages(), {
		error: 0,
		content: [{ type: "text", text: "Hi" }],
		stop_reason: "end_turn",
	}),
	completes("messages-reply-error-beside-content-fails", messages(), {
		error: { message: "late" },
		content: [{ type: "text", text: "Hi" }],
		stop_reason: "end_turn",
	}),
	streams(
		"messages-stream-error-chunk-without-a-type",
		messages(),
		start({ input_tokens: 4, output_tokens: 1 }),
		raw('{"error":{"message":"busy"}}'),
	),
	streams(
		"messages-stream-error-chunk-overloaded-without-a-type",
		messages(),
		raw('{"error":{"type":"overloaded_error","message":"busy"}}'),
	),
	streams("messages-stream-error-chunk-text", messages(), raw('{"error":"busy"}')),
	streams("messages-stream-error-chunk-array", messages(), raw('{"error":[1]}')),
	streams(
		"messages-stream-error-chunk-null-is-ignored",
		messages(),
		raw('{"error":null}'),
		start({ input_tokens: 4, output_tokens: 1 }),
		stop({ output_tokens: 2 }),
		STOP,
	),
	streams(
		"messages-stream-error-chunk-beside-another-event-fails",
		messages(),
		event("content_block_start", {
			index: 0,
			content_block: { type: "text", text: "Hi" },
			error: { message: "bad" },
		}),
	),
	completes("responses-reply-error-without-a-status", responses(), { error: { message: "quota exceeded" } }),
	completes("responses-reply-error-with-a-code-without-a-status", responses(), {
		error: { code: "insufficient_quota", message: "quota exceeded" },
	}),
	completes("responses-reply-error-text-without-a-status", responses(), { error: "quota exceeded" }),
	completes(
		"responses-reply-error-on-a-completed-reply",
		responses(),
		BODY({ error: { code: "server_error", message: "late" }, output: [message("Hi")] }),
	),
	completes(
		"responses-reply-error-on-a-completed-reply-bills-usage",
		responses(),
		BODY({ error: { message: "late" }, usage: { input_tokens: 4, output_tokens: 2 } }),
	),
	completes(
		"responses-reply-error-null-on-a-completed-reply-is-no-error",
		responses(),
		BODY({ error: null, output: [message("Hi")] }),
	),
	completes(
		"responses-reply-error-empty-text-on-a-completed-reply-is-no-error",
		responses(),
		BODY({ error: "", output: [message("Hi")] }),
	),
	completes(
		"responses-reply-error-false-on-a-completed-reply-is-no-error",
		responses(),
		BODY({ error: false, output: [message("Hi")] }),
	),
	completes(
		"responses-reply-error-object-empty-on-a-completed-reply",
		responses(),
		BODY({ error: {}, output: [message("Hi")] }),
	),
	streams(
		"responses-stream-error-chunk-without-a-type",
		responses(),
		event("response.created", { response: { id: "resp_1" } }),
		raw('{"error":{"code":"server_error","message":"busy"}}'),
	),
	streams("responses-stream-error-chunk-text", responses(), raw('{"error":"busy"}')),
	streams("responses-stream-error-chunk-true", responses(), raw('{"error":true}')),
	streams("responses-stream-error-chunk-null-is-ignored", responses(), raw('{"error":null}'), completed()),
	streams(
		"responses-stream-completed-event-with-an-error-fails",
		responses(),
		event("response.completed", { response: { id: "resp_1", status: "completed", error: { message: "late" } } }),
	),
	streams(
		"responses-stream-completed-event-with-error-null-is-fine",
		responses(),
		event("response.completed", { response: { id: "resp_1", status: "completed", error: null } }),
	),
];

////////////////////////////////
//  One usage rule

const family = [
	["responses", responses(), (usage: Json): Json => BODY({ usage })],
	["messages", messages(), (usage: Json): Json => ({ id: "msg_1", content: [], stop_reason: "end_turn", usage })],
	[
		"gemini",
		gemini(),
		(usage: Json): Json => ({ candidates: [{ finishReason: "STOP" }], responseId: "r", usageMetadata: usage }),
	],
] as const;

const COUNTS: Record<string, [input: string, output: string]> = {
	responses: ["input_tokens", "output_tokens"],
	messages: ["input_tokens", "output_tokens"],
	gemini: ["promptTokenCount", "candidatesTokenCount"],
};

const usageRule: CaseSource[] = family.flatMap(([name, codec, reply]) => {
	const [input, output] = COUNTS[name] ?? ["", ""];
	const one = (label: string, usage: Json) => completes(`${name}-reply-usage-${label}`, codec, reply(usage));
	return [
		one("empty-object-is-zero", {}),
		one("only-the-input-count", { [input]: 10 }),
		one("only-the-output-count", { [output]: 5 }),
		one("input-count-mistyped-is-zero", { [input]: "10", [output]: 5 }),
		one("output-count-mistyped-is-zero", { [input]: 10, [output]: [5] }),
		one("input-count-negative-is-zero", { [input]: -10, [output]: 5 }),
		one("output-count-negative-is-zero", { [input]: 10, [output]: -5 }),
		one("input-count-null-is-zero", { [input]: null, [output]: 5 }),
		one("counts-past-2-53", { [input]: 2 ** 60, [output]: 2 ** 61 }),
		one("counts-negative-zero", { [input]: -0, [output]: -0 }),
		one("billing-with-no-counts", { cost: 0.5, opencode: { billing_source: "go" } }),
	];
});

const usageDetails: CaseSource[] = [
	completes(
		"responses-reply-usage-details-negative-are-missing",
		responses(),
		BODY({
			usage: {
				input_tokens: 10,
				output_tokens: 5,
				input_tokens_details: { cached_tokens: -1, cache_write_tokens: -2 },
				output_tokens_details: { reasoning_tokens: -3 },
			},
		}),
	),
	completes(
		"responses-reply-usage-details-valid",
		responses(),
		BODY({
			usage: {
				input_tokens: 10,
				output_tokens: 5,
				input_tokens_details: { cached_tokens: 4, cache_write_tokens: 2 },
				output_tokens_details: { reasoning_tokens: 3 },
			},
		}),
	),
	completes("messages-reply-usage-cache-negative-is-missing", messages(), {
		id: "m",
		content: [],
		stop_reason: "end_turn",
		usage: { input_tokens: 10, cache_read_input_tokens: -50, cache_creation_input_tokens: -5, output_tokens: 5 },
	}),
	completes("messages-reply-usage-cache-counts-sum-into-the-input", messages(), {
		id: "m",
		content: [],
		stop_reason: "end_turn",
		usage: { input_tokens: 10, cache_read_input_tokens: 50, cache_creation_input_tokens: 5, output_tokens: 5 },
	}),
	completes("gemini-reply-usage-thoughts-negative-is-missing", gemini(), {
		candidates: [{ finishReason: "STOP" }],
		usageMetadata: {
			promptTokenCount: 10,
			candidatesTokenCount: 5,
			thoughtsTokenCount: -4,
			cachedContentTokenCount: -1,
		},
	}),
	completes("gemini-reply-usage-tool-prompt-adds-to-the-input", gemini(), {
		candidates: [{ finishReason: "STOP" }],
		usageMetadata: {
			promptTokenCount: 10,
			toolUsePromptTokenCount: 3,
			candidatesTokenCount: 5,
			thoughtsTokenCount: 2,
		},
	}),
	completes("gemini-reply-usage-tool-prompt-negative-adds-nothing", gemini(), {
		candidates: [{ finishReason: "STOP" }],
		usageMetadata: { promptTokenCount: 10, toolUsePromptTokenCount: -3, candidatesTokenCount: 5 },
	}),
	streams(
		"gemini-stream-usage-empty-object-replaces-the-last-usage",
		gemini(),
		chunk([{ text: "Hi" }], {}, { usageMetadata: { promptTokenCount: 10, candidatesTokenCount: 4 } }),
		finished([{ text: "!" }], { usageMetadata: {} }),
	),
	streams(
		"gemini-stream-usage-object-after-the-finish-always-adds-one",
		gemini(),
		finished(),
		chunk([], {}, { usageMetadata: {} }),
	),
];

////////////////////////////////
//  Extreme numbers

const extremes: CaseSource[] = [
	// A usage field.
	streams(
		"responses-stream-extreme-usage-1e999",
		responses(),
		raw(
			'{"type":"response.completed","response":{"status":"completed","usage":{"input_tokens":1e999,"output_tokens":5,"input_tokens_details":{"cached_tokens":1e999}}}}',
		),
	),
	streams(
		"responses-stream-extreme-usage-negative-zero",
		responses(),
		raw(
			'{"type":"response.completed","response":{"status":"completed","usage":{"input_tokens":-0,"output_tokens":-0,"input_tokens_details":{"cached_tokens":-0}}}}',
		),
	),
	streams(
		"responses-stream-extreme-usage-past-2-53",
		responses(),
		raw(
			'{"type":"response.completed","response":{"status":"completed","usage":{"input_tokens":9007199254740993,"output_tokens":18014398509481985}}}',
		),
	),
	streams(
		"messages-stream-extreme-usage-1e999",
		messages(),
		raw(
			'{"type":"message_start","message":{"usage":{"input_tokens":1e999,"output_tokens":3,"cache_read_input_tokens":1e999}}}',
		),
		stop(),
		STOP,
	),
	streams(
		"messages-stream-extreme-usage-negative-zero",
		messages(),
		raw(
			'{"type":"message_start","message":{"usage":{"input_tokens":-0,"output_tokens":-0,"cache_read_input_tokens":-0}}}',
		),
		stop(),
		STOP,
	),
	streams(
		"messages-stream-extreme-usage-past-2-53",
		messages(),
		raw(
			'{"type":"message_start","message":{"usage":{"input_tokens":9007199254740993,"output_tokens":18014398509481985}}}',
		),
		stop(),
		STOP,
	),
	streams(
		"messages-stream-extreme-usage-later-1e999-keeps-the-earlier-count",
		messages(),
		start({ input_tokens: 5, output_tokens: 1 }),
		raw('{"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":1e999}}'),
		STOP,
	),
	streams(
		"gemini-stream-extreme-usage-1e999",
		gemini(),
		raw(
			'{"candidates":[{"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":1e999,"candidatesTokenCount":4,"thoughtsTokenCount":1e999}}',
		),
	),
	streams(
		"gemini-stream-extreme-usage-negative-zero",
		gemini(),
		raw(
			'{"candidates":[{"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":-0,"candidatesTokenCount":-0,"thoughtsTokenCount":-0}}',
		),
	),
	streams(
		"gemini-stream-extreme-usage-past-2-53",
		gemini(),
		raw(
			'{"candidates":[{"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":9007199254740993,"candidatesTokenCount":18014398509481985}}',
		),
	),
	// An index.
	streams(
		"messages-stream-extreme-index-1e999",
		messages(),
		start({ input_tokens: 1, output_tokens: 1 }),
		raw('{"type":"content_block_delta","index":1e999,"delta":{"type":"thinking_delta","thinking":"A"}}'),
		stop(),
		STOP,
	),
	streams(
		"messages-stream-extreme-index-negative-zero",
		messages(),
		start({ input_tokens: 1, output_tokens: 1 }),
		raw('{"type":"content_block_delta","index":-0,"delta":{"type":"thinking_delta","thinking":"A"}}'),
		stop(),
		STOP,
	),
	streams(
		"messages-stream-extreme-index-past-2-53",
		messages(),
		start({ input_tokens: 1, output_tokens: 1 }),
		raw('{"type":"content_block_delta","index":9007199254740993,"delta":{"type":"thinking_delta","thinking":"A"}}'),
		stop(),
		STOP,
	),
	streams(
		"messages-stream-extreme-index-1e21",
		messages(),
		start({ input_tokens: 1, output_tokens: 1 }),
		raw('{"type":"content_block_delta","index":1e21,"delta":{"type":"thinking_delta","thinking":"A"}}'),
		stop(),
		STOP,
	),
	streams(
		"responses-stream-extreme-index-1e999",
		responses(),
		raw('{"type":"response.reasoning_summary_text.delta","item_id":"rs_1","summary_index":1e999,"delta":"A"}'),
		raw('{"type":"response.reasoning_summary_text.delta","item_id":"rs_1","summary_index":0,"delta":"B"}'),
		completed(),
	),
	streams(
		"responses-stream-extreme-index-negative-zero",
		responses(),
		raw('{"type":"response.reasoning_summary_text.delta","item_id":"rs_1","summary_index":-0,"delta":"A"}'),
		raw('{"type":"response.reasoning_summary_text.delta","item_id":"rs_1","summary_index":0,"delta":"B"}'),
		completed(),
	),
	streams(
		"responses-stream-extreme-index-past-2-53",
		responses(),
		raw(
			'{"type":"response.reasoning_summary_text.delta","item_id":"rs_1","summary_index":9007199254740993,"delta":"A"}',
		),
		raw(
			'{"type":"response.reasoning_summary_text.delta","item_id":"rs_1","summary_index":9007199254740992,"delta":"B"}',
		),
		completed(),
	),
	streams(
		"responses-stream-extreme-output-index-1e999-is-no-place",
		responses(),
		raw('{"type":"response.function_call_arguments.delta","output_index":1e999,"delta":"{}"}'),
		completed(),
	),
	streams(
		"responses-stream-extreme-output-index-negative-zero",
		responses(),
		raw(
			'{"type":"response.output_item.added","output_index":-0,"item":{"type":"function_call","call_id":"a","name":"read","arguments":""}}',
		),
		raw('{"type":"response.function_call_arguments.delta","output_index":0,"delta":"{}"}'),
		completed(),
	),
	streams(
		"responses-stream-extreme-output-index-past-2-53",
		responses(),
		raw(
			'{"type":"response.output_item.added","output_index":9007199254740993,"item":{"type":"function_call","call_id":"a","name":"read","arguments":""}}',
		),
		raw('{"type":"response.function_call_arguments.delta","output_index":9007199254740992,"delta":"{}"}'),
		completed(),
	),
	streams(
		"responses-stream-extreme-output-index-1e21",
		responses(),
		raw(
			'{"type":"response.output_item.added","output_index":1e21,"item":{"type":"function_call","call_id":"a","name":"read","arguments":""}}',
		),
		raw('{"type":"response.function_call_arguments.delta","output_index":1e21,"delta":"{}"}'),
		completed(),
	),
];

////////////////////////////////
//  Smaller ones

const smaller: CaseSource[] = [
	streams(
		"messages-stream-block-index-met-twice-keeps-call-indexes-apart",
		messages(),
		start({ input_tokens: 1, output_tokens: 1 }),
		event("content_block_start", {
			index: 0,
			content_block: { type: "tool_use", id: "a", name: "read", input: {} },
		}),
		event("content_block_start", {
			index: 0,
			content_block: { type: "tool_use", id: "b", name: "read", input: {} },
		}),
		event("content_block_start", {
			index: 1,
			content_block: { type: "tool_use", id: "c", name: "read", input: {} },
		}),
		event("content_block_delta", { index: 0, delta: { type: "input_json_delta", partial_json: "{}" } }),
		event("content_block_delta", { index: 1, delta: { type: "input_json_delta", partial_json: "{}" } }),
		stop(undefined, "tool_use"),
		STOP,
	),
	completes("messages-reply-two-calls-share-no-index", messages(), {
		id: "m",
		content: [
			{ type: "tool_use", id: "a", name: "read", input: {} },
			{ type: "tool_use", id: "b", name: "read", input: {} },
		],
		stop_reason: "tool_use",
	}),
	encodes("messages-cache-explicit-last-system-text-empty-marks-the-last-one-sent", messages(), {
		model: HAIKU,
		system: ["Sections.", "Rules.", ""],
		cache: { kind: "explicit" },
	}),
	encodes("messages-cache-explicit-last-system-text-empty-with-a-summary", messages(), {
		model: HAIKU,
		system: ["Rules.", ""],
		summary: "Earlier.",
		cache: { kind: "explicit" },
	}),
	encodes("messages-cache-explicit-every-system-text-empty-marks-the-summary", messages(), {
		model: HAIKU,
		system: ["", ""],
		summary: "Earlier.",
		cache: { kind: "explicit" },
	}),
	encodes("messages-cache-explicit-every-system-text-empty-no-summary", messages(), {
		model: HAIKU,
		system: ["", ""],
		cache: { kind: "explicit" },
	}),
	encodes("messages-cache-explicit-empty-between-marks-the-last-nonempty", messages(), {
		model: HAIKU,
		system: ["One.", "", "Two.", ""],
		cache: { kind: "explicit" },
	}),
	encodes("messages-cache-explicit-prompt-schema-is-never-marked", messages({ api: { structured: false } }), {
		model: HAIKU,
		system: ["Rules.", ""],
		schema: { name: "t", json: { type: "object" } },
		cache: { kind: "explicit" },
	}),
	streams(
		"gemini-stream-blocked-prompt-bills-the-usage-of-the-chunk",
		gemini(),
		data({
			promptFeedback: { blockReason: "SAFETY" },
			usageMetadata: { promptTokenCount: 10, candidatesTokenCount: 0 },
		}),
	),
	streams(
		"gemini-stream-blocked-prompt-bills-the-earlier-usage-and-serving",
		gemini(),
		chunk(
			[{ text: "Hi" }],
			{},
			{ usageMetadata: { promptTokenCount: 7, candidatesTokenCount: 1 }, responseId: "r_1" },
		),
		data({ promptFeedback: { blockReason: "OTHER" } }),
	),
	streams(
		"gemini-stream-blocked-prompt-bills-the-serving",
		gemini(),
		data({ promptFeedback: { blockReason: "SAFETY" }, responseId: "r_9" }),
	),
	streams(
		"gemini-stream-blocked-prompt-with-nothing-billed",
		gemini(),
		data({ promptFeedback: { blockReason: "SAFETY" } }),
	),
	completes("gemini-reply-blocked-prompt-bills-the-usage", gemini(), {
		promptFeedback: { blockReason: "SAFETY" },
		usageMetadata: { promptTokenCount: 10 },
		responseId: "r",
	}),
	// Left as they are.
	streams(
		"messages-stream-pause-turn-reads-as-stop",
		messages(),
		start({ input_tokens: 1, output_tokens: 1 }),
		stop(undefined, "pause_turn"),
		STOP,
	),
	streams(
		"messages-stream-input-json-delta-for-a-block-that-never-started",
		messages(),
		start({ input_tokens: 1, output_tokens: 1 }),
		event("content_block_delta", { index: 4, delta: { type: "input_json_delta", partial_json: "{}" } }),
		stop(),
		STOP,
	),
	completes(
		"responses-reply-refusal-part-is-text",
		responses(),
		BODY({ output: [{ type: "message", id: "m", content: [{ type: "refusal", refusal: "No." }] }] }),
	),
	encodes("gemini-path-model-with-a-slash", gemini(), { model: "google/gemini-3-pro" }, "complete"),
	encodes("responses-replay-family-of-a-prefixed-model", responses(), {
		model: "openai/gpt-6-luna",
		history: [
			step(
				[
					{ kind: "thought", type: "reasoning", opaque: "e", id: "rs_1" },
					{ kind: "text", text: "Hi" },
				],
				"gpt-6-mini",
			),
		],
	}),
	encodes("messages-replay-summary-segments-go-out-as-thinking", messages(), {
		model: HAIKU,
		history: [
			step([
				{ kind: "thought", type: "thinking", opaque: "sig", segments: [{ kind: "summary", text: "sum" }] },
				{ kind: "text", text: "Hi" },
			]),
		],
	}),
];

////////////////////////////////
//  Cases

export const nativeFixCases: CaseSource[] = [
	...messagesUsage,
	...responsesPlaces,
	...responsesArguments,
	...messagesIds,
	...errors,
	...usageRule,
	...usageDetails,
	...extremes,
	...smaller,
];
