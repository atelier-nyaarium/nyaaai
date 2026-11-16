import { failureOf } from "@nyaaai/agentic/failure";
import { replyCalls, replyText, replyThoughts, StepAssembler } from "@nyaaai/agentic/step";
import { parseJson } from "@nyaaai/contract/json";
import { decode, z } from "@nyaaai/contract/shape";
import { Failure, NormalizedUsage, ReplyPart, StepEvent, StepResult, ThoughtPart } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { stepCases } from "../caseSources/step";

/** What a failed attempt was billed. */
export const StepBilled = z.object({
	usage: NormalizedUsage.exactOptional(),
	host: z.string().exactOptional(),
	generationId: z.string().exactOptional(),
});

function eventOf(text: string): StepEvent {
	const held = decode(StepEvent, parseJson(text));
	if (!held.ok) throw new Error(`an event text is not an event: ${held.message}`);
	return held.value;
}

export const step = caseGroup("step", {
	shapes: { StepBilled },
	kinds: {
		/** Events join in order; the first throw stops the run. */
		assemble: kind(
			z.object({
				events: z.array(StepEvent),
				/** JSON texts of events that follow, for numbers a JSON writer cannot carry. */
				more: z.array(z.string()).exactOptional(),
			}),
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
				const events = [...input.events, ...(input.more ?? []).map(eventOf)];
				for (const [at, event] of events.entries()) {
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
