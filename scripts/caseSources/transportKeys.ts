// Transport cases: keys, auth and scrubbing.

import { Buffer } from "node:buffer";
import type { CaseSource } from "../caseAdapter";
import { OPENCODE_BARE } from "./codecFixtures";
import {
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
	keyed,
	ODD_KEY,
	piece,
	postCall,
	sseReply,
	streamCall,
	text,
} from "./transportKit";

////////////////////////////////
//  Constants

const NEEDS_KEY = [streamCall(), completeCall(), getCall("/models"), postCall("/probe", { a: 1 })];

const HI = [piece(HI_FRAMES)];

/** The key as the three spellings the transport removes. */
const forms = (key: string): [string, string, string] => [
	key,
	encodeURIComponent(key),
	Buffer.from(key).toString("base64").replace(/=+$/, ""),
];

const ECHOED = { id: `gen-${KEY}`, provider: `Azure ${KEY}` };

////////////////////////////////
//  Cases

const keyRules: CaseSource[] = [
	exchange("key-missing-empty-and-malformed-make-no-request", [streamCall(), streamCall(), streamCall()], [], {
		keys: [null, "", "key-v1-abc\ndef"],
	}),
	exchange("key-refusals-reach-complete-and-json-calls", NEEDS_KEY, [], { keys: [null] }),
	exchange("key-malformed-reaches-complete-and-json-calls", NEEDS_KEY, [], { keys: ["sk or v1"] }),
	exchange("key-refusal-names-the-label", [streamCall(), streamCall()], [], {
		label: "Lab",
		keys: [null, "bad key"],
	}),
	exchange(
		"key-is-read-again-for-each-request",
		[streamCall(), streamCall(), streamCall()],
		[sseReply(HI), sseReply(HI), sseReply(HI)],
		{ keys: ["key-v1-first", "key-v1-second", null] },
	),
	exchange("key-refused-after-a-good-call", [streamCall(), streamCall()], [sseReply(HI)], {
		keys: [KEY, ""],
	}),
	exchange("auth-bearer-sends-the-key", [streamCall()], [sseReply(HI)], { auth: "bearer" }),
	exchange(
		"auth-none-sends-no-key-and-never-reads-the-source",
		NEEDS_KEY,
		[
			sseReply(HI),
			jsonReply({
				id: "gen-1",
				provider: "Azure",
				choices: [{ message: { content: "x" }, finish_reason: "stop" }],
			}),
			jsonReply({}),
			jsonReply({}),
		],
		{ auth: "none" },
	),
	exchange("auth-none-needs-no-key", [streamCall()], [sseReply(HI)], { auth: "none", keys: [null] }),
	exchange("auth-optional-sends-the-key-it-has", [streamCall()], [sseReply(HI)], { auth: "optional" }),
	exchange("auth-optional-without-a-key-sends-none", [streamCall()], [sseReply(HI)], {
		auth: "optional",
		keys: [null],
	}),
	exchange("auth-optional-an-empty-key-is-no-key", [streamCall()], [sseReply(HI)], { auth: "optional", keys: [""] }),
	exchange("auth-optional-a-malformed-key-is-refused", [streamCall()], [], { auth: "optional", keys: ["bad key"] }),
	exchange(
		"auth-optional-reads-the-source-once-per-call",
		[streamCall(), getCall("/models")],
		[sseReply(HI), jsonReply({})],
		{
			auth: "optional",
		},
	),
	exchange(
		"auth-optional-a-source-that-changes-is-read-once-per-call",
		[streamCall(), streamCall()],
		[sseReply(HI), sseReply(HI)],
		{ auth: "optional", keys: [KEY, null] },
	),
	exchange(
		"auth-optional-an-explicit-key-skips-the-source",
		[getCall("/models", { keyAuth: { kind: "key", key: KEY } })],
		[jsonReply({})],
		{ auth: "optional", keys: [null] },
	),
	exchange("auth-optional-public-read", [getCall("/models", { keyAuth: { kind: "none" } })], [jsonReply({})], {
		auth: "optional",
	}),
	exchange(
		"json-explicit-key-is-sent-and-the-source-is-not-read",
		[
			getCall("/models", { keyAuth: { kind: "key", key: "key-explicit-123" } }),
			postCall("/probe", { a: 1 }, { keyAuth: { kind: "key", key: "key-explicit-456" } }),
		],
		[jsonReply({}), jsonReply({})],
	),
	exchange(
		"json-explicit-key-empty-or-malformed-is-refused",
		[
			getCall("/models", { keyAuth: { kind: "key", key: "" } }),
			getCall("/models", { keyAuth: { kind: "key", key: "has space" } }),
		],
		[],
	),
	exchange(
		"json-explicit-key-under-auth-none-is-not-sent",
		[getCall("/models", { keyAuth: { kind: "key", key: KEY } })],
		[jsonReply({})],
		{ auth: "none" },
	),
	exchange(
		"json-public-read-never-reads-the-source",
		[getCall("/models", { keyAuth: { kind: "none" } })],
		[jsonReply({})],
		{
			keys: [null],
		},
	),
	exchange("local-codec-without-a-key", [streamCall()], [sseReply(HI)], {
		codec: OPENCODE_BARE,
		auth: "optional",
		keys: [null],
		label: "Lab",
		baseUrl: "http://lab.local:11434/v1",
	}),
];

const shapes: CaseSource[] = [
	keyed("shape-plain", "key-v1-abc"),
	keyed("shape-every-printable-symbol", "!\"#$%&'()*+,-./0123456789:;<=>?@AZaz[\\]^_`{|}~"),
	keyed("shape-empty", ""),
	keyed("shape-a-space", "sk or"),
	keyed("shape-a-leading-space", " key-or"),
	keyed("shape-a-trailing-space", "key-or "),
	keyed("shape-a-newline", "key-or\n"),
	keyed("shape-a-newline-inside", "key-v1-abc\ndef"),
	keyed("shape-a-carriage-return", "key-or\r"),
	keyed("shape-a-tab", "sk\tor"),
	keyed("shape-del", "key-or\u007f"),
	keyed("shape-a-control", "sk\u0001or"),
	keyed("shape-latin-1", "key-\u{e9}"),
	keyed("shape-a-no-break-space", "sk\u{a0}or"),
	keyed("shape-wide-text", "\u{30ad}\u{30fc}"),
	keyed("shape-an-emoji", "key-\u{1f600}"),
	keyed("shape-a-lone-surrogate", "key-\ud800"),
	keyed("shape-a-byte-order-mark", "\u{feff}key-or"),
	keyed("shape-a-single-character", "k"),
	keyed("normal-trims-spaces", "  key-v1-abc  "),
	keyed("normal-trims-whitespace-of-every-kind", "\u{a0}\t\n key-or \u{2003}\u{feff}"),
	keyed("normal-blank-is-no-key", "   \n"),
	keyed("normal-empty-is-no-key", ""),
	keyed("normal-a-space-inside-is-refused", " sk or "),
	keyed("normal-a-newline-inside-is-refused", "sk\nor"),
];

const scrubs: CaseSource[] = [
	exchange(
		"scrub-key-in-an-error-message-in-every-spelling",
		[streamCall()],
		[jsonReply({ error: { code: 401, message: `Bad key ${forms(KEY).join(" ")} end` } }, 401)],
	),
	exchange(
		"scrub-key-in-an-error-message-with-awkward-characters",
		[streamCall()],
		[jsonReply({ error: { code: 401, message: `Bad key ${forms(ODD_KEY).join(" ")} end` } }, 401)],
		{ keys: [ODD_KEY] },
	),
	exchange(
		"scrub-key-in-an-error-message-inside-longer-text",
		[streamCall()],
		[jsonReply({ error: { code: 401, message: `a${KEY}b${KEY}${KEY}c` } }, 401)],
	),
	exchange(
		"scrub-key-in-the-reasons-of-a-failure",
		[streamCall()],
		[
			jsonReply(
				{
					error: {
						code: 429,
						message: "Slow down",
						metadata: { reasons: [KEY, `x ${forms(KEY)[1]}`, "plain", forms(KEY)[2]] },
					},
				},
				429,
			),
		],
	),
	exchange(
		"scrub-key-in-a-429-message-and-reasons",
		[streamCall()],
		[
			jsonReply({ error: { code: 429, message: `Slow down ${KEY}`, metadata: { reasons: [KEY] } } }, 429, [
				["retry-after", "7"],
			]),
		],
	),
	exchange(
		"scrub-key-in-a-serving-host-and-generation-id-of-a-stream",
		[streamCall()],
		[sseReply([piece(frame({ ...text("Hi"), ...ECHOED })), piece(frame(finish("stop"))), piece(DONE_TEXT)])],
	),
	exchange(
		"scrub-key-in-the-serving-of-a-stream-in-every-spelling",
		[streamCall()],
		[
			sseReply([
				piece(frame({ ...text("Hi"), id: `gen-${forms(ODD_KEY)[1]}`, provider: forms(ODD_KEY)[2] })),
				piece(frame(finish("stop"))),
				piece(DONE_TEXT),
			]),
		],
		{ keys: [ODD_KEY] },
	),
	exchange(
		"scrub-key-in-a-comment-line-is-never-an-event",
		[streamCall()],
		[sseReply([piece(`: ${KEY}\n`), piece(frame(text("Hi"))), piece(frame(finish("stop"))), piece(DONE_TEXT)])],
	),
	exchange(
		"scrub-key-in-the-serving-of-a-whole-reply",
		[completeCall()],
		[jsonReply({ ...ECHOED, choices: [{ message: { content: "Hi" }, finish_reason: "stop" }] })],
	),
	exchange(
		"scrub-key-in-the-serving-of-a-whole-reply-in-every-spelling",
		[completeCall()],
		[
			jsonReply({
				id: `gen-${forms(ODD_KEY)[1]}`,
				provider: forms(ODD_KEY)[2],
				choices: [{ message: { content: "Hi" }, finish_reason: "stop" }],
			}),
		],
		{ keys: [ODD_KEY] },
	),
	exchange(
		"scrub-key-in-an-error-reported-under-ok",
		[completeCall()],
		[jsonReply({ ...ECHOED, error: { code: 502, message: `Down ${KEY}` } })],
	),
	exchange(
		"scrub-key-in-what-an-in-stream-error-billed",
		[streamCall()],
		[
			sseReply([
				piece(
					frame({
						...ECHOED,
						error: { code: 502, message: "Down" },
						usage: { prompt_tokens: 4, completion_tokens: 1 },
					}),
				),
				piece(DONE_TEXT),
			]),
		],
	),
	exchange(
		"scrub-key-in-an-in-stream-error-message",
		[streamCall()],
		[
			sseReply([
				piece(frame({ id: "gen-1", error: { code: 502, message: `Down ${KEY} ${forms(KEY)[2]}` } })),
				piece(DONE_TEXT),
			]),
		],
	),
	exchange(
		"scrub-key-in-a-network-failure-message",
		[streamCall()],
		[
			{
				status: 0,
				headers: [],
				chunks: [],
				end: "refused",
				error: `connect failed for https://x.test/?key=${KEY}`,
			},
		],
	),
	exchange(
		"scrub-key-in-the-message-of-a-dropped-body",
		[streamCall()],
		[sseReply([piece(frame(text("Hi")))], { end: "dropped", error: `reset ${forms(KEY)[1]}` })],
	),
	exchange(
		"scrub-key-in-the-message-of-a-dropped-json-body",
		[getCall("/models")],
		[{ status: 200, headers: JSON_HEAD, chunks: [], end: "dropped", error: `reset ${KEY}` }],
	),
	exchange(
		"scrub-key-in-the-failure-of-a-json-call",
		[getCall("/models")],
		[jsonReply({ error: { message: `Bad ${KEY}` } }, 401)],
	),
	exchange(
		"scrub-key-in-the-message-of-an-explicit-key-call",
		[getCall("/models", { keyAuth: { kind: "key", key: ODD_KEY } })],
		[jsonReply({ error: { message: `Bad ${forms(ODD_KEY).join(" ")}` } }, 401)],
	),
	exchange(
		"scrub-key-in-a-response-header-reaches-no-failure",
		[streamCall()],
		[
			jsonReply({ error: { code: 429, message: "Slow down" } }, 429, [
				["x-echo", KEY],
				["retry-after", KEY],
				["x-request-id", forms(KEY)[1]],
			]),
		],
	),
	exchange(
		"scrub-key-in-a-response-header-reaches-no-event",
		[streamCall()],
		[
			sseReply(HI, {
				headers: [
					["content-type", "text/event-stream"],
					["x-echo", KEY],
				],
			}),
		],
	),
	exchange(
		"scrub-without-a-key-nothing-changes",
		[streamCall()],
		[jsonReply({ error: { code: 401, message: `Bad ${KEY}` } }, 401)],
		{ auth: "none" },
	),
	exchange(
		"scrub-the-text-a-model-says-is-not-scrubbed",
		[streamCall()],
		[sseReply([piece(frame(text(`The key is ${KEY}`))), piece(frame(finish("stop"))), piece(DONE_TEXT)])],
	),
	exchange(
		"scrub-base64-at-another-alignment-is-not-caught",
		[streamCall()],
		[jsonReply({ error: { code: 401, message: `Bad ${Buffer.from(`x${KEY}`).toString("base64")}` } }, 401)],
	),
	exchange(
		"scrub-twice-the-same-key-and-different-calls",
		[streamCall(), streamCall()],
		[
			jsonReply({ error: { code: 401, message: `Bad ${KEY}` } }, 401),
			jsonReply({ error: { code: 401, message: `Bad ${KEY}` } }, 401),
		],
	),
];

export const keyCases: CaseSource[] = [...keyRules, ...shapes, ...scrubs];
