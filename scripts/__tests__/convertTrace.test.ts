// Trace scrubber tests.

import { describe, expect, it } from "bun:test";
import { base64Decode } from "@nyaaai/contract/base64";
import { convertTrace } from "../convertTrace";

////////////////////////////////
//  Functions & Helpers

const text = (chunk: string): string => new TextDecoder().decode(base64Decode(chunk));

////////////////////////////////
//  Tests

describe("trace scrubber", () => {
	it("scrubs session and generation ids by key and in every string, and keeps replay headers", () => {
		const trace = convertTrace([
			{
				source: "http",
				data: {
					request: {
						url: "/v1/gen-12-abC",
						body: { session_id: "real-1", nested: [{ session_id: "real-2" }] },
					},
				},
			},
			{
				source: "http",
				data: { status: 200, headers: { "content-type": "text/event-stream", "set-cookie": "secret" } },
			},
			{ source: "http", data: { line: 'data: { "id": "gen-7-xYz", "session_id" : "real-3" }' } },
			{ source: "http", data: { line: "data: [DONE]" } },
		]);
		const [exchange] = trace.exchanges;
		expect(JSON.stringify(exchange?.request)).toBe(
			'{"url":"/v1/gen-0-recorded","body":{"session_id":"00000000-0000-4000-8000-000000000000","nested":[{"session_id":"00000000-0000-4000-8000-000000000000"}]}}',
		);
		expect(exchange?.headers).toEqual({ "content-type": "text/event-stream" });
		expect(text(exchange?.chunks[0] as string)).toBe(
			'data: {"id":"gen-0-recorded","session_id":"00000000-0000-4000-8000-000000000000"}\n\ndata: [DONE]\n\n',
		);
	});

	it("replaces a session id it has seen wherever it occurs", () => {
		const trace = convertTrace([
			{ source: "http", data: { request: { session_id: "session-secret-123", sid: "session-secret-123" } } },
			{ source: "http", data: { request: { "x-session-secret-123": 1 } } },
			{ source: "http", data: { line: "data: session=session-secret-123" } },
		]);
		const [first, second] = trace.exchanges;
		const zeros = "00000000-0000-4000-8000-000000000000";
		expect(JSON.stringify(first?.request)).toBe(`{"session_id":"${zeros}","sid":"${zeros}"}`);
		expect(JSON.stringify(second?.request)).toBe(`{"x-${zeros}":1}`);
		expect(text(second?.chunks[0] as string)).toBe(`data: session=${zeros}\n\n`);
	});
});
