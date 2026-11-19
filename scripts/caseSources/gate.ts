// The provider gate on the timer wheel. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { caseOf } from "./catalogKit";

////////////////////////////////
//  Helpers

const START = 1_000_000;

const MINUTE = 60_000;

const LUNA = "lab/model-a";
const FLASH = "lab/model-b";

type Step = Record<string, unknown>;

const admit = (request: string, model = LUNA, extra: Step = {}): Step => ({ kind: "admit", request, model, ...extra });
const again = (request: string, model = LUNA, extra: Step = {}): Step =>
	admit(request, model, { again: true, ...extra });
const tryAdmit = (request: string, model = LUNA): Step => ({ kind: "tryAdmit", request, model });
const settle = (request: string, outcome: string, retryAfterMs?: number): Step => ({
	kind: "settle",
	request,
	outcome,
	...(retryAfterMs === undefined ? {} : { retryAfterMs }),
});
const settleTicket = (ticket: Step, outcome: string): Step => ({ kind: "settleTicket", ticket, outcome });
const TICKET = (gate: string, serial = 1): Step => ({ gate, model: LUNA, generation: 0, probe: false, serial });
const cool = (model = LUNA, retryAfterMs?: number): Step => ({
	kind: "cool",
	model,
	...(retryAfterMs === undefined ? {} : { retryAfterMs }),
});
const clear = (prefix: string): Step => ({ kind: "clear", prefix });
const advance = (ms: number): Step => ({ kind: "advance", ms });
const abort = (request: string): Step => ({ kind: "abort", request });
const dispose = (): Step => ({ kind: "dispose" });

const run = (name: string, steps: Step[], extra: Step = {}): CaseSource =>
	caseOf("gate.run")(name, { startMs: START, steps, ...extra });

/** One overload pause and the probe it ends in: the model is open, then paused, then probing. */
const overloaded = (trigger = "trigger", model = LUNA): Step[] => [
	admit(trigger, model),
	settle(trigger, "overloaded"),
];

/** Probes `from` to `to`, each refused again and named `p<n>`. */
function refusals(from: number, to: number): Step[] {
	const steps: Step[] = [];
	for (let round = from; round <= to; round++)
		steps.push(again(`p${round}`), advance(MINUTE), settle(`p${round}`, "overloaded"));
	return steps;
}

////////////////////////////////
//  Cases

export const gateCases: CaseSource[] = [
	// One recovery probe after a pause.
	run("overload-pauses-requests-until-one-probe-succeeds", [
		admit("a"),
		settle("a", "overloaded"),
		admit("b"),
		admit("c"),
		admit("other", FLASH),
		advance(59_999),
		advance(1),
		settle("b", "ok"),
	]),
	run("a-probe-lets-no-second-request-through", [
		...overloaded(),
		admit("b"),
		admit("c"),
		advance(MINUTE),
		tryAdmit("d"),
		admit("e"),
		settle("b", "ok"),
	]),
	run("the-oldest-waiter-probes-first", [...overloaded(), admit("b"), admit("c"), admit("d"), advance(MINUTE)]),
	run("a-retry-goes-to-the-front-of-the-queue", [
		...overloaded(),
		admit("b"),
		admit("c"),
		again("retry"),
		advance(MINUTE),
		settle("retry", "ok"),
	]),
	run("a-retry-while-open-goes-at-once", [again("retry"), settle("retry", "ok")]),
	run("a-paused-gate-with-no-waiter-turns-ready-when-the-pause-ends", [
		...overloaded(),
		advance(MINUTE),
		admit("late"),
		settle("late", "ok"),
		admit("after"),
	]),
	run("a-success-opens-the-gate-for-the-queue-as-plain-requests", [
		...overloaded(),
		admit("b"),
		admit("c"),
		admit("d"),
		advance(MINUTE),
		settle("b", "ok"),
		admit("e"),
	]),
	run("a-model-does-not-disturb-another", [
		...overloaded("trigger", LUNA),
		admit("other", FLASH),
		tryAdmit("other-try", FLASH),
		cool(FLASH),
		admit("waiting-other", FLASH),
		advance(MINUTE),
		settle("trigger", "ok"),
	]),

	// Giving up.
	run(
		"repeated-overloads-exhaust-the-waiters",
		[
			...overloaded("first"),
			admit("p1"),
			admit("waiter"),
			advance(MINUTE),
			settle("p1", "overloaded"),
			again("p2"),
			advance(MINUTE),
			settle("p2", "overloaded"),
			again("p3"),
			advance(MINUTE),
			settle("p3", "overloaded"),
			admit("after"),
		],
		{ maxPauses: 3 },
	),
	run("ten-overloads-give-up", [
		...overloaded("first"),
		admit("p1"),
		admit("waiter"),
		advance(MINUTE),
		settle("p1", "overloaded"),
		...refusals(2, 10),
		admit("after"),
	]),
	run("nine-overloads-do-not-give-up", [
		...overloaded("first"),
		admit("p1"),
		admit("waiter"),
		advance(MINUTE),
		settle("p1", "overloaded"),
		...refusals(2, 9),
		again("p10"),
		advance(MINUTE),
		settle("p10", "ok"),
	]),
	run(
		"a-single-allowed-pause-gives-up-at-the-first-probe-refusal",
		[...overloaded("first"), admit("p1"), advance(MINUTE), settle("p1", "overloaded")],
		{ maxPauses: 1 },
	),
	run(
		"no-pauses-allowed-gives-up-at-once",
		[...overloaded("first"), admit("p1"), advance(MINUTE), settle("p1", "overloaded")],
		{ maxPauses: 0 },
	),
	run(
		"giving-up-opens-the-gate-for-later-requests",
		[...overloaded("first"), admit("p1"), advance(MINUTE), settle("p1", "overloaded"), admit("w"), tryAdmit("t")],
		{ maxPauses: 1 },
	),

	// A success resetting the count.
	run(
		"in-flight-success-resets-the-overload-pause-count",
		[
			admit("early"),
			admit("trigger"),
			settle("trigger", "overloaded"),
			admit("probing"),
			advance(MINUTE),
			settle("probing", "overloaded"),
			again("retry"),
			settle("early", "ok"),
			advance(MINUTE),
			settle("retry", "overloaded"),
		],
		{ maxPauses: 2 },
	),
	run("a-success-in-a-quiet-gate-keeps-the-count-at-zero", [
		admit("a"),
		settle("a", "ok"),
		admit("b"),
		settle("b", "overloaded"),
	]),
	run("a-failed-request-leaves-the-count", [
		admit("early"),
		admit("trigger"),
		settle("trigger", "overloaded"),
		settle("early", "failed"),
		settle("early", "cancelled"),
	]),

	// A stale result of an older generation.
	run("stale-overloads-retry-and-current-overloads-pause-again", [
		admit("early"),
		admit("trigger"),
		settle("trigger", "overloaded"),
		admit("waiter"),
		advance(MINUTE),
		settle("waiter", "ok"),
		settle("early", "overloaded"),
		admit("resent"),
		settle("resent", "overloaded"),
		admit("next"),
	]),
	run("a-stale-overload-during-a-pause-asks-to-retry-and-pauses-nothing", [
		admit("early"),
		admit("trigger"),
		settle("trigger", "overloaded"),
		advance(30_000),
		settle("early", "overloaded"),
		advance(30_000),
	]),
	run("a-stale-ticket-of-the-first-generation-after-two-reopenings", [
		admit("old"),
		admit("trigger"),
		settle("trigger", "overloaded"),
		admit("p"),
		advance(MINUTE),
		settle("p", "ok"),
		admit("again-trigger"),
		settle("again-trigger", "overloaded"),
		admit("q"),
		advance(MINUTE),
		settle("q", "ok"),
		settle("old", "overloaded"),
		admit("fresh"),
	]),

	// Failed and cancelled probes pass the probe on.
	run("failed-probes-pass-to-the-next-waiter", [
		...overloaded(),
		admit("a"),
		admit("b"),
		advance(MINUTE),
		settle("a", "cancelled"),
		settle("b", "failed"),
		admit("late"),
	]),
	run("a-failed-probe-with-no-waiter-lets-the-next-request-probe", [
		...overloaded(),
		admit("a"),
		advance(MINUTE),
		settle("a", "failed"),
		tryAdmit("t"),
		tryAdmit("u"),
	]),
	run("a-cancelled-probe-keeps-the-pause-count", [
		...overloaded(),
		admit("a"),
		advance(MINUTE),
		settle("a", "cancelled"),
		admit("b"),
		settle("b", "overloaded"),
	]),

	// Retry-After.
	run("retry-after-longer-than-the-pause-extends-it", [
		admit("trigger"),
		settle("trigger", "overloaded", 90_000),
		admit("kept"),
		advance(MINUTE),
		advance(30_000),
		settle("kept", "ok"),
	]),
	run("retry-after-shorter-than-the-pause-leaves-it", [
		admit("trigger"),
		settle("trigger", "overloaded", 10_000),
		admit("kept"),
		advance(10_000),
		advance(50_000),
		settle("kept", "ok"),
	]),
	run("retry-after-equal-to-the-pause", [
		admit("trigger"),
		settle("trigger", "overloaded", MINUTE),
		admit("kept"),
		advance(MINUTE),
	]),
	run("retry-after-on-a-probe-refusal", [
		...overloaded(),
		admit("p1"),
		advance(MINUTE),
		settle("p1", "overloaded", 5 * MINUTE),
		again("p2"),
		advance(MINUTE),
		advance(4 * MINUTE),
		settle("p2", "ok"),
	]),
	run("retry-after-of-zero-keeps-the-pause", [
		admit("trigger"),
		settle("trigger", "overloaded", 0),
		admit("kept"),
		advance(MINUTE),
	]),
	run(
		"a-custom-pause",
		[admit("trigger"), settle("trigger", "overloaded"), admit("kept"), advance(1_000), advance(500)],
		{
			pauseMs: 1_500,
		},
	),
	run(
		"retry-after-against-a-custom-pause",
		[admit("trigger"), settle("trigger", "overloaded", 2_000), admit("kept"), advance(1_999), advance(1)],
		{ pauseMs: 1_500 },
	),

	// A waiter aborted while queued.
	run("stopping-removes-waiters-and-retry-after-extends-pauses", [
		admit("trigger"),
		settle("trigger", "overloaded", 90_000),
		admit("stopped", LUNA, { abortAfterMs: 1_000 }),
		admit("kept"),
		advance(1_000),
		advance(59_000),
		advance(30_000),
	]),
	run("an-abort-by-a-step-removes-the-waiter", [
		...overloaded(),
		admit("stopped"),
		admit("kept"),
		abort("stopped"),
		advance(MINUTE),
	]),
	run("a-request-aborted-before-it-asks-is-stopped", [admit("a", LUNA, { abortedBefore: true }), tryAdmit("b")]),
	run("an-aborted-request-while-paused-takes-no-turn", [
		...overloaded(),
		admit("stopped", LUNA, { abortedBefore: true }),
		admit("kept"),
		advance(MINUTE),
	]),
	run("aborting-the-only-waiter-leaves-the-pause-running", [
		...overloaded(),
		admit("stopped"),
		abort("stopped"),
		advance(MINUTE),
		admit("late"),
	]),
	run("an-abort-after-the-admission-changes-nothing", [admit("a"), abort("a"), settle("a", "ok")]),
	run("a-waiter-aborted-before-its-probe-turn-passes-the-turn", [
		...overloaded(),
		admit("first", LUNA, { abortAfterMs: 30_000 }),
		admit("second"),
		advance(MINUTE),
		settle("second", "ok"),
	]),

	// Admitting without waiting.
	run("try-admit-takes-an-open-gate", [tryAdmit("a"), tryAdmit("b"), settle("a", "ok"), settle("b", "ok")]),
	run("try-admit-refuses-a-paused-gate-and-probes-a-probing-one", [
		...overloaded(),
		tryAdmit("paused"),
		advance(MINUTE),
		tryAdmit("probe"),
		tryAdmit("second"),
		settle("probe", "ok"),
		tryAdmit("open"),
	]),
	run("try-admit-does-not-pass-a-waiter", [...overloaded(), admit("w"), advance(MINUTE), tryAdmit("t")]),

	// Cooling.
	run("cool-pauses-an-open-gate", [cool(), admit("w"), advance(MINUTE), settle("w", "ok")]),
	run("cool-with-a-retry-after", [cool(LUNA, 3 * MINUTE), admit("w"), advance(2 * MINUTE), advance(MINUTE)]),
	run("cool-with-a-retry-after-of-a-thousand-days", [
		cool(LUNA, 1000 * 24 * 60 * MINUTE),
		admit("w"),
		advance(60 * MINUTE - 1),
		advance(1),
	]),
	run("overload-with-a-retry-after-of-a-thousand-days", [
		admit("trigger"),
		settle("trigger", "overloaded", 1000 * 24 * 60 * MINUTE),
		admit("w"),
		advance(60 * MINUTE - 1),
		advance(1),
	]),
	run("cool-leaves-a-paused-gate-alone", [
		...overloaded(),
		advance(30_000),
		cool(LUNA, 10 * MINUTE),
		admit("w"),
		advance(30_000),
	]),
	run("cool-leaves-a-probing-gate-alone", [...overloaded(), advance(MINUTE), cool(), tryAdmit("t")]),
	run("a-cooled-gate-does-not-count-toward-giving-up", [
		cool(),
		admit("p1"),
		advance(MINUTE),
		settle("p1", "overloaded"),
		again("p2"),
		advance(MINUTE),
		settle("p2", "ok"),
	]),

	// Clearing.
	run("clear-reopens-a-matching-paused-gate-and-releases-its-queue", [
		...overloaded("trigger", `entry\n${LUNA}`),
		admit("w1", `entry\n${LUNA}`),
		admit("w2", `entry\n${LUNA}`),
		clear("entry\n"),
		admit("after", `entry\n${LUNA}`),
	]),
	run("clear-leaves-the-other-entries-alone", [
		...overloaded("a", `one\n${LUNA}`),
		...overloaded("b", `two\n${LUNA}`),
		admit("wa", `one\n${LUNA}`),
		admit("wb", `two\n${LUNA}`),
		clear("one\n"),
	]),
	run("clear-cancels-the-pause-timer", [...overloaded(), admit("w"), clear(""), advance(MINUTE), admit("after")]),
	run("clear-takes-the-probe-away-from-a-probing-gate", [
		...overloaded(),
		admit("p"),
		advance(MINUTE),
		clear(""),
		settle("p", "overloaded"),
		admit("after"),
	]),
	run("clear-leaves-an-open-gate-and-its-generation", [admit("a"), settle("a", "ok"), clear(""), admit("b")]),
	run("a-ticket-of-a-cleared-pause-is-stale", [
		admit("early"),
		admit("trigger"),
		settle("trigger", "overloaded"),
		clear(""),
		settle("early", "overloaded"),
	]),

	// Disposing.
	run("dispose-stops-the-waiters-and-cancels-the-timers", [...overloaded(), admit("w1"), admit("w2"), dispose()]),
	run("dispose-with-no-gate-open", [dispose()]),
	run("dispose-with-a-probe-out", [...overloaded(), admit("p"), advance(MINUTE), dispose()]),

	// A ticket of another gate, or from before a dispose.
	run("a-ticket-of-another-gate-does-nothing", [
		admit("a"),
		settleTicket(TICKET("elsewhere"), "overloaded"),
		admit("b"),
		settleTicket(TICKET("elsewhere", 7), "ok"),
	]),
	run("a-settle-of-another-gates-ticket-creates-no-gate", [settleTicket(TICKET("elsewhere"), "overloaded")]),
	run("a-settle-of-this-gates-ticket-for-an-unknown-model-creates-no-gate", [
		settleTicket(TICKET("gate-1"), "overloaded"),
	]),
	run("a-ticket-from-before-a-dispose-does-nothing-to-the-new-state", [
		admit("a"),
		dispose(),
		settleTicket(TICKET("gate-1"), "overloaded"),
	]),

	// After a dispose.
	run("a-disposed-gate-admits-nothing", [
		...overloaded(),
		admit("w"),
		dispose(),
		admit("late"),
		tryAdmit("try"),
		cool(),
		advance(MINUTE),
	]),
	run("a-disposed-gate-keeps-no-timer", [
		admit("a"),
		dispose(),
		cool(),
		cool(FLASH, MINUTE),
		clear(""),
		advance(MINUTE),
	]),

	// Callbacks run after the change.
	run("a-waiters-callback-sees-a-whole-gate", [
		...overloaded(),
		admit("w1", LUNA, { inside: "tryAdmit" }),
		admit("w2", LUNA, { inside: "cool" }),
		advance(MINUTE),
		settle("w1", "overloaded"),
	]),
	run("a-waiters-callback-that-settles-meets-a-consistent-gate", [
		...overloaded(),
		admit("w1", LUNA, { inside: "settleTrigger" }),
		admit("w2", LUNA, { inside: "settleTrigger" }),
		advance(MINUTE),
		settle("w1", "failed"),
	]),

	// What a waiting request is told.
	run("a-waiter-is-told-of-each-pause", [
		...overloaded(),
		admit("w"),
		advance(MINUTE),
		settle("w", "overloaded"),
		again("w2"),
		advance(MINUTE),
		settle("w2", "ok"),
	]),
	run("two-waiters-are-both-told", [
		...overloaded(),
		admit("w1"),
		admit("w2"),
		advance(MINUTE),
		settle("w1", "overloaded"),
	]),
	run("a-waiter-told-while-probing-has-no-resume-time", [...overloaded(), admit("w1"), admit("w2"), advance(MINUTE)]),
	run(
		"pauses-told-count-with-the-limit",
		[...overloaded(), admit("w"), advance(MINUTE), settle("w", "overloaded"), again("v")],
		{
			maxPauses: 4,
		},
	),
];
