// Transport cases: the presets bound each call, and a server's time is clamped at the exit.

import type { CaseSource } from "../caseAdapter";
import { opencode } from "./codecFixtures";
import {
	commentFrame,
	DONE_TEXT,
	exchange,
	finish,
	frame,
	getCall,
	JSON_HEAD,
	jsonReply,
	NOW,
	piece,
	sseReply,
	streamCall,
	text,
} from "./transportKit";

////////////////////////////////
//  Constants

const DAY_MS = 86_400_000;

const SLOW = { error: { code: 429, message: "Slow down" } };

/** OpenCode's pool limit with its reset. */
const poolLimit = (resetsAt: string) =>
	jsonReply({ error: { type: "GoUsageLimitError", message: "Limit.", resetsAt } }, 429);

////////////////////////////////
//  Cases

const budgets: CaseSource[] = [
	exchange(
		"budget-a-stream-of-comment-lines-only-ends-at-its-total-size",
		[streamCall()],
		[sseReply(Array.from({ length: 40 }, () => piece(commentFrame("ping"), 1)))],
		{ budgets: { stream: { maxBytes: 100 } } },
	),
	exchange(
		"budget-a-stream-of-small-events-ends-at-its-total-time",
		[streamCall()],
		[sseReply([...Array.from({ length: 10 }, () => piece(frame(text("a")), 30)), piece(DONE_TEXT, 30)])],
		{ budgets: { stream: { totalMs: 100 } } },
	),
	exchange(
		"budget-a-consumer-slow-to-take-events-is-not-cut-by-the-idle-clock",
		[streamCall({ takeMs: 100 })],
		[sseReply([piece(frame(text("Hi")), 5), piece(frame(finish("stop")), 5), piece(DONE_TEXT, 5)])],
		{ budgets: { stream: { idleMs: 30 } } },
	),
	exchange(
		"budget-a-total-that-passes-while-events-are-taken-lets-the-held-chunk-drain",
		[streamCall({ takeMs: 50 })],
		[
			sseReply([
				piece(`${frame(text("a"))}${frame(text("b"))}${frame(text("c"))}`, 10),
				piece(frame(finish("stop")), 10),
				piece(DONE_TEXT, 10),
			]),
		],
		{ budgets: { stream: { totalMs: 100 } } },
	),
	exchange(
		"budget-a-get-json-whose-body-drips-ends-at-its-span",
		[getCall("/models")],
		[{ status: 200, headers: JSON_HEAD, chunks: Array.from({ length: 6 }, () => piece(" ", 30)) }],
		{ budgets: { catalog: { totalMs: 100, idleMs: 50 } } },
	),
];

const clamps: CaseSource[] = [
	exchange(
		"clamp-a-retry-after-of-a-thousand-days",
		[streamCall()],
		[jsonReply(SLOW, 429, [["retry-after", String(1000 * 86_400)]])],
	),
	exchange("clamp-a-reset-ten-years-away", [streamCall()], [poolLimit("2036-11-07T17:00:00Z")], {
		codec: opencode({ nowMs: NOW }),
	}),
	exchange(
		"clamp-a-codec-wait-ten-years-away",
		[streamCall()],
		[jsonReply({ error: { message: "Slow.", resetsAt: NOW / 1000 + 10 * 365 * 86_400 } }, 429)],
		{ codec: opencode({ nowMs: NOW }) },
	),
	exchange("clamp-a-reset-in-the-past-stays", [streamCall()], [poolLimit("2020-01-01T00:00:00Z")], {
		codec: opencode({ nowMs: NOW }),
	}),
	exchange(
		"clamp-a-wait-of-just-thirty-one-days-stays",
		[streamCall()],
		[jsonReply(SLOW, 429, [["retry-after", String((31 * DAY_MS) / 1000)]])],
	),
];

export const budgetTransportCases: CaseSource[] = [...budgets, ...clamps];
