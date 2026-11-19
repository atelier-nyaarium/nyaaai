// The resolver on scripted ports and stub kinds. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { caseOf } from "./catalogKit";

////////////////////////////////
//  Fixtures

const LUNA = "openai/gpt-6-luna";
const GEMINI = "google/gemini-3.8-flash";
const OSS = "openai/gpt-oss-120b";
const HOSTED = "ollama:qwen";
const LOCAL = "local:lab/local-a:27b";
const OTHER = "local:lab2/local-a:27b";
const MISSING = "openrouter:lab/none";

const ALL_KINDS = ["openrouter", "opencode", "ollama", "lmstudio", "llamacpp", "compatible"];

const KEY = { kind: "key", key: "key-0123456789abcdef" };
const OTHER_KEY = { kind: "key", key: "key-fedcba9876543210" };
const NONE = { kind: "none" };
const REFUSED = { kind: "refused" };
const UNREADY = { kind: "unready" };

const FIRST = { id: "a", kind: "openrouter", label: "First" };
const SECOND = { id: "b", kind: "openrouter", label: "Second" };
const THIRD = { id: "c", kind: "openrouter", label: "Third" };
const GO = { id: "go", kind: "opencode", plan: "go", label: "Go" };
const PAYG = { id: "payg", kind: "opencode", plan: "payg", label: "Pay-as-you-go" };
const LAB = { id: "lab", kind: "ollama", label: "Lab", url: "http://lab.local:11434" };
const LAB2 = { id: "lab2", kind: "ollama", label: "Lab Two", url: "http://lab2.local:11434" };
const STUDIO = { id: "studio", kind: "lmstudio", label: "Studio", url: "http://lab.local:1234" };

const COMPAT = { id: "c", kind: "compatible", label: "Host", url: "http://lab.local:8000/v1" };
const BAD = { id: "bad", kind: "compatible", label: "Broken", url: "not a url" };
const COMPAT_ROW = "compatible:qwen";

const ALLOWANCE = {
	status: 429,
	body: { type: "error", error: { type: "SubscriptionUsageLimitError", message: "Limit.", resetsAt: 1_790_341_200 } },
};
const RATE = { status: 429, body: { type: "error", error: { type: "RateLimitError", message: "Slow down." } } };

const ROWS = [LUNA, GEMINI, OSS];

////////////////////////////////
//  Functions & Helpers

const routes = (model: string, use = "chat") => ({ kind: "routes", model, use });
const resolve = (model: string, use = "chat") => ({ kind: "resolve", model, use });
const rows = (use = "chat") => ({ kind: "rows", use });
const candidates = (use = "chat") => ({ kind: "candidates", use });
const list = (...entries: unknown[]) => ({ kind: "list", entries });
const answer = (id: string, held: unknown) => ({ kind: "answer", id, answer: held });
const registry = (...held: string[]) => ({ kind: "registry", rows: held });
const zdr = (on: boolean) => ({ kind: "zdr", on });
const refreshed = (entry: string) => ({ kind: "refreshed", entry });
const send = (from: number, candidate = 0, extra: Record<string, unknown> = {}) => ({
	kind: "send",
	from,
	candidate,
	...extra,
});
const during = (changes: unknown[], call: unknown) => ({ kind: "during", changes, call });
const route = (entry: unknown) => ({ kind: "route", entry });
const dispose = { kind: "dispose" };

const run = (name: string, input: Record<string, unknown>): CaseSource =>
	caseOf("resolver.routes")(name, {
		entries: [FIRST, SECOND],
		answers: { a: KEY, b: KEY },
		failing: [],
		rows: ROWS,
		kinds: ALL_KINDS,
		listings: {},
		zdrOnly: true,
		...input,
	});

const served = caseOf("resolver.servedBy");

const ZDR_GEMINI = { a: { [GEMINI]: { status: "noZdrHost", whenZdrOff: "ok" } } };

////////////////////////////////
//  Cases

export const resolverCases: CaseSource[] = [
	// Priority order and keys.
	run("order-the-first-eligible-entry-serves", {
		answers: { a: KEY, b: KEY },
		steps: [routes(LUNA), resolve(LUNA), candidates()],
	}),
	run("order-a-missing-key-passes-to-the-next-entry", {
		answers: { a: NONE, b: KEY },
		steps: [routes(LUNA), resolve(LUNA), candidates(), rows()],
	}),
	run("order-a-key_that-arrives-moves-the-route-up", {
		answers: { a: NONE, b: KEY },
		steps: [routes(LUNA), answer("a", KEY), routes(LUNA), candidates()],
	}),
	run("order-the-list-order-is-the-priority", {
		steps: [routes(LUNA), list(SECOND, FIRST), routes(LUNA), resolve(LUNA)],
	}),
	run("order-three-entries-and-what-each-passed-over", {
		entries: [FIRST, SECOND, THIRD],
		answers: { a: NONE, b: REFUSED, c: KEY },
		steps: [routes(LUNA)],
	}),
	run("order-the-passed-position-counts-the-routes-found-before", {
		entries: [FIRST, SECOND, THIRD],
		answers: { a: KEY, b: NONE, c: KEY },
		steps: [routes(LUNA), resolve(LUNA)],
	}),
	run("order-resolve-takes-only-the-first-route", {
		entries: [FIRST, SECOND, THIRD],
		answers: { a: KEY, b: KEY, c: KEY },
		steps: [resolve(LUNA), routes(LUNA)],
	}),
	run("order-no-entries-at-all", { entries: [], answers: {}, steps: [routes(LUNA), rows(), candidates()] }),

	// Key answers.
	run("keys-a-refused-key-passes-with-its-reason", {
		answers: { a: REFUSED, b: KEY },
		steps: [routes(LUNA)],
	}),
	run("keys-an-unready-key-passes-as-the-network", {
		answers: { a: UNREADY, b: KEY },
		steps: [routes(LUNA)],
	}),
	run("keys-a-store-that-cannot-be-read-is-unready", {
		failing: ["a"],
		steps: [routes(LUNA)],
	}),
	run("keys-a-key-read-once-per-entry-per-read", {
		answers: { a: KEY, b: KEY },
		steps: [routes(LUNA), rows(), candidates()],
	}),
	run("keys-an-entry-with-no-answer-has-none", {
		answers: {},
		steps: [routes(LUNA)],
	}),
	run("keys-a-url-kind-needs-no-key", {
		entries: [LAB],
		answers: {},
		rows: [HOSTED],
		steps: [routes(HOSTED)],
	}),
	run("keys-a-url-kind-with-a-refused-key", {
		entries: [LAB],
		answers: { lab: REFUSED },
		rows: [HOSTED],
		steps: [routes(HOSTED)],
	}),
	run("keys-a-url-kind-that-is-not-ready", {
		entries: [LAB],
		answers: { lab: UNREADY },
		rows: [HOSTED],
		steps: [routes(HOSTED)],
	}),
	run("keys-a-url-kind-with-a-key", {
		entries: [LAB],
		answers: { lab: KEY },
		rows: [HOSTED],
		steps: [routes(HOSTED), send(0)],
	}),

	// What a request ends on.
	run("refusal-every-entry-lacks-a-key", {
		answers: {},
		steps: [routes(LUNA), resolve(LUNA)],
	}),
	run("refusal-a-key-beats-a-listing", {
		answers: { a: NONE, b: KEY },
		listings: { b: { [GEMINI]: { status: "retired" } } },
		steps: [routes(GEMINI)],
	}),
	run("refusal-missing-key-outranks-a-refused-one", {
		answers: { a: REFUSED, b: NONE },
		steps: [routes(LUNA)],
	}),
	run("refusal-a-refused-key-outranks-an-unready-one", {
		answers: { a: UNREADY, b: REFUSED },
		steps: [routes(LUNA)],
	}),
	run("refusal-an-unready-key-alone", {
		answers: { a: UNREADY, b: UNREADY },
		steps: [routes(LUNA)],
	}),
	run("refusal-an-unready-key-outranks-a-listing-that-is-retired", {
		answers: { a: UNREADY, b: KEY },
		listings: { b: { [LUNA]: { status: "retired" } } },
		steps: [routes(LUNA)],
	}),
	run("refusal-retired-in-every-entry", {
		listings: { a: { [LUNA]: { status: "retired" } }, b: { [LUNA]: { status: "retired" } } },
		steps: [routes(LUNA), resolve(LUNA)],
	}),
	run("refusal-no-zero-retention-host-outranks-retired", {
		listings: { a: { [LUNA]: { status: "retired" } }, b: { [LUNA]: { status: "noZdrHost" } } },
		steps: [routes(LUNA)],
	}),
	run("refusal-retired-first-then-no-host-still-names-the-no-host-entry", {
		listings: { a: { [LUNA]: { status: "noZdrHost" } }, b: { [LUNA]: { status: "retired" } } },
		steps: [routes(LUNA)],
	}),
	run("refusal-an-unlisted-row-is-no-model", {
		listings: { a: { [LUNA]: { status: "unlisted" } }, b: { [LUNA]: { status: "unlisted" } } },
		steps: [routes(LUNA)],
	}),
	run("refusal-a-view-that-cannot-use-tools-is-no-model", {
		listings: { a: { [LUNA]: { status: "noTools" } }, b: { [LUNA]: { status: "noTools" } } },
		steps: [routes(LUNA)],
	}),
	run("refusal-an-unknown-view-is-served", {
		listings: { a: { [LUNA]: { status: "unknown" } } },
		steps: [routes(LUNA), rows()],
	}),
	run("refusal-a-key-no-row-names", { steps: [routes(MISSING), resolve(MISSING)] }),
	run("refusal-a-row-no-entry-is-bound-to", { steps: [routes(HOSTED)], rows: [HOSTED] }),
	run("refusal-a-kind-the-resolver-does-not-hold", {
		entries: [GO, FIRST],
		kinds: ["openrouter"],
		steps: [routes(LUNA), candidates()],
	}),
	run("refusal-only-an-unheld-kind", { entries: [GO], kinds: ["openrouter"], steps: [routes(LUNA)] }),
	run("refusal-a-listing-behind-a-missing-key-shows-in-rows", {
		answers: { a: NONE, b: NONE },
		listings: { a: { [GEMINI]: { status: "noZdrHost" } } },
		steps: [rows()],
	}),

	// Zero retention.
	run("zdr-gemini-has-no-host-while-zero-retention-is-on", {
		listings: { a: ZDR_GEMINI.a, b: ZDR_GEMINI.a },
		steps: [routes(GEMINI), rows()],
	}),
	run("zdr-turning-it-off-opens-the-route", {
		listings: { a: ZDR_GEMINI.a, b: ZDR_GEMINI.a },
		steps: [routes(GEMINI), zdr(false), routes(GEMINI), rows()],
	}),
	run("zdr-a-route-sends-the-switch-as-it-stands-when-the-request-goes", {
		steps: [routes(LUNA), zdr(false), send(0), zdr(true), send(0)],
	}),
	run("zdr-a-route-reads-the-switch-live", {
		zdrOnly: false,
		steps: [routes(LUNA), zdr(true), routes(LUNA), send(0), send(2)],
	}),
	run("zdr-a-route-held-from-a-resolve-sends-live", {
		steps: [resolve(LUNA), zdr(false), send(0)],
	}),
	run("zdr-the-listing-is-read-with-the-switch-as-it-stands", {
		listings: { a: ZDR_GEMINI.a },
		steps: [routes(GEMINI), zdr(false), routes(GEMINI), candidates()],
	}),
	run("zdr-a-route-another-entry-gives-when-the-first-has-no-host", {
		listings: { a: ZDR_GEMINI.a },
		steps: [routes(GEMINI), candidates()],
	}),

	// Kinds.
	run("kinds-one-model-two-kinds-in-list-order", {
		entries: [GO, FIRST],
		answers: { go: KEY, a: KEY },
		steps: [routes(LUNA), send(0, 0), send(0, 1)],
	}),
	run("kinds-a-model-only-one-kind-has", {
		entries: [GO, FIRST],
		answers: { go: KEY, a: KEY },
		steps: [routes(OSS), candidates()],
	}),
	run("kinds-two-plans-of-one-kind", {
		entries: [PAYG, GO],
		answers: { payg: KEY, go: KEY },
		steps: [routes(LUNA), candidates()],
	}),
	run("kinds-the-codec-is-the-kinds-own", {
		entries: [GO, FIRST],
		answers: { go: KEY, a: KEY },
		steps: [routes(LUNA)],
	}),

	// Local rows.
	run("local-a-local-row-is-served-by-its-own-entry-only", {
		entries: [LAB, LAB2],
		answers: {},
		rows: [LOCAL],
		steps: [routes(LOCAL), candidates()],
	}),
	run("local-a-local-row-of-another-entry-is-served-by-that-one", {
		entries: [LAB, LAB2],
		answers: {},
		rows: [OTHER, LOCAL],
		steps: [routes(OTHER), candidates()],
	}),
	run("local-a-hosted-row-is-served-by-every-entry-of-its-kind", {
		entries: [LAB, LAB2],
		answers: {},
		rows: [HOSTED],
		steps: [routes(HOSTED), candidates()],
	}),
	run("local-a-local-and-a-hosted-row-for-one-model", {
		entries: [LAB, LAB2, FIRST],
		answers: { a: KEY },
		rows: [LOCAL, HOSTED, LUNA],
		steps: [rows(), candidates()],
	}),
	run("local-an-entry-of-another-kind-does-not-serve-the-row", {
		entries: [STUDIO, LAB],
		answers: {},
		rows: [LOCAL],
		steps: [routes(LOCAL)],
	}),
	run("local-the-server-is-down", {
		entries: [LAB],
		answers: { lab: UNREADY },
		rows: [LOCAL],
		steps: [routes(LOCAL), rows()],
	}),
	run("local-a-server-that-is-listed-behind-a-down-one", {
		entries: [LAB, LAB2],
		answers: { lab: UNREADY, lab2: NONE },
		rows: [HOSTED],
		steps: [routes(HOSTED)],
	}),

	// Rows and candidates.
	run("rows-every-row-with-its-first-serving-entry", {
		answers: { a: NONE, b: KEY },
		steps: [rows(), candidates()],
	}),
	run("rows-the-first-entry-only", {
		entries: [FIRST, SECOND, THIRD],
		answers: { a: KEY, b: KEY, c: KEY },
		steps: [rows()],
	}),
	run("rows-an-unserved-row-keeps-the-view-behind-the-refusal", {
		answers: { a: KEY, b: KEY },
		listings: { a: { [GEMINI]: { status: "noZdrHost" } }, b: { [GEMINI]: { status: "retired" } } },
		steps: [rows()],
	}),
	run("rows-keys-missing-everywhere-show-the-view-behind-them", {
		answers: {},
		steps: [rows()],
	}),
	run("rows-an-unserved-row-no-catalog-lists-reads-unknown", {
		answers: {},
		listings: { a: { [GEMINI]: { status: "unlisted" } }, b: { [GEMINI]: { status: "unlisted" } } },
		steps: [rows()],
	}),
	run("rows-a-key-that-fails-does-not-hide-the-view-behind-it", {
		answers: { a: UNREADY, b: NONE },
		listings: { a: { [LUNA]: { status: "retired" } } },
		steps: [rows()],
	}),
	run("rows-the-registry-changes", {
		steps: [rows(), registry(LUNA), rows(), candidates(), registry(LUNA, OSS, GEMINI), rows()],
	}),
	run("rows-a-model-the-registry-dropped-is-no-model", {
		steps: [routes(GEMINI), registry(LUNA), routes(GEMINI)],
	}),
	run("rows-an-empty-registry", { rows: [], steps: [rows(), candidates(), routes(LUNA)] }),

	// Routes, built once per what identifies them.
	run("cache-a-route-is-made-once-per-entry", {
		steps: [routes(LUNA), routes(GEMINI), rows(), candidates(), resolve(OSS)],
	}),
	run("cache-the-same-list-told-again-keeps-the-routes", {
		steps: [routes(LUNA), list(FIRST, SECOND), routes(LUNA)],
	}),
	run("cache-a-reordered-list-keeps-the-routes", {
		steps: [routes(LUNA), list(SECOND, FIRST), routes(LUNA)],
	}),
	run("cache-an-entry-that-leaves-drops-its-route", {
		steps: [routes(LUNA), list(SECOND), routes(LUNA), list(FIRST, SECOND), routes(LUNA)],
	}),
	run("cache-a-relabelled-entry-keeps-its-route", {
		steps: [routes(LUNA), list({ ...FIRST, label: "Renamed" }, SECOND), routes(LUNA)],
	}),
	run("cache-a-changed-plan-drops-the-route", {
		entries: [GO],
		answers: { go: KEY },
		steps: [routes(LUNA), list({ ...GO, plan: "payg" }), routes(LUNA)],
	}),
	run("cache-a-changed-url-drops-the-route", {
		entries: [LAB],
		answers: { lab: KEY },
		rows: [HOSTED],
		steps: [routes(HOSTED), list({ ...LAB, url: "http://lab.local:9999" }), routes(HOSTED)],
	}),
	run("cache-a-changed-kind-drops-the-route", {
		entries: [FIRST],
		steps: [routes(LUNA), list({ ...FIRST, kind: "opencode" }), routes(LUNA)],
	}),
	run("cache-nothing-built-for-an-entry-no-row-reaches", {
		entries: [FIRST, SECOND],
		answers: { a: NONE, b: NONE },
		steps: [routes(LUNA)],
	}),
	run("cache-a-route-dropped-and-back-is-built-again", {
		steps: [routes(LUNA), list(SECOND), list(FIRST, SECOND), routes(LUNA)],
	}),
	run("cache-the-registry-told-changes-no-route", {
		steps: [routes(LUNA), registry(LUNA, GEMINI), routes(LUNA)],
	}),
	run("cache-a-refresh-of-a-catalog-is-told-with-its-entry", {
		steps: [routes(LUNA), refreshed("a"), refreshed("b"), refreshed("nobody")],
	}),

	// A key stays with its URL.
	run("url-a-resolved-route-answers-no-key-once-its-entry-has-moved", {
		entries: [LAB],
		answers: { lab: KEY },
		rows: [HOSTED],
		steps: [
			routes(HOSTED),
			send(0),
			list({ ...LAB, url: "http://lab.local:9999" }),
			send(0),
			routes(HOSTED),
			send(4),
		],
	}),
	run("url-a-route-of-a-removed-url-entry-answers-no-key", {
		entries: [LAB],
		answers: { lab: KEY },
		rows: [HOSTED],
		steps: [routes(HOSTED), list(), send(0)],
	}),
	run("url-a-key-that-changes-is-read-at-the-request", {
		steps: [routes(LUNA), send(0), answer("a", OTHER_KEY), send(0), answer("a", NONE), send(0)],
	}),
	run("url-a-refused-key-reads-as-none-at-the-request", {
		steps: [routes(LUNA), answer("a", REFUSED), send(0)],
	}),
	run("url-a-key-entry-removed-from-the-list-answers-none", {
		steps: [routes(LUNA), list(SECOND), send(0)],
	}),
	run("url-an-entry-moved-back-answers-again", {
		entries: [LAB],
		answers: { lab: KEY },
		rows: [HOSTED],
		steps: [routes(HOSTED), list({ ...LAB, url: "http://lab.local:9999" }), send(0), list(LAB), send(0)],
	}),
	run("keys-a-url-kind-whose-store-cannot-be-read-is-unreachable", {
		entries: [LAB],
		answers: { lab: KEY },
		failing: ["lab"],
		rows: [HOSTED],
		steps: [routes(HOSTED), rows()],
	}),
	run("keys-a-route-whose-read-fails-sends-no-key", {
		answers: { a: KEY },
		steps: [routes(LUNA), answer("a", UNREADY), send(0)],
	}),
	run("keys-a-url-route-whose-read-fails-sends-no-key", {
		entries: [COMPAT],
		answers: { c: KEY },
		rows: [COMPAT_ROW],
		steps: [routes(COMPAT_ROW), send(0), answer("c", UNREADY), send(0)],
	}),

	// A route is for one id, kind, plan and URL.
	run("route-an-entry-edited-to-another-kind-sends-no-key-to-the-other-host", {
		entries: [FIRST],
		answers: { a: KEY },
		steps: [
			resolve(LUNA),
			list({ ...FIRST, kind: "opencode", plan: "go" }),
			answer("a", OTHER_KEY),
			send(0),
			resolve(LUNA),
			send(4),
		],
	}),
	run("route-an-entry-removed-and-added-as-another-kind", {
		entries: [FIRST],
		answers: { a: KEY },
		steps: [
			resolve(LUNA),
			list(),
			list({ id: "a", kind: "opencode", plan: "go", label: "Zen" }),
			answer("a", OTHER_KEY),
			send(0),
			resolve(LUNA),
			send(5),
		],
	}),
	run("route-an-entry-removed-and-added-as-the-same-kind-takes-the-new-key", {
		entries: [FIRST],
		answers: { a: KEY },
		steps: [
			resolve(LUNA),
			send(0),
			list(),
			send(0),
			list(FIRST),
			answer("a", OTHER_KEY),
			send(0),
			resolve(LUNA),
			send(7),
		],
	}),
	run("route-go-edited-to-payg-with-a-route-held", {
		entries: [GO],
		answers: { go: KEY },
		steps: [resolve(LUNA), send(0), list({ ...GO, plan: "payg" }), send(0), resolve(LUNA), send(4)],
	}),
	run("route-an-entry-relabelled-keeps-its-route-and-its-key", {
		entries: [FIRST],
		answers: { a: KEY },
		steps: [resolve(LUNA), list({ ...FIRST, label: "Renamed" }), send(0), resolve(LUNA), send(3)],
	}),
	run("route-the-url-of-a-url-entry-is-the-host-asked", {
		entries: [COMPAT],
		answers: { c: KEY },
		rows: [COMPAT_ROW],
		steps: [
			routes(COMPAT_ROW),
			send(0),
			list({ ...COMPAT, url: "http://lab.local:9000/v1" }),
			send(0),
			resolve(COMPAT_ROW),
			send(4),
		],
	}),

	// The list is checked after the read.
	run("during-the-list-changes-while-the-key-is-read", {
		entries: [COMPAT],
		answers: { c: KEY },
		rows: [COMPAT_ROW],
		steps: [
			routes(COMPAT_ROW),
			during([list({ ...COMPAT, url: "http://lab.local:9000/v1" }), answer("c", OTHER_KEY)], send(0)),
		],
	}),
	run("during-the-entry-is-removed-while-the-key-is-read", {
		entries: [COMPAT],
		answers: { c: KEY },
		rows: [COMPAT_ROW],
		steps: [routes(COMPAT_ROW), during([list()], send(0))],
	}),
	run("during-the-key-changes-while-it-is-read", {
		answers: { a: KEY },
		steps: [routes(LUNA), during([answer("a", OTHER_KEY)], send(0))],
	}),

	// An entry that left during a call.
	run("left-an-entry-that-leaves-while-its-key-is-read-is-skipped", {
		answers: { a: KEY, b: KEY },
		steps: [during([list(SECOND)], routes(LUNA)), routes(LUNA)],
	}),
	run("left-the-only-entry-leaves-while-its-key-is-read", {
		entries: [FIRST],
		steps: [during([list()], routes(LUNA))],
	}),
	run("left-an-entry-edited-while-its-key-is-read-is-skipped", {
		answers: { a: KEY, b: KEY },
		steps: [during([list({ ...FIRST, label: "Moved", kind: "opencode", plan: "go" }, SECOND)], routes(LUNA))],
	}),
	run("left-an-entry-relabelled-while-its-key-is-read-stands", {
		answers: { a: KEY, b: KEY },
		steps: [during([list({ ...FIRST, label: "Renamed" }, SECOND)], routes(LUNA))],
	}),
	run("left-rows-with-an-entry-that-leaves", {
		answers: { a: KEY, b: KEY },
		steps: [during([list(SECOND)], rows())],
	}),

	// A disposed resolver builds nothing.
	run("dispose-routes-after-it-answer-as-over-an-empty-list", {
		steps: [routes(LUNA), dispose, routes(LUNA), resolve(LUNA), rows(), candidates(), dispose],
	}),
	run("dispose-a-direct-route-refuses", {
		steps: [dispose, route(FIRST)],
	}),
	run("dispose-stops-every-catalog-and-a-list-change-after-it-does-nothing", {
		steps: [routes(LUNA), dispose, list(SECOND), routes(LUNA)],
	}),

	// A local row whose entry is not listed.
	run("local-a-row-of-an-entry-that-is-not-listed-is-no-model", {
		entries: [LAB],
		answers: {},
		rows: [OTHER],
		steps: [routes(OTHER), resolve(OTHER), rows(), candidates()],
	}),
	run("local-a-row-of-an-entry-that-was-removed", {
		entries: [LAB],
		answers: {},
		rows: [LOCAL],
		steps: [routes(LOCAL), list(LAB2), routes(LOCAL)],
	}),
	run("local-a-row-of-an-entry-of-a-kind-not-held-keeps-keymissing", {
		entries: [LAB],
		answers: {},
		kinds: ["openrouter"],
		rows: [LOCAL],
		steps: [routes(LOCAL)],
	}),

	// An entry whose route cannot be made.
	run("bad-an-entry-that-cannot-be-routed-is-passed-as-unreachable", {
		entries: [BAD, COMPAT],
		answers: { bad: KEY, c: KEY },
		rows: [COMPAT_ROW],
		steps: [routes(COMPAT_ROW), resolve(COMPAT_ROW), rows(), candidates(), route(BAD), route(COMPAT)],
	}),
	run("bad-the-only-entry-cannot-be-routed", {
		entries: [BAD],
		answers: { bad: KEY },
		rows: [COMPAT_ROW],
		steps: [routes(COMPAT_ROW), rows(), candidates()],
	}),
	run("bad-a-route-that-cannot-be-made-is-tried-again", {
		entries: [BAD],
		answers: { bad: KEY },
		rows: [COMPAT_ROW],
		steps: [routes(COMPAT_ROW), list({ ...BAD, url: "http://lab.local:8000/v1" }), routes(COMPAT_ROW)],
	}),
	run("bad-the-good-entry-behind-keeps-its-rows", {
		entries: [BAD, FIRST],
		answers: { bad: KEY, a: KEY },
		rows: [COMPAT_ROW, LUNA],
		steps: [rows(), candidates()],
	}),

	// The routes of one step are reported in the list's order.
	run("order-the-routes-a-step-made-are-in-the-lists-order", {
		entries: [LAB, FIRST, STUDIO],
		answers: { a: KEY },
		rows: [HOSTED, LUNA, "lmstudio:qwen"],
		steps: [rows()],
	}),

	// Clocks: one per route.
	run("clock-an-allowance-reset-is-dated-by-the-scripted-clock", {
		entries: [GO],
		answers: { go: KEY },
		startMs: 1_790_337_600_000,
		steps: [
			resolve(LUNA),
			send(0, 0, { fail: ALLOWANCE }),
			send(0, 0, { fail: RATE }),
			send(0, 0, { mode: "stream", fail: ALLOWANCE }),
		],
	}),
	run("clock-a-later-clock-moves-the-reset", {
		entries: [GO],
		answers: { go: KEY },
		startMs: 1_790_340_000_000,
		steps: [resolve(LUNA), send(0, 0, { fail: ALLOWANCE })],
	}),

	// A stream's request.
	run("stream-the-switch-is-stamped-at-the-request", {
		steps: [resolve(LUNA), zdr(false), send(0, 0, { mode: "stream" }), zdr(true), send(0, 0, { mode: "stream" })],
	}),
	run("stream-a-failure-reads-as-the-kind-reads-it", {
		steps: [
			resolve(LUNA),
			send(0, 0, { mode: "stream", fail: { status: 401, body: { error: { message: "No." } } } }),
		],
	}),

	// What a catalog's refresh tells.
	run("refreshed-a-relabelled-entry-is-told-as-the-list-holds-it", {
		steps: [routes(LUNA), list({ ...FIRST, label: "Renamed" }, SECOND), refreshed("a")],
	}),
	run("refreshed-a-route-of-an-entry-that-left-is-told-with-its-own", {
		entries: [FIRST],
		steps: [routes(LUNA), list(), refreshed("a")],
	}),
	run("refreshed-a-catalog-that-fails-to-stop-does-not-keep-the-others", {
		answers: { a: KEY, b: KEY },
		disposeFails: ["a", "b"],
		steps: [routes(LUNA), list()],
	}),
];

////////////////////////////////
//  servedBy

const SERVED = [
	{
		tag: "openrouter-curated",
		kind: "openrouter",
		entry: { id: "a", kind: "openrouter", label: "First" },
		key: LUNA,
	},
	{
		tag: "opencode-go-curated",
		kind: "opencode",
		entry: { id: "go", kind: "opencode", plan: "go", label: "Go" },
		key: LUNA,
	},
	{
		tag: "opencode-payg-curated",
		kind: "opencode",
		entry: { id: "payg", kind: "opencode", plan: "payg", label: "Pay" },
		key: GEMINI,
	},
	{
		tag: "ollama-hosted",
		kind: "ollama",
		entry: { id: "lab", kind: "ollama", label: "Lab", url: "http://lab.local:11434" },
		key: HOSTED,
	},
	{
		tag: "ollama-local",
		kind: "ollama",
		entry: { id: "lab", kind: "ollama", label: "Lab", url: "http://lab.local:11434" },
		key: LOCAL,
	},
	{
		tag: "lmstudio-hosted",
		kind: "lmstudio",
		entry: { id: "studio", kind: "lmstudio", label: "Studio", url: "http://lab.local:1234" },
		key: "lmstudio:qwen",
	},
	{
		tag: "compatible-hosted",
		kind: "compatible",
		entry: { id: "host", kind: "compatible", label: "Host", url: "http://lab.local:8000/v1" },
		key: "compatible:qwen",
	},
];

export const resolverServedCases: CaseSource[] = [
	...SERVED.flatMap(({ tag, ...given }) => [
		served(`served-${tag}-with-a-view`, { ...given, use: "chat", view: "ok", zdrOnly: true, flippedTo: false }),
		served(`served-${tag}-with-no-view-reads-an-unknown-row`, {
			...given,
			use: "chat",
			view: null,
			zdrOnly: false,
			flippedTo: true,
		}),
	]),
	served("served-a-bare-key-is-the-kinds-own-id", {
		...SERVED[0],
		key: "lab/none",
		use: "chat",
		view: null,
		zdrOnly: true,
		flippedTo: true,
	}),
	served("served-a-curated-key-with-no-view-keeps-its-key-as-the-binding", {
		...SERVED[1],
		use: "analysis",
		view: null,
		zdrOnly: true,
		flippedTo: true,
	}),
	served("served-a-retired-view-is-still-served", {
		...SERVED[0],
		use: "compaction",
		view: "retired",
		zdrOnly: true,
		flippedTo: false,
	}),
	served("served-a-row-the-kind-has-no-binding-for-takes-the-key", {
		...SERVED[0],
		key: OSS,
		kind: "opencode",
		entry: SERVED[1]?.entry,
		use: "chat",
		view: "ok",
		zdrOnly: true,
		flippedTo: true,
	}),
];

////////////////////////////////
//  Uses of a consumer's own

const SUMMARY = { name: "summary", requires: ["schema"], hides: true };
const TRIAGE = { name: "triage", requires: [], hides: false };

export const resolverUseCases: CaseSource[] = [
	caseOf("resolver.uses")("uses-the-standard-ones", {
		rows: [LUNA, GEMINI],
		uses: [
			{ name: "chat", requires: [], hides: true },
			{ name: "compaction", requires: ["schema"], hides: false },
			{ name: "analysis", requires: ["schema"], hides: false },
		],
	}),
	caseOf("resolver.uses")("uses-a-use-of-a-consumer-reaches-the-catalogs-as-it-is", {
		rows: [LUNA, GEMINI],
		uses: [SUMMARY, TRIAGE, { name: "chat", requires: ["schema"], hides: false }],
	}),
	served("served-a-custom-use-reaches-the-catalog", {
		...SERVED[0],
		tag: undefined,
		use: SUMMARY,
		view: "ok",
		zdrOnly: true,
		flippedTo: false,
	}),
	served("served-a-custom-use-with-no-view", {
		...SERVED[1],
		tag: undefined,
		use: TRIAGE,
		view: null,
		zdrOnly: true,
		flippedTo: true,
	}),
];
