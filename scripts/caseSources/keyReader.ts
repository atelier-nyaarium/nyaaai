// One entry's key reader on the timer wheel. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { caseOf } from "./catalogKit";

////////////////////////////////
//  Helpers

const START = 5_000_000;

type Step = Record<string, unknown>;

const read: Step = { kind: "read" };
const answer = (check: number, state: Step): Step => ({ kind: "answer", check, state });
const fail = (check: number, message = "down"): Step => ({ kind: "fail", check, message });
const advance = (ms: number): Step => ({ kind: "advance", ms });
const dispose: Step = { kind: "dispose" };

const OK: Step = { kind: "ok", usage: 4.12, limit: 20 };
const REJECTED: Step = { kind: "rejected" };

const run = (name: string, steps: Step[], extra: Step = {}): CaseSource =>
	caseOf("keyReader.run")(name, { startMs: START, steps, ...extra });

////////////////////////////////
//  Cases

export const keyReaderCases: CaseSource[] = [
	run("a-read-shows-checking-then-the-state", [read, answer(1, OK)]),
	run("a-rejected-state-is-shown-as-it-is", [read, answer(1, REJECTED)]),
	run("every-state-a-check-may-answer", [
		read,
		answer(1, { kind: "none" }),
		read,
		answer(2, { kind: "unreachable" }),
		read,
		answer(3, { kind: "invalid" }),
		read,
		answer(4, { kind: "ok" }),
	]),
	run("a-newer-read-drops-the-last", [read, read, answer(1, OK), answer(2, REJECTED)]),
	run("a-dropped-check-that-fails-shows-nothing", [read, read, fail(1), answer(2, OK)]),
	run("a-newer-read-ends-the-last-checks-signal", [read, read, read]),
	run("only-the-newest-of-three-reads-is-shown", [
		read,
		read,
		read,
		answer(3, OK),
		answer(2, REJECTED),
		answer(1, REJECTED),
	]),
	run("a-check-that-fails-shows-unreachable", [read, fail(1)]),
	run("a-check-that-outlives-its-deadline-shows-unreachable", [read, advance(29_999), advance(1), answer(1, OK)]),
	run("a-check-answering-just-before-the-deadline-is-shown", [read, advance(29_999), answer(1, OK), advance(1)]),
	run("the-deadline-leaves-no-timer-after-an-answer", [read, answer(1, OK), advance(60_000)]),
	run("the-deadline-leaves-no-timer-after-a-failure", [read, fail(1), advance(60_000)]),
	run("a-read-after-a-deadline-gets-a-fresh-deadline", [read, advance(30_000), read, advance(29_999), advance(1)]),
	run("a-newer-read-takes-a-fresh-deadline", [read, advance(20_000), read, advance(20_000), advance(10_000)]),
	run("a-dispose-during-a-check-shows-nothing", [read, dispose, answer(1, OK)]),
	run("a-dispose-after-the-answer-changes-nothing", [read, answer(1, OK), dispose]),
	run("a-dispose-clears-the-deadline-timer", [read, dispose, advance(60_000)]),
	run("a-read-after-a-dispose-works", [read, dispose, read, answer(2, OK)]),
	run("a-dispose-twice", [read, dispose, dispose]),
	run("a-dispose-with-no-read", [dispose]),
	run("an-answer-that-comes-after-the-deadline-is-dropped", [
		read,
		advance(31_000),
		answer(1, OK),
		read,
		answer(2, OK),
	]),
	run(
		"a-check-that-ignores-its-signal-shows-unreachable-at-the-deadline",
		[read, advance(29_999), advance(1), answer(1, OK)],
		{
			checks: "ignoreSignal",
		},
	),
	run(
		"a-check-that-ignores-its-signal-is-not-busy-after-the-deadline",
		[read, advance(30_000), read, answer(2, OK)],
		{
			checks: "ignoreSignal",
		},
	),
	run(
		"a-dropped-check-that-ignores-its-signal-shows-nothing-later",
		[read, read, answer(1, OK), answer(2, REJECTED)],
		{
			checks: "ignoreSignal",
		},
	),
	run("a-check-that-throws-at-once-shows-unreachable-and-leaves-no-timer", [read, advance(60_000)], {
		checks: "throwAtOnce",
	}),
];
