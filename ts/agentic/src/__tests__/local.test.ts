// Local kind tests the cases cannot carry: the scanners and the host test against the platform's own regular expressions.

import { expect, it } from "bun:test";
import { baseOf, remote } from "../kinds/compatible";
import { labelOf } from "../kinds/local/catalog";
import { numCtxOf } from "../kinds/ollama";

////////////////////////////////
//  Fixtures

/** The old line test, kept here as the platform reads it. */
const NUM_CTX = /^num_ctx\s+(\d+)\s*$/m;

const PRIVATE_HOST =
	/^(localhost|(127|10)\.[\d.]+|192\.168\.[\d.]+|172\.(1[6-9]|2\d|3[01])\.[\d.]+|169\.254\.[\d.]+|100\.(6[4-9]|[7-9]\d|1[01]\d|12[0-7])\.[\d.]+|\[(::1|f[cd][\da-f:]*|fe[89ab][\da-f:]*)\]|[^.[]+|.+\.(localhost|local|lan|internal|home\.arpa))$/i;

const PIECES = ["num_ctx", " ", "\n", "\r", " ", " ", "7", "0", "x", "\t"];

////////////////////////////////
//  Functions & Helpers

/** Every text of up to `length` pieces. */
function* texts(length: number): Generator<string> {
	let level: string[] = [""];
	yield "";
	for (let size = 0; size < length; size++) {
		level = level.flatMap((head) => PIECES.map((piece) => head + piece));
		yield* level;
	}
}

const platformContext = (text: string): number | undefined => {
	const number = Number(NUM_CTX.exec(text)?.[1]);
	return number > 0 ? number : undefined;
};

////////////////////////////////
//  Tests

it("the num_ctx scan reads every short text as the platform's line test does", () => {
	let checked = 0;
	for (const text of texts(5)) {
		expect(numCtxOf(text), JSON.stringify(text)).toBe(platformContext(text));
		checked += 1;
	}
	expect(checked).toBeGreaterThan(100_000);
});

it("the num_ctx scan reads a parameters block as the platform does", () => {
	const blocks = [
		"num_ctx                        262144\ntemperature                    0.6",
		"stop <x>\r\nnum_ctx 4096\r\nrepeat_penalty 1",
		"temperature 1\n  num_ctx 4096",
		"num_ctx 4096 \n num_ctx 8192",
		"num_ctx\n8192",
	];
	expect(blocks.map(numCtxOf)).toEqual(blocks.map(platformContext));
});

it("the host test reads each host as the platform's pattern does", () => {
	const urls = [
		"http://127.0.0.1:8000",
		"http://10.1.2.3",
		"http://10.example.com",
		"http://127.evil.example",
		"http://172.15.0.1",
		"http://172.16.0.1",
		"http://172.31.9.9",
		"http://172.32.0.1",
		"http://192.168.0.1",
		"http://192.169.0.1",
		"http://169.254.0.9",
		"http://169.253.0.9",
		"http://100.63.0.1",
		"http://100.64.0.1",
		"http://100.99.0.1",
		"http://100.127.0.1",
		"http://100.128.0.1",
		"http://[::1]:80",
		"http://[::2]",
		"http://[fd12::1]",
		"http://[fc00::1]",
		"http://[fe80::1]",
		"http://[fec0::1]",
		"http://[2001:db8::1]",
		"http://localhost",
		"http://LOCALHOST:3000",
		"http://a.localhost",
		"http://labhost",
		"http://labhost.",
		"http://labhost.lan",
		"http://labhost.internal",
		"http://labhost.home.arpa",
		"http://labhost.local",
		"http://labhost.local.",
		"http://labhost.example.com",
		"http://local",
		"http://.local",
		"http://example.localx",
		"https://host.lab.example/api/v2",
		"http://8.8.8.8",
		"http://0x7f.1",
		"http://2130706433",
		"http://12345",
		"http://1.2",
	];
	const platform = urls.map((url) => !PRIVATE_HOST.test(new URL(url).hostname));
	const changed = new Set(["http://labhost.", "http://labhost.local."]);
	const asked = urls.filter((url) => !changed.has(url));
	expect(asked.map((url) => remote({ kind: "compatible", label: "Host", url }))).toEqual(
		asked.map((url) => !PRIVATE_HOST.test(new URL(url).hostname)),
	);
	expect(platform.length).toBe(urls.length);
	// A trailing dot is the same host.
	expect(
		["http://labhost.", "http://labhost.local.", "http://8.8.8.8."].map((url) =>
			remote({ kind: "compatible", label: "H", url }),
		),
	).toEqual([false, false, true]);
});

it("a label is the platform's last non-blank part when no pair is cut", () => {
	const tags = ["a/b", "a\\b", "a//b//", " a / b ", "/", "", " ", "x/ ", "org/repo/model.gguf", "y".repeat(300)];
	const platform = (tag: string) =>
		(tag.split(/[\\/]/).findLast((part) => part.trim().length > 0) ?? tag).slice(0, 200);
	expect(tags.map(labelOf)).toEqual(tags.map(platform));
});

it("the request root of a URL reads its path as the platform does, and a URL that does not read throws", () => {
	const urls = [
		"http://lab.local:8000",
		"http://lab.local:8000/v1",
		"https://host.lab.example/api/v2",
		"http://lab.local:8000/.",
		"http://lab.local:8000/a/..",
		"http://lab.local:8000/%2e%2e",
		"http://[::1]:8000",
		"HTTP://LAB.LOCAL",
	];
	const platform = (url: string) => {
		const parsed = new URL(url);
		return parsed.pathname === "/" ? `${parsed.origin}/v1` : `${parsed.origin}${parsed.pathname}`;
	};
	expect(urls.map(baseOf)).toEqual(urls.map(platform));
	for (const bad of [
		"",
		"not a url",
		"labhost:8000",
		"http://",
		"http://b\u00fcro.local",
		"http://h?x=1",
		"http://h/#f",
		"http://u:p@h",
	])
		expect(() => baseOf(bad)).toThrow();
});
