// The conformance fixtures hold every codec to what a case file cannot say across files.

import { describe, expect, it } from "bun:test";
import { runKind } from "../caseAdapter";
import { codec } from "../caseGroups/codec";
import { type Fixture, type Json, request, stepOf } from "../caseSources/codecFixtures";
import { ALL_FIXTURES } from "../caseSources/codecNativeFixtures";

////////////////////////////////
//  Functions & Helpers

type Outcome = {
	events: Array<{ kind: string; usage?: unknown }>;
	result: Json | null;
	failure: Json | null;
	encoded: { body: unknown; headers?: Record<string, string> } | null;
};

async function run(kind: string, input: Json): Promise<Outcome> {
	const held = codec.kinds[kind];
	if (held === undefined) throw new Error(`no kind ${kind}`);
	return (await runKind(held, input)) as Outcome;
}

/** Every text under `key`, wherever it sits. */
function found(body: unknown, key: string): string[] {
	if (Array.isArray(body)) return body.flatMap((each) => found(each, key));
	if (typeof body !== "object" || body === null) return [];
	const record = body as Json;
	const own = typeof record[key] === "string" ? [record[key] as string] : [];
	return [...own, ...Object.values(record).flatMap((each) => found(each, key))];
}

/** What each fixture's thought state looks like on the wire. */
const REPLAYS: Record<
	string,
	{ read: (body: unknown) => unknown[]; state: unknown[]; session: (e: Outcome) => unknown }
> = {
	openrouter: {
		read: (body) =>
			((body as { messages: Json[] }).messages ?? []).flatMap((message) =>
				message["role"] === "assistant"
					? ((message["reasoning_details"] as Json[] | undefined) ?? []).map(
							(detail) => detail["signature"] ?? detail["data"],
						)
					: [],
			),
		state: ["sig_1", "enc_2"],
		session: (e) => ((e.encoded?.body ?? {}) as { session_id?: string }).session_id,
	},
	opencode: {
		read: (body) => found(body, "reasoning_content").filter((each) => each.length > 0),
		state: ["Thinking."],
		session: (e) => e.encoded?.headers?.["x-opencode-session"],
	},
	local: {
		read: (body) => found(body, "reasoning"),
		state: ["Thinking."],
		session: () => undefined,
	},
	responses: {
		read: (body) => found(body, "encrypted_content"),
		state: ["enc_1", "enc_2"],
		session: (e) => e.encoded?.headers?.["x-opencode-session"],
	},
	messages: {
		read: (body) => [...found(body, "signature"), ...found(body, "data")],
		state: ["sig_1", "red_2"],
		session: (e) => e.encoded?.headers?.["x-opencode-session"],
	},
	gemini: {
		read: (body) => found(body, "thoughtSignature").filter((each) => each !== "skip_thought_signature_validator"),
		state: ["sig_g"],
		session: (e) => e.encoded?.headers?.["x-opencode-session"],
	},
};

/** The in-stream failure each fixture ends on. */
const FAILING: Record<string, { category: string; billed: Json }> = {
	openrouter: {
		category: "overloaded",
		billed: { usage: { input: 4, output: 2, cost: 0.1 }, host: "Azure", generationId: "gen-1" },
	},
	opencode: { category: "overloaded", billed: { usage: { input: 4, output: 2, cost: 0.1 }, generationId: "gen-1" } },
	local: { category: "stream", billed: { host: "Lab", generationId: "gen-1" } },
	responses: {
		category: "overloaded",
		billed: { usage: { input: 4, output: 2, cost: 0.1 }, generationId: "resp_1" },
	},
	messages: {
		category: "overloaded",
		billed: { usage: { input: 100, output: 1, cached: 60 }, generationId: "msg_1" },
	},
	gemini: {
		category: "overloaded",
		billed: { usage: { input: 100, output: 0, cached: 60 }, generationId: "resp_g" },
	},
};

////////////////////////////////
//  Tests

describe.each(ALL_FIXTURES)("the $name fixture", (fixture: Fixture) => {
	it("streams to its step, usage and cost last, and a whole reply assembles to the same step", async () => {
		const streamed = await run("stream", { codec: fixture.codec, items: fixture.stream });
		const usages = streamed.events.flatMap((event) => (event.kind === "usage" ? [event.usage] : []));
		expect(streamed.result).toEqual(fixture.result);
		expect(usages.at(-1)).toEqual(fixture.result["usage"]);
		expect(typeof (fixture.result["usage"] as Json)["cost"]).toBe("number");
		expect((await run("complete", { codec: fixture.codec, body: fixture.reply })).result).toEqual(fixture.result);
	});

	it("dies as truncated or garbled, and keeps what was billed", async () => {
		const failing = await run("stream", { codec: fixture.codec, items: fixture.failing });
		expect({
			cut: (await run("stream", { codec: fixture.codec, items: fixture.cut })).failure?.["category"],
			garbled: (await run("stream", { codec: fixture.codec, items: [{ kind: "data", data: "{not json" }] }))
				.failure?.["category"],
			failed: failing.failure,
		}).toEqual({
			cut: "truncated",
			garbled: "stream",
			failed: expect.objectContaining({ ...FAILING[fixture.name], status: "stream" }),
		});
	});

	it("decodes HTTP failures by status, then type", async () => {
		const decoded = await Promise.all(
			fixture.failures.map(async ([status, body]) => {
				const held = codec.kinds["failure"];
				if (held === undefined) throw new Error("no failure kind");
				const out = (await runKind(held, { codec: fixture.codec, status, body, headers: [] })) as {
					failure: { category: string };
				};
				return [status, out.failure.category];
			}),
		);
		expect(decoded).toEqual(fixture.failures.map(([status, , category]) => [status, category]));
	});

	it("sends decoded thoughts back to a model that reads them, and the session along", async () => {
		const replay = REPLAYS[fixture.name];
		if (replay === undefined) throw new Error("no replay row");
		const history = [{ role: "user", text: "Hi" }, stepOf(fixture.result, fixture.model)];
		const encode = (model: string) =>
			run("encode", { codec: fixture.codec, request: request({ model, history }), mode: "stream" });
		const own = await encode(fixture.model);
		const foreign = await encode(fixture.foreign);
		expect({
			own: replay.read(own.encoded?.body),
			foreign: replay.read(foreign.encoded?.body),
			session: replay.session(own),
		}).toEqual({
			own: replay.state,
			foreign: [],
			session: fixture.name === "local" ? undefined : "root-1",
		});
	});
});
