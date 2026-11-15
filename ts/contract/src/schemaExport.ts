// Zod schema export.

import { z } from "zod";
import { CONTRACT_VERSION, type Hook, hooks, type ShapeSet, shapeIds, shapeSets } from "./shape";

////////////////////////////////
//  Interfaces & Types

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

/** Peer input, decoded output. */
export type SchemaForm = "input" | "output";

export interface SetSchema {
	$schema: string;
	$id: string;
	"x-contract-version": string;
	$defs: Record<string, Json>;
}

export interface Exported {
	docs: Map<string, Record<SchemaForm, SetSchema>>;
	/** Unsupported shapes. */
	problems: string[];
}

/** Loose Zod definition. */
interface Def {
	type: string;
	checks?: Array<{ _zod: { def: { check: string } } }>;
	format?: string;
	catchall?: unknown;
	shape?: Record<string, Schema>;
	element?: Schema;
	keyType?: Schema;
	valueType?: Schema;
	options?: Schema[];
	discriminator?: string;
	inclusive?: boolean;
	innerType?: Schema;
	defaultValue?: unknown;
	getter?: () => Schema;
	in?: Schema;
	out?: Schema;
	values?: unknown[];
	entries?: Record<string, unknown>;
}

type Schema = z.core.$ZodType;

type Slot = "property" | "value";

////////////////////////////////
//  Constants

export const SCHEMA_FORMS: readonly SchemaForm[] = ["input", "output"];

const DIALECT = "https://json-schema.org/draft/2020-12/schema";

const TYPES = new Set([
	"string",
	"number",
	"boolean",
	"null",
	"unknown",
	"literal",
	"enum",
	"array",
	"record",
	"object",
	"union",
	"optional",
	"nullable",
	"default",
	"lazy",
	"pipe",
]);

const CHECKS: Record<string, ReadonlySet<string>> = {
	string: new Set(["min_length", "max_length"]),
	number: new Set(["greater_than", "less_than"]),
	array: new Set(["min_length", "max_length"]),
};

/** ECMAScript orders indices first. */
const INDEX_KEY = /^(?:0|[1-9]\d*)$/;

////////////////////////////////
//  Functions & Helpers

const defOf = (schema: Schema) => schema._zod.def as unknown as Def;

/** Node-local faults. */
function faults(schema: Schema, slot: Slot): string[] {
	const def = defOf(schema);
	const hook = hooks.get(schema);
	const found: string[] = [];
	if (!TYPES.has(def.type)) return [`${def.type} is outside the dialect`];
	if ((def.type === "optional" || def.type === "default") && slot !== "property") {
		found.push(`${def.type} sits only on an object property, outermost`);
	}
	if (def.type === "number" && def.format !== undefined && def.format !== "safeint") {
		found.push(`number format ${def.format} is outside the dialect`);
	}
	if (def.type === "object") {
		if (def.catchall !== undefined) found.push("an object keeps no unknown keys; use a record");
		for (const key of Object.keys(def.shape ?? {})) {
			if (key === "__proto__" || INDEX_KEY.test(key)) found.push(`property ${key} would move or vanish`);
		}
	}
	if (def.type === "record") {
		const key = def.keyType ? defOf(def.keyType) : undefined;
		if (key?.type !== "string" || (key.checks?.length ?? 0) > 0) found.push("a record's keys are any text");
	}
	if (def.type === "literal") {
		const [value, ...rest] = def.values ?? [];
		const plain = value === null || ["string", "number", "boolean"].includes(typeof value);
		if (rest.length > 0 || !plain) found.push("a literal holds one text, number, flag or null");
	}
	if (def.type === "enum") {
		const plain = Object.entries(def.entries ?? {}).every(([key, value]) => key === value);
		if (!plain) found.push("an enum holds texts");
	}
	if (def.type === "union" && def.discriminator === undefined && def.inclusive === false) {
		found.push("an exclusive union needs a discriminator");
	}
	if (def.type === "pipe" && (hook?.kind !== "preprocess" || (def.in && defOf(def.in).type) !== "transform")) {
		found.push("a pipe must be a registered preprocess");
	}
	if (def.type === "nullable" && def.innerType && hooks.has(def.innerType) && !shapeIds.has(def.innerType)) {
		found.push("a hook wraps nullable, not the reverse");
	}
	if (def.type === "default" && def.innerType) {
		const inner = z.safeParse(def.innerType, def.defaultValue);
		if (!inner.success || JSON.stringify(inner.data) !== JSON.stringify(def.defaultValue)) {
			found.push("a default must be a decoded value");
		}
	}
	for (const check of def.checks ?? []) {
		const { check: name } = check._zod.def;
		if (name === "custom") {
			if (hook?.kind !== "refine") found.push("a refinement must be registered");
		} else if (!CHECKS[def.type]?.has(name)) {
			found.push(`check ${name} is outside the dialect`);
		}
	}
	return found;
}

function children(def: Def): Array<[string, Schema, Slot]> {
	const inner = (step: string, schema: Schema | undefined, slot: Slot = "value"): Array<[string, Schema, Slot]> =>
		schema ? [[step, schema, slot]] : [];
	switch (def.type) {
		case "object":
			return Object.entries(def.shape ?? {}).map(([key, schema]) => [`.${key}`, schema, "property"]);
		case "array":
			return inner("[]", def.element);
		case "record":
			return inner("{}", def.valueType);
		case "union":
			return (def.options ?? []).map((option, at) => [`|${at}`, option, "value"]);
		case "optional":
		case "nullable":
		case "default":
			return inner("", def.innerType);
		case "lazy":
			return inner("", def.getter?.());
		case "pipe":
			return inner("", def.out);
		default:
			return [];
	}
}

/** Stops at named shapes. */
function walk(schema: Schema, where: string, slot: Slot, open: Set<Schema>, problems: string[]): void {
	if (open.has(schema)) {
		problems.push(`${where}: recursion must pass through a named shape`);
		return;
	}
	for (const fault of faults(schema, slot)) problems.push(`${where}: ${fault}`);
	const def = defOf(schema);
	if (!TYPES.has(def.type)) return;
	open.add(schema);
	for (const [step, child, childSlot] of children(def)) {
		if (!shapeIds.has(child)) walk(child, `${where}${step}`, childSlot, open, problems);
	}
	open.delete(schema);
}

/** Lists unsupported shapes. */
export function dialectProblems(sets: ShapeSet[] = shapeSets()): string[] {
	const problems: string[] = [];
	for (const set of sets) {
		for (const [key, schema] of Object.entries(set.shapes)) {
			walk(schema, `${set.name}.${key}`, "value", new Set(), problems);
		}
	}
	return problems;
}

/** Wrappers copy inner annotations. */
function annotate(schema: Schema, json: Record<string, unknown>): void {
	const hook = hooks.get(schema);
	if (hook) json["x-hooks"] = [...((json["x-hooks"] as Hook[] | undefined) ?? []), hook];
	const { discriminator } = defOf(schema);
	if (discriminator !== undefined) json["x-tag"] = discriminator;
}

function localRefs(node: Json, set: string, form: SchemaForm, problems: string[]): Json {
	if (Array.isArray(node)) return node.map((item) => localRefs(item, set, form, problems));
	if (node === null || typeof node !== "object") return node;
	const held: { [key: string]: Json } = {};
	for (const [key, value] of Object.entries(node)) {
		if (key === "$defs") {
			problems.push(`${set}: recursion must pass through a named shape`);
		} else if (key === "$ref" && typeof value === "string") {
			const [other = "", name = ""] = value.split(".");
			held[key] = other === set ? `#/$defs/${name}` : `${other}.${form}.json#/$defs/${name}`;
		} else {
			held[key] = localRefs(value, set, form, problems);
		}
	}
	return held;
}

function docOf(set: ShapeSet, form: SchemaForm, schemas: Record<string, Json>, problems: string[]): SetSchema {
	const $defs: Record<string, Json> = {};
	for (const key of Object.keys(set.shapes)) {
		const { $schema, $id, ...schema } = schemas[`${set.name}.${key}`] as { [key: string]: Json };
		$defs[key] = localRefs(schema, set.name, form, problems);
	}
	return { $schema: DIALECT, $id: `${set.name}.${form}.json`, "x-contract-version": CONTRACT_VERSION, $defs };
}

export function exportSchemas(): Exported {
	const problems = dialectProblems();
	const docs = new Map<string, Record<SchemaForm, SetSchema>>();
	const forms = new Map<SchemaForm, Record<string, Json>>();
	for (const form of SCHEMA_FORMS) {
		const { schemas } = z.toJSONSchema(shapeIds, {
			io: form,
			unrepresentable: "any",
			cycles: "ref",
			reused: "inline",
			uri: (id) => id,
			override: ({ zodSchema, jsonSchema }) => annotate(zodSchema, jsonSchema as Record<string, unknown>),
		});
		forms.set(form, schemas as Record<string, Json>);
	}
	for (const set of shapeSets()) {
		const doc = (form: SchemaForm) => docOf(set, form, forms.get(form) ?? {}, problems);
		docs.set(set.name, { input: doc("input"), output: doc("output") });
	}
	return { docs, problems: [...new Set(problems)] };
}
