// The codec interface.

import type {
	CanonicalRequest,
	Encoded,
	Failure,
	HeaderList,
	SseItem,
	StepEvent,
	StepResult,
	Turn,
} from "@nyaaai/contract/shapes/step";

////////////////////////////////
//  Interfaces & Types

export type StepTurn = Extract<Turn, { role: "step" }>;

export interface Codec {
	encode(request: CanonicalRequest, mode: "stream" | "complete"): Encoded;
	/** Ends at the API's end marker; in-stream errors and early ends throw `ProviderError`. */
	stream(items: AsyncIterable<SseItem>): AsyncIterable<StepEvent>;
	/** A non-streaming reply, as the stream would have assembled it. */
	complete(body: unknown): StepResult;
	failure(status: number, body: unknown, headers: HeaderList): Failure;
}
