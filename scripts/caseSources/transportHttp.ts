// Transport cases: HTTP failures, Retry-After, the error body cap and recorded traces.

import type { CaseSource } from "../caseAdapter";
import { opencode } from "./codecFixtures";
import {
	completeCall,
	DONE_TEXT,
	exchange,
	finish,
	frame,
	JSON_HEAD,
	jsonReply,
	NOW,
	piece,
	REQUEST,
	repeat,
	replay,
	splitAt,
	sseReply,
	streamCall,
	text,
} from "./transportKit";

////////////////////////////////
//  Constants

/** The wheel's first instant, as an HTTP date. */
const NOW_DATE = "Fri, 15 Jan 2027 08:00:00 GMT";

/** The cap on an error body, in bytes. */
const CAP = 65_536;

const SLOW = { error: { code: 429, message: "Slow down" } };

const GONE = { error: { code: 503, message: "Gone" } };

const TRACE_REQUEST = {
	...REQUEST,
	model: "openai/gpt-6-luna",
	effort: "low",
	session: "00000000-0000-4000-8000-000000000000",
	tools: [
		{
			name: "lookup",
			description: "Look up the value stored under a key.",
			parameters: {
				type: "object",
				properties: { key: { type: "string" } },
				required: ["key"],
				additionalProperties: false,
			},
		},
	],
};

const ASK = 'Use the lookup tool for the keys "alpha" and "beta", in parallel if you can. Then reply with both values.';

////////////////////////////////
//  Functions & Helpers

/** An HTTP 429 that answers with this `Retry-After` header. */
const retried = (name: string, value: string, over: Record<string, unknown> = {}): CaseSource =>
	exchange(`retry-after-${name}`, [streamCall()], [jsonReply(SLOW, 429, [["retry-after", value]])], over);

/** An error body made of a valid JSON object and `pad` spaces, then `tail`. */
function padded(pad: number, tail: string): Record<string, unknown> {
	return {
		status: 500,
		headers: JSON_HEAD,
		chunks: [piece('{"error":{"message":"m"}}'), repeat(" ", pad), piece(tail)],
	};
}

/** A JSON error body with a two-byte character. */
const ACCENTED = '{"error":{"message":"h\u{e9}"}}';

const OBJECT_BYTES = '{"error":{"message":"m"}}'.length;

////////////////////////////////
//  Cases

const failures: CaseSource[] = [
	exchange(
		"http-429-rate-limited-with-seconds",
		[streamCall()],
		[jsonReply({ error: { code: 429, message: "Slow down" } }, 429, [["retry-after", "7"]])],
	),
	exchange(
		"http-503-unavailable-with-a-date",
		[streamCall()],
		[jsonReply(GONE, 503, [["retry-after", "Fri, 15 Jan 2027 08:00:03 GMT"]])],
	),
	exchange(
		"http-500-with-a-body-that-is-not-json",
		[streamCall()],
		[{ status: 500, headers: [], chunks: [repeat("x", 200_000)] }],
	),
	exchange("http-401-by-status", [streamCall()], [jsonReply({ error: { message: "No auth" } }, 401)]),
	exchange(
		"http-402-out-of-credit",
		[streamCall()],
		[jsonReply({ error: { message: "Insufficient credits" } }, 402)],
	),
	exchange("http-404-no-body", [streamCall()], [{ status: 404, headers: [], chunks: [] }]),
	exchange(
		"http-400-replay-rejected",
		[streamCall()],
		[jsonReply({ error: { message: "Reasoning must go to the provider that created it." } }, 400)],
	),
	exchange(
		"http-error-body-in-chunks-with-a-split-character",
		[streamCall()],
		[
			{
				status: 400,
				headers: JSON_HEAD,
				chunks: splitAt('{"error":{"message":"h\u{e9}llo \u{1f600}"}}', 22, 23, 30),
			},
		],
	),
	exchange(
		"http-error-body-with-a-byte-order-mark",
		[streamCall()],
		[
			{
				status: 400,
				headers: JSON_HEAD,
				chunks: [
					{ kind: "bytes", base64: Buffer.from(`\u{feff}{"error":{"message":"Bad"}}`).toString("base64") },
				],
			},
		],
	),
	exchange(
		"http-error-body-with-bad-bytes",
		[streamCall()],
		[
			{
				status: 400,
				headers: JSON_HEAD,
				chunks: [
					{
						kind: "bytes",
						base64: Buffer.from([
							0x7b, 0x22, 0x65, 0x72, 0x72, 0x6f, 0x72, 0x22, 0x3a, 0xff, 0x7d,
						]).toString("base64"),
					},
				],
			},
		],
	),
	exchange(
		"http-error-reaches-complete-and-json-calls-the-same",
		[completeCall(), completeCall()],
		[jsonReply(SLOW, 429, [["retry-after", "5"]]), jsonReply(GONE, 503)],
	),
	exchange(
		"http-error-after-a-good-reply-is-logged-with-its-status",
		[streamCall(), streamCall()],
		[sseReply([piece(frame(text("Hi"))), piece(frame(finish("stop"))), piece(DONE_TEXT)]), jsonReply(GONE, 503)],
	),
];

const retryAfter: CaseSource[] = [
	retried("seconds", "7"),
	retried("seconds-padded", " 7 "),
	retried("seconds-zero", "0"),
	retried("seconds-negative-is-zero", "-3"),
	retried("seconds-fractional", "1.5"),
	retried("seconds-below-a-millisecond-is-whole", "0.0004"),
	retried("seconds-exponent", "1e3"),
	retried("seconds-huge", "1e300"),
	retried("seconds-overflow", "1e400"),
	retried("seconds-leading-zeros", "007"),
	retried("seconds-leading-plus", "+5"),
	retried("seconds-leading-dot", ".5"),
	retried("seconds-trailing-dot", "5."),
	retried("seconds-hex", "0x10"),
	retried("seconds-binary", "0b11"),
	retried("seconds-with-a-separator", "1_000"),
	retried("seconds-with-a-comma", "1,5"),
	retried("seconds-infinity", "Infinity"),
	retried("seconds-not-a-number", "NaN"),
	retried("date-rfc-1123", "Fri, 15 Jan 2027 08:00:03 GMT"),
	retried("date-now", NOW_DATE),
	retried("date-in-the-past-is-zero", "Fri, 15 Jan 2027 07:00:00 GMT"),
	retried("date-padded", " Fri, 15 Jan 2027 08:00:03 GMT "),
	retried("date-a-day-ahead", "Sat, 16 Jan 2027 08:00:00 GMT"),
	retried("date-iso", "2027-01-15T08:00:05Z"),
	retried("date-iso-with-an-offset", "2027-01-15T09:00:05+01:00"),
	retried("date-iso-with-milliseconds", "2027-01-15T08:00:05.600Z"),
	retried("date-iso-without-a-zone", "2027-01-15T08:00:05"),
	retried("date-only", "2027-01-16"),
	retried("date-loose-month-first", "Jan 16 2027 08:00:00 GMT"),
	retried("date-rfc-850", "Friday, 15-Jan-27 08:00:05 GMT"),
	retried("date-asctime", "Fri Jan 15 08:00:05 2027"),
	retried("garbage", "soon"),
	retried("empty", ""),
	retried("blank", "   "),
	exchange(
		"retry-after-two-headers-are-joined",
		[streamCall()],
		[
			jsonReply(SLOW, 429, [
				["retry-after", "7"],
				["Retry-After", "9"],
			]),
		],
	),
	exchange("retry-after-name-in-any-case", [streamCall()], [jsonReply(SLOW, 429, [["RETRY-AFTER", "7"]])]),
	exchange("retry-after-absent", [streamCall()], [jsonReply(SLOW, 429)]),
	exchange("retry-after-on-a-non-limit-status", [streamCall()], [jsonReply(GONE, 503, [["retry-after", "7"]])]),
	exchange(
		"retry-after-on-a-failure-with-no-category-wait",
		[streamCall()],
		[jsonReply({ error: { message: "No auth" } }, 401, [["retry-after", "7"]])],
	),
	exchange("retry-after-on-an-unstreamed-reply", [completeCall()], [jsonReply(SLOW, 429, [["retry-after", "7"]])]),
	exchange(
		"retry-after-date-moves-with-the-wheel",
		[streamCall()],
		[jsonReply(SLOW, 429, [["retry-after", "Fri, 15 Jan 2027 08:00:30 GMT"]], { headMs: 10_000 })],
	),
	exchange(
		"retry-after-date-counts-from-the-failure-not-the-start",
		[streamCall()],
		[
			{
				status: 429,
				headers: [["retry-after", "Fri, 15 Jan 2027 08:00:30 GMT"]],
				chunks: [piece(JSON.stringify(SLOW), 10_000)],
			},
		],
	),
	exchange(
		"retry-after-the-codec-holds-its-own",
		[streamCall()],
		[jsonReply({ error: { message: "Slow.", resetsAt: NOW / 1000 + 90 } }, 429)],
		{ codec: opencode({ nowMs: NOW }) },
	),
	exchange(
		"retry-after-the-header-wins-when-the-codec-has-none",
		[streamCall()],
		[jsonReply({ error: { message: "Slow.", resetsAt: NOW / 1000 + 90 } }, 429, [["retry-after", "7"]])],
		{ codec: opencode({ nowMs: NOW }) },
	),
	exchange(
		"retry-after-the-codec-holds-its-own-and-the-header-is-garbage",
		[streamCall()],
		[jsonReply({ error: { message: "Slow.", resetsAt: NOW / 1000 + 90 } }, 429, [["retry-after", "soon"]])],
		{ codec: opencode({ nowMs: NOW }) },
	),
	exchange(
		"retry-after-on-a-json-call",
		[{ kind: "getJson", path: "/models" }],
		[jsonReply(SLOW, 429, [["retry-after", "7"]])],
	),
];

const caps: CaseSource[] = [
	exchange("error-body-read-to-exactly-the-cap-parses", [streamCall()], [padded(CAP - OBJECT_BYTES, "")]),
	exchange("error-body-one-byte-past-the-cap-is-cut-and-valid", [streamCall()], [padded(CAP - OBJECT_BYTES, "x")]),
	exchange("error-body-last-byte-inside-the-cap-is-read", [streamCall()], [padded(CAP - OBJECT_BYTES - 1, "x")]),
	exchange(
		"error-body-the-cap-falls-inside-a-chunk",
		[streamCall()],
		[
			{
				status: 500,
				headers: JSON_HEAD,
				chunks: [piece('{"error":{"message":"m"}}'), repeat(" ", 40_000), repeat(" ", 40_000), piece("x")],
			},
		],
	),
	exchange(
		"error-body-stops-reading-at-the-cap",
		[streamCall()],
		[
			{
				status: 500,
				headers: JSON_HEAD,
				chunks: [
					repeat("x", 20_000),
					repeat("x", 20_000),
					repeat("x", 20_000),
					repeat("x", 20_000),
					repeat("x", 20_000),
				],
			},
		],
	),
	exchange(
		"error-body-a-chunk-that-ends-on-the-cap-stops-there",
		[streamCall()],
		[
			{
				status: 500,
				headers: JSON_HEAD,
				chunks: [piece('{"error":{"message":"m"}}'), repeat(" ", CAP - OBJECT_BYTES), piece("x"), piece("x")],
			},
		],
	),
	exchange(
		"error-body-a-multi-byte-character-cut-by-the-cap",
		[streamCall()],
		[padded(CAP - OBJECT_BYTES - 1, "\u{e9}")],
	),
	exchange(
		"error-body-a-four-byte-character-cut-by-the-cap",
		[streamCall()],
		[padded(CAP - OBJECT_BYTES - 2, "\u{1f600}")],
	),
	exchange(
		"error-body-a-multi-byte-character-whole-before-the-cap",
		[streamCall()],
		[
			{
				status: 500,
				headers: JSON_HEAD,
				chunks: [piece(ACCENTED), repeat(" ", CAP - Buffer.byteLength(ACCENTED)), piece("x")],
			},
		],
	),
	exchange(
		"error-body-empty-chunks-are-skipped",
		[streamCall()],
		[
			{
				status: 500,
				headers: JSON_HEAD,
				chunks: [piece(""), piece('{"error":{"message":"m"}}'), piece("")],
			},
		],
	),
	exchange(
		"error-body-json-call-has-the-same-cap",
		[{ kind: "getJson", path: "/models" }],
		[padded(CAP - OBJECT_BYTES, "x")],
	),
	exchange(
		"ok-body-has-no-cap",
		[{ kind: "getJson", path: "/models" }],
		[
			{
				status: 200,
				headers: JSON_HEAD,
				chunks: [piece('{"items":"'), repeat("x", 100_000), piece('"}')],
			},
		],
	),
];

const traces: CaseSource[] = [
	replay("trace-tool-loop", "cases/traces/openrouter.tool-loop.json", [
		streamCall({}, { ...TRACE_REQUEST, history: [{ role: "user", text: ASK }] }),
		streamCall(
			{},
			{
				...TRACE_REQUEST,
				history: [
					{ role: "user", text: ASK },
					{
						role: "step",
						producer: { model: "openai/gpt-6-luna", host: "Azure" },
						parts: [
							{
								kind: "call",
								index: 0,
								wireId: "call_ZByuccg6RmQ9bAwU3IkyFvPl",
								name: "lookup",
								arguments: '{"key":"alpha"}',
							},
							{ kind: "result", index: 0, output: "value-of-alpha", isError: false },
							{
								kind: "call",
								index: 1,
								wireId: "call_o58XubUYSReYrz8uvhRqppLA",
								name: "lookup",
								arguments: '{"key":"beta"}',
							},
							{ kind: "result", index: 1, output: "value-of-beta", isError: false },
						],
					},
				],
			},
		),
	]),
	replay("trace-title-with-a-schema-and-usage", "cases/traces/openrouter.title.json", [
		completeCall(
			{},
			{
				...REQUEST,
				model: "openai/gpt-6-luna",
				effort: "low",
				session: "00000000-0000-4000-8000-000000000000",
				history: [
					{
						role: "user",
						text: "Name this conversation in a few words: Rename the tally helper and update its callers.",
					},
				],
				schema: {
					name: "title",
					json: {
						type: "object",
						properties: { title: { type: "string" } },
						required: ["title"],
						additionalProperties: false,
					},
				},
			},
		),
	]),
	replay("trace-routing-failure", "cases/traces/openrouter.no-zdr.json", [
		streamCall(
			{},
			{
				...REQUEST,
				model: "anthropic/claude-fable-5.1",
				session: "00000000-0000-4000-8000-000000000000",
				history: [{ role: "user", text: "Say hi." }],
			},
		),
	]),
];

export const httpCases: CaseSource[] = [...failures, ...retryAfter, ...caps, ...traces];
