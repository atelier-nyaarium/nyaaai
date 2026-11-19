// What the kinds say and build.

import type { CaseSource } from "../caseAdapter";
import { caseOf } from "./catalogKit";

////////////////////////////////
//  Fixtures

const NAMES = ["opencode", "openrouter", "ollama", "lmstudio", "llamacpp", "compatible"];

const STATES = [
	{ kind: "none" },
	{ kind: "checking" },
	{ kind: "ok" },
	{ kind: "ok", usage: 4.5, limit: 20 },
	{ kind: "ok", usage: 0 },
	{ kind: "ok", models: 3 },
	{ kind: "ok", models: 1, usage: 1 },
	{ kind: "rejected" },
	{ kind: "unreachable" },
	{ kind: "invalid" },
];

const BILLS = [
	{ billing: "balance", allowance: 3.256, cost: 0.1 },
	{ billing: "allowance", allowance: 5 },
	{ billing: "allowance" },
	{ billing: "balance" },
	{ billing: "balance", cost: 0 },
	{ billing: "payg", cost: 12.345 },
	{ billing: "allowance", allowance: 0, cost: 0 },
	{ billing: "allowance", allowance: 1, cost: 0.004 },
];

/** An entry per kind; the local ones have a URL. */
const ENTRIES = {
	opencode: { id: "e1", kind: "opencode", label: "Zen" },
	openrouter: { id: "e1", kind: "openrouter", label: "Router" },
	ollama: { id: "e1", kind: "ollama", label: "Lab", url: "http://lab.local:11434" },
	lmstudio: { id: "e1", kind: "lmstudio", label: "Lab", url: "http://lab.local:1234" },
	llamacpp: { id: "e1", kind: "llamacpp", label: "Lab", url: "http://lab.local:8080" },
	compatible: { id: "e1", kind: "compatible", label: "Lab", url: "http://lab.local:8000/v1" },
};

const LUNA = "openai/gpt-6-luna";

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

const CHAT_API = { family: "chat-completions", structured: true, choice: ["auto", "none"], strict: false };

const API = {
	chat: CHAT_API,
	reasoningField: { ...CHAT_API, reasoning: "reasoning_content" },
	responses: { family: "responses", structured: true, choice: ["auto"], strict: true },
	messages: { family: "messages", structured: true, choice: ["auto", "any"], strict: false },
	gemini: { family: "generate-content", structured: true, choice: ["auto"], strict: false },
};

////////////////////////////////
//  Functions & Helpers

const lines = caseOf("kind.lines");
const view = caseOf("kind.providerView");
const facts = caseOf("kind.facts");
const codec = caseOf("kind.codec");
const disposition = caseOf("kind.disposition");

const request = (over: Record<string, unknown> = {}) => ({
	model: LUNA,
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

const codecCase = (name: string, kind: string, input: Record<string, unknown>) =>
	codec(name, {
		kind,
		label: "Zen",
		view: null,
		request: request(),
		mode: "stream",
		nowMs: 1_790_337_600_000,
		failures: [],
		...input,
	});

const TOOL = { name: "answer", description: "Answers.", parameters: { type: "object", properties: {} } };

const FAILURES = [400, 401, 402, 403, 404, 408, 422, 429, 500, 503, 529].map((status) => ({
	status,
	body: { error: { message: "No." } },
}));

const withView = (view: Record<string, unknown>) => ({ efforts: ["low", "high"], ...view });

////////////////////////////////
//  Cases

export const kindCases: CaseSource[] = [
	// The dropdown.
	lines("lines-the-dropdown-order-of-the-kinds", {}),
	...NAMES.map((name) => lines(`lines-${name}-alone`, { kinds: [name] })),
	lines("lines-the-order-follows-the-table", { kinds: [...NAMES].reverse() }),
	lines("lines-an-empty-table-has-no-line", { kinds: [] }),
	lines("lines-a-kind-twice-is-one", { kinds: ["opencode", "ollama", "opencode"] }),

	// The provider list.
	...NAMES.flatMap((name) =>
		STATES.map((status, at) =>
			view(`view-${name}-state-${at}`, { entry: ENTRIES[name as keyof typeof ENTRIES], status }),
		),
	),
	...["go", "payg"].flatMap((plan) =>
		[undefined, false, true].map((extraUsage) =>
			view(`view-opencode-${plan}-extra-usage-${extraUsage ?? "absent"}`, {
				entry: {
					id: "e1",
					kind: "opencode",
					label: "Zen",
					plan,
					...(extraUsage === undefined ? {} : { extraUsage }),
				},
				status: { kind: "ok" },
			}),
		),
	),
	...["go", "payg"].flatMap((plan) =>
		BILLS.map((bill, at) =>
			view(`view-opencode-${plan}-bill-${at}`, {
				entry: { id: "e1", kind: "opencode", label: "Zen", plan },
				status: { kind: "ok" },
				bill,
			}),
		),
	),
	view("view-openrouter-a-bill-adds-nothing", {
		entry: ENTRIES.openrouter,
		status: { kind: "ok", usage: 2 },
		bill: BILLS[0],
	}),
	view("view-a-kind-the-table-lacks-has-no-facts-and-no-balance", {
		entry: ENTRIES.openrouter,
		status: { kind: "ok", usage: 2 },
		kinds: ["opencode"],
	}),
	view("view-an-entry-keeps-its-label-and-url", {
		entry: { id: "e9", kind: "compatible", label: "Host B", url: "https://host.lab.example/api/v2" },
		status: { kind: "unreachable" },
	}),

	// The Settings line.
	facts("facts-openrouter-each-state", {
		kind: "openrouter",
		entry: { label: "Router" },
		rows: STATES.map((state) => ({ state })),
	}),
	facts("facts-openrouter-dollars-round-as-toFixed-does", {
		kind: "openrouter",
		entry: { label: "Router" },
		rows: [
			{ state: { kind: "ok", usage: 4.125, limit: 1.005 } },
			{ state: { kind: "ok", usage: 0.004, limit: 0.005 } },
			{ state: { kind: "ok", usage: 1234567.891, limit: 99.999 } },
			{ state: { kind: "ok", usage: -1.5 } },
			{ state: { kind: "ok", usage: 1e21, limit: 1e-7 } },
		],
	}),
	facts("facts-openrouter-a-bill-is-ignored", {
		kind: "openrouter",
		entry: { label: "Router" },
		rows: BILLS.map((bill) => ({ state: { kind: "ok", usage: 1 }, bill })),
	}),
	...[undefined, "go", "payg"].map((plan) =>
		facts(`facts-opencode-${plan ?? "no-plan"}-each-bill`, {
			kind: "opencode",
			entry: { label: "Zen", ...(plan === undefined ? {} : { plan }) },
			rows: [{ state: { kind: "ok" } }, ...BILLS.map((bill) => ({ state: { kind: "ok" }, bill }))],
		}),
	),
	facts("facts-opencode-the-state-does-not-matter", {
		kind: "opencode",
		entry: { label: "Zen", plan: "go" },
		rows: STATES.map((state) => ({ state, bill: BILLS[0] })),
	}),

	// The codec.
	...["openrouter", "opencode"].flatMap((kind) => [
		codecCase(`codec-${kind}-a-stream-request-no-view`, kind, { failures: FAILURES }),
		codecCase(`codec-${kind}-a-complete-request-no-view`, kind, { mode: "complete" }),
		codecCase(`codec-${kind}-an-offered-effort-is-sent`, kind, {
			view: withView({}),
			request: request({ effort: "low" }),
		}),
		codecCase(`codec-${kind}-an-effort-the-view-does-not-offer`, kind, {
			view: withView({}),
			request: request({ effort: "medium" }),
		}),
		codecCase(`codec-${kind}-an-effort-with-no-view`, kind, { request: request({ effort: "low" }) }),
		codecCase(`codec-${kind}-the-views-facts`, kind, {
			view: { efforts: ["none", "high"], price: "free", outputLimit: 4096 },
		}),
		codecCase(`codec-${kind}-an-unpriced-view-is-not-free`, kind, {
			view: { efforts: [], price: "unknown", outputLimit: 2048 },
		}),
		codecCase(`codec-${kind}-a-forced-tool-goes-as-the-schema`, kind, {
			request: request({ tools: [TOOL], toolChoice: { name: "answer" } }),
		}),
		codecCase(`codec-${kind}-a-view-without-an-api-fact`, kind, { view: withView({}), mode: "complete" }),
		codecCase(`codec-${kind}-the-label-is-not-the-codecs-name`, kind, { label: "Other" }),
	]),
	// OpenCode reads the family of the view.
	...Object.entries(API).flatMap(([name, api]) => [
		codecCase(`codec-opencode-family-${name}-stream`, "opencode", { view: withView({ api }) }),
		codecCase(`codec-opencode-family-${name}-complete`, "opencode", { view: withView({ api }), mode: "complete" }),
		codecCase(`codec-opencode-family-${name}-forced-tool`, "opencode", {
			view: withView({ api }),
			request: request({ tools: [TOOL], toolChoice: { name: "answer" } }),
		}),
		codecCase(`codec-opencode-family-${name}-failures`, "opencode", {
			view: withView({ api }),
			failures: [
				{ status: 401, body: { error: { message: "bad key", type: "AuthError" } } },
				{ status: 429, body: { error: { message: "slow", type: "RateLimitError" } } },
				{ status: 500, body: { type: "error", error: { type: "api_error", message: "boom" } } },
				{ status: 404, body: { error: { message: "no model", code: 404 } } },
			],
		}),
	]),
	codecCase("codec-opencode-the-session-rides-a-header", "opencode", {
		view: withView({ api: API.responses }),
		request: request({ session: "thread-9" }),
	}),
	codecCase("codec-opencode-an-allowance-reset-is-dated-by-the-clock", "opencode", {
		nowMs: 1_790_337_600_000,
		failures: [
			{
				status: 429,
				body: {
					type: "error",
					error: { type: "SubscriptionUsageLimitError", message: "Limit.", resetsAt: 1_790_341_200 },
				},
			},
			{
				status: 429,
				body: { type: "error", error: { type: "FreeUsageLimitError", message: "Limit." } },
			},
		],
	}),
	codecCase("codec-opencode-a-later-clock-moves-the-reset", "opencode", {
		nowMs: 1_790_340_000_000,
		failures: [
			{
				status: 429,
				body: {
					type: "error",
					error: { type: "SubscriptionUsageLimitError", message: "Limit.", resetsAt: 1_790_341_200 },
				},
			},
		],
	}),

	// Failures.
	...["openrouter", "opencode"].map((kind) =>
		disposition(`disposition-${kind}-every-category-against-every-view`, {
			kind,
			rows: CATEGORIES.flatMap((category) => VIEWS.map((status) => ({ failure: failure(category), status }))),
		}),
	),
	...["openrouter", "opencode"].map((kind) =>
		disposition(`disposition-${kind}-out-of-credit-with-retry-after`, {
			kind,
			rows: [
				{ failure: failure("outOfCredit", { retryAfterMs: 1 }), status: null },
				{ failure: failure("outOfCredit", { retryAfterMs: 0 }), status: "ok" },
				{ failure: failure("outOfCredit"), status: "ok" },
			],
		}),
	),
	...["openrouter", "opencode"].map((kind) =>
		disposition(`disposition-${kind}-not-found-against-every-view`, {
			kind,
			rows: VIEWS.map((status) => ({ failure: failure("notFound", { status: 404 }), status })),
		}),
	),
	...["openrouter", "opencode"].map((kind) =>
		disposition(`disposition-${kind}-bad-request-and-replay-rejected`, {
			kind,
			rows: ["badRequest", "replayRejected"].flatMap((category) =>
				VIEWS.map((status) => ({ failure: failure(category, { status: 400 }), status })),
			),
		}),
	),
];
