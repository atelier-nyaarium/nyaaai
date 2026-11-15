import { z } from "@nyaaai/contract/shape";
import { linesOf } from "@nyaaai/operator/claude/sessionFile";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { sessionFileCases } from "../caseSources/sessionFile";

export const sessionFile = caseGroup("sessionFile", {
	kinds: {
		linesOf: kind(z.object({ base64: z.string() }), z.object({ rows: z.array(z.string()) }), (input) => ({
			rows: [...linesOf(Buffer.from(input.base64, "base64"))],
		})),
	},
	sources: casesFor("sessionFile", sessionFileCases),
});
