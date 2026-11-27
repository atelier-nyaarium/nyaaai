// The checks and limits of OpenRouter and OpenCode on a scripted network. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { caseOf, failing, NOW, reply } from "./catalogKit";

////////////////////////////////
//  Fixtures

const KEY = "fake-key-one";

const STATUSES = [400, 401, 402, 403, 404, 408, 422, 429, 500, 502, 503, 529];

const KEY_BODY = reply({ data: { usage: 4.12, limit: 20, limit_reset: "monthly" } });

const MODELS_BODY = reply({ data: [{ id: "glm-5.3-flash" }] });

const DENIED = (status: number) => ({ status, json: { error: { message: "No.", code: status } } });

////////////////////////////////
//  Functions & Helpers

const check = (name: string, kind: string, input: Record<string, unknown>): CaseSource =>
	caseOf("kind.check")(name, {
		kind,
		entry: { label: "Zen" },
		secret: KEY,
		startMs: NOW,
		exchanges: [],
		...input,
	});

const limits = (name: string, kind: string, input: Record<string, unknown>): CaseSource =>
	caseOf("kind.limits")(name, {
		kind,
		entry: { label: "Zen" },
		stored: KEY,
		startMs: NOW,
		exchanges: [],
		...input,
	});

const OR = (name: string, input: Record<string, unknown>) => check(name, "openrouter", input);
const OC = (name: string, input: Record<string, unknown>) => check(name, "opencode", input);

////////////////////////////////
//  Cases

export const kindCheckCases: CaseSource[] = [
	// OpenRouter.
	OR("openrouter-no-key-is-none-and-asks-nothing", { secret: null, exchanges: [KEY_BODY] }),
	OR("openrouter-a-key-reads-usage-and-limit", { exchanges: [KEY_BODY] }),
	OR("openrouter-the-asked-url-and-the-agent", { exchanges: [KEY_BODY] }),
	OR("openrouter-an-unlimited-key", { exchanges: [reply({ data: { usage: 0, limit: null } })] }),
	OR("openrouter-a-reset-is-not-a-state-field", { exchanges: [reply({ data: { usage: 1, limit_reset: "daily" } })] }),
	OR("openrouter-a-key-with-no-body-data-is-ok-and-empty", { exchanges: [reply({})] }),
	OR("openrouter-the-url-of-a-test-loopback", { baseUrl: "http://127.0.0.1:9", exchanges: [KEY_BODY] }),
	OR("openrouter-a-base-url-over-http-to-a-public-host-sends-no-key", {
		baseUrl: "http://api.example.test",
		exchanges: [KEY_BODY],
	}),
	OR("openrouter-a-base-url-that-does-not-parse-sends-no-key", {
		baseUrl: "http://%70ublic.example.test/v1",
		exchanges: [KEY_BODY],
	}),
	...STATUSES.map((status) => OR(`openrouter-status-${status}`, { exchanges: [DENIED(status)] })),
	OR("openrouter-a-dropped-connection-is-unreachable", { exchanges: [{ status: 200, text: "{", end: "dropped" }] }),
	OR("openrouter-a-refused-connection-is-unreachable", { exchanges: [{ status: 200, text: "", end: "refused" }] }),
	OR("openrouter-a-reply-that-is-no-json-is-unreachable", { exchanges: [{ status: 200, text: "<html>" }] }),
	OR("openrouter-a-malformed-key-is-rejected-without-a-request", { secret: "fake bad", exchanges: [KEY_BODY] }),
	OR("openrouter-caller-aborts-after-the-read-starts", {
		exchanges: [{ ...KEY_BODY, headMs: 100 }],
		abortAfterMs: 50,
	}),
	OR("openrouter-caller-aborted-before-the-call", { exchanges: [KEY_BODY], abortedBefore: true }),
	OR("openrouter-a-slow-server-leaves-no-timer", { exchanges: [{ ...KEY_BODY, headMs: 600_000 }] }),

	// OpenCode.
	OC("opencode-no-key-is-none-and-asks-nothing", { secret: null, exchanges: [MODELS_BODY] }),
	OC("opencode-go-is-the-default-plan", { exchanges: [MODELS_BODY] }),
	OC("opencode-go-asks-the-go-url", { entry: { label: "Zen", plan: "go" }, exchanges: [MODELS_BODY] }),
	OC("opencode-payg-asks-the-plain-url", { entry: { label: "Zen", plan: "payg" }, exchanges: [MODELS_BODY] }),
	OC("opencode-the-url-of-a-test-loopback", { baseUrl: "http://127.0.0.1:9", exchanges: [MODELS_BODY] }),
	OC("opencode-any-body-on-200-is-ok", { exchanges: [reply(null)] }),
	...["go", "payg"].flatMap((plan) =>
		STATUSES.map((status) =>
			OC(`opencode-${plan}-status-${status}`, { entry: { label: "Zen", plan }, exchanges: [DENIED(status)] }),
		),
	),
	OC("opencode-a-dropped-connection-is-unreachable", { exchanges: [{ status: 200, text: "{", end: "dropped" }] }),
	OC("opencode-a-refused-connection-is-unreachable", { exchanges: [{ status: 200, text: "", end: "refused" }] }),
	OC("opencode-a-malformed-key-is-rejected-without-a-request", { secret: "fake bad", exchanges: [MODELS_BODY] }),
	OC("opencode-caller-aborts-after-the-read-starts", {
		exchanges: [{ ...MODELS_BODY, headMs: 100 }],
		abortAfterMs: 50,
	}),
	OC("opencode-caller-aborted-before-the-call", { exchanges: [MODELS_BODY], abortedBefore: true }),
	OC("opencode-caller-aborts-while-the-server-fails", {
		exchanges: [{ ...failing(500), headMs: 100 }],
		abortAfterMs: 50,
	}),
	OC("opencode-a-slow-server-leaves-no-timer", { exchanges: [{ ...MODELS_BODY, headMs: 600_000 }] }),

	// Limits.
	limits("limits-openrouter-a-limit-and-its-reset", "openrouter", { exchanges: [KEY_BODY] }),
	limits("limits-openrouter-no-limit", "openrouter", { exchanges: [reply({ data: { usage: 1 } })] }),
	limits("limits-openrouter-only-a-reset", "openrouter", { exchanges: [reply({ data: { limit_reset: "daily" } })] }),
	limits("limits-openrouter-a-rejected-key-has-none", "openrouter", { exchanges: [DENIED(401)] }),
	limits("limits-openrouter-an-unreachable-server-has-none", "openrouter", { exchanges: [failing(503)] }),
	limits("limits-openrouter-no-stored-key-has-none-and-asks-nothing", "openrouter", {
		stored: null,
		exchanges: [KEY_BODY],
	}),
	limits("limits-openrouter-caller-aborts-and-the-limits-are-none", "openrouter", {
		exchanges: [{ ...KEY_BODY, headMs: 100 }],
		abortAfterMs: 50,
	}),
	limits("limits-opencode-has-no-limits", "opencode", { exchanges: [MODELS_BODY] }),
];
