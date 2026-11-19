// The request lifecycle over a real health, the real kinds and one scripted network. Every value is made up.

import { probeAt } from "@nyaaai/agentic/health/pool";
import type { CaseSource } from "../caseAdapter";
import type { Json } from "./codecFixtures";
import {
	A,
	ANALYSIS,
	abort,
	advance,
	answers,
	B,
	BALANCE,
	BALANCE_COOL_MS,
	BOX,
	begin,
	dies,
	down,
	failing,
	GO,
	json,
	KEY,
	key,
	LOCAL_ROW,
	LUNA,
	list,
	NEW_KEY,
	OPUS,
	overloaded,
	POOL,
	plan,
	poolLimit,
	RESET_MS,
	reads,
	run,
	START,
	says,
	send,
	sse,
	USAGE,
	zdr,
} from "./lifecycleKit";
import { DONE_TEXT, frame } from "./transportKit";

////////////////////////////////
//  Fixtures

const GO_PAIR = { go: KEY, b: KEY };
const ALONE = { entries: [A], keys: { a: KEY } };
const GO_ALONE = { entries: [GO], keys: { go: KEY } };
/** A reset later than the balance estimate. */
const LATER = "2026-11-07T20:00:00Z";

const GO_WITH_B = { entries: [GO, B], keys: GO_PAIR };

/** A balance-billed reply that fails with nothing streamed. */
const failedBalance = sse([frame({ error: { code: 502, message: "Down" }, usage: BALANCE })]);

/** A reply of the same model as a chat step, unstreamed. */
const oneShot = send("one-shot", { tag: ANALYSIS, mode: "complete" });

const tiers = [{ minPromptTokens: 0, prompt: 1, completion: 2, cacheRead: 0.1, cacheWrite: 1.25, cacheWrite1h: 2 }];
const PRICE = { prompt: [0, 0], completion: [0, 0], tiers };
const WRITES = {
	prompt_tokens: 1000,
	completion_tokens: 100,
	prompt_tokens_details: { cached_tokens: 200, cache_write_tokens: 300 },
};

////////////////////////////////
//  Cases

const fallThrough: CaseSource[] = [
	run("a-rejected-key-falls-through-and-the-entry-sits-out-until-its-key-is-replaced", {
		exchanges: [failing(401), says("B"), says("B"), says("A")],
		steps: [send("first"), send("second"), key("a", NEW_KEY), send("third")],
	}),
	run("a-key-replaced-clears-rejected-the-cooldowns-and-the-bill", {
		exchanges: [overloaded, says("B"), failing(401), says("B"), says("A"), says("A")],
		steps: [
			send("other", { model: OPUS }),
			send("rejected"),
			{ kind: "billed", id: "a", source: "go", cost: 1 },
			key("a", NEW_KEY),
			send("next"),
			send("again", { model: OPUS }),
		],
	}),
	run("a-key-removed-sends-nothing-to-the-entry-until-one-is-stored", {
		exchanges: [says("B"), says("B"), says("A")],
		steps: [key("a", { kind: "none" }), send("first"), send("second"), key("a", NEW_KEY), send("third")],
	}),
	run("an-overload-cools-its-entry-for-that-model-only-until-the-key-changes", {
		exchanges: [overloaded, says("B"), says("A"), says("B"), says("A")],
		steps: [send("first"), send("other", { model: OPUS }), send("again"), key("a", NEW_KEY), send("cleared")],
	}),
	run("a-pool-limit-cools-the-whole-entry-until-it-resets", {
		...GO_WITH_B,
		exchanges: [poolLimit(), says("B"), says("B"), says("Go")],
		steps: [
			send("first"),
			send("other", { model: OPUS }),
			advance(RESET_MS - START + 1),
			send("reset", { model: OPUS }),
		],
	}),
	run("a-lone-entry-out-of-its-pool-ends-on-the-spend-notice-with-its-reset-sending-nothing-until-then", {
		...GO_ALONE,
		exchanges: [poolLimit(), says("Go")],
		steps: [
			send("first"),
			send("held", { model: OPUS }),
			advance(RESET_MS - START + 1),
			send("reset", { model: OPUS }),
		],
	}),
	run("a-pool-limit-reset-in-epoch-seconds-keeps-the-entry-skipped-until-just-past-it", {
		...GO_WITH_B,
		exchanges: [poolLimit(RESET_MS / 1000), says("B"), says("B"), says("Go")],
		steps: [send("limited"), advance(RESET_MS - START - 1000), send("before"), advance(2000), send("after")],
	}),
	run("a-400-stops-with-its-notice-and-never-falls-through", {
		exchanges: [failing(400), says("B")],
		steps: [send("first")],
	}),
	run("a-403-stops-with-its-notice-and-never-falls-through", {
		exchanges: [failing(403), says("B")],
		steps: [send("first")],
	}),
	run("metering-records-each-attempt-once-across-a-retry-and-a-fall-through", {
		exchanges: [failing(503), failing(401), says("B")],
		steps: [send("first"), advance(1000)],
	}),
	run("a-request-resolved-before-anothers-401-never-sends-to-the-rejected-entry", {
		exchanges: [failing(401), says("B"), says("B")],
		steps: [plan("held"), send("rejecting"), send("late", { plan: "held" })],
	}),
	run("a-request-resolved-before-anothers-401-alone-refuses-naming-the-entry", {
		...ALONE,
		exchanges: [failing(401), says("A")],
		steps: [plan("held"), send("rejecting"), send("late", { plan: "held" })],
	}),
	run("a-stream-that-dies-mid-step-reruns-whole-on-the-next-entry", {
		exchanges: [dies(), says("B")],
		steps: [send("first")],
	}),
	run("a-stream-that-dies-mid-step-on-the-last-route-ends-on-the-stream-notice", {
		...ALONE,
		exchanges: [dies(), says("A")],
		steps: [send("first")],
	}),
	run("when-every-entry-fails-the-run-ends-on-the-last-failures-notice", {
		entries: [A, GO, B],
		keys: { a: KEY, go: KEY, b: KEY },
		exchanges: [failing(401), poolLimit(), failing(503), failing(503), failing(503), failing(503)],
		steps: [send("first"), advance(1000), advance(2000), advance(4000)],
	}),
	run("an-unreachable-local-server-is-not-routed-until-a-reach-check-succeeds", {
		entries: [BOX],
		keys: {},
		rows: [LOCAL_ROW],
		exchanges: [down, down, down, down, says("Local")],
		steps: [
			send("first", { model: LOCAL_ROW }),
			advance(1000),
			advance(2000),
			advance(4000),
			plan("refused", LOCAL_ROW),
			{ kind: "check", id: "box" },
			send("back", { model: LOCAL_ROW }),
		],
	}),
	run("a-server-saved-again-with-another-url-is-rechecked-and-routes-again", {
		entries: [BOX],
		keys: {},
		rows: [LOCAL_ROW],
		exchanges: [down, down, down, down, says("Local")],
		steps: [
			send("first", { model: LOCAL_ROW }),
			advance(1000),
			advance(2000),
			advance(4000),
			list({ ...BOX, url: "http://box.local:11435" }),
			send("back", { model: LOCAL_ROW }),
		],
	}),
	run("an-entry-passed-over-before-any-attempt-shows-once-per-run", {
		entries: [A, B, GO],
		keys: { b: KEY },
		exchanges: [reads(), says("Done.")],
		steps: [
			plan("held"),
			begin("run"),
			send("first", { plan: "held", run: "run" }),
			send("second", { plan: "held", run: "run" }),
		],
	}),
	run("an-entry-skipped-by-the-first-step-is-not-named-again-by-the-next-on-one-run-and-is-by-a-new-run", {
		entries: [A, B],
		keys: { b: KEY },
		exchanges: [says("B"), says("B"), says("B")],
		steps: [
			plan("held"),
			begin("one"),
			send("first", { plan: "held", run: "one" }),
			send("second", { plan: "held", run: "one" }),
			begin("two"),
			send("third", { plan: "held", run: "two" }),
		],
	}),
];

const zeroRetention: CaseSource[] = [
	run("zdr-on-a-data-keeping-entry-is-never-a-candidate", {
		zdrOnly: true,
		listings: { a: { [LUNA]: { keeps: true } } },
		exchanges: [says("B")],
		steps: [plan("held"), send("first")],
	}),
	run("zdr-on-after-resolving-skips-to-the-next-entry-that-keeps-none", {
		listings: { a: { [LUNA]: { keeps: true } } },
		exchanges: [says("B")],
		steps: [plan("held"), zdr(true), send("first", { plan: "held" })],
	}),
	run("zdr-on-after-resolving-ends-on-the-notice-when-every-entry-keeps-data", {
		listings: { a: { [LUNA]: { keeps: true } }, b: { [LUNA]: { keeps: true } } },
		exchanges: [says("A")],
		steps: [plan("held"), zdr(true), send("first", { plan: "held" })],
	}),
	run("zdr-on-after-resolving-a-lone-entry-that-keeps-data-ends-on-the-notice", {
		...ALONE,
		listings: { a: { [LUNA]: { keeps: true } } },
		exchanges: [says("A")],
		steps: [plan("held"), zdr(true), send("first", { plan: "held" })],
	}),
	run("an-entry-removed-from-the-list-is-no-candidate-and-one-added-is-tried", {
		exchanges: [says("B"), says("A")],
		steps: [list(B), send("first"), list(A, B), send("second")],
	}),
	run("zdr-off-again-sends-to-the-entry-that-keeps-data", {
		listings: { a: { [LUNA]: { keeps: true } } },
		exchanges: [says("A")],
		steps: [zdr(true), zdr(false), send("first")],
	}),
];

const runRoute: CaseSource[] = [
	run("a-balance-cooled-entry-serves-the-rest-of-its-run-a-one-shot-request-and-a-new-run-skip-it", {
		...GO_WITH_B,
		exchanges: [reads(BALANCE), says("Go", BALANCE), answers("T"), says("B")],
		steps: [
			begin("run"),
			send("first", { run: "run" }),
			send("second", { run: "run" }),
			oneShot,
			begin("next"),
			send("third", { run: "next" }),
		],
	}),
	run("a-balance-billed-unstreamed-reply-keeps-the-run-on-go-for-its-later-steps", {
		...GO_WITH_B,
		exchanges: [answers("Sum", BALANCE), says("Go", BALANCE), says("B")],
		steps: [
			begin("run"),
			send("summary", { run: "run", tag: ANALYSIS, mode: "complete" }),
			send("step", { run: "run" }),
			send("next"),
		],
	}),
	run("sizing-keeps-the-entries-the-gate-would-try-and-the-last-route-stays", {
		exchanges: [overloaded, says("B")],
		steps: [
			send("cooling"),
			plan("held"),
			begin("run"),
			{ kind: "sizing", run: "run", plan: "held" },
			list(B, A),
			plan("reversed"),
			{ kind: "sizing", run: "run", plan: "reversed" },
		],
	}),
	run("sizing-keeps-every-candidate-when-none-would-be-tried", {
		...GO_ALONE,
		exchanges: [poolLimit()],
		steps: [send("limited"), plan("held"), begin("run"), { kind: "sizing", run: "run", plan: "held" }],
	}),
];

const extraUsage: CaseSource[] = [
	run("off-a-balance-billed-reply-that-fails-still-starts-the-cooldown", {
		...GO_WITH_B,
		exchanges: [failedBalance, says("Go"), says("B")],
		steps: [send("first"), advance(1000), send("next")],
	}),
	run("off-a-balance-billed-reply-that-fails-mid-step-keeps-go-for-the-runs-later-steps-not-the-next-run", {
		...GO_WITH_B,
		exchanges: [dies(BALANCE), reads(), says("Go"), says("B")],
		steps: [
			begin("run"),
			send("first", { run: "run" }),
			advance(60_000),
			send("second", { run: "run" }),
			begin("next"),
			send("next", { run: "next" }),
		],
	}),
	run("off-a-balance-reply-lets-its-run-finish-on-go-later-requests-skip-go-until-the-first-probe", {
		...GO_WITH_B,
		exchanges: [reads(BALANCE), says("Go", BALANCE), says("B"), says("Go", BALANCE)],
		steps: [
			begin("run"),
			send("first", { run: "run" }),
			send("second", { run: "run" }),
			send("next"),
			advance(BALANCE_COOL_MS / 2 + 1),
			send("later"),
		],
	}),
	run("off-past-the-first-window-a-pool-reply-ends-the-cooldown", {
		...GO_WITH_B,
		exchanges: [says("Go", BALANCE), says("Go", POOL), says("Go")],
		steps: [send("bill"), advance(BALANCE_COOL_MS / 2 + 1), send("probe"), send("next")],
	}),
	run("off-zdr-defers-gos-probe-to-the-next-run", {
		...GO_WITH_B,
		listings: { go: { [LUNA]: { keeps: true } } },
		exchanges: [says("Go", BALANCE), says("B"), says("Go")],
		steps: [
			send("bill"),
			advance(BALANCE_COOL_MS / 2 + 1),
			plan("held"),
			zdr(true),
			send("kept", { plan: "held" }),
			zdr(false),
			send("next"),
		],
	}),
	run("on-go-keeps-serving-after-a-balance-reply", {
		entries: [{ ...GO, extraUsage: true }, B],
		keys: GO_PAIR,
		exchanges: [says("Go", BALANCE), says("Go", BALANCE)],
		steps: [send("first"), send("second")],
	}),
	run("turning-extra-usage-on-clears-the-balance-cooldown", {
		...GO_WITH_B,
		exchanges: [says("Go", BALANCE), says("B"), says("Go")],
		steps: [send("bill"), send("cooled"), list({ ...GO, extraUsage: true }, B), send("on")],
	}),
	run("turning-extra-usage-off-again-starts-the-cooldown-only-after-a-balance-reply", {
		entries: [{ ...GO, extraUsage: true }, B],
		keys: GO_PAIR,
		exchanges: [says("Go", BALANCE), says("Go", BALANCE), says("B")],
		steps: [send("first"), list(GO, B), send("second"), send("third")],
	}),
	run("a-plan-change-clears-the-balance-cooldown-and-keeps-the-bill", {
		...GO_WITH_B,
		exchanges: [says("Go", BALANCE), says("B"), says("Go")],
		steps: [send("bill"), send("cooled"), list({ ...GO, plan: "payg" }, B), send("changed")],
	}),
	run("a-known-reset-wins-over-a-balance-cooldown", {
		...GO_WITH_B,
		exchanges: [says("Go", BALANCE), poolLimit(LATER), says("B"), says("B"), says("Go")],
		steps: [
			send("bill"),
			advance(BALANCE_COOL_MS / 2 + 1),
			send("limited"),
			// Past the estimate, before the reset.
			advance(9_500_000),
			send("held"),
			advance(Date.parse(LATER) - START - 18_500_000),
			send("reset"),
		],
	}),
	run("turning-extra-usage-off-again-after-a-pool-reply-starts-no-cooldown", {
		entries: [{ ...GO, extraUsage: true }, B],
		keys: GO_PAIR,
		exchanges: [says("Go", POOL), says("Go")],
		steps: [send("first"), list(GO, B), send("second")],
	}),
];

/** The balance cooldown: each reply while it runs halves what is left, down to its floor. */
function halving(): CaseSource {
	const steps: Json[] = [send("bill")];
	const exchanges: Json[] = [says("Go", BALANCE)];
	let at = 0;
	let eta = BALANCE_COOL_MS;
	let next = probeAt(at, eta);
	for (let window = 1; window <= 8; window++) {
		steps.push(advance(next - at - 1), send(`early-${window}`), advance(1), send(`probe-${window}`));
		exchanges.push(says("B"), says("Go", BALANCE));
		at = next;
		// A reply past the estimate opens a new window.
		if (at >= eta) eta = at + BALANCE_COOL_MS;
		next = probeAt(at, eta);
	}
	return run("the-balance-cooldown-halves-to-its-floor-and-asks-again-at-the-estimate", {
		...GO_WITH_B,
		exchanges,
		steps,
	});
}

const overload: CaseSource[] = [
	run("an-overload-holds-the-next-request-until-one-probe-succeeds", {
		...ALONE,
		exchanges: [overloaded, says("A"), says("A")],
		steps: [send("first"), send("second"), advance(59_999), advance(1)],
	}),
	run("retry-after-lengthens-the-pause", {
		...ALONE,
		exchanges: [failing(429, { error_type: "rate_limit_exceeded" }, [["retry-after", "120"]]), says("A")],
		steps: [send("first"), advance(60_000), advance(59_999), advance(1)],
	}),
	run("an-overload-that-keeps-failing-gives-the-request-and-the-waiter-up-at-the-last-pause", {
		...ALONE,
		maxPauses: 2,
		exchanges: [overloaded, overloaded, overloaded, says("A")],
		steps: [send("first"), send("second"), advance(60_000), advance(60_000), send("after")],
	}),
	run("an-overload-on-the-last-route-after-streamed-output-ends-on-the-stream-notice", {
		...ALONE,
		exchanges: [
			sse([
				frame({
					id: "gen-1",
					provider: "Azure",
					choices: [{ index: 0, delta: { role: "assistant", content: "Half" } }],
				}),
				frame({ error: { code: 429, message: "Busy", metadata: { error_type: "rate_limit_exceeded" } } }),
			]),
			says("A"),
		],
		steps: [send("first")],
	}),
	run("a-live-call-is-never-held-by-a-paused-gate", {
		...ALONE,
		exchanges: [overloaded, says("A")],
		steps: [send("first"), send("live", { live: true }), advance(60_000)],
	}),
	run("a-live-call-gets-two-tries-and-no-more", {
		...ALONE,
		exchanges: [failing(503), failing(503), says("A")],
		steps: [send("live", { live: true }), advance(1000)],
	}),
	run("a-live-call-served-by-its-first-try-never-waits", {
		...ALONE,
		exchanges: [says("A")],
		steps: [send("live", { live: true })],
	}),
	run("a-live-overload-on-the-first-route-falls-through", {
		exchanges: [overloaded, says("B")],
		steps: [send("live", { live: true })],
	}),
];

/** Every failure a send can meet, with whether it retries: its disposition on a first and on a last route. */
const FAILURES: Array<{ name: string; reply: Json; retries: boolean }> = [
	{ name: "400", reply: failing(400), retries: false },
	{ name: "401", reply: failing(401), retries: false },
	{ name: "402", reply: failing(402), retries: false },
	{ name: "403", reply: failing(403), retries: false },
	{ name: "404", reply: failing(404), retries: false },
	{ name: "429-rate-limited", reply: failing(429), retries: true },
	{ name: "500", reply: failing(500), retries: true },
	{ name: "503", reply: failing(503), retries: true },
	{ name: "a-refused-connection", reply: down, retries: true },
	{
		name: "an-empty-reply",
		reply: sse([
			frame({ id: "gen-1", choices: [{ index: 0, delta: { content: "" }, finish_reason: "stop" }] }),
			DONE_TEXT,
		]),
		retries: true,
	},
];

function dispositions(): CaseSource[] {
	const first = FAILURES.map(({ name, reply, retries }) =>
		run(`disposition-${name}-on-a-first-route`, {
			retry: { tries: 2 },
			exchanges: retries ? [reply, reply, says("B")] : [reply, says("B")],
			steps: retries ? [send("first"), advance(1000)] : [send("first")],
		}),
	);
	const last = FAILURES.map(({ name, reply, retries }) =>
		run(`disposition-${name}-on-a-last-route`, {
			...ALONE,
			retry: { tries: 2 },
			exchanges: retries ? [reply, reply, says("A")] : [reply, says("A")],
			steps: retries ? [send("first"), advance(1000)] : [send("first")],
		}),
	);
	// A Go allowance limit is a pool limit of the Go entry alone.
	const limit = poolLimit();
	const go = [
		run("disposition-a-go-allowance-limit-on-a-first-route", {
			...GO_WITH_B,
			exchanges: [limit, says("B")],
			steps: [send("first")],
		}),
		run("disposition-a-go-allowance-limit-on-a-last-route", {
			...GO_ALONE,
			exchanges: [limit, says("Go")],
			steps: [send("first")],
		}),
	];
	return [...first, ...last, ...go];
}

const retries: CaseSource[] = [
	run("a-retry-waits-the-backoff-and-the-next-try-serves", {
		...ALONE,
		exchanges: [failing(503), says("A")],
		steps: [send("first"), advance(999), advance(1)],
	}),
	run("the-backoff-doubles-each-try-and-a-draw-below-one-shortens-it", {
		...ALONE,
		draws: [0, 0.5],
		exchanges: [failing(503), failing(503), failing(503), says("A")],
		steps: [send("first"), advance(500), advance(1500), advance(2000)],
	}),
	run("a-retry-after-wait-replaces-the-backoff", {
		...ALONE,
		exchanges: [failing(503, undefined, [["retry-after", "7"]]), says("A")],
		steps: [send("first"), advance(6999), advance(1)],
	}),
	run("a-retry-budget-met-ends-on-the-exhausted-notice", {
		...ALONE,
		retry: { budgetMs: 1500 },
		exchanges: [failing(503), failing(503), says("A")],
		steps: [send("first"), advance(1000)],
	}),
	run("the-tries-met-end-on-the-exhausted-notice", {
		...ALONE,
		exchanges: [failing(503), failing(503), failing(503), failing(503), says("A")],
		steps: [send("first"), advance(1000), advance(2000), advance(4000)],
	}),
	run("a-retry-on-the-first-route-spends-its-budget-before-the-fall-through", {
		retry: { tries: 3 },
		exchanges: [failing(503), failing(503), failing(503), says("B")],
		steps: [send("first"), advance(1000), advance(2000)],
	}),
	run("a-send-that-throws-what-is-no-failure-throws-on-and-settles-the-gate", {
		exchanges: [says("B")],
		steps: [send("first", { throws: "The send broke." }), send("second")],
	}),
];

const aborts: CaseSource[] = [
	run("a-caller-abort-during-a-send-stops-the-run-and-the-meter-says-cancelled", {
		exchanges: [says("A", USAGE, { headMs: 100 }), says("B")],
		steps: [send("first", { abortAfterMs: 50 })],
	}),
	run("a-caller-abort-before-the-call-stops-it-before-any-admission", {
		exchanges: [says("A")],
		steps: [send("first", { abortedBefore: true })],
	}),
	run("a-caller-abort-during-a-retry-wait-stops-the-run", {
		...ALONE,
		exchanges: [failing(503), says("A")],
		steps: [send("first"), abort("first"), advance(1000)],
	}),
	run("a-caller-abort-during-a-gate-hold-stops-the-run-and-frees-the-queue", {
		...ALONE,
		exchanges: [overloaded, says("A")],
		steps: [send("first"), send("second"), abort("first"), advance(60_000)],
	}),
	run("a-key-replaced-while-a-call-is-held-releases-it", {
		...ALONE,
		exchanges: [overloaded, says("A")],
		steps: [send("first"), key("a", NEW_KEY)],
	}),
	run("an-abort-timer-that-fires-during-a-gate-hold-stops-the-held-call", {
		...ALONE,
		exchanges: [overloaded, says("A")],
		steps: [send("first", { abortAfterMs: 30_000 }), advance(30_000), advance(30_000)],
	}),
	run("an-abort-timer-that-fires-during-a-retry-wait-stops-the-call", {
		...ALONE,
		exchanges: [failing(503), says("A")],
		steps: [send("first", { abortAfterMs: 500 }), advance(500), advance(500)],
	}),
	run("a-caller-abort-after-a-send-failed-onward-stops-the-whole-run", {
		exchanges: [failing(401), says("B", USAGE, { headMs: 100 })],
		steps: [send("first", { abortAfterMs: 50 })],
	}),
	run("a-deadline-that-ends-the-call-is-timed-out-in-the-meter", {
		exchanges: [says("A", USAGE, { headMs: 100 }), says("B")],
		steps: [send("first", { deadlineMs: 50 })],
	}),
	run("a-cancel-with-no-deadline-is-cancelled-in-the-meter", {
		exchanges: [says("A", USAGE, { headMs: 100 }), says("B")],
		steps: [send("first", { abortAfterMs: 50 })],
	}),
	run("a-cancel-before-the-deadline-is-cancelled", {
		exchanges: [says("A", USAGE, { headMs: 200 })],
		steps: [send("first", { deadlineMs: 100, abortAfterMs: 50 })],
	}),
	run("a-deadline-before-the-cancel-is-timed-out", {
		exchanges: [says("A", USAGE, { headMs: 200 })],
		steps: [send("first", { deadlineMs: 50, abortAfterMs: 100 })],
	}),
	run("a-deadline-no-send-reaches-ends-nothing", {
		exchanges: [says("A")],
		steps: [send("first", { deadlineMs: 50 }), advance(100)],
	}),
];

const metering: CaseSource[] = [
	run("the-stats-hook-is-called-with-the-id-the-sink-gave-the-record", {
		usageIds: ["u1", "u2"],
		exchanges: [failing(401), says("B"), says("B"), says("B")],
		steps: [send("first"), send("second"), send("third")],
	}),
	run("a-failed-attempt-is-metered-with-what-it-billed", {
		...GO_WITH_B,
		usageIds: ["u1", "u2", "u3"],
		exchanges: [dies(BALANCE), says("B")],
		steps: [send("first")],
	}),
	run("a-call-that-names-another-feature-is-metered-as-given", {
		exchanges: [answers("T")],
		steps: [send("title", { tag: { feature: "analysis", lane: "title", folder: "/work/app" }, mode: "complete" })],
	}),
	run("long-cache-pricing-is-asked-for-by-chat-and-by-no-other-feature", {
		...ALONE,
		listings: { a: { [LUNA]: { price: PRICE, cacheWrite1h: true }, [OPUS]: { price: PRICE } } },
		exchanges: [says("A", WRITES), says("A", WRITES), says("A", WRITES), says("A", WRITES)],
		steps: [
			send("chat"),
			send("analysis", { tag: ANALYSIS }),
			send("wand", { tag: { feature: "wand" } }),
			send("without-listing", { model: OPUS }),
		],
	}),
	run("a-reply-that-bills-nobody-meters-without-usage", {
		...ALONE,
		exchanges: [
			sse([
				frame({ id: "gen-1", choices: [{ index: 0, delta: { content: "Hi" } }] }),
				frame({ id: "gen-1", choices: [{ index: 0, delta: { content: "" }, finish_reason: "stop" }] }),
				DONE_TEXT,
			]),
		],
		steps: [send("first")],
	}),
];

const notices: CaseSource[] = [
	run("a-spend-limit-notice-reads-the-keys-limits-and-tells-them", {
		...ALONE,
		exchanges: [failing(402), json({ data: { usage: 49, limit: 50, limit_reset: "monthly" } })],
		steps: [send("first")],
	}),
	run("a-spend-limit-notice-with-no-limits-read-still-ends-the-run", {
		...ALONE,
		exchanges: [failing(402), failing(500)],
		steps: [send("first")],
	}),
	run("a-spend-limit-on-an-earlier-route-reads-nothing-and-falls-through", {
		exchanges: [failing(402), says("B")],
		steps: [send("first")],
	}),
	run("the-key-limits-are-not-read-for-a-failure-that-is-no-spend-limit", {
		...ALONE,
		exchanges: [failing(403)],
		steps: [send("first")],
	}),
];

const sinks: CaseSource[] = [
	run("no-sink-given-the-run-is-the-same-and-tells-nothing", {
		trace: false,
		exchanges: [failing(503), failing(401), says("B")],
		steps: [send("first"), advance(1000)],
	}),
	run("no-sink-given-a-hold-and-an-abort-still-end-the-run", {
		trace: false,
		...ALONE,
		exchanges: [overloaded, says("A")],
		steps: [send("first"), send("second"), abort("first"), advance(60_000)],
	}),
];

export const lifecycleCases: CaseSource[] = [
	...fallThrough,
	...zeroRetention,
	...runRoute,
	...extraUsage,
	halving(),
	...overload,
	...dispositions(),
	...retries,
	...aborts,
	...metering,
	...notices,
	...sinks,
];
