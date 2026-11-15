// Scripted adapter runs.

import { CONTRACT_VERSION } from "@nyaaai/contract/shape";
import { CLI_VERSION, compactedSession } from "@nyaaai/testkit/claudeSession";
import type { RunSource, Step } from "../runScript";

////////////////////////////////
//  Constants

const NOW_MS = 1_794_535_200_000;
const SOURCE = "$TMP/sessions/source.jsonl";

////////////////////////////////
//  Functions & Helpers

const hello: Step = { send: { type: "hello", version: CONTRACT_VERSION } };
const welcomed: Step[] = [hello, { await: { type: "welcome" } }];

const run = (name: string, script: Step[], more: Partial<RunSource> = {}): RunSource => ({
	type: "run",
	kind: "adapter.run",
	name,
	script,
	...more,
});

const fixed = (idPrefix: string) => ({ nowMs: NOW_MS, stepMs: 1000, idPrefix });

const converse = (id: string, requests: number, together = false): Step => ({
	send: {
		type: "start",
		run: id,
		command: "testkit.converse",
		input: { requests, together },
		fixed: fixed(`${id}-`),
	},
});

const asked = (id: string, request: string): Step => ({ await: { type: "request", run: id, id: request } });
const ended = (id: string): Step => ({
	await: [
		{ type: "done", run: id },
		{ type: "failed", run: id },
	],
});

function cutRuns(): RunSource[] {
	const { session, second } = compactedSession();
	const files = { "sessions/source.jsonl": session.lines };
	const cut = (name: string, script: Step[], read: string[] = []) =>
		run(name, [...welcomed, ...script], { kind: "sessionCut.run", files, read });
	const apply = (id: string, request: object, more: object = {}): Step => ({
		send: {
			type: "start",
			run: id,
			command: "sessionCut.apply",
			input: { source: SOURCE, out: "$TMP/built", request, cliVersion: CLI_VERSION, ...more },
			fixed: fixed(`${id}-`),
		},
	});
	const compact = { mode: "compact", targetUuid: second };
	return [
		cut("plan", [
			{
				send: {
					type: "start",
					run: "plan",
					command: "sessionCut.plan",
					input: { source: SOURCE, request: { mode: "trim" }, cliVersion: CLI_VERSION },
				},
			},
			ended("plan"),
			{
				send: {
					type: "start",
					run: "refused",
					command: "sessionCut.plan",
					input: { source: SOURCE, request: { mode: "trim" }, cliVersion: "0.0.1" },
				},
			},
			ended("refused"),
		]),
		cut("apply-trim", [apply("trim", { mode: "trim" }), ended("trim")], ["built/trim-1.jsonl"]),
		cut(
			"apply-clear",
			[apply("clear", { mode: "clear", targetUuid: second }), ended("clear")],
			["built/clear-1.jsonl"],
		),
		cut(
			"apply-compact-asks-the-host",
			[
				apply("compact", compact),
				asked("compact", "q1"),
				{ send: { type: "steer", run: "compact", text: "ignored by a cut" } },
				{ send: { type: "reply", run: "compact", id: "q1", value: "  what the host folded  " } },
				ended("compact"),
			],
			["built/compact-1.jsonl"],
		),
		cut("apply-compact-handed", [apply("handed", { ...compact, summaryText: "handed over" }), ended("handed")]),
		cut("apply-compact-host-fails", [
			apply("failed", compact),
			asked("failed", "q1"),
			{ send: { type: "reply-failed", run: "failed", id: "q1", message: "the session would not summarize" } },
			ended("failed"),
			apply("wrong", compact),
			asked("wrong", "q1"),
			{ send: { type: "reply", run: "wrong", id: "q1", value: { text: "not text" } } },
			ended("wrong"),
			apply("blank", compact),
			asked("blank", "q1"),
			{ send: { type: "reply", run: "blank", id: "q1", value: "   " } },
			ended("blank"),
			apply("stopped", compact),
			asked("stopped", "q1"),
			{ send: { type: "interrupt", run: "stopped" } },
			ended("stopped"),
		]),
		cut("apply-bad-input", [
			{ send: { type: "start", run: "bad", command: "sessionCut.apply", input: { source: SOURCE } } },
			ended("bad"),
			apply("mode", { mode: "shrink" }),
			ended("mode"),
		]),
	];
}

////////////////////////////////
//  Cases

export const adapterCases: RunSource[] = [
	run("handshake", [hello, { await: { type: "welcome" } }, { close: true }]),
	run("reject-another-major", [{ send: { type: "hello", version: "9.0.0" } }, { await: { type: "reject" } }]),
	run("reject-another-minor", [{ send: { type: "hello", version: "0.99.0" } }, { await: { type: "reject" } }]),
	run("reject-no-version", [{ send: { type: "hello", version: "latest" } }, { await: { type: "reject" } }]),
	run("welcome-another-patch", [
		{ send: { type: "hello", version: CONTRACT_VERSION.replace(/\d+$/, "0977") } },
		{ await: { type: "welcome" } },
	]),
	run("not-greeted", [
		converse("early", 0),
		{ await: { type: "fault", code: "not_greeted" } },
		{ sendText: "not json" },
		hello,
		{ await: { type: "welcome" } },
		hello,
		{ await: { type: "fault", message: "greeted twice" } },
	]),
	run("bad-frames", [
		...welcomed,
		{ sendText: "not json" },
		{ sendText: "" },
		{ sendText: "   " },
		{ sendText: "[]" },
		{ sendText: '{"type":"start"}' },
		{ sendText: '{"type":"nope","run":"x"}' },
		{ sendText: '{"type":"steer","run":"x","text":5}' },
		{ sendText: '{"type":"start","run":"x","command":"c","input":1,"fixed":{"nowMs":1.5,"idPrefix":""}}' },
		{ sendText: '\u{FEFF}{"type":"interrupt","run":"bom"}' },
		{ sendText: '{"type":"interrupt","run":"cr"}\r' },
		{ sendText: '  {"type":"interrupt","run":"spaced","extra":[1]}  ' },
		{ sendBase64: Buffer.from([0x7b, 0x22, 0x74, 0xff, 0xfe]).toString("base64") },
		{
			sendBase64: Buffer.concat([
				Buffer.from('{"type":"interrupt","run":"bytes '),
				Buffer.from([0xff, 0xe2, 0x82]),
				Buffer.from('"}'),
			]).toString("base64"),
		},
		{ sendText: '{"type":"interrupt","run":"lone \\ud83d"}' },
		{ sendText: `{"type":"interrupt","run":"deep","extra":${"[".repeat(1100)}${"]".repeat(1100)}}` },
		{ await: { type: "fault", code: "bad_frame", message: "not JSON" } },
	]),
	run("oversized-frame", [
		...welcomed,
		{ sendOverCap: 1 },
		{ await: { type: "fault", code: "oversized_frame" } },
		{ sendOverCap: 0 },
		{ await: { type: "fault", code: "bad_frame" } },
		converse("after", 0),
		ended("after"),
	]),
	run("commands", [
		...welcomed,
		{ send: { type: "start", run: "none", command: "no.such", input: null } },
		ended("none"),
		{ send: { type: "start", run: "bad", command: "testkit.converse", input: { requests: -1, together: false } } },
		ended("bad"),
		{ send: { type: "start", run: "crash", command: "testkit.crash", input: null } },
		ended("crash"),
		{ send: { type: "start", run: "pure", command: "case:json.canonical", input: { texts: ['{"b":1,"a":2}'] } } },
		ended("pure"),
		{ send: { type: "steer", run: "pure", text: "too late" } },
		{ await: { type: "fault", code: "unknown_run" } },
	]),
	run("steer-and-interrupt-while-asking", [
		...welcomed,
		converse("talk", 2),
		asked("talk", "q1"),
		{ send: { type: "steer", run: "talk", text: "first" } },
		{ await: { type: "event", run: "talk", seq: 1 } },
		{ send: { type: "reply", run: "talk", id: "q9", value: 0 } },
		{ await: { type: "fault", code: "unknown_reply" } },
		{ send: { type: "reply", run: "talk", id: "q1", value: { answered: [1, null] } } },
		asked("talk", "q2"),
		{ send: { type: "reply", run: "talk", id: "q1", value: "twice" } },
		{ await: { type: "fault", code: "unknown_reply" } },
		{ send: { type: "steer", run: "talk", text: "second" } },
		{ await: { type: "event", run: "talk", seq: 3 } },
		{ send: { type: "interrupt", run: "talk" } },
		ended("talk"),
	]),
	run("replies", [
		...welcomed,
		converse("told", 3),
		asked("told", "q1"),
		{ send: { type: "reply-failed", run: "told", id: "q1", message: "the host has no key" } },
		asked("told", "q2"),
		{ send: { type: "reply", run: "told", id: "q2", value: null } },
		asked("told", "q3"),
		{ send: { type: "reply", run: "told", id: "q3", value: "last" } },
		ended("told"),
	]),
	run("replies-out-of-order", [
		...welcomed,
		converse("many", 3, true),
		asked("many", "q1"),
		asked("many", "q2"),
		asked("many", "q3"),
		{ send: { type: "reply", run: "many", id: "q3", value: "third" } },
		{ send: { type: "steer", run: "many", text: "between" } },
		{ send: { type: "reply-failed", run: "many", id: "q2", message: "the host has no key" } },
		{ send: { type: "reply", run: "many", id: "q3", value: "twice" } },
		{ await: { type: "fault", code: "unknown_reply" } },
		{ send: { type: "reply", run: "many", id: "q1", value: { first: [1] } } },
		ended("many"),
		converse("cut", 2, true),
		asked("cut", "q2"),
		{ send: { type: "reply", run: "cut", id: "q2", value: "kept" } },
		{ send: { type: "interrupt", run: "cut" } },
		ended("cut"),
		{ send: { type: "reply", run: "cut", id: "q1", value: "too late" } },
		{ await: { type: "fault", code: "unknown_run" } },
	]),
	run("interleaved-runs", [
		...welcomed,
		converse("a", 1),
		converse("b", 1),
		converse("a", 1),
		{ await: { type: "fault", code: "duplicate_run" } },
		asked("a", "q1"),
		asked("b", "q1"),
		{ send: { type: "steer", run: "b", text: "for b" } },
		{ await: { type: "event", run: "b", seq: 1 } },
		{ send: { type: "reply", run: "b", id: "q1", value: "b first" } },
		ended("b"),
		{ send: { type: "reply", run: "a", id: "q1", value: "a second" } },
		ended("a"),
		converse("a", 0),
		ended("a"),
	]),
	run("shutdown-interrupts-open-runs", [
		...welcomed,
		converse("open", 1),
		converse("also", 2),
		asked("open", "q1"),
		asked("also", "q1"),
		{ send: { type: "reply", run: "also", id: "q1", value: 1 } },
		asked("also", "q2"),
		{ close: true },
	]),
	run("unterminated-tail", [
		...welcomed,
		converse("kept", 0),
		ended("kept"),
		{ sendTail: '{"type":"start","run":"tail","command":"case:json.canonical","input":{"texts":[]}}' },
	]),
	...cutRuns(),
];
