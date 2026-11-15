// Schema sampler: JSON texts that meet every rule of a shape.

import { type Json, refOf, type SetSchema } from "@nyaaai/contract/schemaExport";

////////////////////////////////
//  Interfaces & Types

export interface Sampled {
	texts: string[];
	/** Texts dropped past the cap. */
	cut: number;
}

type Obj = { [key: string]: Json };

interface Found {
	valid: Json[];
	unknown: Json[];
	broken: Json[];
}

type Put = (value: Json) => Json;

////////////////////////////////
//  Constants

export const SAMPLE_CAP = 2000;

const DEPTH_FULL = 3;
const DEPTH_HARD = 12;
/** Faults paired per place. */
const FAULTS = 4;
const ASTRAL = "\u{1F600}";
const NONE = Symbol("none");
const GONE = Symbol("gone");
const UNKNOWN_KEY = "unknownKey";
const UNKNOWN_TEXT = "unknownText";
const WRONG: Json[] = [null, true, 0, -1, 1.5, "", "x", [], {}];

////////////////////////////////
//  Functions & Helpers

const isObject = (json: Json | undefined): json is Obj =>
	json !== null && json !== undefined && typeof json === "object" && !Array.isArray(json);

const count = (json: Obj, key: string): number | undefined =>
	typeof json[key] === "number" ? (json[key] as number) : undefined;

const isNull = (json: Obj): boolean => json.type === "null" && Object.keys(json).length === 1;

/** Evenly spaced picks, in order. */
function pick<T>(list: T[], wanted: number): T[] {
	if (list.length <= wanted) return list;
	return Array.from({ length: wanted }, (_, at) => list[Math.floor((at * list.length) / wanted)] as T);
}

/** A different value of the same kind. */
function other(value: Json): Json {
	if (typeof value === "string") return `${value}!`;
	if (typeof value === "number") return value + 1;
	if (typeof value === "boolean") return !value;
	return "x";
}

function plainNumber(json: Obj, integer: boolean): number {
	const step = integer ? 1 : 0.5;
	const low = count(json, "minimum") ?? (count(json, "exclusiveMinimum") ?? Number.NEGATIVE_INFINITY) + step;
	const high = count(json, "maximum") ?? (count(json, "exclusiveMaximum") ?? Number.POSITIVE_INFINITY) - step;
	if (1 < low) return low;
	if (1 > high) return high;
	return 1;
}

function walkNumber(json: Obj, integer: boolean, put: Put, found: Found): void {
	const low = ["minimum", "exclusiveMinimum"].flatMap((key) => count(json, key) ?? []);
	const high = ["maximum", "exclusiveMaximum"].flatMap((key) => count(json, key) ?? []);
	const half = integer ? [0.5] : [];
	for (const bound of low)
		for (const step of [0, -1, ...half.map((one) => -one)]) found.broken.push(put(bound + step));
	for (const bound of high) for (const step of [0, 1, ...half]) found.broken.push(put(bound + step));
}

/** One place changed, or removed. */
function swapped(value: Obj, changes: Array<[string, Json | typeof GONE]>): Obj {
	const held: Obj = {};
	for (const [key, old] of Object.entries(value)) {
		const change = changes.find(([name]) => name === key);
		if (!change) held[key] = old;
		else if (change[1] !== GONE) held[key] = change[1];
	}
	return held;
}

class Sampler {
	/** Nonzero while collecting faults. */
	private pairing = 0;

	constructor(private readonly docs: Map<string, SetSchema>) {}

	/** Follows references to a schema. */
	private deref(at: string, schema: Json): { at: string; json: Obj; hops: number } {
		let held = { at, schema };
		for (let hops = 0; ; hops++) {
			if (!isObject(held.schema)) throw new Error(`${held.at}: not a schema`);
			const target = held.schema.$ref;
			if (typeof target !== "string") return { at: held.at, json: held.schema, hops };
			const ref = refOf(target);
			const set = ref?.set ?? held.at;
			const found = ref ? this.docs.get(set)?.$defs[ref.name] : undefined;
			if (found === undefined || hops > 64) throw new Error(`${held.at}: reference ${target}`);
			held = { at: set, schema: found };
		}
	}

	private branches(json: Obj): Json[] | undefined {
		if (Array.isArray(json.anyOf)) return json.anyOf;
		if (Array.isArray(json.oneOf)) return json.oneOf;
		return undefined;
	}

	/** An anyOf of one schema and null. */
	private nullable(at: string, json: Obj, options: Json[]): boolean {
		return Array.isArray(json.anyOf) && options.length === 2 && isNull(this.deref(at, options[1] as Json).json);
	}

	/** Whether a value has the shape of a schema. */
	private fits(at: string, schema: Json, value: Json): boolean {
		const { at: here, json } = this.deref(at, schema);
		if ("const" in json) return JSON.stringify(json.const) === JSON.stringify(value);
		if (Array.isArray(json.enum)) return json.enum.includes(value);
		const options = this.branches(json);
		if (options) return options.some((option) => this.fits(here, option, value));
		const { type } = json;
		if (Array.isArray(type)) return value === null || this.fits(here, { ...json, type: type[0] as Json }, value);
		switch (type) {
			case "string":
				return typeof value === "string";
			case "number":
			case "integer":
				return typeof value === "number";
			case "boolean":
				return typeof value === "boolean";
			case "null":
				return value === null;
			case "array":
				return Array.isArray(value);
			case "object": {
				if (!isObject(value)) return false;
				if ("propertyNames" in json) return true;
				const required = Array.isArray(json.required) ? (json.required as string[]) : [];
				const properties = isObject(json.properties) ? json.properties : {};
				return required.every((name) => {
					const property = properties[name];
					if (property === undefined) return true;
					const held = this.deref(here, property).json;
					if ("default" in held && !("const" in held)) return true;
					return name in value && ("const" in held ? this.fits(here, held, value[name] as Json) : true);
				});
			}
			default:
				return true;
		}
	}

	/** `depth` counts named shapes entered. */
	build(at: string, schema: Json, held: number, full: boolean): Json | typeof NONE {
		const { at: here, json, hops } = this.deref(at, schema);
		const depth = held + hops;
		if (depth > DEPTH_HARD) return NONE;
		const wide = full && depth < DEPTH_FULL;
		if ("const" in json) return json.const as Json;
		if (Array.isArray(json.enum)) return json.enum[0] as Json;
		const options = this.branches(json);
		if (options) return this.buildUnion(here, json, options, depth, wide);
		const { type } = json;
		if (Array.isArray(type)) {
			if (!wide) return null;
			const held = this.build(here, { ...json, type: type[0] as Json }, depth, wide);
			return held === NONE ? null : held;
		}
		switch (type) {
			case "string":
				return "a".repeat(Math.max(count(json, "minLength") ?? 0, wide ? 1 : 0));
			case "number":
			case "integer":
				return plainNumber(json, type === "integer");
			case "boolean":
				return true;
			case "null":
				return null;
			case "array":
				return "prefixItems" in json
					? this.buildTuple(here, json.prefixItems as Json[], depth, wide)
					: this.buildArray(here, json, depth, wide);
			case "object":
				return "propertyNames" in json
					? this.buildRecord(here, json, depth, wide)
					: this.buildObject(here, json, depth, wide);
			default:
				return {};
		}
	}

	private buildUnion(at: string, json: Obj, options: Json[], depth: number, wide: boolean): Json | typeof NONE {
		if (this.nullable(at, json, options)) {
			if (!wide) return null;
			const held = this.build(at, options[0] as Json, depth, wide);
			return held === NONE ? null : held;
		}
		for (const option of options) {
			const held = this.build(at, option, depth, wide);
			if (held !== NONE) return held;
		}
		return NONE;
	}

	private buildTuple(at: string, places: Json[], depth: number, wide: boolean): Json | typeof NONE {
		const held: Json[] = [];
		for (const place of places) {
			const value = this.build(at, place, depth, wide);
			if (value === NONE) return NONE;
			held.push(value);
		}
		return held;
	}

	private buildArray(at: string, json: Obj, depth: number, wide: boolean): Json | typeof NONE {
		const min = count(json, "minItems") ?? 0;
		const max = count(json, "maxItems") ?? Number.POSITIVE_INFINITY;
		const length = Math.min(Math.max(min, wide ? 1 : 0), max);
		if (length === 0) return [];
		const item = this.build(at, json.items as Json, depth, wide);
		if (item === NONE) return min === 0 ? [] : NONE;
		return Array.from({ length }, () => item);
	}

	private buildRecord(at: string, json: Obj, depth: number, wide: boolean): Json | typeof NONE {
		if (!wide) return {};
		const item = this.build(at, json.additionalProperties as Json, depth, wide);
		return item === NONE ? {} : { k: item };
	}

	private buildObject(at: string, json: Obj, depth: number, wide: boolean): Json | typeof NONE {
		const required = new Set(Array.isArray(json.required) ? (json.required as string[]) : []);
		const held: Obj = {};
		for (const [name, schema] of Object.entries(isObject(json.properties) ? json.properties : {})) {
			const must = required.has(name) && !(isObject(schema) && "default" in schema);
			if (!must && !wide) continue;
			const value = this.build(at, schema, depth, wide);
			if (value !== NONE) held[name] = value;
			else if (must) return NONE;
		}
		return held;
	}

	/** A few broken forms of one place, the place's removal first when asked. */
	private faults(
		at: string,
		schema: Json,
		value: Json,
		depth: number,
		removable: boolean,
	): Array<Json | typeof GONE> {
		const found: Found = { valid: [], unknown: [], broken: [] };
		this.pairing++;
		try {
			this.walk(at, schema, value, depth, (held) => held, found);
		} finally {
			this.pairing--;
		}
		const seen = new Set<string>();
		const broken = found.broken.filter((held) => !seen.has(JSON.stringify(held)) && seen.add(JSON.stringify(held)));
		const gone: Array<typeof GONE> = removable ? [GONE] : [];
		return [...gone, ...pick(broken, FAULTS - gone.length)];
	}

	/** Every change of one place of a built value. */
	walk(at: string, schema: Json, value: Json, held: number, put: Put, found: Found): void {
		const { at: here, json, hops } = this.deref(at, schema);
		const depth = held + hops;
		if (depth > DEPTH_FULL) return;
		for (const wrong of WRONG) found.broken.push(put(wrong));
		if ("const" in json) {
			found.broken.push(put(other(json.const as Json)));
			return;
		}
		if (Array.isArray(json.enum)) {
			for (const option of json.enum) found.valid.push(put(option));
			found.broken.push(put(UNKNOWN_TEXT));
			return;
		}
		const options = this.branches(json);
		if (options) {
			this.walkUnion(here, json, options, value, depth, put, found);
			return;
		}
		const { type } = json;
		if (Array.isArray(type)) {
			found.valid.push(put(null));
			if (value !== null) this.walk(here, { ...json, type: type[0] as Json }, value, depth, put, found);
		} else if (type === "string" && typeof value === "string") this.walkString(json, put, found);
		else if ((type === "number" || type === "integer") && typeof value === "number") {
			walkNumber(json, type === "integer", put, found);
		} else if (type === "array" && Array.isArray(value)) {
			if ("prefixItems" in json) this.walkTuple(here, json, value, depth, put, found);
			else this.walkArray(here, json, value, depth, put, found);
		} else if (type === "object" && isObject(value)) {
			if ("propertyNames" in json) this.walkRecord(here, json, value, depth, put, found);
			else this.walkObject(here, json, value, depth, put, found);
		}
	}

	private walkUnion(
		at: string,
		json: Obj,
		options: Json[],
		value: Json,
		depth: number,
		put: Put,
		found: Found,
	): void {
		if (this.nullable(at, json, options)) {
			found.valid.push(put(null));
			const inner = value === null ? this.build(at, options[0] as Json, depth, true) : value;
			if (inner === NONE) return;
			found.valid.push(put(inner));
			this.walk(at, options[0] as Json, inner, depth, put, found);
			return;
		}
		const own = options.findIndex((option) => this.fits(at, option, value));
		const tag = json["x-tag"];
		options.forEach((option, index) => {
			const built = index === own ? value : this.build(at, option, depth, true);
			if (built === NONE) return;
			found.valid.push(put(built));
			this.walk(at, option, built, depth, put, found);
			if (typeof tag === "string" && isObject(built)) {
				found.broken.push(put({ ...built, [tag]: UNKNOWN_TEXT }));
				found.broken.push(put(swapped(built, [[tag, GONE]])));
			}
		});
	}

	private walkString(json: Obj, put: Put, found: Found): void {
		const min = count(json, "minLength");
		const max = count(json, "maxLength");
		if (min !== undefined) {
			found.valid.push(put(ASTRAL.repeat(min)));
			if (min > 0) found.broken.push(put(ASTRAL.repeat(min - 1)));
		}
		if (max !== undefined && max <= 10_000) {
			found.valid.push(put(ASTRAL.repeat(max)));
			found.broken.push(put(ASTRAL.repeat(max + 1)));
		}
	}

	private walkArray(at: string, json: Obj, value: Json[], depth: number, put: Put, found: Found): void {
		const min = count(json, "minItems");
		const max = count(json, "maxItems");
		const item = this.build(at, json.items as Json, depth, true);
		const [first, ...rest] = value;
		const lengths = [
			...(min !== undefined && min > 0 ? [min - 1] : []),
			...(max !== undefined && max <= 1000 ? [max + 1] : []),
		];
		if (item !== NONE) {
			for (const length of lengths) found.broken.push(put(Array.from({ length }, () => item)));
			if (this.pairing === 0 && first !== undefined) {
				for (const fault of this.faults(at, json.items as Json, first, depth, false)) {
					for (const length of lengths.filter((held) => held > 0)) {
						found.broken.push(put([fault as Json, ...Array.from({ length: length - 1 }, () => item)]));
					}
				}
			}
		}
		if (first !== undefined) this.walk(at, json.items as Json, first, depth, (held) => put([held, ...rest]), found);
	}

	private walkTuple(at: string, json: Obj, value: Json[], depth: number, put: Put, found: Found): void {
		const fewer = value.slice(0, -1);
		const more = [...value, null];
		found.broken.push(put(fewer));
		found.broken.push(put(more));
		const places = json.prefixItems as Json[];
		value.forEach((held, place) => {
			const swap: Put = (next) => put(value.map((old, index) => (index === place ? next : old)));
			this.walk(at, places[place] as Json, held, depth, swap, found);
			if (this.pairing > 0) return;
			for (const fault of pick(this.faults(at, places[place] as Json, held, depth, false), 3)) {
				for (const base of place < fewer.length ? [fewer, more] : [more]) {
					found.broken.push(put(base.map((old, index) => (index === place ? (fault as Json) : old))));
				}
			}
		});
	}

	private walkRecord(at: string, json: Obj, value: Obj, depth: number, put: Put, found: Found): void {
		found.valid.push(put({}));
		const schema = json.additionalProperties as Json;
		for (const [key, held] of Object.entries(value)) {
			this.walk(at, schema, held, depth, (next) => put({ ...value, [key]: next }), found);
		}
		const first = Object.values(value)[0];
		if (this.pairing > 0 || first === undefined) return;
		const faults = this.faults(at, schema, first, depth, false) as Json[];
		for (const one of faults) for (const two of faults) found.broken.push(put({ k: one, j: two }));
	}

	private walkObject(at: string, json: Obj, value: Obj, depth: number, put: Put, found: Found): void {
		found.unknown.push(put({ ...value, [UNKNOWN_KEY]: 0 }));
		const properties = isObject(json.properties) ? json.properties : {};
		const named = Object.keys(value).filter((name) => properties[name] !== undefined);
		for (const name of Object.keys(value)) {
			found.broken.push(put(swapped(value, [[name, GONE]])));
			const schema = properties[name];
			if (schema === undefined) continue;
			this.walk(at, schema, value[name] as Json, depth, (next) => put({ ...value, [name]: next }), found);
		}
		if (this.pairing > 0) return;
		const faults = new Map(
			named.map((name) => [name, this.faults(at, properties[name] as Json, value[name] as Json, depth, true)]),
		);
		named.slice(0, -1).forEach((name, at) => {
			const next = named[at + 1] as string;
			for (const one of faults.get(name) ?? []) {
				for (const two of faults.get(next) ?? []) {
					found.broken.push(
						put(
							swapped(value, [
								[name, one],
								[next, two],
							]),
						),
					);
				}
			}
		});
	}
}

/** The built full value of a shape. */
export function fullValue(docs: Map<string, SetSchema>, set: string, name: string): Json | undefined {
	const root = docs.get(set)?.$defs[name];
	if (root === undefined) throw new Error(`no shape ${set}.${name}`);
	const full = new Sampler(docs).build(set, root, 0, true);
	return full === NONE ? undefined : full;
}

/** Texts for one named shape, and how many the cap dropped. */
export function sampleShape(docs: Map<string, SetSchema>, set: string, name: string, base?: Json): Sampled {
	const sampler = new Sampler(docs);
	const root = docs.get(set)?.$defs[name];
	if (root === undefined) throw new Error(`no shape ${set}.${name}`);
	const full = base ?? sampler.build(set, root, 0, true);
	const minimal = sampler.build(set, root, 0, false);
	if (full === NONE || minimal === NONE) return { texts: [], cut: 0 };
	const found: Found = { valid: [], unknown: [], broken: [] };
	sampler.walk(set, root, full, 0, (held) => held, found);
	const seen = new Set<string>();
	const unique = (values: Json[]): string[] =>
		values.map((value) => JSON.stringify(value)).filter((text) => !seen.has(text) && seen.add(text));
	const all = [...unique([full, minimal, ...found.valid, ...found.unknown]), ...unique(found.broken)];
	return { texts: all.slice(0, SAMPLE_CAP), cut: Math.max(all.length - SAMPLE_CAP, 0) };
}

/** JSON texts for one named shape, mutated from a base value when given. */
export function sampleTexts(docs: Map<string, SetSchema>, set: string, name: string, base?: Json): string[] {
	return sampleShape(docs, set, name, base).texts;
}
