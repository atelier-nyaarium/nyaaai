import {
	ALLOWANCE_SOURCES,
	BALANCE_COOL_MS,
	PROBE_FLOOR_MS,
	probeAt,
	step,
	verdict,
} from "@nyaaai/agentic/health/pool";
import { z } from "@nyaaai/contract/shape";
import { PoolEvent, PoolState, PoolVerdict, Ticket } from "@nyaaai/contract/shapes/health";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { poolCases } from "../caseSources/pool";

export const pool = caseGroup("pool", {
	kinds: {
		/** Each run folds its steps over its start state; `states` holds the state after each step. */
		step: kind(
			z.object({
				pauseMs: z.number(),
				runs: z.array(
					z.object({
						start: PoolState,
						steps: z.array(z.object({ event: PoolEvent, nowMs: z.number() })),
					}),
				),
			}),
			z.object({ runs: z.array(z.object({ states: z.array(PoolState) })) }),
			(input) => ({
				runs: input.runs.map((run) => {
					let state = run.start;
					return {
						states: run.steps.map((held) => {
							state = step(state, held.event, held.nowMs, input.pauseMs);
							return state;
						}),
					};
				}),
			}),
		),
		/** Null: no verdict, the route may go. */
		verdict: kind(
			z.object({ rows: z.array(z.object({ state: PoolState, exempt: z.boolean(), nowMs: z.number() })) }),
			z.object({ verdicts: z.array(PoolVerdict.nullable()) }),
			(input) => ({ verdicts: input.rows.map((row) => verdict(row.state, row.exempt, row.nowMs) ?? null) }),
		),
		/**
		 * `verdict` and an `admitted` event, both at `nowMs` of the same state. `admitted` is the state after
		 * the ticket is announced.
		 */
		instant: kind(
			z.object({
				rows: z.array(z.object({ state: PoolState, exempt: z.boolean(), nowMs: z.number(), ticket: Ticket })),
			}),
			z.object({ rows: z.array(z.object({ verdict: PoolVerdict.nullable(), admitted: PoolState })) }),
			(input) => ({
				rows: input.rows.map((row) => ({
					verdict: verdict(row.state, row.exempt, row.nowMs) ?? null,
					admitted: step(row.state, { kind: "admitted", ticket: row.ticket }, row.nowMs, 0),
				})),
			}),
		),
		/** The probe time that halves the remaining wait, and the constants it stands on. */
		probeAt: kind(
			z.object({ rows: z.array(z.object({ nowMs: z.number(), eta: z.number() })) }),
			z.object({
				times: z.array(z.number()),
				balanceCoolMs: z.number(),
				probeFloorMs: z.number(),
				allowanceSources: z.array(z.string()),
			}),
			(input) => ({
				times: input.rows.map((row) => probeAt(row.nowMs, row.eta)),
				balanceCoolMs: BALANCE_COOL_MS,
				probeFloorMs: PROBE_FLOOR_MS,
				allowanceSources: [...ALLOWANCE_SOURCES],
			}),
		),
	},
	sources: casesFor("pool", poolCases),
});
