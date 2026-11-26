// Budget cases: the table, one budgeted exchange, and a span's reads. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import type { Json } from "./codecFixtures";
import { jsonReply, piece } from "./transportKit";

////////////////////////////////
//  Constants

const TEXT_HEAD: Array<[string, string]> = [["content-type", "text/plain"]];

////////////////////////////////
//  Functions & Helpers

const budget = (totalMs: number, idleMs: number, maxBytes = 1000): Json => ({ totalMs, idleMs, maxBytes });

/** A body of `count` chunks of `text`, each `delayMs` after the read. */
const body = (count: number, text: string, delayMs: number, over: Json = {}): Json => ({
	status: 200,
	headers: TEXT_HEAD,
	chunks: Array.from({ length: count }, () => piece(text, delayMs)),
	...over,
});

const fetched = (name: string, input: Json): CaseSource => ({ kind: "budget.fetch", name, input });

const table = (name: string, changes: Json): CaseSource => ({ kind: "budget.table", name, input: { changes } });

const spanned = (name: string, input: Json): CaseSource => ({ kind: "budget.span", name, input });

////////////////////////////////
//  Cases

const tables: CaseSource[] = [
	table("table-the-presets", {}),
	table("table-a-change-inside-the-ceilings-is-applied", {
		discover: { totalMs: 60_000 },
		stream: { maxBytes: 1024 * 1024 * 1024, idleMs: 30 * 60_000 },
		warm: { totalMs: 24 * 60 * 60_000 },
	}),
	table("table-a-total-past-its-ceiling-is-refused", { stream: { totalMs: 24 * 60 * 60_000 + 1 } }),
	table("table-an-idle-past-its-ceiling-is-refused", { reply: { idleMs: 30 * 60_000 + 1 } }),
	table("table-a-size-past-its-ceiling-is-refused", { catalog: { maxBytes: 1024 * 1024 * 1024 + 1 } }),
	table("table-a-zero-is-refused", { reply: { idleMs: 0 } }),
	table("table-a-fraction-is-refused", { catalog: { maxBytes: 1.5 } }),
	table("table-an-unknown-preset-is-refused", { slow: { totalMs: 1 } }),
];

const exchanges: CaseSource[] = [
	fetched("fetch-the-total-ends-while-the-head-is-pending", {
		budget: budget(100, 200),
		exchange: body(1, "a", 0, { headMs: 150 }),
	}),
	fetched("fetch-the-idle-ends-between-two-chunks", {
		budget: budget(1000, 30),
		exchange: { status: 200, headers: TEXT_HEAD, chunks: [piece("a", 10), piece("b", 50)] },
	}),
	fetched("fetch-a-drip-inside-the-idle-bound-ends-at-the-total", {
		budget: budget(100, 30),
		exchange: body(10, "a", 20),
	}),
	fetched("fetch-many-small-chunks-end-at-the-bytes", {
		budget: budget(1000, 30, 10),
		exchange: body(6, "ab", 1),
	}),
	fetched("fetch-a-body-at-exactly-its-bytes-ends-well", {
		budget: budget(1000, 30, 10),
		exchange: body(5, "ab", 1),
	}),
	fetched("fetch-an-error-body-past-the-bytes", {
		budget: budget(1000, 30, 20),
		exchange: jsonReply({ error: { message: "x".repeat(40) } }, 500),
	}),
	fetched("fetch-a-consumer-holding-a-chunk-past-the-idle-bound-trips-nothing", {
		budget: budget(1000, 30),
		exchange: body(3, "a", 5),
		holds: [100, 100],
	}),
	fetched("fetch-a-total-that-passes-while-the-consumer-holds-the-last-chunk-shows-at-the-next-read", {
		budget: budget(100, 50),
		exchange: body(2, "a", 10),
		holds: [0, 200],
	}),
	fetched("fetch-tie-a-caller-abort-wins-over-the-total", {
		budget: budget(100, 200),
		exchange: body(1, "a", 0, { headMs: 150 }),
		abortAfterMs: 100,
	}),
	fetched("fetch-tie-the-total-wins-over-the-idle", {
		budget: budget(100, 100),
		exchange: body(1, "a", 0, { headMs: 150 }),
	}),
	fetched("fetch-tie-the-total-wins-over-a-chunk-due-at-that-instant", {
		budget: budget(100, 200),
		exchange: body(2, "a", 50),
	}),
	fetched("fetch-tie-a-chunk-due-as-its-idle-clock-ends-is-delivered", {
		budget: budget(1000, 30),
		exchange: body(3, "a", 30),
	}),
	fetched("fetch-tie-a-head-due-as-its-idle-clock-ends-is-delivered", {
		budget: budget(1000, 30),
		exchange: body(1, "a", 0, { headMs: 30 }),
	}),
	fetched("fetch-an-abort-after-a-breach-keeps-the-breach", {
		budget: budget(1000, 30),
		exchange: { status: 200, headers: TEXT_HEAD, chunks: [piece("a", 10), piece("b", 50)] },
		abortAfterMs: 60,
		afterMs: 100,
	}),
	fetched("fetch-a-breach-after-an-abort-keeps-the-abort", {
		budget: budget(50, 1000),
		exchange: { status: 200, headers: TEXT_HEAD, chunks: [piece("a", 10), piece("b", 100)] },
		abortAfterMs: 20,
		afterMs: 100,
	}),
	fetched("fetch-a-dropped-body-ends-as-the-network-said", {
		budget: budget(1000, 30),
		exchange: body(1, "a", 5, { end: "dropped", error: "socket hang up" }),
	}),
];

const spans: CaseSource[] = [
	spanned("span-a-second-read-gets-what-the-first-left", {
		budget: budget(10_000, 10_000),
		exchanges: [jsonReply({ a: 1 }, 200, [], { headMs: 4000 }), jsonReply({ b: 2 })],
		steps: [
			{ kind: "read", path: "/one" },
			{ kind: "read", path: "/two" },
		],
	}),
	spanned("span-the-budget-is-drawn-after-the-key-is-read", {
		budget: budget(10_000, 10_000),
		keyMs: 1000,
		exchanges: [jsonReply({ a: 1 })],
		steps: [{ kind: "read", path: "/one" }],
	}),
	spanned("span-the-idle-bound-is-never-more-than-what-is-left", {
		budget: budget(10_000, 8000),
		exchanges: [jsonReply({ a: 1 })],
		steps: [
			{ kind: "advance", ms: 5000 },
			{ kind: "read", path: "/one" },
		],
	}),
	spanned("span-a-read-asked-of-a-span-that-ran-out-sends-nothing", {
		budget: budget(10_000, 10_000),
		exchanges: [jsonReply({ a: 1 })],
		steps: [
			{ kind: "advance", ms: 10_000 },
			{ kind: "read", path: "/one" },
		],
	}),
	spanned("span-a-read-ends-with-its-span", {
		budget: budget(10_000, 10_000),
		exchanges: [jsonReply({ a: 1 }, 200, [], { headMs: 3000 }), jsonReply({ b: 2 }, 200, [], { headMs: 9000 })],
		steps: [
			{ kind: "read", path: "/one" },
			{ kind: "read", path: "/two" },
		],
	}),
];

export const budgetCases: CaseSource[] = [...tables, ...exchanges, ...spans];
