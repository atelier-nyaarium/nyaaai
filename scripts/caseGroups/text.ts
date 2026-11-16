import { base64Decode } from "@nyaaai/contract/base64";
import { z } from "@nyaaai/contract/shape";
import {
	asciiLower,
	asciiUpper,
	collapseSpace,
	cutAfter,
	cutBefore,
	cutUtf8,
	encodePathSegment,
	hasControl,
	headOf,
	indexOfText,
	lastIndexOfText,
	nfc,
	sliceText,
	tailOf,
	textLength,
	trimText,
	Utf8Decoder,
	utf8Length,
} from "@nyaaai/contract/text";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { textCases } from "../caseSources/text";

export const text = caseGroup("text", {
	kinds: {
		cuts: kind(
			z.object({ texts: z.array(z.string()), points: z.array(z.int().min(0)) }),
			z.object({
				rows: z.array(
					z.object({
						length: z.int(),
						utf8: z.int(),
						before: z.array(z.int()),
						after: z.array(z.int()),
						heads: z.array(z.string()),
						tails: z.array(z.string()),
						bytes: z.array(z.string()),
						lower: z.string(),
						upper: z.string(),
					}),
				),
			}),
			(input) => ({
				rows: input.texts.map((held) => ({
					length: textLength(held),
					utf8: utf8Length(held),
					before: input.points.map((at) => cutBefore(held, at)),
					after: input.points.map((at) => cutAfter(held, at)),
					heads: input.points.map((max) => headOf(held, max)),
					tails: input.points.map((max) => tailOf(held, max)),
					bytes: input.points.map((max) => cutUtf8(held, max)),
					lower: asciiLower(held),
					upper: asciiUpper(held),
				})),
			}),
		),
		slices: kind(
			z.object({ text: z.string(), spans: z.array(z.object({ start: z.int().min(0), end: z.int().min(0) })) }),
			z.object({ slices: z.array(z.string()) }),
			(input) => ({ slices: input.spans.map((span) => sliceText(input.text, span.start, span.end)) }),
		),
		spaces: kind(
			z.object({ texts: z.array(z.string()) }),
			z.object({ trimmed: z.array(z.string()), collapsed: z.array(z.string()) }),
			(input) => ({ trimmed: input.texts.map(trimText), collapsed: input.texts.map(collapseSpace) }),
		),
		/** `-1` marks no match. */
		find: kind(
			z.object({ text: z.string(), needles: z.array(z.string()), froms: z.array(z.int().min(0)) }),
			z.object({ first: z.array(z.array(z.int())), last: z.array(z.array(z.int())) }),
			(input) => ({
				first: input.needles.map((needle) => input.froms.map((from) => indexOfText(input.text, needle, from))),
				last: input.needles.map((needle) =>
					input.froms.map((from) => lastIndexOfText(input.text, needle, from)),
				),
			}),
		),
		/** Chunks are base64; each push's text, then the end's. `again` reuses the decoder. */
		utf8Chunks: kind(
			z.object({
				runs: z.array(z.object({ chunks: z.array(z.string()), again: z.array(z.string()).exactOptional() })),
			}),
			z.object({
				runs: z.array(
					z.object({
						pushed: z.array(z.string()),
						ended: z.string(),
						second: z.object({ pushed: z.array(z.string()), ended: z.string() }).exactOptional(),
					}),
				),
			}),
			(input) => ({
				runs: input.runs.map((held) => {
					const decoder = new Utf8Decoder();
					const pass = (chunks: string[]) => {
						const pushed = chunks.map((chunk) => {
							const bytes = base64Decode(chunk);
							if (bytes === undefined) throw new Error("a chunk is not base64");
							return decoder.push(bytes);
						});
						return { pushed, ended: decoder.end() };
					};
					const first = pass(held.chunks);
					return held.again === undefined ? first : { ...first, second: pass(held.again) };
				}),
			}),
		),
		/** True when a text holds a character of category Cc. */
		hasControl: kind(
			z.object({ texts: z.array(z.string()) }),
			z.object({ results: z.array(z.boolean()) }),
			(input) => ({
				results: input.texts.map(hasControl),
			}),
		),
		pathSegment: kind(
			z.object({ texts: z.array(z.string()) }),
			z.object({ texts: z.array(z.string()) }),
			(input) => ({
				texts: input.texts.map(encodePathSegment),
			}),
		),
		nfc: kind(z.object({ texts: z.array(z.string()) }), z.object({ texts: z.array(z.string()) }), (input) => ({
			texts: input.texts.map(nfc),
		})),
	},
	sources: casesFor("text", textCases),
});
