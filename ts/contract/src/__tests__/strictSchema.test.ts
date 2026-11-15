import { describe, expect, it } from "bun:test";
import { z } from "zod";
import { schemaProblems, strictSchemaOf, validate } from "../strictSchema";

interface Tree {
	label: string;
	children: Tree[];
}

function treeShape(): z.ZodType<Tree> {
	let tree: z.ZodType<Tree>;
	tree = z.lazy(() => z.object({ label: z.string(), children: z.array(tree) }));
	return tree;
}

describe("strict schemas", () => {
	it("exports representative recursive schemas", () => {
		const shape = z.object({
			name: z.string().min(1).max(12).exactOptional(),
			ratio: z.number().min(0).max(1),
			state: z.enum(["ready", "busy"]),
			anything: z.unknown(),
			tags: z.array(z.string().max(8)).max(4),
			flags: z.record(z.string(), z.boolean()),
			choice: z.discriminatedUnion("kind", [
				z.object({ kind: z.literal("text"), value: z.string() }),
				z.object({ kind: z.literal("count"), value: z.int() }),
			]),
			tree: treeShape(),
		});
		const schema = strictSchemaOf(shape);
		expect(schemaProblems(schema)).toEqual([]);

		const leaf = { label: "leaf", children: [] };
		const good = {
			ratio: 0.5,
			state: "busy",
			anything: null,
			tags: ["a"],
			flags: { on: true },
			choice: { kind: "count", value: 2 },
			tree: { label: "root", children: [leaf] },
		};
		expect(validate(schema, good)).toEqual([]);
		const bad = {
			...good,
			name: "",
			ratio: 2,
			state: "idle",
			tags: ["a", "b", "c", "d", "e"],
			flags: { on: 1 },
			choice: { kind: "count", value: 2.5 },
			tree: { label: "root", children: [{ label: 1, children: [] }] },
		};
		expect(validate(schema, bad).map(({ path, code }) => `${path.join(".")}:${code}`)).toEqual([
			"ratio:maximum",
			"state:enum",
			"tags:max_items",
			"flags.on:type",
			"choice:no_branch",
			"tree.children.0.label:type",
			"name:min_length",
		]);
	});

	it("rejects unsupported Zod shapes", () => {
		const unsupported = [
			z.any(),
			z.object({ anything: z.any() }),
			z.date(),
			z.tuple([z.string(), z.number()]),
			z.intersection(z.object({ a: z.string() }), z.object({ b: z.number() })),
		];
		for (const shape of unsupported) expect(() => strictSchemaOf(shape)).toThrow();
	});

	it("drops exported string checks and keeps properties named like them", () => {
		const schema = strictSchemaOf(z.object({ pattern: z.string().regex(/^x/), format: z.string().url() }));
		expect(schema).toEqual({
			type: "object",
			properties: { pattern: { type: "string" }, format: { type: "string" } },
			required: ["pattern", "format"],
		});
	});

	it("accepts a recursive root", () => {
		expect(schemaProblems(strictSchemaOf(treeShape()))).toEqual([]);
	});
});
