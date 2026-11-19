// Request cost cases.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Helpers

type Usage = Record<string, unknown>;
type Price = Record<string, unknown> | "free" | "unknown";

interface Row {
	usage?: Usage;
	price?: Price;
	longCache?: boolean;
}

const tier = (minPromptTokens: number, prompt: number, completion: number, extra: Record<string, number> = {}) => ({
	minPromptTokens,
	prompt,
	completion,
	...extra,
});

const priced = (...tiers: object[]): Price => ({ prompt: [0, 0], completion: [0, 0], tiers });

const step = (name: string, rows: Row[]): CaseSource => ({
	kind: "cost.step",
	name,
	input: {
		rows: rows.map((row) => ({
			...(row.usage === undefined ? {} : { usage: row.usage }),
			...(row.price === undefined ? {} : { price: row.price }),
			longCache: row.longCache ?? false,
		})),
	},
});

/** A listing of two tiers. */
const TIERS = priced(tier(0, 0.1, 0.5, { cacheRead: 0.01 }), tier(272_000, 0.2, 0.75, { cacheRead: 0.02 }));

const MAX = 1.7976931348623157e308;

const USAGE: Usage = { input: 100_000, output: 10_000 };

/** A call at one tier whose sum changes with the order of its operations. */
const ordered = (name: string, usage: Usage, prices: { p: number; c: number; cr: number; cw: number }): CaseSource =>
	step(name, [
		{ usage, price: priced(tier(0, prices.p, prices.c, { cacheRead: prices.cr, cacheWrite: prices.cw })) },
	]);

////////////////////////////////
//  Cases

export const costCases: CaseSource[] = [
	// A free model bills nothing, whatever it reported.
	step("free-model", [
		{ usage: { input: 10, output: 5 }, price: "free" },
		{ usage: { input: 10, output: 5, cost: 0 }, price: "free" },
		{ usage: { input: 10, output: 5, cost: 0.25 }, price: "free" },
		{ usage: { input: 10, output: 5, cost: 0.25, byok: true, upstreamCost: 1 }, price: "free" },
		{ price: "free" },
		{ usage: { input: 10, output: 5, billingSource: "balance" }, price: "free", longCache: true },
	]),
	// A reported cost is exact.
	step("reported-cost", [
		{ usage: { input: 10, output: 5, cost: 0.25 }, price: TIERS },
		{ usage: { input: 10, output: 5, cost: 0.25 } },
		{ usage: { input: 10, output: 5, cost: 0 }, price: TIERS },
		{ usage: { input: 10, output: 5, cost: 0.25 }, price: "unknown" },
		{ usage: { input: 10, output: 5, cost: 0.25 }, price: priced() },
		{ usage: { input: 10, output: 5, cost: 1e-9 } },
		{ usage: { input: 10, output: 5, cost: 1e300 } },
	]),
	step("reported-cost-not-usable", [
		{ usage: { input: 100_000, output: 10_000, cost: -1 }, price: TIERS },
		{ usage: { input: 100_000, output: 10_000, cost: -0.000001 }, price: TIERS },
		{ usage: { input: 100_000, output: 10_000, cost: -1 } },
		{ usage: { input: 100_000, output: 10_000, cost: -1 }, price: "unknown" },
		{ usage: { input: 100_000, output: 10_000, cost: -1, byok: true, upstreamCost: 1 }, price: TIERS },
	]),
	// BYOK adds the provider's charge.
	step("byok-reported", [
		{
			usage: {
				input: 1_000,
				output: 50,
				byok: true,
				cost: 0.01,
				upstreamCost: 0.2,
				cached: 400,
				cacheWrite: 100,
			},
		},
		{
			usage: {
				input: 1_000,
				output: 50,
				byok: false,
				cost: 0.2,
				upstreamCost: 0.2,
				cached: 400,
				cacheWrite: 100,
			},
		},
		{ usage: { input: 1_000, output: 50, cost: 0.2, upstreamCost: 0.2 } },
		{ usage: { input: 1_000, output: 50, byok: true, cost: 0.01, upstreamCost: 0 } },
		{ usage: { input: 1_000, output: 50, byok: true, cost: 0, upstreamCost: 0.5 } },
		{ usage: { input: 1_000, output: 50, byok: true, cost: 0, upstreamCost: 0 } },
		{
			usage: {
				input: 3_000,
				output: 150,
				byok: true,
				cost: 0.01 + 0.2 + 0.01,
				upstreamCost: 0.2 + 0.2,
				cached: 1_200,
				cacheWrite: 300,
			},
		},
		{ usage: { input: 1_000, output: 50, byok: true, cost: 0.1, upstreamCost: 0.2 } },
		{ usage: { input: 1_000, output: 50, byok: true, cost: 0.1, upstreamCost: 0.2 }, price: "unknown" },
	]),
	// A BYOK reply missing the provider's charge is not reported: the fee is kept and the tokens are priced.
	step("byok-charge-missing", [
		{ usage: { input: 3_000, output: 75, cost: 0.03, byok: true }, price: TIERS },
		{ usage: { input: 3_000, output: 75, cost: 0.03, byok: true } },
		{ usage: { input: 3_000, output: 75, cost: 0.03, byok: true, upstreamCost: -1 }, price: TIERS },
		{ usage: { input: 3_000, output: 75, cost: 0.03, byok: true }, price: "unknown" },
		{ usage: { input: 3_000, output: 75, byok: true }, price: TIERS },
		{ usage: { input: 3_000, output: 75, cost: -1, byok: true }, price: TIERS },
		{ usage: { input: 3_000, output: 75, cost: 0.03, byok: true, billingSource: "go" }, price: TIERS },
		{ usage: { input: 3_000, output: 75, cost: 0.03, byok: true }, price: priced(tier(5_000, 1, 1)) },
		{ usage: { input: 3_000, output: 75, cost: 0.03, byok: false }, price: TIERS },
	]),
	step("byok-fee-and-cache-writes", [
		{
			usage: { input: 1_000_000, output: 100_000, cost: 0.01, byok: true, cached: 400_000, cacheWrite: 200_000 },
			price: priced(tier(0, 2, 10, { cacheRead: 0.1, cacheWrite: 2.5, cacheWrite1h: 4 })),
		},
		{
			usage: { input: 1_000_000, output: 100_000, cost: 0.01, byok: true, cached: 400_000, cacheWrite: 200_000 },
			price: priced(tier(0, 2, 10, { cacheRead: 0.1, cacheWrite: 2.5, cacheWrite1h: 4 })),
			longCache: true,
		},
	]),
	// Unknown prices.
	step("unknown-price", [
		{ usage: USAGE, price: "unknown" },
		{ usage: { ...USAGE, cost: 0.25 }, price: "unknown" },
		{ usage: { ...USAGE, billingSource: "balance" }, price: "unknown" },
		{ usage: { ...USAGE, billingSource: "go" }, price: "unknown" },
		{ price: "unknown" },
		{ usage: { ...USAGE, byok: true, cost: 0.1 }, price: "unknown" },
	]),
	// No usage.
	step("no-usage", [
		{},
		{ price: TIERS },
		{ price: priced() },
		{ price: "unknown" },
		{ price: "free" },
		{ longCache: true },
	]),
	// An allowance or free reply without a valid cost records zero; balance still estimates.
	step("unbilled-sources", [
		{ usage: { ...USAGE, billingSource: "go" }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "go-plus", cost: -1 }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "free", cost: -1 }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "balance" }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "go", cost: 0.5 }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "go", byok: true, cost: 0.5 }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "go", byok: true, cost: 0.5, upstreamCost: 1 }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "go" } },
		{ usage: { ...USAGE, billingSource: "free" } },
		{ usage: { ...USAGE, billingSource: "go-plus" } },
	]),
	step("billed-sources", [
		{ usage: { ...USAGE, billingSource: "GO" }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "" }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "go-" }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "payg" }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "balance", byok: true, cost: 0.5 }, price: TIERS },
		{ usage: { ...USAGE, billingSource: "free " }, price: TIERS },
	]),
	// No price listed.
	step("not-listed", [
		{ usage: USAGE },
		{ usage: USAGE, price: priced() },
		{ usage: { ...USAGE, byok: true, cost: 0.02 } },
		{ usage: { ...USAGE, byok: true, cost: 0.02 }, price: priced() },
		{ usage: { ...USAGE, byok: false, cost: -3 } },
		{ usage: { input: 0, output: 0 } },
	]),
	// Estimates at one tier.
	step("estimate", [
		{ usage: { input: 100_000, output: 10_000 }, price: TIERS },
		{ usage: { input: 1_000_000, output: 100_000, cached: 400_000 }, price: TIERS },
		{ usage: { input: 0, output: 0 }, price: TIERS },
		{ usage: { input: 1, output: 0 }, price: TIERS },
		{ usage: { input: 0, output: 1 }, price: TIERS },
		{ usage: { input: 1_000_000, output: 1_000_000 }, price: priced(tier(0, 1, 1)) },
		{ usage: { input: 3, output: 7 }, price: priced(tier(0, 0.5, 0.25)) },
		{ usage: { input: 3, output: 7 }, price: priced(tier(0, 0, 0)) },
	]),
	// Tier boundaries: the last listed tier whose floor the prompt reaches.
	step("tier-boundaries", [
		{ usage: { input: 271_999, output: 1_000 }, price: TIERS },
		{ usage: { input: 272_000, output: 1_000 }, price: TIERS },
		{ usage: { input: 272_001, output: 1_000 }, price: TIERS },
		{ usage: { input: 0, output: 1_000 }, price: TIERS },
		{ usage: { input: 1, output: 1_000 }, price: priced(tier(1, 1, 1)) },
		{ usage: { input: 0, output: 1_000 }, price: priced(tier(1, 1, 1)) },
		{ usage: { input: 1_000_000_000, output: 1_000 }, price: TIERS },
	]),
	step("tier-below-every-floor", [
		{ usage: { input: 100, output: 10 }, price: priced(tier(101, 1, 1), tier(500, 2, 2)) },
		{ usage: { input: 100, output: 10, cost: -1, byok: true }, price: priced(tier(101, 1, 1)) },
		{ usage: { input: 100, output: 10, byok: true, cost: 0.4 }, price: priced(tier(101, 1, 1)) },
		{ usage: { input: 100, output: 10, byok: true, cost: 0.4, cached: 50 }, price: priced(tier(101, 1, 1)) },
	]),
	step("tiers-out-of-order", [
		{ usage: { input: 300_000, output: 1_000 }, price: priced(tier(272_000, 0.2, 0.75), tier(0, 0.1, 0.5)) },
		{ usage: { input: 300_000, output: 1_000 }, price: priced(tier(0, 0.1, 0.5), tier(272_000, 0.2, 0.75)) },
		{
			usage: { input: 300_000, output: 1_000 },
			price: priced(tier(200_000, 3, 3), tier(100_000, 2, 2), tier(0, 1, 1)),
		},
		{
			usage: { input: 150_000, output: 1_000 },
			price: priced(tier(200_000, 3, 3), tier(100_000, 2, 2), tier(0, 1, 1)),
		},
		{
			usage: { input: 150_000, output: 1_000 },
			price: priced(tier(100_000, 2, 2), tier(100_000, 5, 5), tier(0, 1, 1)),
		},
		{
			usage: { input: 150_000, output: 1_000 },
			price: priced(tier(100_000, 2, 2), tier(100_000, 5, 5), tier(300_000, 1, 1)),
		},
	]),
	// Cache reads and writes, and what each does above the prompt.
	step("cache-counts", [
		{
			usage: { input: 1_000, output: 100, cached: 400 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 400, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 1_000 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 1_000, cacheWrite: 50 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 0, cacheWrite: 0 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
	]),
	step("cache-above-the-prompt", [
		{
			usage: { input: 1_000, output: 100, cached: 5_000 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cacheWrite: 5_000 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 600, cacheWrite: 600 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 5_000, cacheWrite: 5_000 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 0, output: 100, cached: 5, cacheWrite: 5 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: -200 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 100, cacheWrite: -50 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
	]),
	// A missing cache price falls back to the prompt price.
	step("cache-price-missing", [
		{ usage: { input: 1_000, output: 100, cached: 400, cacheWrite: 300 }, price: priced(tier(0, 1, 2)) },
		{
			usage: { input: 1_000, output: 100, cached: 400, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 400, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 400, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheRead: 0, cacheWrite: 0 })),
		},
	]),
	step("long-cache-price", [
		{
			usage: { input: 1_000, output: 100, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheWrite: 1.25, cacheWrite1h: 2 })),
		},
		{
			usage: { input: 1_000, output: 100, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheWrite: 1.25, cacheWrite1h: 2 })),
			longCache: true,
		},
		{
			usage: { input: 1_000, output: 100, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheWrite: 1.25 })),
			longCache: true,
		},
		{ usage: { input: 1_000, output: 100, cacheWrite: 300 }, price: priced(tier(0, 1, 2, { cacheWrite1h: 2 })) },
		{
			usage: { input: 1_000, output: 100, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheWrite1h: 2 })),
			longCache: true,
		},
		{ usage: { input: 1_000, output: 100, cacheWrite: 300 }, price: priced(tier(0, 1, 2)), longCache: true },
		{
			usage: { input: 1_000, output: 100, cached: 400 },
			price: priced(tier(0, 1, 2, { cacheWrite: 1.25, cacheWrite1h: 2 })),
			longCache: true,
		},
		{
			usage: { input: 1_000, output: 100, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheWrite: 1.25, cacheWrite1h: 0 })),
			longCache: true,
		},
		{ usage: { input: 1_000, output: 100, cacheWrite: 300 }, price: "unknown", longCache: true },
		{
			usage: { input: 1_000, output: 100, cacheWrite: 300, cost: 0.5 },
			price: priced(tier(0, 1, 2, { cacheWrite1h: 2 })),
			longCache: true,
		},
	]),
	// The per-tier arithmetic: each tier carries its own cache prices.
	step("cache-prices-by-tier", [
		{
			usage: { input: 300_000, output: 1_000, cached: 100_000, cacheWrite: 50_000 },
			price: priced(
				tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25, cacheWrite1h: 2 }),
				tier(200_000, 2, 4, { cacheRead: 0.2, cacheWrite: 2.5 }),
			),
			longCache: true,
		},
		{
			usage: { input: 150_000, output: 1_000, cached: 100_000, cacheWrite: 50_000 },
			price: priced(
				tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25, cacheWrite1h: 2 }),
				tier(200_000, 2, 4, { cacheRead: 0.2, cacheWrite: 2.5 }),
			),
			longCache: true,
		},
	]),
	// The order of operations: each call differs under another order of the same sums.
	ordered(
		"order-with-a-fee-one",
		{ input: 175_317, output: 2_107, cached: 49_243, cacheWrite: 5_886, byok: true, cost: 0.003 },
		{ p: 18.75, cr: 7.5, cw: 0.075, c: 0.6 },
	),
	ordered(
		"order-with-a-fee-two",
		{ input: 194_413, output: 4_008, cached: 110_203, cacheWrite: 36_711, byok: true, cost: 0.1 },
		{ p: 0.6, cr: 3.75, cw: 0.3, c: 0.075 },
	),
	ordered(
		"order-with-a-fee-three",
		{ input: 139_459, output: 10_735, cached: 53_602, cacheWrite: 704, byok: true, cost: 0.1 },
		{ p: 0.6, cr: 2.5, cw: 1.1, c: 0.28 },
	),
	ordered(
		"order-without-a-fee-one",
		{ input: 237_822, output: 16_807, cached: 76_181, cacheWrite: 437 },
		{ p: 0.3, cr: 7.5, cw: 0.3, c: 0.0375 },
	),
	ordered(
		"order-without-a-fee-two",
		{ input: 282_065, output: 6_419, cached: 129_581, cacheWrite: 59_542 },
		{ p: 0.0375, cr: 0.0375, cw: 0.0375, c: 0.6 },
	),
	// A cost is a finite number at or above zero.
	step("byok-sum-overflows", [
		{ usage: { input: 3_000, output: 75, cost: MAX, upstreamCost: MAX, byok: true }, price: TIERS },
		{ usage: { input: 3_000, output: 75, cost: MAX, upstreamCost: MAX, byok: true } },
		{ usage: { input: 3_000, output: 75, cost: 1e308, upstreamCost: 1e308, byok: true }, price: TIERS },
		{ usage: { input: 3_000, output: 75, cost: 1e308, upstreamCost: 7e307, byok: true }, price: TIERS },
		{ usage: { input: 3_000, output: 75, cost: MAX, upstreamCost: 0, byok: true }, price: TIERS },
	]),
	step("estimate-overflows", [
		{ usage: { input: 1e10, output: 1 }, price: priced(tier(0, 1e300, 1)) },
		{ usage: { input: 1e300, output: 1e300 }, price: priced(tier(0, 1e300, 1e300)) },
		{ usage: { input: 1e10, output: 1, byok: true, cost: 0.5 }, price: priced(tier(0, 1e300, 1)) },
		{ usage: { input: 10, output: 1 }, price: priced(tier(0, MAX, MAX)) },
		{ usage: { input: 1, output: 1, byok: true, cost: MAX }, price: priced(tier(0, MAX, MAX)) },
		{ usage: { input: 1, output: 0, byok: true, cost: MAX }, price: priced(tier(0, 1, 1)) },
	]),
	step("negative-prices", [
		{ usage: { input: 1_000, output: 100 }, price: priced(tier(0, -1, -1)) },
		{ usage: { input: 1_000, output: 100 }, price: priced(tier(0, -1, 1)) },
		{ usage: { input: 1_000, output: 100 }, price: priced(tier(0, 1, -1)) },
		{ usage: { input: 1_000, output: 100 }, price: priced(tier(0, 0, 0)) },
		{ usage: { input: 1_000, output: 100, byok: true, cost: 0.5 }, price: priced(tier(0, -1, -1)) },
		{ usage: { input: 1_000, output: 100 }, price: priced(tier(0, 1, 2), tier(500, -1, -1)) },
		{ usage: { input: 1_000, output: 100 }, price: priced(tier(0, 1, 2), tier(500, 3, -1), tier(900, 4, 4)) },
		{ usage: { input: 1_000, output: 100 }, price: priced(tier(0, -1, 2), tier(2_000, 1, 1)) },
	]),
	step("negative-cache-prices-read-as-missing", [
		{
			usage: { input: 1_000, output: 100, cached: 400, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheRead: -0.1 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 400, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheWrite: -1 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 400, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheWrite: 1.25, cacheWrite1h: -2 })),
			longCache: true,
		},
		{
			usage: { input: 1_000, output: 100, cached: 400, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheWrite: -1, cacheWrite1h: 2 })),
			longCache: true,
		},
		{
			usage: { input: 1_000, output: 100, cached: 400, cacheWrite: 300 },
			price: priced(tier(0, 1, 2, { cacheRead: 1e308 })),
		},
	]),
	step("negative-counts-read-as-zero", [
		{ usage: { input: -5, output: 100 }, price: TIERS },
		{ usage: { input: 1_000, output: -100 }, price: TIERS },
		{ usage: { input: -1_000, output: -100 }, price: TIERS },
		{ usage: { input: 1_000, output: 100, cached: -200 }, price: TIERS },
		{ usage: { input: 1_000, output: 100, cacheWrite: -200 }, price: TIERS },
		{ usage: { input: -5, output: 100, byok: true, cost: 0.5 }, price: TIERS },
		{ usage: { input: -5, output: 100 }, price: priced(tier(-10, 1, 1)) },
		{ usage: { input: -5, output: 100 }, price: priced(tier(-3, 1, 1)) },
		{ usage: { input: -5, output: 0 }, price: priced(tier(0, 1, 1)) },
	]),
	step("cached-and-written-above-the-prompt", [
		{
			usage: { input: 1_000, output: 100, cached: 1_000, cacheWrite: 1_000 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 800, cacheWrite: 800 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 500, cacheWrite: 500 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 1_001, cacheWrite: 1 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
		{
			usage: { input: 1_000, output: 100, cached: 0, cacheWrite: 1_001 },
			price: priced(tier(0, 1, 2, { cacheRead: 0.1, cacheWrite: 1.25 })),
		},
	]),
	// Rows of the kinds below.
	{
		kind: "cost.price",
		name: "numbers",
		input: {
			values: [
				0, 1, 0.5, 1e-9, 5e-324, 1e308, 1.7976931348623157e308, 123456789.12345679, -1, -0.5, -1e-300, -1e308,
			],
		},
	},
	{
		kind: "cost.price",
		name: "not-numbers",
		input: {
			values: ["0", "1.5", "", "NaN", "Infinity", null, true, false, [], [1], {}, { value: 1 }, "-1", " 1"],
		},
	},
	{
		kind: "cost.reported",
		name: "combinations",
		input: {
			rows: [
				{},
				{ cost: 0.25 },
				{ cost: 0 },
				{ cost: -1 },
				{ byok: true },
				{ byok: true, cost: 0.01 },
				{ byok: true, cost: 0.01, upstreamCost: 0.2 },
				{ byok: true, cost: 0.01, upstreamCost: 0 },
				{ byok: true, cost: 0, upstreamCost: 0 },
				{ byok: true, cost: 0.01, upstreamCost: -0.2 },
				{ byok: true, upstreamCost: 0.2 },
				{ byok: true, cost: -1, upstreamCost: 0.2 },
				{ byok: false, cost: 0.01, upstreamCost: 0.2 },
				{ cost: 0.01, upstreamCost: 0.2 },
				{ byok: true, cost: 0.1, upstreamCost: 0.2 },
				{ byok: true, cost: 0.1, upstreamCost: 0.7 },
				{ byok: true, cost: 1e308, upstreamCost: 1e308 },
				{ byok: true, cost: MAX, upstreamCost: MAX },
				{ byok: true, cost: MAX, upstreamCost: 0 },
				{ byok: true, cost: 1e308, upstreamCost: 7e307 },
				{ byok: true, cost: 1e308, upstreamCost: 8e307 },
			],
		},
	},
	{
		kind: "cost.longCache",
		name: "per-feature",
		input: {
			rows: [
				"chat",
				"analysis",
				"wand",
				"tester",
				"prediction",
				"compaction",
				"Chat",
				"CHAT",
				" chat",
				"chat ",
				"",
				"chat\u0000",
			].flatMap((feature) => [null, {}, { cacheWrite1h: true }].map((view) => ({ tag: { feature }, view }))),
		},
	},
	{
		kind: "cost.longCache",
		name: "chat-with-a-lane-and-a-folder",
		input: {
			rows: [
				{ tag: { feature: "chat", lane: "title", folder: "/work/app" }, view: { cacheWrite1h: true } },
				{ tag: { feature: "chat", lane: "", folder: "" }, view: { cacheWrite1h: true } },
				{ tag: { feature: "analysis", lane: "chat", folder: "chat" }, view: { cacheWrite1h: true } },
			],
		},
	},
];
