import { failureOf } from "@nyaaai/agentic/failure";
import { StepAssembler } from "@nyaaai/agentic/step";
import { z } from "@nyaaai/contract/shape";
import {
	CanonicalRequest,
	Encoded,
	Failure,
	HeaderList,
	SseItem,
	StepEvent,
	StepResult,
} from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { codecCases } from "../caseSources/codec";
import { CodecName, codecOf, LocalName } from "./codecOf";

////////////////////////////////
//  Functions & Helpers

async function* each(items: readonly SseItem[]): AsyncGenerator<SseItem> {
	yield* items;
}

/** A refusal is a decoded failure; anything else is a bug. */
function failureOfThrown(error: unknown): Failure {
	const held = failureOf(error);
	if (held === undefined) throw error;
	return held;
}

export const codec = caseGroup("codec", {
	shapes: { LocalName, CodecName },
	kinds: {
		/** The request as sent, or what the encoder refused. */
		encode: kind(
			z.object({ codec: CodecName, request: CanonicalRequest, mode: z.enum(["stream", "complete"]) }),
			z.object({ encoded: Encoded.nullable(), refusal: z.string().nullable() }),
			(input) => {
				const built = codecOf(input.codec);
				try {
					return { encoded: built.encode(input.request, input.mode), refusal: null };
				} catch (error) {
					if (!(error instanceof Error) || Object.getPrototypeOf(error) !== Error.prototype) throw error;
					return { encoded: null, refusal: error.message };
				}
			},
		),
		/** Events up to the end or failure, and the assembled step. */
		stream: kind(
			z.object({ codec: CodecName, items: z.array(SseItem) }),
			z.object({ events: z.array(StepEvent), result: StepResult.nullable(), failure: Failure.nullable() }),
			async (input) => {
				const assembler = new StepAssembler();
				const events: StepEvent[] = [];
				try {
					for await (const event of codecOf(input.codec).stream(each(input.items))) {
						events.push(event);
						assembler.add(event);
					}
					return { events, result: assembler.result(), failure: null };
				} catch (error) {
					return { events, result: null, failure: failureOfThrown(error) };
				}
			},
		),
		/** A reply that was not streamed. */
		complete: kind(
			z.object({ codec: CodecName, body: z.unknown() }),
			z.object({ result: StepResult.nullable(), failure: Failure.nullable() }),
			(input) => {
				try {
					return { result: codecOf(input.codec).complete(input.body), failure: null };
				} catch (error) {
					return { result: null, failure: failureOfThrown(error) };
				}
			},
		),
		/** An HTTP failure by status, body and headers. */
		failure: kind(
			z.object({ codec: CodecName, status: z.int(), body: z.unknown(), headers: HeaderList }),
			z.object({ failure: Failure }),
			(input) => ({ failure: codecOf(input.codec).failure(input.status, input.body, input.headers) }),
		),
	},
	sources: casesFor("codec", codecCases),
});
