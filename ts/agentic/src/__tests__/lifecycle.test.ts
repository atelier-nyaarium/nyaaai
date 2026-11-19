import { expect, it } from "bun:test";
import { systemClock } from "@nyaaai/contract/time";
import { RetryPolicy } from "../lifecycle/retry";
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
	for (let tries = 0; tries < 50; tries += 1) {
		const wait = policy.wait({}, 1, 0) ?? 0;
		expect(wait).toBeGreaterThanOrEqual(500);
		expect(wait).toBeLessThanOrEqual(1000);
	}
});
