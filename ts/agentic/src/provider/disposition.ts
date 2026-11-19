// What a run does with a failure.

import type { ModelView } from "@nyaaai/contract/shapes/model";
import { type Disposition, SkipReason } from "@nyaaai/contract/shapes/provider";
import type { Failure } from "@nyaaai/contract/shapes/step";

////////////////////////////////
//  Constants

export const SKIP_REASONS: readonly SkipReason[] = SkipReason.options;

////////////////////////////////
//  Functions & Helpers

/** Retry, fall-through, or stop. */
export function dispositionOf(failure: Failure, view: ModelView | undefined): Disposition {
	const category = failure.category;
	switch (category) {
		case "keyMissing":
		case "keyRejected":
			return { act: "fallThrough", notice: category };
		case "refused":
			return { act: "stop", notice: "refused" };
		case "badRequest":
		case "replayRejected":
			return { act: "stop", notice: "badRequest" };
		// Retry-After spends budget.
		case "outOfCredit":
			return failure.retryAfterMs === undefined ? { act: "fallThrough", notice: "spendLimit" } : { act: "retry" };
		// Resets are hours away.
		case "allowance":
			return { act: "fallThrough", notice: "spendLimit" };
		case "notFound":
			if (view?.status === "noZdrHost") return { act: "fallThrough", notice: "noZdrHost" };
			if (view?.status === "retired") return { act: "fallThrough", notice: "modelRetired" };
			return { act: "fallThrough", notice: "badRequest" };
		case "rateLimited":
		case "overloaded":
		case "unavailable":
		case "network":
		case "stream":
		case "empty":
		case "truncated":
		case "idle":
			return { act: "retry" };
		default: {
			const unhandled: never = category;
			throw new Error(`Unhandled failure category ${unhandled}.`);
		}
	}
}

export function skipReasonOf(failure: Failure): SkipReason {
	switch (failure.category) {
		case "keyMissing":
		case "keyRejected":
		case "allowance":
		case "overloaded":
		case "network":
			return failure.category;
		case "outOfCredit":
			return "credit";
		case "notFound":
			return "model";
		default:
			return "unavailable";
	}
}
