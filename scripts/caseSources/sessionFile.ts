// Session byte cases.

import type { CaseSource } from "../caseAdapter";

const LF = 0x0a;
const A = 0x41;

const bytesCase = (name: string, bytes: number[] | string): CaseSource => ({
	kind: "sessionFile.linesOf",
	name,
	input: {
		base64: Buffer.from(typeof bytes === "string" ? new TextEncoder().encode(bytes) : bytes).toString("base64"),
	},
});

/** Invalid UTF-8 rows. */
const BAD_ROWS: Record<string, number[][]> = {
	loneContinuation: [[0x80, 0xbf, A]],
	neverALead: [[0xff, 0xfe, 0xf8, 0xc0, 0xc1, 0xf5, A]],
	truncatedAtRowEnd: [[0xc3], [0xe2, 0x82], [0xf0, 0x9f, 0x98]],
	truncatedBeforeAscii: [[0xe2, 0x82, A, 0xf0, 0x9f, A, 0xf0, 0x9f, 0x98, A]],
	overlong: [[0xc0, 0x80, 0xe0, 0x80, 0x80, 0xf0, 0x80, 0x80, 0x80, 0xe0, 0x9f, 0xbf]],
	surrogates: [[0xed, 0xa0, 0x80, 0xed, 0xbf, 0xbf, 0xed, 0x9f, 0xbf]],
	pastTheLastCodePoint: [[0xf4, 0x90, 0x80, 0x80, 0xf4, 0x8f, 0xbf, 0xbf]],
	leadThenLead: [[0xe2, 0xe2, 0x82, 0xac, 0xf0, 0xc3, 0xa9]],
};

export const sessionFileCases: CaseSource[] = [
	bytesCase("rows", "one\ntwo\r\n\n\rthree \u{1F600}\u{2028}\u{85}\u{b}\u{c} still three\nunterminated"),
	bytesCase("empty", ""),
	bytesCase("only-line-ends", "\n\n\n"),
	bytesCase("no-line-end", "never ended"),
	bytesCase("byte-order-mark", [0xef, 0xbb, 0xbf, A, LF, 0xef, 0xbb, 0xbf, A, LF]),
	bytesCase(
		"bad-bytes",
		Object.values(BAD_ROWS).flatMap((rows) => rows.flatMap((row) => [...row, LF])),
	),
];
