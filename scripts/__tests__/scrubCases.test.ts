import { describe, expect, it } from "bun:test";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";

////////////////////////////////
//  Constants

const DIR = path.join(import.meta.dirname, "..", "..", "cases", "transport");

/** Shorter keys are letters of ordinary words. */
const MIN_KEY = 4;

/** A model's own words are content. */
const SAID = "exchange.scrub-the-text-a-model-says-is-not-scrubbed.json";

////////////////////////////////
//  Functions & Helpers

type Json = null | boolean | number | string | Json[] | { [key: string]: Json };

function spellings(key: string): string[] {
	return [...new Set([key, encodeURIComponent(key), Buffer.from(key).toString("base64").replace(/=+$/, "")])];
}

/** Every key a case gives its transport. */
function keysOf(input: { [key: string]: Json }): string[] {
	const held = Array.isArray(input["keys"]) ? input["keys"] : [];
	const calls = Array.isArray(input["calls"]) ? input["calls"] : [];
	const explicit = calls.map((call) => {
		const auth = call !== null && typeof call === "object" && !Array.isArray(call) ? call["keyAuth"] : null;
		return auth !== null && typeof auth === "object" && !Array.isArray(auth) ? auth["key"] : null;
	});
	return [...held, ...explicit].filter((key): key is string => typeof key === "string" && key.length >= MIN_KEY);
}

////////////////////////////////
//  Tests

describe("scrub cases", () => {
	it("hold no spelling of a key outside the requests they sent", () => {
		const names = readdirSync(DIR).filter((name) => name.startsWith("exchange.scrub-"));
		expect(names.length).toBeGreaterThan(20);
		const leaks: string[] = [];
		for (const name of names) {
			const held = JSON.parse(readFileSync(path.join(DIR, name), "utf8")) as {
				input: { [key: string]: Json };
				expected: { [key: string]: Json };
			};
			if (held.input["auth"] === "none") continue;
			const { requests: _sent, ...shown } = held.expected;
			const text = JSON.stringify(shown);
			for (const key of keysOf(held.input)) {
				for (const spelling of spellings(key)) {
					if (text.includes(spelling)) leaks.push(name);
				}
			}
		}
		expect(leaks).toEqual([SAID]);
	});
});
