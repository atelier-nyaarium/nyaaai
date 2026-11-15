import { describe, expect, it } from "bun:test";
import { jsString, MAX_DEPTH, parseJson, stringifyJson } from "../json";

describe("jsString", () => {
	it("coerces as String does, and never calls a key named toString", () => {
		expect(jsString([1, null, [2, [undefined, "x"]], {}])).toBe("1,,2,,x,[object Object]");
		expect(jsString(parseJson('{"toString":1}'))).toBe("[object Object]");
		expect(jsString(parseJson('[{"toString":"x","valueOf":2}]'))).toBe("[object Object]");
		expect([undefined, null, true, -0, 1e21, "t"].map(jsString)).toEqual([
			"undefined",
			"null",
			"true",
			"0",
			"1e+21",
			"t",
		]);
	});
});

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
