// Builders for the catalog cases. Every value is made up.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Interfaces & Types

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

////////////////////////////////
//  Constants

/** A day in milliseconds. */
export const DAY = 86_400_000;

export const HOUR = 3_600_000;

/** 2026-09-25T12:00:00Z. */
export const NOW = 1_790_337_600_000;

////////////////////////////////
//  Functions & Helpers

const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value));

/** A copy with the field at `path` set. */
export function setIn(base: Json, path: readonly string[], value: Json): Json {
	const copy = clone(base);
	let at = copy as Record<string, Json>;
	for (const key of path.slice(0, -1)) at = at[key] as Record<string, Json>;
	at[path[path.length - 1] as string] = value;
	return copy;
}

/** A copy with the field at `path` left out. */
export function cut(base: Json, path: readonly string[]): Json {
	const copy = clone(base);
	let at = copy as Record<string, Json>;
	for (const key of path.slice(0, -1)) at = at[key] as Record<string, Json>;
	delete at[path[path.length - 1] as string];
	return copy;
}

/** A value of another type than the field holds. */
export function wrongFor(holds: "text" | "number" | "object" | "list" | "boolean"): Json {
	switch (holds) {
		case "text":
			return 7;
		case "number":
			return "7";
		case "boolean":
			return "true";
		case "object":
			return "x";
		case "list":
			return "x";
	}
}

/** A case of one kind. */
export const caseOf =
	(kind: string) =>
	(name: string, input: unknown): CaseSource => ({ kind, name, input });

/** A reply of the scripted network that holds this JSON. */
export const reply = (json: unknown, extra: Record<string, unknown> = {}) => ({ status: 200, json, ...extra });

/** A reply that fails. */
export const failing = (status = 500, extra: Record<string, unknown> = {}) => ({
	status,
	json: { error: { message: "Down." } },
	...extra,
});

/** A reply that never arrives. */
export const refused = () => ({ status: 200, text: "", end: "refused" });
