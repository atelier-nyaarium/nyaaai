import { z } from "@nyaaai/contract/shape";
import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Functions & Helpers

type Projection = "clean" | "nullable" | "withoutNulls" | "strict";

const project = (projection: Projection, name: string, schema: object, value: unknown = null): CaseSource => ({
	kind: "schemaProjection.project",
	name: `${projection}-${name}`,
	input: { projection, schema, value },
});

const clean = (name: string, schema: object) => project("clean", name, schema);
const nullable = (name: string, schema: object) => project("nullable", name, schema);
const strict = (name: string, schema: object) => project("strict", name, schema);
const without = (name: string, schema: object, value: unknown) => project("withoutNulls", name, schema, value);

/** What Zod sends for a shape. */
const sent = (shape: z.ZodType): object => z.toJSONSchema(shape);

const DRAFT = "https://json-schema.org/draft/2020-12/schema";

const str = { type: "string" };

/** Every field optional, in objects and arrays. */
const OPTIONALS = {
	type: "object",
	properties: {
		note: { type: "string", description: "A note." },
		count: { type: "integer" },
		kind: { type: "string", enum: ["a", "b"] },
		fixed: { const: "x" },
		list: { type: "array", items: { type: "object", properties: { tag: { type: "string" } } } },
		at: { anyOf: [{ type: "object", properties: { line: { type: "number" } } }, { type: "string" }] },
		sure: { type: "string" },
	},
	required: ["sure"],
};

/** The null-dropping check's schema, and a value a model would send for it. */
const FILLED = {
	type: "object",
	properties: {
		note: { type: "string" },
		items: { type: "array", items: { type: "object", properties: { tag: { type: "string" } } } },
		at: { anyOf: [{ type: "object", properties: { line: { type: "number" } } }, { type: "string" }] },
	},
	required: [],
};

type Node = { value: string; next: Node | null };
const List: z.ZodType<Node> = z.lazy(() => z.object({ value: z.string(), next: List.nullable() }));

export const schemaProjectionCases: CaseSource[] = [
	// Clean.
	clean("object-root", {
		$schema: DRAFT,
		type: "object",
		properties: {
			url: { type: "string", format: "uri" },
			ref: { type: "string", format: "uri-reference" },
			when: { type: "string", format: "date-time" },
			mail: { type: "string", format: "email" },
			deep: { type: "array", items: { type: "object", properties: { link: { type: "string", format: "uri" } } } },
			either: { anyOf: [{ type: "string", format: "uri" }, { type: "null" }] },
		},
		required: ["url"],
	}),
	clean("non-object-root", { $schema: DRAFT, type: "string", format: "uri" }),
	clean("array-root", {
		$schema: DRAFT,
		type: "array",
		items: { type: "object", properties: { link: { type: "string", format: "uri" } } },
	}),
	clean("root-open-becomes-closed", { type: "object", additionalProperties: true, properties: { a: str } }),
	clean("root-record-becomes-closed", { type: "object", additionalProperties: str }),
	clean("root-without-properties", { type: "object" }),
	clean("no-dollar-schema", { type: "object", properties: { a: str } }),
	clean("empty-dollar-schema", { $schema: "", type: "object" }),
	clean("type-list-root", {
		$schema: DRAFT,
		type: ["object", "null"],
		properties: { u: { type: "string", format: "uri" } },
	}),
	clean("no-type-root", { $schema: DRAFT, properties: { u: { type: "string", format: "uri" } } }),
	clean("format-named-property", {
		type: "object",
		properties: { format: { type: "string" }, other: { type: "string", format: "uri" } },
	}),
	clean("format-in-values", {
		type: "object",
		default: { format: "uri", keep: "uri" },
		properties: {
			u: { type: "string", enum: ["uri"], examples: [{ format: "uri-reference" }, { format: "other" }] },
		},
	}),
	clean("format-not-text", {
		type: "object",
		properties: { a: { format: 1 }, b: { format: null }, c: { format: ["uri"] } },
	}),
	clean("refs-and-defs-untouched", {
		type: "object",
		properties: { a: { $ref: "#/$defs/A" } },
		$defs: { A: { type: "string", format: "uri" } },
	}),
	clean("pattern-kept", { type: "object", properties: { a: { type: "string", pattern: "^x+$" } } }),
	clean("empty-object", {}),

	// Nullable optionals.
	nullable("optionals", OPTIONALS),
	nullable("required-stay", {
		type: "object",
		properties: { a: str, b: { type: "number" } },
		required: ["a", "b"],
	}),
	nullable("required-is-set-to-all", { type: "object", properties: { b: str, a: str } }),
	nullable("required-extra-names-dropped", { type: "object", properties: { a: str }, required: ["gone", "a"] }),
	nullable("required-not-a-list", { type: "object", properties: { a: str }, required: "a" }),
	nullable("accepts-null", {
		type: "object",
		properties: {
			listed: { type: ["string", "null"] },
			member: { type: "null" },
			branch: { anyOf: [str, { type: "null" }] },
			oneBranch: { oneOf: [str, { type: "null" }] },
			enumNull: { type: "string", enum: ["a", null] },
			constNull: { const: null },
			untyped: {},
			untypedWithKeys: { description: "Anything." },
		},
	}),
	nullable("does-not-accept-null", {
		type: "object",
		properties: {
			plain: str,
			enumed: { type: "string", enum: ["a", "b"] },
			constant: { type: "string", const: "x" },
			numberList: { type: ["string", "number"] },
			branches: { anyOf: [str, { type: "number" }] },
			oneBranches: { oneOf: [str, { type: "number" }] },
			branchNotSchema: { anyOf: [true, str] },
		},
	}),
	nullable("wrapped-keeps-description", {
		type: "object",
		properties: {
			a: { type: "string", enum: ["x"], description: "Pick.", title: "A" },
			b: { const: 1, description: "One." },
			c: { type: ["string", "number"], description: "Either." },
		},
	}),
	nullable("type-list-with-description-stays-wrapped", {
		type: "object",
		properties: { a: { type: ["string", "number"] } },
	}),
	nullable("nested-objects", {
		type: "object",
		properties: {
			outer: {
				type: "object",
				properties: { inner: { type: "object", properties: { leaf: str } }, sure: str },
				required: ["sure"],
			},
		},
		required: ["outer"],
	}),
	nullable("optional-object", {
		type: "object",
		properties: { opt: { type: "object", properties: { leaf: str }, required: [] } },
	}),
	nullable("arrays-and-branches", {
		type: "array",
		items: { type: "object", properties: { a: str } },
	}),
	nullable("branches-are-widened", {
		anyOf: [
			{ type: "object", properties: { a: str } },
			{ type: "object", properties: { b: str }, required: ["b"] },
			true,
			"x",
		],
		oneOf: [{ type: "object", properties: { c: str } }],
	}),
	nullable("tuple-items-untouched", { type: "array", items: [{ type: "object", properties: { a: str } }] }),
	nullable("non-schema-properties", {
		type: "object",
		properties: { a: true, b: false, c: "text", d: null, e: [1], f: str },
		required: ["f"],
	}),
	nullable("properties-not-an-object", { type: "object", properties: [str], required: ["a"] }),
	nullable("no-properties-no-required", { type: "object", description: "Free." }),
	nullable("key-order-kept", {
		required: ["z"],
		properties: { z: str, y: str },
		type: "object",
		description: "Ordered.",
	}),
	nullable("integer-like-property-names", { type: "object", properties: { b: str, "2": str, a: str, "1": str } }),
	nullable("empty", {}),
	nullable(
		"zod-optional-fields",
		sent(z.object({ note: z.string().optional(), n: z.number().nullable(), ok: z.string() })),
	),

	// Without nulls.
	without("filled-optionals", FILLED, { note: null, items: [{ tag: null }], at: { line: null } }),
	without("union-node-as-sent", FILLED, { at: null, items: [null, { tag: "x" }], note: "kept" }),
	without(
		"required-null-stays",
		{ type: "object", properties: { a: str, b: str }, required: ["a"] },
		{ a: null, b: null },
	),
	without(
		"accepts-null-stays",
		{ type: "object", properties: { a: { type: ["string", "null"] }, u: {}, c: { const: null } } },
		{ a: null, u: null, c: null },
	),
	without("values-stay", FILLED, { note: "text", items: [{ tag: "t" }] }),
	without("falsy-values-stay", { type: "object", properties: { a: str, b: str, c: str } }, { a: 0, b: "", c: false }),
	without("unknown-keys-stay", FILLED, { other: null, note: null, deeper: { a: null } }),
	without(
		"nested-objects",
		{
			type: "object",
			properties: { outer: { type: "object", properties: { leaf: str, sure: str }, required: ["sure"] } },
		},
		{ outer: { leaf: null, sure: null } },
	),
	without(
		"nested-arrays",
		{ type: "array", items: { type: "array", items: { type: "object", properties: { a: str } } } },
		[[{ a: null }], [], [null]],
	),
	without("array-without-items", { type: "array" }, [{ a: null }, null]),
	without("array-items-not-a-schema", { type: "array", items: true }, [{ a: null }]),
	without("array-value-for-object-schema", FILLED, [{ note: null }]),
	without("object-without-properties", { type: "object" }, { a: null }),
	without("properties-not-an-object", { type: "object", properties: [str] }, { a: null }),
	without("scalar-values", FILLED, "text"),
	without("null-value", FILLED, null),
	without("number-value", FILLED, 5),
	without("property-not-a-schema", { type: "object", properties: { a: true, b: str } }, { a: null, b: null }),
	without("key-order-kept", { type: "object", properties: { a: str, b: str, c: str } }, { c: 1, b: null, a: 2 }),
	without("empty-schema", {}, { a: null }),
	without("empty-object-value", FILLED, {}),
	without("integer-like-keys", { type: "object", properties: { "1": str, b: str } }, { b: 1, "1": null, "2": 3 }),
	without("inherited-names-are-not-properties", FILLED, { toString: null, constructor: null, hasOwnProperty: null }),

	// Strict: accepted.
	strict(
		"zod-nullable-fields",
		sent(z.object({ name: z.string(), depth: z.number().nullable(), tags: z.array(z.string()) })),
	),
	strict(
		"zod-nested",
		sent(
			z.object({ pos: z.object({ x: z.number(), y: z.number() }), list: z.array(z.object({ id: z.string() })) }),
		),
	),
	strict("zod-union", sent(z.object({ pick: z.union([z.object({ a: z.string() }), z.object({ b: z.number() })]) }))),
	strict("zod-enum-and-literal", sent(z.object({ kind: z.enum(["a", "b"]), tag: z.literal("x") }))),
	strict("zod-uri-format", sent(z.object({ home: z.url() }))),
	strict("one-of-becomes-any-of", {
		type: "object",
		properties: { a: { oneOf: [str, { type: "number" }] } },
		required: ["a"],
	}),
	strict("any-of-wins-over-one-of", {
		type: "object",
		properties: { a: { anyOf: [str], oneOf: [{ type: "number" }] } },
		required: ["a"],
	}),
	strict("branches-keep-only-schemas", {
		type: "object",
		properties: { a: { anyOf: [str, true, "x", { type: "number" }] } },
		required: ["a"],
	}),
	strict("branches-are-closed", {
		type: "object",
		properties: {
			a: {
				anyOf: [
					{ type: "object", properties: { x: str }, required: ["x"] },
					{ type: "object", properties: { y: str }, required: ["y"], additionalProperties: true },
				],
			},
		},
		required: ["a"],
	}),
	strict("root-open-flag-is-overwritten", {
		type: "object",
		additionalProperties: true,
		properties: { a: str },
		required: ["a"],
	}),
	strict("nested-open-flag-is-overwritten", {
		type: "object",
		properties: { a: { type: "object", additionalProperties: true, properties: { b: str }, required: ["b"] } },
		required: ["a"],
	}),
	strict("nested-object-without-properties", {
		type: "object",
		properties: { a: { type: "object" } },
		required: ["a"],
	}),
	strict("root-without-properties", { type: "object" }),
	strict("array-root-is-not-cleaned", {
		$schema: DRAFT,
		type: "array",
		items: { type: "object", properties: { u: { type: "string", format: "uri" } }, required: ["u"] },
	}),
	strict("string-root", { $schema: DRAFT, type: "string", format: "uri" }),
	strict("pattern-and-bounds-kept", {
		type: "object",
		properties: { a: { type: "string", pattern: "^x$", minLength: 1 }, b: { type: "integer", minimum: 0 } },
		required: ["a", "b"],
	}),
	strict("items-is-closed", {
		type: "object",
		properties: { list: { type: "array", items: { type: "object", properties: { a: str }, required: ["a"] } } },
		required: ["list"],
	}),
	strict("tuple-items-are-not-closed", {
		type: "object",
		properties: { list: { type: "array", items: [{ type: "object", properties: { a: str } }] } },
		required: ["list"],
	}),
	strict("boolean-additional-properties-is-fine", {
		type: "object",
		properties: { a: { type: "object", additionalProperties: false } },
		required: ["a"],
	}),
	strict("required-with-no-properties", { type: "object", required: ["a"] }),
	strict("not-a-schema-property", { type: "object", properties: { a: true }, required: ["a"] }),
	strict("empty", {}),

	// Strict: refused.
	strict("refuses-optional-field", sent(z.object({ note: z.string().optional() }))),
	strict(
		"refuses-optional-field-in-a-union",
		sent(z.object({ at: z.union([z.object({ a: z.string() }), z.object({ b: z.string().optional() })]) })),
	),
	strict("refuses-record", sent(z.object({ tags: z.record(z.string(), z.string()) }))),
	strict("refuses-recursion", sent(z.object({ head: List }))),
	strict("refuses-tuple", sent(z.object({ pair: z.tuple([z.string(), z.number()]) }))),
	strict("refuses-ref", { type: "object", properties: { a: { $ref: "#/$defs/A" } }, required: ["a"] }),
	strict("refuses-defs", { type: "object", properties: { a: str }, required: ["a"], $defs: { A: str } }),
	strict("refuses-defs-before-ref", {
		type: "object",
		properties: { a: { $ref: "#/$defs/A" } },
		required: ["a"],
		$defs: { A: str },
	}),
	strict("refuses-prefix-items", {
		type: "object",
		properties: { a: { type: "array", prefixItems: [str, { type: "number" }] } },
		required: ["a"],
	}),
	strict("refuses-root-prefix-items", { type: "array", prefixItems: [str] }),
	strict("refuses-open-record-root", { type: "object", additionalProperties: str }),
	strict("refuses-open-record-in-items", {
		type: "object",
		properties: { list: { type: "array", items: { type: "object", additionalProperties: { type: "number" } } } },
		required: ["list"],
	}),
	strict("refuses-in-a-branch", {
		type: "object",
		properties: { a: { anyOf: [str, { type: "object", properties: { b: str } }] } },
		required: ["a"],
	}),
	strict("refuses-first-optional", { type: "object", properties: { a: str, b: str, c: str }, required: ["a"] }),
	strict("refuses-without-required", { type: "object", properties: { only: str } }),
	strict("refuses-in-items", {
		type: "object",
		properties: { list: { type: "array", items: { type: "object", properties: { a: str } } } },
		required: ["list"],
	}),
	strict("refuses-nested-property", {
		type: "object",
		properties: {
			outer: {
				type: "object",
				properties: { inner: { type: "object", properties: { leaf: str } } },
				required: ["inner"],
			},
		},
		required: ["outer"],
	}),
	strict("refuses-ref-beside-properties", { type: "object", properties: { a: str }, required: ["a"], $ref: "#" }),
	strict("refuses-optional-property-name-with-backtick", { type: "object", properties: { "a`b": str } }),
];
