// Usage record, day file and local day cases.

import type { CaseSource } from "../caseAdapter";
import { CALL_LINES, STATS_LINES } from "./usageShapes";

////////////////////////////////
//  Helpers

type Json = Record<string, unknown>;

interface NonFinite {
	path: string;
	is: "notANumber" | "plusInfinity" | "minusInfinity";
}

interface Zone {
	fromMs: number;
	minutes: number;
}

const NOW = Date.UTC(2026, 9, 8, 12);

const MINIMAL: Json = {
	tag: { feature: "chat" },
	model: "vendor/model-a",
	at: 1794528001000,
	ms: 1200,
	cost: { usd: 0.25, estimated: false },
};

const FULL: Json = {
	tag: { feature: "analysis", lane: "title", folder: "/home/user/work/app" },
	model: "vendor/model-a",
	entry: "e1",
	label: "Main",
	at: 1794528001000,
	ms: 1200,
	firstTokenMs: 300,
	usage: {
		input: 100,
		output: 20,
		cached: 10,
		cacheWrite: 5,
		reasoning: 2,
		cost: 0.01,
		upstreamCost: 0.004,
		byok: true,
		billingSource: "balance",
	},
	cost: { usd: 0.0165, estimated: true },
	provider: "Lab",
	generationId: "gen-a1",
	finishReason: "stop",
	failure: "overloaded",
	cancelled: true,
};

const record = (
	name: string,
	call: unknown,
	options: { nonFinite?: NonFinite[]; id?: string; home?: string; nowMs?: number; offsets?: Zone[] } = {},
): CaseSource => ({
	kind: "usage.record",
	name,
	input: {
		call,
		nonFinite: options.nonFinite ?? [],
		id: options.id ?? "call-1",
		home: options.home ?? "/home/user",
		nowMs: options.nowMs ?? NOW,
		offsets: options.offsets ?? [],
	},
});

const over = (base: Json, changes: Json): Json => ({ ...base, ...changes });

const without = (base: Json, ...keys: string[]): Json => {
	const copy = { ...base };
	for (const key of keys) delete copy[key];
	return copy;
};

const stats = (name: string, value: unknown, options: { nonFinite?: NonFinite[]; id?: string } = {}): CaseSource => ({
	kind: "usage.stats",
	name,
	input: { stats: value, nonFinite: options.nonFinite ?? [], id: options.id ?? "call-1" },
});

const lines = (name: string, text: string): CaseSource => ({ kind: "usage.lines", name, input: { text } });

const app = (name: string, rows: Array<[string, string]>): CaseSource => ({
	kind: "usage.app",
	name,
	input: { rows: rows.map(([folder, home]) => ({ folder, home })) },
});

const days = (name: string, offsets: Zone[], instants: number[]): CaseSource => ({
	kind: "usage.day",
	name,
	input: { offsets, instants },
});

const MINUTE = 60_000;

/** Each side of the zone's local midnight at `instant`'s UTC day. */
const aroundMidnight = (minutes: number, utcMidnight: number): number[] => {
	const local = utcMidnight - minutes * MINUTE;
	return [local - MINUTE, local - 1, local, local + 1, local + MINUTE];
};

////////////////////////////////
//  Cases

export const usageCases: CaseSource[] = [
	// A call becomes a record.
	record("minimal-call", MINIMAL),
	record("full-call", FULL),
	record("every-field-once-more", FULL, { id: "other-id", home: "/home/user/work" }),
	record("the-id-is-given", MINIMAL, { id: "" }),
	record("a-surplus-key-is-left-out", over(MINIMAL, { extra: 1, tag: { feature: "chat", extra: 2 } })),
	record(
		"usage-fields-the-record-does-not-hold",
		over(MINIMAL, { usage: { input: 5, output: 6, listCost: 0.5, allowance: 0.25 } }),
	),
	record("usage-with-only-its-counts", over(MINIMAL, { usage: { input: 0, output: 0 } })),
	record("no-usage", without(FULL, "usage")),
	record("lane-and-folder", over(MINIMAL, { tag: { feature: "chat", lane: "", folder: "" } })),
	record("a-feature-no-list-names", over(MINIMAL, { tag: { feature: "frobnicate" } })),
	record("a-non-ascii-feature-and-model", over(MINIMAL, { tag: { feature: "\u{e9}\u{4e2d}" }, model: "m\u{1f600}" })),
	// What cost holds.
	record("priced-and-exact", over(MINIMAL, { cost: { usd: 0.0004, estimated: false } })),
	record("priced-and-estimated", over(MINIMAL, { cost: { usd: 0.0004, estimated: true } })),
	record("priced-zero", over(MINIMAL, { cost: { usd: 0, estimated: false } })),
	record("priced-zero-estimated", over(MINIMAL, { cost: { usd: 0, estimated: true } })),
	record("unpriced", over(MINIMAL, { cost: { usd: 0, estimated: false, unpriced: true } })),
	record("unpriced-and-estimated", over(MINIMAL, { cost: { usd: 0, estimated: true, unpriced: true } })),
	record("unpriced-with-a-price", over(MINIMAL, { cost: { usd: 0.5, estimated: false, unpriced: true } })),
	record("a-price-that-is-not-a-number", MINIMAL, { nonFinite: [{ path: "cost.usd", is: "notANumber" }] }),
	record("a-price-of-infinity", MINIMAL, { nonFinite: [{ path: "cost.usd", is: "plusInfinity" }] }),
	record("a-price-of-minus-infinity", MINIMAL, { nonFinite: [{ path: "cost.usd", is: "minusInfinity" }] }),
	record("negative-price", over(MINIMAL, { cost: { usd: -1, estimated: false } })),
	record("huge-price", over(MINIMAL, { cost: { usd: 1e300, estimated: false } })),
	record("tiny-price", over(MINIMAL, { cost: { usd: 5e-324, estimated: false } })),
	record(
		"reported-cost-is-kept-beside-the-total",
		over(MINIMAL, {
			usage: { input: 1_000, output: 10, cost: 0, byok: true, upstreamCost: 0.12 },
			cost: { usd: 0.12, estimated: false },
		}),
	),
	record(
		"unpriced-keeps-the-usage-numbers",
		over(MINIMAL, {
			usage: { input: 1_000, output: 10, cost: 0.5, billingSource: "go" },
			cost: { usd: 0, estimated: false, unpriced: true },
		}),
	),
	// Nothing is written for a field that is undefined, null or not a number.
	record(
		"null-fields-are-left-out",
		over(MINIMAL, {
			usage: { input: 10, output: 2, cached: null, cacheWrite: null, reasoning: null, cost: null, byok: null },
			provider: null,
			entry: null,
			label: null,
			generationId: null,
			finishReason: null,
			failure: null,
			cancelled: null,
			firstTokenMs: null,
			tag: { feature: "chat", lane: null, folder: null },
		}),
	),
	record(
		"a-field-that-is-not-a-number-is-left-out",
		over(MINIMAL, {
			usage: { input: 10, output: 2, cached: 1, cacheWrite: 1, reasoning: 1, cost: 1, upstreamCost: 1 },
			firstTokenMs: 1,
		}),
		{
			nonFinite: [
				{ path: "usage.cached", is: "notANumber" },
				{ path: "usage.cacheWrite", is: "notANumber" },
				{ path: "usage.reasoning", is: "notANumber" },
				{ path: "usage.cost", is: "notANumber" },
				{ path: "usage.upstreamCost", is: "notANumber" },
				{ path: "firstTokenMs", is: "notANumber" },
			],
		},
	),
	record("an-infinite-field-is-refused", over(MINIMAL, { usage: { input: 10, output: 2, cached: 1 } }), {
		nonFinite: [{ path: "usage.cached", is: "plusInfinity" }],
	}),
	record("an-infinite-first-token-is-refused", over(MINIMAL, { firstTokenMs: 1 }), {
		nonFinite: [{ path: "firstTokenMs", is: "minusInfinity" }],
	}),
	record(
		"false-flags-are-kept",
		over(MINIMAL, {
			usage: { input: 1, output: 1, byok: false },
			cancelled: false,
		}),
	),
	record(
		"a-wire-record",
		over(MINIMAL, {
			tag: { feature: "prediction", lane: "ghost", folder: "/home/me/projects/app" },
			usage: { input: 10, output: 2, cached: null },
			provider: null,
		}),
		{
			home: "/home/me",
			nonFinite: [{ path: "cost.usd", is: "notANumber" }],
		},
	),
	// What the shape refuses.
	record("no-feature", over(MINIMAL, { tag: {} })),
	record("empty-feature", over(MINIMAL, { tag: { feature: "" } })),
	record("a-feature-that-is-not-text", over(MINIMAL, { tag: { feature: 5 } })),
	record("no-model", without(MINIMAL, "model")),
	record("a-null-model", over(MINIMAL, { model: null })),
	record("a-model-that-is-not-text", over(MINIMAL, { model: 5 })),
	record("no-time", without(MINIMAL, "at")),
	record("a-time-that-is-not-a-number", MINIMAL, { nonFinite: [{ path: "at", is: "notANumber" }] }),
	record("an-infinite-time", MINIMAL, { nonFinite: [{ path: "at", is: "plusInfinity" }] }),
	record("no-duration", without(MINIMAL, "ms")),
	record("a-duration-that-is-not-a-number", MINIMAL, { nonFinite: [{ path: "ms", is: "notANumber" }] }),
	record("an-infinite-duration", MINIMAL, { nonFinite: [{ path: "ms", is: "plusInfinity" }] }),
	record("a-count-that-is-text", over(MINIMAL, { usage: { input: "5", output: 2 } })),
	record("no-input-count", over(MINIMAL, { usage: { output: 2 } })),
	record("a-flag-that-is-a-number", over(MINIMAL, { usage: { input: 5, output: 2, byok: 1 } })),
	record("cancelled-that-is-text", over(MINIMAL, { cancelled: "yes" })),
	record("unpriced-that-is-false", over(MINIMAL, { cost: { usd: 0, estimated: false, unpriced: false } })),
	record("the-first-refusal-by-field-order", over(MINIMAL, { tag: { feature: "" }, model: 5, at: "x" })),
	record("a-later-refusal", over(MINIMAL, { cancelled: "yes", provider: 5 })),
	record("a-label-that-is-a-number", over(MINIMAL, { label: 5, entry: 6 })),
	// The folder shown as `app`.
	record("folder-in-home", over(MINIMAL, { tag: { feature: "chat", folder: "/home/user/work/app" } })),
	record("folder-is-home", over(MINIMAL, { tag: { feature: "chat", folder: "/home/user" } })),
	record("folder-outside-home", over(MINIMAL, { tag: { feature: "chat", folder: "/srv/app" } })),
	record("folder-that-is-not-text-is-left-out", over(MINIMAL, { tag: { feature: "chat", folder: 5 } })),
	record("folder-blank-records-no-app", over(MINIMAL, { tag: { feature: "chat", folder: "   " } })),
	record(
		"folder-blank-with-a-tab-and-no-break-space",
		over(MINIMAL, { tag: { feature: "chat", folder: "\t\u{a0}\n" } }),
	),
	record("folder-with-dots", over(MINIMAL, { tag: { feature: "chat", folder: "/home/user/../other/app" } })),
	record("folder-drive-letter-case", over(MINIMAL, { tag: { feature: "chat", folder: "c:\\Users\\user\\app" } }), {
		home: "C:\\Users\\user",
	}),
	record("no-tag", without(MINIMAL, "tag")),
	record("no-cost", without(MINIMAL, "cost")),
	record("no-tag-and-no-cost", without(MINIMAL, "tag", "cost")),
	record("a-null-tag-and-a-null-cost", over(MINIMAL, { tag: null, cost: null })),
	record("a-cost-with-no-usd", over(MINIMAL, { cost: { estimated: false } })),
	record("a-cost-with-no-fields", over(MINIMAL, { cost: {} })),
	record("folder-empty", over(MINIMAL, { tag: { feature: "chat", folder: "" } })),
	record("home-empty", over(MINIMAL, { tag: { feature: "chat", folder: "/home/user/app" } }), { home: "" }),
	// The day the line is written under.
	record("day-before-midnight-utc", MINIMAL, { nowMs: Date.UTC(2026, 9, 8, 23, 59, 59, 999) }),
	record("day-at-midnight-utc", MINIMAL, { nowMs: Date.UTC(2026, 9, 9) }),
	record("day-east-of-utc", MINIMAL, {
		nowMs: Date.UTC(2026, 9, 8, 23, 30),
		offsets: [{ fromMs: 0, minutes: 60 }],
	}),
	record("day-west-of-utc", MINIMAL, {
		nowMs: Date.UTC(2026, 9, 9, 3, 30),
		offsets: [{ fromMs: 0, minutes: -300 }],
	}),
	record("day-a-wire-record-under-a-forty-five-minute-zone", MINIMAL, {
		nowMs: Date.UTC(2026, 9, 8, 18, 14, 59, 999),
		offsets: [{ fromMs: 0, minutes: 345 }],
	}),
	record("day-the-instant-is-not-the-call", over(MINIMAL, { at: Date.UTC(2026, 0, 1, 12) }), {
		nowMs: Date.UTC(2026, 9, 8, 12),
	}),
	record("day-out-of-range", MINIMAL, { nowMs: 253_402_300_799_999, offsets: [{ fromMs: 0, minutes: 60 }] }),
	// Stats.
	stats("stats-full", {
		provider: "Lab",
		latencyMs: 210,
		generationMs: 1100,
		cost: 0.0165,
		accountCost: 0.0125,
		upstreamCost: 0.004,
		byok: true,
		in: 1000,
		out: 200,
		finish: "stop",
	}),
	stats("stats-empty", {}),
	stats("stats-one-field", { provider: "Lab" }, { id: "call-2" }),
	stats("stats-a-surplus-key-is-left-out", { provider: "Lab", extra: 1, nested: { a: 1 } }),
	stats("stats-null-fields-are-left-out", {
		provider: null,
		latencyMs: null,
		generationMs: 12,
		cost: null,
		byok: null,
		finish: null,
	}),
	stats(
		"stats-a-field-that-is-not-a-number-is-left-out",
		{ latencyMs: 1, generationMs: 1, cost: 1 },
		{
			nonFinite: [
				{ path: "latencyMs", is: "notANumber" },
				{ path: "cost", is: "notANumber" },
			],
		},
	),
	stats(
		"stats-an-infinite-field-is-refused",
		{ latencyMs: 1 },
		{ nonFinite: [{ path: "latencyMs", is: "plusInfinity" }] },
	),
	stats("stats-zero-and-false", { cost: 0, byok: false, in: 0, out: 0, generationMs: 0 }),
	stats("stats-text-where-a-number-goes", { latencyMs: "12" }),
	stats("stats-a-number-where-text-goes", { provider: 5 }),
	stats("stats-a-flag-that-is-a-number", { byok: 1 }),
	stats("stats-the-first-refusal-by-field-order", { finish: 5, provider: 5 }),
	stats("stats-an-empty-id", { provider: "Lab" }, { id: "" }),
	stats("stats-a-non-ascii-id", { provider: "L\u{e4}b" }, { id: "\u{4e2d}" }),
	// A day file's text.
	lines("empty-file", ""),
	lines("one-call-line", `${CALL_LINES[0]}\n`),
	lines("a-call-line-without-its-line-feed", CALL_LINES[0] as string),
	lines("every-sample-line", `${[...CALL_LINES, ...STATS_LINES].join("\n")}\n`),
	lines("a-stats-line", `${STATS_LINES[0]}\n${STATS_LINES[1]}\n`),
	lines("blank-lines-are-not-counted", `\n   \n${CALL_LINES[0]}\n\t\n\n${STATS_LINES[1]}\n  \n`),
	lines("a-line-that-is-not-json-is-dropped", `{not json\n${CALL_LINES[1]}\n`),
	lines("a-torn-last-line-is-dropped", `${CALL_LINES[0]}\n${(CALL_LINES[1] as string).slice(0, 40)}`),
	lines("json-that-fits-no-line-is-dropped", `[]\n5\nnull\n"call"\ntrue\n{}\n{"other":1}\n${CALL_LINES[0]}\n`),
	lines(
		"a-call-that-does-not-fit-is-dropped",
		[
			'{"call":{"id":"x","at":1,"feature":"","model":"m","ms":1,"cost":0}}',
			'{"call":{"id":"x","at":"1","feature":"chat","model":"m","ms":1,"cost":0}}',
			'{"call":{"id":"x","at":1,"feature":"chat","model":"m","ms":1}}',
			'{"call":{"id":"x","at":1,"feature":"chat","model":"m","ms":1,"cost":0,"unpriced":false}}',
			'{"call":{"id":"x","at":1,"feature":"chat","model":"m","ms":1,"cost":null}}',
			'{"call":{"id":"ok","at":1,"feature":"chat","model":"m","ms":1,"cost":0}}',
			"",
		].join("\n"),
	),
	lines(
		"a-stats-line-that-does-not-fit-is-dropped",
		[
			'{"stats":{"id":5,"of":{}}}',
			'{"stats":{"id":"x"}}',
			'{"stats":{"id":"x","of":{"latencyMs":"1"}}}',
			'{"stats":{"id":"x","of":5}}',
			'{"stats":{"id":"ok","of":{}}}',
			"",
		].join("\n"),
	),
	lines(
		"an-unknown-key-is-dropped",
		[
			'{"call":{"id":"x","at":1,"feature":"chat","model":"m","ms":1,"cost":0,"mystery":true,"extra":{"a":1}},"outer":1}',
			'{"stats":{"id":"x","of":{"provider":"Lab","mystery":1},"extra":2}}',
			"",
		].join("\n"),
	),
	lines(
		"both-keys-read-as-a-call",
		[
			'{"call":{"id":"c","at":1,"feature":"chat","model":"m","ms":1,"cost":0},"stats":{"id":"c","of":{"provider":"Lab"}}}',
			'{"stats":{"id":"c","of":{"provider":"Lab"}},"call":{"id":"c","at":1,"feature":"chat","model":"m","ms":1,"cost":0}}',
			'{"call":5,"stats":{"id":"c","of":{"provider":"Lab"}}}',
			"",
		].join("\n"),
	),
	lines("crlf-line-ends", `${CALL_LINES[0]}\r\n${STATS_LINES[0]}\r\n`),
	lines("a-line-padded-with-spaces", `  ${CALL_LINES[0]}  \n\t${STATS_LINES[1]}\t\n`),
	lines("a-line-padded-with-non-json-space", `\u{a0}${CALL_LINES[0]}\n${CALL_LINES[1]}\u{a0}\n\u{a0}\n`),
	lines("only-a-line-feed-splits", `${CALL_LINES[0]}\r${CALL_LINES[1]}\n`),
	lines(
		"a-line-separator-is-not-a-line-end",
		`{"call":{"id":"a\u{2028}b","at":1,"feature":"chat","model":"m","ms":1,"cost":0}}\n`,
	),
	lines("a-byte-order-mark-spoils-its-line", `\u{feff}${CALL_LINES[0]}\n${CALL_LINES[1]}\n`),
	lines(
		"escaped-text",
		'{"call":{"id":"a\\"b\\\\c\\n\\u00e9\\ud83d\\ude00","at":1,"feature":"chat","model":"m","ms":1,"cost":0}}\n',
	),
	lines(
		"a-lone-surrogate-reads-as-the-replacement-character",
		'{"call":{"id":"c","at":1,"feature":"\\ud800","model":"m\\udc00x","ms":1,"cost":0}}\n',
	),
	lines(
		"numbers-in-their-spellings",
		[
			'{"call":{"id":"n","at":1.5e12,"feature":"chat","model":"m","ms":1E3,"cost":0.0}}',
			'{"call":{"id":"n","at":01,"feature":"chat","model":"m","ms":1,"cost":0}}',
			'{"call":{"id":"n","at":1e999,"feature":"chat","model":"m","ms":1,"cost":0}}',
			'{"call":{"id":"n","at":-0,"feature":"chat","model":"m","ms":1,"cost":0}}',
			"",
		].join("\n"),
	),
	lines("repeated-ids-are-all-kept", `${CALL_LINES[0]}\n${CALL_LINES[0]}\n${STATS_LINES[0]}\n${STATS_LINES[0]}\n`),
	lines("a-deep-line-is-dropped", `${'{"call":'.repeat(1100)}1${"}".repeat(1100)}\n${CALL_LINES[0]}\n`),
	lines("only-blank-text", "\n\n  \n\t\n"),
	// The folder with home as `~`.
	app("posix", [
		["/home/me/projects/app", "/home/me"],
		["/home/me", "/home/me"],
		["/srv/app/", "/home/me"],
		["/home/meadow/app", "/home/me"],
		["/home/me/", "/home/me/"],
		["/home/me//app//", "/home/me//"],
		["/home/me/app", ""],
		["/home/me/app", "/"],
		["/", "/home/me"],
		["", "/home/me"],
		["", ""],
		["/home/Me/app", "/home/me"],
	]),
	app("windows", [
		["C:\\Users\\me\\projects\\app", "C:\\Users\\me"],
		["C:\\Users\\me", "C:\\Users\\me"],
		["C:\\Users\\me\\", "C:\\Users\\me\\"],
		["C:\\Users\\meadow\\app", "C:\\Users\\me"],
		["D:\\work\\app\\", "C:\\Users\\me"],
		["c:\\users\\me\\app", "C:\\Users\\me"],
		["C:\\Users\\me\\app\\\\", "C:\\Users\\me"],
	]),
	app("mixed-separators", [
		["C:/Users/me/projects/app", "C:\\Users\\me"],
		["C:/Users/me/app", "C:/Users/me"],
		["C:\\Users\\me/app", "C:\\Users\\me"],
		["C:\\Users\\me\\app/", "C:\\Users\\me"],
		["/home/me/a\\b", "/home/me"],
		["\\home\\me\\app", "/home/me"],
		["\\\\server\\share\\app", "\\\\server\\share"],
		["//server/share/app", "//server/share"],
	]),
	app("blank-folders", [
		["", "/home/user"],
		["", ""],
		[" ", "/home/user"],
		["   ", "C:\\Users\\user"],
		["\t", "/home/user"],
		["\u{a0}\u{2003}", "/"],
		["/", "/home/user"],
		[" /", "/home/user"],
	]),
	app("dot-segments", [
		["/home/user/../other", "/home/user"],
		["/home/user/app/..", "/home/user"],
		["/home/user/app/../..", "/home/user"],
		["/home/user/./app", "/home/user"],
		["/home/user/app/.", "/home/user"],
		["/home/user/../user/app", "/home/user"],
		["/home/user/app/../../other/app", "/home/user"],
		["/home/user/..", "/home/user"],
		["/home/./user/app", "/home/user"],
		["/../home/user/app", "/home/user"],
		["/../../..", "/home/user"],
		["/home/user/app", "/home/./user/../user"],
		["/home/user/app", "/home/user/.."],
		["/a/b/c/../../..", "/"],
		["/a/b/../../../..", "/"],
		["/home/user/app/...", "/home/user"],
		["/home/user/.hidden", "/home/user"],
		["/home/user/..hidden", "/home/user"],
		["/home/user/app../x", "/home/user"],
		["/home/user/app/../", "/home/user"],
		["/home/user/app//..", "/home/user"],
		["/home/user//..", "/home/user"],
		["./app", "/home/user"],
		["../app", "/home/user"],
		[".", "/home/user"],
		["..", "/home/user"],
	]),
	app("dot-segments-windows", [
		["C:\\Users\\user\\..\\other", "C:\\Users\\user"],
		["C:\\Users\\user\\app\\..", "C:\\Users\\user"],
		["C:\\Users\\user\\.\\app", "C:\\Users\\user"],
		["C:\\..\\Users\\user\\app", "C:\\Users\\user"],
		["C:\\..\\..", "C:\\Users\\user"],
		["C:\\Users\\..\\..\\..", "C:\\"],
		["C:\\Users\\user\\app", "C:\\Users\\user\\.."],
		["C:\\Users/user/../app", "C:\\Users\\user"],
		["\\\\server\\share\\..\\x", "\\\\server\\share"],
	]),
	app("drive-letters", [
		["c:\\Users\\user\\app", "C:\\Users\\user"],
		["C:\\Users\\user\\app", "c:\\Users\\user"],
		["c:\\users\\USER\\app", "C:\\Users\\user"],
		["C:\\USERS\\user", "c:\\users\\USER"],
		["d:\\Users\\user\\app", "C:\\Users\\user"],
		["c:\\Users\\username\\app", "C:\\Users\\user"],
		["c:\\Users\\user", "C:\\Users\\user\\"],
		["c:\\", "C:\\"],
		["c:", "C:"],
		["C:/Users/user/app", "c:/Users/user"],
		["c:/Users/user/app", "C:\\Users\\user"],
		["c:\\Users\\user\\app", "/home/user"],
		["/home/user/app", "C:\\Users\\user"],
		["/Home/User/app", "/home/user"],
		["1:\\Users\\user\\app", "1:\\users\\user"],
		["\u{e9}:\\Users\\user\\app", "\u{c9}:\\Users\\user"],
		["c:\\Users\\\u{c9}\\app", "C:\\Users\\\u{e9}"],
	]),
	app("doubled-separators-on-windows", [
		["C:\\Users\\\\user", "C:\\Users\\user"],
		["C:\\Users\\user\\\\app", "C:\\Users\\user"],
		["C:\\Users\\user\\\\", "C:\\Users\\user"],
		["C:\\\\Users\\user", "C:\\Users\\user"],
		["C:\\Users\\\\user", "C:\\Users\\\\user"],
		["C:\\Users\\\\\\user\\app", "C:\\Users\\user\\"],
	]),
	app("trailing-separators", [
		["/a/b///", "/a"],
		["/a/b\\/\\", "/a"],
		["\\a\\b\\\\", "\\a"],
		["///", "/"],
		["\\\\\\", "\\"],
		["/home/me///", "/home/me///"],
	]),
	app("non-ascii", [
		["/home/m\u{e9}/\u{4e2d}\u{6587}", "/home/m\u{e9}"],
		["/home/\u{1f600}/app", "/home/\u{1f600}"],
		["/home/\u{1f600}x/app", "/home/\u{1f600}"],
	]),
	// The local day under a scripted zone.
	days(
		"utc-midnight",
		[],
		[
			Date.UTC(2026, 9, 8, 23, 59, 59, 999),
			Date.UTC(2026, 9, 9),
			Date.UTC(2026, 9, 9, 0, 0, 0, 1),
			Date.UTC(2026, 11, 31, 23, 59, 59, 999),
			Date.UTC(2027, 0, 1),
			0,
			-1,
		],
	),
	...[
		["positive", 60],
		["negative", -300],
		["half-hour", 330],
		["forty-five-minute", 345],
		["forty-five-minute-east", 765],
		["negative-half-hour", -570],
		["far-east", 840],
		["far-west", -720],
	].map(([name, minutes]) =>
		days(
			`midnight-${name}`,
			[{ fromMs: 0, minutes: minutes as number }],
			[
				...aroundMidnight(minutes as number, Date.UTC(2026, 9, 9)),
				...aroundMidnight(minutes as number, Date.UTC(2026, 11, 31)),
				...aroundMidnight(minutes as number, Date.UTC(2028, 1, 29)),
				...aroundMidnight(minutes as number, Date.UTC(2027, 2, 1)),
			],
		),
	),
	days(
		"leap-days",
		[{ fromMs: 0, minutes: 0 }],
		[
			Date.UTC(2028, 1, 28, 23, 59, 59, 999),
			Date.UTC(2028, 1, 29),
			Date.UTC(2028, 1, 29, 23, 59, 59, 999),
			Date.UTC(2028, 2, 1),
			Date.UTC(2027, 1, 28, 23, 59, 59, 999),
			Date.UTC(2027, 2, 1),
			Date.UTC(2100, 1, 28, 23, 59, 59, 999),
			Date.UTC(2100, 2, 1),
		],
	),
	// A daylight change: fall back in New York, 2026-11-01 06:00 UTC.
	days(
		"daylight-fall-back",
		[
			{ fromMs: 0, minutes: -240 },
			{ fromMs: Date.UTC(2026, 10, 1, 6), minutes: -300 },
		],
		[
			Date.UTC(2026, 10, 1, 3, 59, 59, 999),
			Date.UTC(2026, 10, 1, 4),
			Date.UTC(2026, 10, 1, 5, 59, 59, 999),
			Date.UTC(2026, 10, 1, 6),
			Date.UTC(2026, 10, 1, 6, 0, 0, 1),
			Date.UTC(2026, 10, 2, 4, 59, 59, 999),
			Date.UTC(2026, 10, 2, 5),
			Date.UTC(2026, 10, 2, 5, 0, 0, 1),
		],
	),
	// Spring forward, 2026-03-08 07:00 UTC.
	days(
		"daylight-spring-forward",
		[
			{ fromMs: 0, minutes: -300 },
			{ fromMs: Date.UTC(2026, 2, 8, 7), minutes: -240 },
		],
		[
			Date.UTC(2026, 2, 8, 4, 59, 59, 999),
			Date.UTC(2026, 2, 8, 5),
			Date.UTC(2026, 2, 8, 6, 59, 59, 999),
			Date.UTC(2026, 2, 8, 7),
			Date.UTC(2026, 2, 9, 3, 59, 59, 999),
			Date.UTC(2026, 2, 9, 4),
		],
	),
	// A change at local midnight: the clock skips the hour after midnight (São Paulo, 2018-11-04).
	days(
		"daylight-change-skips-midnight",
		[
			{ fromMs: 0, minutes: -180 },
			{ fromMs: Date.UTC(2018, 10, 4, 3), minutes: -120 },
		],
		[
			Date.UTC(2018, 10, 4, 2, 59, 59, 999),
			Date.UTC(2018, 10, 4, 3),
			Date.UTC(2018, 10, 4, 3, 0, 0, 1),
			Date.UTC(2018, 10, 4, 22, 59, 59, 999),
			Date.UTC(2018, 10, 4, 23),
		],
	),
	// A change at local midnight: the last hour of a day happens twice (São Paulo, 2019-02-17).
	days(
		"daylight-change-repeats-the-last-hour",
		[
			{ fromMs: 0, minutes: -120 },
			{ fromMs: Date.UTC(2019, 1, 17, 2), minutes: -180 },
		],
		[
			Date.UTC(2019, 1, 17, 1, 59, 59, 999),
			Date.UTC(2019, 1, 17, 2),
			Date.UTC(2019, 1, 17, 2, 59, 59, 999),
			Date.UTC(2019, 1, 17, 3),
			Date.UTC(2019, 1, 17, 3, 0, 0, 1),
		],
	),
	// A zone changes its rule: the offset is read at the instant, not at the call.
	days(
		"zone-changes-its-offset",
		[
			{ fromMs: Date.UTC(2020, 0, 1), minutes: 60 },
			{ fromMs: Date.UTC(2022, 0, 1), minutes: -60 },
		],
		[
			Date.UTC(2019, 11, 31, 22, 59, 59, 999),
			Date.UTC(2019, 11, 31, 23),
			Date.UTC(2021, 11, 31, 22, 59, 59, 999),
			Date.UTC(2021, 11, 31, 23),
			Date.UTC(2022, 0, 1, 0, 59, 59, 999),
			Date.UTC(2022, 0, 1, 1),
		],
	),
	// Years 0000 to 9999 only.
	days(
		"out-of-range",
		[],
		[
			-62_167_219_200_000, -62_167_219_200_001, -62_135_596_800_000, 253_402_300_799_999, 253_402_300_800_000,
			8_640_000_000_000_000, -8_640_000_000_000_000,
		],
	),
	days(
		"out-of-range-by-the-offset",
		[{ fromMs: 0, minutes: 60 }],
		[253_402_300_799_999, 253_402_297_199_999, -62_167_219_200_000],
	),
	days(
		"out-of-range-by-a-negative-offset",
		[{ fromMs: 0, minutes: -60 }],
		[-62_167_219_200_000, -62_167_215_600_000, -62_167_215_599_999],
	),
];
