// The effort ladder.

import type { Effort } from "@nyaaai/contract/shapes/step";

////////////////////////////////
//  Constants

/** Ascending thresholds. */
export const EFFORTS: readonly Effort[] = ["none", "minimal", "low", "medium", "high", "xhigh", "max"];

////////////////////////////////
//  Functions & Helpers

/** Requested, else highest below, else lowest; fallbacks skip `max`. */
export function resolveEffort<T extends string>(offered: readonly T[], wanted: T): T | undefined {
	if (offered.includes(wanted)) return wanted;
	const rank = (effort: string) => EFFORTS.indexOf(effort as Effort);
	const ranked = offered.filter((effort) => effort !== "max" && rank(effort) >= 0).sort((a, b) => rank(a) - rank(b));
	return ranked.findLast((effort) => rank(effort) < rank(wanted)) ?? ranked[0];
}
