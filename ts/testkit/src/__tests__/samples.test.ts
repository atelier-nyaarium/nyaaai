// Sampler tests.

import { expect, it } from "bun:test";
import type { SetSchema } from "@nyaaai/contract/schemaExport";
import { sampleTexts } from "../samples";

////////////////////////////////
//  Constants

const DOCS = new Map<string, SetSchema>([
	[
		"t",
		{
			$schema: "https://json-schema.org/draft/2020-12/schema",
			$id: "t.input.json",
			"x-contract-version": "0.1.0",
			$defs: {
				Thing: {
					type: "object",
					properties: {
						name: { type: "string", minLength: 1 },
						count: { type: "integer", minimum: 0, maximum: 5 },
						flag: { type: "boolean" },
					},
					required: ["name", "count"],
				},
				Pair: {
					type: "object",
					properties: { list: { type: "array", items: { type: "boolean" }, minItems: 2 } },
					required: ["list"],
				},
				Wide: {
					type: "object",
					properties: Object.fromEntries(
						Array.from({ length: 900 }, (_, at) => [`k${at}`, { type: "integer", minimum: 0, maximum: 5 }]),
					),
					required: [],
				},
				Holder: {
					type: "object",
					properties: {
						list: {
							type: "array",
							items: {
								anyOf: [
									{
										type: "object",
										properties: { a: { type: "string" }, b: { type: "boolean" } },
										required: ["a"],
									},
									{ type: "string" },
								],
							},
							minItems: 1,
						},
					},
					required: ["list"],
				},
			},
		},
	],
]);

////////////////////////////////
//  Tests

it("samples a shape the same way twice, valid values first, then broken ones", () => {
	const texts = sampleTexts(DOCS, "t", "Thing");
	expect(sampleTexts(DOCS, "t", "Thing")).toEqual(texts);
	expect(texts[0]).toBe('{"name":"a","count":1,"flag":true}');
	expect(texts[1]).toBe('{"name":"a","count":1}');
	expect(texts).toContain('{"count":1,"flag":true}');
	expect(texts).toContain('{"name":"a","count":6,"flag":true}');
});

it("walks every union option, and pairs a short list with a bad item", () => {
	const texts = sampleTexts(DOCS, "t", "Holder");
	expect(texts).toContain('{"list":[{"a":"a","b":"x"}]}');
	expect(texts).toContain('{"list":["x"]}');
	expect(texts).toContain('{"list":[]}');
	expect(texts).toContain('{"list":[{"a":"a","b":true}],"unknownKey":0}');
});

it("pairs a list one item short with a bad first item", () => {
	expect(sampleTexts(DOCS, "t", "Pair")).toContain('{"list":[null]}');
});

it("drops no text of a wide shape", () => {
	const texts = sampleTexts(DOCS, "t", "Wide");
	const broken = (at: number) => texts.some((text) => text.includes(`"k${at}":6`) || text.includes(`"k${at}":-1`));
	expect({ over: texts.length > 2000, first: broken(0), last: broken(899) }).toEqual({
		over: true,
		first: true,
		last: true,
	});
});
