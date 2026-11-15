import { MAX_ISSUES, MAX_SCHEMA_DEPTH, MAX_STEPS, MAX_VALIDATE_DEPTH } from "@nyaaai/contract/strictSchema";
import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Functions & Helpers

/** Preserves raw JSON. */
class Raw {
	constructor(readonly text: string) {}
}

const raw = (text: string): Raw => new Raw(text);

const normalizes = (name: string, ...schemas: unknown[]): CaseSource => ({
	kind: "strictSchema.normalize",
	name,
	input: { schemas },
});

const problems = (name: string, ...schemas: unknown[]): CaseSource => ({
	kind: "strictSchema.problems",
	name,
	input: { schemas },
});

const validates = (name: string, schema: unknown, ...values: unknown[]): CaseSource => ({
	kind: "strictSchema.validate",
	name,
	input: { schema, texts: values.map((value) => (value instanceof Raw ? value.text : JSON.stringify(value))) },
});

const validatesLists = (name: string, schema: unknown, item: string, ...counts: number[]): CaseSource => ({
	kind: "strictSchema.validateList",
	name,
	input: { schema, item, counts },
});

const chain = (count: number, end: string): Raw => raw(`${'{"next":'.repeat(count)}${end}${"}".repeat(count)}`);

function nested(depth: number): unknown {
	let held: unknown = { type: "null" };
	for (let at = 0; at < depth; at += 1) held = { items: held };
	return held;
}

////////////////////////////////
//  Constants

const leaf = { type: "object", required: ["count"], properties: { count: { type: "integer" } } };

/** Reads input each lap. */
const tree = {
	type: "object",
	properties: { next: { anyOf: [{ type: "null" }, { $ref: "#/$defs/Node" }] } },
	required: ["next"],
};

const link = { type: "object", properties: { next: { $ref: "#/$defs/Link" } } };

/** Recursive branches overlap. */
const twice = {
	anyOf: [
		{ type: "object", properties: { next: { $ref: "#/$defs/Twice" } }, required: ["next"] },
		{ type: "object", properties: { next: { $ref: "#/$defs/Twice" } }, additionalProperties: false },
	],
};

const referenceChain = Object.fromEntries(
	Array.from({ length: MAX_VALIDATE_DEPTH + 2 }, (_, at) => [
		`A${at}`,
		at > MAX_VALIDATE_DEPTH ? { type: "null" } : { $ref: `#/$defs/A${at + 1}` },
	]),
);

////////////////////////////////
//  Cases

export const strictSchemaCases: CaseSource[] = [
	normalizes(
		"keywords",
		{
			$schema: "dialect",
			type: "object",
			properties: {
				url: { type: "string", format: "uri", pattern: "^https" },
				rows: { type: "array", items: { pattern: "x", type: "string" } },
				pattern: { type: "string", pattern: "x" },
				format: { enum: ["a"], format: "f" },
				$schema: { type: "null", $schema: "inner" },
			},
			required: ["pattern", "format"],
		},
		{
			additionalProperties: false,
			description: "kept",
			examples: [{ pattern: "data" }],
			default: { format: "data" },
			const: { pattern: 1 },
			enum: [{ format: 2, $schema: 3 }],
		},
		{
			$defs: { Name: { pattern: "x", type: "string" } },
			anyOf: [{ format: "f" }],
			oneOf: [{ pattern: "p" }],
			propertyNames: { pattern: "^a" },
			additionalProperties: { format: "x" },
			items: { pattern: "y" },
		},
		{ allOf: [{ pattern: "kept" }], not: { format: "kept" } },
		{ properties: { "10": { format: "a" }, b: {}, "2": { pattern: "b" } }, format: "x", type: "object" },
		true,
		null,
		"text",
		[{ pattern: "x" }],
		{ properties: [{ pattern: "x" }], anyOf: { format: "x" }, $defs: "x" },
		nested(MAX_SCHEMA_DEPTH + 2),
	),
	normalizes(
		"root-reference",
		{
			anyOf: [
				{ type: "null" },
				{ type: "array", items: { $ref: "#" } },
				{ type: "object", additionalProperties: { $ref: "#" } },
			],
		},
		{
			type: "object",
			properties: { next: { $ref: "#", description: "kept" } },
			$defs: { Root: { type: "null" }, Root_: { type: "string" } },
			description: "moved",
		},
		{ $ref: "#" },
		{ properties: { a: { const: { $ref: "#" } } } },
		{ $ref: "#", $defs: [] },
		{ properties: { "10": { $ref: "#" }, b: {}, "2": {} }, type: "object" },
		{ allOf: [{ $ref: "#" }] },
	),
	problems(
		"keywords",
		{ anyOf: [{ type: "string" }] },
		{ allOf: [{ type: "string" }] },
		{ const: null },
		{ not: { const: null } },
		JSON.parse('{"if":{"type":"string"},"then":{"type":"string"},"else":{}}'),
		{ items: { type: "string" } },
		{ prefixItems: [{ type: "string" }] },
		{ uniqueItems: true },
		{ multipleOf: 2 },
		{ properties: { value: { type: "string" } } },
		{ patternProperties: { "^x": { type: "string" } } },
		{ contains: { type: "string" } },
		{ dependentRequired: { a: ["b"] } },
		{ unevaluatedProperties: false },
		{ nullable: true },
		{ definitions: { Name: { type: "string" } } },
		{ $id: "local" },
		{ $anchor: "local" },
		{ $dynamicRef: "#node" },
		{ unknownThing: true },
		{ description: 1, title: [], default: { allOf: [] }, examples: "x" },
		{ properties: { pattern: { type: "string", pattern: "x" }, format: { format: "uri" } }, $schema: "x" },
	),
	problems(
		"forms",
		true,
		false,
		null,
		[],
		{},
		{ type: [] },
		{ type: ["number", "number"] },
		{ type: ["text"] },
		{ type: ["null", 1] },
		{ type: 1 },
		{ type: ["null", "array"] },
		{ enum: [] },
		{ enum: [null] },
		{ enum: "a" },
		{ required: ["a", "a"] },
		{ required: ["a", 1] },
		{ required: ["a", "b"] },
		{ required: "a" },
		{ properties: [] },
		{ properties: { child: false } },
		{ items: true },
		{ items: {} },
		{ propertyNames: false },
		{ propertyNames: { type: "string" } },
		{ anyOf: [] },
		{ anyOf: [true] },
		{ anyOf: {} },
		{ oneOf: [{}] },
		{ additionalProperties: null },
		{ additionalProperties: true },
		{ additionalProperties: false },
		{ additionalProperties: { type: "integer" } },
		{ minimum: 1 },
		{ minimum: "1" },
		{ maximum: 1.5 },
		{ exclusiveMinimum: null },
		{ exclusiveMaximum: 2 },
		{ minLength: 0 },
		{ minLength: -1 },
		{ maxLength: 1.5 },
		{ minItems: 2 },
		{ maxItems: -1 },
	),
	problems(
		"references",
		{ $defs: { Good: { type: "null" } } },
		{ $defs: [] },
		{ properties: { inner: { $defs: {} } } },
		{ $ref: "#/$defs/Missing" },
		{ $ref: "#/$defs/" },
		{ $ref: "#/$defs/Good/properties", $defs: { Good: {} } },
		{ $ref: 5, $defs: { "5": {} } },
		{ $ref: "#/$defs/Good", $defs: { Good: { type: "string" } } },
		{ $ref: "#/$defs/Good", $defs: { Good: { type: "string" } }, description: "allowed" },
		{ $ref: "#/$defs/Good", $defs: { Good: { type: "string" } }, type: "string" },
		{ $ref: "#/$defs/toString", $defs: {} },
		{
			$defs: { Good: { type: "string" } },
			properties: { value: { $ref: "#/$defs/Good", description: "allowed" } },
		},
		{
			$defs: { Good: { type: "string" } },
			properties: { value: { $ref: "#/$defs/Good", minLength: 1 } },
		},
		{
			$defs: {
				Good: { type: "string" },
				Alias: { $ref: "#/$defs/Good" },
				Holder: { properties: { value: { $ref: "#/$defs/Alias" } } },
				LoopA: { $ref: "#/$defs/LoopB" },
				LoopB: { anyOf: [{ $ref: "#/$defs/LoopA" }, { type: "null" }] },
				Self: { oneOf: [{ $ref: "#/$defs/Self" }] },
				IntoLoop: { $ref: "#/$defs/LoopA" },
				Node: tree,
			},
			properties: { value: { $ref: "#/$defs/Holder" } },
		},
		// Root refs trigger naming.
		{ anyOf: [{ type: "null" }, { type: "array", items: { $ref: "#" } }] },
		{ $ref: "#" },
		{ anyOf: [{ $ref: "#" }, { type: "null" }] },
	),
	problems("depth", nested(MAX_SCHEMA_DEPTH - 1), nested(MAX_SCHEMA_DEPTH), nested(MAX_SCHEMA_DEPTH + 1)),
	validates(
		"object-order",
		{
			type: "object",
			required: ["missingB", "missingA"],
			propertyNames: { minLength: 2 },
			properties: {
				"0": { type: "string" },
				"2": { type: "string" },
				"10": { type: "string" },
				constructor: { type: "number" },
				toString: { type: "boolean" },
				["__proto__"]: { const: "okay" },
				rows: { type: "array", items: leaf },
			},
			additionalProperties: false,
		},
		raw(
			'{"10":false,"2":false,"0":false,"constructor":"first","constructor":"bad","toString":1,"__proto__":"bad","rows":[{"count":"bad","extra":1},{}],"extra":1,"x":2}',
		),
		raw('{"2":"two","10":"ten","constructor":1,"toString":true,"__proto__":"okay","rows":[{"count":1}]}'),
		raw('{"missingA":1,"missingB":2,"missingA":3}'),
	),
	validates(
		"prototype-keys",
		{ type: "object", additionalProperties: false },
		raw('{"10":1,"2":2,"0":0,"constructor":3,"toString":4,"__proto__":5}'),
		{},
	),
	validates(
		"inherited-names",
		{
			type: "object",
			required: ["constructor", "toString", "__proto__", "hasOwnProperty"],
			properties: { valueOf: { type: "string" } },
		},
		{},
		raw('{"__proto__":{"constructor":1},"valueOf":1}'),
	),
	validates("additional-true", { type: "object", additionalProperties: true }, { extra: 1 }),
	validates(
		"additional-schema",
		{ type: "object", additionalProperties: { type: "integer" } },
		{ extra: "bad", fine: 1 },
	),
	validates("additional-absent", { type: "object" }, { extra: 1 }),
	validates(
		"property-names",
		{
			propertyNames: { maxLength: 2, enum: ["ab", "\u{1F600}\u{1F600}", "abc"] },
			additionalProperties: { type: "null" },
		},
		{ ab: null, abc: null, "\u{1F600}\u{1F600}": 1, zz: null },
	),
	validates(
		"references",
		{
			type: "object",
			$defs: { Count: { type: "integer", minimum: 2 }, Alias: { $ref: "#/$defs/Count", description: "beside" } },
			properties: { value: { $ref: "#/$defs/Alias" } },
		},
		{ value: 1 },
		{ value: 2 },
		{ value: "2" },
	),
	validates(
		"root-reference",
		{ $ref: "#/$defs/Node", $defs: { Node: tree } },
		{ next: null },
		{ next: { next: { next: null } } },
		{ next: { next: {} } },
		{},
		null,
	),
	validates("outside-the-subset", { $ref: "#/$defs/Missing", type: "string" }, 1),
	validates(
		"number-bounds",
		{ minimum: 1, exclusiveMinimum: 1, maximum: 2, exclusiveMaximum: 2 },
		0,
		1,
		1.5,
		2,
		3,
		raw("-0"),
		raw("5e-324"),
		raw("9007199254740993"),
		raw("1e400"),
		raw("-1e400"),
		"1",
	),
	validates("bound-spelling", { minimum: 1e21, maximum: -1e-7, exclusiveMinimum: 0.1, minLength: 3 }, 0, "ab"),
	validates(
		"types",
		{ type: ["integer", "null"] },
		null,
		1,
		raw("1.0"),
		raw("-0"),
		1.5,
		"1",
		[],
		{},
		true,
		raw("5e-324"),
		raw("9007199254740993"),
		raw("1e300"),
		raw("1e400"),
	),
	validates("number-types", { type: "number" }, 1, raw("1.0"), raw("-0"), raw("1e400"), raw("5e-324"), "1", null),
	validates("list-against-object", { type: "object" }, [], {}, null),
	validates("object-against-list", { type: "array" }, [], {}, "[]"),
	validates(
		"unicode-bounds",
		{ minLength: 2, maxLength: 1 },
		"",
		"x",
		"\u{1F600}",
		"e\u{301}",
		"\u{1F600}\u{1F600}",
		raw('"\\ud83d"'),
		raw('"\\ud83d\\ude00"'),
		raw('"\\ude00\\ud83d"'),
		1,
	),
	validates("list-bounds", { minItems: 2, maxItems: 1, items: { type: "integer" } }, [], ["bad"], [1, "bad"], [1, 2]),
	validates(
		"on-the-bounds",
		{ minItems: 1, maxItems: 2, minLength: 1, maxLength: 2 },
		[],
		[1],
		[1, 2],
		[1, 2, 3],
		"",
		"a",
		"ab",
		"abc",
	),
	validates(
		"equality",
		{ enum: [1, { a: 1, b: [null, "x"] }, null, [1, [2]], "1"] },
		raw("1.0"),
		raw("1e0"),
		{ b: [null, "x"], a: 1 },
		null,
		[1, [2]],
		[[2], 1],
		2,
		true,
		"1",
		{ a: 1 },
	),
	validates(
		"const",
		{ const: { a: [1, null], b: "x" } },
		raw('{"b":"x","a":[1.0,null]}'),
		{ a: [1, "null"], b: "x" },
		raw('{"a":[1,null],"b":"x","b":"x"}'),
	),
	validates("const-null", { const: null }, null, 0, false, "", raw("1e400")),
	validates(
		"overflowed-numbers",
		{ enum: [null, [null], { a: null }, 0] },
		raw("1e400"),
		raw("-1e400"),
		raw("[1e400]"),
		raw('{"a":1e400}'),
		null,
		[null],
		{ a: null },
		raw("-0"),
	),
	validates(
		"unions",
		{
			anyOf: [{ type: "string" }, { anyOf: [{ type: "integer" }, { const: false }] }],
			oneOf: [{ type: "string" }, { type: "number" }, { minimum: 5 }],
		},
		null,
		"x",
		1,
		7,
		false,
		1.5,
	),
	validates(
		"tagged-union",
		{
			oneOf: [
				{
					type: "object",
					properties: { kind: { const: "cat" }, lives: { type: "integer" } },
					required: ["kind", "lives"],
				},
				{
					type: "object",
					properties: { kind: { const: "dog" }, bark: { type: "boolean" } },
					required: ["kind", "bark"],
				},
			],
		},
		{ kind: "cat", lives: 9 },
		{ kind: "dog", bark: true },
		{ kind: "bird" },
		{ kind: "cat", lives: 9, bark: true },
	),
	validates(
		"issue-order",
		{
			type: "object",
			required: ["a", "b"],
			properties: {
				rows: { type: "array", minItems: 3, items: leaf },
				name: { type: "string", minLength: 3, maxLength: 1, enum: ["abc"], const: "abc" },
				size: { minimum: 5, maximum: 1, enum: [9], anyOf: [{ type: "string" }], oneOf: [{}, {}] },
			},
			additionalProperties: false,
		},
		{ size: 3, extra: 1, name: "zz", rows: [{}, { count: 1.5 }], other: [] },
	),
	validates(
		"recursive",
		{ ...tree, $defs: { Node: tree } },
		chain(50, "null"),
		chain(50, '"bad"'),
		chain(84, "null"),
		chain(85, "null"),
		chain(86, "null"),
	),
	validates(
		"depth-limit",
		{ $ref: "#/$defs/Link", $defs: { Link: link } },
		chain(MAX_VALIDATE_DEPTH / 2 - 2, "{}"),
		chain(MAX_VALIDATE_DEPTH / 2 - 1, "{}"),
		chain(MAX_VALIDATE_DEPTH / 2, "{}"),
		chain(MAX_VALIDATE_DEPTH / 2 + 1, "{}"),
		chain(MAX_VALIDATE_DEPTH / 2 - 1, "1"),
		chain(1000, "{}"),
	),
	validates("reference-chain", { $ref: "#/$defs/A0", $defs: referenceChain }, null, 1),
	validates(
		"limit-inside-a-probe",
		{
			oneOf: [{ $ref: "#/$defs/Link" }, { type: "object" }],
			anyOf: [{ $ref: "#/$defs/Link" }, {}],
			$defs: { Link: link },
		},
		chain(4, "{}"),
		chain(MAX_VALIDATE_DEPTH / 2 + 1, "{}"),
		1,
	),
	validates(
		"limit-inside-a-name-probe",
		{ propertyNames: { $ref: "#/$defs/A0" }, $defs: referenceChain },
		{},
		{ a: 1 },
	),
	validates("doubling-unions", { $ref: "#/$defs/Twice", $defs: { Twice: twice } }, chain(8, "1"), chain(30, "1")),
	validatesLists("issue-cap", { items: { type: "string" } }, "0", MAX_ISSUES - 1, MAX_ISSUES, MAX_ISSUES + 1, 1000),
	validatesLists("step-limit", { items: { type: "integer" } }, "0", MAX_STEPS - 2, MAX_STEPS - 1, MAX_STEPS),
	validatesLists(
		"step-limit-in-probes",
		{ items: { anyOf: [{ type: "string" }, { type: "integer" }] } },
		"0",
		(MAX_STEPS - 1) / 3,
		(MAX_STEPS - 1) / 3 + 1,
	),
	validatesLists(
		"step-limit-after-an-issue",
		{ minItems: 2 * MAX_STEPS, items: { type: "integer" } },
		"0",
		5,
		MAX_STEPS,
	),
];
