// Pool cooldown cases.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Helpers

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const NOW = 1_000 * MINUTE;

const at = (minutes: number): number => NOW + minutes * MINUTE;

const ticket = (serial: number, extra: Record<string, unknown> = {}) => ({
	gate: "go",
	model: "go\nm",
	era: 1,
	generation: 0,
	probe: false,
	serial,
	...extra,
});

const ONE = ticket(1);
const TWO = ticket(2, { generation: 1 });

type State = Record<string, unknown>;
type Event = Record<string, unknown>;

const CLEAR: State = { kind: "clear" };
const balance = (eta: number, next: number, probe: { ticket: object; at: number } | null = null): State => ({
	kind: "balance",
	eta,
	next,
	probe,
});

const STATES = {
	clear: CLEAR,
	paused: { kind: "paused", reason: "credit", until: at(10) },
	reset: { kind: "reset", at: at(60) },
	open: balance(at(100), at(-1)),
	cooling: balance(at(100), at(10)),
	probing: balance(at(100), at(-1), { ticket: ONE, at: at(-1) }),
	expired: balance(at(-1), at(-2)),
	floored: balance(at(-1), at(4)),
} satisfies Record<string, State>;

const EVENTS = {
	balance: { kind: "billed", source: "balance", plan: "go", extraUsage: false },
	extra: { kind: "billed", source: "balance", plan: "go", extraUsage: true },
	pool: { kind: "billed", source: "go", plan: "go", extraUsage: false },
	payg: { kind: "billed", source: "balance", plan: "payg", extraUsage: false },
	pause: { kind: "refused", reason: "allowance" },
	resets: { kind: "refused", reason: "allowance", resetsAt: at(30) },
	on: { kind: "extraUsage", on: true },
	off: { kind: "extraUsage", on: false, lastBill: "balance" },
	admitted: { kind: "admitted", ticket: TWO },
	settled: { kind: "settled", ticket: ONE },
	other: { kind: "settled", ticket: TWO },
} satisfies Record<string, Event>;

const step = (
	name: string,
	pauseMs: number,
	runs: Array<{ start: State; steps: Array<[Event, number]> }>,
): CaseSource => ({
	kind: "pool.step",
	name,
	input: {
		pauseMs,
		runs: runs.map((run) => ({ start: run.start, steps: run.steps.map(([event, nowMs]) => ({ event, nowMs })) })),
	},
});

/** One run: a start state and the events that follow it, each at its time. */
const chain = (name: string, start: State, steps: Array<[Event, number]>, pauseMs = MINUTE): CaseSource =>
	step(name, pauseMs, [{ start, steps }]);

const billed = (source: string, extraUsage = false, plan: string | null = "go"): Event => ({
	kind: "billed",
	source,
	...(plan === null ? {} : { plan }),
	extraUsage,
});

const refused = (reason: "allowance" | "credit", extra: Record<string, number> = {}): Event => ({
	kind: "refused",
	reason,
	...extra,
});

const instant = (name: string, state: State, times: number[]): CaseSource => ({
	kind: "pool.instant",
	name,
	input: {
		rows: times.flatMap((nowMs) => [false, true].map((exempt) => ({ state, exempt, nowMs, ticket: TWO }))),
	},
});

const verdicts = (name: string, rows: Array<[State, boolean, number]>): CaseSource => ({
	kind: "pool.verdict",
	name,
	input: { rows: rows.map(([state, exempt, nowMs]) => ({ state, exempt, nowMs })) },
});

////////////////////////////////
//  Cases

export const poolCases: CaseSource[] = [
	// Every state against every event at one instant.
	step(
		"matrix",
		MINUTE,
		Object.values(STATES).flatMap((start) =>
			Object.values(EVENTS).map((event) => ({ start, steps: [[event, NOW] as [Event, number]] })),
		),
	),
	// The same matrix a day later: every window has passed.
	step(
		"matrix-a-day-later",
		MINUTE,
		Object.values(STATES).flatMap((start) =>
			Object.values(EVENTS).map((event) => ({ start, steps: [[event, NOW + 24 * HOUR] as [Event, number]] })),
		),
	),
	// Halving.
	chain("halving", CLEAR, [
		[billed("balance"), NOW],
		[billed("balance"), NOW + 150 * MINUTE],
		[billed("balance"), NOW + 225 * MINUTE],
		[billed("balance"), NOW + 262.5 * MINUTE],
		[billed("balance"), NOW + 281.25 * MINUTE],
		[billed("balance"), NOW + 290 * MINUTE],
		[billed("balance"), NOW + 295 * MINUTE],
		[billed("balance"), NOW + 299 * MINUTE],
	]),
	chain("halving-past-eta", CLEAR, [
		[billed("balance"), NOW],
		[billed("balance"), NOW + 5 * HOUR - 1],
		[billed("balance"), NOW + 5 * HOUR],
		[billed("balance"), NOW + 5 * HOUR + 1],
	]),
	// The floor: the remaining wait counts as ten minutes at least.
	chain("floor-short-wait", balance(at(4), at(-5)), [[billed("balance"), NOW]]),
	chain("floor-exactly-ten-minutes", balance(at(10), at(-5)), [[billed("balance"), NOW]]),
	chain("floor-just-above", balance(at(10) + 2, at(-5)), [[billed("balance"), NOW]]),
	chain("floor-at-eta", balance(NOW, at(-5)), [[billed("balance"), NOW]]),
	chain("cooling-keeps-a-probe", balance(at(100), at(-1), { ticket: ONE, at: at(-1) }), [[billed("balance"), NOW]]),
	{
		kind: "pool.probeAt",
		name: "halves-the-wait",
		input: {
			rows: [
				{ nowMs: NOW, eta: at(100) },
				{ nowMs: NOW, eta: at(10) },
				{ nowMs: NOW, eta: at(9) },
				{ nowMs: NOW, eta: NOW },
				{ nowMs: NOW, eta: at(-50) },
				{ nowMs: NOW, eta: at(0) + 1 },
				{ nowMs: 0, eta: 0 },
				{ nowMs: 1, eta: 4 },
			],
		},
	},
	// A known reset.
	chain("known-reset-holds-against-a-balance-bill", CLEAR, [
		[refused("allowance", { resetsAt: at(30) }), NOW],
		[billed("balance"), at(10)],
		[billed("balance"), at(30) - 1],
		[billed("balance"), at(30)],
	]),
	chain("known-reset-then-pool-bill", CLEAR, [
		[refused("allowance", { resetsAt: at(30) }), NOW],
		[billed("go"), at(10)],
	]),
	chain("reset-replaced-by-a-pause", CLEAR, [
		[refused("allowance", { resetsAt: at(30) }), NOW],
		[refused("credit"), at(1)],
		[refused("allowance"), at(2)],
	]),
	chain("reset-from-a-retry-after", CLEAR, [
		[refused("allowance", { retryAfterMs: 5 * MINUTE }), NOW],
		[refused("allowance", { retryAfterMs: 0 }), at(1)],
	]),
	chain("reset-wins-over-a-retry-after", CLEAR, [
		[refused("allowance", { resetsAt: at(40), retryAfterMs: 5 * MINUTE }), NOW],
		[refused("allowance", { resetsAt: 0, retryAfterMs: 5 * MINUTE }), NOW],
	]),
	chain("reset-ten-years-away-ends-in-thirty-one-days", CLEAR, [
		[refused("allowance", { resetsAt: NOW + 3650 * 24 * HOUR }), NOW],
		[refused("allowance", { retryAfterMs: 3650 * 24 * HOUR }), NOW + 32 * 24 * HOUR],
	]),
	chain("credit-ignores-a-reset", CLEAR, [
		[refused("credit", { resetsAt: at(40) }), NOW],
		[refused("credit", { resetsAt: at(40), retryAfterMs: 2 * MINUTE }), NOW],
		[refused("credit"), NOW],
	]),
	chain(
		"pause-uses-the-pause-given",
		CLEAR,
		[
			[refused("allowance"), NOW],
			[refused("credit"), NOW],
		],
		5 * MINUTE,
	),
	chain("pause-of-zero", CLEAR, [[refused("allowance"), NOW]], 0),
	// Pauses never shorten a balance window.
	chain("pause-inside-a-balance-window", balance(at(100), at(10)), [
		[refused("allowance"), NOW],
		[refused("allowance", { retryAfterMs: 30 * MINUTE }), NOW],
		[refused("credit", { retryAfterMs: 200 * MINUTE }), NOW],
		[refused("allowance", { retryAfterMs: 1 }), NOW],
	]),
	chain("pause-at-the-end-of-a-balance-window", balance(at(100), at(10)), [
		[refused("allowance"), at(100) - 1],
		[refused("allowance"), at(100)],
	]),
	chain("pause-when-eta-has-passed-but-next-has-not", balance(at(-1), at(10)), [[refused("allowance"), NOW]]),
	chain("pause-inside-a-window-keeps-the-probe", balance(at(100), at(10), { ticket: ONE, at: at(-1) }), [
		[refused("allowance", { retryAfterMs: 30 * MINUTE }), NOW],
	]),
	chain("a-reset-ends-a-balance-window", balance(at(100), at(10), { ticket: ONE, at: at(-1) }), [
		[refused("allowance", { resetsAt: at(50) }), NOW],
	]),
	// A change of plan.
	chain("plan-changes-are-not-judged", CLEAR, [
		[billed("balance", false, "payg"), NOW],
		[billed("balance", false, null), NOW],
		[billed("balance", false, "go-plus"), NOW],
		[billed("balance", false, "GO"), NOW],
		[billed("balance", false, ""), NOW],
		[billed("balance", false, "go"), NOW],
		[billed("balance", false, "payg"), NOW],
		[billed("go", false, "payg"), NOW],
	]),
	chain("plan-changes-leave-a-cooldown", balance(at(100), at(10)), [
		[billed("go", false, "payg"), NOW],
		[billed("go", false, null), NOW],
		[billed("go", false, "go"), NOW],
	]),
	chain("sources-that-clear-a-balance", balance(at(100), at(10)), [[billed("go-plus"), NOW]]),
	chain("sources-that-do-not-clear-a-balance", balance(at(100), at(10)), [
		[billed("free"), NOW],
		[billed("payg"), NOW],
		[billed("GO"), NOW],
		[billed(""), NOW],
		[billed("Balance"), NOW],
	]),
	chain("pool-bill-leaves-other-states", CLEAR, [
		[billed("go"), NOW],
		[refused("credit"), NOW],
		[billed("go-plus"), NOW],
		[refused("allowance", { resetsAt: at(30) }), NOW],
		[billed("go"), NOW],
	]),
	// Extra usage switched on and off.
	chain("extra-usage-on-clears-a-balance", balance(at(100), at(10), { ticket: ONE, at: at(-1) }), [
		[{ kind: "extraUsage", on: true }, NOW],
		[{ kind: "extraUsage", on: true }, NOW],
	]),
	chain("extra-usage-on-leaves-pauses-and-resets", CLEAR, [
		[refused("credit"), NOW],
		[{ kind: "extraUsage", on: true }, NOW],
		[refused("allowance", { resetsAt: at(30) }), NOW],
		[{ kind: "extraUsage", on: true }, NOW],
	]),
	chain("extra-usage-off-after-a-balance-bill", CLEAR, [
		[{ kind: "extraUsage", on: false, lastBill: "balance" }, NOW],
		[{ kind: "extraUsage", on: false, lastBill: "balance" }, at(150)],
	]),
	chain("extra-usage-off-after-another-bill", CLEAR, [
		[{ kind: "extraUsage", on: false, lastBill: "go" }, NOW],
		[{ kind: "extraUsage", on: false, lastBill: "payg" }, NOW],
		[{ kind: "extraUsage", on: false }, NOW],
		[{ kind: "extraUsage", on: false, lastBill: "" }, NOW],
	]),
	chain("extra-usage-off-under-a-known-reset", { kind: "reset", at: at(60) }, [
		[{ kind: "extraUsage", on: false, lastBill: "balance" }, NOW],
		[{ kind: "extraUsage", on: false, lastBill: "balance" }, at(60)],
	]),
	chain("extra-usage-billed-while-on", CLEAR, [
		[billed("balance", true), NOW],
		[billed("balance", false), NOW],
		[billed("balance", true), at(1)],
	]),
	chain("extra-usage-off-keeps-a-window", balance(at(100), at(10), { ticket: ONE, at: at(-1) }), [
		[{ kind: "extraUsage", on: false, lastBill: "balance" }, NOW],
	]),
	// A probe admitted and settled.
	chain("probe-admitted-and-settled-by-its-own-ticket", balance(at(100), at(-1)), [
		[{ kind: "admitted", ticket: TWO }, NOW],
		[{ kind: "settled", ticket: TWO }, at(1)],
	]),
	chain("probe-settled-by-another-ticket", balance(at(100), at(-1)), [
		[{ kind: "admitted", ticket: TWO }, NOW],
		[{ kind: "settled", ticket: ONE }, at(1)],
		[{ kind: "settled", ticket: ticket(3) }, at(1)],
		[{ kind: "settled", ticket: TWO }, at(2)],
	]),
	chain("probe-settled-keeps-the-next-window", balance(at(100), at(5)), [
		[{ kind: "admitted", ticket: ONE }, at(5)],
		[{ kind: "settled", ticket: ONE }, at(6)],
		[{ kind: "admitted", ticket: TWO }, at(7)],
	]),
	chain("one-probe-at-a-time", balance(at(100), at(-1)), [
		[{ kind: "admitted", ticket: ONE }, NOW],
		[{ kind: "admitted", ticket: TWO }, at(1)],
		[{ kind: "settled", ticket: TWO }, at(2)],
		[{ kind: "settled", ticket: ONE }, at(3)],
		[{ kind: "admitted", ticket: TWO }, at(4)],
	]),
	chain("probe-admitted-before-the-window", balance(at(100), at(10)), [
		[{ kind: "admitted", ticket: ONE }, at(10) - 1],
		[{ kind: "admitted", ticket: ONE }, at(10)],
	]),
	chain("probe-admitted-at-the-end-of-the-window", balance(at(100), at(-1)), [
		[{ kind: "admitted", ticket: ONE }, at(100)],
		[{ kind: "admitted", ticket: ONE }, at(100) - 1],
	]),
	chain("probe-admitted-outside-a-balance", CLEAR, [
		[{ kind: "admitted", ticket: ONE }, NOW],
		[refused("credit"), NOW],
		[{ kind: "admitted", ticket: ONE }, NOW],
		[{ kind: "settled", ticket: ONE }, NOW],
	]),
	// A ticket is the same when every field is equal.
	chain("same-in-every-field-settles", balance(at(100), at(-1), { ticket: ONE, at: at(-1) }), [
		[{ kind: "settled", ticket: ticket(1) }, NOW],
	]),
	chain("same-serial-other-fields-does-not-settle", balance(at(100), at(-1), { ticket: ONE, at: at(-1) }), [
		[{ kind: "settled", ticket: ticket(1, { gate: "other", model: "x", generation: 9, probe: true }) }, NOW],
	]),
	chain("differs-only-in-gate-does-not-settle", balance(at(100), at(-1), { ticket: ONE, at: at(-1) }), [
		[{ kind: "settled", ticket: ticket(1, { gate: "other" }) }, NOW],
	]),
	chain("differs-only-in-probe-does-not-settle", balance(at(100), at(-1), { ticket: ONE, at: at(-1) }), [
		[{ kind: "settled", ticket: ticket(1, { probe: true }) }, NOW],
	]),
	chain("differs-only-in-model-does-not-settle", balance(at(100), at(-1), { ticket: ONE, at: at(-1) }), [
		[{ kind: "settled", ticket: ticket(1, { model: "x" }) }, NOW],
	]),
	chain("differs-only-in-generation-does-not-settle", balance(at(100), at(-1), { ticket: ONE, at: at(-1) }), [
		[{ kind: "settled", ticket: ticket(1, { generation: 4 }) }, NOW],
	]),
	chain("same-fields-other-serial-does-not-settle", balance(at(100), at(-1), { ticket: ONE, at: at(-1) }), [
		[{ kind: "settled", ticket: ticket(7) }, NOW],
	]),
	chain("admitted-ticket-is-kept-whole", balance(at(100), at(-1)), [
		[{ kind: "admitted", ticket: ticket(5, { probe: true, generation: 3, model: "m", gate: "g" }) }, NOW],
	]),
	// A rising bill under a window.
	chain("bill-after-a-probe-settled", balance(at(100), at(-1)), [
		[{ kind: "admitted", ticket: ONE }, NOW],
		[billed("balance"), at(1)],
		[{ kind: "settled", ticket: ONE }, at(2)],
		[billed("balance"), at(3)],
	]),
	chain("bill-past-eta-drops-the-probe", balance(at(10), at(-1), { ticket: ONE, at: at(-1) }), [
		[billed("balance"), at(20)],
	]),
	chain("bill-at-eta-drops-the-probe", balance(at(10), at(-1), { ticket: ONE, at: at(-1) }), [
		[billed("balance"), at(10)],
	]),
	chain("bill-one-before-eta-keeps-the-probe", balance(at(10), at(-1), { ticket: ONE, at: at(-1) }), [
		[billed("balance"), at(10) - 1],
	]),
	// A settlement that never comes holds the entry until eta and no longer.
	chain("lost-settlement-then-a-new-window", balance(at(100), at(-1)), [
		[{ kind: "admitted", ticket: ONE }, NOW],
		[billed("balance"), at(50)],
		[{ kind: "admitted", ticket: TWO }, at(100) - 1],
		[billed("balance"), at(101)],
		[{ kind: "admitted", ticket: TWO }, at(101)],
		[{ kind: "admitted", ticket: TWO }, at(101) + 150 * MINUTE],
		[{ kind: "settled", ticket: ONE }, at(101) + 151 * MINUTE],
		[{ kind: "settled", ticket: TWO }, at(101) + 152 * MINUTE],
	]),
	// The pool takes any ticket admitted inside its window as the probe, marked or not.
	chain("a-ticket-not-marked-probe-is-the-probe", balance(at(100), at(-1)), [
		[{ kind: "admitted", ticket: ticket(9, { probe: false }) }, NOW],
		[{ kind: "settled", ticket: ticket(9, { probe: true }) }, at(1)],
		[{ kind: "settled", ticket: ticket(9, { probe: false }) }, at(2)],
	]),
	chain("a-ticket-marked-probe-is-the-probe", balance(at(100), at(-1)), [
		[{ kind: "admitted", ticket: ticket(9, { probe: true }) }, NOW],
		[{ kind: "settled", ticket: ticket(9, { probe: true }) }, at(1)],
	]),
	// A refusal never ends a running pause or reset earlier.
	chain("second-refusal-keeps-the-longer-pause", CLEAR, [
		[refused("credit", { retryAfterMs: 2 * HOUR }), NOW],
		[refused("credit"), NOW + 1000],
		[refused("allowance"), NOW + 2000],
		[refused("credit", { retryAfterMs: 3 * HOUR }), NOW + 3000],
		[refused("credit", { retryAfterMs: HOUR }), NOW + 4000],
	]),
	chain("second-refusal-ending-at-the-same-instant-is-the-newer", CLEAR, [
		[refused("credit", { retryAfterMs: HOUR }), NOW],
		[refused("allowance", { retryAfterMs: HOUR - 1000 }), NOW + 1000],
		[refused("allowance", { resetsAt: NOW + HOUR }), NOW + 2000],
	]),
	chain("refusal-after-a-pause-has-ended-replaces-it", CLEAR, [
		[refused("credit", { retryAfterMs: HOUR }), NOW],
		[refused("allowance"), NOW + HOUR],
		[refused("credit", { retryAfterMs: HOUR }), NOW + 3 * HOUR],
		[refused("credit"), NOW + 3 * HOUR + 1],
	]),
	chain("refusal-keeps-the-later-known-reset", CLEAR, [
		[refused("allowance", { resetsAt: at(180) }), NOW],
		[refused("allowance"), NOW + 1000],
		[refused("credit", { retryAfterMs: HOUR }), NOW + 2000],
		[refused("allowance", { resetsAt: at(120) }), NOW + 3000],
		[refused("allowance", { resetsAt: at(180) }), NOW + 4000],
		[refused("allowance", { resetsAt: at(200) }), NOW + 5000],
	]),
	chain("a-reset-after-a-longer-pause-keeps-the-pause", CLEAR, [
		[refused("credit", { retryAfterMs: 5 * HOUR }), NOW],
		[refused("allowance", { resetsAt: at(120) }), NOW + 1000],
		[refused("allowance", { resetsAt: at(400) }), NOW + 2000],
	]),
	chain("a-pause-longer-than-the-reset-replaces-it", CLEAR, [
		[refused("allowance", { resetsAt: at(60) }), NOW],
		[refused("credit", { retryAfterMs: 2 * HOUR }), NOW + 1000],
	]),
	// A balance bill leaves a running pause or reset.
	chain("balance-bill-leaves-a-running-pause", CLEAR, [
		[refused("credit", { retryAfterMs: 4 * HOUR }), NOW],
		[billed("balance"), NOW + 1000],
		[billed("balance"), NOW + 4 * HOUR - 1],
		[billed("balance"), NOW + 4 * HOUR],
	]),
	chain("balance-bill-over-a-running-pause-and-extra-usage-off", CLEAR, [
		[refused("allowance", { retryAfterMs: HOUR }), NOW],
		[{ kind: "extraUsage", on: false, lastBill: "balance" }, NOW + 1000],
	]),
	chain("balance-bill-over-a-reset-that-has-passed", CLEAR, [
		[refused("allowance", { resetsAt: at(10) }), NOW],
		[billed("balance"), at(10)],
	]),
	// Verdicts.
	verdicts("clear", [
		[CLEAR, false, NOW],
		[CLEAR, true, NOW],
	]),
	verdicts("paused", [
		[{ kind: "paused", reason: "credit", until: at(10) }, false, NOW],
		[{ kind: "paused", reason: "allowance", until: at(10) }, false, NOW],
		[{ kind: "paused", reason: "credit", until: at(10) }, true, NOW],
		[{ kind: "paused", reason: "credit", until: NOW }, false, NOW],
		[{ kind: "paused", reason: "credit", until: NOW + 1 }, false, NOW],
		[{ kind: "paused", reason: "credit", until: at(-1) }, false, NOW],
	]),
	verdicts("reset", [
		[{ kind: "reset", at: at(60) }, false, NOW],
		[{ kind: "reset", at: at(60) }, true, NOW],
		[{ kind: "reset", at: NOW }, false, NOW],
		[{ kind: "reset", at: NOW + 1 }, false, NOW],
		[{ kind: "reset", at: at(-1) }, false, NOW],
	]),
	verdicts("balance", [
		[STATES.open, false, NOW],
		[STATES.cooling, false, NOW],
		[STATES.probing, false, NOW],
		[STATES.expired, false, NOW],
		[STATES.floored, false, NOW],
		[STATES.open, true, NOW],
		[STATES.cooling, true, NOW],
		[STATES.probing, true, NOW],
		[STATES.floored, true, NOW],
	]),
	// A pause bounds the probe.
	verdicts("balance-probe-bounded-by-a-pause", [
		[balance(at(100), at(80), { ticket: ONE, at: at(-1) }), false, NOW],
		[balance(at(100), at(40), { ticket: ONE, at: at(-1) }), false, NOW],
		[balance(at(100), at(50), { ticket: ONE, at: at(-1) }), false, NOW],
	]),
	// The edges of the windows.
	verdicts("balance-edges", [
		[balance(at(100), NOW), false, NOW],
		[balance(at(100), NOW + 1), false, NOW],
		[balance(NOW, at(5)), false, NOW],
		[balance(NOW + 1, at(5)), false, NOW],
		[balance(NOW, NOW - 1), false, NOW],
		[balance(NOW, NOW, { ticket: ONE, at: at(-1) }), false, NOW],
		[balance(NOW + 1, NOW, { ticket: ONE, at: at(-1) }), false, NOW],
		[balance(at(100), at(-1), { ticket: ONE, at: NOW }), false, NOW],
		[balance(at(100), at(-1), { ticket: ONE, at: at(-90) }), false, NOW],
		[balance(at(2), at(-1), { ticket: ONE, at: at(-1) }), false, NOW],
	]),
	verdicts("balance-with-a-probe-past-its-eta", [
		[balance(at(-1), at(10), { ticket: ONE, at: at(-5) }), false, NOW],
		[balance(at(-1), at(-10), { ticket: ONE, at: at(-5) }), false, NOW],
	]),
	// Verdict and admission at the same instant, on each side of every threshold.
	instant("balance-thresholds", balance(at(100), at(10)), [
		at(10) - 1,
		at(10),
		at(10) + 1,
		at(100) - 1,
		at(100),
		at(100) + 1,
	]),
	instant("balance-open-window", balance(at(100), at(-1)), [NOW - 1, NOW, NOW + 1, at(100) - 1, at(100)]),
	instant("balance-with-a-probe", balance(at(100), at(10), { ticket: ONE, at: at(10) }), [
		at(10) - 1,
		at(10),
		at(55),
		at(100) - 1,
		at(100),
	]),
	instant("balance-with-an-old-probe", balance(at(100), at(10), { ticket: ONE, at: at(-60) }), [
		at(10),
		at(50),
		at(99),
		at(100),
	]),
	instant("balance-past-eta-before-next", balance(at(-1), at(10)), [NOW, at(10) - 1, at(10)]),
	instant("paused-thresholds", { kind: "paused", reason: "credit", until: at(10) }, [at(10) - 1, at(10), at(10) + 1]),
	instant("reset-thresholds", { kind: "reset", at: at(60) }, [at(60) - 1, at(60), at(60) + 1]),
];
