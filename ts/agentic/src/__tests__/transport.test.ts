// Transport tests the cases cannot carry.

import { describe, expect, it } from "bun:test";
import type { Fetch } from "@nyaaai/contract/fetch";
import type { CanonicalRequest } from "@nyaaai/contract/shapes/step";
import { fakeScheduler } from "@nyaaai/testkit/scheduler";
import { OPENROUTER_CHAT } from "../dialects/openrouter";
import { failureOf } from "../failure";
import { staticKey, Transport } from "../transport";

////////////////////////////////
//  Functions & Helpers

const REQUEST: CanonicalRequest = {
	model: "openai/gpt-6-luna",
	system: [],
	history: [{ role: "user", text: "Hi" }],
	cache: { kind: "none" },
	session: "root-1",
	zdrOnly: true,
};

function transportOver(fetch: Fetch): Transport {
	return new Transport({
		fetch,
		scheduler: fakeScheduler(),
		key: staticKey("key-test-123"),
		log: () => {},
		baseUrl: "https://lab.example.test/v1",
		label: "Lab",
		userAgent: "lab-app/1.0",
		errors: OPENROUTER_CHAT,
	});
}

const rejecting =
	(held: unknown): Fetch =>
	async () => {
		throw held;
	};

async function failureAfter(fetch: Fetch): Promise<ReturnType<typeof failureOf>> {
	const made = transportOver(fetch);
	try {
		await made.complete(OPENROUTER_CHAT, REQUEST, new AbortController().signal);
	} catch (error) {
		return failureOf(error);
	}
	throw new Error("the call did not fail");
}

////////////////////////////////
//  Tests

describe("the transport", () => {
	it("tells a listener the model of each reply until it is removed", async () => {
		const empty: Fetch = async () => ({ status: 500, headers: [], chunks: (async function* () {})() });
		const made = transportOver(empty);
		const told: string[] = [];
		const other: string[] = [];
		const off = made.onSettled((model) => told.push(model));
		made.onSettled((model) => other.push(model));
		const run = () => made.complete(OPENROUTER_CHAT, REQUEST, new AbortController().signal).catch(() => {});
		await run();
		expect(off()).toBe(true);
		expect(off()).toBe(false);
		await run();
		expect(told).toEqual(["openai/gpt-6-luna"]);
		expect(other).toEqual(["openai/gpt-6-luna", "openai/gpt-6-luna"]);
	});

	it("reads a thrown value that is no Error as its text", async () => {
		expect((await failureAfter(rejecting("boom")))?.message).toBe("boom");
		expect((await failureAfter(rejecting(5)))?.message).toBe("5");
		expect((await failureAfter(rejecting({})))?.message).toBe("[object Object]");
		expect((await failureAfter(rejecting(new Error("down"))))?.category).toBe("network");
	});

	it("answers a caller's abort with the caller's own reason, not a failure", async () => {
		const made = transportOver(async (_request, signal) => {
			throw signal.reason;
		});
		const stop = new AbortController();
		stop.abort("stopped");
		const error = await made.complete(OPENROUTER_CHAT, REQUEST, stop.signal).catch((held) => held);
		expect(error).toBe("stopped");
	});

	it("holds a static key", async () => {
		expect(await staticKey("k").get()).toBe("k");
	});
});
