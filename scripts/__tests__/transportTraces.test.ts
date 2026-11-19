// The transport trace cases send what the traces recorded.

import { describe, expect, it } from "bun:test";
import path from "node:path";
import { OPENROUTER_CHAT } from "@nyaaai/agentic/dialects/openrouter";
import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import type { CanonicalRequest } from "@nyaaai/contract/shapes/step";
import { readTrace } from "@nyaaai/testkit/trace";
import { httpCases } from "../caseSources/transportHttp";
import { ROOT } from "../child";

////////////////////////////////
//  Functions & Helpers

interface Call {
	kind: "stream" | "complete";
	request: CanonicalRequest;
}

interface Input {
	script: { file: string };
	calls: Call[];
}

////////////////////////////////
//  Tests

describe("the recorded transport cases", () => {
	const traced = httpCases.filter((held) => held.name.startsWith("trace-"));

	it("hold the three traces", () => {
		expect(traced.length).toBeGreaterThan(0);
	});

	for (const held of traced) {
		it(`${held.name} encodes each request to the recorded body`, async () => {
			const input = held.input as Input;
			const trace = await readTrace(path.join(ROOT, input.script.file));
			expect(input.calls).toHaveLength(trace.exchanges.length);
			input.calls.forEach((call, at) => {
				const sent = OPENROUTER_CHAT.encode(call.request, call.kind).body;
				expect(parseJson(stringifyJson(sent))).toEqual(trace.exchanges[at]?.request);
			});
		});
	}
});
