// JSON Schema subset implementation.

import { numberText } from "@nyaaai/contract/number";
import type { Json } from "@nyaaai/contract/schemaExport";
import type { AnyShape, PathPart } from "@nyaaai/contract/shape";
import { z } from "zod";
import { ISSUE_MESSAGES, type Issue, type IssueCode } from "./issues";

////////////////////////////////
//  Interfaces & Types

type Schema = { [key: string]: Json };

////////////////////////////////
//  Constants

export const TYPES = new Set(["null", "boolean", "number", "integer", "string", "array", "object"]);
export const VALIDATED = new Set([
	"type",
	"enum",
	"const",
	"minimum",
	"maximum",
	"exclusiveMinimum",
	"exclusiveMaximum",
	"minLength",
	"maxLength",
	"minItems",
	"maxItems",
	"items",
	"properties",
	"required",
	"additionalProperties",
	"propertyNames",
	"anyOf",
	"oneOf",
	"$ref",
	"$defs",
]);
export const ANNOTATIONS = new Set(["description", "title", "default", "examples"]);
export const DROPPED = new Set(["$schema", "pattern", "format"]);

export const MAPS = new Set(["properties", "$defs"]);
export const SINGLES = new Set(["items", "propertyNames", "additionalProperties"]);
export const LISTS = new Set(["anyOf", "oneOf"]);

export const MAX_SCHEMA_DEPTH = 64;

/** Refs count toward nesting. */
export const MAX_VALIDATE_DEPTH = 256;

/** Per-run node limit. */
export const MAX_STEPS = 1_000_000;

export const MAX_ISSUES = 64;

const REF_PREFIX = "#/$defs/";
const ROOT_REF = "#";
const BOUNDS: Array<[keyword: string, code: IssueCode, fails: (value: number, bound: number) => boolean]> = [
	["minimum", "minimum", (value, bound) => value < bound],
	["exclusiveMinimum", "exclusive_minimum", (value, bound) => value <= bound],
	["maximum", "maximum", (value, bound) => value > bound],
	["exclusiveMaximum", "exclusive_maximum", (value, bound) => value >= bound],
];

////////////////////////////////
//  Functions & Helpers

function record(value: Json | undefined): value is Schema {
	return value !== null && typeof value === "object" && !Array.isArray(value);
}

const bare = (): Schema => Object.create(null) as Schema;

function issue(path: PathPart[], code: IssueCode, values: Record<string, string> = {}): Issue {
	let message: string = ISSUE_MESSAGES[code];
	for (const [key, value] of Object.entries(values)) message = message.replaceAll(`{${key}}`, value);
	return { path: [...path], code, message };
}

function refName(value: Json | undefined): string | undefined {
	if (typeof value !== "string" || !value.startsWith(REF_PREFIX)) return undefined;
	const name = value.slice(REF_PREFIX.length);
	return name.length > 0 && !name.includes("/") ? name : undefined;
}

function rootDefs(root: Json): Schema {
	return record(root) && record(root["$defs"]) ? root["$defs"] : bare();
}

const typesOf = (schema: Schema): string[] | undefined =>
	typeof schema["type"] === "string"
		? [schema["type"]]
		: Array.isArray(schema["type"])
			? schema["type"].filter((type) => typeof type === "string")
			: undefined;

/** Transforms before subschema traversal. */
function eachSchema(value: Json, depth: number, change: (node: Schema) => Schema): Json {
	if (!record(value) || depth > MAX_SCHEMA_DEPTH) return value;
	const held = bare();
	for (const [key, child] of Object.entries(change(value))) {
		if (MAPS.has(key) && record(child)) {
			const named = bare();
			for (const [name, item] of Object.entries(child)) named[name] = eachSchema(item, depth + 1, change);
			held[key] = named;
		} else if (SINGLES.has(key)) held[key] = eachSchema(child, depth + 1, change);
		else if (LISTS.has(key) && Array.isArray(child))
			held[key] = child.map((item) => eachSchema(item, depth + 1, change));
		else held[key] = child;
	}
	return held;
}

function schemaNode(value: Json, path: PathPart[], defs: Schema, out: Issue[], depth: number): void {
	if (!record(value)) {
		out.push(issue(path, "bad_keyword", { name: "schema" }));
		return;
	}
	if (depth > MAX_SCHEMA_DEPTH) {
		out.push(issue(path, "limit"));
		return;
	}
	const keys = Object.keys(value);
	const refSibling =
		Object.hasOwn(value, "$ref") && keys.some((key) => VALIDATED.has(key) && key !== "$ref" && key !== "$defs");
	for (const key of keys) {
		const child = value[key] as Json;
		const at = [...path, key];
		const bad = (): void => {
			out.push(issue(at, "bad_keyword", { name: key }));
		};
		if (ANNOTATIONS.has(key)) continue;
		if (!VALIDATED.has(key)) {
			out.push(issue(at, "unknown_keyword", { name: key }));
			continue;
		}
		switch (key) {
			case "$defs":
				if (path.length !== 0 || !record(child)) bad();
				else
					for (const name of Object.keys(child))
						schemaNode(child[name] as Json, [...at, name], defs, out, depth + 1);
				break;
			case "$ref": {
				const name = refName(child);
				if (refSibling || name === undefined || !Object.hasOwn(defs, name))
					out.push(issue(at, "bad_reference"));
				break;
			}
			case "type": {
				const held = typeof child === "string" ? [child] : Array.isArray(child) ? child : [];
				if (
					held.length === 0 ||
					!held.every((item) => typeof item === "string" && TYPES.has(item)) ||
					new Set(held).size !== held.length
				)
					bad();
				break;
			}
			case "enum":
				if (!Array.isArray(child) || child.length === 0) bad();
				break;
			case "minimum":
			case "maximum":
			case "exclusiveMinimum":
			case "exclusiveMaximum":
				if (typeof child !== "number" || !Number.isFinite(child)) bad();
				break;
			case "minLength":
			case "maxLength":
			case "minItems":
			case "maxItems":
				if (typeof child !== "number" || !Number.isInteger(child) || child < 0) bad();
				break;
			case "required":
				if (
					!Array.isArray(child) ||
					!child.every((item) => typeof item === "string") ||
					new Set(child).size !== child.length
				)
					bad();
				break;
			case "properties":
				if (!record(child)) bad();
				else
					for (const name of Object.keys(child))
						schemaNode(child[name] as Json, [...at, name], defs, out, depth + 1);
				break;
			case "items":
			case "propertyNames":
				schemaNode(child, at, defs, out, depth + 1);
				break;
			case "additionalProperties":
				if (typeof child !== "boolean") schemaNode(child, at, defs, out, depth + 1);
				break;
			case "anyOf":
			case "oneOf":
				if (!Array.isArray(child) || child.length === 0) bad();
				else
					child.forEach((branch, index) => {
						schemaNode(branch, [...at, index], defs, out, depth + 1);
					});
				break;
		}
	}
}

/** Finds inputless refs. */
function inputlessRefs(value: Json, defs: Schema, found: Set<string>, depth = 0): void {
	if (!record(value) || depth > MAX_SCHEMA_DEPTH) return;
	const name = refName(value["$ref"]);
	if (name !== undefined && Object.hasOwn(defs, name)) found.add(name);
	for (const keyword of LISTS) {
		const branches = value[keyword];
		if (Array.isArray(branches)) for (const branch of branches) inputlessRefs(branch, defs, found, depth + 1);
	}
}

function hasLoop(start: string, graph: Map<string, Set<string>>): boolean {
	const seen = new Set<string>();
	const pending = [...(graph.get(start) ?? [])];
	for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
		if (next === start) return true;
		if (seen.has(next)) continue;
		seen.add(next);
		pending.push(...(graph.get(next) ?? []));
	}
	return false;
}

/** Key order aside. An overflowed number equals no null. */
function sameJson(left: Json, right: Json): boolean {
	if (Array.isArray(left))
		return (
			Array.isArray(right) &&
			left.length === right.length &&
			left.every((item, at) => sameJson(item, right[at] as Json))
		);
	if (!record(left)) return left === right;
	if (!record(right)) return false;
	const keys = Object.keys(left);
	return (
		keys.length === Object.keys(right).length &&
		keys.every((key) => Object.hasOwn(right, key) && sameJson(left[key] as Json, right[key] as Json))
	);
}

function validType(type: string, value: Json): boolean {
	switch (type) {
		case "null":
			return value === null;
		case "boolean":
			return typeof value === "boolean";
		case "number":
			return typeof value === "number" && Number.isFinite(value);
		case "integer":
			return typeof value === "number" && Number.isInteger(value);
		case "string":
			return typeof value === "string";
		case "array":
			return Array.isArray(value);
		case "object":
			return record(value);
		default:
			return false;
	}
}

/** Lone surrogates count once. */
function codePoints(text: string): number {
	let count = 0;
	for (let at = 0; at < text.length; at += 1) {
		const unit = text.charCodeAt(at);
		const next = text.charCodeAt(at + 1);
		if (unit >= 0xd800 && unit <= 0xdbff && next >= 0xdc00 && next <= 0xdfff) at += 1;
		count += 1;
	}
	return count;
}

class Run {
	readonly issues: Issue[] = [];
	private readonly path: PathPart[] = [];
	private steps = 0;
	private halted = false;
	/** Limit issue path. */
	private limit: PathPart[] | undefined;

	constructor(private readonly defs: Schema) {}

	result(): Issue[] {
		if (this.limit && this.issues.length < MAX_ISSUES) this.issues.push(issue(this.limit, "limit"));
		return this.issues;
	}

	/** Halts checks, including probes. */
	private halt(path: PathPart[]): false {
		this.halted = true;
		this.limit ??= path;
		return false;
	}

	private fails(report: boolean, code: IssueCode, values?: Record<string, string>): boolean {
		if (!report) return true;
		this.issues.push(issue(this.path, code, values));
		if (this.issues.length >= MAX_ISSUES) this.halted = true;
		return this.halted;
	}

	/** Probes suppress issue reporting. */
	check(schema: Json, value: Json, depth: number, report: boolean): boolean {
		if (this.halted) return false;
		this.steps += 1;
		if (this.steps > MAX_STEPS) return this.halt([]);
		if (!record(schema)) return true;
		if (depth > MAX_VALIDATE_DEPTH) return this.halt([...this.path]);
		if (Object.hasOwn(schema, "$ref")) {
			const name = refName(schema["$ref"]);
			if (name === undefined || !Object.hasOwn(this.defs, name)) return true;
			return this.check(this.defs[name] as Json, value, depth + 1, report);
		}
		const types = typesOf(schema);
		if (types !== undefined && !types.some((type) => validType(type, value))) {
			this.fails(report, "type", { types: types.join(" or ") });
			return false;
		}
		let clean = true;
		if (Array.isArray(schema["enum"]) && !schema["enum"].some((member) => sameJson(member, value))) {
			clean = false;
			if (this.fails(report, "enum")) return false;
		}
		if (Object.hasOwn(schema, "const") && !sameJson(schema["const"] as Json, value)) {
			clean = false;
			if (this.fails(report, "const")) return false;
		}
		if (typeof value === "number") {
			for (const [keyword, code, fails] of BOUNDS) {
				const bound = schema[keyword];
				if (typeof bound !== "number" || !fails(value, bound)) continue;
				clean = false;
				if (this.fails(report, code, { bound: numberText(bound) })) return false;
			}
		}
		if (typeof value === "string") {
			const length = codePoints(value);
			if (typeof schema["minLength"] === "number" && length < schema["minLength"]) {
				clean = false;
				if (this.fails(report, "min_length", { bound: numberText(schema["minLength"]) })) return false;
			}
			if (typeof schema["maxLength"] === "number" && length > schema["maxLength"]) {
				clean = false;
				if (this.fails(report, "max_length", { bound: numberText(schema["maxLength"]) })) return false;
			}
		}
		if (Array.isArray(value) && !this.list(schema, value, depth, report)) {
			clean = false;
			if (!report || this.halted) return false;
		}
		if (record(value) && !this.object(schema, value, depth, report)) {
			clean = false;
			if (!report || this.halted) return false;
		}
		return this.unions(schema, value, depth, report) && clean;
	}

	private list(schema: Schema, value: Json[], depth: number, report: boolean): boolean {
		let clean = true;
		if (typeof schema["minItems"] === "number" && value.length < schema["minItems"]) {
			clean = false;
			if (this.fails(report, "min_items", { bound: numberText(schema["minItems"]) })) return false;
		}
		if (typeof schema["maxItems"] === "number" && value.length > schema["maxItems"]) {
			clean = false;
			if (this.fails(report, "max_items", { bound: numberText(schema["maxItems"]) })) return false;
		}
		if (!Object.hasOwn(schema, "items")) return clean;
		for (let index = 0; index < value.length; index += 1) {
			this.path.push(index);
			const held = this.check(schema["items"] as Json, value[index] as Json, depth + 1, report);
			this.path.pop();
			if (held) continue;
			clean = false;
			if (!report || this.halted) return false;
		}
		return clean;
	}

	private object(schema: Schema, value: Schema, depth: number, report: boolean): boolean {
		let clean = true;
		for (const name of Array.isArray(schema["required"]) ? schema["required"] : []) {
			if (typeof name !== "string" || Object.hasOwn(value, name)) continue;
			clean = false;
			this.path.push(name);
			const stops = this.fails(report, "required", { name });
			this.path.pop();
			if (stops) return false;
		}
		for (const [name, item] of Object.entries(value)) {
			this.path.push(name);
			const held = this.member(schema, name, item, depth, report);
			this.path.pop();
			if (held) continue;
			clean = false;
			if (!report || this.halted) return false;
		}
		return clean;
	}

	private member(schema: Schema, name: string, item: Json, depth: number, report: boolean): boolean {
		let clean = true;
		if (Object.hasOwn(schema, "propertyNames")) {
			const named = this.check(schema["propertyNames"] as Json, name, depth + 1, false);
			if (this.halted) return false;
			if (!named) {
				clean = false;
				if (this.fails(report, "property_name")) return false;
			}
		}
		const properties = schema["properties"];
		const additional = schema["additionalProperties"];
		if (record(properties) && Object.hasOwn(properties, name))
			return this.check(properties[name] as Json, item, depth + 1, report) && clean;
		if (additional === false) {
			this.fails(report, "additional", { name });
			return false;
		}
		return record(additional) ? this.check(additional, item, depth + 1, report) && clean : clean;
	}

	private unions(schema: Schema, value: Json, depth: number, report: boolean): boolean {
		let clean = true;
		if (Array.isArray(schema["anyOf"])) {
			let matched = false;
			for (const branch of schema["anyOf"]) {
				matched = this.check(branch, value, depth + 1, false);
				if (this.halted) return false;
				if (matched) break;
			}
			if (!matched) {
				clean = false;
				if (this.fails(report, "no_branch")) return false;
			}
		}
		if (Array.isArray(schema["oneOf"])) {
			let matches = 0;
			for (const branch of schema["oneOf"]) {
				if (this.check(branch, value, depth + 1, false)) matches += 1;
				if (this.halted) return false;
				if (matches > 1) break;
			}
			if (matches !== 1) {
				clean = false;
				if (this.fails(report, matches === 0 ? "no_branch" : "many_branches")) return false;
			}
		}
		return clean;
	}
}

/** Drops keywords and rewrites root refs. */
export function normalizeSchema(schema: Json): Json {
	let rooted = false;
	const dropped = eachSchema(schema, 0, (node) => {
		const held = bare();
		for (const [key, value] of Object.entries(node)) if (!DROPPED.has(key)) held[key] = value;
		if (held["$ref"] === ROOT_REF) rooted = true;
		return held;
	});
	if (!rooted || !record(dropped)) return dropped;
	const defs = Object.hasOwn(dropped, "$defs") ? dropped["$defs"] : bare();
	if (!record(defs)) return dropped;
	let name = "Root";
	while (Object.hasOwn(defs, name)) name += "_";
	const target = `#/$defs/${name}`;
	const named = eachSchema(dropped, 0, (node) => {
		if (node["$ref"] !== ROOT_REF) return node;
		const held = bare();
		for (const [key, value] of Object.entries(node)) held[key] = key === "$ref" ? target : value;
		return held;
	}) as Schema;
	const root = bare();
	const allDefs = bare();
	for (const [key, value] of Object.entries(named)) {
		if (key !== "$defs") root[key] = value;
		else if (record(value)) for (const [held, definition] of Object.entries(value)) allDefs[held] = definition;
	}
	allDefs[name] = root;
	const moved = bare();
	moved["$ref"] = target;
	moved["$defs"] = allDefs;
	return moved;
}

/** Reports issues by path. */
export function schemaProblems(schema: Json): Issue[] {
	const defs = rootDefs(schema);
	const problems: Issue[] = [];
	schemaNode(schema, [], defs, problems, 0);
	const graph = new Map<string, Set<string>>();
	for (const name of Object.keys(defs)) {
		const refs = new Set<string>();
		inputlessRefs(defs[name] as Json, defs, refs);
		graph.set(name, refs);
	}
	for (const name of Object.keys(defs))
		if (hasLoop(name, graph)) problems.push(issue(["$defs", name], "reference_loop"));
	return problems;
}

/** Preserves issue order. */
export function validate(schema: Json, value: Json): Issue[] {
	const run = new Run(rootDefs(schema));
	run.check(schema, value, 0, true);
	return run.result();
}

/** Exports strict Zod schemas. */
export function strictSchemaOf(shape: AnyShape, form: "input" | "output" = "input"): Json {
	const exported = z.toJSONSchema(shape, {
		io: form,
		unrepresentable: "throw",
		override: ({ zodSchema }) => {
			const { type } = zodSchema._zod.def;
			if (type === "any" || type === "intersection") throw new Error(`${type} is outside strict schemas`);
		},
	}) as Json;
	const normalized = normalizeSchema(exported);
	const problems = schemaProblems(normalized);
	if (problems.length > 0)
		throw new Error(problems.map((problem) => `${problem.path.join(".")}: ${problem.code}`).join("\n"));
	return normalized;
}

export type { Issue, IssueCode };
