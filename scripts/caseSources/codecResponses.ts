// Codec cases: the Responses codec.

import type { CaseSource } from "../caseAdapter";
import { comment, DONE, data, type Json } from "./codecFixtures";
import { completes, encodes, fails, streams } from "./codecKit";
import { event, eventNamed, GPT, NOW, responses } from "./codecNativeFixtures";

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
	encodes(name, codec, { model: GPT, ...over }, mode);

const completed = (status = "completed", details?: unknown, extra: Json = {}): Json =>
	event(`response.${status}`, { response: { id: "resp_1", status, incomplete_details: details, ...extra } });

const text = (delta: unknown, itemId: unknown = "msg_1"): Json =>
	event("response.output_text.delta", { item_id: itemId, delta });

const added = (item: Json): Json => event("response.output_item.added", { item });
const done = (item: Json): Json => event("response.output_item.done", { item });

const MESSAGE = (content: unknown, over: Json = {}): Json => ({
	type: "message",
	id: "msg_1",
	role: "assistant",
	content,
	...over,
});

const CALL_ITEM: Json = {
	type: "function_call",
	id: "fc_1",
	call_id: "call_a",
	name: "read",
	arguments: '{"path":"a"}',
};

/** A step that read a file, thinking first. */
function readStep(model: string, reasoning: Json = { opaque: "enc", id: "rs_1" }, host?: string): Json {
	return {
		role: "step",
		producer: { model, ...(host === undefined ? {} : { host }) },
		parts: [
			{ kind: "thought", type: "reasoning", segments: [{ kind: "summary", text: "Plan." }], ...reasoning },
			{ kind: "text", text: "Reading." },
			{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: "{}" },
			{ kind: "result", index: 0, output: "source", isError: false },
		],
	};
}

const STEP_OF = (parts: Json[], model: string | null = GPT): Json => ({
	role: "step",
	...(model === null ? {} : { producer: { model } }),
	parts,
});

const USAGE = { input_tokens: 4, output_tokens: 2 };

////////////////////////////////
//  Requests

const requests: CaseSource[] = [
	asks(
		"responses-request-full",
		responses(),
		{
			summary: "Earlier.",
			state: "Live.",
			effort: "high",
			tools: [FINISH_TOOL],
			toolChoice: { name: "finish" },
			schema: TITLE_SCHEMA,
			maxTokens: 8,
		},
		"stream",
	),
	asks("responses-request-complete-path", responses(), {}, "complete"),
	asks("responses-request-no-facts", responses({ bare: true }), {
		effort: "high",
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
		schema: TITLE_SCHEMA,
	}),
	asks("responses-request-no-system", responses(), { system: [], history: [] }),
	asks("responses-request-empty-system-blocks-keep-their-gaps", responses(), {
		system: ["", "One.", ""],
		summary: "",
	}),
	asks("responses-request-tools-empty-list", responses(), { tools: [] }),
	// Forcing follows the catalog.
	asks("responses-choice-required-only", responses({ api: { choice: ["auto", "required"] } }), {
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
	}),
	asks("responses-choice-tool-wins-over-required", responses({ api: { choice: ["required", "tool"] } }), {
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
	}),
	asks("responses-choice-auto-only-sends-no-choice", responses({ api: { choice: ["auto"] } }), {
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
	}),
	asks("responses-choice-not-asked", responses(), { tools: [FINISH_TOOL] }),
	// Schemas follow the catalog.
	asks("responses-schema-enforced", responses(), { schema: TITLE_SCHEMA }),
	asks("responses-schema-in-prompt", responses({ api: { structured: false } }), { schema: TITLE_SCHEMA }),
	asks("responses-schema-in-prompt-after-summary", responses({ api: { structured: false } }), {
		schema: TITLE_SCHEMA,
		summary: "Earlier.",
	}),
	asks("responses-schema-refused-by-ref", responses(), {
		schema: { name: "tree", json: { $ref: "#/$defs/node", $defs: { node: { type: "object" } } } },
	}),
	asks("responses-schema-refused-optional-field", responses(), {
		schema: { name: "pick", json: { type: "object", properties: { a: { type: "string" } } } },
	}),
	// Effort is sent only when the model lists it.
	asks("responses-effort-listed", responses(), { effort: "low" }),
	asks("responses-effort-none-has-no-summary", responses(), { effort: "none" }),
	asks("responses-effort-not-listed", responses(), { effort: "medium" }),
	asks("responses-effort-model-lists-none", responses({ efforts: [] }), { effort: "high" }),
	// The token floor.
	asks("responses-tokens-below-floor", responses(), { maxTokens: 8 }),
	asks("responses-tokens-at-floor", responses(), { maxTokens: 16 }),
	asks("responses-tokens-above-floor", responses(), { maxTokens: 4096 }),
	asks("responses-tokens-absent", responses(), {}),
	// Key order.
	asks("responses-key-order-schema-enforced", responses(), { schema: SCRAMBLED }),
	asks("responses-key-order-schema-in-prompt", responses({ api: { structured: false } }), { schema: SCRAMBLED }),
	asks("responses-key-order-tools", responses(), { tools: [SCRAMBLED_TOOL], toolChoice: { name: "scramble" } }),
	asks("responses-key-order-call-arguments-are-text", responses(), {
		history: [
			STEP_OF([
				{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: '{"zeta":1,"alpha":2,"10":3}' },
				{ kind: "result", index: 0, output: "x", isError: false },
			]),
		],
	}),
];

////////////////////////////////
//  History and thought replay

const history: CaseSource[] = [
	// Same family replays reasoning, with ids and sealed content.
	asks("responses-replay-same-model", responses(), { history: [readStep(GPT)] }),
	asks("responses-replay-sibling-model", responses(), { model: "gpt-6-sol", history: [readStep(GPT)] }),
	asks("responses-replay-family-ignores-the-vendor-prefix", responses(), {
		model: "openai/gpt-6-sol",
		history: [readStep("gpt-6-luna")],
	}),
	asks("responses-replay-producer-prefixed", responses(), { history: [readStep("openai/gpt-6-luna")] }),
	asks("responses-replay-other-family", responses(), { model: "gpt-5.5", history: [readStep(GPT)] }),
	asks("responses-replay-other-vendor", responses(), { history: [readStep("claude-haiku-5-5")] }),
	asks("responses-replay-family-is-the-first-two-words", responses(), {
		model: "gpt-6-luna-mini",
		history: [readStep("gpt-6-sol-max")],
	}),
	asks("responses-replay-one-word-names", responses(), { model: "mini", history: [readStep("mini")] }),
	asks("responses-replay-no-producer", responses(), {
		history: [
			STEP_OF(
				[
					{ kind: "thought", type: "reasoning", opaque: "enc", id: "rs_1" },
					{ kind: "text", text: "Hi" },
				],
				null,
			),
		],
	}),
	// A part without an id or sealed content drops every thought of the step.
	asks("responses-replay-without-sealed-content-drops-the-step-thoughts", responses(), {
		history: [readStep(GPT, { id: "rs_1" })],
	}),
	asks("responses-replay-without-id-drops-the-step-thoughts", responses(), {
		history: [readStep(GPT, { opaque: "enc" })],
	}),
	asks("responses-replay-one-bad-part-drops-all", responses(), {
		history: [
			STEP_OF([
				{ kind: "thought", type: "reasoning", opaque: "enc_1", id: "rs_1" },
				{ kind: "thought", type: "reasoning", opaque: "enc_2" },
				{ kind: "text", text: "Hi" },
			]),
		],
	}),
	asks("responses-replay-other-thought-type-drops-the-step-thoughts", responses(), {
		history: [
			STEP_OF([
				{ kind: "thought", type: "thinking", opaque: "sig", id: "rs_1" },
				{ kind: "text", text: "Hi" },
			]),
		],
	}),
	asks("responses-replay-summary-and-content-segments", responses(), {
		history: [
			STEP_OF([
				{
					kind: "thought",
					type: "reasoning",
					opaque: "enc",
					id: "rs_1",
					segments: [
						{ kind: "content", text: "Raw one." },
						{ kind: "summary", text: "Short." },
						{ kind: "content", text: "Raw two." },
						{ kind: "summary", text: "Tail." },
					],
				},
			]),
		],
	}),
	asks("responses-replay-no-segments", responses(), {
		history: [STEP_OF([{ kind: "thought", type: "reasoning", opaque: "enc", id: "rs_1" }])],
	}),
	asks("responses-replay-a-step-with-no-thoughts", responses(), {
		history: [STEP_OF([{ kind: "text", text: "Hi" }])],
	}),
	// Phases: kept, or derived from whether the step called anything.
	asks("responses-phase-kept", responses(), {
		history: [STEP_OF([{ kind: "text", text: "Done.", phase: "final_answer" }])],
	}),
	asks("responses-phase-derived-final-answer", responses(), {
		history: [STEP_OF([{ kind: "text", text: "Done." }], "claude-haiku-5-5")],
	}),
	asks("responses-phase-derived-commentary-with-calls", responses(), {
		history: [
			STEP_OF([
				{ kind: "text", text: "Reading." },
				{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: "{}" },
			]),
		],
	}),
	asks("responses-phase-kept-over-derived", responses(), {
		history: [
			STEP_OF([
				{ kind: "text", text: "Reading.", phase: "final" },
				{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: "{}" },
			]),
		],
	}),
	asks("responses-text-empty-is-dropped", responses(), {
		history: [
			STEP_OF([
				{ kind: "text", text: "" },
				{ kind: "text", text: "Hi" },
			]),
		],
	}),
	// Calls and outputs.
	asks("responses-parallel-calls-then-outputs", responses(), {
		history: [
			{ role: "user", text: "Read." },
			STEP_OF([
				{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: '{"file":"a"}' },
				{ kind: "call", index: 1, wireId: "call_b", name: "read", arguments: '{"file":"b"}' },
				{ kind: "result", index: 0, output: "a", isError: false },
				{ kind: "result", index: 1, output: "b", isError: false },
			]),
		],
	}),
	asks("responses-outputs-pair-by-index", responses(), {
		history: [
			STEP_OF([
				{ kind: "call", index: 1, wireId: "call_x", name: "read", arguments: "{}" },
				{ kind: "call", index: 0, wireId: "call_y", name: "read", arguments: "{}" },
				{ kind: "result", index: 0, output: "second", isError: false },
				{ kind: "result", index: 1, output: "first", isError: true },
			]),
		],
	}),
	asks("responses-output-of-unknown-call-is-dropped", responses(), {
		history: [
			STEP_OF([
				{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: "{}" },
				{ kind: "result", index: 4, output: "lost", isError: false },
			]),
		],
	}),
	asks("responses-call-without-output", responses(), {
		history: [STEP_OF([{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: "{}" }])],
	}),
	asks("responses-state-after-history-as-developer", responses(), {
		state: "Live.",
		history: [{ role: "user", text: "Hi" }, STEP_OF([{ kind: "text", text: "Hello." }])],
	}),
	asks("responses-empty-step", responses(), { history: [STEP_OF([])] }),
];

////////////////////////////////
//  Stream decoding

const streamsOf: CaseSource[] = [
	streams(
		"responses-stream-summary-parts-stay-apart",
		responses(),
		event("response.reasoning_summary_text.delta", { item_id: "rs_1", summary_index: 0, delta: "One" }),
		event("response.reasoning_summary_text.delta", { item_id: "rs_1", summary_index: 1, delta: "Tw" }),
		event("response.reasoning_summary_text.delta", { item_id: "rs_1", summary_index: 1, delta: "o" }),
		done({ type: "reasoning", id: "rs_1", encrypted_content: "enc" }),
		done(MESSAGE([{ type: "output_text", text: "Hi" }], { phase: "final_answer" })),
		done({ ...CALL_ITEM, arguments: "{}" }),
		completed(),
	),
	streams(
		"responses-stream-content-parts-split-by-index",
		responses(),
		event("response.reasoning_text.delta", { item_id: "rs_1", content_index: 0, delta: "Raw" }),
		event("response.reasoning_text.delta", { item_id: "rs_1", content_index: 1, delta: "More" }),
		event("response.reasoning_summary_text.delta", { item_id: "rs_1", summary_index: 0, delta: "Short" }),
		completed(),
	),
	streams(
		"responses-stream-two-reasoning-items",
		responses(),
		event("response.reasoning_summary_text.delta", { item_id: "rs_1", summary_index: 0, delta: "A" }),
		event("response.reasoning_summary_text.delta", { item_id: "rs_2", summary_index: 0, delta: "B" }),
		event("response.reasoning_summary_text.delta", { item_id: "rs_1", summary_index: 0, delta: "C" }),
		completed(),
	),
	streams(
		"responses-stream-reasoning-index-missing-counts-as-zero",
		responses(),
		event("response.reasoning_summary_text.delta", { item_id: "rs_1", delta: "A" }),
		event("response.reasoning_summary_text.delta", { item_id: "rs_1", summary_index: "1", delta: "B" }),
		event("response.reasoning_summary_text.delta", { item_id: "rs_1", summary_index: 0, delta: "C" }),
		completed(),
	),
	streams(
		"responses-stream-done-reasoning-after-deltas-adds-only-the-seal",
		responses(),
		event("response.reasoning_summary_text.delta", { item_id: "rs_1", summary_index: 0, delta: "Think" }),
		done({
			type: "reasoning",
			id: "rs_1",
			summary: [{ type: "summary_text", text: "Think" }],
			encrypted_content: "enc",
		}),
		completed(),
	),
	streams(
		"responses-stream-done-reasoning-with-segments-and-content",
		responses(),
		done({
			type: "reasoning",
			id: "rs_1",
			summary: [{ type: "summary_text", text: "Short." }],
			content: [{ type: "reasoning_text", text: "Raw." }],
			encrypted_content: "enc",
		}),
		completed(),
	),
	streams(
		"responses-stream-done-reasoning-empty-adds-nothing",
		responses(),
		done({ type: "reasoning", id: "rs_1", summary: [], encrypted_content: "" }),
		done({ type: "reasoning", id: "rs_2" }),
		text("Hi"),
		completed(),
	),
	streams(
		"responses-stream-done-reasoning-segments-mistyped",
		responses(),
		done({
			type: "reasoning",
			id: "rs_1",
			summary: [{ type: "summary_text", text: 1 }, null, "x", { type: "summary_text" }, { text: "Kept." }],
			content: "not a list",
			encrypted_content: 7,
		}),
		completed(),
	),
	// Text and phases.
	streams(
		"responses-stream-text-phase-from-the-added-item",
		responses(),
		added(MESSAGE([], { phase: "commentary" })),
		text("Hi"),
		text(" there"),
		done(MESSAGE([{ type: "output_text", text: "Hi there" }], { phase: "commentary" })),
		completed(),
	),
	streams(
		"responses-stream-text-phase-mistyped-is-none",
		responses(),
		added(MESSAGE([], { phase: 3 })),
		text("Hi"),
		completed(),
	),
	streams(
		"responses-stream-text-phase-from-the-done-item",
		responses(),
		done(MESSAGE([{ type: "output_text", text: "Hi" }], { phase: "final_answer" })),
		completed(),
	),
	streams(
		"responses-stream-done-phase-does-not-replace-the-added-one",
		responses(),
		added(MESSAGE([], { phase: "commentary" })),
		done(MESSAGE([{ type: "output_text", text: "Hi" }], { phase: "final_answer" })),
		completed(),
	),
	streams(
		"responses-stream-refusal-delta-is-text",
		responses(),
		event("response.refusal.delta", { item_id: "msg_1", delta: "No." }),
		completed(),
	),
	streams(
		"responses-stream-done-message-joins-text-and-refusals",
		responses(),
		done(
			MESSAGE([
				{ type: "output_text", text: "A" },
				{ type: "refusal", refusal: "B" },
				null,
				{ type: "other" },
				"x",
				{ text: "C", refusal: "D" },
			]),
		),
		completed(),
	),
	streams(
		"responses-stream-done-message-after-deltas-adds-nothing",
		responses(),
		text("Hi"),
		done(MESSAGE([{ type: "output_text", text: "Hi" }])),
		completed(),
	),
	streams(
		"responses-stream-done-message-content-not-a-list",
		responses(),
		done(MESSAGE("Hi")),
		done(MESSAGE(null, { id: "msg_2" })),
		completed(),
	),
	streams(
		"responses-stream-delta-missing-is-empty-text",
		responses(),
		event("response.output_text.delta", { item_id: "msg_1" }),
		event("response.output_text.delta", { delta: 5 }),
		completed(),
	),
	streams("responses-stream-item-id-mistyped", responses(), text("Hi", 5), text("again", null), completed()),
	// Calls.
	streams(
		"responses-stream-call-by-deltas",
		responses(),
		added({ ...CALL_ITEM, arguments: "" }),
		event("response.function_call_arguments.delta", { item_id: "fc_1", delta: '{"path":' }),
		event("response.function_call_arguments.delta", { item_id: "fc_1", delta: '"a"}' }),
		event("response.function_call_arguments.done", { item_id: "fc_1", arguments: '{"path":"a"}' }),
		done(CALL_ITEM),
		completed(),
	),
	streams(
		"responses-stream-call-final-arguments-backfill",
		responses(),
		added({ ...CALL_ITEM, arguments: "" }),
		event("response.function_call_arguments.delta", { item_id: "fc_1", delta: '{"pa' }),
		event("response.function_call_arguments.done", { item_id: "fc_1", arguments: '{"path":"a"}' }),
		completed(),
	),
	streams(
		"responses-stream-call-final-arguments-that-do-not-extend-fail-the-stream",
		responses(),
		added({ ...CALL_ITEM, arguments: "" }),
		event("response.function_call_arguments.delta", { item_id: "fc_1", delta: '{"path":"a"}' }),
		event("response.function_call_arguments.done", { item_id: "fc_1", arguments: '{"other":1}' }),
		done({ ...CALL_ITEM, arguments: '{"other":1}' }),
		completed(),
	),
	streams("responses-stream-call-added-with-arguments", responses(), added(CALL_ITEM), done(CALL_ITEM), completed()),
	streams(
		"responses-stream-two-calls-index-by-arrival",
		responses(),
		added({ ...CALL_ITEM, arguments: "" }),
		added({ ...CALL_ITEM, id: "fc_2", call_id: "call_b", name: "write", arguments: "" }),
		event("response.function_call_arguments.delta", { item_id: "fc_2", delta: "{}" }),
		event("response.function_call_arguments.delta", { item_id: "fc_1", delta: "{}" }),
		completed(),
	),
	streams(
		"responses-stream-call-delta-for-an-unknown-item",
		responses(),
		event("response.function_call_arguments.delta", { item_id: "fc_9", delta: "{}" }),
		completed(),
	),
	streams(
		"responses-stream-call-done-without-arguments",
		responses(),
		event("response.function_call_arguments.done", { item_id: "fc_1" }),
		completed(),
	),
	streams(
		"responses-stream-call-fields-mistyped",
		responses(),
		added({ type: "function_call", id: 1, call_id: 2, name: ["x"], arguments: {} }),
		completed(),
	),
	// The end.
	streams("responses-stream-completed-with-calls-is-tool-calls", responses(), added({ ...CALL_ITEM }), completed()),
	streams("responses-stream-completed-no-calls-is-stop", responses(), text("Hi"), completed()),
	streams(
		"responses-stream-incomplete-max-tokens",
		responses(),
		text("Hi"),
		completed("incomplete", { reason: "max_output_tokens" }),
	),
	streams(
		"responses-stream-incomplete-content-filter",
		responses(),
		text("Hi"),
		completed("incomplete", { reason: "content_filter" }),
	),
	streams("responses-stream-incomplete-no-reason", responses(), text("Hi"), completed("incomplete")),
	streams(
		"responses-stream-incomplete-details-mistyped",
		responses(),
		text("Hi"),
		completed("incomplete", "content_filter"),
	),
	streams(
		"responses-stream-incomplete-reason-mistyped",
		responses(),
		text("Hi"),
		completed("incomplete", { reason: 4 }),
	),
	streams(
		"responses-stream-status-wins-over-the-event-type",
		responses(),
		text("Hi"),
		event("response.completed", { response: { id: "resp_1", status: "incomplete" } }),
	),
	streams(
		"responses-stream-status-missing-falls-back-to-the-event-type",
		responses(),
		text("Hi"),
		event("response.incomplete", { response: { id: "resp_1" } }),
	),
	streams(
		"responses-stream-status-mistyped-falls-back-to-the-event-type",
		responses(),
		text("Hi"),
		event("response.completed", { response: { id: "resp_1", status: 3 } }),
	),
	streams(
		"responses-stream-status-unknown-ends-the-step",
		responses(),
		text("Hi"),
		event("response.completed", { response: { status: "cancelled" } }),
	),
	streams("responses-stream-completed-without-a-response", responses(), text("Hi"), event("response.completed")),
	streams(
		"responses-stream-completed-response-not-an-object",
		responses(),
		event("response.completed", { response: "done" }),
	),
	streams(
		"responses-stream-items-after-the-end-are-never-read",
		responses(),
		text("Hi"),
		completed(),
		text("late"),
		data("{not json"),
	),
	streams("responses-stream-done-marker-before-the-end", responses(), text("Hi"), DONE),
	streams("responses-stream-no-end-is-truncated", responses(), text("Hi")),
	streams("responses-stream-no-items-is-truncated", responses()),
	// Failures.
	streams(
		"responses-stream-failed-status-bills-usage-and-serving",
		responses(),
		event("response.created", { response: { id: "resp_1" } }),
		text("Hi"),
		event("response.failed", {
			response: {
				id: "resp_1",
				status: "failed",
				error: { code: "server_error", message: "Down." },
				usage: { ...USAGE, cost: 0.1 },
			},
		}),
	),
	streams(
		"responses-stream-failed-overloaded-marker",
		responses(),
		text("Hi"),
		event("response.failed", {
			response: { status: "failed", error: { code: "server_error", message: "[overloaded] busy" } },
		}),
	),
	streams(
		"responses-stream-failed-no-error-object",
		responses(),
		event("response.failed", { response: { status: "failed" } }),
	),
	streams(
		"responses-stream-failed-error-text-is-the-message",
		responses(),
		event("response.failed", { response: { status: "failed", error: "It broke." } }),
	),
	streams(
		"responses-stream-failed-error-array-has-no-message",
		responses(),
		event("response.failed", { response: { status: "failed", error: ["x"] } }),
	),
	streams(
		"responses-stream-failed-error-mistyped-fields",
		responses(),
		event("response.failed", { response: { status: "failed", error: { code: 5, message: 6 } } }),
	),
	streams(
		"responses-stream-failed-error-empty-code-is-no-type",
		responses(),
		event("response.failed", { response: { status: "failed", error: { code: "", message: "No." } } }),
	),
	streams(
		"responses-stream-failed-by-the-event-type-alone",
		responses(),
		event("response.failed", { response: { error: { message: "Gone." } } }),
	),
	streams(
		"responses-stream-error-event-allowance",
		responses(),
		text("Hi"),
		event("error", { code: "GoUsageLimitError", message: "Usage limit reached." }),
	),
	streams("responses-stream-error-event-no-message", responses(), event("error")),
	streams("responses-stream-error-event-fields-mistyped", responses(), event("error", { code: 5, message: ["x"] })),
	streams(
		"responses-stream-error-event-overloaded-type",
		responses(),
		event("error", { code: "overloaded_error", message: "Slow down." }),
	),
	streams(
		"responses-stream-error-event-bills-the-serving-id",
		responses(),
		event("response.created", { response: { id: "resp_1" } }),
		event("response.in_progress", { response: { id: "resp_1", usage: { ...USAGE, cost: 0.5 } } }),
		event("error", { message: "Down." }),
	),
	// Serving.
	streams(
		"responses-stream-serving-first-id-only",
		responses(),
		event("response.created", { response: { id: "resp_1" } }),
		event("response.in_progress", { response: { id: "resp_2" } }),
		completed("completed", undefined, { id: "resp_3" }),
	),
	streams(
		"responses-stream-serving-from-the-end-event",
		responses(),
		text("Hi"),
		completed("completed", undefined, { id: "resp_9" }),
	),
	streams(
		"responses-stream-serving-id-empty-or-mistyped",
		responses(),
		event("response.created", { response: { id: "" } }),
		event("response.in_progress", { response: { id: 5 } }),
		completed("completed", undefined, { id: null }),
	),
	streams("responses-stream-created-without-a-response", responses(), event("response.created"), completed()),
	// Usage.
	streams(
		"responses-stream-usage-with-details",
		responses(),
		text("Hi"),
		completed("completed", undefined, {
			usage: {
				input_tokens: 100,
				input_tokens_details: { cached_tokens: 60, cache_write_tokens: 5 },
				output_tokens: 10,
				output_tokens_details: { reasoning_tokens: 4 },
			},
		}),
	),
	streams(
		"responses-stream-usage-with-billing",
		responses(),
		text("Hi"),
		completed("completed", undefined, {
			usage: {
				...USAGE,
				cost: 0.002,
				opencode: {
					billing_source: "go",
					list_cost: 0.01,
					allowance: 0.5,
					tokens: { reasoning: 3, cache_write_5m: 5, cache_write_1h: 2 },
				},
			},
		}),
	),
	streams(
		"responses-stream-usage-counts-only",
		responses(),
		text("Hi"),
		completed("completed", undefined, { usage: USAGE }),
	),
	streams(
		"responses-stream-usage-missing-output-is-zero",
		responses(),
		text("Hi"),
		completed("completed", undefined, { usage: { input_tokens: 4 } }),
	),
	streams(
		"responses-stream-usage-counts-mistyped-is-zero",
		responses(),
		text("Hi"),
		completed("completed", undefined, { usage: { input_tokens: "4", output_tokens: 2 } }),
	),
	streams(
		"responses-stream-usage-details-mistyped",
		responses(),
		text("Hi"),
		completed("completed", undefined, {
			usage: {
				...USAGE,
				input_tokens_details: { cached_tokens: "x", cache_write_tokens: null },
				output_tokens_details: [1],
			},
		}),
	),
	streams(
		"responses-stream-usage-not-an-object",
		responses(),
		text("Hi"),
		completed("completed", undefined, { usage: "x" }),
	),
	streams(
		"responses-stream-usage-on-an-in-progress-event-is-not-read",
		responses(),
		event("response.in_progress", { response: { usage: USAGE } }),
		completed(),
	),
	streams(
		"responses-stream-usage-of-the-end-event-only",
		responses(),
		event("response.in_progress", { response: { usage: USAGE } }),
		completed("completed", undefined, { usage: { input_tokens: 9, output_tokens: 8 } }),
	),
	// Event names and comments.
	streams(
		"responses-stream-comments-are-ignored",
		responses(),
		comment("ping"),
		text("Hi"),
		comment("ping"),
		completed(),
	),
	streams(
		"responses-stream-the-event-line-names-the-type",
		responses(),
		eventNamed("response.output_text.delta", { item_id: "msg_1", delta: "Hi" }),
		eventNamed("response.completed", { response: { id: "resp_1", status: "completed" } }),
	),
	streams(
		"responses-stream-the-data-type-wins-over-the-event-line",
		responses(),
		{
			kind: "data",
			event: "response.failed",
			data: JSON.stringify({ type: "response.output_text.delta", item_id: "m", delta: "Hi" }),
		},
		completed(),
	),
	streams(
		"responses-stream-no-type-anywhere-is-ignored",
		responses(),
		data({ item_id: "msg_1", delta: "Hi" }),
		completed(),
	),
	streams(
		"responses-stream-null-type-uses-the-event-line",
		responses(),
		{
			kind: "data",
			event: "response.output_text.delta",
			data: JSON.stringify({ type: null, item_id: "m", delta: "Hi" }),
		},
		completed(),
	),
	streams(
		"responses-stream-type-mistyped-is-ignored",
		responses(),
		data({ type: 5, item_id: "msg_1", delta: "Hi" }),
		completed(),
	),
	streams(
		"responses-stream-unknown-event-types-are-ignored",
		responses(),
		event("response.something.new", { delta: "x" }),
		event("response.content_part.added"),
		completed(),
	),
	streams(
		"responses-stream-item-not-an-object",
		responses(),
		added({}),
		event("response.output_item.added", { item: "x" }),
		event("response.output_item.done", { item: null }),
		event("response.output_item.done"),
		completed(),
	),
	streams(
		"responses-stream-item-of-an-unknown-type",
		responses(),
		added({ type: "web_search_call", id: "w_1" }),
		done({ type: "web_search_call", id: "w_1" }),
		completed(),
	),
	streams("responses-stream-chunk-not-json", responses(), data("{not json")),
	streams("responses-stream-chunk-is-an-array", responses(), data("[1]")),
	streams("responses-stream-chunk-is-null", responses(), data("null")),
	streams("responses-stream-chunk-is-a-number", responses(), data("5")),
	streams("responses-stream-chunk-is-empty-text", responses(), data("")),
];

////////////////////////////////
//  Whole replies

const BODY = (over: Json = {}): Json => ({ id: "resp_1", status: "completed", output: [], ...over });

const replies: CaseSource[] = [
	completes(
		"responses-reply-text",
		responses(),
		BODY({ output: [MESSAGE([{ type: "output_text", text: "Hello." }])] }),
	),
	completes(
		"responses-reply-text-with-phase",
		responses(),
		BODY({ output: [MESSAGE([{ type: "output_text", text: "Hello." }], { phase: "final_answer" })] }),
	),
	completes(
		"responses-reply-refusal",
		responses(),
		BODY({ output: [MESSAGE([{ type: "refusal", refusal: "No." }])] }),
	),
	completes("responses-reply-call", responses(), BODY({ output: [CALL_ITEM] })),
	completes(
		"responses-reply-two-calls-by-position",
		responses(),
		BODY({ output: [CALL_ITEM, { ...CALL_ITEM, id: "fc_2", call_id: "call_b", name: "write" }] }),
	),
	completes(
		"responses-reply-call-arguments-keep-their-text",
		responses(),
		BODY({ output: [{ ...CALL_ITEM, arguments: '{"zeta":1,"alpha":2,"10":3}' }] }),
	),
	completes(
		"responses-reply-call-without-arguments",
		responses(),
		BODY({ output: [{ type: "function_call", id: "fc_1", call_id: "c", name: "read" }] }),
	),
	completes(
		"responses-reply-reasoning-and-text-and-call",
		responses(),
		BODY({
			output: [
				{
					type: "reasoning",
					id: "rs_1",
					summary: [{ type: "summary_text", text: "Plan." }],
					encrypted_content: "enc",
				},
				MESSAGE([{ type: "output_text", text: "Reading." }], { phase: "commentary" }),
				CALL_ITEM,
			],
		}),
	),
	completes(
		"responses-reply-incomplete-length",
		responses(),
		BODY({ status: "incomplete", incomplete_details: { reason: "max_output_tokens" } }),
	),
	completes(
		"responses-reply-incomplete-filter",
		responses(),
		BODY({ status: "incomplete", incomplete_details: { reason: "content_filter" } }),
	),
	completes("responses-reply-no-output-stop", responses(), BODY()),
	completes("responses-reply-output-not-a-list", responses(), BODY({ output: "x" })),
	completes("responses-reply-output-items-not-objects", responses(), BODY({ output: [1, null, "x", [CALL_ITEM]] })),
	completes("responses-reply-no-status-ends-the-step", responses(), { id: "resp_1", output: [] }),
	completes("responses-reply-status-unknown-ends-the-step", responses(), BODY({ status: "queued" })),
	completes("responses-reply-status-mistyped-ends-the-step", responses(), BODY({ status: 1 })),
	completes(
		"responses-reply-failed",
		responses(),
		BODY({ status: "failed", error: { code: "server_error", message: "Down." }, usage: { ...USAGE, cost: 0.1 } }),
	),
	completes("responses-reply-failed-error-text", responses(), BODY({ status: "failed", error: "It broke." })),
	completes("responses-reply-failed-no-error", responses(), BODY({ status: "failed" })),
	completes(
		"responses-reply-usage",
		responses(),
		BODY({
			usage: {
				input_tokens: 100,
				input_tokens_details: { cached_tokens: 60 },
				output_tokens: 10,
				output_tokens_details: { reasoning_tokens: 4 },
			},
		}),
	),
	completes("responses-reply-usage-no-id", responses(), { status: "completed", usage: USAGE }),
	completes("responses-reply-id-mistyped", responses(), BODY({ id: 5 })),
	completes("responses-reply-is-an-array", responses(), [1]),
	completes("responses-reply-is-text", responses(), "ok"),
	completes("responses-reply-is-null", responses(), null),
	completes("responses-reply-empty-object", responses(), {}),
	completes(
		"responses-reply-reasoning-with-no-segments-and-no-seal-adds-nothing",
		responses(),
		BODY({ output: [{ type: "reasoning", id: "rs_1", summary: [] }] }),
	),
];

////////////////////////////////
//  HTTP failures

const BODY_ERR = (type: string, message: string): Json => ({ error: { type, message } });

const failures: CaseSource[] = [
	fails("responses-http-400-bad-request", responses(), 400, BODY_ERR("invalid_request_error", "Bad field.")),
	fails(
		"responses-http-400-model-unavailable",
		responses(),
		400,
		BODY_ERR("invalid_request_error", "Model is unavailable."),
	),
	fails(
		"responses-http-400-model-unavailable-ignores-case",
		responses(),
		400,
		BODY_ERR("x", "The MODEL IS UNAVAILABLE right now"),
	),
	fails("responses-http-400-no-body", responses(), 400, null),
	fails("responses-http-401", responses(), 401, BODY_ERR("invalid_request_error", "Invalid API key.")),
	fails("responses-http-402", responses(), 402, BODY_ERR("x", "Insufficient balance.")),
	fails("responses-http-403", responses(), 403, BODY_ERR("x", "Forbidden.")),
	fails("responses-http-404", responses(), 404, BODY_ERR("x", "No such model.")),
	fails("responses-http-408", responses(), 408, BODY_ERR("x", "Slow.")),
	fails("responses-http-418", responses(), 418, BODY_ERR("x", "Teapot.")),
	fails("responses-http-429-rate-limited", responses(), 429, BODY_ERR("rate_limit_error", "Slow down.")),
	fails(
		"responses-http-429-retry-after-header-wins",
		responses({ nowMs: NOW }),
		429,
		{ error: { message: "Slow.", resetsAt: NOW / 1000 + 90 } },
		[["Retry-After", "5"]],
	),
	fails("responses-http-429-reset-in-the-future", responses({ nowMs: NOW }), 429, {
		error: { message: "Slow.", resetsAt: NOW / 1000 + 90 },
	}),
	fails("responses-http-429-reset-in-the-past", responses({ nowMs: NOW }), 429, {
		error: { message: "Slow.", resetsAt: NOW / 1000 - 90 },
	}),
	fails("responses-http-429-reset-as-an-iso-time", responses({ nowMs: NOW }), 429, {
		error: { message: "Slow.", resetsAt: "2027-01-15T08:01:30Z" },
	}),
	fails("responses-http-429-reset-mistyped", responses({ nowMs: NOW }), 429, {
		error: { message: "Slow.", resetsAt: true },
	}),
	fails("responses-http-429-overloaded-marker", responses(), 429, BODY_ERR("x", "[overloaded] busy")),
	fails("responses-http-429-overloaded-type", responses(), 429, BODY_ERR("overloaded_error", "Busy.")),
	fails("responses-http-529-overloaded-marker-is-not-a-429", responses(), 529, BODY_ERR("x", "[overloaded] busy")),
	fails("responses-http-429-allowance", responses({ nowMs: NOW }), 429, {
		type: "error",
		error: { type: "GoUsageLimitError", message: "Usage limit reached.", resetsAt: NOW / 1000 + 3600 },
	}),
	fails("responses-http-429-allowance-reset-in-the-past", responses({ nowMs: NOW }), 429, {
		type: "error",
		error: { type: "GoUsageLimitError", message: "Usage limit reached.", resetsAt: NOW / 1000 - 3600 },
	}),
	fails("responses-http-429-budget-limit", responses({ nowMs: NOW }), 429, {
		type: "error",
		error: { type: "BudgetLimitError", message: "Budget spent.", resetsAt: "2027-01-15T09:00:00Z" },
	}),
	fails(
		"responses-http-429-allowance-no-reset",
		responses(),
		429,
		BODY_ERR("GoUsageLimitError", "Usage limit reached."),
	),
	fails("responses-http-500", responses(), 500, BODY_ERR("server_error", "Down.")),
	fails("responses-http-503", responses(), 503, BODY_ERR("x", "Unavailable.")),
	fails("responses-http-error-status-field-is-the-type", responses(), 429, {
		error: { status: "GoUsageLimitError", message: "Limit." },
	}),
	fails("responses-http-error-text-is-the-message", responses(), 429, { error: "Slow down." }),
	fails("responses-http-message-blank", responses(), 500, { error: { message: "   " } }),
	fails("responses-http-message-mistyped", responses(), 500, { error: { message: 5, type: 7 } }),
	fails("responses-http-body-array", responses(), 500, [1]),
	fails("responses-http-body-text", responses(), 502, "Bad gateway"),
	fails("responses-http-body-empty-object", responses(), 503, {}),
];

////////////////////////////////
//  Cases

export const responsesCases: CaseSource[] = [...requests, ...history, ...streamsOf, ...replies, ...failures];
