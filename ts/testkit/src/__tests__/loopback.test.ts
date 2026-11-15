// Loopback provider tests.

import { describe, expect, it } from "bun:test";
import { base64Encode } from "@nyaaai/contract/base64";
import { loopbackProvider } from "../loopback";
import type { Trace } from "../trace";

////////////////////////////////
//  Functions & Helpers

const bytes = (text: string): string => base64Encode(new TextEncoder().encode(text));

////////////////////////////////
//  Tests

describe("the loopback provider", () => {
	it("answers each request with the next exchange, then refuses", async () => {
		const trace: Trace = {
			format: 1,
			exchanges: [
				{
					request: null,
					status: 200,
					headers: { "content-type": "text/event-stream", "x-ratelimit-remaining": "3" },
					chunks: [bytes("data: one\n\n"), "", bytes("data: tw"), bytes("o \u{1F600}\n\n")],
				},
				{ request: null, status: 429, headers: { "retry-after": "2" }, chunks: [bytes('{"error":"slow"}')] },
			],
		};
		const provider = await loopbackProvider(trace);
		try {
			const post = (body: unknown) =>
				fetch(`${provider.url}/v1/chat`, { method: "POST", body: JSON.stringify(body) });
			const first = await post({ turn: 1 });
			expect([
				first.status,
				first.headers.get("content-type"),
				first.headers.get("x-ratelimit-remaining"),
			]).toEqual([200, "text/event-stream", "3"]);
			expect(await first.text()).toBe("data: one\n\ndata: two \u{1F600}\n\n");

			const second = await post({ turn: 2 });
			expect([second.status, second.headers.get("retry-after"), await second.text()]).toEqual([
				429,
				"2",
				'{"error":"slow"}',
			]);

			expect((await post("late")).status).toBe(500);
			expect(provider.requests).toEqual([{ turn: 1 }, { turn: 2 }, "late"]);
		} finally {
			await provider.close();
		}
	});
});
