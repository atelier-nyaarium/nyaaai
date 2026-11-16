// UTF-16 text cases.

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
];
