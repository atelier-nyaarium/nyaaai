// Usage shape round-trip cases.

import type { CaseSource } from "../caseAdapter";

type Text = string | object | number | boolean | null;

const usage = (shape: string, texts: Text[]): CaseSource => ({
	kind: "contract.roundTrip",
	name: `usage-${shape}`,
	input: {
		set: "usage",
		shape,
		texts: texts.map((text) => (typeof text === "string" ? text : JSON.stringify(text))),
	},
});

/** Lines as the day file's writer wrote them, key order and all. */
export const CALL_LINES: string[] = [
	'{"call":{"id":"call-1","at":1794528001000,"feature":"chat","lane":"title","model":"vendor/model-a","entry":"e1","label":"Main","app":"~/work/app/","ms":1234,"first":321,"in":1000,"out":200,"cached":300,"cacheWrite":50,"reasoning":40,"cost":0.0165,"accountCost":0.0125,"upstreamCost":0.004,"byok":true,"billingSource":"balance","provider":"Lab","gen":"gen-a1","finish":"stop"}}',
	'{"call":{"id":"call-2","at":1794528002000,"feature":"wand","model":"local-a:27b","ms":90,"cost":0,"estimated":true,"unpriced":true,"failure":"overloaded","cancelled":true}}',
	'{"call":{"id":"call-3","at":1794528003000,"feature":"analysis","lane":"summary","model":"vendor/model-b","entry":"e2","ms":15,"cost":0,"estimated":true}}',
];

/** A text cut mid-pair: the contract reads a lone surrogate as U+FFFD. */
export const SURROGATE_LINE = '{"call":{"id":"call-9","at":1,"feature":"\\ud800","model":"m\\udc00x","ms":1,"cost":0}}';

export const STATS_LINES: string[] = [
	'{"stats":{"id":"call-1","of":{"provider":"Lab","latencyMs":210,"generationMs":1100,"cost":0.0165,"accountCost":0.0125,"upstreamCost":0.004,"byok":true,"in":1000,"out":200,"finish":"stop"}}}',
	'{"stats":{"id":"call-2","of":{"provider":"Lab","latencyMs":5}}}',
];

const cost = { usd: 0.25, estimated: false };
const tag = { feature: "chat" };
const used = { input: 100, output: 20 };
const call = {
	tag,
	model: "vendor/model-a",
	at: 1794528001000,
	ms: 1200,
	cost,
};
const fullCall = {
	tag: { feature: "analysis", lane: "title", folder: "/home/user/work/app" },
	model: "vendor/model-a",
	entry: "e1",
	label: "Main",
	at: 1794528001000,
	ms: 1200,
	firstTokenMs: 300,
	usage: { ...used, cached: 10, cacheWrite: 5, reasoning: 2, cost: 0.01, billingSource: "go" },
	cost: { usd: 0, estimated: true, unpriced: true },
	provider: "Lab",
	generationId: "gen-a1",
	finishReason: "stop",
	failure: "overloaded",
	cancelled: true,
};
const record = { id: "call-9", at: 1794528001000, feature: "chat", model: "vendor/model-a", ms: 10, cost: 0 };
const fullRecord = {
	id: "call-9",
	at: 1794528001000,
	feature: "chat",
	lane: "title",
	model: "vendor/model-a",
	entry: "e1",
	label: "Main",
	app: "~/work/app/",
	ms: 10,
	first: 5,
	in: 100,
	out: 20,
	cached: 10,
	cacheWrite: 4,
	reasoning: 2,
	cost: 0.5,
	accountCost: 0.3,
	upstreamCost: 0.2,
	byok: true,
	billingSource: "balance",
	estimated: true,
	unpriced: true,
	provider: "Lab",
	gen: "gen-a1",
	finish: "stop",
	failure: "overloaded",
	cancelled: false,
};
const fullStats = {
	provider: "Lab",
	latencyMs: 210,
	generationMs: 1100,
	cost: 0.5,
	accountCost: 0.3,
	upstreamCost: 0.2,
	byok: false,
	in: 100,
	out: 20,
	finish: "stop",
};

export const usageShapeCases: CaseSource[] = [
	usage("Cost", [
		cost,
		{ usd: 0, estimated: true, unpriced: true },
		{ ...cost, extra: { deep: 1 } },
		{ ...cost, unpriced: false },
		{ ...cost, unpriced: null },
		{ usd: 1 },
		{ estimated: true },
		{ ...cost, usd: "1" },
		[],
	]),
	usage("CallTag", [
		tag,
		{ feature: "wand", lane: "summary", folder: "/home/user/app" },
		{ feature: "" },
		{ ...tag, extra: 1 },
		{ lane: "title" },
		{ feature: "chat", lane: 1 },
		{ feature: "chat", folder: null },
	]),
	usage("MeteredCall", [
		call,
		fullCall,
		{ ...call, usage: used },
		{ ...call, extra: [1] },
		{ ...call, cost: { usd: 1, estimated: true, extra: 1 } },
		{ ...call, usage: { input: 1 } },
		{ ...call, cost: undefined },
		{ ...call, tag: undefined },
		{ ...call, at: "now" },
		{ ...call, cancelled: "yes" },
		{ ...call, firstTokenMs: null },
	]),
	usage("CallRecord", [
		...CALL_LINES.map((line) => JSON.stringify(JSON.parse(line).call)),
		record,
		fullRecord,
		{ ...record, extra: { deep: 1 } },
		{ ...record, feature: "" },
		{ ...record, unpriced: false },
		{ ...record, byok: "yes" },
		{ ...record, cost: null },
		{ at: 1, feature: "chat", model: "m", ms: 1, cost: 0 },
		{ ...record, in: "1" },
	]),
	usage("GenerationStats", [
		{},
		fullStats,
		...STATS_LINES.map((line) => JSON.stringify(JSON.parse(line).stats.of)),
		{ provider: "Lab", extra: 1 },
		{ in: "1" },
		{ byok: 1 },
		{ finish: null },
		[],
	]),
	usage("CallLine", [
		{ call: record },
		{ call: fullRecord },
		{ call: record, extra: 1 },
		{ call: { ...record, extra: 1 } },
		{ stats: { id: "call-9", of: {} } },
		{ call: { ...record, feature: "" } },
		{ call: null },
		{},
	]),
	usage("StatsRecord", [
		{ id: "call-9", of: {} },
		{ id: "call-9", of: fullStats },
		{ id: "call-9", of: fullStats, extra: 1 },
		{ id: "call-9" },
		{ of: {} },
		{ id: 1, of: {} },
		{ id: "call-9", of: { in: "1" } },
	]),
	usage("StatsLine", [
		{ stats: { id: "call-9", of: {} } },
		{ stats: { id: "call-9", of: fullStats, extra: 1 }, extra: 2 },
		{ call: record },
		{ stats: { id: "call-9" } },
		{ stats: null },
		{},
	]),
	usage("UsageLine", [
		...CALL_LINES,
		...STATS_LINES,
		SURROGATE_LINE,
		// Both keys: the call, in either order.
		`{"stats":{"id":"call-9","of":{"provider":"Lab"}},"call":${JSON.stringify(record)}}`,
		`{"call":${JSON.stringify(record)},"stats":{"id":"call-9","of":{"provider":"Lab"}}}`,
		{ call: record },
		{ stats: { id: "call-9", of: {} } },
		{ call: record, stats: { id: "call-9", of: fullStats } },
		{ call: { ...record, extra: 1 }, other: 2 },
		{ stats: { id: "call-9", of: { ...fullStats, extra: 1 }, extra: 2 } },
		{ call: { ...record, feature: "" } },
		{ call: record, stats: { id: 1 } },
		{ stats: { id: "call-9" } },
		{ stats: { of: {} } },
		{ stats: { id: "call-9", of: { in: "1" } } },
		{ call: null },
		{},
		[],
		"call",
	]),
	usage("UnfilledRecord", [
		{ id: "call-1", generationId: "gen-a1", at: 1794528001000 },
		{ id: "call-1", generationId: "gen-a1", at: 1794528001000, entry: "e1" },
		{ id: "call-1", generationId: "gen-a1", at: 1794528001000, extra: 1 },
		{ id: "call-1", at: 1 },
		{ id: "call-1", generationId: "gen-a1" },
		{ id: "call-1", generationId: "gen-a1", at: 1, entry: 5 },
		[],
	]),
];
