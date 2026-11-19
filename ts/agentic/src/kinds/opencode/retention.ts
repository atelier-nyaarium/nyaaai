// OpenCode's zero-retention table by plan.

import type { ProviderPlan } from "@nyaaai/contract/shapes/provider";
import { parseIsoTime } from "@nyaaai/contract/time";

////////////////////////////////
//  Interfaces & Types

export interface Retention {
	zdr: boolean;
	/** Last zero-retention day, UTC. */
	until?: string;
}

////////////////////////////////
//  Constants

const ZERO: Retention = { zdr: true };

/** 30-day or longer retention. */
const KEPT: Retention = { zdr: false };

const DEEPSEEK: Retention = { zdr: true, until: "2026-11-30" };

const DAY_MS = 24 * 60 * 60 * 1000;

/** Unlisted IDs: non-ZDR. */
const OPENCODE_ZDR: Record<ProviderPlan, Readonly<Record<string, Retention>>> = {
	go: {
		"glm-*": ZERO,
		"kimi-*": ZERO,
		"minimax-*": ZERO,
		"qwen*": ZERO,
		"gemini-4-argon": ZERO,
		"hy-*": ZERO,
		"longcat-*": ZERO,
		"mimo-*": ZERO,
		"step-*": ZERO,
		"deepseek-*": DEEPSEEK,
		"grok-4.7": KEPT,
		"gpt-6-luna": KEPT,
		"gpt-5.6-luna": KEPT,
		"claude-haiku-5-5": KEPT,
		"beam-free": KEPT,
		"muse-spark-*": KEPT,
		"iceman*": KEPT,
	},
	payg: {
		"glm-*": ZERO,
		"kimi-*": ZERO,
		"minimax-*": ZERO,
		"qwen*": ZERO,
		"gemini-*": ZERO,
		"hy-*": ZERO,
		"longcat-*": ZERO,
		"mimo-*": ZERO,
		"step-*": ZERO,
		"deepseek-*": DEEPSEEK,
		"gpt-*": KEPT,
		"claude-*": KEPT,
		"mistral-*": KEPT,
		"grok-*": KEPT,
		"beam*": KEPT,
	},
};

////////////////////////////////
//  Functions & Helpers

/** Exact, then longest prefix. */
export function retentionOf(plan: ProviderPlan, id: string): Retention | undefined {
	const table = OPENCODE_ZDR[plan];
	if (Object.hasOwn(table, id)) return table[id];
	const prefixes = Object.keys(table)
		.filter((key) => key.endsWith("*") && id.startsWith(key.slice(0, -1)))
		.sort((left, right) => right.length - left.length);
	return prefixes[0] === undefined ? undefined : table[prefixes[0]];
}

export function isZdr(plan: ProviderPlan, id: string, now: number): boolean {
	const retention = retentionOf(plan, id);
	if (retention?.zdr !== true) return false;
	return retention.until === undefined || now < parseIsoTime(`${retention.until}T00:00:00Z`) + DAY_MS;
}
