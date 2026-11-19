// Stored registry cases.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Helpers

const ENTRY = "e1";
const OTHER = "e2";
const GO = "go1";
const GLM_ID = "z-ai/glm-5.2";
const GLM = `openrouter:${GLM_ID}`;
const LUNA = "openai/gpt-6-luna";
const KIMI = "moonshotai/kimi-k3";
const SONNET = "anthropic/claude-sonnet-5.5";
const OPUS = "anthropic/claude-opus-5.5";
const SOL = "openai/gpt-6.1-sol";
const HAIKU = "anthropic/claude-haiku-5.5";
/** Over 120 characters. */
const HF_TAG =
	"hf.co/user/Coder-30B-A3B-Instruct-1M-Distill-Reasoning-Long-Context-Experimental-GGUF:Q4_K_M-imatrix-extra-long-name-here";
const HF = `local:${ENTRY}/${HF_TAG}`;

const hosted = (id: string): string => `openrouter:${id}`;

type Checks = Record<string, unknown>;

/** A passing record; `over` replaces checks, `extra` adds or replaces fields. */
function record(over: Checks = {}, schema: unknown = "pass", extra: Record<string, unknown> = {}) {
	return {
		label: "Test Model",
		date: "2026-09-28",
		cost: 0.03,
		checks: { toolPlus: "pass", recovery: "pass", reasoning: "n/a", schema, ...over },
		reasoning: true,
		cache: "none",
		verifiedPrice: { prompt: 0.32, completion: 4.4 },
		...extra,
	};
}

const raw = (records: unknown, added: unknown, hidden: unknown) => ({ records, added, hidden });

const listed = (...entries: Array<[string, string]>) => entries.map(([id, kind]) => ({ id, kind }));

const OR = listed([ENTRY, "openrouter"]);

/** One entry's registry in every case's raw form. */
const one = (records: Record<string, unknown>, added: string[], hidden: unknown[] = []) =>
	raw({ [ENTRY]: records }, { [ENTRY]: added }, hidden);

const parse = (name: string, input: unknown, entries: Array<{ id: string }> = []): CaseSource => ({
	kind: "registry.parse",
	name,
	input: { raw: input, entries },
});

const rows = (name: string, input: unknown, entries = OR): CaseSource => ({
	kind: "registry.rows",
	name,
	input: { raw: input, entries },
});

const offeredOf = (name: string, input: unknown, scope: string | null, keys: string[], entries = OR): CaseSource => ({
	kind: "registry.offered",
	name,
	input: { raw: input, entries, scope, keys },
});

const listOf = (
	name: string,
	input: unknown,
	use: string | object,
	views: Array<[string, string]>,
	scope: string | null = ENTRY,
	entries = OR,
): CaseSource => ({
	kind: "registry.list",
	name,
	input: { raw: input, entries, scope, use, views: views.map(([key, status]) => ({ key, status })) },
});

const scoped = (
	name: string,
	input: unknown,
	entries: Array<{ id: string; kind: string }>,
	scopes: Array<string | null>,
) => ({
	kind: "registry.scoped",
	name,
	input: { raw: input, entries, scopes },
});

const pick = (name: string, wanted: string, list: unknown[]): CaseSource => ({
	kind: "registry.select",
	name,
	input: { wanted, list },
});

const priced = (key: string, verifiedCompletion: number, price?: unknown) => ({
	key,
	verifiedCompletion,
	...(price === undefined ? {} : { price }),
});

const range = (completion: number) => ({ prompt: [0, 0], completion: [0, completion], tiers: [] });

////////////////////////////////
//  Test records

const BASE = record();

/** Values a stored record is judged on. */
const RECORDS: Array<[string, unknown]> = [
	["valid", BASE],
	["valid-estimated", record({}, "pass", { estimated: true })],
	["valid-estimated-false", record({}, "pass", { estimated: false })],
	["valid-explicit-cache", record({}, "pass", { cache: "explicit" })],
	["valid-implicit-cache", record({}, "pass", { cache: "implicit" })],
	["valid-all-verdicts", record({ toolPlus: "pass", recovery: "n/a", reasoning: "fail" }, "n/a")],
	["valid-failing-checks", record({ toolPlus: "fail", recovery: "fail", reasoning: "fail" }, "fail")],
	["valid-zero-prices", record({}, "pass", { cost: 0, verifiedPrice: { prompt: 0, completion: 0 } })],
	["valid-negative-zero", record({}, "pass", { cost: -0 })],
	[
		"valid-unknown-keys-dropped",
		record({}, "pass", { prediction: "pass", lanes: "x", latency: { cold: { p50: 1 } } }),
	],
	["valid-unknown-check-dropped", { ...BASE, checks: { ...BASE.checks, prediction: "pass", cache: "pass" } }],
	["valid-price-unknown-key-dropped", record({}, "pass", { verifiedPrice: { prompt: 1, completion: 2, cached: 3 } })],
	["label-trimmed", record({}, "pass", { label: "  Test Model \n" })],
	["label-trims-ecma-space", record({}, "pass", { label: "\u{feff}\u{a0}Test\u{2029}" })],
	["label-200", record({}, "pass", { label: "L".repeat(200) })],
	["label-201", record({}, "pass", { label: "L".repeat(201) })],
	["label-200-after-trim", record({}, "pass", { label: ` ${"L".repeat(200)} ` })],
	["label-100-astral", record({}, "pass", { label: "\u{1f600}".repeat(100) })],
	["label-101-astral", record({}, "pass", { label: "\u{1f600}".repeat(101) })],
	["label-200-astral", record({}, "pass", { label: "\u{1f600}".repeat(200) })],
	["label-201-astral", record({}, "pass", { label: "\u{1f600}".repeat(201) })],
	["label-empty", record({}, "pass", { label: "" })],
	["label-blank", record({}, "pass", { label: " \t " })],
	["label-zero-width-only", record({}, "pass", { label: "\u{200b}" })],
	["label-number", record({}, "pass", { label: 7 })],
	["label-null", record({}, "pass", { label: null })],
	["date-short", record({}, "pass", { date: "2026-9-28" })],
	["date-long", record({}, "pass", { date: "2026-09-28T00:00:00Z" })],
	["date-padded", record({}, "pass", { date: " 2026-09-28" })],
	["date-newline", record({}, "pass", { date: "2026-09-28\n" })],
	["date-slashes", record({}, "pass", { date: "2026/09/28" })],
	["date-impossible-kept", record({}, "pass", { date: "2026-13-45" })],
	["date-non-ascii-digits", record({}, "pass", { date: "\u{661}026-09-28" })],
	["date-number", record({}, "pass", { date: 20260928 })],
	["date-empty", record({}, "pass", { date: "" })],
	["cost-negative", record({}, "pass", { cost: -0.01 })],
	["cost-text", record({}, "pass", { cost: "0.03" })],
	["cost-null", record({}, "pass", { cost: null })],
	["cost-large", record({}, "pass", { cost: 1e21 })],
	["estimated-text", record({}, "pass", { estimated: "yes" })],
	["estimated-null", record({}, "pass", { estimated: null })],
	["estimated-number", record({}, "pass", { estimated: 1 })],
	["checks-missing", { ...BASE, checks: undefined }],
	["checks-null", { ...BASE, checks: null }],
	["checks-array", { ...BASE, checks: [] }],
	["checks-text", { ...BASE, checks: "pass" }],
	["check-tool-missing", { ...BASE, checks: { recovery: "pass", reasoning: "n/a", schema: "pass" } }],
	["check-recovery-missing", { ...BASE, checks: { toolPlus: "pass", reasoning: "n/a", schema: "pass" } }],
	["check-reasoning-missing", { ...BASE, checks: { toolPlus: "pass", recovery: "pass", schema: "pass" } }],
	["check-schema-missing", { ...BASE, checks: { toolPlus: "pass", recovery: "pass", reasoning: "n/a" } }],
	["verdict-upper", record({ toolPlus: "PASS" })],
	["verdict-boolean", record({ toolPlus: true })],
	["verdict-null", record({ recovery: null })],
	["verdict-na-spelled", record({ reasoning: "na" })],
	["verdict-schema-bad", record({}, "maybe")],
	["reasoning-missing", { ...BASE, reasoning: undefined }],
	["reasoning-text", record({}, "pass", { reasoning: "true" })],
	["cache-missing", { ...BASE, cache: undefined }],
	["cache-unknown", record({}, "pass", { cache: "automatic" })],
	["cache-null", record({}, "pass", { cache: null })],
	["price-missing", { ...BASE, verifiedPrice: undefined }],
	["price-null", { ...BASE, verifiedPrice: null }],
	["price-array", { ...BASE, verifiedPrice: [1, 2] }],
	["price-prompt-missing", record({}, "pass", { verifiedPrice: { completion: 2 } })],
	["price-completion-missing", record({}, "pass", { verifiedPrice: { prompt: 2 } })],
	["price-prompt-negative", record({}, "pass", { verifiedPrice: { prompt: -1, completion: 2 } })],
	["price-completion-text", record({}, "pass", { verifiedPrice: { prompt: 1, completion: "2" } })],
	["not-an-object-null", null],
	["not-an-object-text", "record"],
	["not-an-object-number", 3],
	["not-an-object-array", [BASE]],
	["not-an-object-empty", {}],
];

////////////////////////////////
//  Cases

const FIRST_OR_OTHER = raw(
	{
		[ENTRY]: {
			[GLM]: record(),
			[hosted("x/failed")]: record({ toolPlus: "fail" }),
			[hosted("x/unrecovered")]: record({ recovery: "fail" }),
			[hosted("x/unreasoned")]: record({ reasoning: "fail" }),
			[hosted("x/broken")]: { label: "" },
			[HF]: record({}, "pass", { label: HF_TAG }),
			"not an id": record(),
			// Curated and foreign keys.
			[LUNA]: record(),
			[`local:${OTHER}/x`]: record(),
		},
		[OTHER]: { [hosted("x/other")]: record() },
		[GLM]: { [GLM]: record() },
	},
	{
		[ENTRY]: [
			GLM,
			GLM,
			hosted("x/failed"),
			hosted("x/unrecovered"),
			hosted("x/unreasoned"),
			hosted("x/untested"),
			LUNA,
			`local:${OTHER}/x`,
			HF,
			7,
		],
		[OTHER]: [hosted("x/other")],
		[GLM]: [GLM],
	},
	["moonshotai/kimi-k*", GLM, KIMI, hosted("x/failed"), hosted("x/other"), 3],
);

const SHARED = raw(
	{
		[GO]: { [GLM]: record({}, "pass", { date: "2026-11-01" }) },
		[ENTRY]: { [GLM]: record() },
		[OTHER]: { [GLM]: record({}, "pass", { date: "2026-10-01" }) },
	},
	{ [GO]: [GLM], [ENTRY]: [GLM], [OTHER]: [GLM] },
	[],
);

const DISCOVERED = raw(
	{ [GO]: { "opencode:glm-5.3-flash": record(), "opencode:gpt-6-luna": record() } },
	{ [GO]: ["opencode:glm-5.3-flash", "opencode:gpt-6-luna"] },
	["local:box/local-a:27b", "local:gone/local-a:27b"],
);

const REMOVED = raw(
	{
		[ENTRY]: {
			[hosted("x/removed")]: record(),
			[GLM]: record({}, "fail"),
			[hosted("x/failed")]: record({ toolPlus: "fail" }),
		},
	},
	{ [ENTRY]: [GLM] },
	[],
);

const LIST = raw(
	{
		[ENTRY]: {
			[GLM]: record(),
			[hosted("x/noschema")]: record({}, "fail"),
			[hosted("x/removed")]: record(),
		},
	},
	{ [ENTRY]: [GLM, hosted("x/noschema")] },
	["moonshotai/kimi-k*", GLM],
);

const ALL_KEYS = [LUNA, KIMI, GLM, hosted("x/noschema"), hosted("x/removed")];

const STATUSES = ["ok", "retired", "noZdrHost", "noTools", "unknown"];

/** Every Chat gate verdict, with schema passing. */
const GRID = ["pass", "fail", "n/a"].flatMap((toolPlus) =>
	["pass", "fail", "n/a"].flatMap((recovery) =>
		["pass", "fail", "n/a"].map((reasoning) => ({ toolPlus, recovery, reasoning, schema: "pass" })),
	),
);

export const registryCases: CaseSource[] = [
	{ kind: "registry.testRecord", name: "records", input: { values: RECORDS.map(([, value]) => value) } },
	...RECORDS.map(
		([name, value]): CaseSource => ({
			kind: "registry.testRecord",
			name: `record-${name}`,
			input: { values: [value] },
		}),
	),

	// Parsing the three settings values.
	parse("parse-empty", raw({}, {}, [])),
	parse("parse-absent", raw(undefined, undefined, undefined)),
	parse("parse-nulls", raw(null, null, null)),
	parse("parse-wrong-types", raw([], "x", 7)),
	parse("parse-arrays-for-objects", raw([BASE], [GLM], {})),
	parse(
		"parse-entry-values-not-objects",
		raw({ [ENTRY]: [BASE], [OTHER]: "x", e3: null }, { [ENTRY]: "x", [OTHER]: 5 }, []),
	),
	parse("parse-manual-rows", FIRST_OR_OTHER),
	parse("parse-one-record", one({ [GLM]: record() }, [GLM])),
	parse("parse-record-without-added", one({ [GLM]: record() }, [])),
	parse("parse-added-without-record", one({}, [GLM])),
	parse("parse-added-duplicates", one({ [GLM]: record() }, [GLM, GLM, GLM])),
	parse("parse-added-non-text", one({ [GLM]: record() }, [GLM, 7, null, {}, [GLM]] as string[])),
	parse("parse-added-failing-record", one({ [GLM]: record({ toolPlus: "fail" }) }, [GLM])),
	parse("parse-added-schema-failing-kept", one({ [GLM]: record({}, "fail") }, [GLM])),
	parse(
		"parse-added-order",
		one({ [GLM]: record(), [hosted("a/b")]: record(), [hosted("c/d")]: record() }, [
			hosted("c/d"),
			GLM,
			hosted("a/b"),
		]),
	),
	parse(
		"parse-broken-record-dropped",
		one({ [GLM]: { label: "" }, [hosted("a/b")]: record() }, [GLM, hosted("a/b")]),
	),
	parse("parse-hosted-under-any-entry", raw({ [GO]: { [GLM]: record() }, [ENTRY]: { [GLM]: record() } }, {}, [])),
	parse(
		"parse-local-under-its-entry",
		one({ [`local:${ENTRY}/local-a:27b`]: record() }, [`local:${ENTRY}/local-a:27b`]),
	),
	parse(
		"parse-local-under-another-entry",
		one({ [`local:${OTHER}/local-a:27b`]: record() }, [`local:${OTHER}/local-a:27b`]),
	),
	parse("parse-curated-key-dropped", one({ [LUNA]: record(), [GLM]: record() }, [LUNA, GLM])),
	parse("parse-unreadable-key-dropped", one({ "vendor:x": record(), "": record(), [hosted("a b")]: record() }, [])),
	parse(
		"parse-entry-id-invalid",
		raw({ "not an id": { [GLM]: record() }, [`${"x".repeat(65)}`]: { [GLM]: record() } }, {}, []),
	),
	parse("parse-entry-id-edge", raw({ ["x".repeat(64)]: { [GLM]: record() }, "a-b_c": { [GLM]: record() } }, {}, [])),
	parse(
		"parse-entry-id-proto",
		raw(
			Object.fromEntries([
				["__proto__", { [GLM]: record() }],
				[ENTRY, { [GLM]: record() }],
			]),
			Object.fromEntries([["__proto__", [GLM]]]),
			[],
		),
	),
	parse(
		"parse-entry-id-integer-like",
		raw(
			{ b: { [GLM]: record() }, "10": { [GLM]: record() }, "2": { [GLM]: record() } },
			{ "10": [GLM], "2": [GLM], b: [GLM] },
			[],
		),
	),
	parse("parse-entry-with-no-valid-record", raw({ [ENTRY]: { [GLM]: { label: "" } } }, { [ENTRY]: [GLM] }, [])),
	parse(
		"parse-removed-entries-keep-records",
		raw({ gone: { [GLM]: record() } }, { gone: [GLM] }, []),
		listed([ENTRY, "openrouter"]),
	),
	parse(
		"parse-prediction-fields-stripped",
		one(
			{
				[GLM]: record({}, "pass", {
					checks: {
						toolPlus: "pass",
						recovery: "pass",
						reasoning: "n/a",
						schema: "pass",
						prediction: "pass",
						latency: "pass",
					},
					lanes: "a b",
					edits: { exact: 1 },
				}),
			},
			[GLM],
		),
	),
	// Hidden: families, added keys and the local keys of listed entries.
	parse("hidden-family", one({}, [], ["moonshotai/kimi-k*", "openai/gpt-*-luna"])),
	parse("hidden-unknown-dropped", one({}, [], ["moonshotai/kimi-k*", "x/y", "", "openai/gpt-6-luna"])),
	parse("hidden-added-key", one({ [GLM]: record() }, [GLM], [GLM, hosted("x/y")])),
	parse("hidden-key-of-a-removed-row-dropped", one({ [GLM]: record() }, [], [GLM])),
	parse(
		"hidden-local-of-listed-entry",
		raw({}, {}, ["local:box/local-a:27b", "local:gone/local-a:27b"]),
		listed(["box", "ollama"]),
	),
	parse("hidden-local-unlisted", raw({}, {}, ["local:box/local-a:27b"])),
	parse("hidden-local-bad-tag", raw({}, {}, ["local:box/a\u{1}b", "local:box/ "]), listed(["box", "ollama"])),
	parse("hidden-non-text", one({}, [], ["moonshotai/kimi-k*", 3, null, {}, ["moonshotai/kimi-k*"]])),
	parse(
		"hidden-duplicates",
		one({}, [], ["moonshotai/kimi-k*", "moonshotai/kimi-k*", "openai/gpt-*-luna", "moonshotai/kimi-k*"]),
	),
	parse("hidden-order", one({}, [], ["x-ai/grok-*", "anthropic/claude-haiku-*", "openai/gpt-oss-*"])),
	parse("hidden-object", raw({}, {}, { a: 1 })),
	parse(
		"parse-shared-and-discovered",
		DISCOVERED,
		listed(["box", "ollama"], [GO, "opencode"]).map(({ id }) => ({ id })),
	),

	// One entry's registry and the merge.
	scoped("scoped-entry", FIRST_OR_OTHER, listed([ENTRY, "openrouter"], [OTHER, "openrouter"]), [
		ENTRY,
		OTHER,
		null,
		"nobody",
	]),
	scoped("scoped-merge-order", SHARED, listed([GO, "opencode"], [ENTRY, "openrouter"], [OTHER, "openrouter"]), [
		GO,
		ENTRY,
		OTHER,
	]),
	scoped("scoped-merge-reversed", SHARED, listed([OTHER, "openrouter"], [ENTRY, "openrouter"], [GO, "opencode"]), [
		OTHER,
	]),
	scoped(
		"scoped-merge-added-union",
		raw(
			{
				[ENTRY]: { [GLM]: record(), [hosted("a/b")]: record() },
				[OTHER]: { [GLM]: record(), [hosted("c/d")]: record() },
			},
			{ [ENTRY]: [GLM, hosted("a/b")], [OTHER]: [hosted("c/d"), GLM] },
			["openai/gpt-*-luna", hosted("c/d"), "x/y"],
		),
		listed([ENTRY, "openrouter"], [OTHER, "openrouter"]),
		[ENTRY, OTHER],
	),
	scoped("scoped-merge-none", SHARED, [], [null]),
	scoped(
		"scoped-hidden-local",
		raw({}, {}, ["local:box/local-a:27b", "local:other/local-b:9b", "openai/gpt-*-luna"]),
		listed(["box", "ollama"], ["other", "lmstudio"]),
		["box", "other", null, "third"],
	),
	scoped("scoped-names-of-object-prototype", FIRST_OR_OTHER, listed([ENTRY, "openrouter"]), [
		"constructor",
		"toString",
		"hasOwnProperty",
		"__proto__",
	]),

	// Rows from the registry.
	rows("rows-empty", raw({}, {}, []), []),
	rows("rows-no-records", raw({}, {}, [])),
	rows("rows-removed-follow-added", REMOVED),
	rows(
		"rows-shared-hosted-first-entry-wins",
		SHARED,
		listed([GO, "opencode"], [ENTRY, "openrouter"], [OTHER, "openrouter"]),
	),
	rows("rows-shared-hosted-opencode-only", SHARED, listed([GO, "opencode"])),
	rows("rows-shared-hosted-other-order", SHARED, listed([OTHER, "openrouter"], [ENTRY, "openrouter"])),
	rows("rows-discovered-and-curated-under-kind", DISCOVERED, listed(["box", "ollama"], [GO, "opencode"])),
	rows(
		"rows-curated-under-openrouter",
		one({ [hosted("openai/gpt-6-luna")]: record(), [hosted("x/y")]: record() }, [
			hosted("openai/gpt-6-luna"),
			hosted("x/y"),
		]),
	),
	rows(
		"rows-curated-under-opencode-not-openrouter",
		raw(
			{
				[GO]: { "opencode:glm-5.3-flash": record(), "opencode:gpt-6-luna": record() },
				[ENTRY]: { "openrouter:gpt-6-luna": record() },
			},
			{ [GO]: ["opencode:glm-5.3-flash", "opencode:gpt-6-luna"], [ENTRY]: ["openrouter:gpt-6-luna"] },
			[],
		),
		listed([GO, "opencode"], [ENTRY, "openrouter"]),
	),
	rows(
		"rows-hosted-of-another-kind",
		one({ [hosted("x/y")]: record() }, [hosted("x/y")]),
		listed([ENTRY, "opencode"]),
	),
	rows(
		"rows-local",
		one(
			{
				[`local:${ENTRY}/local-a:27b`]: record({}, "pass", {
					label: "Local A",
					cache: "explicit",
					reasoning: false,
				}),
			},
			[`local:${ENTRY}/local-a:27b`],
		),
		listed([ENTRY, "ollama"]),
	),
	rows("rows-local-removed", one({ [`local:${ENTRY}/local-a:27b`]: record() }, []), listed([ENTRY, "lmstudio"])),
	rows("rows-local-hf-label", one({ [HF]: record({}, "pass", { label: HF_TAG }) }, [HF]), listed([ENTRY, "ollama"])),
	rows(
		"rows-local-of-another-entry",
		one({ [`local:${ENTRY}/local-a:27b`]: record() }, [`local:${ENTRY}/local-a:27b`]),
		listed([OTHER, "ollama"], [ENTRY, "ollama"]),
	),
	rows(
		"rows-failing-records-skipped",
		one(
			{
				[GLM]: record({ toolPlus: "fail" }),
				[hosted("a/b")]: record({ recovery: "fail" }),
				[hosted("c/d")]: record({ reasoning: "fail" }),
				[hosted("e/f")]: record({ toolPlus: "n/a" }),
			},
			[GLM, hosted("a/b")],
		),
	),
	rows(
		"rows-order-added-then-removed",
		one(
			{
				[hosted("a/b")]: record(),
				[hosted("c/d")]: record(),
				[hosted("e/f")]: record(),
				[hosted("g/h")]: record(),
			},
			[hosted("g/h"), hosted("c/d")],
		),
	),
	rows(
		"rows-schema-and-cache",
		one(
			{
				[hosted("a/b")]: record({}, "pass", { cache: "explicit" }),
				[hosted("c/d")]: record({}, "fail", { cache: "implicit" }),
				[hosted("e/f")]: record({}, "n/a", { cache: "none", reasoning: false }),
			},
			[hosted("a/b"), hosted("c/d"), hosted("e/f")],
		),
	),
	rows(
		"rows-prices-and-dates",
		one(
			{
				[hosted("a/b")]: record({}, "pass", {
					date: "2026-01-02",
					verifiedPrice: { prompt: 0.5, completion: 1.25 },
					label: "  Spaced  ",
				}),
			},
			[hosted("a/b")],
		),
	),

	// What is offered.
	offeredOf("offered", FIRST_OR_OTHER, ENTRY, [
		LUNA,
		KIMI,
		GLM,
		hosted("x/failed"),
		HF,
		hosted("x/untested"),
		"nobody",
	]),
	offeredOf("offered-no-scope", FIRST_OR_OTHER, null, [LUNA, GLM, hosted("x/failed")]),
	offeredOf(
		"offered-other-scope",
		FIRST_OR_OTHER,
		OTHER,
		[LUNA, GLM, hosted("x/other")],
		listed([ENTRY, "openrouter"], [OTHER, "openrouter"]),
	),
	offeredOf("offered-removed-row", REMOVED, ENTRY, [GLM, hosted("x/removed")]),

	// The list a pick offers.
	listOf(
		"list-chat-hides-unchecked-and-removed",
		LIST,
		"chat",
		ALL_KEYS.map((key) => [key, "ok"]),
	),
	listOf(
		"list-analysis",
		LIST,
		"analysis",
		ALL_KEYS.map((key) => [key, "ok"]),
	),
	listOf(
		"list-compaction",
		LIST,
		"compaction",
		ALL_KEYS.map((key) => [key, "ok"]),
	),
	...["chat", "analysis", "compaction"].flatMap((use) =>
		STATUSES.map((status) =>
			listOf(
				`list-${use}-all-${status}`,
				LIST,
				use,
				ALL_KEYS.map((key) => [key, status]),
			),
		),
	),
	listOf("list-order-follows-views", LIST, "chat", [
		[KIMI, "ok"],
		[LUNA, "ok"],
		[GLM, "ok"],
	]),
	listOf(
		"list-no-scope",
		LIST,
		"chat",
		ALL_KEYS.map((key) => [key, "ok"]),
		null,
	),
	listOf("list-hidden-family-only-hides-chat", LIST, "analysis", [[KIMI, "ok"]]),
	listOf("list-empty", LIST, "chat", []),
	listOf(
		"list-hidden-curated-family",
		raw({}, {}, ["openai/gpt-*-luna", "anthropic/claude-haiku-*"]),
		"chat",
		[
			[LUNA, "ok"],
			[HAIKU, "ok"],
			[OPUS, "ok"],
		],
		ENTRY,
	),
	listOf(
		"list-local-hidden-rows",
		raw({ [ENTRY]: { [`local:${ENTRY}/local-a:27b`]: record() } }, { [ENTRY]: [`local:${ENTRY}/local-a:27b`] }, [
			`local:${ENTRY}/local-a:27b`,
		]),
		"chat",
		[
			[`local:${ENTRY}/local-a:27b`, "ok"],
			[LUNA, "ok"],
		],
		ENTRY,
		listed([ENTRY, "ollama"]),
	),

	// A selection falls to the list's first cheap row.
	pick("select-wanted-listed", KIMI, [priced(LUNA, 0.55), priced(KIMI, 15)]),
	pick("select-first-listed", LUNA, [priced(LUNA, 0.55), priced(KIMI, 15)]),
	pick("select-gone-falls-to-first-cheap", hosted("x/gone"), [priced(LUNA, 0.55), priced(KIMI, 15)]),
	pick("select-gone-skips-pricey", hosted("x/gone"), [
		priced(SONNET, 11),
		priced(OPUS, 22),
		priced(SOL, 11),
		priced(KIMI, 15),
		priced(HAIKU, 0.55),
	]),
	pick("select-gone-nothing-cheap", hosted("x/gone"), [priced(SONNET, 11), priced(OPUS, 22)]),
	pick("select-empty-list", LUNA, []),
	pick("select-empty-wanted", "", [priced(LUNA, 0.55)]),
	pick("select-boundary-two", "gone", [priced(LUNA, 2)]),
	pick("select-just-over-two", "gone", [priced(LUNA, 2.0000001), priced(KIMI, 1)]),
	pick("select-catalog-price-over-verified", "gone", [priced(LUNA, 0.55, range(9)), priced(KIMI, 15)]),
	pick("select-catalog-price-under-verified", "gone", [priced(SONNET, 11, range(1.5))]),
	pick("select-catalog-price-high-end-counts", "gone", [
		priced(LUNA, 0.1, { prompt: [0, 0], completion: [0.1, 5], tiers: [] }),
	]),
	pick("select-free", "gone", [priced(SONNET, 11, "free"), priced(LUNA, 0.55)]),
	pick("select-free-over-pricey-verified", "gone", [priced(OPUS, 22, "free")]),
	pick("select-unknown-price-never-cheap", "gone", [priced(LUNA, 0, "unknown"), priced(KIMI, 15)]),
	pick("select-unknown-then-cheap", "gone", [priced(LUNA, 0, "unknown"), priced(HAIKU, 0.55)]),
	pick("select-wanted-unknown-price-still-listed", LUNA, [priced(LUNA, 0, "unknown")]),
	pick("select-zero-verified", "gone", [priced(LUNA, 0)]),

	// The gate.
	{ kind: "registry.passed", name: "gate-grid", input: { rows: GRID } },
	{
		kind: "registry.passed",
		name: "gate-schema-never-matters",
		input: {
			rows: ["pass", "fail", "n/a"].map((schema) => ({
				toolPlus: "pass",
				recovery: "pass",
				reasoning: "n/a",
				schema,
			})),
		},
	},
	{
		kind: "registry.passed",
		name: "gate-order",
		input: {
			rows: [
				{ toolPlus: "fail", recovery: "fail", reasoning: "fail", schema: "fail" },
				{ toolPlus: "n/a", recovery: "fail", reasoning: "fail", schema: "pass" },
				{ toolPlus: "pass", recovery: "fail", reasoning: "fail", schema: "pass" },
				{ toolPlus: "pass", recovery: "n/a", reasoning: "fail", schema: "pass" },
				{ toolPlus: "pass", recovery: "n/a", reasoning: "n/a", schema: "pass" },
			],
		},
	},

	// Ids and key kinds.
	{
		kind: "registry.ids",
		name: "model-ids",
		input: {
			modelIds: [
				"a/b",
				"a/b/c",
				"vendor/model-1.5:free",
				"z-ai/glm-5.2",
				"a",
				"",
				"/a",
				"a/",
				"a//b",
				"a/./b",
				"a/../b",
				".a/b",
				"-a/b",
				"a-b.c/d",
				"a/b c",
				"a/b?c",
				"a:b/c",
				"a/b:c",
				"\u{e9}/b",
				"a/b\n",
				42,
				null,
				{},
			],
			curated: [],
			tested: [],
		},
	},
	{
		kind: "registry.ids",
		name: "curated-under-kind",
		input: {
			modelIds: [],
			curated: [
				{ kind: "openrouter", id: "openai/gpt-6-luna" },
				{ kind: "openrouter", id: "gpt-6-luna" },
				{ kind: "opencode", id: "gpt-6-luna" },
				{ kind: "opencode", id: "openai/gpt-6-luna" },
				{ kind: "opencode", id: "claude-sonnet-5-5" },
				{ kind: "openrouter", id: "google/gemma-4-26b-a4b-it" },
				{ kind: "opencode", id: "google/gemma-4-26b-a4b-it" },
				{ kind: "ollama", id: "openai/gpt-6-luna" },
				{ kind: "compatible", id: "" },
			],
			tested: [],
		},
	},
	{
		kind: "registry.ids",
		name: "tested-under-entry",
		input: {
			modelIds: [],
			curated: [],
			tested: [
				{ key: "openrouter:a/b", entry: "e1" },
				{ key: "opencode:a", entry: "e1" },
				{ key: "ollama:a", entry: "" },
				{ key: "local:e1/t", entry: "e1" },
				{ key: "local:e1/t", entry: "e2" },
				{ key: "local:e1/t", entry: "" },
				{ key: "local:e1/", entry: "e1" },
				{ key: "openai/gpt-6-luna", entry: "e1" },
				{ key: "vendor:x", entry: "e1" },
				{ key: "", entry: "e1" },
			],
		},
	},
];

////////////////////////////////
//  Uses of a consumer's own

const SUMMARY = { name: "summary", requires: ["schema"], hides: true };
const TRIAGE = { name: "triage", requires: [], hides: false };

registryCases.push(
	...[
		["requires-schema-and-hides", SUMMARY],
		["requires-nothing-and-does-not-hide", TRIAGE],
		["requires-nothing-and-hides", { name: "triage", requires: [], hides: true }],
		["requires-schema-and-does-not-hide", { name: "summary", requires: ["schema"], hides: false }],
	].flatMap(([label, use]) => [
		listOf(
			`list-custom-use-${label}`,
			LIST,
			use as object,
			ALL_KEYS.map((key) => [key, "ok"]),
		),
		...STATUSES.map((status) =>
			listOf(
				`list-custom-use-${label}-all-${status}`,
				LIST,
				use as object,
				ALL_KEYS.map((key) => [key, status]),
			),
		),
	]),
	listOf("list-custom-use-is-read-by-value-not-by-name", LIST, { name: "chat", requires: ["schema"], hides: false }, [
		...ALL_KEYS.map((key): [string, string] => [key, "ok"]),
	]),
	listOf(
		"list-custom-use-named-like-a-standard-one-requiring-nothing",
		LIST,
		{ name: "analysis", requires: [], hides: false },
		[...ALL_KEYS.map((key): [string, string] => [key, "ok"])],
	),
);
