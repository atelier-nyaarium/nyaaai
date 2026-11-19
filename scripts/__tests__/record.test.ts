// The recorder's parts, without a network.

import { describe, expect, it } from "bun:test";
import { readdirSync } from "node:fs";
import path from "node:path";
import { base64Decode, base64Encode } from "@nyaaai/contract/base64";
import type { Fetch } from "@nyaaai/contract/fetch";
import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import { recordingFetch } from "@nyaaai/testkit/recorder";
import { rejection } from "@nyaaai/testkit/rejection";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedFetch } from "@nyaaai/testkit/scriptedFetch";
import { readTrace, replayFetch, type Trace } from "@nyaaai/testkit/trace";
import { ROOT } from "../child";
import { recordScenario, SCENARIOS } from "../record";
import { replaceTraceIds, SESSION } from "../traceIds";

////////////////////////////////
//  Functions & Helpers

const KEY = "fake-key-one";
const TRACES = path.join(ROOT, "cases", "traces");
const bytes = (text: string): string => base64Encode(new TextEncoder().encode(text));
const text = (chunk: string): string => new TextDecoder().decode(base64Decode(chunk));
const signal = new AbortController().signal;

/** Sends each exchange's request to a Fetch and drains the reply. */
async function replayThrough(fetch: Fetch, trace: Trace): Promise<void> {
	for (const held of trace.exchanges) {
		const answer = await fetch(
			held.request === null
				? { method: "GET", url: "http://lab.test/v1", headers: [] }
				: { method: "POST", url: "http://lab.test/v1", headers: [], body: stringifyJson(held.request) },
			signal,
		);
		for await (const chunk of answer.chunks) void chunk;
	}
}

////////////////////////////////
//  Tests

describe("recording a replay", () => {
	const files = readdirSync(TRACES)
		.filter((name) => name.endsWith(".json"))
		.sort();

	for (const file of files) {
		it(`${file} comes back equal, chunk edges included`, async () => {
			const trace = await readTrace(path.join(TRACES, file));
			const scenario = SCENARIOS.find((held) => file === `openrouter.${held.name}.json`);
			if (scenario) {
				const played = await recordScenario(scenario, replayFetch(trace).fetch, KEY, SESSION, fakeScheduler());
				expect(parseJson(played)).toEqual(trace);
			} else {
				const recorder = recordingFetch(replayFetch(trace).fetch, [KEY]);
				await replayThrough(recorder.fetch, trace);
				expect(recorder.trace()).toEqual(trace);
			}
		});
	}

	it("refuses a recording whose reply echoes the key, and says so without the key", async () => {
		const title = SCENARIOS.find((held) => held.name === "title");
		if (!title) throw new Error("no title scenario");
		const echoing = scriptedFetch(
			[
				{
					status: 200,
					headers: [["content-type", "application/json"]],
					chunks: [
						{
							bytes: new TextEncoder().encode(
								`{"id":"${KEY}","choices":[{"index":0,"finish_reason":"stop","message":{"role":"assistant","content":"{\\"title\\":\\"Tally\\"}"}}]}`,
							),
						},
					],
				},
			],
			new TimerWheel(),
		);
		const reason = await rejection(recordScenario(title, echoing.fetch, KEY, SESSION, fakeScheduler()));
		expect((reason as Error).message).not.toContain(KEY);
	});
});

describe("id replacement", () => {
	const REAL = "5f0c2a54-1d47-4b9e-8d6b-0a1b2c3d4e5f";
	const trace = (chunks: string[]): Trace => ({
		format: 1,
		exchanges: [{ request: { session_id: REAL, n: 1 }, status: 200, headers: {}, chunks: chunks.map(bytes) }],
	});

	it("replaces session and generation ids inside their chunks and keeps the edges", () => {
		const held = replaceTraceIds(
			trace([
				`data: {"id":"gen-12-abC","session_id":"${REAL}"}\n\n`,
				'data: {"note":"caf\u{e9}"}\n\n',
				`data: {"again":"${REAL}"}\n\n`,
			]),
		);
		expect(held.exchanges[0]?.request).toEqual({ session_id: SESSION, n: 1 });
		expect(held.exchanges[0]?.chunks.map(text)).toEqual([
			`data: {"id":"gen-0-recorded","session_id":"${SESSION}"}\n\n`,
			'data: {"note":"caf\u{e9}"}\n\n',
			`data: {"again":"${SESSION}"}\n\n`,
		]);
	});

	it("keeps a character split across an edge whole", () => {
		const raw = new TextEncoder().encode("data: \u{1F600}\n\n");
		const held = replaceTraceIds({
			format: 1,
			exchanges: [
				{
					request: null,
					status: 200,
					headers: {},
					chunks: [base64Encode(raw.slice(0, 8)), base64Encode(raw.slice(8))],
				},
			],
		});
		expect(held.exchanges[0]?.chunks.map((chunk) => base64Decode(chunk))).toEqual([raw.slice(0, 8), raw.slice(8)]);
	});

	it("fails a recording whose session or generation id crosses a chunk edge", () => {
		const whole = `data: {"session_id":"${REAL}"}\n\n`;
		const cut = whole.indexOf(REAL) + 10;
		expect(() => replaceTraceIds(trace([whole.slice(0, cut), whole.slice(cut)]))).toThrow();
		expect(() => replaceTraceIds(trace(['data: {"id":"gen-12-', 'abC"}\n\n']))).toThrow();
	});
});
