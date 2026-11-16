import { overloadedText, unavailableText } from "@nyaaai/agentic/noticeText";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { noticeTextCases } from "../caseSources/noticeText";

export const noticeText = caseGroup("noticeText", {
	kinds: {
		/** Null: the label is unknown. */
		texts: kind(
			z.object({ rows: z.array(z.object({ model: z.string().nullable(), entry: z.string().nullable() })) }),
			z.object({ overloaded: z.array(z.string()), unavailable: z.array(z.string()) }),
			(input) => ({
				overloaded: input.rows.map((row) => overloadedText(row.model ?? undefined)),
				unavailable: input.rows.map((row) => unavailableText(row.model ?? undefined, row.entry ?? undefined)),
			}),
		),
	},
	sources: casesFor("noticeText", noticeTextCases),
});
