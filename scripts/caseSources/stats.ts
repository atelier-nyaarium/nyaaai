// Generation stats cases. Every value is made up.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Interfaces & Types

type Body = Record<string, unknown>;

type Step = Record<string, unknown>;

interface Network {
	status: number;
	json?: unknown;
	headMs?: number;
	end?: "closed" | "dropped" | "refused";
}

interface Run {
	startMs?: number;
	exchanges: Network[];
	known?: Array<{ id: string; at: number }>;
	unfilled?: Array<Record<string, unknown>>;
	fails?: { filled?: string[]; fill?: string[]; unfilled?: boolean };
	holdFilled?: Array<{ id: string; ms?: number }>;
	late?: Array<{ generationId: string; ms: number; body: unknown }>;
	steps: Step[];
}

////////////////////////////////
//  Constants

const MINUTE = 60_000;

const HOUR = 60 * MINUTE;

/** Every field of a body, each with a value. */
const FULL = {
	provider_name: "Acme",
	latency: 812,
	generation_time: 430,
	total_cost: 0.004,
	is_byok: false,
	upstream_inference_cost: 0.001,
	tokens_prompt: 100,
	tokens_completion: 40,
	native_tokens_prompt: 120,
	native_tokens_completion: 50,
	finish_reason: "stop",
};

/** What each field holds, and a value of another type for each way to be wrong. */
const FIELDS: Array<[string, "text" | "number" | "flag", unknown[]]> = [
	["provider_name", "text", [7, true, [], {}]],
	["latency", "number", ["7", true, [], {}]],
	["generation_time", "number", ["7", true, [], {}]],
	["total_cost", "number", ["0.5", true, [], {}]],
	["is_byok", "flag", ["true", 1, 0, []]],
	["upstream_inference_cost", "number", ["0.5", true, [], {}]],
	["tokens_prompt", "number", ["7", true, [], {}]],
	["tokens_completion", "number", ["7", true, [], {}]],
	["native_tokens_prompt", "number", ["7", true, [], {}]],
	["native_tokens_completion", "number", ["7", true, [], {}]],
	["finish_reason", "text", [7, true, [], {}]],
];

/** A body the queue can fill from. */
const STATS = { data: { provider_name: "Acme", latency: 800, generation_time: 400, native_tokens_prompt: 3900 } };

////////////////////////////////
//  Helpers

const data = (fields: Body): Body => ({ data: fields });

const without = (key: string): Body => {
	const rest: Body = { ...FULL };
	delete rest[key];
	return data(rest);
};

const decode = (name: string, bodies: unknown[]): CaseSource => ({ kind: "stats.decode", name, input: { bodies } });

const run = (name: string, held: Run): CaseSource => ({
	kind: "stats.run",
	name,
	input: { startMs: held.startMs ?? 0, ...held },
});

/** The network's replies. */
const hit = (body: unknown = STATS): Network => ({ status: 200, json: body });
const miss = (): Network => ({ status: 404, json: { error: { message: "No such generation." } } });
const slow = (headMs: number, body: unknown = STATS): Network => ({ status: 200, json: body, headMs });

const add = (id: string, generationId = `gen-${id}`, extra: Record<string, unknown> = {}): Step => ({
	kind: "add",
	id,
	generationId,
	...extra,
});

const advance = (ms: number, during?: unknown[]): Step => ({ kind: "advance", ms, ...(during ? { during } : {}) });

////////////////////////////////
//  Decode

const decodeCases: CaseSource[] = [
	decode("every-field", [data(FULL)]),
	decode("byok-adds-provider-charges-and-a-missing-one-leaves-the-estimate-out", [
		data({ total_cost: 0.01, is_byok: true, upstream_inference_cost: 0.2 }),
		data({ total_cost: 0.01, is_byok: false, upstream_inference_cost: 0.2 }),
		data({ total_cost: 0.01, is_byok: true, upstream_inference_cost: null }),
	]),
	decode("not-a-body", [null, 7, "stats", true, [], [data(FULL)]]),
	decode("no-data", [{}, { other: FULL }, { Data: FULL }]),
	decode("data-is-not-an-object", [{ data: null }, { data: 7 }, { data: "x" }, { data: [] }, { data: [FULL] }]),
	decode("empty-data-is-empty-stats", [data({})]),
	decode("extra-keys-are-ignored", [
		{ ...data({ ...FULL, extra: { deep: 1 } }), id: "gen-a", top: [1, 2] },
		data({ unknown_field: "x", latency: 5 }),
	]),
	...FIELDS.flatMap(([key, kind, wrong]): CaseSource[] => [
		decode(`${key}-omitted`, [without(key)]),
		decode(`${key}-null`, [data({ ...FULL, [key]: null })]),
		decode(
			`${key}-mistyped`,
			wrong.map((value) => data({ ...FULL, [key]: value })),
		),
		...(kind === "number"
			? [decode(`${key}-zero-and-negative`, [data({ [key]: 0 }), data({ [key]: -1 }), data({ [key]: 1.5 })])]
			: []),
		...(kind === "text"
			? [decode(`${key}-empty-and-wide`, [data({ [key]: "" }), data({ [key]: "caf\u{e9} \u{1f600}" })])]
			: []),
	]),
	decode("cost-from-the-account-alone", [
		data({ total_cost: 0.25 }),
		data({ total_cost: 0 }),
		data({ total_cost: 0, is_byok: false }),
		data({ total_cost: 0.25, is_byok: false, upstream_inference_cost: 9 }),
	]),
	decode("cost-of-a-byok-reply", [
		data({ total_cost: 0.01, is_byok: true, upstream_inference_cost: 0.2 }),
		data({ total_cost: 0, is_byok: true, upstream_inference_cost: 0 }),
		data({ total_cost: 0.1, is_byok: true, upstream_inference_cost: 0.2 }),
		data({ is_byok: true, upstream_inference_cost: 0.2 }),
		data({ total_cost: 0.1, is_byok: true }),
		data({ total_cost: 0.1, is_byok: true, upstream_inference_cost: null }),
	]),
	decode("a-byok-sum-that-overflows-holds-no-cost", [
		data({ total_cost: 1.7e308, is_byok: true, upstream_inference_cost: 1.7e308 }),
		data({ total_cost: 1.7e308, is_byok: false, upstream_inference_cost: 1.7e308 }),
	]),
	decode("cost-that-is-not-an-amount", [
		data({ total_cost: -1 }),
		data({ total_cost: -1, is_byok: true, upstream_inference_cost: 1 }),
		data({ total_cost: 1, is_byok: true, upstream_inference_cost: -1 }),
		data({ upstream_inference_cost: 0.5 }),
	]),
	decode("native-counts-beside-plain-ones", [
		data({ native_tokens_prompt: 11, native_tokens_completion: 12 }),
		data({ tokens_prompt: 21, tokens_completion: 22 }),
		data({ tokens_prompt: 21, tokens_completion: 22, native_tokens_prompt: 11, native_tokens_completion: 12 }),
		data({ tokens_prompt: 21, tokens_completion: 22, native_tokens_prompt: null, native_tokens_completion: null }),
		data({ tokens_prompt: 21, tokens_completion: 22, native_tokens_prompt: 0, native_tokens_completion: 0 }),
		data({ tokens_prompt: 21, native_tokens_completion: 12 }),
		data({ native_tokens_prompt: 11, tokens_completion: 22 }),
		data({ tokens_prompt: 0 }),
	]),
	decode("a-native-count-of-the-wrong-type-spoils-the-body", [
		data({ tokens_prompt: 21, native_tokens_prompt: "11" }),
		data({ tokens_completion: 22, native_tokens_completion: true }),
	]),
	decode("a-field-of-the-wrong-type-spoils-the-whole-body", [
		data({ provider_name: "Acme", latency: "fast", finish_reason: "stop" }),
	]),
];

////////////////////////////////
//  Run

const MISSES = (count: number): Network[] => Array.from({ length: count }, miss);

const runCases: CaseSource[] = [
	run("reads-are-spaced-and-misses-retry", {
		exchanges: [hit(), miss(), miss(), hit(), miss(), miss(), miss()],
		steps: [add("a"), add("b"), advance(1_000), add("c"), add("x", "not-a-generation"), advance(2 * HOUR - 1_000)],
	}),
	run("a-hit-fills-the-record-once", {
		exchanges: [hit(data(FULL))],
		steps: [add("a"), advance(8 * MINUTE), advance(2 * HOUR)],
	}),
	run("a-record-that-misses-at-each-delay-is-dropped", {
		exchanges: MISSES(4),
		steps: [
			add("a"),
			advance(7 * MINUTE),
			advance(8 * MINUTE),
			advance(15 * MINUTE),
			advance(30 * MINUTE),
			advance(HOUR),
		],
	}),
	run("a-hit-after-two-misses-ends-the-retries", {
		exchanges: [miss(), miss(), hit()],
		steps: [add("a", "gen-a", { entry: "entry-1" }), advance(HOUR), advance(HOUR)],
	}),
	run("reload-fills-overdue-reads-and-skips-records-filled-elsewhere", {
		startMs: 20 * MINUTE,
		known: [{ id: "e", at: 0 }],
		exchanges: [miss(), hit()],
		steps: [add("d", "gen-d", { at: 0, entry: "entry-1" }), add("e", "gen-e", { at: 0 }), advance(40 * MINUTE)],
	}),
	run("disposing-drops-what-waits", {
		exchanges: [hit()],
		steps: [add("a"), { kind: "dispose" }, advance(10_000)],
	}),
	run("dispose-twice-and-add-after-it", {
		exchanges: [hit()],
		steps: [add("a"), { kind: "dispose" }, { kind: "dispose" }, add("b"), advance(HOUR)],
	}),
	run("ids-that-are-no-generation-id", {
		exchanges: [],
		steps: [
			"not-a-generation",
			"gen-",
			"GEN-a",
			"gen-a_b",
			"gen-a b",
			"gen-a\n",
			" gen-a",
			"gen-a.b",
			"gen-caf\u{e9}",
			"gen-\u{1f600}",
			`gen-${"a".repeat(125)}`,
			"",
			"gen",
			"gen_a",
		].map((generationId, at) => add(`r${at}`, generationId)),
	}),
	run("ids-that-are-generation-ids", {
		exchanges: [],
		steps: ["gen-a", "gen--", "gen-A-z-0-9", `gen-${"a".repeat(124)}`, "gen-1234567890abcdef"].map(
			(generationId, at) => add(`r${at}`, generationId),
		),
	}),
	run("two-records-due-at-one-instant-are-spaced-apart", {
		exchanges: [hit(), hit(), hit()],
		steps: [add("a"), add("b"), add("c"), advance(8 * MINUTE)],
	}),
	run("the-spacing-counts-from-the-end-of-a-read", {
		exchanges: [slow(1_000), hit(), hit()],
		steps: [add("a"), add("b"), add("c"), advance(8 * MINUTE)],
	}),
	run("a-record-filled-elsewhere-costs-no-spacing", {
		startMs: 8 * MINUTE,
		known: [{ id: "e", at: 0 }],
		exchanges: [hit()],
		steps: [add("e", "gen-e", { at: 0 }), add("f", "gen-f", { at: 0 }), advance(MINUTE)],
	}),
	run("an-add-during-a-read", {
		exchanges: [slow(2_000), hit(), hit()],
		steps: [
			add("a"),
			advance(HOUR, [
				{ afterMs: 7 * MINUTE + 500, do: { kind: "add", id: "b", generationId: "gen-b", entry: "entry-2" } },
			]),
		],
	}),
	run("an-add-during-a-read-that-is-due-sooner", {
		exchanges: [slow(2_000), hit()],
		steps: [
			add("a", "gen-a", { at: 0 }),
			advance(HOUR, [
				{ afterMs: 7 * MINUTE + 500, do: { kind: "add", id: "b", generationId: "gen-b", at: -2 * HOUR } },
			]),
		],
	}),
	run("a-dispose-during-a-read", {
		exchanges: [slow(2_000), hit()],
		steps: [add("a"), add("b"), advance(HOUR, [{ afterMs: 7 * MINUTE + 500, do: { kind: "dispose" } }])],
	}),
	run("a-dispose-during-a-read-that-misses", {
		exchanges: [{ status: 404, json: { error: { message: "No such generation." } }, headMs: 2_000 }, hit()],
		steps: [add("a"), advance(HOUR, [{ afterMs: 7 * MINUTE + 500, do: { kind: "dispose" } }])],
	}),
	run("an-add-moves-the-timer-earlier", {
		exchanges: [hit(), hit()],
		steps: [add("a"), add("b", "gen-b", { at: -HOUR }), advance(8 * MINUTE)],
	}),
	run("a-later-add-leaves-the-timer", {
		exchanges: [hit(), hit()],
		steps: [add("a"), advance(MINUTE), add("b"), advance(HOUR)],
	}),
	run("times-that-pass-some-delays", {
		startMs: 100 * MINUTE,
		exchanges: [],
		steps: [
			-7 * MINUTE + 1,
			-7 * MINUTE,
			-7 * MINUTE - 1,
			-15 * MINUTE + 1,
			-15 * MINUTE,
			-30 * MINUTE + 1,
			-30 * MINUTE,
			-60 * MINUTE + 1,
			-60 * MINUTE,
			-24 * HOUR,
			HOUR,
			0,
		].map((offset, index) => add(`t${index}`, `gen-t${index}`, { at: 100 * MINUTE + offset })),
	}),
	run("a-record-older-than-every-delay-reads-once", {
		startMs: 3 * HOUR,
		exchanges: [miss(), hit()],
		steps: [add("a", "gen-a", { at: 0 }), advance(HOUR), add("b", "gen-b", { at: 0 }), advance(HOUR)],
	}),
	run("a-hit-for-a-record-older-than-every-delay", {
		startMs: 3 * HOUR,
		exchanges: [hit()],
		steps: [add("a", "gen-a", { at: 0, entry: "entry-1" }), advance(MINUTE)],
	}),
	run("reads-name-their-entry", {
		exchanges: [hit(), hit(), hit()],
		steps: [
			add("a", "gen-a", { entry: "entry-1" }),
			add("b"),
			add("c", "gen-c", { entry: "entry-2" }),
			advance(8 * MINUTE),
		],
	}),
	run("every-kind-of-failed-read-counts-as-a-miss", {
		exchanges: [
			{ status: 500, json: { error: { message: "Server error." } } },
			{ status: 200, json: {}, end: "refused" },
			{ status: 401, json: { error: { message: "Bad key." } } },
			{ status: 200, json: STATS, end: "dropped" },
		],
		steps: [add("a"), advance(2 * HOUR)],
	}),
	run("a-read-that-times-out-is-a-miss", {
		exchanges: [slow(20_000), slow(10_000), hit()],
		steps: [add("a"), advance(8 * MINUTE), advance(HOUR)],
	}),
	run("a-read-that-answers-at-the-deadline", {
		exchanges: [slow(9_999), slow(10_000), slow(10_001), hit(), hit()],
		steps: [add("a"), add("b"), add("c"), advance(HOUR)],
	}),
	run("a-body-that-is-no-stats-is-a-miss", {
		exchanges: [hit({ data: null }), hit({}), hit([]), hit({ data: { latency: "7" } })],
		steps: [add("a"), advance(2 * HOUR)],
	}),
	run("a-body-of-empty-data-is-filled-empty", {
		exchanges: [hit({ data: {} })],
		steps: [add("a"), advance(8 * MINUTE)],
	}),
	run("filled-failing-reads-the-record", {
		known: [{ id: "a", at: 0 }],
		fails: { filled: ["a"] },
		exchanges: [hit()],
		steps: [add("a"), advance(8 * MINUTE)],
	}),
	run("filled-answering-true-skips-the-read", {
		known: [
			{ id: "a", at: 0 },
			{ id: "b", at: 0 },
		],
		exchanges: [],
		steps: [add("a"), add("b"), advance(8 * MINUTE), advance(HOUR)],
	}),
	run("a-record-filled-elsewhere-after-a-miss", {
		known: [{ id: "a", at: 0 }],
		exchanges: [miss()],
		steps: [add("a", "gen-a", { at: 0 }), advance(8 * MINUTE), advance(HOUR)],
	}),
	run("fill-failing-drops-the-record-and-goes-on", {
		fails: { fill: ["a"] },
		exchanges: [hit(), hit()],
		steps: [add("a"), add("b"), advance(8 * MINUTE), advance(HOUR)],
	}),
	run("the-501st-record-drops-the-oldest", {
		exchanges: [],
		steps: [{ kind: "addMany", count: 501, at: 0 }],
	}),
	run("a-full-queue-drops-the-oldest-for-each-new-record", {
		exchanges: [hit()],
		steps: [{ kind: "addMany", count: 500, at: 0 }, add("last", "gen-last", { at: 0 }), advance(7 * MINUTE)],
	}),
	run("restore-adds-recent-unfilled-records", {
		startMs: 3 * HOUR,
		known: [{ id: "u4", at: 3 * HOUR - 10 * MINUTE }],
		unfilled: [
			{ id: "u1", generationId: "gen-u1", at: 3 * HOUR - 30 * MINUTE, entry: "entry-1" },
			{ id: "u2", generationId: "gen-u2", at: 3 * HOUR - HOUR - 50 * MINUTE },
			{ id: "u3", generationId: "gen-u3", at: 3 * HOUR - 3 * HOUR },
			{ id: "u4", generationId: "gen-u4", at: 3 * HOUR - 10 * MINUTE },
			{ id: "u5", generationId: "not-a-generation", at: 3 * HOUR - 5 * MINUTE },
			{ id: "u6", generationId: "gen-u6", at: 3 * HOUR - 2 * MINUTE, entry: "entry-2" },
		],
		exchanges: [hit(), miss(), hit()],
		steps: [{ kind: "restore" }, advance(HOUR)],
	}),
	run("restore-leaves-out-the-entries-named", {
		startMs: 3 * HOUR,
		unfilled: [
			{ id: "u1", generationId: "gen-u1", at: 3 * HOUR - MINUTE, entry: "entry-1" },
			{ id: "u2", generationId: "gen-u2", at: 3 * HOUR - MINUTE, entry: "entry-2" },
			{ id: "u3", generationId: "gen-u3", at: 3 * HOUR - MINUTE },
		],
		exchanges: [hit(), hit()],
		steps: [{ kind: "restore", skip: ["entry-2"] }, advance(8 * MINUTE)],
	}),
	run("restore-when-the-sink-fails", {
		startMs: 3 * HOUR,
		fails: { unfilled: true },
		unfilled: [{ id: "u1", generationId: "gen-u1", at: 3 * HOUR - MINUTE }],
		exchanges: [hit()],
		steps: [{ kind: "restore" }, advance(HOUR)],
	}),
	run("restore-after-dispose-adds-nothing", {
		startMs: 3 * HOUR,
		unfilled: [{ id: "u1", generationId: "gen-u1", at: 3 * HOUR - MINUTE }],
		exchanges: [hit()],
		steps: [{ kind: "dispose" }, { kind: "restore" }, advance(HOUR)],
	}),

	run("a-filled-ask-that-waits-past-its-deadline-is-read-as-unfilled", {
		holdFilled: [{ id: "a", ms: 20_000 }],
		exchanges: [hit()],
		steps: [add("a"), advance(8 * MINUTE), advance(HOUR)],
	}),
	run("a-filled-ask-that-answers-in-time-is-used", {
		known: [{ id: "a", at: 0 }],
		holdFilled: [{ id: "a", ms: 9_999 }],
		exchanges: [hit()],
		steps: [add("a"), add("b"), advance(8 * MINUTE)],
	}),
	run("a-filled-ask-that-never-answers-does-not-stop-the-queue", {
		holdFilled: [{ id: "a" }],
		exchanges: [hit(), hit()],
		steps: [add("a"), add("b"), advance(8 * MINUTE), advance(HOUR)],
	}),
	run("a-dispose-with-a-filled-ask-pending", {
		holdFilled: [{ id: "a", ms: 5_000 }],
		exchanges: [hit()],
		steps: [add("a"), add("b"), advance(HOUR, [{ afterMs: 7 * MINUTE + 1_000, do: { kind: "dispose" } }])],
	}),
	run("a-dispose-with-a-filled-ask-that-never-answers", {
		holdFilled: [{ id: "a" }],
		exchanges: [hit()],
		steps: [add("a"), advance(HOUR, [{ afterMs: 7 * MINUTE + 1_000, do: { kind: "dispose" } }])],
	}),
	run("a-read-that-outlives-its-deadline-is-a-miss-and-its-answer-is-dropped", {
		late: [{ generationId: "gen-a", ms: 3_600_000, body: STATS }],
		exchanges: [hit()],
		steps: [add("a"), add("b"), advance(2 * HOUR)],
	}),
	run("a-late-answer-after-a-dispose-is-dropped", {
		late: [{ generationId: "gen-a", ms: 60_000, body: STATS }],
		exchanges: [],
		steps: [add("a"), advance(HOUR, [{ afterMs: 7 * MINUTE + 1_000, do: { kind: "dispose" } }])],
	}),
	run("restore-twice-keeps-each-record-once", {
		startMs: 3 * HOUR,
		unfilled: [
			{ id: "u1", generationId: "gen-u1", at: 3 * HOUR - 30 * MINUTE },
			{ id: "u2", generationId: "gen-u2", at: 3 * HOUR - MINUTE, entry: "entry-1" },
		],
		exchanges: [hit(), hit()],
		steps: [{ kind: "restore" }, { kind: "restore" }, advance(HOUR)],
	}),
	run("restore-with-a-record-already-waiting-keeps-the-earlier-due", {
		startMs: 3 * HOUR,
		unfilled: [
			{ id: "u1", generationId: "gen-u1", at: 3 * HOUR - 30 * MINUTE },
			{ id: "u2", generationId: "gen-u2", at: 3 * HOUR - 2 * MINUTE },
		],
		exchanges: [hit(), hit()],
		steps: [
			add("u1", "gen-u1", { at: 3 * HOUR - MINUTE }),
			add("u2", "gen-u2", { at: 3 * HOUR - 5 * MINUTE }),
			{ kind: "restore" },
			advance(HOUR),
		],
	}),
	run("an-add-of-a-record-already-waiting-keeps-one", {
		exchanges: [hit()],
		steps: [
			add("a", "gen-a", { at: 0 }),
			add("a", "gen-a", { at: 1_000 }),
			add("a", "gen-a", { at: -MINUTE }),
			advance(HOUR),
		],
	}),
	run("an-add-of-a-record-being-read-is-ignored", {
		exchanges: [slow(2_000), hit()],
		steps: [
			add("a"),
			advance(HOUR, [{ afterMs: 7 * MINUTE + 500, do: { kind: "add", id: "a", generationId: "gen-a", at: 0 } }]),
		],
	}),
	run("a-record-added-again-after-a-read-is-read-again", {
		exchanges: [hit(), hit()],
		steps: [add("a"), advance(8 * MINUTE), add("a"), advance(HOUR)],
	}),
	run("a-miss-with-the-queue-full-drops-the-oldest", {
		exchanges: [{ status: 404, json: { error: { message: "No such generation." } }, headMs: 2_000 }, hit()],
		steps: [
			add("a"),
			advance(7 * MINUTE + 3_000, [{ afterMs: 7 * MINUTE + 500, do: { kind: "addMany", count: 500 } }]),
		],
	}),
	run("a-record-restored-and-added-is-read-once", {
		startMs: 3 * HOUR,
		unfilled: [{ id: "u1", generationId: "gen-u1", at: 3 * HOUR - MINUTE }],
		exchanges: [hit(), hit()],
		steps: [{ kind: "restore" }, add("u1", "gen-u1", { at: 3 * HOUR - MINUTE }), advance(8 * MINUTE)],
	}),
];

export const statsCases: CaseSource[] = [...decodeCases, ...runCases];
