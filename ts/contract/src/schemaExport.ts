// Zod schema export.

import { z } from "zod";
import { CONTRACT_VERSION, type Hook, hooks, type ShapeSet, shapeIds, shapeSets } from "./shape";

////////////////////////////////
//  Interfaces & Types

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

/** Raw input; decoded output. */
export type SchemaForm = "input" | "output";

export interface SetSchema {
	$schema: string;
	$id: string;
	"x-contract-version": string;
	$defs: Record<string, Json>;
}

export interface Exported {
	docs: Map<string, Record<SchemaForm, SetSchema>>;
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
	items?: Schema[];
	rest?: Schema | null;
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
	"tuple",
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

/** Rust derives its traits for tuples this long. */
const MAX_PLACES = 12;

////////////////////////////////
//  Functions & Helpers

/** ECMAScript orders indices first. */
function isIndexKey(key: string): boolean {
	if (key.length === 0 || (key[0] === "0" && key.length > 1)) return false;
	for (let at = 0; at < key.length; at++) {
		if (key.charCodeAt(at) < 0x30 || key.charCodeAt(at) > 0x39) return false;
	}
	return true;
}

/** Word characters from `from`, the next index. */
function wordEnd(text: string, from: number): number {
	let end = from;
	for (; end < text.length; end++) {
		const unit = text.charCodeAt(end);
		const word =
			(unit >= 0x30 && unit <= 0x39) ||
			(unit >= 0x41 && unit <= 0x5a) ||
			unit === 0x5f ||
			(unit >= 0x61 && unit <= 0x7a);
		if (!word) break;
	}
	return end;
}

export interface SchemaRef {
	set?: string;
	form?: "input" | "output";
	name: string;
}

/** Reads `#/$defs/Name` or `<set>.<form>.json#/$defs/Name`. */
export function refOf(target: string): SchemaRef | undefined {
	const defs = "#/$defs/";
	let at = 0;
	let set: string | undefined;
	let form: "input" | "output" | undefined;
	if (!target.startsWith(defs)) {
		const end = wordEnd(target, 0);
		if (end === 0) return undefined;
		set = target.slice(0, end);
		const suffix = target.startsWith(".input.json", end)
			? "input"
			: target.startsWith(".output.json", end)
				? "output"
				: undefined;
		if (suffix === undefined) return undefined;
		form = suffix;
		at = end + `.${suffix}.json`.length;
	}
	if (!target.startsWith(defs, at)) return undefined;
	const from = at + defs.length;
	const end = wordEnd(target, from);
	if (end === from || end !== target.length) return undefined;
	const name = target.slice(from, end);
	return set === undefined || form === undefined ? { name } : { set, form, name };
}

const defOf = (schema: Schema) => schema._zod.def as unknown as Def;

/** A pipe, under any nullable. */
function readsMissing(schema: Schema): boolean {
	let def = defOf(schema);
	while (def.type === "nullable" && def.innerType) def = defOf(def.innerType);
	return def.type === "pipe";
}

function faults(schema: Schema, slot: Slot): string[] {
	const def = defOf(schema);
	const held = hooks.get(schema)?.list ?? [];
	const found: string[] = [];
	if (!TYPES.has(def.type)) return [`${def.type} is outside the dialect`];
	if ((def.type === "optional" || def.type === "default") && slot !== "property") {
		found.push(`${def.type} sits only on an object property, outermost`);
	}
	if (def.type === "number" && def.format !== undefined && def.format !== "safeint") {
		found.push(`number format ${def.format} is outside the dialect`);
	}
	// Zod checks a string format itself.
	if (def.type === "string" && def.format !== undefined) {
		found.push(`string format ${def.format} is outside the dialect`);
	}
	if (def.type === "object") {
		if (def.catchall !== undefined) found.push("an object keeps no unknown keys; use a record");
		for (const key of Object.keys(def.shape ?? {})) {
			if (key === "__proto__" || isIndexKey(key)) found.push(`property ${key} would move or vanish`);
		}
	}
	if (def.type === "tuple") {
		const places = def.items ?? [];
		if (def.rest) found.push("a tuple holds no rest");
		if (places.length === 0) found.push("a tuple holds an item");
		if (places.length > MAX_PLACES) found.push(`a tuple holds at most ${MAX_PLACES} places`);
		// Zod lets a pipe's place go missing.
		if (places.some(readsMissing)) found.push("a tuple place holds no preprocess");
	}
	if (def.type === "record") {
		const key = def.keyType ? defOf(def.keyType) : undefined;
		if (key?.type !== "string" || key.format !== undefined || (key.checks?.length ?? 0) > 0) {
			found.push("a record's keys are any text");
		}
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
	if (def.type === "pipe" && (held[0]?.kind !== "preprocess" || (def.in && defOf(def.in).type) !== "transform")) {
		found.push("a pipe must be a registered preprocess");
	}
	if (def.type === "nullable" && def.innerType && hooks.get(def.innerType) && !shapeIds.has(def.innerType)) {
		found.push("a hook wraps nullable, not the reverse");
	}
	if (def.type === "default" && def.innerType) {
		const inner = z.safeParse(def.innerType, def.defaultValue);
		if (!inner.success || JSON.stringify(inner.data) !== JSON.stringify(def.defaultValue)) {
			found.push("a default must be a decoded value");
		}
	}
	const checks = (def.checks ?? []).map((check) => check._zod.def.check);
	for (const name of checks) {
		if (name !== "custom" && !CHECKS[def.type]?.has(name)) found.push(`check ${name} is outside the dialect`);
	}
	const hooked = checks.indexOf("custom");
	if (hooked !== -1 && checks.slice(hooked).some((name) => name !== "custom")) {
		found.push("a bound sits before a hook");
	}
	const custom = checks.filter((name) => name === "custom").length;
	if (custom !== held.filter((hook) => hook.kind !== "preprocess").length) {
		found.push("a refinement must be registered");
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
		case "tuple":
			return (def.items ?? []).map((item, at) => [`[${at}]`, item, "value"]);
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

export function dialectProblems(sets: ShapeSet[] = shapeSets()): string[] {
	const problems: string[] = [];
	for (const set of sets) {
		for (const [key, schema] of Object.entries(set.shapes)) {
			walk(schema, `${set.name}.${key}`, "value", new Set(), problems);
		}
	}
	return problems;
}

function annotate(schema: Schema, json: Record<string, unknown>): void {
	const held = hooks.get(schema)?.list;
	if (held) json["x-hooks"] = [...((json["x-hooks"] as Hook[] | undefined) ?? []), ...held];
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
