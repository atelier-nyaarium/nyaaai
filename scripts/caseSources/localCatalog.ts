// `LocalCatalog` cases on a scripted server and the timer wheel. Every value is made up.
//
// A script gives the server's replies, one list per call a catalog can make, in the order they are asked.
// A reply spends `delayMs` of the wheel before it answers; a reply with `fails` throws instead.
// A step is a call, calls at once, a reply seen for a tag, a rediscovery, time passing or new rows.

import type { CaseSource } from "../caseAdapter";
import { caseOf, NOW } from "./catalogKit";

////////////////////////////////
//  Fixtures

const A = "local-a:27b";
const B = "local-b:9b";
const C = "local-c:3b";

const ROW_A = `local:lab/${A}`;
const ROW_B = `local:lab/${B}`;
const ROW_C = `local:lab/${C}`;

/** A configured context and efforts. */
const MA = { tag: A, tools: true, thinking: true, efforts: ["low", "high"], context: 8192 };

/** No context of its own; the trained length is shorter than any loaded one. */
const MB = { tag: B, tools: true, thinking: false, efforts: [], trained: 4096 };

const MC = { tag: C, tools: false, thinking: false, efforts: [] };

/** Facts not read this load. */
const MB_ASLEEP = { tag: B, tools: true, thinking: false, efforts: [], probed: false };

const MB_WOKEN = { tag: B, tools: true, thinking: true, efforts: ["medium"], context: 16_384 };

const HOME = { kept: false, price: "free" };

const REMOTE = { kept: true, price: "unknown" };

const NO_SERVER = { live: false, loaded: false, warm: false, models: [], loads: [], warms: [] };

const READS = { loaded: true };

const FAILS = { fails: true, models: [] };

const models = (...held: unknown[]) => ({ models: held });

const slow = (delayMs: number, ...held: unknown[]) => ({ delayMs, models: held });

const contexts = (...held: Array<[string, number]>) => ({
	contexts: held.map(([tag, context]) => ({ tag, context })),
});

////////////////////////////////
//  Functions & Helpers

const views = (extra: Record<string, unknown> = {}) => ({ kind: "call", call: { kind: "views", ...extra } });

const call = (body: Record<string, unknown>) => ({ kind: "call", call: body });

const discovered = (extra: Record<string, unknown> = {}) => call({ kind: "discovered", ...extra });

const settled = (tag: string, times?: number) => ({ kind: "settled", tag, ...(times === undefined ? {} : { times }) });

const advance = (ms: number) => ({ kind: "advance", ms });

const rows = (...keys: string[]) => ({ kind: "rows", rows: keys });

const rediscover = { kind: "rediscover" };

const run = (name: string, input: Record<string, unknown>): CaseSource => {
	const { server, ...rest } = input;
	return caseOf("local.catalogRun")(name, {
		startMs: NOW,
		entry: "lab",
		terms: HOME,
		rows: [ROW_A, ROW_B],
		...rest,
		server: { ...NO_SERVER, ...(server as Record<string, unknown> | undefined) },
	});
};

////////////////////////////////
//  Cases

export const localCatalogCases: CaseSource[] = [
	// Discovery.
	run("run-discovers-once-and-keeps-the-list", {
		server: { models: [models(MA, MB)] },
		steps: [views(), views(), advance(86_400_000), views()],
	}),
	run("run-views-are-the-tested-rows-of-this-entry-the-server-lists", {
		rows: [ROW_A, "local:other/local-b:9b", ROW_C, "local:lab/gone:1b"],
		server: { models: [models(MA, MB, MC)] },
		steps: [views()],
	}),
	run("run-views-follow-the-servers-order-not-the-rows", {
		rows: [ROW_B, ROW_A],
		server: { models: [models(MA, MB)] },
		steps: [views()],
	}),
	run("run-no-rows-no-views-but-the-server-is-still-asked", {
		rows: [],
		server: { models: [models(MA)] },
		steps: [views(), views()],
	}),
	run("run-rows-change-between-calls-with-no-new-discovery", {
		rows: [],
		server: { models: [models(MA, MB)] },
		steps: [views(), rows(ROW_A), views(), rows(ROW_A, ROW_B), views(), rows(), views()],
	}),
	run("run-views-by-retention-and-tools", {
		terms: REMOTE,
		rows: [ROW_A, ROW_B, ROW_C],
		server: { models: [models(MA, MB, MC)] },
		steps: [views({ zdrOnly: true }), views({ zdrOnly: false })],
	}),
	run("run-views-of-a-home-server-ignore-the-retention-switch", {
		rows: [ROW_A, ROW_C],
		server: { models: [models(MA, MC)] },
		steps: [views({ zdrOnly: true }), views({ zdrOnly: false })],
	}),
	run("run-views-take-the-use-and-change-nothing", {
		server: { models: [models(MA, MB)] },
		steps: [views({ use: "chat" }), views({ use: "compaction" }), views({ use: "analysis" })],
	}),
	run("run-discovered-lists-keys-display-and-the-capped-context", {
		server: { models: [models(MA, { ...MB, context: 8192 }, MC)] },
		steps: [discovered()],
	}),
	run("run-discovered-lists-every-model-not-only-the-tested-ones", {
		rows: [],
		server: { models: [models(MA, MB, MC)] },
		steps: [discovered()],
	}),
	run("run-fresh-discovery-rereads-with-the-known-list", {
		server: { models: [models(MA, MB), models(MA, MB)] },
		steps: [views(), discovered({ fresh: true }), discovered()],
	}),
	run("run-fresh-discovery-replaces-the-list", {
		server: { models: [models(MA, MB), models(MB)] },
		steps: [views(), discovered({ fresh: true }), views()],
	}),
	run("run-calls-at-once-share-one-discovery", {
		server: { models: [models(MA, MB)] },
		steps: [{ kind: "together", calls: [{ kind: "views" }, { kind: "discovered" }, { kind: "views" }] }],
	}),
	run("run-fresh-calls-at-once-share-one-discovery", {
		server: { models: [models(MA, MB), models(MB)] },
		steps: [
			views(),
			{
				kind: "together",
				calls: [
					{ kind: "discovered", fresh: true },
					{ kind: "discovered", fresh: true },
				],
			},
		],
	}),
	run("run-discovery-counts-each-success-once", {
		server: { models: [models(MA), models(MA), models(MA)] },
		steps: [views(), discovered({ fresh: true }), rediscover, views()],
	}),

	// Failures.
	run("run-failure-before-any-success-waits-a-minute", {
		server: { models: [FAILS, models(MA, MB)] },
		steps: [views(), advance(59_999), views(), advance(1), views()],
	}),
	run("run-failure-before-any-success-gives-no-list", {
		server: { models: [FAILS] },
		steps: [views(), discovered()],
	}),
	run("run-a-fresh-call-skips-the-wait", {
		server: { models: [FAILS, models(MA, MB)] },
		steps: [views(), discovered({ fresh: true }), views()],
	}),
	run("run-a-failed-fresh-call-starts-the-wait-again", {
		server: { models: [FAILS, FAILS, models(MA)] },
		steps: [views(), advance(30_000), discovered({ fresh: true }), advance(59_999), views(), advance(1), views()],
	}),
	run("run-failure-after-success-keeps-the-list", {
		server: { models: [models(MA, MB), FAILS, FAILS] },
		steps: [views(), discovered({ fresh: true }), views(), rediscover, views()],
	}),
	run("run-failure-after-success-is-not-a-refresh", {
		server: { models: [models(MA), FAILS] },
		steps: [views(), discovered({ fresh: true })],
	}),
	run("run-rediscovery-runs-in-the-background-and-lands", {
		server: { models: [models(MA), models(MA, MB)] },
		steps: [views(), rediscover, views()],
	}),
	run("run-rediscovery-while-one-runs-shares-it", {
		server: { models: [models(MA), models(MB)] },
		steps: [
			views(),
			{
				kind: "together",
				calls: [
					{ kind: "discovered", fresh: true },
					{ kind: "discovered", fresh: true },
				],
			},
		],
	}),

	// The deadline and the caller.
	run("run-a-slow-discovery-ends-at-ten-seconds", {
		server: { models: [slow(10_000, MA, MB), models(MA, MB)] },
		steps: [views(), advance(60_000), views()],
	}),
	run("run-a-discovery-just-inside-the-deadline-lands", {
		server: { models: [slow(9999, MA, MB)] },
		steps: [views(), views()],
	}),
	run("run-a-failure-after-the-deadline-is-the-same-as-any-failure", {
		server: { models: [slow(10_000, MA)] },
		steps: [views(), advance(59_999), views()],
	}),
	run("run-a-callers-abort-leaves-the-discovery-running", {
		server: { models: [slow(20, MA, MB)] },
		steps: [views({ abortAfterMs: 5 }), views()],
	}),
	run("run-a-caller-aborted-before-the-call-still-starts-the-discovery", {
		server: { models: [models(MA, MB)] },
		steps: [views({ abortedBefore: true }), views()],
	}),
	run("run-one-caller-aborts-the-other-gets-the-views", {
		server: { models: [slow(20, MA, MB)] },
		steps: [{ kind: "together", calls: [{ kind: "views", abortAfterMs: 5 }, { kind: "views" }] }],
	}),
	run("run-a-caller-that-aborts-a-fresh-discovery-leaves-it-running", {
		server: { models: [models(MA), slow(20, MA, MB)] },
		steps: [views(), discovered({ fresh: true, abortAfterMs: 5 }), views()],
	}),

	// A reply was seen: the loaded context.
	run("run-settled-reads-the-loaded-context", {
		server: { ...READS, models: [models(MA, MB)], loads: [contexts([B, 65_536])] },
		steps: [views(), settled(B), views()],
	}),
	run("run-settled-reads-once-when-the-server-is-not-live", {
		server: { ...READS, models: [models(MA, MB)], loads: [contexts([B, 65_536]), contexts([B, 2048])] },
		steps: [views(), settled(B), settled(B), views()],
	}),
	run("run-settled-rereads-each-time-when-the-server-is-live", {
		server: {
			...READS,
			live: true,
			models: [models(MA, MB)],
			loads: [contexts([B, 65_536]), contexts([B, 2048]), contexts([B, 2048])],
		},
		steps: [views(), settled(B), views(), settled(B), views(), settled(B), views()],
	}),
	run("run-settled-unchanged-contexts-say-nothing", {
		server: { ...READS, live: true, models: [models(MA, MB)], loads: [contexts([B, 1024]), contexts([B, 1024])] },
		steps: [views(), settled(B), settled(B)],
	}),
	run("run-settled-skips-a-model-with-a-configured-context", {
		server: { ...READS, models: [models(MA, MB)], loads: [contexts([A, 1024])] },
		steps: [views(), settled(A), views()],
	}),
	run("run-settled-skips-a-tag-the-server-does-not-list", {
		server: { ...READS, models: [models(MA, MB)], loads: [contexts([B, 1024])] },
		steps: [views(), settled("ghost:1b")],
	}),
	run("run-settled-before-any-discovery-does-nothing", {
		server: { ...READS, models: [models(MA, MB)], loads: [contexts([B, 1024])] },
		steps: [settled(B), views()],
	}),
	run("run-settled-skips-a-server-without-a-loaded-reader", {
		server: { models: [models(MA, MB)] },
		steps: [views(), settled(B), views()],
	}),
	run("run-settled-twice-at-once-reads-once", {
		server: { ...READS, models: [models(MA, MB)], loads: [contexts([B, 65_536]), contexts([B, 2048])] },
		steps: [views(), settled(B, 2), views()],
	}),
	run("run-settled-failure-is-forgotten-and-the-next-reply-retries", {
		server: { ...READS, models: [models(MA, MB)], loads: [{ fails: true, contexts: [] }, contexts([B, 65_536])] },
		steps: [views(), settled(B), views(), settled(B), views()],
	}),
	run("run-settled-read-ends-at-ten-seconds", {
		server: {
			...READS,
			models: [models(MA, MB)],
			loads: [{ delayMs: 10_000, ...contexts([B, 65_536]) }, contexts([B, 1024])],
		},
		steps: [views(), settled(B), views(), settled(B), views()],
	}),
	run("run-settled-read-just-inside-the-deadline-lands", {
		server: { ...READS, models: [models(MA, MB)], loads: [{ delayMs: 9999, ...contexts([B, 65_536]) }] },
		steps: [views(), settled(B), views()],
	}),
	run("run-settled-contexts-may-name-other-models", {
		server: { ...READS, models: [models(MA, MB)], loads: [contexts([A, 1024], [B, 2048])] },
		steps: [views(), settled(B), views(), discovered()],
	}),

	// A model whose facts were not read.
	run("run-settled-rereads-an-unprobed-model-once-a-minute", {
		server: {
			models: [models(MA, MB_ASLEEP), models(MA, MB_ASLEEP), models(MA, MB_ASLEEP), models(MA, MB_ASLEEP)],
		},
		steps: [views(), settled(B), settled(B), advance(59_999), settled(B), advance(1), settled(B), settled(B)],
	}),
	run("run-the-reread-picks-up-the-facts-the-model-has-now", {
		server: { ...READS, models: [models(MA, MB_ASLEEP), models(MA, MB_WOKEN)], loads: [contexts([B, 1024])] },
		steps: [views(), settled(B), views(), settled(B), views()],
	}),
	run("run-the-reread-window-is-per-tag", {
		rows: [ROW_A, ROW_B],
		server: {
			models: [
				models({ ...MA, probed: false }, MB_ASLEEP),
				models({ ...MA, probed: false }, MB_ASLEEP),
				models({ ...MA, probed: false }, MB_ASLEEP),
			],
		},
		steps: [views(), settled(A), settled(B), settled(A), settled(B)],
	}),
	run("run-a-reread-that-fails-still-spends-the-window", {
		server: { models: [models(MA, MB_ASLEEP), FAILS, models(MA, MB_WOKEN)] },
		steps: [views(), settled(B), settled(B), advance(60_000), settled(B), views()],
	}),
	run("run-an-unprobed-model-never-reads-a-loaded-context", {
		server: { ...READS, models: [models(MA, MB_ASLEEP), models(MA, MB_ASLEEP)], loads: [contexts([B, 1024])] },
		steps: [views(), settled(B)],
	}),

	// A model that is loaded or not.
	run("run-a-live-server-forgets-a-context-it-no-longer-reports", {
		server: {
			live: true,
			models: [models({ ...MB, loaded: 2048 }, MA), models(MB, MA), models({ ...MB, loaded: 1024 }, MA)],
		},
		steps: [views(), discovered({ fresh: true }), views(), discovered({ fresh: true }), views()],
	}),
	run("run-a-server-that-is-not-live-keeps-a-context-it-stops-reporting", {
		server: { models: [models({ ...MB, loaded: 2048 }, MA), models(MB, MA)] },
		steps: [views(), discovered({ fresh: true }), views()],
	}),
	run("run-a-model-that-leaves-the-list-loses-its-loaded-context", {
		server: { models: [models({ ...MB, loaded: 2048 }, MA), models(MA), models(MB, MA)] },
		steps: [views(), discovered({ fresh: true }), discovered({ fresh: true }), views()],
	}),
	run("run-a-loaded-context-from-a-reply-survives-a-list-that-is-not-live", {
		server: { ...READS, models: [models(MA, MB), models(MA, MB)], loads: [contexts([B, 65_536])] },
		steps: [views(), settled(B), discovered({ fresh: true }), views()],
	}),

	// Loading a model for a test.
	run("run-warming-an-unprobed-model-loads-it-and-reads-the-list-again", {
		server: { warm: true, models: [models(MA, MB_ASLEEP), models(MA, MB_WOKEN)], warms: [{}] },
		steps: [views(), call({ kind: "warmed", tag: B }), views()],
	}),
	run("run-warming-a-model-that-is-probed-does-nothing", {
		server: { warm: true, models: [models(MA, MB_WOKEN)], warms: [{}] },
		steps: [views(), call({ kind: "warmed", tag: B })],
	}),
	run("run-warming-with-no-loader-does-nothing", {
		server: { models: [models(MA, MB_ASLEEP)] },
		steps: [views(), call({ kind: "warmed", tag: B })],
	}),
	run("run-warming-a-tag-the-server-does-not-list-does-nothing", {
		server: { warm: true, models: [models(MA, MB_ASLEEP)], warms: [{}] },
		steps: [views(), call({ kind: "warmed", tag: "ghost:1b" })],
	}),
	run("run-warming-discovers-first-when-nothing-was-listed", {
		server: { warm: true, models: [models(MA, MB_ASLEEP), models(MA, MB_WOKEN)], warms: [{}] },
		steps: [call({ kind: "warmed", tag: B }), views()],
	}),
	run("run-a-warm-that-fails-throws-and-skips-the-reread", {
		server: { warm: true, models: [models(MA, MB_ASLEEP), models(MA, MB_WOKEN)], warms: [{ fails: true }] },
		steps: [views(), call({ kind: "warmed", tag: B }), views()],
	}),
	run("run-a-slow-warm-has-no-deadline-of-its-own", {
		server: { warm: true, models: [models(MA, MB_ASLEEP), models(MA, MB_WOKEN)], warms: [{ delayMs: 20_000 }] },
		steps: [views(), call({ kind: "warmed", tag: B }), views()],
	}),
	run("run-a-warm-the-caller-aborts-throws-at-once", {
		server: { warm: true, models: [models(MA, MB_ASLEEP), models(MA, MB_WOKEN)], warms: [{ delayMs: 20 }] },
		steps: [views(), call({ kind: "warmed", tag: B, abortAfterMs: 5 })],
	}),
	run("run-a-warm-when-discovery-fails-does-nothing", {
		server: { warm: true, models: [FAILS], warms: [{}] },
		steps: [call({ kind: "warmed", tag: B })],
	}),

	// The tester's view.
	run("run-the-testers-view-ignores-the-retention-gate", {
		terms: REMOTE,
		server: { models: [models(MA, MB)] },
		steps: [views(), call({ kind: "viewFor", row: ROW_A })],
	}),
	run("run-the-testers-view-of-a-home-server-is-the-same-view", {
		server: { models: [models(MA, MB)] },
		steps: [call({ kind: "viewFor", row: ROW_A }), views()],
	}),
	run("run-the-testers-view-without-tools-is-refused-as-no-tools", {
		rows: [ROW_C],
		server: { models: [models(MA, MC)] },
		steps: [call({ kind: "viewFor", row: ROW_C })],
	}),
	run("run-the-testers-view-of-a-tag-the-server-lacks-throws", {
		server: { models: [models(MA, MB)] },
		steps: [call({ kind: "viewFor", row: "local:lab/gone:1b" })],
	}),
	run("run-the-testers-view-of-another-entrys-row-throws", {
		server: { models: [models(MA, MB)] },
		steps: [call({ kind: "viewFor", row: "local:other/local-a:27b" })],
	}),
	run("run-the-testers-view-with-no-list-throws", {
		server: { models: [FAILS] },
		steps: [call({ kind: "viewFor", row: ROW_A })],
	}),
	run("run-the-testers-view-carries-the-loaded-context", {
		server: { ...READS, models: [models(MA, MB)], loads: [contexts([B, 65_536])] },
		steps: [views(), settled(B), call({ kind: "viewFor", row: ROW_B })],
	}),

	// Nothing to show.
	run("run-an-unreachable-server-lists-nothing", {
		server: { models: [FAILS] },
		steps: [discovered(), discovered({ fresh: true })],
	}),
	run("run-a-server-with-no-models", {
		server: { models: [models()] },
		steps: [views(), discovered()],
	}),
];
