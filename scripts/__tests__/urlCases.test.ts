import { expect, it } from "bun:test";
import { hasControl } from "@nyaaai/contract/text";
import { originOf } from "@nyaaai/contract/url";
import { FUZZED_URLS, ORIGIN_URLS, UNREAD_URLS } from "../caseSources/url";

/** The platform's origin, or undefined when it has none. */
function platformOrigin(text: string): string | undefined {
	try {
		const { origin } = new URL(text);
		return origin === "null" ? undefined : origin;
	} catch {
		return undefined;
	}
}

it("origins read as the platform's URL reads them", () => {
	const wrong = [...ORIGIN_URLS, ...FUZZED_URLS]
		.filter((text) => !UNREAD_URLS.includes(text))
		.filter((text) => originOf(text) !== platformOrigin(text));
	expect(wrong).toEqual([]);
});

it("leaves non-ASCII and percent-encoded hosts to the platform", () => {
	for (const text of UNREAD_URLS) {
		expect(originOf(text)).toBeUndefined();
		expect(platformOrigin(text)).toBeDefined();
	}
});

it("a lone surrogate is not a control character", () => {
	expect(hasControl("\u{D800}")).toBe(false);
	expect(hasControl("a\u{DC00}")).toBe(false);
	expect(hasControl("a\u{D800}\u{80}")).toBe(true);
});
