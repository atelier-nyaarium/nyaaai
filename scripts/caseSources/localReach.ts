// Local kind checks on a scripted network. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { failing, NOW, refused, reply } from "./catalogKit";

////////////////////////////////
//  Fixtures

const KEY = "fake-key-one";

const A = "local-a:27b";
const B = "local-b:9b";

const TAGS = reply({
	models: [
		{ name: A },
		{ name: B },
		{ name: "embed-a:latest" },
		{ name: "far:1b-cloud", remote_host: "https://remote.example:443" },
	],
});

const LM_LIST = reply({
	models: [
		{ key: "local-a", type: "llm" },
		{ key: "embed-a", type: "embedding" },
		{ key: "local-b", type: "llm" },
		{ type: "llm" },
	],
});

const LLAMA_LIST = reply({ object: "list", data: [{ id: "local-a.gguf" }, { id: "local-b.gguf" }, {}, { id: "" }] });

const COMPAT_LIST = reply({ data: [{ id: "local-a", context_length: 32_768 }, { id: "local-b" }, {}] });

/** An answer in the wrong place: another server's list. */
const FOREIGN = reply({ object: "list", data: [] });

const DENIED = (status: number) => ({ status, json: { error: { message: "No." } } });

////////////////////////////////
//  Functions & Helpers

const check = (name: string, kind: string, input: Record<string, unknown>): CaseSource => ({
	kind: "local.reach",
	name,
	input: { kind, entry: { label: "Lab" }, secret: null, startMs: NOW, exchanges: [], ...input },
});

const at = (url: string) => ({ entry: { label: "Lab", url } });

/** One case per status a server can answer with. */
const statuses = (kind: string, extra: Record<string, unknown> = {}): CaseSource[] =>
	[400, 401, 402, 403, 404, 408, 429, 500, 503].map((status) =>
		check(`${kind}-status-${status}`, kind, { exchanges: [DENIED(status)], ...extra }),
	);

////////////////////////////////
//  Cases

export const localReachCases: CaseSource[] = [
	// Ollama.
	check("ollama-counts-local-models", "ollama", { exchanges: [TAGS] }),
	check("ollama-one-model-says-model", "ollama", { exchanges: [reply({ models: [{ name: A }] })] }),
	check("ollama-no-models-is-ok-with-zero", "ollama", { exchanges: [reply({ models: [] })] }),
	check("ollama-a-url-of-the-entry-is-asked", "ollama", { ...at("http://lab.local:11434"), exchanges: [TAGS] }),
	check("ollama-trailing-slashes-of-the-url-are-trimmed", "ollama", {
		...at("http://lab.local:11434///"),
		exchanges: [TAGS],
	}),
	check("ollama-a-stored-key-is-never-sent", "ollama", { secret: KEY, exchanges: [TAGS] }),
	check("ollama-foreign-answer-is-unreachable", "ollama", { exchanges: [FOREIGN] }),
	check("ollama-models-wrong-type-is-unreachable", "ollama", { exchanges: [reply({ models: "x" })] }),
	check("ollama-body-null-is-unreachable", "ollama", { exchanges: [reply(null)] }),
	check("ollama-body-not-json-is-unreachable", "ollama", { exchanges: [{ status: 200, text: "<html>no</html>" }] }),
	check("ollama-refused-connection-is-unreachable", "ollama", { exchanges: [refused()] }),
	check("ollama-dropped-body-is-unreachable", "ollama", { exchanges: [{ ...TAGS, end: "dropped" }] }),
	check("ollama-no-answer-left-is-unreachable", "ollama", { exchanges: [] }),
	...statuses("ollama"),
	check("ollama-aborted-before-rethrows-and-asks-nothing", "ollama", { exchanges: [TAGS], abortedBefore: true }),
	check("ollama-abort-during-the-wait-rethrows", "ollama", {
		exchanges: [{ ...TAGS, headMs: 1000 }],
		abortAfterMs: 10,
	}),
	check("ollama-a-slow-answer-still-counts", "ollama", { exchanges: [{ ...TAGS, headMs: 5000 }] }),

	// LM Studio.
	check("lmstudio-counts-chat-models", "lmstudio", { exchanges: [LM_LIST] }),
	check("lmstudio-a-url-of-the-entry-is-asked", "lmstudio", {
		...at("http://lab.local:1234/"),
		exchanges: [LM_LIST],
	}),
	check("lmstudio-foreign-answer-is-unreachable", "lmstudio", { exchanges: [reply({ data: [] })] }),
	check("lmstudio-a-stored-key-is-never-sent", "lmstudio", { secret: KEY, exchanges: [LM_LIST] }),
	check("lmstudio-no-chat-models-is-ok-with-zero", "lmstudio", {
		exchanges: [reply({ models: [{ key: "embed-a", type: "embedding" }] })],
	}),
	check("lmstudio-refused-connection-is-unreachable", "lmstudio", { exchanges: [refused()] }),
	...statuses("lmstudio"),
	check("lmstudio-aborted-before-rethrows", "lmstudio", { exchanges: [LM_LIST], abortedBefore: true }),

	// llama.cpp.
	check("llamacpp-counts-models-with-an-id", "llamacpp", { exchanges: [LLAMA_LIST] }),
	check("llamacpp-a-url-of-the-entry-is-asked", "llamacpp", {
		...at("http://lab.local:9000"),
		exchanges: [LLAMA_LIST],
	}),
	check("llamacpp-foreign-answer-is-unreachable", "llamacpp", { exchanges: [reply({ models: [] })] }),
	check("llamacpp-a-stored-key-is-never-sent", "llamacpp", { secret: KEY, exchanges: [LLAMA_LIST] }),
	check("llamacpp-refused-connection-is-unreachable", "llamacpp", { exchanges: [refused()] }),
	...statuses("llamacpp"),

	// OpenAI-compatible.
	check("compatible-a-bare-origin-means-v1", "compatible", {
		...at("http://lab.local:8000"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-a-bare-origin-with-a-slash-means-v1", "compatible", {
		...at("http://lab.local:8000/"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-a-path-is-kept", "compatible", {
		...at("https://host.lab.example/api/v2"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-a-path-with-slashes-is-trimmed", "compatible", {
		...at("https://host.lab.example/api/v2///"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-v1-is-a-path-and-is-kept", "compatible", {
		...at("http://lab.local:8000/v1"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-counts-models-with-an-id", "compatible", {
		...at("http://lab.local:8000"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-a-stored-key-is-sent", "compatible", {
		...at("https://host.lab.example/api/v2"),
		secret: KEY,
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-no-key-sends-none", "compatible", { ...at("http://lab.local:8000"), exchanges: [COMPAT_LIST] }),
	check("compatible-a-key-on-a-lan-server-is-sent-too", "compatible", {
		...at("http://lab.local:8000"),
		secret: KEY,
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-a-malformed-key-is-rejected-without-a-request", "compatible", {
		...at("https://host.lab.example/api/v2"),
		secret: "fake bad",
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-an-empty-key-is-no-key", "compatible", {
		...at("https://host.lab.example/api/v2"),
		secret: "",
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-foreign-answer-is-unreachable", "compatible", {
		...at("http://lab.local:8000"),
		exchanges: [reply({ models: [] })],
	}),
	check("compatible-refused-connection-is-unreachable", "compatible", {
		...at("http://lab.local:8000"),
		exchanges: [refused()],
	}),
	check("compatible-no-url-is-unreachable-and-asks-nothing", "compatible", { exchanges: [COMPAT_LIST] }),
	check("compatible-a-url-that-does-not-read-is-unreachable-and-asks-nothing", "compatible", {
		...at("not a url"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-a-host-the-parser-refuses-is-unreachable-and-asks-nothing", "compatible", {
		...at("http://b\u00fcro.local:8000"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-a-query-is-unreachable-and-asks-nothing", "compatible", {
		...at("http://lab.local:8000?x=1"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-a-fragment-is-unreachable-and-asks-nothing", "compatible", {
		...at("http://lab.local:8000/#f"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-credentials-are-unreachable-and-ask-nothing", "compatible", {
		...at("http://user:pw@lab.local:8000"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-the-base-is-the-origin-and-the-path-as-parsed", "compatible", {
		...at("HTTP://LAB.LOCAL:8000/a/./b/../c//"),
		exchanges: [COMPAT_LIST],
	}),
	check("compatible-the-default-port-is-dropped-from-the-base", "compatible", {
		...at("http://lab.local:80"),
		exchanges: [COMPAT_LIST],
	}),
	...statuses("compatible", at("http://lab.local:8000")),
	check("compatible-aborted-before-rethrows", "compatible", {
		...at("http://lab.local:8000"),
		exchanges: [COMPAT_LIST],
		abortedBefore: true,
	}),
	check("compatible-401-with-a-key-is-rejected", "compatible", {
		...at("https://host.lab.example/api/v2"),
		secret: KEY,
		exchanges: [failing(401)],
	}),
	check("compatible-402-with-a-key-is-unreachable", "compatible", {
		...at("https://host.lab.example/api/v2"),
		secret: KEY,
		exchanges: [DENIED(402)],
	}),
];
