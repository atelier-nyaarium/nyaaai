// Zod schema export.

import { z } from "zod";
import { tryPascal, trySnake, tryVariant } from "./rustNames";
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
	checks?: Array<{ _zod: { def: { check: string; value?: unknown } } }>;
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

/** A union that is a nullable. */
function isNullable(def: Def): boolean {
	const [, second] = def.options ?? [];
	return (
		(def.options ?? []).length === 2 && second !== undefined && defOf(second).type === "null" && !hooks.get(second)
	);
}

/** A branch tag: text, one value each. */
function tagFaults(def: Def, tag: string): string[] {
	const found: string[] = [];
	const seen = new Map<string, string>();
	(def.options ?? []).forEach((option, at) => {
		const branch = `branch ${at}`;
		const held = defOf(option);
		const place = held.type === "object" ? held.shape?.[tag] : undefined;
		const kind = place ? defOf(place) : undefined;
		if (kind?.type === "enum") {
			found.push(`${branch}: the tag ${tag} is an enum; write one branch per value, each a one-value literal`);
		} else if (kind?.type !== "literal" || (kind.values ?? []).length !== 1) {
			found.push(`${branch}: each branch is an object with a required text tag ${tag}`);
		} else {
			const [value] = kind.values ?? [];
			if (typeof value !== "string") {
				found.push(`${branch}: the tag ${tag} is not text; write a plain union, not a discriminated one`);
			} else {
				const variant = tryVariant(value);
				const twin = variant === undefined ? undefined : seen.get(variant);
				if (variant === undefined) {
					found.push(
						`${branch}: the tag ${JSON.stringify(value)} comes to no Rust variant name; use other values`,
					);
				} else if (twin === value) {
					found.push(`two branches share the tag ${JSON.stringify(value)}`);
				} else if (twin !== undefined) {
					found.push(
						`tags ${JSON.stringify(twin)} and ${JSON.stringify(value)} both come to the variant ${variant}`,
					);
				} else {
					seen.set(variant, value);
				}
			}
		}
	});
	return found;
}

/** Texts that come to no variant name, or to one. */
function enumFaults(def: Def): string[] {
	const found: string[] = [];
	const seen = new Map<string, string>();
	for (const value of Object.keys(def.entries ?? {})) {
		const variant = tryVariant(value);
		const twin = variant === undefined ? undefined : seen.get(variant);
		if (variant === undefined) {
			found.push(`the value ${JSON.stringify(value)} comes to no Rust variant name; use other values`);
		} else if (twin !== undefined) {
			found.push(
				`values ${JSON.stringify(twin)} and ${JSON.stringify(value)} both come to the variant ${variant}`,
			);
		} else {
			seen.set(variant, value);
		}
	}
	return found;
}

/** The generator's first name for a union branch; numbered after twins. */
function draftOf(schema: Schema): string | undefined {
	const named = shapeIds.get(schema)?.id;
	if (named !== undefined) return named.slice(named.indexOf(".") + 1);
	const def = defOf(schema);
	switch (def.type) {
		case "string":
			return "Text";
		case "number":
			return def.format === "safeint" ? "Integer" : "Number";
		case "boolean":
			return "Flag";
		case "null":
			return "Null";
		case "unknown":
			return "Any";
		case "literal": {
			const [value] = def.values ?? [];
			return tryVariant(String(value));
		}
		case "enum":
			return "Choice";
		case "array":
			return "List";
		case "tuple":
			return "Tuple";
		case "record":
			return "Record";
		case "object":
			return "Object";
		case "nullable":
			return def.innerType === undefined ? undefined : draftOf(def.innerType);
		case "union":
			return "Either";
		default:
			return undefined;
	}
}

/** Branches that come to one variant name once twins are numbered. */
function twinFaults(def: Def): string[] {
	const drafts = (def.options ?? []).map(draftOf);
	const finals = drafts.map((draft, at) => {
		if (draft === undefined) return undefined;
		const twins = drafts.filter((held) => held === draft).length;
		return twins > 1 ? `${draft}${drafts.slice(0, at + 1).filter((held) => held === draft).length}` : draft;
	});
	const found: string[] = [];
	const seen = new Map<string, number>();
	finals.forEach((final, at) => {
		if (final === undefined) return;
		const twin = seen.get(final);
		if (twin !== undefined) found.push(`branches ${twin} and ${at} both come to the variant ${final}`);
		else seen.set(final, at);
	});
	return found;
}

/** A literal branch names its variant. */
function branchFaults(def: Def): string[] {
	if (isNullable(def)) return [];
	const found: string[] = [...twinFaults(def)];
	(def.options ?? []).forEach((option, at) => {
		let branch = option;
		let held = defOf(branch);
		while (!shapeIds.has(branch) && held.type === "nullable" && held.innerType) {
			branch = held.innerType;
			held = defOf(branch);
		}
		// A named shape names it.
		if (shapeIds.has(branch)) return;
		const [value] = held.values ?? [];
		if (held.type === "literal" && value !== null && tryVariant(String(value)) === undefined) {
			found.push(
				`branch ${at}: the literal ${JSON.stringify(value)} comes to no Rust variant name; name the shape`,
			);
		}
	});
	return found;
}

/** A property comes to a field; a constant has none. */
function fieldFaults(def: Def): string[] {
	const found: string[] = [];
	const seen = new Map<string, string>();
	for (const [key, place] of Object.entries(def.shape ?? {})) {
		if (key === "__proto__" || isIndexKey(key)) continue;
		const field = trySnake(key);
		const kind = defOf(place).type;
		const constant = (kind === "literal" || kind === "null") && !hooks.get(place)?.list.length;
		const twin = field === undefined ? undefined : seen.get(field);
		if (field === undefined) found.push(`property ${key} comes to no Rust field name`);
		else if (!constant && twin !== undefined)
			found.push(`properties ${twin} and ${key} both come to the field ${field}`);
		else if (!constant) seen.set(field, key);
	}
	return found;
}

function faults(schema: Schema, slot: Slot): string[] {
	const def = defOf(schema);
	const held = hooks.get(schema)?.list ?? [];
	const found: string[] = [];
	if (!TYPES.has(def.type)) return [`${def.type} is outside the dialect`];
	if (z.globalRegistry.has(schema)) found.push("describe and meta are outside the dialect; write a comment");
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
	if (def.type === "object") found.push(...fieldFaults(def));
	if (def.type === "enum") found.push(...enumFaults(def));
	if (def.type === "union" && def.discriminator !== undefined) found.push(...tagFaults(def, def.discriminator));
	if (def.type === "union" && def.discriminator === undefined) found.push(...branchFaults(def));
	if (held.some((hook) => hook.kind === "match") && def.type !== "string") found.push("a pattern checks text");
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
	if (def.type === "number" && def.format === "safeint") {
		for (const check of def.checks ?? []) {
			const bound = check._zod.def.value;
			if (typeof bound === "number" && !Number.isInteger(bound))
				found.push(`an integer bound of ${bound} is whole`);
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
function walk(
	schema: Schema,
	where: string,
	slot: Slot,
	open: Set<Schema>,
	problems: string[],
	names: Map<string, string>,
): void {
	if (open.has(schema)) {
		problems.push(`${where}: recursion must pass through a named shape`);
		return;
	}
	for (const fault of faults(schema, slot)) problems.push(`${where}: ${fault}`);
	for (const hook of hooks.get(schema)?.list ?? []) {
		if (!("name" in hook)) continue;
		const rust = trySnake(hook.name);
		const twin = rust === undefined ? undefined : names.get(rust);
		if (rust === undefined) problems.push(`${where}: hook ${hook.name} comes to no Rust name`);
		else if (twin !== undefined && twin !== hook.name) {
			problems.push(`${where}: hooks ${twin} and ${hook.name} both come to ${rust} in Rust; rename one`);
		} else names.set(rust, hook.name);
	}
	const def = defOf(schema);
	if (!TYPES.has(def.type)) return;
	open.add(schema);
	for (const [step, child, childSlot] of children(def)) {
		if (!shapeIds.has(child)) walk(child, `${where}${step}`, childSlot, open, problems, names);
	}
	open.delete(schema);
}

/** An object, an enum or a union is a type of its own. */
function isItem(schema: Schema): boolean {
	let def = defOf(schema);
	while ((def.type === "lazy" && def.getter) || (def.type === "pipe" && def.out)) {
		def = defOf(def.type === "lazy" ? (def.getter as () => Schema)() : (def.out as Schema));
	}
	if (def.type === "union") return !isNullable(def);
	return def.type === "object" || def.type === "enum";
}

/** Named shapes an alias reaches without passing through a type of its own. */
function aliasNext(schema: Schema, root: boolean, found: string[]): void {
	const named = shapeIds.get(schema);
	if (!root && named) {
		found.push(named.id);
		return;
	}
	const def = defOf(schema);
	const next: Array<Schema | undefined> =
		def.type === "array"
			? [def.element]
			: def.type === "record"
				? [def.valueType]
				: def.type === "nullable"
					? [def.innerType]
					: def.type === "tuple"
						? (def.items ?? [])
						: def.type === "lazy"
							? [def.getter?.()]
							: def.type === "pipe"
								? [def.out]
								: def.type === "union" && isNullable(def)
									? [(def.options ?? [])[0]]
									: [];
	for (const child of next) if (child) aliasNext(child, false, found);
}

/** A type alias cannot name itself, through other aliases either. */
function aliasProblems(sets: ShapeSet[]): string[] {
	const aliases = new Map<string, Schema>();
	for (const set of sets) {
		for (const [key, schema] of Object.entries(set.shapes)) {
			if (!isItem(schema)) aliases.set(`${set.name}.${key}`, schema);
		}
	}
	const reaches = (from: string, to: string, seen: Set<string>): boolean => {
		const schema = aliases.get(from);
		if (!schema || seen.has(from)) return false;
		seen.add(from);
		const next: string[] = [];
		aliasNext(schema, true, next);
		return next.some((held) => held === to || reaches(held, to, seen));
	};
	return [...aliases.keys()]
		.filter((id) => reaches(id, id, new Set()))
		.map((id) => `${id}: a recursive shape is an object or a union; wrap this one in an object`);
}

export function dialectProblems(sets: ShapeSet[] = shapeSets()): string[] {
	const problems: string[] = [];
	for (const set of sets) {
		const names = new Map<string, string>();
		for (const [key, schema] of Object.entries(set.shapes)) {
			if (tryPascal(key) !== key) problems.push(`${set.name}.${key}: a shape name is PascalCase`);
			walk(schema, `${set.name}.${key}`, "value", new Set(), problems, names);
		}
	}
	return [...problems, ...aliasProblems(sets)];
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
