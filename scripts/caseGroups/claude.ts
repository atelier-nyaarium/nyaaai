import { z } from "@nyaaai/contract/shape";
import { versionIn } from "@nyaaai/operator/claude/cliVersion";
import { caseGroup, kind } from "@nyaaai/testkit/caseGroup";

export const claude = caseGroup("claude", {
	kinds: {
		versionIn: kind(
			z.object({ texts: z.array(z.string()) }),
			z.object({ found: z.array(z.string().nullable()) }),
			(input) => ({ found: input.texts.map((text) => versionIn(text) ?? null) }),
		),
	},
	sources: [
		{
			kind: "claude.versionIn",
			name: "answers",
			input: {
				texts: [
					"2.1.323 (Claude Code)",
					"v10.20.3-beta",
					"1.2 then 3.4.5",
					"no version 1.",
					"",
					"1.2.3.4",
					"1..2.3 4.5.6",
					"\u{663}.\u{664}.\u{665} 7.8.9",
					"1.2.x 3.4.5",
					"007.08.9",
					"1.2.\n3 4.5.6",
				],
			},
		},
	],
});
