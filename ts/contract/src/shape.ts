// Contract shapes.

import { z } from "zod";
import { parseJson } from "./json";

////////////////////////////////
//  Interfaces & Types

export type PathPart = string | number;

/** Refusal reports first issue. */
export type Decoded<T> = { ok: true; value: T } | { ok: false; path: PathPart[]; message: string };

export type AnyShape = z.ZodType;

export type Output<T extends z.ZodType> = z.output<T>;

export type Input<T extends z.ZodType> = z.input<T>;

export type HookKind = "preprocess" | "refine";

/** Hooks require language ports. */
export interface Hook {
	kind: HookKind;
	name: string;
}

export interface ShapeSet<Shapes extends Record<string, z.ZodType> = Record<string, z.ZodType>> {
	name: string;
	shapes: Shapes;
}

////////////////////////////////
//  Constants

export const CONTRACT_VERSION = "0.1.0";

/** Named shape IDs: `<set>.<Name>`. */
export const shapeIds = z.registry<{ id: string }>();

export const hooks = z.registry<Hook>();

const sets = new Map<string, ShapeSet>();

////////////////////////////////
//  Functions & Helpers

/** Registers shapes as `<set>.<key>`. */
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

/** Pre-decode input transform. */
export function preprocessed<T extends z.ZodType>(name: string, fix: (raw: unknown) => unknown, schema: T) {
	const hooked = z.preprocess(fix, schema);
	hooks.add(hooked, { kind: "preprocess", name });
	return hooked;
}

/** Checks decoded output. */
export function refined<T extends z.ZodType>(name: string, schema: T, holds: (value: z.output<T>) => boolean): T {
	const hooked = schema.refine(holds);
	hooks.add(hooked, { kind: "refine", name });
	return hooked;
}

export function decode<T extends z.ZodType>(schema: T, value: unknown): Decoded<z.output<T>> {
	const parsed = schema.safeParse(value);
	if (parsed.success) return { ok: true, value: parsed.data };
	const [first] = parsed.error.issues;
	return { ok: false, path: (first?.path ?? []) as PathPart[], message: first?.message ?? "refused" };
}

export function decodeText<T extends z.ZodType>(schema: T, text: string): Decoded<z.output<T>> {
	const value = parseJson(text);
	if (value === undefined) return { ok: false, path: [], message: "not JSON" };
	return decode(schema, value);
}

const VERSION = /^0*(\d+)\.0*(\d+)\.\d+$/;

/** Same major; before 1.0, the same minor too. */
export function compatibleVersions(left: string, right: string): boolean {
	const [, leftMajor, leftMinor] = VERSION.exec(left) ?? [];
	const [, rightMajor, rightMinor] = VERSION.exec(right) ?? [];
	if (leftMajor === undefined || leftMajor !== rightMajor) return false;
	return leftMajor !== "0" || leftMinor === rightMinor;
}
