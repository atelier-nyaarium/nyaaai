// Transport cases: requests, replies and cancellation.

import type { CaseSource } from "../caseAdapter";
import {
	commentFrame,
	completeCall,
	DONE_TEXT,
	exchange,
	finish,
	frame,
	getCall,
	HI_FRAMES,
	JSON_HEAD,
	jsonReply,
	piece,
	postCall,
	REQUEST,
	splitAt,
	sseReply,
	streamCall,
	text,
} from "./transportKit";

////////////////////////////////
//  Constants

const REPLY = {
	id: "gen-1",
	provider: "Azure",
	choices: [{ message: { content: '{"title":"A"}' }, finish_reason: "stop" }],
	usage: { prompt_tokens: 4, completion_tokens: 2, cost: 0.1 },
};

const TITLE_REQUEST = {
	...REQUEST,
	schema: {
		name: "title",
		json: {
			type: "object",
			properties: { title: { type: "string" } },
			required: ["title"],
			additionalProperties: false,
		},
	},
};

const REFUSED_SCHEMA = { name: "pick", json: { type: "object", properties: { a: { type: "string" } } } };

/** A text with a two-byte and a four-byte character. */
const WIDE_FRAMES = [frame(text("h\u{e9}llo \u{1f600}")), frame(finish("stop")), DONE_TEXT].join("");

/** Where the two wide characters begin, in bytes. */
const WIDE_E = Buffer.from(WIDE_FRAMES).indexOf("\u{e9}");
const WIDE_FACE = Buffer.from(WIDE_FRAMES).indexOf("\u{1f600}");

/** The pieces of a streamed `Hi`, one frame each. */
const HI_PIECES = [
	piece(commentFrame(" PROCESSING")),
	piece(frame(text("Hi"))),
	piece(frame(finish("stop"))),
	piece(DONE_TEXT),
];

////////////////////////////////
//  Cases

const requests: CaseSource[] = [
	exchange(
		"stream-posts-codec-bodies-under-own-headers",
		[streamCall()],
		[
			sseReply([
				piece(
					`${commentFrame(" OPENROUTER PROCESSING")}${frame(text("Hi"))}${frame(finish("stop"))}${DONE_TEXT}`,
				),
			]),
		],
	),
	exchange(
		"stream-chunk-edges-do-not-matter",
		[streamCall()],
		[sseReply(splitAt(WIDE_FRAMES, 7, WIDE_E + 1, WIDE_FACE + 1, WIDE_FACE + 3))],
	),
	exchange("stream-codec-headers-come-first", [streamCall()], [sseReply(HI_PIECES)], {
		codecHeaders: { "X-Session": "root-1", "X-Trace": "t-1" },
	}),
	exchange("stream-codec-claims-authorization-and-agent", [streamCall()], [sseReply(HI_PIECES)], {
		codecHeaders: { Authorization: "Bearer other", "User-Agent": "other", "X-Session": "root-1" },
	}),
	exchange("stream-codec-cannot-override-owned-headers", [streamCall()], [sseReply(HI_PIECES)], {
		codecHeaders: {
			Authorization: "Bearer other",
			"User-Agent": "other",
			accept: "text/plain",
			"CONTENT-TYPE": "text/plain",
			"X-Session": "root-1",
		},
	}),
	exchange("stream-codec-cannot-set-authorization-without-a-key", [streamCall()], [sseReply(HI_PIECES)], {
		auth: "none",
		codecHeaders: { authorization: "Bearer other", "X-Session": "root-1" },
	}),
	exchange("stream-without-a-done-is-truncated", [streamCall()], [sseReply([piece(frame(text("Hi")))])]),
	exchange(
		"stream-stops-reading-at-done",
		[streamCall()],
		[sseReply([piece(HI_FRAMES), piece(frame(text("late"))), piece(DONE_TEXT)])],
	),
	exchange("stream-empty-ok-answer-is-truncated", [streamCall()], [sseReply([])]),
	exchange(
		"stream-a-chunk-that-is-not-json",
		[streamCall()],
		[sseReply([piece(frame(text("Hi"))), piece("data: {not json\n\n")])],
	),
	exchange(
		"stream-error-chunk-keeps-what-was-billed",
		[streamCall()],
		[
			sseReply([
				piece(frame(text("Hi"))),
				piece(
					frame({
						id: "gen-1",
						provider: "Azure",
						error: { code: 502, message: "Down" },
						usage: { prompt_tokens: 4, completion_tokens: 1, cost: 0.1 },
					}),
				),
				piece(DONE_TEXT),
			]),
		],
	),
	exchange(
		"stream-redirect-is-a-failure",
		[streamCall()],
		[jsonReply({ error: { code: 302, message: "Moved" } }, 302)],
	),
	exchange("stream-status-299-is-a-stream", [streamCall()], [sseReply(HI_PIECES, { status: 299 })]),
	exchange("stream-status-199-is-a-failure", [streamCall()], [jsonReply({ error: { message: "Early" } }, 199)]),
	exchange(
		"two-streams-on-one-transport",
		[streamCall(), streamCall({}, { ...REQUEST, model: "anthropic/claude-opus-5.5" })],
		[sseReply(HI_PIECES), sseReply(HI_PIECES)],
	),
	exchange(
		"encoder-refusal-is-a-bad-request-before-anything-is-sent",
		[streamCall({}, { ...REQUEST, schema: REFUSED_SCHEMA })],
		[],
	),
	exchange("complete-refusal-sends-nothing", [completeCall({}, { ...REQUEST, schema: REFUSED_SCHEMA })], []),
];

const completes: CaseSource[] = [
	exchange("complete-answers-the-codec-result", [completeCall({}, TITLE_REQUEST)], [jsonReply(REPLY)]),
	exchange(
		"complete-reports-an-error-under-ok",
		[completeCall()],
		[jsonReply({ error: { code: 400, message: "Bad schema" } })],
	),
	exchange(
		"complete-body-in-chunks",
		[completeCall()],
		[{ status: 200, headers: JSON_HEAD, chunks: splitAt(JSON.stringify(REPLY), 5, 60, 61) }],
	),
	exchange("complete-reply-that-is-not-json", [completeCall()], [jsonReply("<html>busy</html>")]),
	exchange("complete-empty-reply", [completeCall()], [{ status: 200, headers: JSON_HEAD, chunks: [] }]),
	exchange("complete-reply-that-is-null", [completeCall()], [jsonReply("null")]),
	exchange(
		"complete-http-error",
		[completeCall()],
		[jsonReply({ error: { code: 402, message: "Insufficient credits" } }, 402)],
	),
	exchange(
		"complete-without-done-or-finish",
		[completeCall()],
		[jsonReply({ id: "gen-1", choices: [{ message: { content: "x" } }] })],
	),
	exchange(
		"complete-reply-nested-past-the-limit",
		[completeCall()],
		[jsonReply(`${"[".repeat(1100)}${"]".repeat(1100)}`)],
	),
	exchange(
		"complete-reply-with-a-lone-surrogate",
		[completeCall()],
		[
			jsonReply(
				'{"id":"gen-1","provider":"Azure","choices":[{"message":{"content":"a\\ud800b"},"finish_reason":"stop"}]}',
			),
		],
	),
];

const json: CaseSource[] = [
	exchange(
		"public-reads-send-no-key",
		[getCall("/models", { keyAuth: { kind: "none" } })],
		[jsonReply({ data: [] })],
	),
	exchange("get-json-sends-the-stored-key", [getCall("/models")], [jsonReply({ data: [{ id: "m" }] })]),
	exchange("get-json-sends-no-body-and-no-content-type", [getCall("/key")], [jsonReply({ ok: true })]),
	exchange("post-json-sends-its-body", [postCall("/probe", { a: 1, b: ["x"] })], [jsonReply({ ok: true })]),
	exchange("post-json-body-null-is-sent", [postCall("/probe", null)], [jsonReply({ ok: true })]),
	exchange("post-json-without-a-body-sends-none", [postCall("/probe", undefined)], [jsonReply({ ok: true })]),
	exchange(
		"post-json-body-is-ecmascript-text",
		[postCall("/probe", { z: 1, a: { b: [1, 2.5, -0] }, u: "\u{e9}\u{1f600}" })],
		[jsonReply({ ok: true })],
	),
	exchange("get-json-answers-null", [getCall("/models")], [jsonReply("null")]),
	exchange("get-json-answer-that-is-not-json", [getCall("/models")], [jsonReply("not json")]),
	exchange("get-json-empty-answer", [getCall("/models")], [{ status: 200, headers: JSON_HEAD, chunks: [] }]),
	exchange(
		"get-json-http-error-is-decoded",
		[getCall("/models")],
		[jsonReply({ error: { message: "No auth" } }, 401)],
	),
	exchange("get-json-http-error-without-json", [getCall("/models")], [jsonReply("gateway down", 502)]),
	exchange(
		"get-json-refused",
		[getCall("/models")],
		[{ status: 0, headers: [], chunks: [], end: "refused", error: "connect ECONNREFUSED lab.local:11434" }],
	),
	exchange(
		"get-json-body-dropped",
		[getCall("/models")],
		[{ status: 200, headers: JSON_HEAD, chunks: [piece('{"data":')], end: "dropped" }],
	),
	exchange(
		"get-json-answer-in-chunks-with-a-split-character",
		[getCall("/models")],
		[{ status: 200, headers: JSON_HEAD, chunks: splitAt('{"name":"h\u{e9}llo \u{1f600}"}', 11, 18) }],
	),
	exchange(
		"json-calls-have-no-idle-abort",
		[getCall("/models")],
		[jsonReply({ data: [] }, 200, [], { headMs: 300_000 })],
	),
	exchange(
		"json-calls-are-not-logged-or-settled",
		[getCall("/models"), postCall("/probe", {})],
		[jsonReply({}), jsonReply({})],
	),
	exchange(
		"json-caller-abort-before-the-head",
		[getCall("/models", { abortAfterMs: 10 })],
		[jsonReply({}, 200, [], { headMs: 50 })],
	),
	exchange(
		"json-caller-abort-before-the-call-makes-no-request",
		[getCall("/models", { abortedBefore: true })],
		[jsonReply({})],
	),
	exchange(
		"json-caller-abort-while-reading",
		[getCall("/models", { abortAfterMs: 10 })],
		[{ status: 200, headers: JSON_HEAD, chunks: [piece("{", 5), piece("}", 20)] }],
	),
];

const cancels: CaseSource[] = [
	exchange("caller-abort-before-the-head", [streamCall({ abortAfterMs: 10 })], [sseReply(HI_PIECES, { headMs: 50 })]),
	exchange(
		"caller-abort-before-the-call-makes-no-request",
		[streamCall({ abortedBefore: true })],
		[sseReply(HI_PIECES)],
	),
	exchange(
		"caller-abort-between-chunks",
		[streamCall({ abortAfterMs: 25 })],
		[sseReply([piece(frame(text("Hi")), 10), piece(frame(text(" there")), 20), piece(DONE_TEXT)])],
	),
	exchange(
		"caller-abort-just-as-a-chunk-arrives",
		[streamCall({ abortAfterMs: 10 })],
		[sseReply([piece(frame(text("Hi")), 10), piece(frame(text(" there")), 10), piece(DONE_TEXT)])],
	),
	exchange(
		"caller-abort-after-the-end",
		[streamCall({ abortAfterMs: 1000 })],
		[sseReply([piece(frame(text("Hi")), 5), piece(frame(finish("stop")), 5), piece(DONE_TEXT, 5)])],
	),
	exchange(
		"caller-abort-while-reading-an-error-body",
		[streamCall({ abortAfterMs: 10 })],
		[{ status: 500, headers: JSON_HEAD, chunks: [piece('{"error":', 5), piece('{"message":"x"}}', 30)] }],
	),
	exchange(
		"caller-abort-during-complete",
		[completeCall({ abortAfterMs: 10 })],
		[jsonReply(REPLY, 200, [], { headMs: 40 })],
	),
	exchange(
		"caller-abort-leaves-the-next-call-alone",
		[streamCall({ abortAfterMs: 10 }), streamCall()],
		[sseReply(HI_PIECES, { headMs: 50 }), sseReply(HI_PIECES)],
	),
	exchange("idle-before-the-head", [streamCall()], [sseReply(HI_PIECES, { headMs: 100 })], { idleMs: 30 }),
	exchange(
		"idle-a-stream-that-goes-silent",
		[streamCall()],
		[sseReply([piece(frame(text("Hi"))), piece("", 1_000_000)])],
		{ idleMs: 30 },
	),
	exchange(
		"idle-between-chunks",
		[streamCall()],
		[sseReply([piece(frame(text("Hi")), 5), piece(frame(text(" there")), 50), piece(DONE_TEXT)])],
		{ idleMs: 30 },
	),
	exchange(
		"idle-silence-is-idle-but-comments-keep-the-stream-alive",
		[streamCall()],
		[
			sseReply([
				piece(":ping\n", 15),
				piece(":ping\n", 15),
				piece(":ping\n", 15),
				piece(":ping\n", 15),
				piece(frame(text("Hi")), 15),
				piece(frame(finish("stop")), 15),
				piece(DONE_TEXT, 15),
			]),
		],
		{ idleMs: 40 },
	),
	exchange(
		"idle-edge-a-chunk-at-exactly-the-limit-is-idle",
		[streamCall()],
		[sseReply([piece(frame(text("Hi")), 30), piece(frame(finish("stop")), 30), piece(DONE_TEXT, 30)])],
		{ idleMs: 30 },
	),
	exchange(
		"idle-edge-a-chunk-just-inside-the-limit-is-alive",
		[streamCall()],
		[sseReply([piece(frame(text("Hi")), 29), piece(frame(finish("stop")), 29), piece(DONE_TEXT, 29)])],
		{ idleMs: 30 },
	),
	exchange(
		"idle-default-limit-is-two-minutes",
		[streamCall()],
		[
			sseReply([
				piece(frame(text("Hi")), 119_999),
				piece(frame(finish("stop")), 119_999),
				piece(DONE_TEXT, 119_999),
			]),
		],
	),
	exchange("idle-default-limit-passed", [streamCall()], [sseReply(HI_PIECES, { headMs: 120_001 })]),
	exchange(
		"idle-an-unstreamed-body-has-one-window-after-its-head",
		[completeCall()],
		[
			{
				status: 200,
				headers: JSON_HEAD,
				chunks: [piece(JSON.stringify(REPLY).slice(0, 10), 20), piece(JSON.stringify(REPLY).slice(10), 20)],
			},
		],
		{ idleMs: 30 },
	),
	exchange(
		"idle-an-unstreamed-body-inside-its-window",
		[completeCall()],
		[
			{
				status: 200,
				headers: JSON_HEAD,
				chunks: [piece(JSON.stringify(REPLY).slice(0, 10), 10), piece(JSON.stringify(REPLY).slice(10), 10)],
			},
		],
		{ idleMs: 30 },
	),
	exchange(
		"idle-an-error-body-has-one-window-from-the-request",
		[streamCall()],
		[{ status: 500, headers: JSON_HEAD, chunks: [piece('{"error":', 20), piece('{"message":"x"}}', 20)] }],
		{ idleMs: 30 },
	),
	exchange("idle-a-slow-head-then-a-fast-body", [streamCall()], [sseReply(HI_PIECES, { headMs: 25 })], {
		idleMs: 30,
	}),
	exchange(
		"idle-then-the-next-call-is-clean",
		[streamCall(), streamCall()],
		[sseReply(HI_PIECES, { headMs: 100 }), sseReply(HI_PIECES)],
		{ idleMs: 30 },
	),
	exchange("dropped-mid-stream", [streamCall()], [sseReply([piece(frame(text("Hi")))], { end: "dropped" })]),
	exchange(
		"dropped-mid-stream-with-a-reason",
		[streamCall()],
		[sseReply([piece(frame(text("Hi")))], { end: "dropped", error: "socket hang up" })],
	),
	exchange(
		"dropped-in-the-middle-of-a-frame",
		[streamCall()],
		[sseReply([piece(frame(text("Hi")).slice(0, 20))], { end: "dropped" })],
	),
	exchange(
		"dropped-an-unstreamed-body",
		[completeCall()],
		[{ status: 200, headers: JSON_HEAD, chunks: [piece('{"id":"gen-1"')], end: "dropped" }],
	),
	exchange(
		"dropped-an-error-body",
		[streamCall()],
		[{ status: 500, headers: JSON_HEAD, chunks: [piece('{"error":')], end: "dropped" }],
	),
	exchange(
		"refused-before-the-head",
		[streamCall()],
		[{ status: 0, headers: [], chunks: [], end: "refused", error: "connect ECONNREFUSED lab.local:11434" }],
	),
	exchange(
		"refused-after-a-wait",
		[streamCall()],
		[{ status: 0, headers: [], chunks: [], end: "refused", headMs: 20 }],
	),
	exchange(
		"refused-then-the-next-call-works",
		[streamCall(), streamCall()],
		[{ status: 0, headers: [], chunks: [], end: "refused" }, sseReply(HI_PIECES)],
	),
];

export const flowCases: CaseSource[] = [...requests, ...completes, ...json, ...cancels];
