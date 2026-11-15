import { compatibleVersions, z } from "@nyaaai/contract/shape";
import { caseGroup, kind } from "@nyaaai/testkit/caseGroup";

const PAIRS = [
	["1.2.3", "1.9.0"],
	["1.2.3", "2.0.0"],
	["0.1.0", "0.1.9"],
	["0.1.0", "0.2.0"],
	["0.0.1", "0.0.2"],
	["01.2.3", "1.02.003"],
	["000.1.0", "0.01.5"],
	["10.0.0", "1.0.0"],
	["1.2", "1.2.0"],
	["1.2.3.4", "1.2.3"],
	["1.2.x", "1.2.0"],
	["v1.2.3", "1.2.3"],
	["1.2.3 ", "1.2.3"],
	["1..3", "1.0.3"],
	["", ""],
	["latest", "latest"],
	["\u{661}.2.3", "1.2.3"],
	["1.2.3-beta", "1.2.3"],
	["99999999999999999999999.0.0", "99999999999999999999999.5.1"],
];

export const shape = caseGroup("shape", {
	kinds: {
		compatibleVersions: kind(
			z.object({ pairs: z.array(z.object({ left: z.string(), right: z.string() })) }),
			z.object({ compatible: z.array(z.boolean()) }),
			(input) => ({ compatible: input.pairs.map((pair) => compatibleVersions(pair.left, pair.right)) }),
		),
	},
	sources: [
		{
			kind: "shape.compatibleVersions",
			name: "pairs",
			input: { pairs: PAIRS.map(([left, right]) => ({ left, right })) },
		},
	],
});
