import { SseDecoder } from "@nyaaai/agentic/sse";
import { base64Decode } from "@nyaaai/contract/base64";
import { stringifyJson } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { SseItem } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { sseCases } from "../caseSources/sse";

////////////////////////////////
//  Functions & Helpers

function bytesOf(chunk: string): Uint8Array {
	const bytes = base64Decode(chunk);
	if (bytes === undefined) throw new Error("a chunk is not base64");
	return bytes;
}

/** Items of each push, then of `end`. */
function decodeChunks(chunks: readonly Uint8Array[]): { pushed: SseItem[][]; ended: SseItem[] } {
	const decoder = new SseDecoder();
	const pushed = chunks.map((chunk) => decoder.push(chunk));
	return { pushed, ended: decoder.end() };
}

const flat = (chunks: readonly Uint8Array[]): string => {
	const { pushed, ended } = decodeChunks(chunks);
	return stringifyJson([...pushed.flat(), ...ended]);
};

export const sse = caseGroup("sse", {
	kinds: {
		/** The items each chunk yields, then what `end` adds. */
		items: kind(
			z.object({
				chunks: z.array(z.string()),
				/** Chunks pushed after the first `end`, then `end` again. */
				again: z.array(z.string()).exactOptional(),
			}),
			z.object({
				pushed: z.array(z.array(SseItem)),
				ended: z.array(SseItem),
				pushedAgain: z.array(z.array(SseItem)).exactOptional(),
				endedAgain: z.array(SseItem).exactOptional(),
			}),
			(input) => {
				const decoder = new SseDecoder();
				const pushed = input.chunks.map(bytesOf).map((chunk) => decoder.push(chunk));
				const ended = decoder.end();
				if (input.again === undefined) return { pushed, ended };
				const pushedAgain = input.again.map(bytesOf).map((chunk) => decoder.push(chunk));
				return { pushed, ended, pushedAgain, endedAgain: decoder.end() };
			},
		),
		/** Whether every two-way split, and each byte alone, gives the whole stream's items. */
		splits: kind(
			z.object({ bytes: z.string() }),
			z.object({
				items: z.array(SseItem),
				agree: z.boolean(),
				/** The first cut whose items differ. */
				firstDifferent: z.int().nullable(),
				bytewise: z.boolean(),
			}),
			(input) => {
				const bytes = bytesOf(input.bytes);
				const { pushed, ended } = decodeChunks([bytes]);
				const items = [...pushed.flat(), ...ended];
				const whole = stringifyJson(items);
				let firstDifferent: number | null = null;
				for (let cut = 1; cut < bytes.length && firstDifferent === null; cut++) {
					if (flat([bytes.subarray(0, cut), bytes.subarray(cut)]) !== whole) firstDifferent = cut;
				}
				const singles = Array.from(bytes, (byte) => new Uint8Array([byte]));
				return {
					items,
					agree: firstDifferent === null,
					firstDifferent,
					bytewise: flat(singles) === whole,
				};
			},
		),
	},
	sources: casesFor("sse", sseCases),
});
