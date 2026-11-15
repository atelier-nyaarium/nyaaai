// JSON parsing and serialization.

import { compareText } from "./order";

////////////////////////////////
//  Constants

/** Maximum container depth. */
export const MAX_DEPTH = 1024;

/** The library target predates it. */
type WellFormable = string & { toWellFormed(): string };

const HEX = "0123456789abcdefABCDEF";

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
	return (text as WellFormable).toWellFormed();
}

/** Four hex digits at `at`, as a number. */
function hexUnit(text: string, at: number): number | undefined {
	if (at + 4 > text.length) return undefined;
	let unit = 0;
	for (let place = at; place < at + 4; place++) {
		const digit = HEX.indexOf(text[place] as string);
		if (digit < 0) return undefined;
		unit = unit * 16 + (digit > 15 ? digit - 6 : digit);
	}
	return unit;
}

/** Escape unit at `at`. */
function escapedUnit(text: string, at: number): number | undefined {
	return text[at] === "\\" && text[at + 1] === "u" ? hexUnit(text, at + 2) : undefined;
}

/** Decoding may merge keys. */
function decodable(text: string): string {
	let out = "";
	let from = 0;
	let inString = false;
	for (let at = 0; at < text.length; ) {
		const held = text[at];
		if (held === '"') {
			inString = !inString;
			at += 1;
		} else if (held === "\\" && inString) {
			const unit = escapedUnit(text, at);
			const next = escapedUnit(text, at + 6);
			if (
				unit !== undefined &&
				unit >= 0xd800 &&
				unit <= 0xdbff &&
				next !== undefined &&
				next >= 0xdc00 &&
				next <= 0xdfff
			) {
				at += 12;
			} else if (unit !== undefined && unit >= 0xd800 && unit <= 0xdfff) {
				out += `${text.slice(from, at)}\\ufffd`;
				at += 6;
				from = at;
			} else {
				at += 2;
			}
		} else {
			at += 1;
		}
	}
	return wellFormed(from === 0 ? text : out + text.slice(from));
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
