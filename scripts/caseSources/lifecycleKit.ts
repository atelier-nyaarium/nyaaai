// Builders for the lifecycle cases. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { caseOf } from "./catalogKit";
import { call, finish, type Json, text } from "./codecFixtures";
import { DONE_TEXT, frame, JSON_HEAD, SSE_HEAD } from "./transportKit";

////////////////////////////////
//  Constants

/** 2026-11-07T12:00:00Z. */
export const START = Date.parse("2026-11-07T12:00:00Z");

export const RESET = "2026-11-07T17:00:00Z";

export const RESET_MS = Date.parse(RESET);

export const LUNA = "openai/gpt-6-luna";

export const OPUS = "anthropic/claude-opus-5.5";

export const LOCAL_ROW = "local:box/qwen";

export const KEY = { kind: "key", key: "fake-key-one" };

export const NEW_KEY = { kind: "key", key: "fake-key-two" };

export const A = { id: "a", kind: "openrouter", label: "Alpha" };

export const B = { id: "b", kind: "openrouter", label: "Beta" };

export const GO = { id: "go", kind: "opencode", plan: "go", label: "Go" };

export const BOX = { id: "box", kind: "ollama", label: "Box", url: "http://box.local:11434" };

export const USAGE = { prompt_tokens: 100, completion_tokens: 10, cost: 0.001 };

export const BALANCE = { ...USAGE, opencode: { billing_source: "balance" } };

export const POOL = { ...USAGE, opencode: { billing_source: "go" } };

export const CHAT = { feature: "chat" };

export const ANALYSIS = { feature: "analysis" };

/** A balance cooldown's estimate, five hours. */
export const BALANCE_COOL_MS = 5 * 3_600_000;

////////////////////////////////
//  Functions & Helpers

/** A streamed answer. */
export const sse = (frames: string[], over: Json = {}): Json => ({
	status: 200,
	headers: SSE_HEAD,
	chunks: frames,
	...over,
});

/** A whole JSON answer. */
export const json = (body: unknown, status = 200, headers: Array<[string, string]> = [], over: Json = {}): Json => ({
	status,
	headers: [...JSON_HEAD, ...headers],
	chunks: [JSON.stringify(body)],
	...over,
});

export const says = (content: string, usage: unknown = USAGE, over: Json = {}): Json =>
	sse([frame(text(content)), frame(finish("stop", usage)), DONE_TEXT], over);

/** A step that asks for a tool, so the run goes on. */
export const reads = (usage: unknown = USAGE): Json =>
	sse([frame(call(0, { id: "c1", name: "read_file", args: "{}" })), frame(finish("tool_calls", usage)), DONE_TEXT]);

/** An unstreamed reply. */
export const answers = (content: string, usage: unknown = USAGE): Json =>
	json({ id: "gen-1", provider: "Azure", choices: [{ message: { content }, finish_reason: "stop" }], usage });

export const failing = (status: number, metadata?: Json, headers: Array<[string, string]> = []): Json =>
	json({ error: { code: status, message: `Failed with ${status}`, metadata } }, status, headers);

export const overloaded = failing(429, { error_type: "rate_limit_exceeded", provider_name: "Azure" });

/** A Go pool limit with its reset. */
export const poolLimit = (resetsAt: string | number = RESET): Json =>
	json({ error: { type: "GoUsageLimitError", message: "Limit.", resetsAt } }, 429);

/** The connection is refused. */
export const down: Json = { status: 0, headers: [], chunks: [], end: "refused", error: "fetch failed" };

/** Streamed output, then the provider fails. */
export const dies = (usage?: unknown): Json =>
	sse([
		frame(text("Half")),
		frame(
			usage === undefined
				? { error: { code: 502, message: "Down" } }
				: { error: { code: 502, message: "Down" }, usage },
		),
	]);

export const send = (callName: string, over: Json = {}): Json => ({
	kind: "send",
	call: callName,
	model: LUNA,
	tag: CHAT,
	...over,
});

export const advance = (ms: number): Json => ({ kind: "advance", ms });

export const plan = (name: string, model = LUNA): Json => ({ kind: "plan", name, model });

export const begin = (run: string): Json => ({ kind: "begin", run });

export const key = (id: string, answer: Json): Json => ({ kind: "key", id, answer });

export const list = (...entries: Json[]): Json => ({ kind: "list", entries });

export const zdr = (on: boolean): Json => ({ kind: "zdr", on });

export const abort = (callName: string): Json => ({ kind: "abort", call: callName });

const BASE = {
	startMs: START,
	entries: [A, B],
	keys: { a: KEY, b: KEY },
	rows: [LUNA, OPUS],
	kinds: ["openrouter", "opencode", "ollama"],
	listings: {},
	zdrOnly: false,
	exchanges: [],
	steps: [],
};

/** One lifecycle run. `over` replaces any input field. */
export const run = (name: string, over: Json): CaseSource => caseOf("lifecycle.run")(name, { ...BASE, ...over });
