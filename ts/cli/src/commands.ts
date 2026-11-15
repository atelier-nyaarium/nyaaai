// Adapter commands.

import {
	ApplyCommand,
	CutOutcome,
	PlanCommand,
	PlanOutcome,
	type SummarizeRequest,
} from "@nyaaai/contract/shapes/sessionCut";
import { cutSession, indexSession, planCut } from "@nyaaai/operator/claude/sessionCut";
import { fileCutPorts, sessionLines } from "@nyaaai/operator/claude/sessionFile";
import { type Command, command, HostRequestFailed } from "./adapter";

////////////////////////////////
//  Constants

/** Host summary request. */
export const SUMMARIZE = "sessionCut.summarize";

export const COMMANDS: ReadonlyMap<string, Command> = new Map([
	[
		"sessionCut.plan",
		command(PlanCommand, PlanOutcome, async (input) =>
			planCut(await indexSession(sessionLines(input.source)), input.request, input.cliVersion),
		),
	],
	[
		"sessionCut.apply",
		command(ApplyCommand, CutOutcome, (input, context) => {
			const ports = fileCutPorts({
				source: input.source,
				out: input.out,
				cliVersion: input.cliVersion,
				...(input.keepOutOf === undefined ? {} : { keepOutOf: input.keepOutOf }),
				summarize: async (given: SummarizeRequest) => {
					const summary = await context.request(SUMMARIZE, given);
					if (typeof summary !== "string") throw new HostRequestFailed("the summary is not text");
					return summary;
				},
			});
			return cutSession({ ...ports, ids: context.ids, clock: context.clock }, input.request);
		}),
	],
]);
