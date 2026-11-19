// Row key cases.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Helpers

type Identity =
	| { kind: "curated"; key: string }
	| { kind: "hosted"; provider: string; id: string }
	| { kind: "local"; entry: string; tag: string };

const ENTRY = "3f2a1b4c-5d6e-4f70-8a9b-0c1d2e3f4a5b";

const hosted = (provider: string, id: string): Identity => ({ kind: "hosted", provider, id });

const local = (entry: string, tag: string): Identity => ({ kind: "local", entry, tag });

const parses = (name: string, values: unknown[]): CaseSource => ({ kind: "rowKey.parse", name, input: { values } });

/** `local:lab/<tag>`. */
const labTag = (tag: string): string => `local:lab/${tag}`;

////////////////////////////////
//  Cases

/** Every identity form, with separators inside ids. */
const TABLE: Identity[] = [
	{ kind: "curated", key: "openai/gpt-6-luna" },
	{ kind: "curated", key: "deepseek/deepseek-v4.1-flash" },
	hosted("openrouter", "z-ai/glm-5.3-flash"),
	hosted("openrouter", "openai/gpt-4o:extended"),
	hosted("openrouter", "~anthropic/claude-sonnet-latest"),
	hosted("openrouter", "a:b:c"),
	hosted("openrouter", "a+b/c-d.e"),
	hosted("opencode", "mimo-v2.6-flash"),
	hosted("ollama", "model"),
	hosted("lmstudio", "vendor/model"),
	hosted("llamacpp", "model"),
	hosted("compatible", "model"),
	local(ENTRY, "local-a:27b"),
	local("lab", "hf.co/user/repo:Q4_K_M"),
	local("opencode", "local-b:9b"),
	local("local", "local-b:9b"),
	local("lab", "/models/my model.gguf"),
	local("lab", "C:\\models\\a b.gguf"),
	local("lab", "a/b/c"),
	local("lab", "local:x/y"),
	local("lab", " padded "),
	local("lab", "\u{e9}\u{4e2d}\u{1f600}"),
	local("lab", "\u{a0}x"),
	local("lab", "x\u{2028}y"),
	local("x".repeat(64), "t"),
];

const VALUES: unknown[] = [
	"openai/gpt-6-luna",
	"OpenAI/GPT-6-Luna",
	"openai/gpt-6-luna ",
	"z-ai/glm-5.3-flash",
	"vendor:model",
	"ollama",
	"ollama:",
	"",
	"openrouter",
	"OpenRouter:vendor/model",
	"openrouter:vendor/model",
	"openrouter:",
	"openrouter:~",
	"openrouter:~~a",
	"openrouter:a b",
	"openrouter:a/~b",
	"openrouter:a//b",
	"openrouter:a/",
	"openrouter:/a",
	"openrouter:a?b",
	"openrouter:\u{e9}",
	"openrouter:a\n",
	"openrouter:a:b:c",
	"openrouter:a/b/c/d",
	"openrouter:_a",
	"openrouter:.a",
	"openrouter:-a",
	"openrouter:a.:+-",
	"opencode:a//b",
	"opencode:mimo-v2.6-flash",
	`opencode:${"x".repeat(200)}`,
	`opencode:${"x".repeat(201)}`,
	`opencode:${"x".repeat(100)}/${"y".repeat(99)}`,
	`opencode:${"x".repeat(100)}/${"y".repeat(100)}`,
	"a/b:c",
	"openai/gpt-6-luna:extended",
	"local",
	"local:",
	"local:lab",
	"local:lab/",
	"local:lab/  ",
	"local:/tag",
	"local:bad entry/tag",
	"local:lab/qwen:8b",
	"local::x/y",
	"local:a/b/c",
	"local:LAB/x",
	"LOCAL:lab/x",
	`local:${"x".repeat(64)}/t`,
	`local:${"x".repeat(65)}/t`,
	42,
	null,
	true,
	{},
	["openai/gpt-6-luna"],
];

/** Tags with controls and spaces. */
const TAGS: Array<[string, string]> = [
	["plain", "local-a:27b"],
	["inner-space", "my model"],
	["leading-space", " m"],
	["trailing-space", "m "],
	["blank", "   "],
	["nbsp-only", "\u{a0}"],
	["bom-only", "\u{feff}"],
	["line-separator-only", "\u{2028}"],
	["ideographic-space-only", "\u{3000}"],
	["zero-width-only", "\u{200b}"],
	["zero-width-inside", "a\u{200b}b"],
	["nbsp-inside", "a\u{a0}b"],
	["line-separator-inside", "a\u{2028}b"],
	["nul", "a\u{0}b"],
	["c0-first", "\u{1}"],
	["c0-last", "a\u{1f}b"],
	["tab", "a\tb"],
	["newline", "a\nb"],
	["carriage-return", "a\rb"],
	["escape", "\u{1b}[0m"],
	["del", "a\u{7f}b"],
	["c1-first", "a\u{80}b"],
	["c1-next-line", "a\u{85}b"],
	["c1-last", "a\u{9f}b"],
	["after-c1", "a\u{a0}b"],
	["control-at-end", "model\u{7f}"],
	["control-at-start", "\u{85}model"],
	["max-length", "t".repeat(1024)],
	["over-length", "t".repeat(1025)],
	["astral-max", "\u{1f600}".repeat(512)],
	["astral-over", "\u{1f600}".repeat(513)],
	["slashes", "///"],
	["backslashes", "\\\\x"],
	["colons", ":::"],
];

export const rowKeyCases: CaseSource[] = [
	{ kind: "rowKey.build", name: "table", input: { identities: TABLE } },
	// A key that does not read back.
	{
		kind: "rowKey.build",
		name: "unreadable-identities",
		input: {
			identities: [
				{ kind: "curated", key: "not-curated" },
				hosted("openrouter", "a b"),
				hosted("openrouter", ""),
				local("bad entry", "t"),
				local("lab", "a\u{1}b"),
				local("lab", "  "),
				local("", "t"),
			],
		},
	},
	parses("values", VALUES),
	parses(
		"local-tags",
		TAGS.map(([, tag]) => labTag(tag)),
	),
	...TAGS.map(([name, tag]): CaseSource => parses(`tag-${name}`, [labTag(tag)])),
	{
		kind: "rowKey.shown",
		name: "table",
		input: {
			identities: TABLE,
			keys: [
				"openai/gpt-6-luna",
				"openrouter:z-ai/glm-5.3-flash",
				"openrouter:a:b:c",
				"opencode:mimo-v2.6-flash",
				`local:${ENTRY}/local-a:27b`,
				"local:lab//models/my model.gguf",
				"z-ai/glm-5.3-flash",
				"vendor:model",
				"local:",
				"local:lab/a\u{1}b",
				"",
				"openrouter:a b",
			],
		},
	},
	{
		kind: "rowKey.shown",
		name: "slashless-hosted",
		input: {
			identities: [
				hosted("openrouter", "model"),
				hosted("opencode", "model"),
				hosted("openrouter", "a/b"),
				hosted("compatible", "a:b"),
			],
			keys: ["openrouter:model", "opencode:model", "openrouter:a/b"],
		},
	},
	{ kind: "rowKey.servedId", name: "table", input: { identities: TABLE } },
];
