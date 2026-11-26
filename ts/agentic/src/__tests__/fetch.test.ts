// The platform Fetch against a loopback provider.

import { describe, expect, it } from "bun:test";
import { base64Encode } from "@nyaaai/contract/base64";
import { BUDGETS, BudgetExceeded } from "@nyaaai/contract/fetch";
import { headerOf } from "@nyaaai/contract/headers";
import { systemScheduler } from "@nyaaai/contract/time";
import { type Loopback, type LoopbackHold, loopbackProvider } from "@nyaaai/testkit/loopback";
import { rejection } from "@nyaaai/testkit/rejection";
import type { Trace, TraceExchange } from "@nyaaai/testkit/trace";
import { platformFetch } from "../fetch";

////////////////////////////////
//  Functions & Helpers

const encoded = (text: string): string => base64Encode(new TextEncoder().encode(text));

const exchange = (patch: Partial<TraceExchange> = {}): TraceExchange => ({
	request: null,
	status: 200,
	headers: { "content-type": "text/event-stream" },
	chunks: [],
	...patch,
});

async function serving<T>(
	exchanges: TraceExchange[],
	holds: readonly (LoopbackHold | undefined)[],
	use: (provider: Loopback) => Promise<T>,
): Promise<T> {
	const trace: Trace = { format: 1, exchanges };
	const provider = await loopbackProvider(trace, holds);
	try {
		return await use(provider);
	} finally {
		await provider.close();
	}
}

async function joined(chunks: AsyncIterable<Uint8Array>): Promise<string> {
	const parts: string[] = [];
	const decoder = new TextDecoder();
	for await (const chunk of chunks) parts.push(decoder.decode(chunk, { stream: true }));
	return parts.join("");
}

const post = (provider: Loopback, body: string, path = "/v1/chat") => ({
	method: "POST" as const,
	url: `${provider.url}${path}`,
	headers: [["content-type", "application/json"]] as Array<[string, string]>,
	body,
	budget: BUDGETS.catalog,
});

const fetched = platformFetch(systemScheduler);

////////////////////////////////
//  Tests

describe("the platform fetch", () => {
	it("answers the status, the headers in response form and the joined bytes", async () => {
		const chunks = [encoded("data: one\n\n"), encoded("data: tw"), encoded("o \u{1F600}\n\n")];
		await serving(
			[exchange({ headers: { "Content-Type": "text/event-stream", "X-RateLimit-Remaining": "3" }, chunks })],
			[],
			async (provider) => {
				const answer = await fetched(post(provider, '{"turn":1}'), new AbortController().signal);
				expect(answer.status).toBe(200);
				const names = answer.headers.map(([name]) => name);
				expect(names).toEqual([...names].sort());
				expect(names.every((name) => name === name.toLowerCase())).toBe(true);
				expect(headerOf(answer.headers, "content-type")).toBe("text/event-stream");
				expect(headerOf(answer.headers, "x-ratelimit-remaining")).toBe("3");
				expect(await joined(answer.chunks)).toBe("data: one\n\ndata: two \u{1F600}\n\n");
			},
		);
	});

	it("sends the request body and headers as given, and no body on a GET", async () => {
		await serving([exchange(), exchange()], [], async (provider) => {
			const signal = new AbortController().signal;
			await joined(
				(await fetched({ ...post(provider, '{"model":"m","n":[1,2]}'), headers: [["x-lab", "1"]] }, signal))
					.chunks,
			);
			await joined(
				(
					await fetched(
						{
							method: "GET",
							url: `${provider.url}/v1/models`,
							headers: [["x-lab", "2"]],
							budget: BUDGETS.catalog,
						},
						signal,
					)
				).chunks,
			);
			expect(provider.requests).toEqual([{ model: "m", n: [1, 2] }, null]);
			expect(provider.headers.map((held) => held["x-lab"])).toEqual(["1", "2"]);
		});
	});

	it("answers no chunks for a response with no body", async () => {
		await serving([exchange({ status: 204, headers: {} })], [], async (provider) => {
			const answer = await fetched(post(provider, "{}"), new AbortController().signal);
			expect(answer.status).toBe(204);
			expect(await joined(answer.chunks)).toBe("");
		});
	});

	it("runs on a fetch function it is given", async () => {
		const asked: string[] = [];
		const given: typeof fetch = Object.assign(
			async (input: Parameters<typeof fetch>[0]) => {
				asked.push(String(input));
				return new Response("held", { status: 201, headers: { "X-B": "2", "x-a": "1" } });
			},
			{ preconnect: fetch.preconnect },
		);
		const answer = await platformFetch(systemScheduler, given)(
			{ method: "GET", url: "http://lab.local/x", headers: [], budget: BUDGETS.catalog },
			new AbortController().signal,
		);
		const parts: Uint8Array[] = [];
		for await (const chunk of answer.chunks) parts.push(chunk);
		expect(asked).toEqual(["http://lab.local/x"]);
		expect(answer.status).toBe(201);
		expect(answer.headers.filter(([name]) => name.startsWith("x-"))).toEqual([
			["x-a", "1"],
			["x-b", "2"],
		]);
		expect(new TextDecoder().decode(parts[0])).toBe("held");
	});

	// A key must never follow a redirect to another origin.
	it("answers a 307 to another origin as its status, and the other origin sees no request", async () => {
		await serving([exchange({ chunks: [encoded("there")] })], [], async (other) => {
			const trace: Trace = {
				format: 1,
				exchanges: [exchange({ status: 307, headers: { location: `${other.url}/next` } })],
			};
			const provider = await loopbackProvider(trace);
			try {
				const answer = await fetched(post(provider, '{"prompt":"p"}'), new AbortController().signal);
				expect(answer.status).toBe(307);
				expect(headerOf(answer.headers, "location")).toBe(`${other.url}/next`);
				await joined(answer.chunks);
				expect(provider.requests).toEqual([{ prompt: "p" }]);
				expect(other.requests).toEqual([]);
			} finally {
				await provider.close();
			}
		});
	});

	it("rejects with the signal's reason when aborted before the head, and the server sees the close", async () => {
		await serving([exchange()], ["head"], async (provider) => {
			const controller = new AbortController();
			const call = fetched(post(provider, "{}"), controller.signal);
			await provider.arrived(0);
			const reason = new Error("stop now");
			controller.abort(reason);
			expect(await rejection(call)).toBe(reason);
			await provider.closed(0);
		});
	});

	it("rejects at once when the signal is already aborted", async () => {
		await serving([exchange()], [], async (provider) => {
			const controller = new AbortController();
			const reason = new Error("early");
			controller.abort(reason);
			expect(await rejection(fetched(post(provider, "{}"), controller.signal))).toBe(reason);
			expect(provider.requests).toEqual([]);
		});
	});

	it("rejects the read with the signal's reason when aborted mid-body, and the server sees the close", async () => {
		await serving([exchange({ chunks: [encoded("first")] })], ["chunks"], async (provider) => {
			const controller = new AbortController();
			const answer = await fetched(post(provider, "{}"), controller.signal);
			const reader = answer.chunks[Symbol.asyncIterator]();
			expect(new TextDecoder().decode((await reader.next()).value)).toBe("first");
			const pending = reader.next();
			const reason = new Error("stop mid-body");
			controller.abort(reason);
			expect(await rejection(pending)).toBe(reason);
			await provider.closed(0);
		});
	});

	// No case reaches the platform adapter.
	it("ends a silent body at its idle bound, and the server sees the close", async () => {
		await serving([exchange({ chunks: [encoded("first")] })], ["chunks"], async (provider) => {
			const answer = await fetched(
				{ ...post(provider, "{}"), budget: { ...BUDGETS.catalog, idleMs: 50 } },
				new AbortController().signal,
			);
			const reader = answer.chunks[Symbol.asyncIterator]();
			await reader.next();
			const reason = await rejection(reader.next());
			expect(reason).toBeInstanceOf(BudgetExceeded);
			expect((reason as BudgetExceeded).bound).toBe("idle");
			await provider.closed(0);
		});
	});

	it("releases the connection when the reader stops early", async () => {
		await serving([exchange({ chunks: [encoded("first")] })], ["chunks"], async (provider) => {
			const answer = await fetched(post(provider, "{}"), new AbortController().signal);
			const reader = answer.chunks[Symbol.asyncIterator]();
			await reader.next();
			await reader.return?.();
			await provider.closed(0);
		});
	});

	it("rejects when the connection is refused", async () => {
		const provider = await loopbackProvider({ format: 1, exchanges: [] });
		const url = provider.url;
		await provider.close();
		const reason = await rejection(
			fetched({ method: "GET", url, headers: [], budget: BUDGETS.catalog }, new AbortController().signal),
		);
		expect(reason).toBeInstanceOf(Error);
	});
});
