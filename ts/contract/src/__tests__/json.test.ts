import { describe, expect, it } from "bun:test";
import { MAX_DEPTH, parseJson, stringifyJson } from "../json";

describe("parseJson", () => {
	it("replaces a lone surrogate, in a key too, and keeps a pair", () => {
		const value = parseJson('{"\\ud83d":["\\ud83d\\ude00","\\ude00x"],"__proto__":"\\ud83d"}');
		expect(stringifyJson(value)).toBe('{"\u{fffd}":["\u{1F600}","\u{fffd}x"],"__proto__":"\u{fffd}"}');
	});

	it("refuses nesting past the cap and text JSON refuses", () => {
		const nested = (depth: number) => "[".repeat(depth) + "]".repeat(depth);
		expect(parseJson(nested(MAX_DEPTH))).toBeDefined();
		expect(parseJson(nested(MAX_DEPTH + 1))).toBeUndefined();
		expect(parseJson("[1,]")).toBeUndefined();
	});
});
