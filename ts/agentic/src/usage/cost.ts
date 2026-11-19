// What one request cost.

import type { ModelView } from "@nyaaai/contract/shapes/model";
import type { NormalizedUsage } from "@nyaaai/contract/shapes/step";
import type { CallTag, Cost } from "@nyaaai/contract/shapes/usage";
import { tiersOf } from "../models/view";

////////////////////////////////
//  Constants

/** No-balance sources. */
const UNBILLED: ReadonlySet<string> = new Set(["free", "go", "go-plus"]);

////////////////////////////////
//  Functions & Helpers

/** A reported amount: finite and not negative. */
export function price(value: unknown): number | undefined {
	return typeof value === "number" && Number.isFinite(value) && value >= 0 ? value : undefined;
}

/** A count below zero, or no number, is zero. */
const count = (value: number): number => (value >= 0 ? value : 0);

/** None when a BYOK reply lacks the provider's charge, or the sum overflows. */
export function reportedCost(usage: Pick<NormalizedUsage, "cost" | "byok" | "upstreamCost">): number | undefined {
	const account = price(usage.cost);
	if (account === undefined || usage.byok !== true) return account;
	const upstream = price(usage.upstreamCost);
	return upstream === undefined ? undefined : price(account + upstream);
}

/** BYOK adds provider charges. Free, or unknown unreported, is unpriced. */
export function stepCost(usage: NormalizedUsage | undefined, view?: Pick<ModelView, "price">, longCache = false): Cost {
	if (view?.price === "free") return { usd: 0, estimated: false, unpriced: true };
	const reported = usage === undefined ? undefined : reportedCost(usage);
	if (reported !== undefined) return { usd: reported, estimated: false };
	if (view?.price === "unknown") return { usd: 0, estimated: false, unpriced: true };
	if (usage === undefined) return { usd: 0, estimated: true };
	if (usage.billingSource !== undefined && UNBILLED.has(usage.billingSource)) return { usd: 0, estimated: false };
	const fee = usage.byok === true ? (price(usage.cost) ?? 0) : 0;
	const input = count(usage.input);
	const priced = (tiersOf(view) ?? []).filter(
		(candidate) => price(candidate.prompt) !== undefined && price(candidate.completion) !== undefined,
	);
	const tier = [...priced].reverse().find((candidate) => candidate.minPromptTokens <= input);
	if (tier === undefined) return { usd: fee, estimated: true };
	const cached = Math.min(input, count(usage.cached ?? 0));
	const written = Math.min(input - cached, count(usage.cacheWrite ?? 0));
	const writePrice = price(longCache ? tier.cacheWrite1h : tier.cacheWrite) ?? tier.prompt;
	const usd =
		((input - cached - written) * tier.prompt +
			cached * (price(tier.cacheRead) ?? tier.prompt) +
			written * writePrice +
			count(usage.output) * tier.completion) /
		1_000_000;
	const total = usd + fee;
	return { usd: Number.isFinite(total) ? total : fee, estimated: true };
}

/** Only user chat pays for 1-hour writes. */
export function wantsLongCache(tag: CallTag, view: Pick<ModelView, "cacheWrite1h"> | undefined): boolean {
	return tag.feature === "chat" && view?.cacheWrite1h === true;
}
