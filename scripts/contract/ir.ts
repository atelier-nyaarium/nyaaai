// Contract intermediate form.

import { type Json, refOf, type SetSchema } from "@nyaaai/contract/schemaExport";
import type { Hook } from "@nyaaai/contract/shape";

////////////////////////////////
//  Interfaces & Types

interface Hooked {
	/** Innermost first. */
	hooks?: Hook[];
}

export type Node = Hooked &
	(
		| { kind: "string"; minLength?: number; maxLength?: number }
		| {
				kind: "number";
				integer: boolean;
				minimum?: number;
				maximum?: number;
				exclusiveMinimum?: number;
				exclusiveMaximum?: number;
		  }
		| { kind: "boolean" }
		| { kind: "null" }
		| { kind: "unknown" }
		| { kind: "literal"; value: string | number | boolean }
		| { kind: "enum"; values: string[] }
		| { kind: "array"; items: Node; minItems?: number; maxItems?: number }
		/** Fixed-length tuples. */
		| { kind: "tuple"; items: Node[] }
		| { kind: "record"; values: Node }
		| { kind: "object"; fields: Field[] }
		| { kind: "nullable"; inner: Node }
		/** First matching branch wins. */
		| { kind: "union"; branches: Node[] }
		| { kind: "tagged"; tag: string; branches: TaggedBranch[] }
		| { kind: "ref"; set: string; name: string }
	);

export interface TaggedBranch {
	value: string;
	node: Node;
}

export interface Field {
	name: string;
	node: Node;
	presence: "required" | "optional" | "default";
	default?: Json;
}

export interface Shape {
	name: string;
	node: Node;
}

export interface SetIr {
	set: string;
	version: string;
	shapes: Shape[];
}

type JsonObject = { [key: string]: Json };

////////////////////////////////
//  Constants

/** JavaScript safe-integer bound. */
const SAFE = Number.MAX_SAFE_INTEGER;

////////////////////////////////
//  Functions & Helpers

class Reader {
	/** Pending tagged branch values. */
	readonly tagged: Array<{ node: Extract<Node, { kind: "tagged" }>; where: string }> = [];

	constructor(private readonly set: string) {}

	node(json: Json, where: string): Node {
		if (json === null || typeof json !== "object" || Array.isArray(json)) {
			throw new Error(`${where}: not a schema`);
		}
		const { "x-hooks": listed, ...rest } = json;
		const node = this.bare(rest, where);
		if (listed === undefined) return node;
		const hooks = listed as unknown as Hook[];
		if (node.kind !== "string" && hooks.some((hook) => hook.kind === "match")) {
			throw new Error(`${where}: a pattern checks text`);
		}
		return { ...node, hooks };
	}

	private bare(json: JsonObject, where: string): Node {
		const only = (...allowed: string[]) => {
			for (const key of Object.keys(json)) {
				if (!allowed.includes(key)) throw new Error(`${where}: keyword ${key} is outside the dialect`);
			}
		};
		const count = (key: string) => (json[key] === undefined ? {} : { [key]: json[key] as number });
		if ("$ref" in json) {
			only("$ref");
			return this.ref(json.$ref as string, where);
		}
		if ("const" in json) {
			only("const", "type");
			const value = json.const as string | number | boolean | null;
			return value === null ? { kind: "null" } : { kind: "literal", value };
		}
		if ("enum" in json) {
			only("enum", "type");
			const values = json.enum as Json[];
			if (json.type !== "string" || !values.every((value) => typeof value === "string")) {
				throw new Error(`${where}: an enum holds texts`);
			}
			return { kind: "enum", values: values as string[] };
		}
		if ("anyOf" in json) {
			only("anyOf");
			const branches = (json.anyOf as Json[]).map((branch, at) => this.node(branch, `${where}|${at}`));
			const [first, second, ...rest] = branches;
			if (first && second && rest.length === 0 && second.kind === "null" && !second.hooks) {
				return { kind: "nullable", inner: first };
			}
			return { kind: "union", branches };
		}
		if ("oneOf" in json) {
			only("oneOf", "x-tag");
			if (typeof json["x-tag"] !== "string") throw new Error(`${where}: an exclusive union needs a tag`);
			const node: Extract<Node, { kind: "tagged" }> = {
				kind: "tagged",
				tag: json["x-tag"],
				branches: (json.oneOf as Json[]).map((branch, at) => ({
					value: "",
					node: this.node(branch, `${where}|${at}`),
				})),
			};
			this.tagged.push({ node, where });
			return node;
		}
		const { type } = json;
		if (Array.isArray(type)) {
			const [inner, nothing, ...rest] = type;
			if (nothing !== "null" || rest.length > 0) throw new Error(`${where}: type ${JSON.stringify(type)}`);
			return { kind: "nullable", inner: this.bare({ ...json, type: inner as string }, where) };
		}
		switch (type) {
			case undefined:
				only();
				return { kind: "unknown" };
			case "string":
				only("type", "minLength", "maxLength");
				return { kind: "string", ...count("minLength"), ...count("maxLength") };
			case "number":
			case "integer": {
				only("type", "minimum", "maximum", "exclusiveMinimum", "exclusiveMaximum");
				const integer = type === "integer";
				const implied = (key: "minimum" | "maximum", bound: number) =>
					integer && json[key] === bound ? {} : count(key);
				return {
					kind: "number",
					integer,
					...implied("minimum", -SAFE),
					...implied("maximum", SAFE),
					...count("exclusiveMinimum"),
					...count("exclusiveMaximum"),
				};
			}
			case "boolean":
				only("type");
				return { kind: "boolean" };
			case "null":
				only("type");
				return { kind: "null" };
			case "array":
				if ("prefixItems" in json) return this.tuple(json, where, only);
				only("type", "items", "minItems", "maxItems");
				if (json.items === undefined) throw new Error(`${where}: an array names its items`);
				return {
					kind: "array",
					items: this.node(json.items, `${where}[]`),
					...count("minItems"),
					...count("maxItems"),
				};
			case "object":
				return "propertyNames" in json ? this.record(json, where, only) : this.object(json, where, only);
			default:
				throw new Error(`${where}: type ${JSON.stringify(type)} is outside the dialect`);
		}
	}

	private tuple(json: JsonObject, where: string, only: (...allowed: string[]) => void): Node {
		only("type", "prefixItems", "items", "minItems", "maxItems");
		const places = json.prefixItems;
		const fixed = (count: Json | undefined) => Array.isArray(places) && count === places.length;
		if (!Array.isArray(places) || places.length === 0 || json.items !== false) {
			throw new Error(`${where}: a tuple holds fixed places`);
		}
		if (!fixed(json.minItems) || !fixed(json.maxItems)) throw new Error(`${where}: a tuple fills every place`);
		if (places.length > 12) throw new Error(`${where}: a tuple holds at most 12 places`);
		return { kind: "tuple", items: places.map((place, at) => this.node(place, `${where}[${at}]`)) };
	}

	private record(json: JsonObject, where: string, only: (...allowed: string[]) => void): Node {
		only("type", "propertyNames", "additionalProperties");
		if (JSON.stringify(json.propertyNames) !== '{"type":"string"}' || json.additionalProperties === undefined) {
			throw new Error(`${where}: a record's keys are any text`);
		}
		return { kind: "record", values: this.node(json.additionalProperties, `${where}{}`) };
	}

	private object(json: JsonObject, where: string, only: (...allowed: string[]) => void): Node {
		only("type", "properties", "required", "additionalProperties");
		if (json.additionalProperties !== false) throw new Error(`${where}: an object keeps no unknown keys`);
		const required = new Set((json.required as string[] | undefined) ?? []);
		const fields = Object.entries((json.properties as JsonObject | undefined) ?? {}).map(
			([name, schema]): Field => {
				const at = `${where}.${name}`;
				if (schema === null || typeof schema !== "object" || Array.isArray(schema)) {
					throw new Error(`${at}: not a schema`);
				}
				if (!("default" in schema)) {
					return {
						name,
						node: this.node(schema, at),
						presence: required.has(name) ? "required" : "optional",
					};
				}
				if (!required.has(name)) throw new Error(`${at}: an optional property holds no default`);
				const { default: fallback, ...rest } = schema;
				return { name, node: this.node(rest, at), presence: "default", default: fallback as Json };
			},
		);
		return { kind: "object", fields };
	}

	private ref(target: string, where: string): Node {
		const held = refOf(target);
		if (held?.set === undefined && held) return { kind: "ref", set: this.set, name: held.name };
		if (held?.set !== undefined && held.form === "output") return { kind: "ref", set: held.set, name: held.name };
		throw new Error(`${where}: reference ${target}`);
	}
}

/** Resolves named-shape references. */
export function resolved(node: Node, sets: Map<string, SetIr>): Node {
	let held = node;
	for (let hops = 0; held.kind === "ref"; hops++) {
		const { set, name } = held;
		const target = sets.get(set)?.shapes.find((shape) => shape.name === name);
		if (!target || hops > 64) throw new Error(`${set}.${name} names no shape`);
		held = target.node;
	}
	return held;
}

export function irOf(docs: Map<string, SetSchema>): Map<string, SetIr> {
	const sets = new Map<string, SetIr>();
	const tagged: Reader["tagged"] = [];
	for (const [set, doc] of docs) {
		const reader = new Reader(set);
		const shapes = Object.entries(doc.$defs).map(([name, json]) => ({
			name,
			node: reader.node(json, `${set}.${name}`),
		}));
		sets.set(set, { set, version: doc["x-contract-version"], shapes });
		tagged.push(...reader.tagged);
	}
	for (const { node, where } of tagged) {
		for (const branch of node.branches) {
			const target = resolved(branch.node, sets);
			const field = target.kind === "object" ? target.fields.find((held) => held.name === node.tag) : undefined;
			if (
				field?.presence !== "required" ||
				field.node.kind !== "literal" ||
				typeof field.node.value !== "string"
			) {
				throw new Error(`${where}: each branch is an object with a text ${node.tag}`);
			}
			branch.value = field.node.value;
		}
		if (new Set(node.branches.map((branch) => branch.value)).size !== node.branches.length) {
			throw new Error(`${where}: two branches share a ${node.tag}`);
		}
	}
	return sets;
}

/** Output-to-input conversion. */
function asInput(node: Json): Json {
	if (Array.isArray(node)) return node.map(asInput);
	if (node === null || typeof node !== "object") return node;
	const held: JsonObject = {};
	for (const [key, value] of Object.entries(node)) {
		if (key === "additionalProperties" && value === false) continue;
		if (key === "$ref" && typeof value === "string") held[key] = value.replace(".output.json#", ".input.json#");
		else held[key] = asInput(value);
	}
	const properties = held.properties;
	if (Array.isArray(held.required) && properties !== null && typeof properties === "object") {
		const filled = (name: Json) => {
			const property = (properties as JsonObject)[name as string];
			return property !== null && typeof property === "object" && "default" in property;
		};
		held.required = held.required.filter((name) => !filled(name));
	}
	if (Array.isArray(held.required) && held.required.length === 0) delete held.required;
	return held;
}

/** Forms differ by defaults. */
export function formProblems(set: string, input: SetSchema, output: SetSchema): string[] {
	const problems: string[] = [];
	for (const [name, schema] of Object.entries(output.$defs)) {
		if (JSON.stringify(asInput(schema)) !== JSON.stringify(asInput(input.$defs[name] ?? null))) {
			problems.push(`${set}.${name}: the input and output forms differ beyond defaults`);
		}
	}
	return problems;
}
