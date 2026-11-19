// The OpenRouter key check over statuses and bodies. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { failing, NOW, reply } from "./catalogKit";

////////////////////////////////
//  Functions & Helpers

const KEY = "key-0123456789abcdef";

const check = (name: string, input: Record<string, unknown>): CaseSource => ({
	kind: "kind.openrouterKey",
	name,
	input: { startMs: NOW, stored: KEY, ...input },
});

const stored = { auth: "stored" };

const body = (data: unknown) => reply({ data });

////////////////////////////////
//  Cases

export const kindKeyCases: CaseSource[] = [
	check("key-usage-and-limit", {
		exchanges: [body({ usage: 4.12, limit: 20, limit_reset: "monthly" })],
		checks: [stored],
	}),
	check("key-unlimited", { exchanges: [body({ usage: 0, limit: null })], checks: [stored] }),
	check("key-rejected-by-401", {
		exchanges: [{ status: 401, json: { error: { message: "No auth", code: 401 } } }],
		checks: [stored],
	}),
	check("key-given-and-sent", {
		exchanges: [body({ usage: 1 })],
		checks: [{ auth: "key", key: "key-fedcba9876543210" }],
	}),
	check("key-stored-and-missing-is-rejected-without-a-request", {
		stored: null,
		exchanges: [body({ usage: 1 })],
		checks: [stored],
	}),
	check("key-answers-by-status", {
		exchanges: [400, 401, 402, 403, 404, 408, 422, 429, 500, 502, 503, 529].map((status) => ({
			status,
			json: { error: { message: "Failed.", code: status } },
		})),
		checks: Array.from({ length: 12 }, () => stored),
	}),
	check("key-answers-by-status-with-plain-text-bodies", {
		exchanges: [401, 403, 429, 500].map((status) => ({ status, text: "gateway text" })),
		checks: Array.from({ length: 4 }, () => stored),
	}),
	check("key-a-dropped-and-a-refused-connection-are-unreachable", {
		exchanges: [
			{ status: 200, text: "{", end: "dropped" },
			{ status: 200, text: "", end: "refused" },
		],
		checks: [stored, stored],
	}),
	check("key-a-reply-that-is-no-json-is-unreachable", {
		exchanges: [{ status: 200, text: "<html>" }],
		checks: [stored],
	}),
	check("key-data-missing-or-not-an-object", {
		exchanges: [
			reply({}),
			reply({ data: null }),
			reply({ data: "x" }),
			reply({ data: [] }),
			reply({ data: 5 }),
			reply([]),
			reply("x"),
			reply(5),
			reply(true),
		],
		checks: Array.from({ length: 9 }, () => stored),
	}),
	check("key-body-null-reads-as-an-empty-key", {
		exchanges: [reply(null)],
		checks: [stored],
	}),
	check("key-fields-of-the-wrong-type-read-as-absent", {
		exchanges: [
			body({ usage: "4", limit: "20", limit_reset: 5 }),
			body({ usage: null, limit: true, limit_reset: null }),
			body({ usage: {}, limit: [1], limit_reset: {} }),
			body({ usage: -3.5, limit: 0, limit_reset: "" }),
		],
		checks: Array.from({ length: 4 }, () => stored),
	}),
	check("key-numbers", {
		exchanges: [body({ usage: 1e21, limit: 1e-7 }), body({ usage: 0.1, limit: 100 })],
		checks: [stored, stored],
	}),
	check("key-unknown-fields-ignored", {
		exchanges: [body({ usage: 2, limit: 5, limit_reset: "daily", label: "x", is_free_tier: false })],
		checks: [stored],
	}),
	check("key-caller-aborts-after-the-read-starts", {
		exchanges: [{ ...body({ usage: 1 }), headMs: 100 }],
		checks: [{ auth: "stored", abortAfterMs: 50 }],
	}),
	check("key-caller-aborted-before-the-call", {
		exchanges: [body({ usage: 1 })],
		checks: [{ auth: "stored", abortedBefore: true }],
	}),
	check("key-caller-aborts-while-the-server-fails", {
		exchanges: [{ ...failing(500), headMs: 100 }],
		checks: [{ auth: "stored", abortAfterMs: 50 }],
	}),
	check("key-no-deadline-of-its-own", {
		exchanges: [{ ...body({ usage: 1 }), headMs: 600_000 }],
		checks: [stored],
	}),
];
