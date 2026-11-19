// Cases for the scripted ports and the deadline.

import type { CaseSource } from "../caseAdapter";

/** 2026-11-15T00:00:00Z. */
const MIDNIGHT = 1_794_700_800_000;
const HOUR = 3_600_000;
const MINUTE = 60_000;

/** Asks one instant either side of each edge. */
const around = (...edges: number[]): number[] => edges.flatMap((edge) => [edge - 1, edge, edge + 1]);

const offsetCase = (name: string, clock: string, offsets: object[], asks: number[]): CaseSource => ({
	kind: "testkit.timeOffset",
	name,
	input: { clock, startMs: 0, offsets, asks },
});

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
				{ op: "set", id: "e1", answer: "key", key: "fake-one" },
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
	offsetCase("none-set-stepped", "stepped", [], around(0, MIDNIGHT)),
	offsetCase("none-set-wheel", "wheel", [], around(0, MIDNIGHT)),
	offsetCase("none-set-scheduler", "scheduler", [], around(0, MIDNIGHT)),
	offsetCase(
		"before-the-first",
		"stepped",
		[{ fromMs: MIDNIGHT, minutes: 60 }],
		[-1, 0, MIDNIGHT - 1, MIDNIGHT, MIDNIGHT + 1],
	),
	offsetCase(
		"last-at-or-before-wins",
		"wheel",
		[
			{ fromMs: 100, minutes: 10 },
			{ fromMs: 200, minutes: -30 },
		],
		[99, 100, 101, 199, 200, 201, 1000],
	),
	offsetCase(
		"a-later-entry-wins-a-tie",
		"scheduler",
		[
			{ fromMs: 100, minutes: 10 },
			{ fromMs: 50, minutes: 5 },
			{ fromMs: 100, minutes: 20 },
			{ fromMs: 200, minutes: -30 },
		],
		[49, 50, 99, 100, 150, 199, 200, 1000],
	),
	offsetCase(
		"out-of-order",
		"stepped",
		[
			{ fromMs: 200, minutes: 7 },
			{ fromMs: 100, minutes: 3 },
		],
		[99, 100, 150, 199, 200, 250],
	),
	offsetCase("midnight-east-of-utc", "stepped", [{ fromMs: -HOUR * 24 * 365, minutes: 60 }], around(MIDNIGHT - HOUR)),
	offsetCase("midnight-west-of-utc", "wheel", [{ fromMs: 0, minutes: -300 }], around(MIDNIGHT + 5 * HOUR)),
	offsetCase("half-hour-zone", "stepped", [{ fromMs: 0, minutes: 330 }], around(MIDNIGHT - 330 * MINUTE)),
	offsetCase("quarter-hour-zone", "scheduler", [{ fromMs: 0, minutes: 345 }], around(MIDNIGHT - 345 * MINUTE)),
	offsetCase("half-hour-zone-west", "wheel", [{ fromMs: 0, minutes: -210 }], around(MIDNIGHT + 210 * MINUTE)),
	offsetCase(
		"change-forward-across-midnight",
		"stepped",
		[
			{ fromMs: 0, minutes: 0 },
			{ fromMs: MIDNIGHT, minutes: 120 },
		],
		around(MIDNIGHT),
	),
	offsetCase(
		"change-back-across-midnight",
		"wheel",
		[
			{ fromMs: 0, minutes: 120 },
			{ fromMs: MIDNIGHT, minutes: 0 },
		],
		around(MIDNIGHT - 2 * HOUR).concat(around(MIDNIGHT)),
	),
	offsetCase(
		"spring-forward",
		"scheduler",
		[
			{ fromMs: 0, minutes: -300 },
			{ fromMs: 1_772_953_200_000, minutes: -240 },
			{ fromMs: 1_793_513_999_000, minutes: -300 },
		],
		around(1_772_953_200_000, 1_793_513_999_000),
	),
	{
		kind: "testkit.usageSink",
		name: "ids-in-order",
		input: {
			steps: [
				{ op: "record", model: "m1", at: 10 },
				{ op: "ids", ids: ["id-1", "id-2"] },
				{ op: "record", model: "m2", at: 20, generationId: "gen-a1" },
				{ op: "record", model: "m3", at: 30 },
				{ op: "record", model: "m4", at: 40 },
				{ op: "ids", ids: ["id-3", "id-4", "id-5"] },
				{ op: "record", model: "m5", at: 50 },
				{ op: "ids", ids: [] },
				{ op: "record", model: "m6", at: 60 },
			],
		},
	},
	{
		kind: "testkit.usageSink",
		name: "none-on-demand",
		input: {
			steps: [
				{ op: "ids", ids: ["id-1", "id-2", "id-3"] },
				{ op: "none", on: true },
				{ op: "record", model: "m1", at: 10 },
				{ op: "record", model: "m2", at: 20, generationId: "gen-a1" },
				{ op: "none", on: false },
				{ op: "record", model: "m3", at: 30 },
				{ op: "none", on: true },
				{ op: "none", on: false },
				{ op: "record", model: "m4", at: 40 },
				{ op: "record", model: "m5", at: 50 },
			],
		},
	},
	{
		kind: "testkit.usageSink",
		name: "fill-and-filled",
		input: {
			steps: [
				{ op: "ids", ids: ["id-1", "id-2", "id-3"] },
				{ op: "record", model: "m1", at: 10 },
				{ op: "record", model: "m2", at: 20 },
				{ op: "record", model: "m3", at: 30 },
				{ op: "filled", id: "id-1", at: 10 },
				{ op: "fill", id: "id-1", provider: "Lab" },
				{ op: "filled", id: "id-1", at: 10 },
				{ op: "filled", id: "id-2", at: 20 },
				{ op: "markFilled", ids: ["id-2"] },
				{ op: "filled", id: "id-2", at: 20 },
				{ op: "fill", id: "id-3", provider: "Lab" },
				{ op: "fill", id: "id-3", provider: "Other" },
				{ op: "filled", id: "id-3", at: 30 },
			],
		},
	},
	{
		kind: "testkit.usageSink",
		name: "a-wrong-id-fills-nothing",
		input: {
			steps: [
				{ op: "ids", ids: ["id-1"] },
				{ op: "record", model: "m1", at: 10 },
				{ op: "fill", id: "id-4", provider: "Lab" },
				{ op: "markFilled", ids: ["id-9"] },
				{ op: "filled", id: "id-4", at: 0 },
				{ op: "filled", id: "id-9", at: 0 },
				{ op: "filled", id: "id-1", at: 0 },
				{ op: "rows", rows: [{ id: "id-4", generationId: "gen-a4", at: 10 }] },
				{ op: "unfilled", at: 0 },
			],
		},
	},
	{
		kind: "testkit.usageSink",
		name: "an-older-call-is-not-filled-for-a-later-ask",
		input: {
			steps: [
				{ op: "ids", ids: ["id-1"] },
				{ op: "record", model: "m1", at: 100 },
				{ op: "fill", id: "id-1", provider: "Lab" },
				{ op: "filled", id: "id-1", at: 100 },
				{ op: "filled", id: "id-1", at: 101 },
				{ op: "filled", id: "id-1", at: 99 },
				{ op: "rows", rows: [{ id: "id-1", generationId: "gen-a1", at: 100 }] },
				{ op: "unfilled", at: 100 },
				{ op: "unfilled", at: 101 },
			],
		},
	},
	{
		kind: "testkit.usageSink",
		name: "a-call-edited-after-record",
		input: {
			steps: [
				{ op: "ids", ids: ["id-1", "id-2"] },
				{ op: "record", model: "m1", at: 10 },
				{ op: "edit", model: "changed" },
				{ op: "record", model: "m2", at: 20, generationId: "gen-a2" },
				{ op: "edit", model: "changed-too" },
			],
		},
	},
	{
		kind: "testkit.usageSink",
		name: "unfilled",
		input: {
			steps: [
				{ op: "ids", ids: ["id-1", "id-2", "id-3"] },
				{ op: "unfilled", at: 0 },
				{ op: "record", model: "m1", at: 10, generationId: "gen-a1" },
				{ op: "record", model: "m2", at: 20, generationId: "gen-a2" },
				{ op: "record", model: "m3", at: 30, generationId: "gen-a3" },
				{
					op: "rows",
					rows: [
						{ id: "id-1", generationId: "gen-a1", at: 10 },
						{ id: "id-2", generationId: "gen-a2", at: 20, entry: "e1" },
						{ id: "id-3", generationId: "gen-a3", at: 30 },
					],
				},
				{ op: "unfilled", at: 0 },
				{ op: "unfilled", at: 20 },
				{ op: "unfilled", at: 21 },
				{ op: "fill", id: "id-2", provider: "Lab" },
				{ op: "unfilled", at: 0 },
				{ op: "markFilled", ids: ["id-1"] },
				{ op: "unfilled", at: 0 },
				{ op: "rows", rows: [] },
				{ op: "unfilled", at: 0 },
			],
		},
	},
	{
		kind: "testkit.timeOffsetRun",
		name: "set-advance-set",
		input: {
			startMs: 1_794_700_000_000,
			offsets: [{ fromMs: 0, minutes: 60 }],
			steps: [
				{ op: "ask", atMs: MIDNIGHT },
				{ op: "advance", ms: 500_000 },
				{ op: "ask", atMs: MIDNIGHT },
				{ op: "set", offsets: [{ fromMs: 0, minutes: -300 }] },
				{ op: "ask", atMs: MIDNIGHT },
				{ op: "advance", ms: 1000 },
				{ op: "set", offsets: [{ fromMs: MIDNIGHT, minutes: 330 }] },
				{ op: "ask", atMs: MIDNIGHT - 1 },
				{ op: "ask", atMs: MIDNIGHT },
				{ op: "set", offsets: [] },
				{ op: "ask", atMs: MIDNIGHT },
			],
		},
	},
];
