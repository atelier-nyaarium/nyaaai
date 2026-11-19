// What a local kind builds and says.

import type { CaseSource } from "../caseAdapter";
import { caseOf } from "./catalogKit";

////////////////////////////////
//  Fixtures

const KINDS = ["ollama", "lmstudio", "llamacpp", "compatible"];

const A = "local-a:27b";

const STATES = [
	{ kind: "none" },
	{ kind: "checking" },
	{ kind: "ok" },
	{ kind: "ok", models: 0 },
	{ kind: "ok", models: 1 },
	{ kind: "ok", models: 2 },
	{ kind: "ok", models: 12, usage: 1, limit: 2 },
	{ kind: "rejected" },
	{ kind: "unreachable" },
	{ kind: "invalid" },
];

const CATEGORIES = [
	"keyMissing",
	"keyRejected",
	"outOfCredit",
	"allowance",
	"refused",
	"notFound",
	"badRequest",
	"replayRejected",
	"rateLimited",
	"overloaded",
	"unavailable",
	"network",
	"stream",
	"empty",
	"truncated",
	"idle",
];

/** No view first, then each status a view holds. */
const VIEWS: Array<string | null> = [null, "ok", "retired", "noZdrHost", "noTools", "unknown"];

const REPLAY_HISTORY = [
	{ role: "user", text: "Hi" },
	{
		role: "step",
		producer: { model: A },
		parts: [
			{ kind: "thought", type: "reasoning", segments: [{ kind: "content", text: "Thinking." }] },
			{ kind: "text", text: "Hello." },
		],
	},
	{ role: "user", text: "More" },
];

const STATUSES = [400, 401, 402, 403, 404, 408, 429, 500, 503];

////////////////////////////////
//  Functions & Helpers

const facts = caseOf("local.kindFacts");
const codec = caseOf("local.kindCodec");
const disposition = caseOf("local.kindDisposition");

const request = (over: Record<string, unknown> = {}) => ({
	model: A,
	system: ["Rules."],
	history: [{ role: "user", text: "Hi" }],
	cache: { kind: "implicit" },
	session: "root-1",
	zdrOnly: true,
	...over,
});

const failure = (category: string, extra: Record<string, unknown> = {}) => ({
	category,
	status: category,
	message: "m",
	...extra,
});

const FAILURES = STATUSES.map((status) => ({ status, body: { error: { message: "No." } } }));

const codecCase = (name: string, kind: string, input: Record<string, unknown>) =>
	codec(name, {
		kind,
		label: "Lab",
		view: null,
		request: request(),
		mode: "stream",
		failures: [],
		...input,
	});

////////////////////////////////
//  Cases

export const localKindCases: CaseSource[] = [
	// The Settings line.
	...KINDS.filter((kind) => kind !== "compatible").map((kind) =>
		facts(`facts-${kind}-the-default-url-and-each-state`, { kind, states: STATES }),
	),
	...KINDS.flatMap((kind) => [
		facts(`facts-${kind}-a-url-of-the-entry`, { kind, url: "http://lab.local:9000", states: STATES }),
		facts(`facts-${kind}-trailing-slashes-are-trimmed`, {
			kind,
			url: "http://lab.local:9000///",
			states: STATES.slice(0, 5),
		}),
		facts(`facts-${kind}-a-path-is-kept`, {
			kind,
			url: "https://host.lab.example/api/v2",
			states: STATES.slice(0, 5),
		}),
	]),
	facts("facts-compatible-a-bare-origin-shows-v1", {
		kind: "compatible",
		url: "http://lab.local:8000",
		states: STATES,
	}),
	facts("facts-compatible-a-bare-origin-with-slashes-shows-v1", {
		kind: "compatible",
		url: "http://lab.local:8000//",
		states: STATES.slice(0, 5),
	}),
	facts("facts-compatible-v1-is-a-path", {
		kind: "compatible",
		url: "http://lab.local:8000/v1",
		states: STATES.slice(0, 5),
	}),
	facts("facts-compatible-a-port-and-a-host-in-capitals", {
		kind: "compatible",
		url: "HTTP://LAB.LOCAL:8000",
		states: STATES.slice(0, 5),
	}),
	facts("facts-the-default-url-wins-over-nothing-but-the-entrys-wins-over-it", {
		kind: "ollama",
		url: "http://127.0.0.1:11434/",
		states: STATES.slice(0, 5),
	}),

	// The codec.
	...KINDS.flatMap((kind) => [
		codecCase(`codec-${kind}-a-stream-request-no-view`, kind, { failures: FAILURES }),
		codecCase(`codec-${kind}-a-complete-request-no-view`, kind, { mode: "complete" }),
		codecCase(`codec-${kind}-an-offered-effort-is-sent`, kind, {
			view: { efforts: ["low", "high"] },
			request: request({ effort: "low" }),
		}),
		codecCase(`codec-${kind}-an-effort-the-view-does-not-offer-is-not-sent`, kind, {
			view: { efforts: ["low", "high"] },
			request: request({ effort: "medium" }),
		}),
		codecCase(`codec-${kind}-an-effort-with-no-view-is-not-sent`, kind, { request: request({ effort: "low" }) }),
		codecCase(`codec-${kind}-the-views-facts`, kind, {
			view: { efforts: ["none", "high"], price: "free", outputLimit: 4096 },
		}),
		codecCase(`codec-${kind}-an-unpriced-view-is-not-free`, kind, {
			view: { efforts: [], price: "unknown", outputLimit: 2048 },
		}),
		codecCase(`codec-${kind}-the-reply-of-the-same-model-is-replayed-by-the-kind`, kind, {
			request: request({ history: REPLAY_HISTORY }),
		}),
		codecCase(`codec-${kind}-the-reply-of-another-model-is-not-replayed`, kind, {
			request: request({ model: "local-b:9b", history: REPLAY_HISTORY }),
		}),
		codecCase(`codec-${kind}-a-forced-tool-goes-as-the-schema`, kind, {
			request: request({
				tools: [{ name: "answer", description: "Answers.", parameters: { type: "object", properties: {} } }],
				toolChoice: { name: "answer" },
			}),
		}),
		codecCase(`codec-${kind}-a-view-with-an-api-fact`, kind, {
			view: {
				efforts: [],
				api: {
					family: "chat-completions",
					structured: true,
					choice: [],
					strict: false,
					reasoning: "reasoning",
				},
			},
		}),
	]),
	codecCase("codec-a-label-is-the-name-failures-carry", "ollama", {
		label: "Lab Node",
		failures: [
			{ status: 500, body: { error: { message: "" } } },
			{ status: 500, body: { error: {} } },
			{ status: 503, body: "text" },
		],
	}),
	codecCase("codec-compatible-keyed-failures-name-the-key", "compatible", {
		failures: [
			{ status: 401, body: { error: { message: "bad key" } } },
			{ status: 402, body: { error: { message: "pay" } } },
			{ status: 403, body: { error: { message: "no" } } },
		],
	}),
	codecCase("codec-ollama-unkeyed-failures-never-name-the-key", "ollama", {
		failures: [
			{ status: 401, body: { error: { message: "bad key" } } },
			{ status: 402, body: { error: { message: "pay" } } },
			{ status: 403, body: { error: { message: "no" } } },
		],
	}),

	// Failures.
	...KINDS.map((kind) =>
		disposition(`disposition-${kind}-every-category-against-every-view`, {
			kind,
			rows: CATEGORIES.flatMap((category) => VIEWS.map((status) => ({ failure: failure(category), status }))),
		}),
	),
	...KINDS.map((kind) =>
		disposition(`disposition-${kind}-out-of-credit-with-retry-after`, {
			kind,
			rows: [
				{ failure: failure("outOfCredit", { retryAfterMs: 1 }), status: null },
				{ failure: failure("outOfCredit", { retryAfterMs: 0 }), status: "ok" },
			],
		}),
	),
	...KINDS.map((kind) =>
		disposition(`disposition-${kind}-not-found-is-unavailable-whatever-the-view`, {
			kind,
			rows: VIEWS.map((status) => ({ failure: failure("notFound", { status: 404 }), status })),
		}),
	),
];
