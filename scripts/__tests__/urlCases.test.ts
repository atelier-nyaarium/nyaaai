import { expect, it } from "bun:test";
import { hasControl } from "@nyaaai/contract/text";
import { originOf, parseUrl } from "@nyaaai/contract/url";
import {
	FUZZED_PATHS,
	FUZZED_URLS,
	ORIGIN_URLS,
	PART_URLS,
	PROTOTYPE_URLS,
	PUNYCODE_URLS,
	UNREAD_URLS,
} from "../caseSources/url";

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

/** The platform's parts, as far as this primitive keeps them. */
function platformParts(text: string): Record<string, unknown> | undefined {
	let held: URL;
	try {
		held = new URL(text);
	} catch {
		return undefined;
	}
	if (!["http:", "https:", "ws:", "wss:", "ftp:"].includes(held.protocol)) return undefined;
	return {
		scheme: held.protocol.slice(0, -1),
		host: held.hostname,
		port: held.port,
		origin: held.origin,
		path: held.pathname,
		hasQuery: held.search !== "",
		hasFragment: held.hash !== "",
		hasUsername: held.username !== "",
		hasPassword: held.password !== "",
	};
}

function ownParts(text: string): Record<string, unknown> | undefined {
	const parts = parseUrl(text);
	if (parts === undefined) return undefined;
	return {
		scheme: parts.scheme,
		host: parts.host,
		port: parts.port,
		origin: parts.origin,
		path: parts.path,
		hasQuery: parts.query !== "",
		hasFragment: parts.fragment !== "",
		hasUsername: parts.username !== "",
		hasPassword: parts.password !== "",
	};
}

it("parts read as the platform's URL reads them", () => {
	const wrong = [...PART_URLS, ...PROTOTYPE_URLS, ...ORIGIN_URLS, ...FUZZED_URLS, ...FUZZED_PATHS]
		.filter((text) => !UNREAD_URLS.includes(text))
		.filter((text) => JSON.stringify(ownParts(text)) !== JSON.stringify(platformParts(text)));
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

it("keeps an xn-- label the platform refuses, as written", () => {
	for (const text of PUNYCODE_URLS) {
		expect(platformParts(text), text).toBeUndefined();
		expect(parseUrl(text), text).toBeDefined();
	}
});
