// Cases for the scripted ports and the deadline.

import type { CaseSource } from "../caseAdapter";

export const testkitPortCases: CaseSource[] = [
	{
		kind: "testkit.entryKeys",
		name: "order",
		input: {
			steps: [
				{ op: "get", id: "e1" },
				{ op: "listen", name: "a" },
				{ op: "listen", name: "b" },
				{ op: "listen", name: "c" },
				{ op: "set", id: "e1", answer: "key", key: "key-one" },
				{ op: "get", id: "e1" },
				{ op: "unlisten", name: "b" },
				{ op: "set", id: "e2", answer: "refused" },
				{ op: "get", id: "e2" },
				{ op: "unlisten", name: "b" },
				{ op: "set", id: "e1", answer: "unready" },
				{ op: "get", id: "e1" },
				{ op: "unlisten", name: "a" },
				{ op: "unlisten", name: "c" },
				{ op: "set", id: "e1", answer: "none" },
				{ op: "get", id: "e1" },
				{ op: "get", id: "never" },
			],
		},
	},
	{
		kind: "testkit.providerList",
		name: "order",
		input: {
			steps: [
				{ op: "read" },
				{ op: "listen", name: "a" },
				{ op: "listen", name: "b" },
				{ op: "set", ids: ["x", "y", "z"] },
				{ op: "read" },
				{ op: "unlisten", name: "a" },
				{ op: "set", ids: ["z", "x"] },
				{ op: "read" },
				{ op: "unlisten", name: "a" },
				{ op: "unlisten", name: "b" },
				{ op: "set", ids: [] },
				{ op: "read" },
			],
		},
	},
	{
		kind: "testkit.modelRegistry",
		name: "writes",
		input: {
			steps: [
				{ op: "read" },
				{ op: "listen", name: "a" },
				{ op: "listen", name: "b" },
				{ op: "setRows", keys: ["m1", "m2"] },
				{ op: "setHidden", keys: ["m2", "fam-*"] },
				{ op: "read" },
				{ op: "record", entry: "e1", id: "openrouter:v/m", label: "first" },
				{ op: "record", entry: "e1", id: "openrouter:v/n", label: "second" },
				{ op: "record", entry: "e2", id: "openrouter:v/m", label: "third" },
				{ op: "read" },
				{ op: "unlisten", name: "a" },
				{ op: "record", entry: "e1", id: "openrouter:v/m", label: "again" },
				{ op: "refuse", why: "settings are read only" },
				{ op: "record", entry: "e1", id: "openrouter:v/q", label: "never" },
				{ op: "read" },
				{ op: "allow" },
				{ op: "record", entry: "e1", id: "openrouter:v/q", label: "later" },
				{ op: "read" },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "deadline-first",
		input: {
			startMs: 100,
			steps: [
				{ op: "deadline", id: "d", ms: 50 },
				{ op: "advance", ms: 49 },
				{ op: "advance", ms: 1 },
				{ op: "advance", ms: 100 },
				{ op: "clear", id: "d" },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "caller-first",
		input: {
			startMs: 0,
			steps: [
				{ op: "caller", id: "c" },
				{ op: "deadline", id: "d", ms: 50, caller: "c" },
				{ op: "abortAt", id: "c", ms: 20 },
				{ op: "advance", ms: 20 },
				{ op: "advance", ms: 100 },
				{ op: "clear", id: "d" },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "same-instant-deadline-scheduled-first",
		input: {
			startMs: 0,
			steps: [
				{ op: "caller", id: "c" },
				{ op: "deadline", id: "d", ms: 50, caller: "c" },
				{ op: "abortAt", id: "c", ms: 50 },
				{ op: "advance", ms: 50 },
				{ op: "clear", id: "d" },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "same-instant-caller-scheduled-first",
		input: {
			startMs: 0,
			steps: [
				{ op: "caller", id: "c" },
				{ op: "abortAt", id: "c", ms: 50 },
				{ op: "deadline", id: "d", ms: 50, caller: "c" },
				{ op: "advance", ms: 50 },
				{ op: "clear", id: "d" },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "cleared-first",
		input: {
			startMs: 0,
			steps: [
				{ op: "deadline", id: "d", ms: 50 },
				{ op: "caller", id: "c" },
				{ op: "deadline", id: "e", ms: 70, caller: "c" },
				{ op: "clear", id: "d" },
				{ op: "clear", id: "e" },
				{ op: "advance", ms: 200 },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "caller-already-aborted",
		input: {
			startMs: 0,
			steps: [
				{ op: "caller", id: "c" },
				{ op: "abortCaller", id: "c" },
				{ op: "deadline", id: "d", ms: 50, caller: "c" },
				{ op: "advance", ms: 100 },
				{ op: "clear", id: "d" },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "two-deadlines-one-caller",
		input: {
			startMs: 0,
			steps: [
				{ op: "caller", id: "c" },
				{ op: "deadline", id: "short", ms: 10, caller: "c" },
				{ op: "deadline", id: "long", ms: 90, caller: "c" },
				{ op: "advance", ms: 10 },
				{ op: "abortCaller", id: "c" },
				{ op: "advance", ms: 100 },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "clear-then-caller-aborts",
		input: {
			startMs: 0,
			steps: [
				{ op: "caller", id: "c" },
				{ op: "deadline", id: "d", ms: 50, caller: "c" },
				{ op: "clear", id: "d" },
				{ op: "abortCaller", id: "c" },
				{ op: "advance", ms: 100 },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "caller-aborts-then-clear",
		input: {
			startMs: 0,
			steps: [
				{ op: "caller", id: "c" },
				{ op: "deadline", id: "d", ms: 50, caller: "c" },
				{ op: "abortCaller", id: "c" },
				{ op: "clear", id: "d" },
				{ op: "advance", ms: 100 },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "deadline-fires-then-caller-aborts",
		input: {
			startMs: 0,
			steps: [
				{ op: "caller", id: "c" },
				{ op: "deadline", id: "d", ms: 50, caller: "c" },
				{ op: "advance", ms: 50 },
				{ op: "abortCaller", id: "c" },
				{ op: "clear", id: "d" },
				{ op: "advance", ms: 100 },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "clear-twice",
		input: {
			startMs: 0,
			steps: [
				{ op: "caller", id: "c" },
				{ op: "deadline", id: "d", ms: 50, caller: "c" },
				{ op: "clear", id: "d" },
				{ op: "clear", id: "d" },
				{ op: "abortCaller", id: "c" },
				{ op: "clear", id: "d" },
				{ op: "advance", ms: 100 },
			],
		},
	},
	{
		kind: "testkit.deadline",
		name: "clear-after-caller-abort-twice",
		input: {
			startMs: 0,
			steps: [
				{ op: "caller", id: "c" },
				{ op: "deadline", id: "d", ms: 50, caller: "c" },
				{ op: "abortCaller", id: "c" },
				{ op: "clear", id: "d" },
				{ op: "clear", id: "d" },
			],
		},
	},
	{
		kind: "testkit.entryKeys",
		name: "listener-unsubscribes-another",
		input: {
			steps: [
				{ op: "listen", name: "a", unlistens: "b" },
				{ op: "listen", name: "b" },
				{ op: "listen", name: "c" },
				{ op: "set", id: "e1", answer: "none" },
				{ op: "set", id: "e1", answer: "refused" },
				{ op: "listen", name: "d", unlistens: "a" },
				{ op: "set", id: "e1", answer: "unready" },
				{ op: "set", id: "e1", answer: "none" },
			],
		},
	},
	{
		kind: "testkit.providerList",
		name: "listener-unsubscribes-another",
		input: {
			steps: [
				{ op: "listen", name: "a", unlistens: "b" },
				{ op: "listen", name: "b" },
				{ op: "listen", name: "c", unlistens: "c" },
				{ op: "listen", name: "d" },
				{ op: "set", ids: ["x"] },
				{ op: "set", ids: ["x", "y"] },
				{ op: "unlisten", name: "b" },
			],
		},
	},
	{
		kind: "testkit.modelRegistry",
		name: "listener-unsubscribes-another",
		input: {
			steps: [
				{ op: "listen", name: "a", unlistens: "b" },
				{ op: "listen", name: "b" },
				{ op: "listen", name: "c" },
				{ op: "setRows", keys: ["r1"] },
				{ op: "setRows", keys: ["r1", "r2"] },
				{ op: "record", entry: "e1", id: "openrouter:a/b", label: "L" },
			],
		},
	},
];
