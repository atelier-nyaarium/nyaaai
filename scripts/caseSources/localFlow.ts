// Local kind server and flow cases on a scripted network.

import type { CaseSource } from "../caseAdapter";
import { failing, NOW, refused, reply } from "./catalogKit";

////////////////////////////////
//  Fixtures

const KEY = "key-0123456789abcdef";

const A = "local-a:27b";
const B = "local-b:9b";
const C = "local-c:3b";

const EMBED = "embed-a:latest";

const TAGS = reply({
	models: [
		{ name: A },
		{ name: B },
		{ name: C },
		{ name: EMBED },
		{ name: "far:1b-cloud", remote_host: "https://remote.example:443" },
	],
});

const SHOW_A = reply({
	parameters: "num_ctx                        32768\ntemperature                    0.6",
	capabilities: ["completion", "tools", "thinking"],
	thinking: { values: [false, true], default: true },
	model_info: { "general.architecture": "arch-a", "arch-a.context_length": 262_144 },
});

/** Tools, no baked context, a trained length. */
const SHOW_B = reply({
	capabilities: ["completion", "tools"],
	model_info: { "general.architecture": "arch-b", "arch-b.context_length": 131_072 },
});

/** No tools. */
const SHOW_C = reply({ capabilities: ["completion"] });

const SHOW_EMBED = reply({ capabilities: ["embedding"] });

const SHOWS = [SHOW_A, SHOW_B, SHOW_C, SHOW_EMBED];

const CHAT = reply({
	id: "c1",
	choices: [{ message: { content: "Hi" }, finish_reason: "stop" }],
	usage: { prompt_tokens: 5, completion_tokens: 1 },
});

const NOT_FOUND = {
	status: 404,
	json: { error: { type: "not_found_error", message: `model "${A}" not found` } },
};

const LM_LIST = (b: Record<string, unknown> = {}) =>
	reply({
		models: [
			{
				key: "local-a",
				type: "llm",
				max_context_length: 262_144,
				loaded_instances: [{ id: "local-a", config: { context_length: 8192 } }],
				capabilities: { reasoning: { allowed_options: ["off", "on"], default: "on" } },
			},
			{ key: "local-b", type: "llm", max_context_length: 4096, loaded_instances: [], ...b },
			{ key: "embed-a", type: "embedding", loaded_instances: [] },
			{ type: "llm" },
		],
	});

const LLAMA_LIST = reply({
	object: "list",
	data: [{ id: "local-a.gguf", meta: { n_ctx_train: 131_072 } }, { id: "local-b.gguf", meta: null }, {}],
});

const PROPS_A = reply({
	default_generation_settings: { n_ctx: 262_144 },
	chat_template_caps: { reasoning_effort_levels: ["low", "high", "bogus"] },
});

const PROPS_B = reply({ default_generation_settings: {}, chat_template_caps: {} });

const SLEEPING = { status: 400, json: { error: { message: "model is not loaded" } } };

const COMPAT_LIST = reply({ data: [{ id: "local-a", context_length: 32_768 }, { id: "local-b" }, {}] });

const REMOTE_URL = "https://host.lab.example/api/v2";

////////////////////////////////
//  Functions & Helpers

const server = (name: string, kind: string, input: Record<string, unknown>): CaseSource => ({
	kind: "local.server",
	name,
	input: { kind, entry: { label: "Lab" }, stored: null, startMs: NOW, exchanges: [], ...input },
});

const flow = (name: string, kind: string, input: Record<string, unknown>): CaseSource => ({
	kind: "local.flow",
	name,
	input: { kind, entry: { id: "lab", label: "Lab" }, stored: null, startMs: NOW, exchanges: [], rows: [], ...input },
});

const models = (...known: unknown[]) => ({ kind: "models", known });

const views = (extra: Record<string, unknown> = {}) => ({ kind: "views", ...extra });

const chat = (model: string, extra: Record<string, unknown> = {}) => ({ kind: "chat", model, ...extra });

const known = (tag: string, extra: Record<string, unknown> = {}) => ({
	tag,
	tools: true,
	thinking: false,
	efforts: [],
	...extra,
});

const row = (tag: string, entry = "lab") => `local:${entry}/${tag}`;

////////////////////////////////
//  Cases

export const localFlowCases: CaseSource[] = [
	// Ollama's server.
	server("ollama-lists-completion-models-with-efforts-and-context", "ollama", {
		exchanges: [TAGS, ...SHOWS],
		steps: [models()],
	}),
	server("ollama-shows-go-out-in-tag-order-and-skip-remote-models", "ollama", {
		exchanges: [TAGS, ...SHOWS],
		steps: [models(known(C))],
	}),
	server("ollama-a-failed-show-keeps-the-prior-model", "ollama", {
		exchanges: [TAGS, { ...SHOW_A, end: "refused" }, SHOW_B, SHOW_C, SHOW_EMBED],
		steps: [models(known(A, { context: 4096 }))],
	}),
	server("ollama-a-failed-show-with-no-prior-drops-the-model", "ollama", {
		exchanges: [TAGS, { ...SHOW_A, end: "refused" }, SHOW_B, SHOW_C, SHOW_EMBED],
		steps: [models()],
	}),
	server("ollama-a-show-that-answers-an-error-keeps-the-prior-model", "ollama", {
		exchanges: [TAGS, failing(500), SHOW_B, SHOW_C, SHOW_EMBED],
		steps: [models(known(A, { thinking: true, efforts: ["low"] }))],
	}),
	server("ollama-a-show-that-is-no-chat-model-drops-a-known-one", "ollama", {
		exchanges: [TAGS, SHOW_EMBED, SHOW_B, SHOW_C, SHOW_EMBED],
		steps: [models(known(A, { context: 4096 }))],
	}),
	server("ollama-a-show-that-is-no-object-drops-a-known-one", "ollama", {
		exchanges: [TAGS, reply("text"), SHOW_B, SHOW_C, SHOW_EMBED],
		steps: [models(known(A))],
	}),
	server("ollama-a-prior-model-not-in-the-tags-is-gone", "ollama", {
		exchanges: [reply({ models: [{ name: B }] }), SHOW_B],
		steps: [models(known(A), known(C))],
	}),
	server("ollama-tags-that-do-not-read-throw", "ollama", {
		exchanges: [reply({ data: [] })],
		steps: [models(known(A))],
	}),
	server("ollama-tags-that-fail-throw", "ollama", {
		exchanges: [failing(500)],
		steps: [models()],
	}),
	server("ollama-tags-that-are-refused-throw", "ollama", {
		exchanges: [refused()],
		steps: [models()],
	}),
	server("ollama-no-tags-no-shows", "ollama", {
		exchanges: [reply({ models: [] })],
		steps: [models(known(A))],
	}),
	server("ollama-loaded-reads-the-running-list", "ollama", {
		exchanges: [reply({ models: [{ name: A, context_length: 65_536 }, { name: B }] })],
		steps: [{ kind: "loaded", tag: A }],
	}),
	server("ollama-loaded-that-does-not-read-is-empty", "ollama", {
		exchanges: [reply({ data: [] })],
		steps: [{ kind: "loaded", tag: A }],
	}),
	server("ollama-loaded-that-fails-throws", "ollama", {
		exchanges: [failing(500)],
		steps: [{ kind: "loaded", tag: A }],
	}),
	server("ollama-has-no-loader", "ollama", {
		exchanges: [],
		steps: [{ kind: "warm", tag: A }],
	}),
	server("ollama-never-sends-the-key", "ollama", {
		stored: KEY,
		exchanges: [TAGS, ...SHOWS, reply({ models: [] })],
		steps: [models(), { kind: "loaded", tag: A }],
	}),
	server("ollama-a-url-of-the-entry-is-the-base", "ollama", {
		entry: { label: "Lab", url: "http://lab.local:11434/" },
		exchanges: [reply({ models: [{ name: A }] }), SHOW_A],
		steps: [models()],
	}),
	server("ollama-a-caller-aborted-before-asks-nothing", "ollama", {
		exchanges: [TAGS, ...SHOWS],
		steps: [{ ...models(), abortedBefore: true }],
	}),
	server("ollama-an-abort-during-the-tags-throws", "ollama", {
		exchanges: [{ ...TAGS, headMs: 100 }, ...SHOWS],
		steps: [{ ...models(), abortAfterMs: 10 }],
	}),

	// LM Studio's server.
	server("lmstudio-models-read-the-list-once", "lmstudio", {
		exchanges: [LM_LIST()],
		steps: [models()],
	}),
	server("lmstudio-loaded-reads-the-list-again", "lmstudio", {
		exchanges: [LM_LIST(), LM_LIST({ loaded_instances: [{ config: { context_length: 16_384 } }] })],
		steps: [models(), { kind: "loaded", tag: "local-b" }],
	}),
	server("lmstudio-a-list-that-does-not-read-throws", "lmstudio", {
		exchanges: [reply({ data: [] })],
		steps: [models()],
	}),
	server("lmstudio-a-list-that-does-not-read-loads-nothing", "lmstudio", {
		exchanges: [reply({ data: [] })],
		steps: [{ kind: "loaded", tag: "local-a" }],
	}),
	server("lmstudio-a-list-that-fails-throws", "lmstudio", {
		exchanges: [failing(503), failing(503)],
		steps: [models(), { kind: "loaded", tag: "local-a" }],
	}),
	server("lmstudio-ignores-the-known-list", "lmstudio", {
		exchanges: [LM_LIST()],
		steps: [models(known("local-a", { efforts: ["high"], thinking: true }), known("gone"))],
	}),
	server("lmstudio-never-sends-the-key", "lmstudio", {
		stored: KEY,
		exchanges: [LM_LIST()],
		steps: [models()],
	}),
	server("lmstudio-has-no-loader-for-a-warm", "lmstudio", {
		exchanges: [],
		steps: [{ kind: "warm", tag: "local-a" }],
	}),

	// llama.cpp's server.
	server("llamacpp-models-read-props-for-each-model-at-once", "llamacpp", {
		exchanges: [LLAMA_LIST, PROPS_A, PROPS_B],
		steps: [models()],
	}),
	server("llamacpp-a-sleeping-model-carries-the-known-facts", "llamacpp", {
		exchanges: [LLAMA_LIST, SLEEPING, PROPS_B],
		steps: [models(known("local-a.gguf", { thinking: true, efforts: ["low", "high"], context: 8192 }))],
	}),
	server("llamacpp-a-sleeping-model-with-no-known-facts-is-unprobed", "llamacpp", {
		exchanges: [LLAMA_LIST, SLEEPING, SLEEPING],
		steps: [models()],
	}),
	server("llamacpp-a-refused-props-read-counts-as-asleep", "llamacpp", {
		exchanges: [LLAMA_LIST, { ...PROPS_A, end: "refused" }, PROPS_B],
		steps: [models()],
	}),
	server("llamacpp-props-that-are-no-object-count-as-asleep", "llamacpp", {
		exchanges: [LLAMA_LIST, reply(null), reply("text")],
		steps: [models()],
	}),
	server("llamacpp-empty-props-are-awake-with-nothing", "llamacpp", {
		exchanges: [LLAMA_LIST, reply({}), reply({})],
		steps: [models(known("local-a.gguf", { thinking: true, efforts: ["low"], context: 8192 }))],
	}),
	server("llamacpp-an-id-is-escaped-in-the-query", "llamacpp", {
		exchanges: [reply({ data: [{ id: "dir/local a.gguf" }, { id: "x&autoload=true" }] }), PROPS_B, PROPS_B],
		steps: [models()],
	}),
	server("llamacpp-a-list-that-does-not-read-throws", "llamacpp", {
		exchanges: [reply({ models: [] })],
		steps: [models()],
	}),
	server("llamacpp-a-list-that-fails-throws", "llamacpp", {
		exchanges: [failing(500)],
		steps: [models()],
	}),
	server("llamacpp-loaded-reads-the-props-of-one-tag", "llamacpp", {
		exchanges: [PROPS_A, PROPS_B],
		steps: [
			{ kind: "loaded", tag: "local-a.gguf" },
			{ kind: "loaded", tag: "local-b.gguf" },
		],
	}),
	server("llamacpp-loaded-of-a-sleeping-model-is-empty", "llamacpp", {
		exchanges: [SLEEPING, reply(null)],
		steps: [
			{ kind: "loaded", tag: "local-a.gguf" },
			{ kind: "loaded", tag: "local-b.gguf" },
		],
	}),
	server("llamacpp-warm-asks-with-autoload", "llamacpp", {
		exchanges: [PROPS_A],
		steps: [{ kind: "warm", tag: "local-a.gguf" }],
	}),
	server("llamacpp-warm-that-fails-is-still-done", "llamacpp", {
		exchanges: [failing(500)],
		steps: [{ kind: "warm", tag: "local-a.gguf" }],
	}),
	server("llamacpp-a-caller-aborted-before-asks-nothing", "llamacpp", {
		exchanges: [LLAMA_LIST, PROPS_A, PROPS_B],
		steps: [{ ...models(), abortedBefore: true }],
	}),
	server("llamacpp-never-sends-the-key", "llamacpp", {
		stored: KEY,
		exchanges: [LLAMA_LIST, PROPS_A, PROPS_B],
		steps: [models()],
	}),
	server("llamacpp-a-url-of-the-entry-is-the-base", "llamacpp", {
		entry: { label: "Lab", url: "http://lab.local:9000" },
		exchanges: [LLAMA_LIST, PROPS_A, PROPS_B],
		steps: [models()],
	}),

	// The OpenAI-compatible server.
	server("compatible-models-use-the-stored-key-and-read-lengths", "compatible", {
		entry: { label: "Lab", url: REMOTE_URL },
		stored: KEY,
		exchanges: [COMPAT_LIST],
		steps: [models()],
	}),
	server("compatible-without-a-key-sends-none", "compatible", {
		entry: { label: "Lab", url: "http://lab.local:8000" },
		exchanges: [COMPAT_LIST],
		steps: [models()],
	}),
	server("compatible-a-list-that-does-not-read-throws", "compatible", {
		entry: { label: "Lab", url: "http://lab.local:8000" },
		exchanges: [reply({ models: [] })],
		steps: [models()],
	}),
	server("compatible-a-refused-key-throws-the-failure", "compatible", {
		entry: { label: "Lab", url: REMOTE_URL },
		stored: KEY,
		exchanges: [{ status: 401, json: { error: { message: "bad key" } } }],
		steps: [models()],
	}),
	server("compatible-a-list-that-fails-throws", "compatible", {
		entry: { label: "Lab", url: REMOTE_URL },
		exchanges: [failing(503)],
		steps: [models()],
	}),
	server("compatible-ignores-the-known-list", "compatible", {
		entry: { label: "Lab", url: REMOTE_URL },
		exchanges: [COMPAT_LIST],
		steps: [models(known("local-a", { efforts: ["high"], thinking: true }))],
	}),
	server("compatible-has-no-loader-and-no-warm", "compatible", {
		entry: { label: "Lab", url: REMOTE_URL },
		exchanges: [],
		steps: [
			{ kind: "loaded", tag: "local-a" },
			{ kind: "warm", tag: "local-a" },
		],
	}),

	// Ollama through a catalog, a browser and a tester.
	flow("ollama-views-are-the-tested-rows-and-a-baked-context-needs-no-running-read", "ollama", {
		rows: [row(A), row(B), row(C)],
		exchanges: [TAGS, ...SHOWS, CHAT],
		steps: [views(), chat(A)],
	}),
	flow("ollama-a-reply-fills-an-unknown-context-from-the-running-list-capped-at-the-trained-length", "ollama", {
		rows: [row(A), row(B), row(C)],
		exchanges: [TAGS, ...SHOWS, CHAT, reply({ models: [{ name: B, context_length: 262_144 }] }), CHAT],
		steps: [views(), chat(B), views(), chat(B), views()],
	}),
	flow("ollama-a-reply-reads-the-running-list-once-per-model-that-it-names", "ollama", {
		rows: [row(B)],
		exchanges: [TAGS, ...SHOWS, CHAT, reply({ models: [{ name: B, context_length: 8192 }] }), CHAT, CHAT],
		steps: [views(), chat(B), chat(B), chat(B)],
	}),
	flow("ollama-a-model-the-running-list-lacks-is-read-again-after-each-reply", "ollama", {
		rows: [row(B)],
		exchanges: [
			TAGS,
			...SHOWS,
			CHAT,
			reply({ models: [] }),
			CHAT,
			reply({ models: [] }),
			CHAT,
			reply({ models: [] }),
		],
		steps: [views(), chat(B), chat(B), chat(B)],
	}),
	flow("ollama-a-failed-reply-also-triggers-the-running-read", "ollama", {
		rows: [row(B)],
		exchanges: [TAGS, ...SHOWS, NOT_FOUND, reply({ models: [{ name: B, context_length: 8192 }] })],
		steps: [views(), chat(B), views()],
	}),
	flow("ollama-a-caller-aborted-reply-triggers-nothing", "ollama", {
		rows: [row(B)],
		exchanges: [TAGS, ...SHOWS, { ...CHAT, headMs: 100 }],
		steps: [views(), chat(B, { abortAfterMs: 10 }), views()],
	}),
	flow("ollama-a-relist-keeps-a-model-whose-show-fails-and-its-running-context", "ollama", {
		rows: [row(A), row(B), row(C)],
		exchanges: [
			TAGS,
			...SHOWS,
			CHAT,
			reply({ models: [{ name: B, context_length: 65_536 }] }),
			TAGS,
			SHOW_A,
			{ ...SHOW_B, end: "refused" },
			SHOW_C,
			SHOW_EMBED,
		],
		steps: [views(), chat(B), { kind: "list", fresh: true }, views()],
	}),
	flow("ollama-a-tag-that-leaves-the-list-loses-its-running-context", "ollama", {
		rows: [row(B)],
		exchanges: [
			TAGS,
			...SHOWS,
			CHAT,
			reply({ models: [{ name: B, context_length: 65_536 }] }),
			reply({ models: [{ name: A }] }),
			SHOW_A,
			TAGS,
			...SHOWS,
		],
		steps: [views(), chat(B), { kind: "list", fresh: true }, { kind: "list", fresh: true }, views()],
	}),
	flow("ollama-browse-lists-the-servers-models-free-and-retestable", "ollama", {
		exchanges: [TAGS, ...SHOWS],
		steps: [{ kind: "list" }, { kind: "list" }],
	}),
	flow("ollama-browse-a-fresh-list-reads-the-server-again", "ollama", {
		exchanges: [TAGS, ...SHOWS, reply({ models: [{ name: A }] }), SHOW_A],
		steps: [{ kind: "list" }, { kind: "list", fresh: true }, { kind: "list" }],
	}),
	flow("ollama-browse-of-an-unreachable-server-lists-nothing", "ollama", {
		exchanges: [refused()],
		steps: [{ kind: "list" }, { kind: "list" }],
	}),
	flow("ollama-browse-detail-is-empty", "ollama", {
		exchanges: [],
		steps: [{ kind: "detail", id: row(A) }],
	}),
	flow("ollama-the-tester-binds-a-listed-tag", "ollama", {
		exchanges: [TAGS, ...SHOWS],
		steps: [
			{ kind: "listed", id: A },
			{ kind: "listed", id: "missing:1b" },
			{ kind: "listed", id: C },
		],
	}),
	flow("ollama-the-tester-labels-a-path-tag-by-its-last-part", "ollama", {
		exchanges: [reply({ models: [{ name: "hub/lab/local-d:7b" }] }), SHOW_B],
		steps: [{ kind: "listed", id: "hub/lab/local-d:7b" }],
	}),
	flow("ollama-the-tester-views-a-row-of-its-own-entry-only", "ollama", {
		exchanges: [TAGS, ...SHOWS],
		steps: [
			{ kind: "view", row: row(C) },
			{ kind: "view", row: row(A) },
			{ kind: "view", row: row("gone:1b") },
			{ kind: "view", row: row(A, "twin") },
		],
	}),
	flow("ollama-a-chat-goes-keyless-to-the-v1-path", "ollama", {
		stored: KEY,
		rows: [row(A)],
		exchanges: [CHAT],
		steps: [chat(A)],
	}),
	flow("ollama-a-missing-model-ends-on-not-found", "ollama", {
		rows: [row(A)],
		exchanges: [NOT_FOUND],
		steps: [chat(A)],
	}),
	flow("ollama-a-url-of-the-entry-is-where-chats-go", "ollama", {
		entry: { id: "lab", label: "Lab", url: "http://lab.local:11434" },
		exchanges: [CHAT],
		steps: [chat(A)],
	}),
	flow("ollama-a-failed-discovery-gives-no-views-and-waits-a-minute", "ollama", {
		rows: [row(A)],
		exchanges: [failing(500), TAGS, ...SHOWS],
		steps: [views(), { kind: "advance", ms: 59_999 }, views(), { kind: "advance", ms: 1 }, views()],
	}),
	flow("ollama-a-discovery-that-stalls-ends-at-ten-seconds", "ollama", {
		rows: [row(A)],
		exchanges: [{ ...TAGS, headMs: 10_000 }, TAGS, ...SHOWS],
		steps: [views(), { kind: "advance", ms: 60_000 }, views()],
	}),

	// LM Studio through a catalog.
	flow("lmstudio-views-carry-efforts-and-the-loaded-context-and-embeddings-stay-out", "lmstudio", {
		rows: [row("local-a"), row("local-b"), row("embed-a")],
		exchanges: [LM_LIST()],
		steps: [views()],
	}),
	flow("lmstudio-a-model-loaded-on-demand-reports-its-context-capped-at-the-trained-length", "lmstudio", {
		rows: [row("local-a"), row("local-b")],
		exchanges: [LM_LIST(), CHAT, LM_LIST({ loaded_instances: [{ config: { context_length: 16_384 } }] })],
		steps: [views(), chat("local-b"), views()],
	}),
	flow("lmstudio-a-model-reloaded-at-another-context-shows-the-new-one-after-the-next-reply", "lmstudio", {
		rows: [row("local-a"), row("local-b")],
		exchanges: [
			LM_LIST({ loaded_instances: [{ config: { context_length: 65_536 } }], max_context_length: 262_144 }),
			CHAT,
			LM_LIST({ loaded_instances: [{ config: { context_length: 4096 } }], max_context_length: 262_144 }),
			CHAT,
			LM_LIST({ loaded_instances: [{ config: { context_length: 2048 } }], max_context_length: 262_144 }),
		],
		steps: [views(), chat("local-b"), views(), chat("local-b"), views()],
	}),
	flow("lmstudio-an-overflow-reply-reads-the-list-too", "lmstudio", {
		rows: [row("local-a")],
		exchanges: [
			LM_LIST(),
			{ status: 400, json: { error: { message: "Context is only 2048 tokens." } } },
			reply({
				models: [
					{
						key: "local-a",
						type: "llm",
						max_context_length: 262_144,
						loaded_instances: [{ config: { context_length: 2048 } }],
					},
				],
			}),
		],
		steps: [views(), chat("local-a"), views()],
	}),
	flow("lmstudio-browse-and-the-tester", "lmstudio", {
		rows: [row("local-a")],
		exchanges: [LM_LIST()],
		steps: [{ kind: "list" }, { kind: "listed", id: "local-a" }, { kind: "view", row: row("local-a") }],
	}),

	// llama.cpp through a catalog.
	flow("llamacpp-views-take-context-and-efforts-from-props-and-a-sleeping-model-has-neither", "llamacpp", {
		rows: [row("local-a.gguf"), row("local-b.gguf")],
		exchanges: [LLAMA_LIST, PROPS_A, SLEEPING],
		steps: [views()],
	}),
	flow("llamacpp-a-sleeping-model-loads-before-its-test-which-then-reads-its-efforts", "llamacpp", {
		rows: [row("local-a.gguf"), row("local-b.gguf")],
		exchanges: [LLAMA_LIST, SLEEPING, SLEEPING, PROPS_A, LLAMA_LIST, PROPS_A, SLEEPING],
		steps: [views(), { kind: "listed", id: "local-a.gguf" }, views()],
	}),
	flow("llamacpp-a-sleeping-model-is-never-loaded-by-a-view", "llamacpp", {
		rows: [row("local-a.gguf")],
		exchanges: [LLAMA_LIST, SLEEPING, SLEEPING],
		steps: [views(), views()],
	}),
	flow("llamacpp-a-server-without-props-is-reread-at-most-once-per-window", "llamacpp", {
		rows: [row("local-a.gguf")],
		exchanges: [
			LLAMA_LIST,
			failing(404),
			failing(404),
			CHAT,
			LLAMA_LIST,
			failing(404),
			failing(404),
			CHAT,
			CHAT,
			LLAMA_LIST,
			failing(404),
			failing(404),
		],
		steps: [
			views(),
			chat("local-a.gguf"),
			chat("local-a.gguf"),
			{ kind: "advance", ms: 60_000 },
			chat("local-a.gguf"),
		],
	}),
	flow("llamacpp-the-next-reply-rereads-a-model-so-a-preset-change-shows", "llamacpp", {
		rows: [row("local-a.gguf")],
		exchanges: [
			LLAMA_LIST,
			SLEEPING,
			SLEEPING,
			CHAT,
			LLAMA_LIST,
			reply({
				default_generation_settings: { n_ctx: 32_768 },
				chat_template_caps: { reasoning_effort_levels: ["low"] },
			}),
			SLEEPING,
		],
		steps: [views(), chat("local-a.gguf"), views()],
	}),
	flow("llamacpp-a-woken-model-reads-its-loaded-context-after-a-reply", "llamacpp", {
		rows: [row("local-a.gguf")],
		exchanges: [
			reply({ data: [{ id: "local-a.gguf", meta: { n_ctx_train: 131_072 } }] }),
			reply({ default_generation_settings: {}, chat_template_caps: {} }),
			CHAT,
			PROPS_A,
		],
		steps: [views(), chat("local-a.gguf"), views()],
	}),
	flow("llamacpp-browse-lists-models-with-their-context", "llamacpp", {
		exchanges: [LLAMA_LIST, PROPS_A, PROPS_B, LLAMA_LIST, PROPS_A, PROPS_B],
		steps: [{ kind: "list" }, { kind: "list", fresh: true }],
	}),

	// The OpenAI-compatible kind through a catalog.
	flow("compatible-a-remote-host-has-no-retention-promise-and-is-never-free", "compatible", {
		entry: { id: "lab", label: "Lab", url: REMOTE_URL },
		stored: KEY,
		rows: [row("local-a")],
		exchanges: [COMPAT_LIST],
		steps: [
			views({ zdrOnly: true }),
			views({ zdrOnly: false }),
			{ kind: "view", row: row("local-a") },
			{ kind: "list" },
		],
	}),
	flow("compatible-a-lan-host-is-free-and-local", "compatible", {
		entry: { id: "lab", label: "Lab", url: "http://lab.local:8000" },
		rows: [row("local-a")],
		exchanges: [COMPAT_LIST],
		steps: [views({ zdrOnly: true }), { kind: "view", row: row("local-a") }, { kind: "list" }],
	}),
	flow("compatible-a-url-with-a-bare-origin-lists-under-v1", "compatible", {
		entry: { id: "lab", label: "Lab", url: "http://lab.local:8000" },
		exchanges: [COMPAT_LIST, CHAT],
		steps: [{ kind: "list" }, chat("local-a")],
	}),
	flow("compatible-chats-use-the-stored-key-and-post-under-the-urls-path", "compatible", {
		entry: { id: "lab", label: "Lab", url: REMOTE_URL },
		stored: KEY,
		rows: [row("local-a")],
		exchanges: [CHAT, COMPAT_LIST],
		steps: [chat("local-a"), { kind: "list" }],
	}),
	flow("compatible-chats-with-no-key-send-none", "compatible", {
		entry: { id: "lab", label: "Lab", url: "http://lab.local:8000/v1" },
		exchanges: [CHAT],
		steps: [chat("local-a")],
	}),
	flow("compatible-every-listed-model-is-tested-for-tools-and-has-no-reasoning", "compatible", {
		entry: { id: "lab", label: "Lab", url: REMOTE_URL },
		rows: [row("local-a"), row("local-b")],
		exchanges: [COMPAT_LIST],
		steps: [{ kind: "list" }, { kind: "listed", id: "local-b" }],
	}),
	flow("compatible-a-reply-reads-nothing-the-kind-has-no-loader", "compatible", {
		entry: { id: "lab", label: "Lab", url: REMOTE_URL },
		rows: [row("local-a")],
		exchanges: [COMPAT_LIST, CHAT],
		steps: [views({ zdrOnly: false }), chat("local-b")],
	}),
];
