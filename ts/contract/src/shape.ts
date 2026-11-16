// Contract shapes.

import { z } from "zod";
import { parseJson } from "./json";
import { Pattern } from "./pattern";

/** Shared Zod instance. */
export { z };

////////////////////////////////
//  Interfaces & Types

export type PathPart = string | number;

/** First issue; fixed marks authored messages. */
export type Decoded<T> = { ok: true; value: T } | { ok: false; path: PathPart[]; message: string; fixed?: true };

export type AnyShape = z.ZodType;

export type Output<T extends z.ZodType> = z.output<T>;

export type Input<T extends z.ZodType> = z.input<T>;

/** Port hooks with matching messages. */
export type Hook =
	| { kind: "preprocess"; name: string }
	| { kind: "refine"; name: string; message?: string }
	| { kind: "match"; source: string; flags: string; message?: string };

export type HookKind = Hook["kind"];

export interface ShapeSet<Shapes extends Record<string, z.ZodType> = Record<string, z.ZodType>> {
	name: string;
	shapes: Shapes;
}

type Schema = z.core.$ZodType;

/** Loose Zod definition. */
interface Def {
	type: string;
	shape?: Record<string, Schema>;
	element?: Schema;
	items?: Schema[];
	valueType?: Schema;
	options?: Schema[];
	discriminator?: string;
	innerType?: Schema;
	getter?: () => Schema;
	out?: Schema;
	values?: unknown[];
}

////////////////////////////////
//  Constants

export const CONTRACT_VERSION = "0.1.0";

/** Named shape IDs: `<set>.<Name>`. */
export const shapeIds = z.registry<{ id: string }>();

/** Hook order: inner first. */
export const hooks = z.registry<{ list: Hook[] }>();

const sets = new Map<string, ShapeSet>();

const FIXED = { fixed: true };

/** A failed union's refusal. Rust holds the same. */
const NO_BRANCH = "no branch matches";

////////////////////////////////
//  Functions & Helpers

export function shapeSet<Shapes extends Record<string, z.ZodType>>(name: string, shapes: Shapes): ShapeSet<Shapes> {
	if (sets.has(name)) throw new Error(`two shape sets are named ${name}`);
	for (const [key, schema] of Object.entries(shapes)) {
		const held = shapeIds.get(schema);
		if (held) throw new Error(`${name}.${key} is already ${held.id}`);
		shapeIds.add(schema, { id: `${name}.${key}` });
	}
	const set = { name, shapes };
	sets.set(name, set);
	return set;
}

export const shapeSets = (): ShapeSet[] => [...sets.values()];

const worded = (message: string | undefined) => (message === undefined ? {} : { message });

/** Preserves earlier hooks. */
function checked<T extends z.ZodType>(
	schema: T,
	holds: (value: z.output<T>) => boolean,
	message: string | undefined,
	hook: Hook,
): T {
	const hooked = schema.refine(holds, message === undefined ? undefined : { error: message, params: FIXED });
	hooks.add(hooked, { list: [...(hooks.get(schema)?.list ?? []), hook] });
	return hooked;
}

/** Transforms input before decoding. A missing value stays missing, as in Rust. */
export function preprocessed<T extends z.ZodType>(name: string, fix: (raw: unknown) => unknown, schema: T) {
	const hooked = z.preprocess((raw) => (raw === undefined ? raw : fix(raw)), schema);
	hooks.add(hooked, { list: [{ kind: "preprocess", name }] });
	return hooked;
}

/** Checks decoded output. */
export function refined<T extends z.ZodType>(
	name: string,
	schema: T,
	holds: (value: z.output<T>) => boolean,
	message?: string,
): T {
	return checked(schema, holds, message, { kind: "refine", name, ...worded(message) });
}

/** Searches text; rejects unsupported patterns. */
export function matching<T extends z.ZodType<string>>(schema: T, source: string, flags = "", message?: string): T {
	const pattern = Pattern.compile(source, flags);
	return checked(schema, (text) => pattern.test(text), message, {
		kind: "match",
		source,
		flags,
		...worded(message),
	});
}

const defOf = (schema: Schema) => schema._zod.def as unknown as Def;

const keyed = (value: unknown, part: PathPart): unknown =>
	value !== null && typeof value === "object" ? (value as Record<PathPart, unknown>)[part] : undefined;

/** The tagged branch `value` names. */
function taggedBranch(def: Def, value: unknown): Schema | undefined {
	const tag = def.discriminator as string;
	return def.options?.find((option) => {
		let held = defOf(option);
		while (held.type === "lazy" && held.getter) held = defOf(held.getter());
		const literal = held.shape?.[tag];
		return literal !== undefined && defOf(literal).values?.[0] === keyed(value, tag);
	});
}

/** How many parts of `path` lead to the first plain union on it. */
function failedUnion(schema: Schema, value: unknown, path: PathPart[]): number | undefined {
	let held: Schema | undefined = schema;
	let seen = value;
	for (let at = 0; held; ) {
		const def = defOf(held);
		if (def.type === "union" && def.discriminator === undefined) {
			// A branch that matches leaves the refusal to the union's own hooks.
			const matched = def.options?.some((option) => z.safeParse(option, seen).success);
			return matched ? undefined : at;
		}
		if (def.type === "union") held = taggedBranch(def, seen);
		else if (def.type === "optional" || def.type === "nullable" || def.type === "default") held = def.innerType;
		else if (def.type === "lazy") held = def.getter?.();
		else if (def.type === "pipe") held = def.out;
		else {
			if (at === path.length) return undefined;
			const part = path[at] as PathPart;
			if (def.type === "object") held = def.shape?.[part];
			else if (def.type === "tuple") held = def.items?.[part as number];
			else if (def.type === "array") held = def.element;
			else if (def.type === "record") held = def.valueType;
			else return undefined;
			seen = keyed(seen, part);
			at += 1;
		}
	}
	return undefined;
}

/** Zod's first issue. A failed union names itself, whatever its branches reported. */
export function decode<T extends z.ZodType>(schema: T, value: unknown): Decoded<z.output<T>> {
	const parsed = schema.safeParse(value);
	if (parsed.success) return { ok: true, value: parsed.data };
	const [first] = parsed.error.issues;
	const path = (first?.path ?? []) as PathPart[];
	const union = failedUnion(schema, value, path);
	if (union !== undefined) return { ok: false, path: path.slice(0, union), message: NO_BRANCH };
	const refusal = { ok: false as const, path, message: first?.message ?? "refused" };
	return first?.code === "custom" && first.params?.["fixed"] === true ? { ...refusal, fixed: true } : refusal;
}

export function decodeText<T extends z.ZodType>(schema: T, text: string): Decoded<z.output<T>> {
	const value = parseJson(text);
	if (value === undefined) return { ok: false, path: [], message: "not JSON" };
	return decode(schema, value);
}

/** Digits without leading zeros. */
function bare(part: string | undefined): string | undefined {
	if (part === undefined || part.length === 0) return undefined;
	for (let at = 0; at < part.length; at++) {
		if (part.charCodeAt(at) < 0x30 || part.charCodeAt(at) > 0x39) return undefined;
	}
	let from = 0;
	while (from < part.length && part[from] === "0") from += 1;
	return from === part.length ? "0" : part.slice(from);
}

function versionParts(version: string): [string, string] | undefined {
	const parts = version.split(".");
	if (parts.length !== 3) return undefined;
	const major = bare(parts[0]);
	const minor = bare(parts[1]);
	if (major === undefined || minor === undefined || bare(parts[2]) === undefined) return undefined;
	return [major, minor];
}

/** Pre-1.0 needs minor match. */
export function compatibleVersions(left: string, right: string): boolean {
	const held = versionParts(left);
	const other = versionParts(right);
	if (held === undefined || other === undefined) return false;
	return held[0] === other[0] && (held[0] !== "0" || held[1] === other[1]);
}
