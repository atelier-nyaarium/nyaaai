// JSON parsing and serialization.

import { compareText } from "./order";

////////////////////////////////
//  Constants

/** Maximum container depth. */
export const MAX_DEPTH = 1024;

const SURROGATE_ESCAPE = /\\u[dD][89a-fA-F]/;
const ESCAPES =
	/\\(?:u[dD][89abAB][0-9a-fA-F]{2}\\u[dD][c-fC-F][0-9a-fA-F]{2}|(u[dD][89a-fA-F][0-9a-fA-F]{2})|[\s\S])/g;
const LONE_SURROGATE = /[\ud800-\udbff](?![\udc00-\udfff])|(?<![\ud800-\udbff])[\udc00-\udfff]/;
const LONE_SURROGATES = new RegExp(LONE_SURROGATE, "g");

////////////////////////////////
//  Functions & Helpers

function deeperThan(value: unknown, limit: number): boolean {
	const pending: Array<[unknown, number]> = [[value, 0]];
	for (let held = pending.pop(); held; held = pending.pop()) {
		const [node, depth] = held;
		if (node === null || typeof node !== "object") continue;
		if (depth + 1 > limit) return true;
		for (const child of Array.isArray(node) ? node : Object.values(node)) {
			if (child !== null && typeof child === "object") pending.push([child, depth + 1]);
		}
	}
	return false;
}

/** Replaces lone surrogates. */
function wellFormed(text: string): string {
	return LONE_SURROGATE.test(text) ? text.replace(LONE_SURROGATES, "\u{fffd}") : text;
}

/** Decoding may merge keys. */
function decodable(text: string): string {
	const escaped = SURROGATE_ESCAPE.test(text)
		? text.replace(ESCAPES, (held, lone?: string) => (lone === undefined ? held : "\\ufffd"))
		: text;
	return wellFormed(escaped);
}

/** Normalizes surrogates. Limits nesting. */
export function parseJson(text: string): unknown {
	let value: unknown;
	try {
		value = JSON.parse(decodable(text));
	} catch {
		return undefined;
	}
	return deeperThan(value, MAX_DEPTH) ? undefined : value;
}

/** ECMAScript-compatible JSON text. */
export function stringifyJson(value: unknown): string {
	return JSON.stringify(value);
}

/** Ignores `toString` keys. */
export function jsString(value: unknown): string {
	if (Array.isArray(value))
		return value.map((item) => (item === null || item === undefined ? "" : jsString(item))).join(",");
	return value !== null && typeof value === "object" ? "[object Object]" : String(value);
}

const skipped = (value: unknown) => value === undefined || typeof value === "function" || typeof value === "symbol";

/** Code-unit key order. */
export function canonicalJson(value: unknown): string {
	const held =
		value !== null && typeof value === "object" && typeof (value as { toJSON?: unknown }).toJSON === "function"
			? (value as { toJSON: () => unknown }).toJSON()
			: value;
	if (skipped(held)) return "null";
	if (held === null || typeof held !== "object") return JSON.stringify(held);
	if (Array.isArray(held)) return `[${held.map(canonicalJson).join(",")}]`;
	const fields = Object.keys(held)
		.sort(compareText)
		.filter((key) => !skipped((held as Record<string, unknown>)[key]))
		.map((key) => `${JSON.stringify(key)}:${canonicalJson((held as Record<string, unknown>)[key])}`);
	return `{${fields.join(",")}}`;
}
