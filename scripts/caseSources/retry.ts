// Retry wait cases.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Helpers

interface Options {
	tries?: number;
	budgetMs?: number;
	capMs?: number;
	baseMs?: number;
}

interface Row {
	attempt: number;
	waitedMs?: number;
	retryAfterMs?: number;
}

/** The largest draw below one. */
const TOP = 0.9999999999999999;

const wait = (name: string, options: Options, draws: number[], rows: Row[]): CaseSource => ({
	kind: "retry.wait",
	name,
	input: {
		options,
		draws,
		rows: rows.map((row) => ({
			attempt: row.attempt,
			waitedMs: row.waitedMs ?? 0,
			...(row.retryAfterMs === undefined ? {} : { retryAfterMs: row.retryAfterMs }),
		})),
	},
});

const attempts = (...numbers: number[]): Row[] => numbers.map((attempt) => ({ attempt }));

////////////////////////////////
//  Cases

export const retryCases: CaseSource[] = [
	{ kind: "retry.lifetime", name: "every-kind", input: {} },
	// Four tries by default: attempts one to three wait.
	wait("default-low-draw", {}, [0], attempts(1, 2, 3, 4, 5)),
	wait("default-mid-draw", {}, [0.5], attempts(1, 2, 3, 4)),
	wait("default-top-draw", {}, [TOP], attempts(1, 2, 3, 4)),
	wait("draw-of-one", {}, [1], attempts(1, 2, 3, 4)),
	wait("capped-low", { capMs: 3000 }, [0], attempts(1, 2, 3, 4)),
	wait("capped-high", { capMs: 3000 }, [1], attempts(1, 2, 3, 4)),
	// The default cap is reached from the sixth attempt.
	wait("default-cap-reached", { tries: 10, budgetMs: 1_000_000 }, [1], attempts(1, 2, 3, 4, 5, 6, 7, 8, 9, 10)),
	wait("default-cap-low-draw", { tries: 10, budgetMs: 1_000_000 }, [0], attempts(1, 2, 3, 4, 5, 6, 7, 8, 9)),
	wait("draws-cycle", {}, [0, 1, 0.5], attempts(1, 1, 1, 1, 1, 1, 1)),
	wait("two-draws-over-three-rows", {}, [0.25, 0.75], attempts(1, 2, 3)),
	wait("attempt-zero-and-below", { tries: 4 }, [1], attempts(0, -1, -2)),
	wait("huge-attempt", { tries: 5000 }, [1], attempts(60, 1100, 4999)),
	wait("custom-base", { baseMs: 250, tries: 6 }, [0.5], attempts(1, 2, 3, 4, 5)),
	wait("fractional-base", { baseMs: 0.5, tries: 3 }, [0], attempts(1, 2)),
	// Rounding is half up.
	wait("rounds-half-up", { baseMs: 3 }, [0], attempts(1)),
	wait("rounds-half-up-again", { baseMs: 5 }, [0], attempts(1)),
	wait("rounds-down-below-half", { baseMs: 2.5 }, [0], attempts(1)),
	wait("rounds-up-above-half", { baseMs: 1 }, [0.99], attempts(1)),
	wait("custom-tries", { tries: 2 }, [0.5], attempts(1, 2, 3)),
	wait("one-try", { tries: 1 }, [0.5], attempts(0, 1, 2)),
	// Retry-After replaces the backoff, still drawn for, and is not capped.
	wait("retry-after-under-budget", {}, [1], [{ attempt: 1, retryAfterMs: 45_000 }]),
	wait(
		"retry-after-replaces-backoff",
		{},
		[0],
		[
			{ attempt: 1, retryAfterMs: 7 },
			{ attempt: 2, retryAfterMs: 7 },
			{ attempt: 3, retryAfterMs: 7 },
		],
	),
	wait(
		"retry-after-draws-once",
		{},
		[0, 1],
		[{ attempt: 1, retryAfterMs: 100 }, { attempt: 1 }, { attempt: 1, retryAfterMs: 100 }, { attempt: 1 }],
	),
	wait(
		"retry-after-at-budget",
		{},
		[1],
		[
			{ attempt: 1, retryAfterMs: 45_000, waitedMs: 45_000 },
			{ attempt: 1, retryAfterMs: 90_000, waitedMs: 0 },
		],
	),
	wait(
		"retry-after-over-budget",
		{},
		[1],
		[
			{ attempt: 2, retryAfterMs: 45_000, waitedMs: 45_001 },
			{ attempt: 1, retryAfterMs: 90_001, waitedMs: 0 },
			{ attempt: 1, retryAfterMs: 1, waitedMs: 90_000 },
		],
	),
	wait(
		"retry-after-past-cap",
		{ capMs: 1000 },
		[1],
		[
			{ attempt: 1, retryAfterMs: 60_000 },
			{ attempt: 2, retryAfterMs: 5000 },
		],
	),
	wait(
		"retry-after-zero",
		{},
		[1],
		[
			{ attempt: 1, retryAfterMs: 0 },
			{ attempt: 3, retryAfterMs: 0, waitedMs: 90_000 },
		],
	),
	wait(
		"retry-after-on-the-last-attempt",
		{},
		[1],
		[
			{ attempt: 4, retryAfterMs: 1 },
			{ attempt: 5, retryAfterMs: 1 },
		],
	),
	// The budget counts what was waited; the backoff that meets it is kept.
	wait(
		"budget-met-exactly",
		{ budgetMs: 1000 },
		[1],
		[
			{ attempt: 1, waitedMs: 0 },
			{ attempt: 1, waitedMs: 1 },
		],
	),
	wait("budget-low-draw", { budgetMs: 500 }, [0], [{ attempt: 1 }, { attempt: 1, waitedMs: 1 }]),
	wait(
		"budget-spent-by-waits",
		{},
		[1],
		[
			{ attempt: 3, waitedMs: 85_000 },
			{ attempt: 3, waitedMs: 86_000 },
			{ attempt: 3, waitedMs: 86_001 },
			{ attempt: 3, waitedMs: 90_000 },
		],
	),
	wait("backoff-over-budget", { baseMs: 100_000, capMs: 200_000 }, [1], attempts(1, 2)),
	wait("negative-waited", { budgetMs: 1000 }, [1], [{ attempt: 1, waitedMs: -5000 }]),
	// Options at zero.
	wait("no-tries", { tries: 0 }, [0.5], attempts(0, 1, 2)),
	wait("no-budget", { budgetMs: 0 }, [1], [{ attempt: 1 }, { attempt: 1, retryAfterMs: 0 }]),
	wait("no-cap", { capMs: 0, tries: 4 }, [1], attempts(1, 2, 3)),
	wait("no-base", { baseMs: 0, tries: 4 }, [1], attempts(1, 2, 3)),
	wait("no-base-huge-attempt", { baseMs: 0, tries: 5000, budgetMs: 10 }, [0, 1], attempts(1, 60, 1100, 4999)),
	// A wait is a finite number at or above zero, or none.
	wait(
		"negative-retry-after",
		{},
		[1],
		[
			{ attempt: 1, retryAfterMs: -5 },
			{ attempt: 1, retryAfterMs: -1, waitedMs: 90_000 },
			{ attempt: 1, retryAfterMs: -90_000, waitedMs: 90_001 },
			{ attempt: 4, retryAfterMs: -5 },
		],
	),
	wait("negative-wait-from-a-draw-below-zero", {}, [-1e300, -1e308, 1e308, 0.5], attempts(1, 1, 1, 1)),
	wait(
		"a-wait-too-large-to-be-a-number",
		{ capMs: 1e308, budgetMs: 1e308, tries: 3 },
		[1e308, 0.5],
		attempts(2, 2, 2),
	),
	wait("negative-zero-wait", { baseMs: 1 }, [-1], attempts(1)),
	wait("all-zero", { tries: 0, budgetMs: 0, capMs: 0, baseMs: 0 }, [0], attempts(0, 1)),
	wait("no-rows", {}, [0], []),
];
