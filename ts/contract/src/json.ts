// Shared JSON decode rules.

////////////////////////////////
//  Constants

/** Maximum JSON nesting. */
export const MAX_DEPTH = 1024;

/** Escaped surrogate code units. */
const SURROGATE_ESCAPE = /\\u[dD][89a-fA-F]/;
/** Detects lone surrogate escapes. */
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

/** Lone surrogates become U+FFFD. */
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

/** Refused text returns undefined. */
export function parseJson(text: string): unknown {
	let value: unknown;
	try {
		value = JSON.parse(decodable(text));
	} catch {
		return undefined;
	}
	return deeperThan(value, MAX_DEPTH) ? undefined : value;
}

/** Rust-compatible JSON spelling. */
export function stringifyJson(value: unknown): string {
	return JSON.stringify(value);
}
