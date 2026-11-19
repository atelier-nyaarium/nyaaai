// Transport cases: header checks, the reply cap, early stops and more spellings of a key.

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
	KEY,
	piece,
	postCall,
	repeat,
	replay,
	sseReply,
	streamCall,
	text,
} from "./transportKit";

////////////////////////////////
//  Constants

const HI = [piece(HI_FRAMES)];

const REPLY = JSON.stringify({
	id: "gen-1",
	provider: "Azure",
	choices: [{ message: { content: "Hi" }, finish_reason: "stop" }],
});

const ANSWER = '{"a":"xxxxxxxxxx"}';

const b64 = (body: string): string => Buffer.from(body).toString("base64").replace(/=+$/, "");

const echoed = (name: string, message: string, over: Record<string, unknown> = {}): CaseSource =>
	exchange(name, [streamCall()], [jsonReply({ error: { code: 401, message } }, 401)], over);

////////////////////////////////
//  Cases

const spellings: CaseSource[] = [
	echoed("scrub-base64-at-another-alignment-is-caught", `Bad ${b64(`x${KEY}`)}`),
	echoed("scrub-base64-of-a-bearer-echo", `Dump ${b64(`Bearer ${KEY}`)}`),
	echoed("scrub-base64-of-a-bearer-echo-with-a-tail", `Dump ${b64(`Bearer ${KEY}\r\n`)} end`),
	echoed("scrub-base64-of-user-and-key", `Dump ${b64(`user:${KEY}:x`)} end`),
	echoed("scrub-lower-case-hex-echo", `Bad ${encodeURIComponent("fake/cd+ef=gh").toLowerCase()} end`, {
		keys: ["fake/cd+ef=gh"],
	}),
	echoed("scrub-mixed-case-hex-echo", "Bad fake%2fcd%2Bef%3dgh end", { keys: ["fake/cd+ef=gh"] }),
	echoed("scrub-one-character-key-does-not-reread-the-marker", "key is e here", { keys: ["k"] }),
	exchange(
		"scrub-spellings-in-reasons-and-serving",
		[streamCall(), streamCall()],
		[
			jsonReply(
				{
					error: { code: 429, message: "Slow", metadata: { reasons: [b64(`Bearer ${KEY}`), "x%2f"] } },
				},
				429,
			),
			sseReply([
				piece(frame({ ...text("Hi"), id: `gen-${b64(`id:${KEY}`)}`, provider: "Azure" })),
				piece(frame(finish("stop"))),
				piece(DONE_TEXT),
			]),
		],
	),
];

/** A made-up header value is bad in one place; a request that would carry it is refused first. */
const refusedHeader = (name: string, over: Record<string, unknown>, calls = [streamCall()]): CaseSource =>
	exchange(name, calls, [sseReply(HI)], over);

const headers: CaseSource[] = [
	refusedHeader("header-cr-lf-in-a-session-value", { codecHeaders: { "X-Session": "s\r\nX-Evil: 1" } }),
	refusedHeader("header-lf-alone-in-a-value", { codecHeaders: { "X-Session": "s\nt" } }),
	refusedHeader("header-nul-in-a-value", { codecHeaders: { "X-Session": "s\u{0}t" } }),
	refusedHeader("header-non-ascii-value", { codecHeaders: { "X-Session": "caf\u{e9}" } }),
	refusedHeader("header-delete-in-a-value", { codecHeaders: { "X-Session": "s\u{7f}t" } }),
	refusedHeader("header-name-with-a-space", { codecHeaders: { "X Session": "s" } }),
	refusedHeader("header-name-with-a-colon", { codecHeaders: { "X:Session": "s" } }),
	refusedHeader("header-name-with-a-line-break", { codecHeaders: { "X-Session\r\nX-Evil": "s" } }),
	refusedHeader("header-empty-name", { codecHeaders: { "": "s" } }),
	refusedHeader("header-cr-lf-in-the-user-agent", { userAgent: "lab-app/1.0\r\nX-Evil: 1" }),
	refusedHeader("header-non-ascii-user-agent", { userAgent: "lab-app/1.0 \u{e9}" }),
	refusedHeader("header-nul-in-the-user-agent", { userAgent: "lab\u{0}app" }),
	refusedHeader("header-bad-value-in-a-complete-call", { codecHeaders: { "X-Session": "s\r\nt" } }, [completeCall()]),
	refusedHeader("header-bad-user-agent-in-a-json-call", { userAgent: "lab\r\napp" }, [getCall("/models")]),
	refusedHeader("header-bad-user-agent-in-a-post-json-call", { userAgent: "lab\r\napp" }, [postCall("/probe", {})]),
	refusedHeader("header-is-refused-before-the-key-is-read", {
		codecHeaders: { "X-Session": "s\r\nt" },
		keys: [null],
	}),
	exchange("header-good-headers-pass-every-call", [streamCall(), streamCall()], [sseReply(HI), sseReply(HI)], {
		codecHeaders: { "X-Session": "s" },
		userAgent: "lab-app/1.0",
	}),
	refusedHeader("header-tab-and-space-are-allowed-in-a-value", { codecHeaders: { "X-Session": "a b\tc" } }),
	refusedHeader("header-empty-value-is-allowed", { codecHeaders: { "X-Session": "" } }),
	refusedHeader("header-every-token-symbol-is-allowed-in-a-name", { codecHeaders: { "!#$%&'*+-.^_`|~Az09": "s" } }),
	refusedHeader("header-owned-name-with-a-bad-value-is-dropped-unchecked", {
		codecHeaders: { accept: "bad\r\nvalue" },
	}),
];

const caps: CaseSource[] = [
	exchange("cap-a-whole-reply-at-the-cap", [completeCall()], [jsonReply(REPLY)], { maxBytes: REPLY.length }),
	exchange("cap-a-whole-reply-one-byte-past-the-cap", [completeCall()], [jsonReply(REPLY)], {
		maxBytes: REPLY.length - 1,
	}),
	exchange(
		"cap-a-whole-reply-in-chunks-past-the-cap",
		[completeCall()],
		[
			{
				status: 200,
				headers: JSON_HEAD,
				chunks: [piece(REPLY.slice(0, 30)), piece(REPLY.slice(30, 60)), piece(REPLY.slice(60))],
			},
		],
		{ maxBytes: 70 },
	),
	exchange("cap-a-whole-reply-counts-bytes-not-characters", [completeCall()], [jsonReply('"\u{e9}\u{e9}\u{e9}"')], {
		maxBytes: 6,
	}),
	exchange("cap-a-json-answer-at-the-cap", [getCall("/models")], [jsonReply(ANSWER)], { maxBytes: ANSWER.length }),
	exchange("cap-a-json-answer-one-byte-past-the-cap", [getCall("/models")], [jsonReply(ANSWER)], {
		maxBytes: ANSWER.length - 1,
	}),
	exchange("cap-a-post-json-answer-past-the-cap", [postCall("/probe", {})], [jsonReply(ANSWER)], { maxBytes: 5 }),
	exchange(
		"cap-an-error-body-keeps-its-own-cap",
		[streamCall()],
		[jsonReply({ error: { code: 400, message: "Bad request body" } }, 400)],
		{ maxBytes: 10 },
	),
	exchange(
		"cap-a-stream-with-an-endless-line",
		[streamCall()],
		[sseReply([repeat("x", 40), repeat("x", 40), repeat("x", 40), repeat("x", 40)])],
		{ maxBytes: 100 },
	),
	exchange(
		"cap-a-stream-line-at-the-cap-then-its-end",
		[streamCall()],
		[sseReply([piece(`data: ${"x".repeat(44)}`), piece("x".repeat(50)), piece("\n\n")])],
		{ maxBytes: 100 },
	),
	exchange(
		"cap-a-stream-line-one-byte-past-the-cap",
		[streamCall()],
		[sseReply([piece(`data: ${"x".repeat(44)}`), piece("x".repeat(51)), piece("\n\n")])],
		{ maxBytes: 100 },
	),
	exchange(
		"cap-a-stream-with-an-endless-event",
		[streamCall()],
		[sseReply(Array.from({ length: 8 }, () => piece("data: aaaaaaaaaaaaaa\n")))],
		{ maxBytes: 100 },
	),
	exchange(
		"cap-a-stream-with-a-comment-after-each-data-line",
		[streamCall()],
		[sseReply(Array.from({ length: 8 }, () => piece("data: aaaaaaaaaaaaaa\n:k\n")))],
		{ maxBytes: 100 },
	),
	exchange(
		"cap-a-stream-with-many-small-items-is-not-capped",
		[streamCall()],
		[sseReply([...Array.from({ length: 30 }, () => piece(commentFrame("ping"))), piece(HI_FRAMES)])],
		{ maxBytes: 30 },
	),
	exchange(
		"cap-a-stream-counts-from-the-last-chunk-that-finished-an-item",
		[streamCall()],
		[sseReply([piece(frame(text("Hi"))), repeat("y", 20), repeat("y", 20), piece("\n\n"), piece(DONE_TEXT)])],
		{ maxBytes: 30 },
	),
	exchange(
		"cap-the-next-call-after-a-capped-stream-is-clean",
		[streamCall(), streamCall()],
		[sseReply([repeat("x", 60), repeat("x", 60)]), sseReply(HI)],
		{ maxBytes: 100 },
	),
];

const stops: CaseSource[] = [
	exchange("stop-after-the-first-event", [streamCall({ stopAfter: 1 })], [sseReply([piece(HI_FRAMES)])]),
	exchange("stop-after-the-finish", [streamCall({ stopAfter: 4 })], [sseReply([piece(HI_FRAMES)])]),
	exchange(
		"stop-after-more-events-than-the-stream-has",
		[streamCall({ stopAfter: 40 })],
		[sseReply([piece(HI_FRAMES)])],
	),
	exchange(
		"stop-then-the-next-call-is-clean",
		[streamCall({ stopAfter: 1 }), streamCall()],
		[sseReply([piece(HI_FRAMES)]), sseReply([piece(HI_FRAMES)])],
	),
	exchange(
		"stop-leaves-the-idle-timer-cleared-on-a-slow-stream",
		[streamCall({ stopAfter: 1 })],
		[sseReply([piece(frame(text("Hi")), 5), piece(frame(text("more")), 20), piece(DONE_TEXT)])],
		{ idleMs: 30 },
	),
	exchange(
		"stop-with-an-abort-time-that-never-comes",
		[streamCall({ stopAfter: 1, abortAfterMs: 1000 })],
		[sseReply([piece(HI_FRAMES)])],
	),
];

const traces: CaseSource[] = [
	replay("trace-cut-by-an-abort-time", "cases/traces/openrouter.no-zdr.json", [
		streamCall(
			{ abortAfterMs: 0 },
			{
				...{ model: "anthropic/claude-fable-5.1", system: [], cache: { kind: "none" }, zdrOnly: true },
				session: "00000000-0000-4000-8000-000000000000",
				history: [{ role: "user", text: "Say hi." }],
			},
		),
	]),
];

export const fixCases: CaseSource[] = [...spellings, ...headers, ...caps, ...stops, ...traces];
