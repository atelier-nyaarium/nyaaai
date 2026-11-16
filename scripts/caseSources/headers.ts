import type { CaseSource } from "../caseAdapter";

const lookup = (name: string, headers: Array<[string, string]>, wanted: string): CaseSource => ({
	kind: "headers.of",
	name,
	input: { headers, name: wanted },
});

export const headerCases: CaseSource[] = [
	lookup("other-case", [["Content-Type", "text/plain"]], "content-type"),
	lookup("same-case", [["retry-after", "5"]], "retry-after"),
	lookup(
		"first-of-two",
		[
			["X-Id", "1"],
			["x-id", "2"],
		],
		"X-ID",
	),
	lookup(
		"none",
		[
			["a", "1"],
			["b", "2"],
		],
		"c",
	),
	lookup("empty-list", [], "a"),
	lookup(
		"non-ascii-name",
		[
			["\u{c9}tag", "upper"],
			["\u{e9}tag", "lower"],
		],
		"\u{e9}tag",
	),
	lookup("non-ascii-no-fold", [["\u{e9}tag", "lower"]], "\u{c9}tag"),
	lookup(
		"empty-name",
		[
			["", "blank"],
			["a", "1"],
		],
		"",
	),
	{
		kind: "headers.response",
		name: "forms",
		input: {
			lists: [
				[],
				[["Content-Type", "text/plain"]],
				[
					["X-B", "1"],
					["x-a", "2"],
					["X-b", "3"],
					["x-B", "4"],
				],
				[
					["Set-Cookie", "a=1"],
					["Vary", "Accept"],
					["set-cookie", "b=2"],
					["VARY", "Origin"],
					["SET-COOKIE", "c=3"],
				],
				[
					["b", ""],
					["a", ""],
					["b", "x"],
				],
				[
					["\u{e9}tag", "1"],
					["\u{c9}tag", "2"],
					["\u{10000}", "3"],
					["\u{ffff}", "4"],
					["Z", "5"],
				],
			],
		},
	},
];
