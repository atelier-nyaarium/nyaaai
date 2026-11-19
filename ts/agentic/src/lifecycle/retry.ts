// How long a request waits before it tries a route again.

import { roundHalfUp } from "@nyaaai/contract/number";
import { type Random, systemRandom } from "@nyaaai/contract/random";
import type { RetryOptions } from "@nyaaai/contract/shapes/health";
import type { NoticeKind } from "@nyaaai/contract/shapes/provider";
import type { Failure } from "@nyaaai/contract/shapes/step";

////////////////////////////////
//  Constants

/** Mid-turn notices stand alone; run-ending notices share one entry. */
export const NOTICE_LIFETIME: Record<NoticeKind, "outcome" | "midTurn"> = {
	keyMissing: "outcome",
	keyRejected: "outcome",
	spendLimit: "outcome",
	refused: "outcome",
	noZdrHost: "outcome",
	modelRetired: "outcome",
	modelUnavailable: "outcome",
	unreachable: "outcome",
	badRequest: "outcome",
	retriesExhausted: "outcome",
	providerOverloaded: "outcome",
	streamFailed: "outcome",
	outputCut: "outcome",
	noModel: "outcome",
	historyDropped: "midTurn",
};

////////////////////////////////
//  Classes

/** Four tries; 90s wait cap. */
export class RetryPolicy {
	private readonly tries: number;
	private readonly budgetMs: number;
	private readonly capMs: number;
	private readonly baseMs: number;

	constructor(
		options: RetryOptions = {},
		private readonly random: Random = systemRandom,
	) {
		this.tries = options.tries ?? 4;
		this.budgetMs = options.budgetMs ?? 90_000;
		this.capMs = options.capMs ?? 30_000;
		this.baseMs = options.baseMs ?? 1000;
	}

	/** One draw per call that gets past the tries, even when `retryAfterMs` replaces the backoff. A `baseMs` of 0 backs off 0 at any attempt. */
	wait(failure: Pick<Failure, "retryAfterMs">, attempt: number, waitedMs: number): number | undefined {
		if (attempt >= this.tries) return undefined;
		const base = this.baseMs === 0 ? 0 : Math.min(this.capMs, this.baseMs * 2 ** (attempt - 1));
		const backoff = base * (0.5 + this.random.next() / 2);
		const waitMs = roundHalfUp(failure.retryAfterMs ?? backoff);
		return waitedMs + waitMs > this.budgetMs ? undefined : waitMs;
	}
}
