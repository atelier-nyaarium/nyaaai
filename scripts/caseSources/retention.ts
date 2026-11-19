// OpenCode retention cases.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Helpers

type Plan = "go" | "payg";

/** 2026-12-01T00:00:00Z: the day after the last zero-retention day of the dated rows. */
const END = 1796083200000;

const of = (name: string, plan: Plan, ids: string[]): CaseSource => ({
	kind: "retention.of",
	name,
	input: { rows: ids.map((id) => ({ plan, id })) },
});

const zdr = (name: string, rows: Array<[Plan, string, number]>): CaseSource => ({
	kind: "retention.isZdr",
	name,
	input: { rows: rows.map(([plan, id, now]) => ({ plan, id, now })) },
});

/** Ids that read nothing on either plan. */
const NOBODY = [
	"",
	"unknown-model",
	"GLM-5",
	"Glm-5",
	"glm",
	"constructor",
	"__proto__",
	"toString",
	"hasOwnProperty",
	"valueOf",
	"*",
	"-*",
	" glm-5",
	"glm-5 ",
	"\u{e9}",
];

////////////////////////////////
//  Cases

export const retentionCases: CaseSource[] = [
	of("go-zero", "go", [
		"glm-5",
		"glm-",
		"glm-*",
		"glm-5.3-flash",
		"kimi-k3",
		"minimax-m3",
		"qwen",
		"qwen3",
		"qwen3-coder",
		"gemini-4-argon",
		"hy-1",
		"longcat-flash",
		"mimo-v2.6-flash",
		"step-3",
	]),
	of("go-dated", "go", ["deepseek-v4", "deepseek-v4.1-flash", "deepseek-", "deepseek-*"]),
	of("go-kept", "go", [
		"grok-4.7",
		"gpt-6-luna",
		"gpt-5.6-luna",
		"claude-haiku-5-5",
		"beam-free",
		"muse-spark-1",
		"muse-spark-",
		"iceman",
		"iceman-2",
	]),
	of("go-unlisted", "go", [
		"gemini-4-argon-2",
		"gemini-3.8-flash",
		"gemini-4",
		"grok-4.8",
		"grok-4",
		"gpt-6-sol",
		"gpt-6-luna-2",
		"claude-sonnet-5-5",
		"claude-haiku-5-4",
		"beam",
		"beam-free-2",
		"muse-spark",
		"mistral-large",
		...NOBODY,
	]),
	of("payg-zero", "payg", [
		"glm-5",
		"kimi-k3",
		"minimax-m3",
		"qwen",
		"qwen3-coder",
		"gemini-4-argon",
		"gemini-3.8-flash",
		"gemini-",
		"hy-1",
		"longcat-flash",
		"mimo-v2.6-flash",
		"step-3",
	]),
	of("payg-dated", "payg", ["deepseek-v4", "deepseek-v4.1-flash", "deepseek-"]),
	of("payg-kept", "payg", [
		"gpt-6-luna",
		"gpt-",
		"claude-opus-5-5",
		"claude-",
		"mistral-large",
		"mistral-small-2603",
		"grok-4.7",
		"grok-5",
		"beam",
		"beam-free",
		"beam-anything",
	]),
	of("payg-unlisted", "payg", ["gemini", "gpt", "claude", "mistral", "grok", "muse-spark-1", "iceman", ...NOBODY]),

	// Zero retention by the clock: the dated rows end with their last day, UTC.
	zdr("go-undated-any-time", [
		["go", "glm-5", 0],
		["go", "glm-5", END],
		["go", "glm-5", END * 10],
		["go", "kimi-k3", -1],
		["go", "gemini-4-argon", END + 1],
	]),
	zdr("go-dated-boundary", [
		["go", "deepseek-v4", 0],
		["go", "deepseek-v4", END - 2],
		["go", "deepseek-v4", END - 1],
		["go", "deepseek-v4", END],
		["go", "deepseek-v4", END + 1],
		["go", "deepseek-v4", 1795996800000],
		["go", "deepseek-v4", -1],
	]),
	zdr("payg-dated-boundary", [
		["payg", "deepseek-v4.1-flash", END - 1],
		["payg", "deepseek-v4.1-flash", END],
		["payg", "deepseek-", END - 1],
		["payg", "deepseek-", END],
	]),
	zdr("kept-never", [
		["go", "grok-4.7", 0],
		["go", "gpt-6-luna", END],
		["go", "claude-haiku-5-5", END * 10],
		["payg", "gpt-6-luna", 0],
		["payg", "claude-opus-5-5", END],
		["payg", "grok-5", 0],
		["payg", "mistral-large", 0],
		["payg", "beam-x", 0],
	]),
	zdr("unlisted-never", [
		["go", "gemini-3.8-flash", 0],
		["go", "", 0],
		["go", "constructor", 0],
		["go", "__proto__", 0],
		["go", "toString", END],
		["payg", "unknown-model", 0],
		["payg", "", 0],
		["payg", "constructor", 0],
		["payg", "valueOf", 0],
	]),
	zdr("plans-differ", [
		["go", "gemini-3.8-flash", 0],
		["payg", "gemini-3.8-flash", 0],
		["go", "gemini-4-argon-2", 0],
		["payg", "gemini-4-argon-2", 0],
		["go", "grok-4.8", 0],
		["payg", "grok-4.8", 0],
		["go", "gpt-6-sol", 0],
		["payg", "gpt-6-sol", 0],
	]),
];
