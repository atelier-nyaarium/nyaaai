// Notice sentence cases.

import type { CaseSource } from "../caseAdapter";

export const noticeTextCases: CaseSource[] = [
	{
		kind: "noticeText.texts",
		name: "labels",
		input: {
			rows: [
				{ model: null, entry: null },
				{ model: "Model A", entry: null },
				{ model: null, entry: "Host B" },
				{ model: "Model A", entry: "Host B" },
				{ model: "", entry: "" },
				{ model: "Mod\u{e8}le \u{6587}", entry: "H\u{f4}te" },
			],
		},
	},
];
