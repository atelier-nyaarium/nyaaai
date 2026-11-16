import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Functions & Helpers

const json = (value: unknown): string => JSON.stringify(value);

const readers = (name: string, ...texts: string[]): CaseSource => ({ kind: "native.readers", name, input: { texts } });

const prompts = (name: string, ...schemas: object[]): CaseSource => ({
	kind: "native.schemaPrompt",
	name,
	input: { schemas },
});

const systems = (
	name: string,
	...rows: Array<{ system: string[]; summary?: string; prompted?: string }>
): CaseSource => ({
	kind: "native.systemTexts",
	name,
	input: { rows: rows.map((row) => ({ summary: null, prompted: null, ...row })) },
});

const argumentsFor = (name: string, ...texts: string[]): CaseSource => ({
	kind: "native.arguments",
	name,
	input: { texts },
});

const user = (text: string) => ({ role: "user", text });
const stepTurn = { role: "step", parts: [{ kind: "text", text: "a" }] };

const turns = (name: string, ...histories: object[][]): CaseSource => ({
	kind: "native.currentTurn",
	name,
	input: { histories },
});

const chunks = (name: string, ...texts: string[]): CaseSource => ({ kind: "native.chunk", name, input: { texts } });

const wires = (name: string, ...rows: Array<[status: number, body: string]>): CaseSource => ({
	kind: "native.wireError",
	name,
	input: { rows: rows.map(([status, body]) => ({ status, body })) },
});

const streams = (name: string, error: object, usage: object | null = null, serving: object = {}): CaseSource => ({
	kind: "native.streamFailure",
	name,
	input: { error, usage, serving },
});

const assembles = (name: string, ...events: object[]): CaseSource => ({
	kind: "native.assembled",
	name,
	input: { events },
});

const servings = (name: string, ...ids: unknown[]): CaseSource => ({ kind: "native.serving", name, input: { ids } });

/** A text that nests objects this deep. */
const objectsDeep = (depth: number): string => `${'{"a":'.repeat(depth)}1${"}".repeat(depth)}`;

const text = (held: string) => ({ kind: "text", text: held });
const finish = (reason: string) => ({ kind: "finish", reason });

export const nativeCases: CaseSource[] = [
	// Readers.
	readers(
		"readers-kinds",
		'"abc"',
		'""',
		"5",
		"-3.5",
		"0",
		"1e21",
		"1e-7",
		"null",
		"true",
		"false",
		"[]",
		"[1]",
		"{}",
		'{"a":1}',
		'"5"',
		"1e999",
		"-1e999",
	),
	readers("readers-text-keeps-escapes", json("line\nbreak"), json("\u{e9}\u{1f600}"), json("\u{2028}")),

	// Prompts.
	prompts("prompt-simple", { type: "object", properties: { title: { type: "string" } }, required: ["title"] }),
	prompts("prompt-key-order", { required: ["a"], type: "object", properties: { a: { type: "string" } } }),
	prompts("prompt-reordered", { b: 1, a: 2, c: { z: 1, y: 2 } }),
	prompts("prompt-integer-keys", { b: 1, "2": 2, a: 3, "1": 4, "10": 5, "-1": 6, "01": 7 }),
	prompts("prompt-empty", {}),
	prompts("prompt-numbers", { a: 1.0, b: 1e21, c: 1e-7, d: 0.1, e: -0, f: 123456789012345680000, g: 1.5e300 }),
	prompts("prompt-escapes", {
		description: 'line\nbreak "quoted" \\ back\u{1} \u{7f}',
		text: "\u{e9}\u{20ac}\u{1f600}\u{2028}\u{2029}",
		slash: "a/b",
	}),
	prompts("prompt-nested", {
		items: { type: "array", items: [{ b: 1, a: [true, false, null] }] },
		enum: ["x", 2, null],
	}),

	// System texts.
	systems("system-plain", { system: ["Rules.", "More rules."] }),
	systems("system-summary", { system: ["Rules."], summary: "Earlier." }),
	systems("system-prompted", { system: ["Rules."], prompted: "Answer in JSON." }),
	systems("system-both", { system: ["A", "B"], summary: "S", prompted: "P" }),
	systems("system-none", { system: [] }),
	systems("system-only-extras", { system: [], summary: "S", prompted: "P" }),
	systems("system-empty-strings", { system: ["", "B"], summary: "", prompted: "" }),

	// Arguments.
	argumentsFor(
		"arguments-objects",
		'{"a":1}',
		"{}",
		' {"a":1} \n',
		'{"b":1,"a":2}',
		'{"2":1,"1":2,"a":3}',
		'{"a":1,"a":2}',
		'{"a":{"b":[1,{"c":null}]}}',
	),
	argumentsFor(
		"arguments-malformed",
		"",
		"{not json",
		'{"a":',
		'{"a":1',
		'{"a":1}}',
		'{"a":1} x',
		"{'a':1}",
		"{a:1}",
		'{"a":1,}',
		"\u{feff}{}",
	),
	argumentsFor("arguments-not-objects", "[1,2]", "[]", "null", '"text"', "5", "true", "[{}]", "undefined"),
	argumentsFor(
		"arguments-lone-surrogate-escape",
		'{"a":"\\ud800"}',
		'{"a":"x\\udc00y"}',
		'{"\\ud800":1}',
		'{"a":"\\ud83d\\ude00"}',
	),
	argumentsFor("arguments-deep-nesting", objectsDeep(1100), `{"a":1}`),

	// Current turn.
	turns("turn-empty", []),
	turns("turn-user-only", [user("a")]),
	turns("turn-step-only", [stepTurn]),
	turns("turn-user-then-step", [user("a"), stepTurn]),
	turns("turn-step-then-user", [stepTurn, user("a")]),
	turns("turn-latest-user", [user("a"), stepTurn, user("b"), stepTurn, stepTurn]),
	turns("turn-many-users", [user("a"), user("b"), user("c")]),
	turns("turn-several-histories", [], [user("a")], [stepTurn, stepTurn], [user("a"), stepTurn, user("b")]),

	// Chunks.
	chunks("chunk-objects", '{"a":1}', "{}", '  {"a":1}\n', '{"b":1,"a":2}', '{"2":1,"1":2}', '{"a":{"b":[1]}}'),
	chunks("chunk-not-json", "", "{not json", '{"a":', "data: {}", "\u{feff}{}", "undefined"),
	chunks("chunk-not-objects", "[1]", "[]", "null", "5", '"x"', "true"),
	chunks("chunk-lone-surrogate-escape", '{"a":"\\ud800"}'),
	chunks("chunk-deep-nesting", objectsDeep(1100), objectsDeep(3)),

	// Wire errors.
	wires(
		"wire-error-fields",
		[502, json({ error: { type: "overloaded_error", message: "Busy." } })],
		[429, json({ error: { type: "rate_limit", message: "Slow.", resetsAt: 1794535200 } })],
		[429, json({ error: { message: "Slow.", resetsAt: "2026-11-11T10:00:00Z" } })],
		[400, json({ error: { status: "INVALID_ARGUMENT", message: "Bad." } })],
		[400, json({ error: { type: "t", status: "S", message: "Both." } })],
		[400, json({ error: { type: 5, status: "S", message: "Type is not text." } })],
		[400, json({ error: { type: 5, status: 6, message: "Neither." } })],
	),
	wires(
		"wire-error-messages",
		[503, json({ error: { message: "" } })],
		[503, json({ error: { message: "   " } })],
		[503, json({ error: { message: 5 } })],
		[503, json({ error: {} })],
		[418, json({ error: { type: "x" } })],
		[500, json({ error: { message: " kept " } })],
	),
	wires(
		"wire-error-reset-types",
		[429, json({ error: { message: "m", resetsAt: 0 } })],
		[429, json({ error: { message: "m", resetsAt: "" } })],
		[429, json({ error: { message: "m", resetsAt: true } })],
		[429, json({ error: { message: "m", resetsAt: null } })],
		[429, json({ error: { message: "m", resetsAt: [1] } })],
		[429, json({ error: { message: "m", resetsAt: 1.5 } })],
	),
	wires(
		"wire-error-body-shapes",
		[500, "null"],
		[500, "[]"],
		[500, "5"],
		[500, '"text"'],
		[500, "{}"],
		[500, json({ message: "top level" })],
		[500, json({ error: "boom" })],
		[500, json({ error: [] })],
		[500, json({ error: null })],
		[500, json({ error: [{ message: "in a list" }] })],
		[200, json({ error: { message: "ok status" } })],
	),
	wires("wire-error-ignores-other-keys", [
		500,
		json({ error: { code: 7, metadata: { a: 1 }, message: "m", extra: true } }),
	]),

	// Stream failures.
	streams("stream-failure-bare", { message: "Cut." }),
	streams("stream-failure-usage", { message: "Cut." }, { input: 5, output: 2, cost: 0.25 }),
	streams("stream-failure-generation", { message: "Cut." }, null, { generationId: "gen-1" }),
	streams("stream-failure-host", { message: "Cut." }, null, { host: "h1" }),
	streams("stream-failure-all", { message: "Cut." }, { input: 5, output: 2 }, { host: "h1", generationId: "gen-1" }),
	streams("stream-failure-empty-generation", { message: "Cut." }, null, { generationId: "" }),
	streams("stream-failure-empty-host", {}, null, { host: "" }),
	streams("stream-failure-no-message", {}),
	streams("stream-failure-zero-usage", { type: "t", code: 7 }, { input: 0, output: 0 }),

	// Whole replies.
	assembles("assembled-text", text("Hi"), finish("stop")),
	assembles(
		"assembled-pieces",
		text("a"),
		text("b"),
		{ kind: "usage", usage: { input: 1, output: 2 } },
		finish("length"),
	),
	assembles("assembled-error-finish", text("a"), finish("error")),
	assembles("assembled-no-finish", text("a")),
	assembles("assembled-empty-finish", text("a"), finish("")),
	assembles("assembled-no-events"),
	assembles("assembled-only-finish", finish("stop")),
	assembles(
		"assembled-bad-call",
		{ kind: "call", index: -1, wireId: "a", name: "r", argumentsText: "{}" },
		finish("tool_calls"),
	),
	assembles(
		"assembled-call",
		{ kind: "serving", serving: { generationId: "gen-3" } },
		{ kind: "call", index: 0, wireId: "", name: "read", argumentsText: "" },
		finish("tool_calls"),
	),

	// Serving events.
	servings("serving-ids", "gen-1", "", " ", null, 5, true, [], {}, "x"),
	servings("serving-none"),
];
