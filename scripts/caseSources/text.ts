// UTF-16 text cases.

import { base64Encode } from "@nyaaai/contract/base64";
import type { CaseSource } from "../caseAdapter";

const GRIN = "\u{1F600}";
/** Four UTF-16 pairs. */
const FAMILY = "\u{1F1EF}\u{1F1F5}\u{1F44D}\u{1F3FD}";

const TEXTS = [
	"",
	"plain ASCII",
	`a${GRIN}b`,
	`${GRIN}${GRIN}`,
	`${FAMILY}!`,
	"e\u{301}\u{e9}\u{df}\u{130}\u{17f}\u{212a}",
	"\u{ffff}\u{10000}\u{e000}",
	"line\none\r\n\u{2028}two",
	"MiXeD Case \u{c9}\u{e9} \u{3a3}\u{3c3}\u{3c2} \u{ff21}\u{ff41}",
];

const HAY = `ab${GRIN}ab\n\n${GRIN}${GRIN}ab\u{e9}\n\nab`;

/** NFC quick-check edges. */
const NFC_QUICK_TEXTS = [
	"\u{65b0}\u{8a2d}\u{5b9a}\u{30d5}\u{30a1}\u{30a4}\u{30eb}",
	"abc e\u{301}d",
	"x\u{212b}y",
	"a\u{323}\u{301}",
	"a\u{301}\u{323}",
	"\u{1100}\u{1161}",
	"\u{1d11e}\u{20bb7}",
	"caf\u{e9} \u{d800} ok",
];

/** NFC normalization texts. */
const NFC_TEXTS = [
	"",
	"plain ASCII",
	"The quick brown fox 0123456789",
	"\u{e9}",
	"\u{e7}\u{e0}\u{fc}\u{f6}\u{f1}",
	"e\u{301}",
	"c\u{327}",
	"A\u{30A}",
	"n\u{303}",
	"\u{e9}e\u{301}",
	"a\u{301}\u{323}",
	"a\u{323}\u{301}",
	"x\u{308}\u{301}",
	"a\u{301}\u{301}",
	"a\u{301}b\u{301}",
	"\u{1100}\u{1161}",
	"\u{1100}\u{1161}\u{11A8}",
	"\u{AC00}",
	"\u{AC01}",
	"\u{AC01}\u{11A8}",
	"\u{D7A3}",
	"\u{1112}\u{1161}\u{11C2}",
	"\u{0958}",
	"\u{0915}\u{093C}",
	"\u{FB1D}",
	"\u{05D9}\u{05B4}",
	"\u{2ADC}",
	"\u{2ADD}\u{0338}",
	"\u{037E}",
	"\u{1FEF}",
	"\u{212A}",
	"\u{2126}",
	"\u{212B}",
	`a${"\u{301}".repeat(31)}`,
	"\u{1D15E}",
	"\u{2F800}",
	"\u{0378}",
	"e\u{301}c\u{327}A\u{30A}\u{212B}",
	"\u{03D2}\u{0301}",
	"o\u{308}\u{304}",
	"\u{0F73}",
	"\u{1E0B}\u{0323}",
	`a${"\u{301}\u{316}".repeat(5000)}`,
];

const asBase64 = (bytes: number[]): string => base64Encode(new Uint8Array(bytes));

/** One text's bytes, in the given chunks. */
const run = (...chunks: number[][]) => ({ chunks: chunks.map(asBase64) });

/** A second pass on the same decoder. */
const reused = (first: number[][], again: number[][]) => ({
	chunks: first.map(asBase64),
	again: again.map(asBase64),
});

/** Two-, three- and four-byte characters between ASCII. */
const MIXED = [0x61, 0xc3, 0xa9, 0xe2, 0x82, 0xac, 0xf0, 0x9f, 0x98, 0x80, 0x7a, 0xdf, 0xbf, 0xef, 0xbf, 0xbd];

/** Every two-way split, and the bytes one at a time. */
export const UTF8_SPLIT_RUNS = [
	...Array.from({ length: MIXED.length + 1 }, (_, at) => run(MIXED.slice(0, at), MIXED.slice(at))),
	run(...MIXED.map((byte) => [byte])),
];

/** Each one whole, then one byte at a time. */
const whole = (...bytes: number[]) => [run(bytes), run(...bytes.map((byte) => [byte]))];

export const UTF8_MALFORMED_RUNS = [
	// Overlong forms.
	...whole(0xc0, 0x80),
	...whole(0xc1, 0xbf),
	...whole(0xe0, 0x80, 0x80),
	...whole(0xe0, 0x9f, 0xbf),
	...whole(0xf0, 0x80, 0x80, 0x80),
	...whole(0xf0, 0x8f, 0xbf, 0xbf),
	// Surrogates, and past U+10FFFF.
	...whole(0xed, 0xa0, 0x80),
	...whole(0xed, 0xbf, 0xbf),
	...whole(0xf4, 0x90, 0x80, 0x80),
	...whole(0xf5, 0x80, 0x80, 0x80),
	// Stray bytes.
	...whole(0x80),
	...whole(0xbf, 0x41),
	...whole(0xfe, 0xff, 0x42),
	...whole(0xe2, 0x41),
	...whole(0xe2, 0x82, 0x41),
	...whole(0xf0, 0x9f, 0x98, 0x41),
	...whole(0xf0, 0x9f, 0xc3, 0xa9),
	// Truncated tails.
	...whole(0x41, 0xe2, 0x82),
	...whole(0xf0, 0x9f, 0x98),
	...whole(0xc3),
	run([0xe2], [0x82]),
	run([0xf0, 0x9f], [], [0x98]),
];

export const UTF8_BOM_RUNS = [
	...whole(0xef, 0xbb, 0xbf, 0x41),
	...whole(0x41, 0xef, 0xbb, 0xbf, 0x42),
	...whole(0xef, 0xbb, 0xbf, 0xef, 0xbb, 0xbf, 0x41),
	...whole(0xff, 0xef, 0xbb, 0xbf, 0x41),
	...whole(0xef, 0xbb),
	run([0xef], [0xbb, 0xbf, 0x41]),
	run([0xef, 0xbb, 0xbf]),
	run([], [0xef, 0xbb, 0xbf], [], [0x41]),
];

export const UTF8_REUSED_RUNS = [
	reused([[0xef, 0xbb, 0xbf, 0x41]], [[0xef, 0xbb, 0xbf, 0x42]]),
	reused([[0x41, 0xe2, 0x82]], [[0xac, 0x42]]),
	reused([[0xef], [0xbb]], [[0xbf, 0x43]]),
	reused([[0xf0, 0x9f]], [[0x98, 0x80], [0xe2]]),
];

export const UTF8_EMPTY_RUNS = [run(), run([]), run([], [], []), run([], [0x41], [], [0xc3], [], [0xa9], [])];

/** Text for path-segment encoding. */
export const SEGMENT_TEXTS = [
	"",
	"plain-Text_09.~",
	"!*'()",
	" ",
	"a b",
	"/?#[]@:;=&+$,%",
	'"<>^`{|}',
	"\u{e9}\u{df}\u{20ac}",
	"\u{3042}\u{6f22}",
	GRIN,
	`a${GRIN}/b`,
	"\u{7f}\u{80}\u{7ff}\u{800}\u{ffff}",
	"\u{fffd}",
	"\r\n\t\u{0}",
];

/** Each boundary of C0, DEL and C1, and text near them. */
const CONTROL_TEXTS = [
	"",
	"plain",
	"\u{0}",
	"a\u{0}b",
	"\u{1}",
	"\u{9}",
	"\u{a}",
	"\u{1f}",
	"\u{20}",
	"\u{21}",
	"\u{7e}",
	"\u{7f}",
	"x\u{7f}",
	"\u{80}",
	"\u{85}",
	"\u{9f}",
	"\u{a0}",
	"\u{a1}",
	"\u{ad}",
	"\u{2028}",
	"\u{200b}",
	"\u{feff}",
	"\u{ffff}",
	"\u{3042}\u{6f22}",
	GRIN,
	`a${GRIN}b`,
	`${GRIN}\u{1f}`,
	"\u{10000}\u{10ffff}",
	"\u{D800}",
	"\u{DC00}",
	"a\u{D800}b\u{80}",
];

export const textCases: CaseSource[] = [
	{ kind: "text.cuts", name: "pairs", input: { texts: TEXTS, points: [0, 1, 2, 3, 4, 5, 7, 8, 9, 100] } },
	{
		kind: "text.slices",
		name: "spans",
		input: {
			text: `0${GRIN}3${GRIN}6${FAMILY}`,
			spans: [
				{ start: 0, end: 0 },
				{ start: 0, end: 1 },
				{ start: 0, end: 2 },
				{ start: 1, end: 2 },
				{ start: 2, end: 3 },
				{ start: 2, end: 2 },
				{ start: 2, end: 5 },
				{ start: 1, end: 3 },
				{ start: 3, end: 1 },
				{ start: 5, end: 100 },
				{ start: 100, end: 200 },
				{ start: 8, end: 12 },
				{ start: 9, end: 11 },
			],
		},
	},
	{
		kind: "text.spaces",
		name: "ecmascript-whitespace",
		input: {
			texts: [
				"",
				"   ",
				"a  b",
				" a\tb\nc\r\nd ",
				"\u{FEFF}\u{A0} one\u{2028}\u{3000}two \t\n",
				"\u{B}\u{C}x\u{B}\u{C}y",
				"\u{1680}a\u{2000}\u{2005}\u{200A}b\u{202F}\u{205F}",
				"\u{2028}\u{2029}",
				// Not ECMAScript whitespace.
				"\u{85}x\u{85}",
				"\u{180E}x \u{180E}",
				"\u{2060}x \u{2060} y\u{2060}",
				"\u{1C}\u{1F}x",
				`${GRIN} x  ${GRIN}`,
			],
		},
	},
	{
		kind: "text.find",
		name: "matches",
		input: {
			text: HAY,
			needles: ["ab", "\n\n", GRIN, `${GRIN}${GRIN}`, "\u{e9}", "", "zz", "b\n", HAY, `${HAY}!`],
			froms: [0, 1, 2, 3, 4, 5, 6, 9, 10, 11, 12, 16, 17, 100],
		},
	},
	{ kind: "text.nfc", name: "forms", input: { texts: NFC_TEXTS } },
	{ kind: "text.nfc", name: "quick-check-edges", input: { texts: NFC_QUICK_TEXTS } },
	{ kind: "text.utf8Chunks", name: "splits", input: { runs: UTF8_SPLIT_RUNS } },
	{ kind: "text.utf8Chunks", name: "malformed", input: { runs: UTF8_MALFORMED_RUNS } },
	{ kind: "text.utf8Chunks", name: "byte-order-mark", input: { runs: UTF8_BOM_RUNS } },
	{ kind: "text.utf8Chunks", name: "reused", input: { runs: UTF8_REUSED_RUNS } },
	{ kind: "text.utf8Chunks", name: "empty-chunks", input: { runs: UTF8_EMPTY_RUNS } },
	{ kind: "text.hasControl", name: "ranges", input: { texts: CONTROL_TEXTS } },
	{ kind: "text.pathSegment", name: "encoded", input: { texts: SEGMENT_TEXTS } },
];
