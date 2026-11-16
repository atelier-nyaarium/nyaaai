// Scripted fetch and key source tests.

import { describe, expect, it } from "bun:test";
import type { FetchRequest } from "@nyaaai/contract/fetch";
import { fakeKeys } from "../keys";
import { rejection } from "../rejection";
import { TimerWheel } from "../scheduler";
import { type ScriptedExchange, scriptedFetch } from "../scriptedFetch";
import { replayFetch } from "../trace";

////////////////////////////////
//  Functions & Helpers

const REQUEST: FetchRequest = { method: "GET", url: "http://lab.local/x", headers: [] };

const bytes = (text: string): Uint8Array => new TextEncoder().encode(text);

const exchange = (over: Partial<ScriptedExchange> = {}): ScriptedExchange => ({
	status: 200,
	headers: [],
	chunks: [{ bytes: bytes("a") }, { bytes: bytes("b") }],
	...over,
});

async function drain(chunks: AsyncIterable<Uint8Array>): Promise<string[]> {
	const seen: string[] = [];
	for await (const chunk of chunks) seen.push(new TextDecoder().decode(chunk));
	return seen;
}

////////////////////////////////
//  Tests

describe("the scripted fetch", () => {
	it("plays its exchanges in order and records each request and each chunk handed out", async () => {
		const wheel = new TimerWheel(100);
		const played = scriptedFetch([exchange(), exchange({ status: 404, chunks: [] })], wheel);
		const signal = new AbortController().signal;
		const first = await played.fetch(REQUEST, signal);
		expect(first.status).toBe(200);
		expect(await drain(first.chunks)).toEqual(["a", "b"]);
		const second = await played.fetch({ ...REQUEST, url: "http://lab.local/y" }, signal);
		expect(second.status).toBe(404);
		expect(played.requests.map((request) => request.url)).toEqual(["http://lab.local/x", "http://lab.local/y"]);
		expect(played.read()).toEqual([2, 0]);
		const error = await rejection(played.fetch(REQUEST, signal));
		expect((error as Error).message).toBe("the script holds no further exchange");
	});

	it("answers headers lowercase, sorted and joined, as the port says", async () => {
		const played = scriptedFetch(
			[
				exchange({
					headers: [
						["X-B", "1"],
						["a", "2"],
						["x-b", "3"],
						["Set-Cookie", "p=1"],
						["set-cookie", "q=2"],
					],
				}),
			],
			new TimerWheel(),
		);
		const response = await played.fetch(REQUEST, new AbortController().signal);
		expect(response.headers).toEqual([
			["a", "2"],
			["set-cookie", "p=1"],
			["set-cookie", "q=2"],
			["x-b", "1, 3"],
		]);
	});

	it("spends the wheel's time on the head and on each chunk, and fires what comes due", async () => {
		const wheel = new TimerWheel(0);
		const fired: string[] = [];
		wheel.after(15, "mid", () => fired.push(`mid@${wheel.nowMs()}`));
		const played = scriptedFetch(
			[exchange({ headMs: 10, chunks: [{ bytes: bytes("a"), delayMs: 10 }, { bytes: bytes("b") }] })],
			wheel,
		);
		const response = await played.fetch(REQUEST, new AbortController().signal);
		expect(wheel.nowMs()).toBe(10);
		await drain(response.chunks);
		expect(wheel.nowMs()).toBe(20);
		expect(fired).toEqual(["mid@15"]);
	});

	it("fails with the signal's reason when it aborts during the head or between chunks", async () => {
		const wheel = new TimerWheel(0);
		const stop = new AbortController();
		wheel.after(5, "stop", () => stop.abort(new Error("stopped")));
		const played = scriptedFetch([exchange({ headMs: 10 }), exchange()], wheel);
		expect(((await rejection(played.fetch(REQUEST, stop.signal))) as Error).message).toBe("stopped");

		const later = new AbortController();
		const response = await played.fetch(REQUEST, later.signal);
		const reader = response.chunks[Symbol.asyncIterator]();
		expect((await reader.next()).value).toEqual(bytes("a"));
		later.abort(new Error("later"));
		expect(((await rejection(reader.next())) as Error).message).toBe("later");
	});

	it("refuses before the head or drops after the last chunk, with its own message", async () => {
		const played = scriptedFetch(
			[
				exchange({ end: "refused" }),
				exchange({ end: "refused", error: "no route" }),
				exchange({ end: "dropped" }),
				exchange({ end: "dropped", error: "reset" }),
			],
			new TimerWheel(),
		);
		const signal = new AbortController().signal;
		expect(((await rejection(played.fetch(REQUEST, signal))) as Error).message).toBe("connection refused");
		expect(((await rejection(played.fetch(REQUEST, signal))) as Error).message).toBe("no route");
		const dropped = await played.fetch(REQUEST, signal);
		expect(((await rejection(drain(dropped.chunks))) as Error).message).toBe("connection dropped");
		const reset = await played.fetch(REQUEST, signal);
		expect(((await rejection(drain(reset.chunks))) as Error).message).toBe("reset");
		expect(played.read()).toEqual([0, 0, 2, 2]);
	});

	it("takes a request when the signal is already aborted, and plays no exchange for it", async () => {
		const played = scriptedFetch([exchange()], new TimerWheel());
		const stop = new AbortController();
		stop.abort(new Error("early"));
		expect(((await rejection(played.fetch(REQUEST, stop.signal))) as Error).message).toBe("early");
		expect(played.requests).toHaveLength(1);
		const response = await played.fetch(REQUEST, new AbortController().signal);
		expect(await drain(response.chunks)).toEqual(["a", "b"]);
	});

	it("replays a recorded trace with its chunk edges and no waiting", async () => {
		const played = replayFetch({
			format: 1,
			exchanges: [
				{
					request: {},
					status: 201,
					headers: { "Content-Type": "text/plain" },
					chunks: [btoa("ab"), btoa("c")],
				},
			],
		});
		const response = await played.fetch(REQUEST, new AbortController().signal);
		expect(response.status).toBe(201);
		expect(response.headers).toEqual([["content-type", "text/plain"]]);
		expect(await drain(response.chunks)).toEqual(["ab", "c"]);
	});
});

describe("the fake key source", () => {
	it("answers each read from its list, the last repeating, and counts the reads", async () => {
		const keys = fakeKeys(["one", undefined, "three"]);
		expect([await keys.get(), await keys.get(), await keys.get(), await keys.get()]).toEqual([
			"one",
			undefined,
			"three",
			"three",
		]);
		expect(keys.reads()).toBe(4);
	});

	it("refuses an empty list", () => {
		expect(() => fakeKeys([])).toThrow("fakeKeys needs a value");
	});
});
