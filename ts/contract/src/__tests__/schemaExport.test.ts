// Dialect tests: bun test.

import { describe, expect, it } from "bun:test";
import { z } from "zod";
import { dialectProblems } from "../schemaExport";
import { matching, preprocessed, refined } from "../shape";

////////////////////////////////
//  Functions & Helpers

const problemsOf = (shape: z.ZodType) => dialectProblems([{ name: "t", shapes: { Shape: shape } }]);

interface Loop {
	next: Loop[];
}

////////////////////////////////
//  Tests

describe("the contract dialect", () => {
	it("takes a hook only when it is registered", () => {
		const positive = refined("positive", z.number(), (ratio) => ratio > 0);
		const registered = z.object({
			ratio: positive,
			small: refined("small", positive, (ratio) => ratio < 9, "Stay small."),
			label: preprocessed("trimmed", (raw) => raw, z.string()),
			code: matching(z.string().max(9), "^[a-z]+$", "i"),
			pair: z.tuple([z.string(), z.tuple([z.number()]).nullable()]),
		});
		expect(problemsOf(registered)).toEqual([]);

		const bare: Array<[string, z.ZodType]> = [
			["t.Shape.ratio", z.object({ ratio: z.number().refine((ratio) => ratio > 0) })],
			["t.Shape.ratio", z.object({ ratio: positive.refine((ratio) => ratio < 9) })],
			["t.Shape.label", z.object({ label: z.preprocess((raw) => raw, z.string()) })],
			["t.Shape.size", z.object({ size: z.string().transform((text) => text.length) })],
		];
		for (const [where, shape] of bare) {
			const problems = problemsOf(shape);
			expect(problems.length).toBeGreaterThan(0);
			expect(problems.every((problem) => problem.startsWith(`${where}:`))).toBe(true);
		}
	});

	it("refuses what the two languages would read differently", () => {
		const anonymous: z.ZodType<Loop> = z.object({
			get next() {
				return z.array(anonymous);
			},
		});
		const refusedAt: Array<[string, z.ZodType]> = [
			["t.Shape.bytes", z.object({ bytes: z.base64() })],
			["t.Shape.at", z.object({ at: z.iso.datetime() })],
			["t.Shape.byId", z.object({ byId: z.record(z.uuid(), z.number()) })],
			["t.Shape.pair", z.object({ pair: z.tuple([z.string()], z.number()) })],
			["t.Shape.none", z.object({ none: z.tuple([]) })],
			[
				"t.Shape.wide",
				z.object({ wide: z.tuple(Array.from({ length: 13 }, () => z.string()) as [z.ZodString]) }),
			],
			[
				"t.Shape.fixed",
				z.object({ fixed: z.tuple([z.string(), preprocessed("fix", (raw) => raw, z.string())]) }),
			],
			["t.Shape.pair[1]", z.object({ pair: z.tuple([z.string(), z.number().optional()]) })],
			["t.Shape.code", z.object({ code: matching(z.string(), "^a").max(3) })],
			["t.Shape.any", z.object({ any: z.any() })],
			["t.Shape.when", z.object({ when: z.date() })],
			["t.Shape.open", z.object({ open: z.looseObject({ a: z.string() }) })],
			["t.Shape.shut", z.object({ shut: z.strictObject({ a: z.string() }) })],
			["t.Shape.byKind", z.object({ byKind: z.record(z.enum(["a", "b"]), z.number()) })],
			["t.Shape.either", z.object({ either: z.literal(["a", "b"]) })],
			["t.Shape.list[]", z.object({ list: z.array(z.string().optional()) })],
			["t.Shape.late", z.object({ late: z.string().optional().nullable() })],
			["t.Shape.count", z.object({ count: z.int().default(1.5) })],
			["t.Shape.small", z.object({ small: z.int32() })],
			["t.Shape.code", z.object({ code: z.string().regex(/^a/) })],
			["t.Shape", z.object({ ["__proto__"]: z.string() })],
			["t.Shape", z.object({ 7: z.string() })],
			["t.Shape.fixed", z.object({ fixed: preprocessed("fix", (raw) => raw, z.string()).nullable() })],
			["t.Shape.next[]", anonymous],
		];
		for (const [where, shape] of refusedAt) {
			expect(problemsOf(shape).map((problem) => problem.slice(0, problem.indexOf(":")))).toEqual([where]);
		}
	});
});
