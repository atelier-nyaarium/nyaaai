import { failureOf } from "@nyaaai/agentic/failure";
import { replyCalls, replyText, replyThoughts, StepAssembler } from "@nyaaai/agentic/step";
import { z } from "@nyaaai/contract/shape";
import { Failure, NormalizedUsage, ReplyPart, StepEvent, StepResult, ThoughtPart } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { stepCases } from "../caseSources/step";

/** What a failed attempt was billed. */
export const StepBilled = z.object({
	usage: NormalizedUsage.exactOptional(),
	host: z.string().exactOptional(),
	generationId: z.string().exactOptional(),
});

export const step = caseGroup("step", {
	shapes: { StepBilled },
	kinds: {
		/** Events join in order; the first throw stops the run. */
		assemble: kind(
			z.object({ events: z.array(StepEvent) }),
			z.object({
				/** What `add` answered, per event it took. */
				added: z.array(z.boolean()),
				/** The event whose `add` threw. */
				failedAt: z.int().nullable(),
				failure: Failure.nullable(),
				outputSeen: z.boolean(),
				/** Null until an event arrived. */
				billed: StepBilled.nullable(),
				snapshot: z.array(ReplyPart),
				/** Null when `add` threw. */
				result: StepResult.nullable(),
			}),
			(input) => {
				const assembler = new StepAssembler();
				const added: boolean[] = [];
				let failedAt: number | null = null;
				let failure: Failure | null = null;
				for (const [at, event] of input.events.entries()) {
					try {
						added.push(assembler.add(event));
					} catch (error) {
						failure = failureOf(error) ?? null;
						if (failure === null) throw error;
						failedAt = at;
						break;
					}
				}
				return {
					added,
					failedAt,
					failure,
					outputSeen: assembler.outputSeen,
					billed: assembler.billed ?? null,
					snapshot: assembler.snapshot,
					result: failure === null ? assembler.result() : null,
				};
			},
		),
		/** The three readers of a result. */
		replies: kind(
			z.object({ result: StepResult }),
			z.object({ text: z.string(), calls: z.array(ReplyPart), thoughts: z.array(ThoughtPart) }),
			(input) => ({
				text: replyText(input.result),
				calls: replyCalls(input.result),
				thoughts: replyThoughts(input.result),
			}),
		),
	},
	sources: casesFor("step", stepCases),
});
