// Provider health on scripted ports and network, and the wheel. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { caseOf, failing, reply } from "./catalogKit";

////////////////////////////////
//  Fixtures

const START = 2_000_000;

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;

const KEY = "fake-key-one";
const OTHER = "fake-key-two";

const MAIN = { id: "main", kind: "openrouter", label: "Main" };
const GONE = { id: "gone", kind: "openrouter", label: "Gone" };
const GO = { id: "go", kind: "opencode", plan: "go", label: "Go" };
const BOX = { id: "box", kind: "ollama", label: "Box", url: "http://lab.local:11434" };

type Fields = Record<string, unknown>;
type Step = Record<string, unknown>;

/** Replies of the checks. */
const OR_OK = reply({ data: { usage: 1 } });
const OC_OK = reply({ data: [{ id: "model-a" }] });
const HOST = { id: "host", kind: "compatible", label: "Host", url: "http://lab.local:8000/v1" };

const BOX_OK = reply({ models: [{ name: "local-a:27b" }, { name: "local-b:9b" }] });
const DENIED = { status: 401, json: { error: { message: "No." } } };

const route = (entry: Fields, key = "lab/model-a") => ({ entry, key });

/** The route a `take` step held under the name. */
const heldRoute = (entry: Fields, taken: string, key = "lab/model-a") => ({ entry, key, taken });
const take = (name: string, entry: Fields): Step => ({ kind: "take", name, route: route(entry) });

const failure = (category: string, extra: Fields = {}) => ({ category, status: category, message: category, ...extra });

const key = (id: string, text = KEY): Step => ({ kind: "key", id, answer: { kind: "key", key: text } });
const noKey = (id: string): Step => ({ kind: "key", id, answer: { kind: "none" } });
const list = (...entries: Fields[]): Step => ({ kind: "list", entries });
const check = (id: string, extra: Fields = {}): Step => ({ kind: "check", id, ...extra });
const keysOf = (id: string, extra: Fields = {}): Step => ({ kind: "keys", id, ...extra });
const served = (entry: Fields, billingSource?: string, extra: Fields = {}): Step => ({
	kind: "served",
	route: route(entry),
	...(billingSource === undefined ? {} : { billingSource }),
	...extra,
});
const failed = (entry: Fields, failureOf: Fields, onward = false, model = "lab/model-a"): Step => ({
	kind: "failed",
	route: route(entry, model),
	failure: failureOf,
	onward,
});
const billed = (id: string, source: string, extra: Fields = {}): Step => ({
	kind: "billed",
	id,
	reply: { source, ...extra },
});
const deferred = (entry: Fields, keep?: boolean, model = "lab/model-a"): Step => ({
	kind: "deferred",
	route: route(entry, model),
	...(keep === undefined ? {} : { keep }),
});
const blocked = (entry: Fields, keep?: boolean): Step => ({
	kind: "blocked",
	route: route(entry),
	...(keep === undefined ? {} : { keep }),
});
const tryAdmit = (request: string, entry: Fields, model = "lab/model-a"): Step => ({
	kind: "tryAdmit",
	request,
	route: route(entry, model),
});
const admit = (request: string, entry: Fields, model = "lab/model-a"): Step => ({
	kind: "admit",
	request,
	route: route(entry, model),
});
const settle = (request: string, outcome: string, retryAfterMs?: number): Step => ({
	kind: "settle",
	request,
	outcome,
	...(retryAfterMs === undefined ? {} : { retryAfterMs }),
});
const advance = (ms: number, extra: Fields = {}): Step => ({ kind: "advance", ms, ...extra });
const hold = (at: string): Step => ({ kind: "hold", at });
const release = (n: number, extra: Fields = {}): Step => ({ kind: "release", hold: n, ...extra });
const listenAndCheck = (id: string): Step => ({ kind: "listenAndCheck", id });
const dispose: Step = { kind: "dispose" };

const NONE = { kind: "none" };

const row = (entry: Fields, prepare: string): Fields => ({ entry, prepare, exchanges: [] });

const registryCase = (name: string, entries: Fields[], steps: Step[]): CaseSource =>
	caseOf("health.registry")(name, { entries, steps });
const watch = (name: string, extra: Fields = {}): Step => ({ kind: "watch", name, ...extra });
const listen = (name: string, extra: Fields = {}): Step => ({ kind: "listen", name, ...extra });
const EDITED = { ...MAIN, plan: "payg" };
const EDITED_AGAIN = { ...MAIN, plan: "go" };

const keysCase = (name: string, rows: Fields[]): CaseSource => caseOf("health.keys")(name, { rows });

const run = (name: string, input: Fields): CaseSource =>
	caseOf("health.run")(name, { startMs: START, entries: [], keys: {}, exchanges: [], ...input });

/** The balance cooldown of Go: a reply billed from the balance, then waiting for the next probe. */
const balance = (entry: Fields = GO): Step => served(entry, "balance");

/** Waits until the route is unskipped, one minute short then the last. */
function until(entry: Fields, minutes: number): Step[] {
	return [advance((minutes - 1) * MINUTE), deferred(entry), advance(MINUTE), deferred(entry)];
}

////////////////////////////////
//  Cases

export const healthCases: CaseSource[] = [
	// Construction and checks.
	run("construction-checks-nothing", {
		entries: [MAIN, GO, BOX],
		keys: { main: { kind: "key", key: KEY } },
		steps: [{ kind: "keys", id: "box" }, advance(1_000)],
	}),
	run("a-check-reads-the-key-and-shows-the-state", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [reply({ data: { usage: 4.12, limit: 20 } })],
		steps: [check("main")],
	}),
	run("a-key-kind-with-no-key-is-none-and-asks-nothing", {
		entries: [MAIN, { ...MAIN, id: "bare", label: "Bare" }],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [check("bare"), check("main")],
	}),
	run("a-url-kind-with-no-key-is-checked-without-one", {
		entries: [BOX],
		exchanges: [BOX_OK],
		steps: [check("box")],
	}),
	run("an-unlisted-entry-is-not-checked", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [check("elsewhere")],
	}),
	run("check-unread-checks-only-entries-with-no-status", {
		entries: [MAIN, GO],
		keys: { main: { kind: "key", key: KEY }, go: { kind: "key", key: KEY } },
		exchanges: [OR_OK, OC_OK],
		steps: [check("main"), { kind: "checkUnread" }],
	}),
	run("a-rejected-key-is-shown-as-rejected", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [DENIED],
		steps: [check("main"), keysOf("main"), deferred(MAIN), blocked(MAIN)],
	}),
	run("a-server-error-is-unreachable", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [failing(503)],
		steps: [check("main")],
	}),
	run("a-check-that-outlives-its-deadline-is-unreachable", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [{ ...OR_OK, headMs: 40_000 }],
		steps: [check("main")],
	}),
	run("a-check-that-ends-just-inside-its-deadline-is-shown", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [{ ...OR_OK, headMs: 29_000 }],
		steps: [check("main")],
	}),
	run("a-store-that-cannot-be-read-shows-unreachable", {
		entries: [MAIN],
		keys: { main: { kind: "unready" } },
		steps: [check("main")],
	}),
	run("a-store-that-cannot-be-read-leaves-a-url-kind-checked-without-a-key", {
		entries: [BOX],
		keys: { box: { kind: "unready" } },
		exchanges: [BOX_OK],
		steps: [check("box"), keysOf("box")],
	}),
	run("a-store-that-cannot-be-read-still-holds-an-optional-key-url-kind-out", {
		entries: [HOST],
		keys: { host: { kind: "unready" } },
		steps: [check("host"), keysOf("host")],
	}),
	run("a-store-that-cannot-be-read-still-holds-a-key-kind-out", {
		entries: [MAIN],
		keys: { main: { kind: "unready" } },
		steps: [check("main"), keysOf("main")],
	}),
	run("a-store-that-refuses-the-key-shows-rejected", {
		entries: [MAIN],
		keys: { main: { kind: "refused" } },
		steps: [check("main")],
	}),

	// A key replaced or removed.
	run("a-key-replaced-checks-again", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK, reply({ data: { usage: 2 } })],
		steps: [check("main"), key("main", OTHER)],
	}),
	run("a-key-replaced-clears-rejected-the-bill-the-pool-and-the-gate", {
		entries: [GO],
		keys: { go: { kind: "key", key: KEY } },
		exchanges: [DENIED, OC_OK],
		steps: [
			check("go"),
			billed("go", "go", { allowance: 0.25, cost: 0 }),
			failed(GO, failure("allowance", { retryAfterMs: 10 * MINUTE })),
			failed(GO, failure("unavailable"), true),
			deferred(GO),
			key("go", OTHER),
			deferred(GO),
		],
	}),
	run("a-key-removed-clears-everything-and-shows-none", {
		entries: [GO],
		keys: { go: { kind: "key", key: KEY } },
		exchanges: [OC_OK],
		steps: [
			check("go"),
			billed("go", "go", { allowance: 1 }),
			failed(GO, failure("allowance")),
			failed(GO, failure("unavailable"), true),
			noKey("go"),
			keysOf("go"),
		],
	}),
	run("a-key-changed-for-an-unlisted-entry-is-ignored", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [key("elsewhere"), check("main")],
	}),
	// An entry removed or added.
	run("a-removed-entry-has-no-state", {
		entries: [MAIN, GONE],
		keys: { main: { kind: "key", key: KEY }, gone: { kind: "key", key: KEY } },
		exchanges: [OR_OK, OR_OK],
		steps: [
			check("main"),
			check("gone"),
			billed("gone", "go", { allowance: 1 }),
			failed(GONE, failure("keyRejected")),
			list(MAIN),
			check("gone"),
		],
	}),
	run("an-entry-added-later-is-checked-once", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY }, fresh: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [
			list(MAIN, { ...MAIN, id: "fresh", label: "Fresh" }),
			list(MAIN, { ...MAIN, id: "fresh", label: "Fresh" }),
			list({ ...MAIN, id: "fresh", label: "Fresh again" }, MAIN),
		],
	}),
	run("a-removed-entry-releases-its-gates", {
		entries: [GO],
		steps: [failed(GO, failure("unavailable"), true), deferred(GO), list(), deferred(GO)],
	}),
	run("an-entry-removed-and-added-again-starts-clean", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK, OR_OK],
		steps: [check("main"), billed("main", "go", { allowance: 1 }), list(), list(MAIN)],
	}),
	run("a-label-changed-touches-nothing", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [check("main"), billed("main", "go", { allowance: 1 }), list({ ...MAIN, label: "Renamed" })],
	}),

	// A URL or plan changed.
	run("a-url-changed-rechecks-an-unreachable-server-and-clears-its-pauses-and-its-bill", {
		entries: [BOX],
		exchanges: [BOX_OK],
		steps: [
			failed(BOX, failure("network")),
			tryAdmit("a", BOX, "box/qwen"),
			settle("a", "overloaded"),
			billed("box", "local", { cost: 0 }),
			deferred(BOX, undefined, "box/qwen"),
			list({ ...BOX, url: "http://lab.local:11500" }),
			deferred(BOX, undefined, "box/qwen"),
		],
	}),
	run("a-plan-changed-clears-the-status-the-pool-the-gate-and-the-bill", {
		entries: [GO],
		keys: { go: { kind: "key", key: KEY } },
		exchanges: [OC_OK, OC_OK],
		steps: [
			check("go"),
			billed("go", "go", { allowance: 0.5 }),
			failed(GO, failure("allowance", { retryAfterMs: 10 * MINUTE })),
			failed(GO, failure("unavailable"), true),
			deferred(GO),
			list({ ...GO, plan: "payg" }),
			deferred(GO),
		],
	}),
	run("a-plan-added-is-a-change-of-plan", {
		entries: [{ id: "go", kind: "opencode", label: "Go" }],
		keys: { go: { kind: "key", key: KEY } },
		exchanges: [OC_OK, OC_OK],
		steps: [check("go"), list(GO)],
	}),
	run("an-unchanged-url-and-plan-leaves-the-status", {
		entries: [BOX],
		exchanges: [BOX_OK],
		steps: [check("box"), list({ ...BOX, label: "Box again" })],
	}),

	// Extra usage.
	run("enabling-extra-usage-clears-balance-cooldowns", {
		entries: [GO],
		steps: [
			balance(),
			deferred(GO),
			list({ ...GO, extraUsage: true }),
			deferred(GO),
			balance({ ...GO, extraUsage: true }),
			deferred(GO),
		],
	}),
	run("disabling-extra-usage-starts-a-cooldown-only-after-a-balance-bill", {
		entries: [{ ...GO, extraUsage: true }],
		steps: [billed("go", "go", { allowance: 0.1 }), list(GO), deferred(GO)],
	}),
	run("disabling-extra-usage-after-a-balance-bill-starts-the-cooldown", {
		entries: [{ ...GO, extraUsage: true }],
		steps: [billed("go", "balance", { cost: 0.1 }), list(GO), deferred(GO)],
	}),
	run("extra-usage-switched-in-flight-is-read-when-the-reply-is-served", {
		entries: [{ ...GO, extraUsage: true }],
		steps: [list(GO), served({ ...GO, extraUsage: true }, "balance"), deferred(GO)],
	}),
	run("extra-usage-touches-only-the-pool", {
		entries: [GO],
		keys: { go: { kind: "key", key: KEY } },
		exchanges: [OC_OK],
		steps: [check("go"), billed("go", "balance", { cost: 0.1 }), balance(), list({ ...GO, extraUsage: true })],
	}),

	// The balance cooldown.
	run("balance-probes-halve-toward-the-estimate-then-start-again", {
		entries: [GO],
		steps: [
			balance(),
			deferred(GO),
			...until(GO, 150),
			balance(),
			...until(GO, 75),
			balance(),
			...until(GO, 38),
			balance(),
			...until(GO, 19),
			balance(),
			...until(GO, 9),
			balance(),
			...until(GO, 5),
			balance(),
			...until(GO, 5),
			balance(),
			...until(GO, 150),
		],
	}),
	run("a-pool-bill-clears-the-balance-cooldown", {
		entries: [GO],
		steps: [balance(), deferred(GO), served(GO, "go"), deferred(GO)],
	}),
	run("a-go-plus-bill-clears-the-balance-cooldown", {
		entries: [GO],
		steps: [balance(), served(GO, "go-plus"), deferred(GO)],
	}),
	run("a-free-reply-ends-nothing", {
		entries: [GO],
		steps: [balance(), served(GO, "free"), deferred(GO)],
	}),
	run("a-reply-with-no-billing-changes-nothing", {
		entries: [GO],
		steps: [balance(), served(GO), deferred(GO)],
	}),
	run("pauses-keep-the-estimate-and-a-known-reset-wins", {
		entries: [GO],
		steps: [
			balance(),
			failed(GO, failure("allowance")),
			advance(2 * MINUTE),
			deferred(GO),
			failed(GO, failure("allowance", { resetsAt: START + 4 * HOUR })),
			balance(),
			advance(3 * HOUR),
			blocked(GO),
			deferred(GO),
			advance(HOUR),
			deferred(GO),
		],
	}),
	run("a-credit-refusal-pauses-the-entry", {
		entries: [GO],
		steps: [failed(GO, failure("outOfCredit")), deferred(GO), blocked(GO), advance(MINUTE), deferred(GO)],
	}),
	run("a-refusal-with-a-retry-after-pauses-that-long", {
		entries: [GO],
		steps: [
			failed(GO, failure("allowance", { retryAfterMs: 5 * MINUTE })),
			advance(4 * MINUTE),
			deferred(GO),
			advance(MINUTE),
			deferred(GO),
		],
	}),
	run("a-custom-pause-is-the-pool-pause", {
		entries: [GO],
		pauseMs: 5_000,
		steps: [failed(GO, failure("outOfCredit")), advance(4_999), deferred(GO), advance(1), deferred(GO)],
	}),
	run("a-balance-bill-of-an-entry-on-another-plan-is-not-judged", {
		entries: [{ ...GO, plan: "payg" }],
		steps: [served({ ...GO, plan: "payg" }, "balance"), deferred({ ...GO, plan: "payg" })],
	}),
	run("a-kept-balance-entry-serves-the-rest-of-its-run", {
		entries: [GO],
		steps: [
			served(GO, "balance", { keep: true }),
			deferred(GO),
			deferred(GO, true),
			blocked(GO),
			blocked(GO, true),
		],
	}),
	run("the-first-request-past-the-window-probes-and-others-wait", {
		entries: [GO],
		steps: [
			balance(),
			advance(150 * MINUTE),
			deferred(GO),
			tryAdmit("probe", GO),
			blocked(GO),
			served(GO, "free"),
			deferred(GO),
			served(GO, "go-plus"),
			deferred(GO),
		],
	}),
	run("a-call-the-gate-turns-away-reserves-nothing-and-only-the-probes-settle-frees-it", {
		entries: [GO],
		steps: [
			balance(),
			advance(100 * MINUTE),
			failed(GO, failure("unavailable", { retryAfterMs: HOUR }), true, "x/held"),
			advance(50 * MINUTE),
			tryAdmit("refused", GO, "x/held"),
			deferred(GO),
			tryAdmit("probe", GO),
			tryAdmit("other", GO),
			settle("other", "ok"),
			deferred(GO),
			settle("probe", "ok"),
			deferred(GO),
		],
	}),
	run("an-admission-past-the-window-takes-the-probe-through-admit", {
		entries: [GO],
		steps: [balance(), advance(150 * MINUTE), admit("probe", GO), blocked(GO), settle("probe", "ok"), deferred(GO)],
	}),
	run("a-probe-before-the-window-is-not-recorded", {
		entries: [GO],
		steps: [balance(), tryAdmit("early", GO), settle("early", "ok"), blocked(GO)],
	}),

	// Failures.
	run("failures-that-cool-a-gate-when-the-run-goes-on", {
		entries: [GO],
		steps: ["rateLimited", "notFound", "unavailable", "stream", "empty", "truncated", "idle", "network"].map(
			(category) =>
				failed(
					GO,
					failure(category, category === "rateLimited" ? { retryAfterMs: 3 * MINUTE } : {}),
					true,
					`lab/${category}`,
				),
		),
	}),
	run("failures-that-leave-the-gates-open-when-the-run-ends", {
		entries: [GO],
		steps: ["rateLimited", "notFound", "unavailable", "stream", "empty", "truncated", "idle", "network"].map(
			(category) => failed(GO, failure(category), false, `lab/${category}`),
		),
	}),
	run("failures-settled-elsewhere-change-nothing", {
		entries: [GO],
		steps: ["overloaded", "keyMissing", "refused", "badRequest", "replayRejected"].map((category) =>
			failed(GO, failure(category), true, `lab/${category}`),
		),
	}),
	run("a-rejected-key-failure-marks-the-entry-and-holds-it-out", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		steps: [
			failed(MAIN, failure("keyRejected")),
			failed(MAIN, failure("keyRejected")),
			keysOf("main"),
			deferred(MAIN),
			blocked(MAIN),
		],
	}),
	run("a-rejected-key-is-cleared-only-by-a-change", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [
			failed(MAIN, failure("keyRejected")),
			advance(HOUR),
			deferred(MAIN),
			key("main", OTHER),
			deferred(MAIN),
		],
	}),
	run("a-network-failure-of-a-url-entry-marks-it-unreachable", {
		entries: [BOX],
		steps: [failed(BOX, failure("network"), true), deferred(BOX)],
	}),
	run("a-check-brings-a-marked-server-back", {
		entries: [BOX],
		exchanges: [BOX_OK],
		steps: [failed(BOX, failure("network")), check("box"), keysOf("box"), deferred(BOX)],
	}),
	run("a-network-failure-of-an-unlisted-entry-marks-nothing", {
		entries: [],
		steps: [failed(BOX, failure("network")), keysOf("box")],
	}),
	run("a-cooled-key-model-leaves-its-entrys-other-models", {
		entries: [MAIN],
		steps: [
			failed(MAIN, failure("unavailable"), true, "lab/model-a"),
			deferred(MAIN, undefined, "lab/model-a"),
			deferred(MAIN, undefined, "lab/model-b"),
		],
	}),

	// Gates.
	run("a-cooled-gate-waits-then-probes-then-clears", {
		entries: [MAIN],
		steps: [
			failed(MAIN, failure("unavailable"), true),
			deferred(MAIN),
			tryAdmit("early", MAIN),
			advance(MINUTE),
			deferred(MAIN),
			tryAdmit("probe", MAIN),
			deferred(MAIN),
			settle("probe", "ok"),
			deferred(MAIN),
		],
	}),
	run("an-overload-settled-through-health-pauses-the-model", {
		entries: [MAIN],
		steps: [
			tryAdmit("a", MAIN),
			settle("a", "overloaded", 90_000),
			deferred(MAIN),
			advance(60_000),
			deferred(MAIN),
			advance(30_000),
			tryAdmit("probe", MAIN),
			settle("probe", "ok"),
			deferred(MAIN),
		],
	}),
	run("the-retry-after-of-a-cool-extends-the-pause", {
		entries: [MAIN],
		steps: [
			failed(MAIN, failure("unavailable", { retryAfterMs: 3 * MINUTE }), true),
			advance(2 * MINUTE),
			deferred(MAIN),
			advance(MINUTE),
			tryAdmit("probe", MAIN),
		],
	}),
	run("two-models-of-an-entry-wait-apart", {
		entries: [MAIN],
		steps: [
			failed(MAIN, failure("unavailable"), true, "lab/model-a"),
			failed(MAIN, failure("unavailable"), true, "lab/model-b"),
			advance(MINUTE),
			tryAdmit("a", MAIN, "lab/model-a"),
			settle("a", "ok"),
		],
	}),
	run("a-settled-probe-is-forgotten", {
		entries: [GO],
		steps: [
			balance(),
			advance(150 * MINUTE),
			tryAdmit("probe", GO),
			settle("probe", "ok"),
			settle("probe", "ok"),
			blocked(GO),
		],
	}),

	// Entries do not disturb each other.
	run("two-entries-do-not-disturb-each-other", {
		entries: [MAIN, GO, BOX],
		keys: { main: { kind: "key", key: KEY }, go: { kind: "key", key: KEY } },
		exchanges: [OR_OK, OC_OK, OR_OK],
		steps: [
			check("main"),
			check("go"),
			billed("go", "go", { allowance: 1 }),
			failed(GO, failure("allowance", { retryAfterMs: 10 * MINUTE })),
			failed(MAIN, failure("keyRejected")),
			failed(BOX, failure("network")),
			key("main", OTHER),
			deferred(GO),
			deferred(BOX),
		],
	}),
	run("a-key-change-of-one-entry-leaves-the-others-gates", {
		entries: [MAIN, GO],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [failed(GO, failure("unavailable"), true), failed(MAIN, failure("unavailable"), true), key("main")],
	}),
	run("an-entry-id-that-starts-another-does-not-share-gates", {
		entries: [{ ...MAIN, id: "ma" }, MAIN],
		keys: { ma: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [
			failed(MAIN, failure("unavailable"), true),
			failed({ ...MAIN, id: "ma" }, failure("unavailable"), true),
			key("ma"),
		],
	}),

	// A URL entry's recheck.
	run("an-unreachable-server-is-rechecked-when-routing-asks-at-most-every-minute", {
		entries: [BOX],
		exchanges: [BOX_OK, BOX_OK, BOX_OK],
		steps: [
			failed(BOX, failure("network")),
			keysOf("box"),
			keysOf("box"),
			failed(BOX, failure("network")),
			keysOf("box"),
			keysOf("box"),
			advance(MINUTE),
			keysOf("box"),
		],
	}),
	run("an-unreachable-server-that-is-still-down-stays-out", {
		entries: [BOX],
		exchanges: [failing(503), failing(503)],
		steps: [failed(BOX, failure("network")), keysOf("box"), keysOf("box"), advance(MINUTE), keysOf("box")],
	}),
	run("a-key-kind-is-never-rechecked-by-asking", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		steps: [failed(MAIN, failure("network")), keysOf("main"), keysOf("main")],
	}),
	run("the-answers-for-an-entry-no-list-holds-and-a-store-that-fails", {
		entries: [MAIN],
		keys: { main: { kind: "unready" } },
		steps: [keysOf("elsewhere"), keysOf("main")],
	}),

	// Bills.
	run("bills-start-at-the-first-reply-and-sum-on-the-entry", {
		entries: [MAIN, GONE],
		steps: [
			billed("main", "go", { allowance: 0.25, cost: 0 }),
			billed("main", "go", { allowance: 0.5, cost: 0 }),
			billed("main", "balance", { cost: 0.1 }),
			billed("main", "balance"),
			billed("gone", "go", { allowance: 1 }),
			billed("unlisted", "go", { allowance: 1 }),
		],
	}),
	run("a-bill-resets-on-a-key-change-or-removal", {
		entries: [MAIN, GONE],
		keys: { main: { kind: "key", key: KEY }, gone: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [
			billed("main", "go", { allowance: 0.25 }),
			billed("gone", "go", { allowance: 1 }),
			key("main"),
			list(MAIN),
		],
	}),

	// Changes told.
	run("changes-are-told-with-their-parts-in-the-shapes-order", {
		entries: [GO],
		keys: { go: { kind: "key", key: KEY } },
		exchanges: [OC_OK, OC_OK],
		steps: [
			check("go"),
			billed("go", "go", { allowance: 0.5 }),
			failed(GO, failure("allowance")),
			failed(GO, failure("unavailable"), true),
			key("go"),
		],
	}),
	run("a-dispose-tells-nothing-and-stops-listening", {
		entries: [GO],
		keys: { go: { kind: "key", key: KEY } },
		steps: [
			failed(GO, failure("unavailable"), true),
			dispose,
			key("go", OTHER),
			list(),
			failed(GO, failure("allowance")),
		],
	}),

	// A route held across a change of the entry.
	run("a-held-route-reads-extra-usage-live", {
		entries: [{ ...GO, extraUsage: true }],
		steps: [
			take("held", { ...GO, extraUsage: true }),
			list(GO),
			{ kind: "served", route: heldRoute({ ...GO, extraUsage: true }, "held"), billingSource: "balance" },
			{ kind: "deferred", route: heldRoute({ ...GO, extraUsage: true }, "held") },
			{ kind: "blocked", route: heldRoute({ ...GO, extraUsage: true }, "held") },
			list({ ...GO, extraUsage: true }),
			{ kind: "deferred", route: heldRoute(GO, "held") },
		],
	}),
	run("a-held-route-of-an-entry-removed-and-added-again-reports-nothing", {
		entries: [GO],
		steps: [
			take("held", GO),
			list(),
			list(GO),
			{ kind: "served", route: heldRoute(GO, "held"), billingSource: "balance" },
			{ kind: "failed", route: heldRoute(GO, "held"), failure: failure("allowance"), onward: false },
			deferred(GO),
			{ kind: "tryAdmit", request: "late", route: heldRoute(GO, "held") },
			failed(GO, failure("allowance")),
			deferred(GO),
		],
	}),
	run("a-held-route-of-an-entry-whose-key-was-replaced-still-reports", {
		entries: [GO],
		keys: { go: { kind: "key", key: KEY } },
		exchanges: [OC_OK, OC_OK],
		steps: [
			check("go"),
			take("held", GO),
			key("go", OTHER),
			{ kind: "failed", route: heldRoute(GO, "held"), failure: failure("allowance"), onward: false },
			deferred(GO),
		],
	}),

	// A check held open, and what happens to it.
	run("a-key-replaced-twice-during-a-check-leaves-only-the-newest-check", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [reply({ data: { usage: 2 } }), reply({ data: { usage: 3 } })],
		steps: [
			hold("check"),
			check("main"),
			key("main", OTHER),
			key("main", "fake-key-three"),
			release(1, { state: { kind: "ok", usage: 9 } }),
		],
	}),
	run("a-key-change-during-a-held-store-read-drops-that-check", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [reply({ data: { usage: 2 } })],
		steps: [hold("store"), check("main"), key("main", OTHER), release(1)],
	}),
	run("a-key-change-during-a-held-check-drops-that-check", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [reply({ data: { usage: 2 } })],
		steps: [hold("check"), check("main"), key("main", OTHER), release(1)],
	}),
	run("a-held-check-that-answers-after-a-key-change-is-dropped", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [reply({ data: { usage: 2 } })],
		steps: [hold("check"), check("main"), key("main", OTHER), release(1, { state: { kind: "ok", usage: 9 } })],
	}),
	run("an-entry-removed-during-its-check-has-no-state-and-tells-nothing-after", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		steps: [hold("check"), check("main"), list(), release(1, { state: { kind: "ok", usage: 9 } })],
	}),
	run("an-entry-removed-during-its-store-read-is-not-checked", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [hold("store"), check("main"), list(), release(1)],
	}),
	run("an-entry-whose-url-changed-during-its-check-is-checked-again-and-the-old-answer-is-dropped", {
		entries: [BOX],
		exchanges: [BOX_OK],
		steps: [
			hold("check"),
			check("box"),
			list({ ...BOX, url: "http://lab.local:11500" }),
			release(1, { state: { kind: "unreachable" } }),
		],
	}),
	run("an-entry-whose-url-changed-during-its-store-read-ends-without-effect", {
		entries: [BOX],
		exchanges: [BOX_OK],
		steps: [hold("store"), check("box"), list({ ...BOX, url: "http://lab.local:11500" }), release(1)],
	}),
	run("a-changed-kind-under-the-same-id-is-a-change-of-identity", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK, OC_OK],
		steps: [
			check("main"),
			billed("main", "go", { allowance: 1 }),
			failed(MAIN, failure("unavailable"), true),
			list({ ...MAIN, kind: "opencode", plan: "go" }),
		],
	}),
	run("a-check-that-outlives-its-deadline-and-answers-later-shows-unreachable-and-drops-the-answer", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		steps: [
			hold("check"),
			check("main"),
			advance(29_999, { idle: false }),
			advance(1),
			release(1, { state: { kind: "ok", usage: 9 } }),
		],
	}),
	run("a-store-read-that-never-answers-ends-at-the-deadline", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		steps: [
			hold("store"),
			check("main"),
			advance(29_999, { idle: false }),
			advance(1),
			release(1, { answer: { kind: "key", key: KEY } }),
		],
	}),
	run("a-late-served-after-a-removal-changes-nothing", {
		entries: [GO],
		steps: [list(), served(GO, "balance"), deferred(GO), list(GO)],
	}),
	run("a-late-failed-after-a-removal-changes-nothing", {
		entries: [GO],
		steps: [list(), failed(GO, failure("outOfCredit")), failed(GO, failure("allowance")), list(GO), deferred(GO)],
	}),
	run("an-entry-removed-and-added-again-after-late-events-starts-clean", {
		entries: [GO],
		keys: { go: { kind: "key", key: KEY } },
		exchanges: [OC_OK],
		steps: [list(), served(GO, "balance"), failed(GO, failure("outOfCredit")), list(GO), deferred(GO)],
	}),
	run("a-removed-and-added-url-entry-may-recheck-at-once", {
		entries: [BOX],
		exchanges: [BOX_OK, BOX_OK],
		steps: [
			failed(BOX, failure("network")),
			keysOf("box"),
			list(),
			list(BOX),
			failed(BOX, failure("network")),
			keysOf("box"),
		],
	}),
	run("a-list-that-repeats-an-id-keeps-its-first-row", {
		entries: [MAIN, { ...MAIN, kind: "opencode", plan: "go", label: "Again" }],
		steps: [
			billed("main", "go", { allowance: 1 }),
			list(MAIN, { ...MAIN, kind: "ollama", url: "http://lab.local:1" }),
			list({ ...MAIN, label: "Renamed" }, { ...MAIN, kind: "opencode" }),
		],
	}),
	run("a-dispose-checks-nothing", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [dispose, check("main"), key("main", OTHER), list(), billed("main", "go", { allowance: 1 })],
	}),
	run("a-listener-that-starts-a-check-is-told-after-the-changes-already-waiting", {
		entries: [MAIN],
		keys: { main: { kind: "key", key: KEY } },
		exchanges: [OR_OK],
		steps: [listenAndCheck("main"), billed("main", "go", { allowance: 1 })],
	}),

	// What a resolver reads for an entry.
	keysCase("key-kind-by-status", [
		{ ...row(MAIN, "none"), stored: { kind: "key", key: KEY } },
		{ ...row(MAIN, "none"), stored: NONE },
		{ ...row(MAIN, "checking"), stored: { kind: "key", key: KEY }, exchanges: [OR_OK] },
		{ ...row(MAIN, "ok"), stored: { kind: "key", key: KEY }, exchanges: [OR_OK] },
		{ ...row(MAIN, "ok"), stored: { kind: "key", key: KEY }, storedThen: NONE, exchanges: [OR_OK] },
		{ ...row(MAIN, "rejected"), stored: { kind: "key", key: KEY } },
		{ ...row(MAIN, "rejected"), stored: NONE },
		{ ...row(MAIN, "unreachable"), stored: { kind: "key", key: KEY } },
	]),
	keysCase("url-kind-by-status", [
		{ ...row(BOX, "none"), stored: NONE },
		{ ...row(BOX, "none"), stored: { kind: "key", key: KEY } },
		{ ...row(BOX, "checking"), stored: NONE, exchanges: [BOX_OK] },
		{ ...row(BOX, "ok"), stored: NONE, exchanges: [BOX_OK] },
		{ ...row(BOX, "rejected"), stored: { kind: "key", key: KEY } },
		{ ...row(BOX, "unreachable"), stored: NONE, exchanges: [BOX_OK] },
		{ ...row(BOX, "unreachable"), stored: NONE, exchanges: [failing(503)] },
	]),
	keysCase("the-store-answers-pass-through", [
		{ ...row(MAIN, "none"), stored: { kind: "unready" } },
		{ ...row(MAIN, "none"), stored: { kind: "refused" } },
		{ ...row(BOX, "none"), stored: { kind: "unready" } },
		{ ...row(BOX, "none"), stored: { kind: "refused" } },
	]),
	keysCase("an-unreachable-server-asked-twice-rechecks-once-and-stays-out-meanwhile", [
		{ ...row(BOX, "unreachable"), stored: NONE, exchanges: [BOX_OK], asks: 2 },
		{ ...row(BOX, "unreachable"), stored: NONE, exchanges: [{ ...BOX_OK, headMs: 20_000 }], asks: 3 },
		{ ...row(BOX, "unreachable"), stored: NONE, exchanges: [failing(503)], asks: 2 },
	]),
	keysCase("a-first-check-in-flight-does-not-hold-a-url-server-out", [
		{ ...row(BOX, "checking"), stored: NONE, exchanges: [BOX_OK], asks: 2 },
	]),
	keysCase("a-key-change-during-a-check-drops-that-check", [
		{
			...row(MAIN, "checking"),
			stored: { kind: "key", key: KEY },
			exchanges: [reply({ data: { usage: 2 } })],
			tell: ["main"],
		},
		{
			...row(MAIN, "ok"),
			stored: { kind: "key", key: KEY },
			exchanges: [OR_OK, reply({ data: { usage: 3 } })],
			tell: ["main"],
		},
	]),
	keysCase("an-entry-the-list-does-not-hold-is-none", [
		{ ...row(MAIN, "none"), listed: false, stored: { kind: "key", key: KEY } },
		{ ...row(BOX, "none"), listed: false, stored: { kind: "unready" } },
	]),
	keysCase("a-change-of-key-is-told-by-id-until-the-listener-is-released", [
		{ ...row(MAIN, "none"), stored: NONE, tell: ["main", "elsewhere"], tellAfter: ["main", "later"] },
		{ ...row(MAIN, "none"), stored: NONE },
	]),

	// The registry tells in the order made.
	registryCase(
		"a-change-made-by-a-watcher-is-told-after-the-change-it-came-from",
		[MAIN],
		[watch("w1", { onFirst: { kind: "list", entries: [EDITED_AGAIN] } }), watch("w2"), list(EDITED)],
	),
	registryCase(
		"a-key-change-made-inside-a-list-change-is-told-after-it",
		[MAIN],
		[watch("w1", { onFirst: { kind: "key", id: "main" } }), watch("w2"), listen("k1"), list(EDITED)],
	),
	registryCase(
		"a-watcher-that-throws-does-not-starve-the-others",
		[MAIN],
		[watch("w1", { throws: true }), watch("w2"), listen("k1"), list(EDITED), { kind: "key", id: "main" }],
	),
	registryCase(
		"disposing-retires-every-incarnation-and-tells-the-watchers-last",
		[MAIN, GONE],
		[watch("w1"), { kind: "dispose" }, list(EDITED)],
	),
];
