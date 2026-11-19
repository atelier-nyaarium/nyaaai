// The dialect check and the generator agree.

import { describe, expect, it } from "bun:test";
import { dialectProblems, exportSchemas } from "@nyaaai/contract/schemaExport";
import { refined, shapeSet, z } from "@nyaaai/contract/shape";
import { irOf } from "../contract/ir";
import { emitRust } from "../contract/rust";

////////////////////////////////
//  Functions & Helpers

/** Generates the set, as the contract script does after the check. */
function generated(name: string): string {
	const doc = exportSchemas().docs.get(name)?.output;
	if (!doc) throw new Error(`no set ${name}`);
	const sets = irOf(new Map([[name, doc]]));
	return emitRust(sets.get(name) as never, sets, new Map([[name, { crate: "c", module: "m", file: "m.rs" }]]));
}

function runs(run: () => unknown): boolean {
	try {
		run();
	} catch {
		return false;
	}
	return true;
}

let counter = 0;

////////////////////////////////
//  Tests

describe("the dialect check and the generator", () => {
	it("refuse the same shapes, and the check says where and what to write", () => {
		const flag = (value: boolean) => z.object({ ok: z.literal(value) });
		const rows: Array<[string, Record<string, z.ZodType>, string, string]> = [
			["boolean tag", { Shape: z.discriminatedUnion("ok", [flag(true), flag(false)]) }, "Shape", "plain union"],
			[
				"enum tag",
				{
					Shape: z.discriminatedUnion("kind", [
						z.object({ kind: z.enum(["a", "b"]) }),
						z.object({ kind: z.literal("c") }),
					]),
				},
				"Shape",
				"one branch per value",
			],
			[
				"nameless literal",
				{ Shape: z.union([z.literal("."), z.number()]) },
				"Shape",
				"comes to no Rust variant name",
			],
			["describe", { Shape: z.object({ a: z.string().describe("text") }) }, "Shape.a", "write a comment"],
			["variant twins", { Shape: z.enum(["a-b", "a_b"]) }, "Shape", "both come to the variant AB"],
			[
				"tag twins",
				{
					Shape: z.discriminatedUnion("kind", [
						z.object({ kind: z.literal("a-b") }),
						z.object({ kind: z.literal("a_b") }),
					]),
				},
				"Shape",
				"both come to the variant AB",
			],
			["empty twins", { Shape: z.enum(["", "empty"]) }, "Shape", "both come to the variant Empty"],
			[
				"empty tag twins",
				{
					Shape: z.discriminatedUnion("kind", [
						z.object({ kind: z.literal("") }),
						z.object({ kind: z.literal("empty") }),
					]),
				},
				"Shape",
				"both come to the variant Empty",
			],
			[
				"union twins",
				{ Shape: z.union([z.literal("text1"), z.string(), z.string()]) },
				"Shape",
				"both come to the variant Text1",
			],
			["shape name", { bad_name: z.string() }, "bad_name", "PascalCase"],
			[
				"field twins",
				{ Shape: z.object({ "a-b": z.string(), a_b: z.string() }) },
				"Shape",
				"both come to the field a_b",
			],
			["fractional integer bound", { Shape: z.object({ n: z.int().min(0.5) }) }, "Shape.n", "is whole"],
			[
				"hook twins",
				{
					Shape: z.object({
						a: refined("fooBar", z.number(), () => true),
						b: refined("foo_bar", z.number(), () => true),
					}),
				},
				"Shape.b",
				"both come to foo_bar",
			],
		];
		for (const [label, shapes, where, hint] of rows) {
			counter += 1;
			const name = `agree${counter}`;
			const set = shapeSet(name, shapes);
			const problems = dialectProblems([set]);
			expect([label, problems.some((problem) => problem.startsWith(`${name}.${where}:`))]).toEqual([label, true]);
			expect([label, problems.some((problem) => problem.includes(hint))]).toEqual([label, true]);
			expect([label, runs(() => generated(name))]).toEqual([label, false]);
		}
	});

	it("pass a literal branch that is a named shape, nullable or not", () => {
		counter += 1;
		const name = `agree${counter}`;
		const Root = z.literal(".");
		const set = shapeSet(name, {
			Root,
			Shape: z.object({ plain: z.union([Root, z.string()]), absent: z.union([Root.nullable(), z.number()]) }),
		});
		expect(dialectProblems([set])).toEqual([]);
		expect(generated(name)).toContain("Root(Root)");
	});

	it("reads a type list of distinct plain types only", () => {
		const read = (type: string[]) => {
			const doc = { $schema: "", $id: "", "x-contract-version": "0.0.0", $defs: { Shape: { type } as never } };
			return irOf(new Map([["lists", doc]]));
		};
		expect(runs(() => read(["number", "string"]))).toBe(true);
		expect(runs(() => read(["string", "string"]))).toBe(false);
		expect(runs(() => read(["number", "integer"]))).toBe(false);
		expect(runs(() => read(["integer", "null", "integer"]))).toBe(false);
	});

	it("refuses an alias that names itself before the generator runs", () => {
		counter += 1;
		const name = `agree${counter}`;
		const A: z.ZodType = z.array(z.lazy(() => B));
		const B: z.ZodType = z.array(z.lazy(() => A));
		const set = shapeSet(name, { A, B });
		expect(dialectProblems([set]).filter((problem) => problem.includes("object or a union"))).toHaveLength(2);
		expect(runs(() => generated(name))).toBe(false);
	});

	it("names a keyword set's hook module without the underscore", () => {
		counter += 1;
		const name = "move";
		const doc = { type: "string", "x-hooks": [{ kind: "refine", name: "isShort" }] };
		const sets = irOf(
			new Map([
				[
					name,
					{
						$schema: "",
						$id: "",
						"x-contract-version": "0.0.0",
						$defs: { Shape: doc as never },
					},
				],
			]),
		);
		const rust = emitRust(
			sets.get(name) as never,
			sets,
			new Map([[name, { crate: "c", module: "move_", file: "move_.rs" }]]),
		);
		expect(rust).toContain("use super::move_hooks as hooks;");
		expect(rust).not.toContain("move__hooks");
	});
});
