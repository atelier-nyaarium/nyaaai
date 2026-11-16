// Recording Fetch tests.

import { describe, expect, it } from "bun:test";
import { base64Decode, base64Encode } from "@nyaaai/contract/base64";
import type { Fetch, FetchRequest } from "@nyaaai/contract/fetch";
import { encodePathSegment } from "@nyaaai/contract/text";
import { recordingFetch } from "../recorder";
import { TimerWheel } from "../scheduler";
import { type ScriptedExchange, scriptedFetch } from "../scriptedFetch";

////////////////////////////////
//  Functions & Helpers

const SECRET = "key-0123456789abcdef";
const SLASHED = "key/with+odd=chars!";
const signal = new AbortController().signal;
const bytes = (text: string): Uint8Array => new TextEncoder().encode(text);
const text = (chunk: string): string => new TextDecoder().decode(base64Decode(chunk));
const unpadded = (secret: string): string => base64Encode(bytes(secret)).split("=").join("");

const exchange = (patch: Partial<ScriptedExchange> = {}): ScriptedExchange => ({
	status: 200,
	headers: [["content-type", "text/event-stream"]],
	chunks: [],
	...patch,
});

const request = (body?: string): FetchRequest => ({
	method: "POST",
	url: "http://lab.test/v1",
	headers: [["authorization", `Bearer ${SECRET}`]],
	...(body === undefined ? {} : { body }),
});

async function play(inner: Fetch, sent: FetchRequest = request("{}")): Promise<void> {
	const answer = await inner(sent, signal);
	for await (const chunk of answer.chunks) void chunk;
}

async function refusal(held: ReturnType<typeof recordingFetch>): Promise<string> {
	try {
		held.trace();
	} catch (error) {
		return (error as Error).message;
	}
	throw new Error("the trace was not refused");
}

function recording(exchanges: ScriptedExchange[], secrets: string[] = [SECRET]) {
	return recordingFetch(scriptedFetch(exchanges, new TimerWheel()).fetch, secrets);
}

////////////////////////////////
//  Tests

describe("the recording fetch", () => {
	it("keeps the request body, the status, the replay headers and each chunk's bytes", async () => {
		const held = recording([
			exchange({
				headers: [
					["Content-Type", "application/json"],
					["Retry-After", "2"],
					["X-RateLimit-Remaining", "3"],
					["Set-Cookie", "a=b"],
					["Authorization", "echo"],
				],
				chunks: [{ bytes: bytes("ab") }, { bytes: new Uint8Array() }, { bytes: bytes("c\u{1F600}") }],
			}),
			exchange({ status: 404, headers: [] }),
			exchange(),
		]);
		await play(held.fetch, request('{"model":"m","n":[1]}'));
		await play(held.fetch, request("not json"));
		await play(held.fetch, { method: "GET", url: "http://lab.test/v1", headers: [] });
		const [first, second, third] = held.trace().exchanges;
		expect(first?.request).toEqual({ model: "m", n: [1] });
		expect([first?.status, first?.headers]).toEqual([
			200,
			{ "content-type": "application/json", "retry-after": "2", "x-ratelimit-remaining": "3" },
		]);
		expect(first?.chunks.map(text)).toEqual(["ab", "", "c\u{1F600}"]);
		expect([second?.request, second?.status, second?.headers]).toEqual(["not json", 404, {}]);
		expect(third?.request).toBeNull();
		expect(JSON.stringify(held.trace())).not.toContain(SECRET);
	});

	it("keeps what passed when the reader stops early", async () => {
		const held = recording([exchange({ chunks: [{ bytes: bytes("one") }, { bytes: bytes("two") }] })]);
		const answer = await held.fetch(request("{}"), signal);
		for await (const chunk of answer.chunks) {
			void chunk;
			break;
		}
		expect(held.trace().exchanges[0]?.chunks.map(text)).toEqual(["one"]);
	});

	it("refuses an exchange that got no response", async () => {
		const held = recordingFetch(async () => {
			throw new Error("down");
		}, [SECRET]);
		await play(held.fetch).catch(() => undefined);
		expect(await refusal(held)).toBe("exchange 1: no response was kept");
	});

	for (const [form, make] of [
		["as is", (secret: string) => secret],
		["path encoded", encodePathSegment],
		["in base64 without padding", unpadded],
		["path encoded in lower-case hex", (secret: string) => encodePathSegment(secret).toLowerCase()],
		["in base64 after a bearer prefix", (secret: string) => unpadded(`Bearer ${secret}`)],
		["in base64 after one byte, with a tail", (secret: string) => unpadded(`a${secret}-tail`)],
	] as const) {
		it(`refuses a secret ${form} in a response body, naming the place and not the secret`, async () => {
			const held = recording(
				[exchange(), exchange({ chunks: [{ bytes: bytes(`echo ${make(SLASHED)} end`) }] })],
				[SLASHED],
			);
			await play(held.fetch);
			await play(held.fetch);
			const message = await refusal(held);
			expect(message).toBe("exchange 2: a secret occurs in the response body");
			expect(message).not.toContain(SLASHED);
		});

		it(`refuses a secret ${form} in a request body`, async () => {
			const held = recording([exchange()], [SLASHED]);
			await play(held.fetch, request(JSON.stringify({ note: make(SLASHED) })));
			expect(await refusal(held)).toBe("exchange 1: a secret occurs in the request body");
		});

		it(`refuses a secret ${form} across a chunk edge`, async () => {
			const whole = bytes(`x ${make(SLASHED)} y`);
			const cut = 6;
			const held = recording(
				[exchange({ chunks: [{ bytes: whole.slice(0, cut) }, { bytes: whole.slice(cut) }] })],
				[SLASHED],
			);
			await play(held.fetch);
			expect(await refusal(held)).toBe("exchange 1: a secret occurs in the response body");
		});
	}

	it("refuses a secret in a kept header", async () => {
		const held = recording([exchange({ headers: [["x-ratelimit-note", `see ${SECRET}`]] })]);
		await play(held.fetch);
		expect(await refusal(held)).toBe("exchange 1: a secret occurs in the response header x-ratelimit-note");
	});

	it("does not keep a header it does not replay, so a secret there is not in the trace", async () => {
		const held = recording([exchange({ headers: [["x-echo", SECRET]] })]);
		await play(held.fetch);
		expect(held.trace().exchanges[0]?.headers).toEqual({});
	});

	it("never rewrites bytes to hide a secret", async () => {
		const held = recording([exchange({ chunks: [{ bytes: bytes("fine") }] })]);
		await play(held.fetch);
		expect(held.trace().exchanges[0]?.chunks.map(text)).toEqual(["fine"]);
	});
});
