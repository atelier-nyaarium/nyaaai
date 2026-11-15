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
		kind: "text.find",
		name: "matches",
		input: {
			text: HAY,
			needles: ["ab", "\n\n", GRIN, `${GRIN}${GRIN}`, "\u{e9}", "", "zz", "b\n", HAY, `${HAY}!`],
			froms: [0, 1, 2, 3, 4, 5, 6, 9, 10, 11, 12, 16, 17, 100],
		},
	},
];
