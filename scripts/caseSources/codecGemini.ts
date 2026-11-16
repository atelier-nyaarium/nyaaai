// Codec cases: the generateContent codec.

import type { CaseSource } from "../caseAdapter";
import { comment, DONE, data, type Json } from "./codecFixtures";
import { completes, encodes, fails, streams } from "./codecKit";
import { chunk, FLASH, gemini, NOW, SIGNED_CALL } from "./codecNativeFixtures";

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

const asks = (name: string, codec: Json, over: Json = {}, mode: "stream" | "complete" = "stream"): CaseSource =>
	encodes(name, codec, { model: FLASH, ...over }, mode);

const stepOf = (parts: Json[], model: string | null = FLASH): Json => ({
	role: "step",
	...(model === null ? {} : { producer: { model } }),
	parts,
});

const PLAN = { kind: "thought", type: "thought", segments: [{ kind: "content", text: "Plan." }] };
const ON_FIRST = { kind: "thought", type: "thought", opaque: "sig", anchor: "call:0" };

/** Two parallel reads, the signature on the first call. */
const readsStep = (model: string | null = FLASH, thoughts: Json[] = [PLAN, ON_FIRST]): Json =>
	stepOf(
		[
			...thoughts,
			{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: '{"file":"a"}' },
			{ kind: "call", index: 1, wireId: "call_b", name: "read", arguments: '{"file":"b"}' },
			{ kind: "result", index: 0, output: "a", isError: false },
			{ kind: "result", index: 1, output: "gone", isError: true },
		],
		model,
	);

const OLDER = [{ role: "user", text: "Read." }, "STEP", { role: "user", text: "Again." }];

const withStep = (step: Json): Json[] => OLDER.map((turn) => (turn === "STEP" ? step : (turn as Json)));

const finished = (reason: unknown, parts: unknown[] = [{ text: "Hi" }], top: Json = {}): Json =>
	chunk(parts, { finishReason: reason }, top);

const FINISH_REASONS = [
	"STOP",
	"MAX_TOKENS",
	"CONTINUATION",
	"SAFETY",
	"RECITATION",
	"LANGUAGE",
	"BLOCKLIST",
	"PROHIBITED_CONTENT",
	"SPII",
	"IMAGE_SAFETY",
	"IMAGE_PROHIBITED_CONTENT",
	"IMAGE_RECITATION",
	"MALFORMED_FUNCTION_CALL",
	"OTHER",
	"UNEXPECTED_TOOL_CALL",
	"stop",
	"FINISH_REASON_UNSPECIFIED",
];

const USAGE = { promptTokenCount: 10, candidatesTokenCount: 4 };

////////////////////////////////
//  Requests

const requests: CaseSource[] = [
	asks("gemini-request-full", gemini(), {
		summary: "Earlier.",
		state: "Live.",
		effort: "high",
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
		schema: TITLE_SCHEMA,
		maxTokens: 900,
	}),
	asks("gemini-request-complete-path", gemini(), {}, "complete"),
	asks("gemini-request-no-facts", gemini({ bare: true }), {
		effort: "high",
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
		schema: TITLE_SCHEMA,
	}),
	asks("gemini-request-no-system", gemini(), { system: [], history: [] }),
	asks("gemini-request-empty-system-blocks-are-dropped", gemini(), { system: ["", "One.", ""], summary: "" }),
	asks("gemini-request-tools-empty-list", gemini(), { tools: [] }),
	// The model in the path.
	asks("gemini-path-model-with-reserved-characters", gemini(), { model: "tuned/gemini 3:flash?x=1" }, "stream"),
	asks(
		"gemini-path-model-with-reserved-characters-complete",
		gemini(),
		{ model: "tuned/gemini 3:flash?x=1" },
		"complete",
	),
	asks("gemini-path-model-non-ascii", gemini(), { model: "gem\u{e9}ni-\u{1f600}" }),
	asks("gemini-path-model-keeps-its-marks", gemini(), { model: "a-b_c.d!e~f*g'h(i)" }),
	// Forcing follows the catalog.
	asks("gemini-choice-required-is-any", gemini({ api: { choice: ["auto", "required"] } }), {
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
	}),
	asks("gemini-choice-tool-wins-over-required", gemini({ api: { choice: ["required", "tool"] } }), {
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
	}),
	asks("gemini-choice-auto-only-sends-no-choice", gemini({ api: { choice: ["auto"] } }), {
		tools: [FINISH_TOOL],
		toolChoice: { name: "finish" },
	}),
	asks("gemini-choice-not-asked", gemini(), { tools: [FINISH_TOOL] }),
	// Schemas follow the catalog.
	asks("gemini-schema-enforced", gemini(), { schema: TITLE_SCHEMA }),
	asks("gemini-schema-in-prompt", gemini({ api: { structured: false } }), { schema: TITLE_SCHEMA }),
	asks("gemini-schema-in-prompt-after-summary", gemini({ api: { structured: false } }), {
		schema: TITLE_SCHEMA,
		summary: "Earlier.",
	}),
	asks("gemini-schema-refused-by-ref", gemini(), {
		schema: { name: "tree", json: { $ref: "#/$defs/node", $defs: { node: { type: "object" } } } },
	}),
	asks("gemini-schema-refused-optional-field", gemini(), {
		schema: { name: "pick", json: { type: "object", properties: { a: { type: "string" } } } },
	}),
	// Thinking follows the model's efforts.
	...["minimal", "low", "medium", "high"].map((effort) =>
		asks(`gemini-thinking-level-${effort}`, gemini({ efforts: ["minimal", "low", "medium", "high"] }), { effort }),
	),
	...["none", "xhigh", "max"].map((effort) =>
		asks(`gemini-thinking-effort-${effort}-has-no-level`, gemini({ efforts: ["none", "xhigh", "max"] }), {
			effort,
		}),
	),
	asks("gemini-thinking-effort-not-listed", gemini(), { effort: "xhigh" }),
	asks("gemini-thinking-not-asked", gemini(), {}),
	asks("gemini-thinking-none-when-no-efforts", gemini({ efforts: [] }), { effort: "high" }),
	asks("gemini-generation-omitted-when-empty", gemini({ efforts: [] }), {}),
	asks("gemini-generation-max-tokens-only", gemini({ efforts: [] }), { maxTokens: 64 }),
	// Key order.
	asks("gemini-key-order-schema-enforced", gemini(), { schema: SCRAMBLED }),
	asks("gemini-key-order-schema-in-prompt", gemini({ api: { structured: false } }), { schema: SCRAMBLED }),
	asks("gemini-key-order-tools", gemini(), { tools: [SCRAMBLED_TOOL], toolChoice: { name: "scramble" } }),
	asks("gemini-key-order-call-args", gemini(), {
		history: [
			stepOf([
				{
					kind: "call",
					index: 0,
					wireId: "call_a",
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
	// Signatures go back on their anchors, to the same model only.
	asks("gemini-replay-same-model", gemini(), { history: [readsStep(FLASH)] }),
	asks("gemini-replay-same-model-older-step", gemini(), { history: withStep(readsStep(FLASH)) }),
	asks("gemini-replay-other-model-current-turn-skips-validation", gemini(), {
		history: [readsStep("gemini-4-argon")],
	}),
	asks("gemini-replay-other-model-older-step-has-no-sentinel", gemini(), {
		history: withStep(readsStep("gemini-4-argon")),
	}),
	asks("gemini-replay-no-producer-current-turn", gemini(), { history: [readsStep(null)] }),
	asks("gemini-replay-other-vendor", gemini(), { history: [readsStep("gpt-6-luna")] }),
	asks("gemini-replay-prefixed-producer-is-another-model", gemini(), {
		history: [readsStep("google/gemini-3.8-flash")],
	}),
	asks("gemini-replay-sentinel-only-on-the-first-call", gemini(), {
		history: [readsStep("gemini-4-argon", [PLAN])],
	}),
	asks("gemini-replay-signed-first-call-keeps-its-signature", gemini(), { history: [readsStep(FLASH, [ON_FIRST])] }),
	asks("gemini-replay-signature-on-the-second-call", gemini(), {
		history: [readsStep(FLASH, [{ kind: "thought", type: "thought", opaque: "sig_b", anchor: "call:1" }])],
	}),
	asks("gemini-replay-both-calls-signed", gemini(), {
		history: [
			readsStep(FLASH, [
				{ kind: "thought", type: "thought", opaque: "sig_a", anchor: "call:0" },
				{ kind: "thought", type: "thought", opaque: "sig_b", anchor: "call:1" },
			]),
		],
	}),
	asks("gemini-replay-first-call-unsigned-second-signed-gets-the-sentinel-first", gemini(), {
		history: [readsStep(FLASH, [{ kind: "thought", type: "thought", opaque: "sig_b", anchor: "call:1" }])],
	}),
	asks("gemini-replay-signature-on-text-is-an-empty-part", gemini(), {
		history: [
			stepOf([
				{ kind: "text", text: "Done." },
				{ kind: "thought", type: "thought", opaque: "sig_t", anchor: "text" },
			]),
		],
	}),
	asks("gemini-replay-text-anchor-without-a-signature-is-dropped", gemini(), {
		history: [
			stepOf([
				{ kind: "text", text: "Done." },
				{ kind: "thought", type: "thought", anchor: "text" },
			]),
		],
	}),
	asks("gemini-replay-unknown-anchor-is-dropped", gemini(), {
		history: [
			stepOf([
				{ kind: "thought", type: "thought", opaque: "sig_x", anchor: "call:9" },
				{ kind: "thought", type: "thought", opaque: "sig_y", anchor: "elsewhere" },
				{ kind: "text", text: "Done." },
			]),
		],
	}),
	asks("gemini-replay-anchor-for-a-missing-call-signs-nothing", gemini(), {
		history: [
			stepOf([
				{ kind: "thought", type: "thought", opaque: "sig_x", anchor: "call:5" },
				{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: "{}" },
				{ kind: "result", index: 0, output: "a", isError: false },
			]),
		],
	}),
	asks("gemini-replay-thought-without-a-signature", gemini(), { history: [readsStep(FLASH, [PLAN])] }),
	asks("gemini-replay-thought-with-text-and-signature", gemini(), {
		history: [
			stepOf([
				{
					kind: "thought",
					type: "thought",
					opaque: "sig",
					segments: [
						{ kind: "content", text: "One " },
						{ kind: "summary", text: "two." },
					],
				},
				{ kind: "text", text: "Done." },
			]),
		],
	}),
	asks("gemini-replay-thought-with-no-segments", gemini(), {
		history: [
			stepOf([
				{ kind: "thought", type: "thought" },
				{ kind: "text", text: "Done." },
			]),
		],
	}),
	asks("gemini-replay-other-thought-type-drops-every-thought", gemini(), {
		history: [
			readsStep(FLASH, [
				PLAN,
				ON_FIRST,
				{ kind: "thought", type: "reasoning.text", segments: [{ kind: "content", text: "?" }] },
			]),
		],
	}),
	asks("gemini-replay-a-step-with-no-thoughts-current-turn", gemini(), {
		history: [
			stepOf([
				{ kind: "call", index: 0, wireId: "call_a", name: "read", arguments: "{}" },
				{ kind: "result", index: 0, output: "a", isError: false },
			]),
		],
	}),
	asks("gemini-replay-a-step-with-no-calls-gets-no-sentinel", gemini(), {
		history: [stepOf([{ kind: "text", text: "Hi." }], "gemini-4-argon")],
	}),
	asks("gemini-replay-no-user-turn-makes-every-step-current", gemini(), {
		history: [
			stepOf([{ kind: "call", index: 0, wireId: "a", name: "read", arguments: "{}" }], null),
			stepOf([{ kind: "call", index: 0, wireId: "b", name: "read", arguments: "{}" }], null),
		],
	}),
	asks("gemini-replay-steps-after-the-last-user-turn-are-current", gemini(), {
		history: [
			{ role: "user", text: "One." },
			stepOf([{ kind: "call", index: 0, wireId: "a", name: "read", arguments: "{}" }], null),
			{ role: "user", text: "Two." },
			stepOf([{ kind: "call", index: 0, wireId: "b", name: "read", arguments: "{}" }], null),
			stepOf([{ kind: "call", index: 0, wireId: "c", name: "read", arguments: "{}" }], null),
		],
	}),
	// Calls and results.
	asks("gemini-results-pair-by-index", gemini(), {
		history: [
			stepOf([
				{ kind: "call", index: 1, wireId: "x", name: "write", arguments: "{}" },
				{ kind: "call", index: 0, wireId: "y", name: "read", arguments: "{}" },
				{ kind: "result", index: 0, output: "first", isError: false },
				{ kind: "result", index: 1, output: "second", isError: true },
			]),
		],
	}),
	asks("gemini-result-of-unknown-call-is-dropped", gemini(), {
		history: [
			stepOf([
				{ kind: "call", index: 0, wireId: "a", name: "read", arguments: "{}" },
				{ kind: "result", index: 4, output: "lost", isError: false },
			]),
		],
	}),
	asks("gemini-call-without-result", gemini(), {
		history: [stepOf([{ kind: "call", index: 0, wireId: "a", name: "read", arguments: "{}" }])],
	}),
	asks("gemini-call-arguments-not-json", gemini(), {
		history: [stepOf([{ kind: "call", index: 0, wireId: "a", name: "read", arguments: "{not json" }])],
	}),
	asks("gemini-call-arguments-not-an-object", gemini(), {
		history: [
			stepOf([
				{ kind: "call", index: 0, wireId: "a", name: "read", arguments: "[1]" },
				{ kind: "call", index: 1, wireId: "b", name: "read", arguments: "7" },
				{ kind: "call", index: 2, wireId: "c", name: "read", arguments: "" },
			]),
		],
	}),
	asks("gemini-text-empty-is-dropped", gemini(), {
		history: [
			stepOf([
				{ kind: "text", text: "" },
				{ kind: "text", text: "Hi" },
			]),
		],
	}),
	asks("gemini-empty-step-is-dropped", gemini(), {
		history: [{ role: "user", text: "Hi" }, stepOf([]), { role: "user", text: "Again" }],
	}),
	asks("gemini-state-is-a-last-user-content", gemini(), {
		history: [{ role: "user", text: "Hi" }, stepOf([{ kind: "text", text: "Hello." }])],
		state: "Live.",
	}),
	asks("gemini-state-alone", gemini(), { history: [], state: "Live." }),
];

////////////////////////////////
//  Stream decoding

const streamsOf: CaseSource[] = [
	// Finish reasons.
	...FINISH_REASONS.map((reason) =>
		streams(
			`gemini-stream-finish-${reason.toLowerCase().replaceAll("_", "-")}${reason === "stop" ? "-lowercase" : ""}`,
			gemini(),
			finished(reason),
		),
	),
	streams("gemini-stream-finish-stop-with-a-call-is-tool-calls", gemini(), finished("STOP", [SIGNED_CALL])),
	streams("gemini-stream-finish-max-tokens-with-a-call-is-length", gemini(), finished("MAX_TOKENS", [SIGNED_CALL])),
	streams(
		"gemini-stream-finish-missing-signature-is-replay-rejected",
		gemini(),
		finished("MISSING_THOUGHT_SIGNATURE"),
	),
	streams("gemini-stream-finish-reason-mistyped-is-no-finish", gemini(), finished(5)),
	streams("gemini-stream-finish-reason-empty-is-no-finish", gemini(), finished("")),
	streams("gemini-stream-no-finish-reason-is-truncated", gemini(), chunk([{ text: "Hi" }])),
	streams("gemini-stream-no-items-is-truncated", gemini()),
	streams("gemini-stream-done-marker-without-a-finish", gemini(), chunk([{ text: "Hi" }]), DONE),
	streams("gemini-stream-done-marker-after-a-finish", gemini(), finished("STOP"), DONE),
	streams(
		"gemini-stream-finish-in-a-later-chunk",
		gemini(),
		chunk([{ text: "Hi" }]),
		chunk([{ text: " there" }]),
		finished("STOP", []),
	),
	// Parts.
	streams(
		"gemini-stream-thought-text-and-text",
		gemini(),
		chunk([{ text: "Think", thought: true }]),
		chunk([{ text: "ing.", thought: true }]),
		chunk([{ text: "Reading." }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-thought-with-signature-only",
		gemini(),
		chunk([{ text: "", thought: true, thoughtSignature: "sig_1" }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-thought-empty-is-no-event",
		gemini(),
		chunk([{ text: "", thought: true }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-thought-flag-mistyped-is-text",
		gemini(),
		chunk([
			{ text: "A", thought: "true" },
			{ text: "B", thought: 1 },
			{ text: "C", thought: false },
		]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-text-with-signature-is-anchored",
		gemini(),
		chunk([{ text: "Done.", thoughtSignature: "sig_t" }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-empty-text-with-signature-is-only-the-anchor",
		gemini(),
		chunk([{ text: "", thoughtSignature: "sig_t" }]),
		finished("STOP", []),
	),
	streams("gemini-stream-empty-text-is-no-event", gemini(), chunk([{ text: "" }]), finished("STOP", [])),
	streams(
		"gemini-stream-signature-mistyped-is-none",
		gemini(),
		chunk([{ text: "A", thoughtSignature: 5 }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-signature-empty-is-none",
		gemini(),
		chunk([{ text: "A", thoughtSignature: "" }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-call-with-signature-anchors-its-index",
		gemini(),
		chunk([{ functionCall: { name: "read", args: { path: "a" } } }]),
		chunk([{ functionCall: { id: "call_b", name: "write", args: {} }, thoughtSignature: "sig_b" }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-calls-without-ids-take-the-response-id",
		gemini(),
		data({ candidates: [{ content: { parts: [{ functionCall: { name: "read", args: {} } }] } }], responseId: "r" }),
		data({ candidates: [{ content: { parts: [{ text: "Done." }] } }] }),
		data({ candidates: [{ content: { parts: [{ text: "", thoughtSignature: "sig" }] }, finishReason: "STOP" }] }),
	),
	streams(
		"gemini-stream-calls-in-one-chunk",
		gemini(),
		chunk([{ functionCall: { name: "read", args: { a: 1 } } }, { functionCall: { name: "read", args: { a: 2 } } }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-call-args-keep-their-key-order",
		gemini(),
		chunk([{ functionCall: { name: "read", args: { zeta: 1, alpha: { b: 1, a: 2 }, "10": 3, "2": 4 } } }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-call-args-not-an-object",
		gemini(),
		chunk([
			{ functionCall: { name: "a", args: "x" } },
			{ functionCall: { name: "b", args: [1] } },
			{ functionCall: { name: "c" } },
		]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-call-fields-mistyped",
		gemini(),
		chunk([{ functionCall: { id: 1, name: ["x"], args: {} } }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-function-call-not-an-object",
		gemini(),
		chunk([{ functionCall: "read" }, { functionCall: null, text: "kept" }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-unknown-part-kinds-are-ignored",
		gemini(),
		chunk([{ inlineData: { mimeType: "x", data: "y" } }, {}, { executableCode: {} }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-text-mistyped-is-ignored",
		gemini(),
		chunk([{ text: 5 }, { text: ["x"] }, { text: null }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-parts-not-objects",
		gemini(),
		chunk([null, "x", 5, [1], { text: "kept" }]),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-parts-not-a-list",
		gemini(),
		data({ candidates: [{ content: { parts: "x" }, finishReason: "STOP" }] }),
	),
	streams(
		"gemini-stream-content-not-an-object",
		gemini(),
		data({ candidates: [{ content: "x", finishReason: "STOP" }] }),
	),
	streams("gemini-stream-no-content", gemini(), data({ candidates: [{ finishReason: "STOP" }] })),
	streams("gemini-stream-candidates-empty", gemini(), data({ candidates: [] }), finished("STOP", [])),
	streams("gemini-stream-candidates-not-a-list", gemini(), data({ candidates: { finishReason: "STOP" } })),
	streams(
		"gemini-stream-candidate-not-an-object",
		gemini(),
		data({ candidates: ["x"] }),
		data({ candidates: [null] }),
	),
	streams(
		"gemini-stream-only-the-first-candidate-is-read",
		gemini(),
		data({
			candidates: [
				{ content: { parts: [{ text: "A" }] }, finishReason: "STOP" },
				{ content: { parts: [{ text: "B" }] } },
			],
		}),
	),
	streams("gemini-stream-chunk-without-candidates", gemini(), data({ responseId: "r" }), finished("STOP", [])),
	// Usage and serving.
	streams(
		"gemini-stream-usage-on-the-first-chunk",
		gemini(),
		chunk([{ text: "Hi" }], {}, { usageMetadata: { promptTokenCount: 100, cachedContentTokenCount: 60 } }),
		finished("STOP", [{ text: "!" }], {
			usageMetadata: {
				promptTokenCount: 100,
				cachedContentTokenCount: 60,
				candidatesTokenCount: 6,
				thoughtsTokenCount: 4,
			},
		}),
	),
	streams(
		"gemini-stream-usage-counts-thoughts-and-tool-prompts",
		gemini(),
		finished("STOP", [{ text: "Hi" }], {
			usageMetadata: {
				promptTokenCount: 100,
				toolUsePromptTokenCount: 20,
				candidatesTokenCount: 6,
				thoughtsTokenCount: 4,
			},
		}),
	),
	streams(
		"gemini-stream-usage-prompt-count-only",
		gemini(),
		finished("STOP", [{ text: "Hi" }], { usageMetadata: { promptTokenCount: 5 } }),
	),
	streams(
		"gemini-stream-usage-without-a-prompt-count-is-no-usage",
		gemini(),
		finished("STOP", [{ text: "Hi" }], { usageMetadata: { candidatesTokenCount: 5 } }),
	),
	streams(
		"gemini-stream-usage-without-a-prompt-count-repeats-the-last-usage",
		gemini(),
		chunk([{ text: "Hi" }], {}, { usageMetadata: USAGE }),
		finished("STOP", [{ text: "!" }], { usageMetadata: {} }),
	),
	streams(
		"gemini-stream-usage-counts-mistyped",
		gemini(),
		finished("STOP", [{ text: "Hi" }], {
			usageMetadata: {
				promptTokenCount: 5,
				candidatesTokenCount: "x",
				thoughtsTokenCount: null,
				cachedContentTokenCount: [],
			},
		}),
	),
	streams(
		"gemini-stream-usage-prompt-count-mistyped",
		gemini(),
		finished("STOP", [{ text: "Hi" }], { usageMetadata: { promptTokenCount: "5" } }),
	),
	streams("gemini-stream-usage-not-an-object", gemini(), finished("STOP", [{ text: "Hi" }], { usageMetadata: "x" })),
	streams(
		"gemini-stream-usage-with-billing",
		gemini(),
		finished("STOP", [{ text: "Hi" }], {
			usageMetadata: {
				...USAGE,
				cost: 0.002,
				opencode: {
					billing_source: "go",
					list_cost: 0.01,
					allowance: 0.5,
					tokens: { cache_write_5m: 5, cache_write_1h: 2 },
				},
			},
		}),
	),
	streams(
		"gemini-stream-billing-source-unknown-and-negatives-are-dropped",
		gemini(),
		finished("STOP", [{ text: "Hi" }], {
			usageMetadata: { ...USAGE, cost: -1, opencode: { billing_source: "gift", list_cost: -2, allowance: -3 } },
		}),
	),
	streams(
		"gemini-stream-chunks-after-the-finish-add-only-usage",
		gemini(),
		finished("STOP", [{ text: "Hi" }]),
		chunk([{ text: " more" }], { finishReason: "MAX_TOKENS" }, { usageMetadata: USAGE, responseId: "late" }),
	),
	streams(
		"gemini-stream-chunk-after-the-finish-without-usage-adds-nothing",
		gemini(),
		finished("STOP", [{ text: "Hi" }], { usageMetadata: USAGE }),
		chunk([{ text: " more" }]),
	),
	streams(
		"gemini-stream-serving-first-id-only",
		gemini(),
		chunk([{ text: "A" }], {}, { responseId: "r_1" }),
		chunk([{ text: "B" }], {}, { responseId: "r_2" }),
		finished("STOP", []),
	),
	streams(
		"gemini-stream-serving-id-empty-or-mistyped",
		gemini(),
		chunk([{ text: "A" }], {}, { responseId: "" }),
		chunk([{ text: "B" }], {}, { responseId: 5 }),
		finished("STOP", [], { responseId: "r_3" }),
	),
	streams(
		"gemini-stream-no-response-id",
		gemini(),
		data({ candidates: [{ content: { parts: [{ text: "A" }] }, finishReason: "STOP" }] }),
	),
	// Failures.
	streams(
		"gemini-stream-error-after-output-bills-usage",
		gemini(),
		chunk([{ text: "Hi" }]),
		data({ error: { code: 503, status: "UNAVAILABLE", message: "Down." }, usageMetadata: USAGE }),
	),
	streams(
		"gemini-stream-error-overloaded-marker",
		gemini(),
		data({ error: { code: 503, status: "UNAVAILABLE", message: "[overloaded] busy" } }),
	),
	streams(
		"gemini-stream-error-allowance",
		gemini(),
		data({ error: { code: 429, status: "GoUsageLimitError", message: "Usage limit reached." } }),
	),
	streams("gemini-stream-error-code-only", gemini(), data({ error: { code: 400 } })),
	streams("gemini-stream-error-code-as-text", gemini(), data({ error: { code: "400", message: "Bad." } })),
	streams("gemini-stream-error-no-message", gemini(), data({ error: { status: "INTERNAL" } })),
	streams("gemini-stream-error-empty-object", gemini(), data({ error: {} })),
	streams("gemini-stream-error-text-is-the-message", gemini(), data({ error: "It broke." })),
	streams("gemini-stream-error-array-has-no-message", gemini(), data({ error: ["x"] })),
	streams("gemini-stream-error-true-has-no-message", gemini(), data({ error: true })),
	streams("gemini-stream-error-number-has-no-message", gemini(), data({ error: 500 })),
	streams(
		"gemini-stream-error-null-is-no-error",
		gemini(),
		data({ error: null, candidates: [{ content: { parts: [{ text: "Hi" }] }, finishReason: "STOP" }] }),
	),
	streams(
		"gemini-stream-error-false-is-no-error",
		gemini(),
		data({ error: false, candidates: [{ content: { parts: [{ text: "Hi" }] }, finishReason: "STOP" }] }),
	),
	streams(
		"gemini-stream-error-zero-is-no-error",
		gemini(),
		data({ error: 0, candidates: [{ content: { parts: [{ text: "Hi" }] }, finishReason: "STOP" }] }),
	),
	streams(
		"gemini-stream-error-empty-text-is-no-error",
		gemini(),
		data({ error: "", candidates: [{ content: { parts: [{ text: "Hi" }] }, finishReason: "STOP" }] }),
	),
	streams("gemini-stream-error-fields-mistyped", gemini(), data({ error: { code: "x", status: 5, message: ["x"] } })),
	streams(
		"gemini-stream-error-bills-serving",
		gemini(),
		chunk([{ text: "Hi" }], {}, { responseId: "r_1" }),
		data({ error: { message: "Down." } }),
	),
	streams(
		"gemini-stream-error-before-the-finish-wins-over-candidates",
		gemini(),
		data({
			error: { message: "Down." },
			candidates: [{ content: { parts: [{ text: "Hi" }] }, finishReason: "STOP" }],
		}),
	),
	streams(
		"gemini-stream-error-after-the-finish-is-ignored",
		gemini(),
		finished("STOP"),
		data({ error: { message: "Late." } }),
	),
	streams("gemini-stream-blocked-prompt", gemini(), data({ promptFeedback: { blockReason: "SAFETY" } })),
	streams(
		"gemini-stream-blocked-prompt-other-reason",
		gemini(),
		data({ promptFeedback: { blockReason: "PROHIBITED_CONTENT" } }),
	),
	streams(
		"gemini-stream-blocked-prompt-with-a-candidate-is-not-blocked",
		gemini(),
		data({
			promptFeedback: { blockReason: "SAFETY" },
			candidates: [{ content: { parts: [{ text: "Hi" }] }, finishReason: "STOP" }],
		}),
	),
	streams("gemini-stream-blocked-prompt-reason-mistyped", gemini(), data({ promptFeedback: { blockReason: 3 } })),
	streams("gemini-stream-blocked-prompt-feedback-not-an-object", gemini(), data({ promptFeedback: "SAFETY" })),
	streams(
		"gemini-stream-blocked-prompt-empty-candidates",
		gemini(),
		data({ promptFeedback: { blockReason: "OTHER" }, candidates: [] }),
	),
	// Comments and loose chunks.
	streams("gemini-stream-comments-are-ignored", gemini(), comment("ping"), finished("STOP"), comment("ping")),
	streams("gemini-stream-event-names-are-ignored", gemini(), {
		kind: "data",
		event: "message",
		data: JSON.stringify({ candidates: [{ content: { parts: [{ text: "Hi" }] }, finishReason: "STOP" }] }),
	}),
	streams("gemini-stream-chunk-not-json", gemini(), data("{not json")),
	streams("gemini-stream-chunk-is-an-array", gemini(), data("[1]")),
	streams("gemini-stream-chunk-is-null", gemini(), data("null")),
	streams("gemini-stream-chunk-is-empty-text", gemini(), data("")),
	streams("gemini-stream-empty-chunk-object", gemini(), data({}), finished("STOP")),
];

////////////////////////////////
//  Whole replies

const BODY = (over: Json = {}): Json => ({
	candidates: [{ content: { role: "model", parts: [{ text: "Hello." }] }, finishReason: "STOP" }],
	responseId: "resp_g",
	usageMetadata: { promptTokenCount: 4, candidatesTokenCount: 2 },
	...over,
});

const replies: CaseSource[] = [
	completes("gemini-reply-text", gemini(), BODY()),
	completes(
		"gemini-reply-thoughts-text-and-signed-call",
		gemini(),
		BODY({
			candidates: [
				{
					content: {
						role: "model",
						parts: [{ text: "Thinking.", thought: true }, { text: "Reading." }, SIGNED_CALL],
					},
					finishReason: "STOP",
				},
			],
		}),
	),
	completes(
		"gemini-reply-two-calls-by-position",
		gemini(),
		BODY({
			candidates: [
				{
					content: {
						parts: [
							{ functionCall: { name: "read", args: {} } },
							{ functionCall: { name: "write", args: {} }, thoughtSignature: "sig_b" },
						],
					},
					finishReason: "STOP",
				},
			],
		}),
	),
	completes(
		"gemini-reply-signature-on-text",
		gemini(),
		BODY({
			candidates: [{ content: { parts: [{ text: "Done.", thoughtSignature: "sig_t" }] }, finishReason: "STOP" }],
		}),
	),
	completes(
		"gemini-reply-max-tokens",
		gemini(),
		BODY({ candidates: [{ content: { parts: [{ text: "Cut" }] }, finishReason: "MAX_TOKENS" }] }),
	),
	completes("gemini-reply-safety", gemini(), BODY({ candidates: [{ finishReason: "SAFETY" }] })),
	completes(
		"gemini-reply-no-finish-reason-ends-the-step",
		gemini(),
		BODY({ candidates: [{ content: { parts: [{ text: "Hi" }] } }] }),
	),
	completes("gemini-reply-no-candidates-ends-the-step", gemini(), { responseId: "r" }),
	completes(
		"gemini-reply-missing-signature",
		gemini(),
		BODY({ candidates: [{ finishReason: "MISSING_THOUGHT_SIGNATURE" }] }),
	),
	completes("gemini-reply-blocked-prompt", gemini(), { promptFeedback: { blockReason: "SAFETY" } }),
	completes("gemini-reply-error-object", gemini(), { error: { code: 503, status: "UNAVAILABLE", message: "Down." } }),
	completes("gemini-reply-error-text-is-the-message", gemini(), { error: "It broke." }),
	completes("gemini-reply-error-array", gemini(), { error: [1] }),
	completes("gemini-reply-error-null-is-no-error", gemini(), BODY({ error: null })),
	completes("gemini-reply-error-empty-text-is-no-error", gemini(), BODY({ error: "" })),
	completes(
		"gemini-reply-usage-with-billing",
		gemini(),
		BODY({
			usageMetadata: {
				promptTokenCount: 100,
				cachedContentTokenCount: 60,
				candidatesTokenCount: 6,
				thoughtsTokenCount: 4,
				cost: 0.002,
				opencode: { billing_source: "go", tokens: { cache_write_5m: 5, cache_write_1h: 2 } },
			},
		}),
	),
	completes("gemini-reply-no-usage", gemini(), BODY({ usageMetadata: undefined })),
	completes("gemini-reply-usage-mistyped", gemini(), BODY({ usageMetadata: { promptTokenCount: "4" } })),
	completes("gemini-reply-no-response-id", gemini(), BODY({ responseId: undefined })),
	completes("gemini-reply-response-id-mistyped", gemini(), BODY({ responseId: 5 })),
	completes("gemini-reply-is-an-array", gemini(), [1]),
	completes("gemini-reply-is-text", gemini(), "ok"),
	completes("gemini-reply-is-null", gemini(), null),
	completes("gemini-reply-empty-object", gemini(), {}),
];

////////////////////////////////
//  HTTP failures

const ERR = (code: number, status: string, message: string): Json => ({ error: { code, status, message } });

const failures: CaseSource[] = [
	fails("gemini-http-400-bad-request", gemini(), 400, ERR(400, "INVALID_ARGUMENT", "Bad field.")),
	fails("gemini-http-400-model-unavailable", gemini(), 400, ERR(400, "INVALID_ARGUMENT", "Model is unavailable.")),
	fails(
		"gemini-http-400-model-unavailable-ignores-case",
		gemini(),
		400,
		ERR(400, "x", "The MODEL IS UNAVAILABLE right now"),
	),
	fails("gemini-http-400-no-body", gemini(), 400, null),
	fails("gemini-http-401", gemini(), 401, ERR(401, "UNAUTHENTICATED", "API key not valid.")),
	fails("gemini-http-402", gemini(), 402, ERR(402, "x", "Insufficient balance.")),
	fails("gemini-http-403", gemini(), 403, ERR(403, "PERMISSION_DENIED", "Forbidden.")),
	fails("gemini-http-404", gemini(), 404, ERR(404, "NOT_FOUND", "No such model.")),
	fails("gemini-http-408", gemini(), 408, ERR(408, "x", "Slow.")),
	fails("gemini-http-429-rate-limited", gemini(), 429, ERR(429, "RESOURCE_EXHAUSTED", "Slow down.")),
	fails(
		"gemini-http-429-retry-after-header-wins",
		gemini({ nowMs: NOW }),
		429,
		{ error: { message: "Slow.", resetsAt: NOW / 1000 + 90 } },
		[["Retry-After", "5"]],
	),
	fails("gemini-http-429-reset-in-the-future", gemini({ nowMs: NOW }), 429, {
		error: { message: "Slow.", resetsAt: NOW / 1000 + 90 },
	}),
	fails("gemini-http-429-reset-in-the-past", gemini({ nowMs: NOW }), 429, {
		error: { message: "Slow.", resetsAt: NOW / 1000 - 90 },
	}),
	fails("gemini-http-429-overloaded-marker", gemini(), 429, ERR(429, "x", "[overloaded] busy")),
	fails("gemini-http-429-allowance-by-status", gemini({ nowMs: NOW }), 429, {
		error: { status: "GoUsageLimitError", message: "Usage limit reached.", resetsAt: NOW / 1000 + 3600 },
	}),
	fails("gemini-http-429-allowance-reset-in-the-past", gemini({ nowMs: NOW }), 429, {
		error: { type: "GoUsageLimitError", message: "Usage limit reached.", resetsAt: NOW / 1000 - 3600 },
	}),
	fails("gemini-http-429-budget-limit", gemini({ nowMs: NOW }), 429, {
		error: { type: "BudgetLimitError", message: "Budget spent.", resetsAt: "2027-01-15T09:00:00Z" },
	}),
	fails("gemini-http-500", gemini(), 500, ERR(500, "INTERNAL", "Down.")),
	fails("gemini-http-503", gemini(), 503, ERR(503, "UNAVAILABLE", "Unavailable.")),
	fails("gemini-http-error-text-is-the-message", gemini(), 500, { error: "Down." }),
	fails("gemini-http-message-blank", gemini(), 500, { error: { message: "   " } }),
	fails("gemini-http-body-array", gemini(), 500, [1]),
	fails("gemini-http-body-text", gemini(), 502, "Bad gateway"),
	fails("gemini-http-body-empty-object", gemini(), 503, {}),
];

////////////////////////////////
//  Cases

export const geminiCases: CaseSource[] = [...requests, ...history, ...streamsOf, ...replies, ...failures];
