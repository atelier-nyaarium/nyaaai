import { describe, expect, it } from "bun:test";
import type { TraceEntry } from "@nyaaai/contract/shapes/lifecycle";
import type { NormalizedUsage } from "@nyaaai/contract/shapes/step";
import { systemClock } from "@nyaaai/contract/time";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { type Call, RequestLifecycle } from "../lifecycle/lifecycle";
import { RetryPolicy } from "../lifecycle/retry";
import type { Candidates, Resolved } from "../provider/resolver";
import { price, stepCost } from "../usage/cost";
import { dayOf } from "../usage/record";

it("the local day is the platform's own, for every quarter hour of a year in this machine's zone", () => {
	const wrong: number[] = [];
	for (let at = Date.UTC(2025, 11, 31); at < Date.UTC(2027, 0, 2); at += 15 * 60_000) {
		const local = new Date(at);
		const text = (n: number) => String(n).padStart(2, "0");
		const platform = `${local.getFullYear()}-${text(local.getMonth() + 1)}-${text(local.getDate())}`;
		if (dayOf(at, systemClock) !== platform) wrong.push(at);
	}
	expect(wrong).toEqual([]);
});

it("a cost that is not a number is not a price, and an allowance reply with one still records zero", () => {
	expect([Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY].map(price)).toEqual([
		undefined,
		undefined,
		undefined,
	]);
	const usage = { input: 100_000, output: 10_000, billingSource: "go-plus", cost: Number.NaN };
	expect(stepCost(usage)).toEqual({ usd: 0, estimated: false });
});

it("a retry policy with no source of chance draws from the system's", () => {
	const policy = new RetryPolicy();
	const wait = policy.wait({}, 1, 0) ?? 0;
	expect(wait).toBeGreaterThanOrEqual(500);
	expect(wait).toBeLessThanOrEqual(1000);
});

/** A route with only what a lifecycle with no gate reads. */
function routeOf(): Candidates {
	const entry = {
		id: "a",
		kind: "openrouter",
		label: "Alpha",
	};
	const route = {
		ok: true,
		key: "m",
		entry,
		kind: { failure: () => ({ act: "retry" }) },
		view: { row: { label: "M" }, status: "ok", efforts: [], hosts: [] },
		zdrOnly: false,
	} as unknown as Resolved;
	return [route];
}

type Reply = { usage?: NormalizedUsage };

const callOf = (send: Call<Reply>["send"]): Call<Reply> => ({
	tag: { feature: "chat" },
	signal: new AbortController().signal,
	send,
	read: () => ({}),
});

describe("the lifecycle's trace", () => {
	const scheduler = fakeScheduler(new TimerWheel(0));

	it("is not altered by a sink or a meter that throws", async () => {
		const sunk = await new RequestLifecycle({ scheduler }).run(
			routeOf(),
			callOf(async () => ({})),
			{
				onTrace: () => {
					throw new Error("The sink broke.");
				},
			},
		);
		expect(sunk.ok).toBe(true);

		const told: TraceEntry[] = [];
		const lifecycle = new RequestLifecycle({
			scheduler,
			meter: {
				called: () => {
					throw new Error("The meter broke.");
				},
			},
		});
		const metered = await lifecycle.run(
			routeOf(),
			callOf(async () => ({})),
			{ onTrace: (entry) => told.push(entry) },
		);
		expect(metered.ok).toBe(true);
		expect(told.map((entry) => entry.kind)).toEqual(["attempt", "metered", "served"]);
	});

	it("falls through a route the gate gave the request up on when a route follows, and ends on the last", async () => {
		const told: TraceEntry[] = [];
		const [route] = routeOf();
		const second = { ...route, entry: { id: "b", kind: "openrouter", label: "Beta" } } as unknown as Resolved;
		const gate = {
			admit: async () => ({ kind: "gaveUp" as const }),
			tryAdmit: () => null,
			settle: () => "done" as const,
			deferred: () => undefined,
			blocked: () => undefined,
			failed: () => false,
			served: () => false,
		};
		const lifecycle = new RequestLifecycle({ scheduler, gate });
		const result = await lifecycle.run(
			[route as Resolved, second],
			callOf(async () => ({})),
			{ onTrace: (entry) => told.push(entry) },
		);
		expect(result.ok).toBe(false);
		expect(
			told.map((entry) => (entry.kind === "skip" ? `skip:${entry.cause}:${entry.reason}` : entry.kind)),
		).toEqual(["admitted", "gateFailed", "skip:fellThrough:overloaded", "admitted", "notice"]);
		const live = await lifecycle.run([route as Resolved, second], { ...callOf(async () => ({})), live: true });
		expect(live.ok).toBe(false);
	});
});
