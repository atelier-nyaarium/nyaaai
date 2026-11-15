import { stringifyJson } from "@nyaaai/contract/json";
import { decodeText, shapeSets, z } from "@nyaaai/contract/shape";
import { PathPart } from "@nyaaai/contract/shapes/frames";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { contractCases } from "../caseSources/contract";

export const contract = caseGroup("contract", {
	kinds: {
		roundTrip: kind(
			z.object({ set: z.string(), shape: z.string(), texts: z.array(z.string()) }),
			z.object({
				results: z.array(
					z.object({
						ok: z.boolean(),
						text: z.string().exactOptional(),
						path: z.array(PathPart).exactOptional(),
						/** An authored refusal text. */
						message: z.string().exactOptional(),
					}),
				),
			}),
			(input) => {
				const schema = shapeSets().find((set) => set.name === input.set)?.shapes[input.shape];
				if (!schema) throw new Error(`no shape ${input.set}.${input.shape}`);
				return {
					results: input.texts.map((text) => {
						const held = decodeText(schema, text);
						if (held.ok) return { ok: true, text: stringifyJson(held.value) };
						return { ok: false, path: held.path, ...(held.fixed ? { message: held.message } : {}) };
					}),
				};
			},
		),
	},
	sources: casesFor("contract", contractCases),
});
