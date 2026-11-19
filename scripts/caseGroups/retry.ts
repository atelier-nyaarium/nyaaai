import { NOTICE_LIFETIME, RetryPolicy } from "@nyaaai/agentic/lifecycle/retry";
import type { Random } from "@nyaaai/contract/random";
import { sequenceRandom } from "@nyaaai/contract/random";
import { z } from "@nyaaai/contract/shape";
import { RetryOptions } from "@nyaaai/contract/shapes/health";
import { NoticeKind } from "@nyaaai/contract/shapes/provider";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { retryCases } from "../caseSources/retry";

export const retry = caseGroup("retry", {
	kinds: {
		/**
		 * One policy over the rows in order, its draws taken from `draws` in turn. `waitMs` is none when the
		 * request stops; `draws` counts what the row took.
		 */
		wait: kind(
			z.object({
				options: RetryOptions,
				draws: z.array(z.number()).min(1),
				rows: z.array(
					z.object({ attempt: z.int(), waitedMs: z.number(), retryAfterMs: z.int().exactOptional() }),
				),
			}),
			z.object({ rows: z.array(z.object({ waitMs: z.number().nullable(), draws: z.int() })) }),
			(input) => {
				const scripted = sequenceRandom(input.draws);
				let taken = 0;
				const random: Random = {
					next: () => {
						taken += 1;
						return scripted.next();
					},
				};
				const policy = new RetryPolicy(input.options, random);
				return {
					rows: input.rows.map((row) => {
						const before = taken;
						const waitMs = policy.wait(
							row.retryAfterMs === undefined ? {} : { retryAfterMs: row.retryAfterMs },
							row.attempt,
							row.waitedMs,
						);
						return { waitMs: waitMs ?? null, draws: taken - before };
					}),
				};
			},
		),
		/** Every notice kind with how its entry is named. */
		lifetime: kind(
			z.object({}),
			z.object({ rows: z.array(z.object({ notice: NoticeKind, lifetime: z.enum(["outcome", "midTurn"]) })) }),
			() => ({
				rows: NoticeKind.options.map((notice) => ({ notice, lifetime: NOTICE_LIFETIME[notice] })),
			}),
		),
	},
	sources: casesFor("retry", retryCases),
});
