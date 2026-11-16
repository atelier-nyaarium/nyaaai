import { base64Encode } from "@nyaaai/contract/base64";
import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Functions & Helpers

const ENCODER = new TextEncoder();

const b64 = (bytes: ArrayLike<number>): string => base64Encode(Uint8Array.from(bytes));
const textB64 = (text: string): string => b64(ENCODER.encode(text));

/** Text chunks; a number list is raw bytes. */
const chunked = (...chunks: Array<string | number[]>): string[] =>
	chunks.map((chunk) => (typeof chunk === "string" ? textB64(chunk) : b64(chunk)));

const items = (name: string, ...chunks: Array<string | number[]>): CaseSource => ({
	kind: "sse.items",
	name,
	input: { chunks: chunked(...chunks) },
});

const again = (name: string, chunks: Array<string | number[]>, later: Array<string | number[]>): CaseSource => ({
	kind: "sse.items",
	name,
	input: { chunks: chunked(...chunks), again: chunked(...later) },
});

const splits = (name: string, bytes: ArrayLike<number> | string): CaseSource => ({
	kind: "sse.splits",
	name,
	input: { bytes: typeof bytes === "string" ? textB64(bytes) : b64(bytes) },
});

const crlf = (text: string): string => text.replaceAll("\n", "\r\n");

const bytesOfText = (text: string): number[] => [...ENCODER.encode(text)];

/** Two-, three- and four-byte characters. */
const WIDE = `{"text":"caf\u{e9} \u{20ac} \u{1f600}"}`;

/** A made-up stream in the shape of a chat completion. */
const STREAM = [
	": PROCESSING\n\n",
	'data: {"id":"gen-1","model":"vendor/model-a","choices":[{"delta":{"role":"assistant","content":""}}]}\n\n',
	'data: {"id":"gen-1","choices":[{"delta":{"content":"Hel"}}]}\n\n',
	'data: {"id":"gen-1","choices":[{"delta":{"content":"lo"}}]}\n\n',
	'data: {"id":"gen-1","choices":[{"delta":{},"finish_reason":"stop"}],"usage":{"prompt_tokens":3,"completion_tokens":2}}\n\n',
	"data: [DONE]\n\n",
	`data: ${WIDE}\n\n`,
].join("");

/** Events, comments, multi-line data and a last line with no newline. */
const MIXED =
	"event: x\nid: 1\ndata: one\ndata:two\n\nevent: lone\n\n:ping\ndata: three\n\nevent: e2\n:inner\ndata: four\n\ndata: [DONE]";

export const sseCases: CaseSource[] = [
	items("lf", 'data: {"a":1}\n\ndata: {"a":2}\n\n'),
	items("crlf", crlf('data: {"a":1}\n\ndata: {"a":2}\n\n')),
	items("one-event-per-chunk", "data: a\n\n", "data: b\n\n", "data: c\n\n"),
	items("event-across-chunks", "data: a", "\n", "\n", "data: b\n", "\n"),
	items("multi-line-data", "data: one\ndata:two\ndata:  three\ndata\n\n"),
	items("event-names", "event: x\nid: 1\nretry: 5\ndata: one\n\nevent: y\ndata: two\n\ndata: three\n\n"),
	items("event-without-data-resets", "event: lone\n\ndata: after\n\n"),
	items("event-last-line-wins", "event: a\nevent: b\ndata: x\n\n"),
	items("event-after-data", "data: 1\nevent: z\n\n"),
	items("event-empty-name", "event\ndata: x\n\nevent:\ndata: y\n\n"),
	items("comments", ":ping\n: spaced\n:\ndata: x\n\n"),
	items("comment-inside-event", "event: e\n:c\ndata: d\n\n"),
	items("comment-unterminated", ":tail"),
	items("done", "data: [DONE]\n\n"),
	items("done-then-data", "data: [DONE]\n\ndata: late\n\n"),
	items("done-without-space", "data:[DONE]\n\n"),
	items("done-is-exact", "data:  [DONE]\n\ndata: [DONE]\ndata: more\n\ndata: [DONE] \n\ndata: [done]\n\n"),
	items("done-drops-event", "event: end\ndata: [DONE]\n\n"),
	items("done-unterminated", "data: [DONE]"),
	items("unterminated-last-line", "data: a\n\ndata: tail"),
	items("unterminated-no-blank-line", "data: x\n"),
	items("unterminated-event", "event: e\ndata: x"),
	items("unterminated-carriage-return", "data: x\r"),
	items("unterminated-crlf-line", "data: y\r\n"),
	items("nothing-pending", ""),
	items("no-chunks"),
	items("empty-chunks", "", "data: a\n", "", "\n", ""),
	items("blank-lines-only", "\n\n\n\r\n"),
	items("carriage-return-split", "data: a\r", "\n\r", "\n"),
	items("lone-carriage-return", "data: a\rdata: b\n\n"),
	items("carriage-return-inside-value", "data: a\r\r\n\n"),
	items("no-colon-field", "data\n\n"),
	items("empty-data-value", "data:\n\ndata: \n\n"),
	items("colon-in-value", "data: a:b: c\n\ndata:\tx\n\n"),
	items("only-one-space-dropped", "data:   x\n\n"),
	items("field-names-are-exact", "Data: x\n\ndata : x\n\n data: x\n\nid: 3\n\nretry: 2\n\n"),
	items("bytes-order-mark-dropped", [0xef, 0xbb, 0xbf], "data: x\n\n"),
	items("bytes-order-mark-with-text", [0xef, 0xbb, 0xbf, 0x64, 0x61, 0x74, 0x61, 0x3a, 0x20, 0x78, 0x0a, 0x0a]),
	items("bytes-order-mark-after-empty-chunk", "", [0xef, 0xbb, 0xbf], "data: x\n\n"),
	items("bytes-order-mark-split", [0xef], [0xbb, 0xbf, 0x64], "ata: x\n\n"),
	items("bytes-order-mark-in-the-middle", "data: a\n\n", [0xef, 0xbb, 0xbf], "data: b\n\ndata: c\n\n"),
	items("character-split-two-bytes", "data: caf", [0xc3], [0xa9], "\n\n"),
	items("character-split-four-bytes", "data: ", [0xf0], [0x9f], [0x98], [0x80], "\n\n"),
	items("character-split-three-bytes", "data: ", [0xe2, 0x82], [0xac], "\n\n"),
	items("character-held-across-empty-chunks", "data: ", [0xe2], [], [0x82], [], [0xac], "\n\n"),
	items("invalid-byte", "data: a", [0xff], "b\n\n"),
	items("invalid-continuation", "data: ", [0xe2, 0x41], "\n\n"),
	items("truncated-character-at-end", "data: caf", [0xc3]),
	items("truncated-character-before-newline", "data: caf", [0xc3], "\n"),
	items("truncated-character-in-event-name", "event: e", [0xe2, 0x82], "\ndata: x\n\n"),
	items("surrogate-bytes", "data: ", [0xed, 0xa0, 0x80], "\n\n"),
	items("wide-data-whole", `data: ${WIDE}\n\n`),
	items("long-line-in-pieces", "data: ", ..."abcdefghij".repeat(20).split(""), "\n\n"),
	again("end-twice", ["data: a"], []),
	again("push-after-end", ["data: a\n\n"], ["data: b\n\n", "data: tail"]),
	again("push-after-end-keeps-no-state", ["event: e\ndata: a"], ["data: b\n\n"]),
	again("bytes-order-mark-after-end", [[0xef, 0xbb, 0xbf], "data: x\n\n"], [[0xef, 0xbb, 0xbf], "data: y\n\n"]),
	again("character-held-at-end", ["data: caf", [0xc3]], [[0xa9], "\n\n"]),
	items("nul-in-data", "data: a\u{0}b\n\n"),
	items("nul-in-field-and-comment", "da\u{0}ta: x\n\n:c\u{0}\ndata: y\n\n"),
	items("thousands-of-lines", `${Array.from({ length: 3000 }, (_, at) => `id: ${at}\n`).join("")}data: end\n\n`),
	items("many-events-in-one-chunk", "data: x\n\n".repeat(400)),
	items("recorded-shape", STREAM),
	items("recorded-shape-crlf", crlf(STREAM)),
	items("mixed", MIXED),
	splits("splits-recorded-shape-lf", STREAM),
	splits("splits-recorded-shape-crlf", crlf(STREAM)),
	splits("splits-mixed", MIXED),
	splits("splits-mixed-crlf", crlf(MIXED)),
	splits("splits-wide-characters", `data: ${WIDE}\n\ndata: \u{10348}\u{800}\u{7ff}\u{80}\n\n`),
	splits("splits-bytes-order-mark", [0xef, 0xbb, 0xbf, ...bytesOfText("data: x\n\ndata: y")]),
	splits("splits-invalid-bytes", [
		...bytesOfText("data: a"),
		0xe2,
		0x82,
		0x41,
		0xf0,
		0x9f,
		0xc3,
		0xa9,
		0xff,
		...bytesOfText("\n\ndata: tail"),
		0xf0,
		0x9f,
	]),
	splits("splits-carriage-returns", "data: a\r\n\r\ndata: b\rdata: c\r\n\r\n:x\r\n"),
	splits("splits-unterminated", "data: a\n\ndata: tail"),
	splits("splits-empty", ""),
	splits("splits-one-byte", "x"),
	splits("splits-one-line", "data: x\n"),
	splits("splits-done-and-comments", ":a\ndata: [DONE]\n\n:b\ndata: z\n\n"),
];
