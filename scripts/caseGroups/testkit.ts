import { existsSync } from "node:fs";
import path from "node:path";
import { BUDGETS } from "@nyaaai/contract/fetch";
import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import { sequenceRandom } from "@nyaaai/contract/random";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, kind } from "@nyaaai/testkit/caseGroup";
import { compactedSession } from "@nyaaai/testkit/claudeSession";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { readTrace, replayFetch } from "@nyaaai/testkit/trace";
import { testkitPortCases } from "../caseSources/testkitPorts";
import { portKinds } from "./testkitPorts";

////////////////////////////////
//  Shapes

/** Requests precede replies. */
export const TestkitConverseInput = z.object({ requests: z.int().min(0), together: z.boolean() });
export const TestkitConverseResult = z.object({
	ids: z.array(z.string()),
	nowMs: z.int(),
	/** Steers no event reported. */
	steers: z.array(z.string()),
});

/** The handler answers an empty text unless told it fits. */
export const TestkitMisfitInput = z.object({ fits: z.boolean() });
export const TestkitMisfitResult = z.object({ text: z.string().min(1) });

////////////////////////////////
//  Functions & Helpers

interface SchedulerStep {
	op: string;
	id?: string;
	ms?: number;
	tag?: string;
}

/** A settle after every step. */
const settled = (steps: SchedulerStep[]): SchedulerStep[] => steps.flatMap((step) => [step, { op: "settle" }]);

////////////////////////////////
//  Group

export const testkit = caseGroup("testkit", {
	shapes: { TestkitConverseInput, TestkitConverseResult, TestkitMisfitInput, TestkitMisfitResult },
	kinds: {
		...portKinds,
		compactedSession: kind(z.object({}), z.object({ lines: z.array(z.string()) }), () => ({
			lines: compactedSession().session.lines,
		})),
		wheel: kind(
			z.object({
				startMs: z.int(),
				steps: z.array(
					z.object({
						op: z.enum(["after", "cancel", "advance"]),
						ms: z.int().exactOptional(),
						tag: z.string().exactOptional(),
					}),
				),
			}),
			z.object({
				steps: z.array(
					z.object({
						nowMs: z.int(),
						waiting: z.array(z.string()),
						fired: z.array(z.object({ tag: z.string(), atMs: z.int() })),
						cancelled: z.boolean(),
					}),
				),
			}),
			(input) => {
				const wheel = new TimerWheel(input.startMs);
				return {
					steps: input.steps.map((step) => {
						const fired = step.op === "advance" ? wheel.advance(step.ms ?? 0) : [];
						const cancelled = step.op === "cancel" && wheel.cancel(step.tag ?? "");
						if (step.op === "after") wheel.after(step.ms ?? 0, step.tag ?? "");
						return { nowMs: wheel.nowMs(), waiting: wheel.waiting(), fired, cancelled };
					}),
				};
			},
		),
		/** Sleeps and calls on one wheel; runs settle only at `settle`. `cancel` and a repeated `sleep` name an earlier `id`. */
		scheduler: kind(
			z.object({
				startMs: z.int(),
				steps: z.array(
					z.object({
						op: z.enum(["sleep", "schedule", "cancel", "advance", "settle", "wheelCancel", "wheelAfter"]),
						id: z.string().exactOptional(),
						ms: z.int().exactOptional(),
						tag: z.string().exactOptional(),
					}),
				),
			}),
			z.object({
				steps: z.array(
					z.object({
						nowMs: z.int(),
						waiting: z.array(z.string()),
						fired: z.array(z.object({ tag: z.string(), atMs: z.int() })),
						/** Scheduled calls that ran. */
						ran: z.array(z.string()),
						/** Sleeps that ended: done or cancelled. */
						ended: z.array(z.object({ id: z.string(), outcome: z.string() })),
					}),
				),
			}),
			async (input) => {
				const scheduler = fakeScheduler(new TimerWheel(input.startMs));
				const sleepers = new Map<string, AbortController>();
				const calls = new Map<string, () => void>();
				let ran: string[] = [];
				let ended: Array<{ id: string; outcome: string }> = [];
				const steps = [];
				for (const step of input.steps) {
					const id = step.id ?? "";
					const ms = step.ms ?? 0;
					const tag = step.tag ?? "";
					let fired: Array<{ tag: string; atMs: number }> = [];
					if (step.op === "sleep") {
						const stop = sleepers.get(id) ?? new AbortController();
						sleepers.set(id, stop);
						scheduler.sleep(ms, stop.signal).then(
							() => ended.push({ id, outcome: "done" }),
							() => ended.push({ id, outcome: "cancelled" }),
						);
					} else if (step.op === "schedule") {
						calls.set(
							id,
							scheduler.schedule(ms, () => ran.push(id)),
						);
					} else if (step.op === "cancel") {
						sleepers.get(id)?.abort();
						calls.get(id)?.();
					} else if (step.op === "wheelCancel") {
						scheduler.wheel.cancel(tag);
					} else if (step.op === "wheelAfter") {
						scheduler.wheel.after(ms, tag);
					} else if (step.op === "settle") {
						for (let turn = 0; turn < 4; turn++) await Promise.resolve();
					} else {
						fired = scheduler.wheel.advance(ms);
					}
					steps.push({ nowMs: scheduler.nowMs(), waiting: scheduler.wheel.waiting(), fired, ran, ended });
					ran = [];
					ended = [];
				}
				return { steps };
			},
		),
		random: kind(
			z.object({ values: z.array(z.number()), draws: z.int().min(0) }),
			z.object({ drawn: z.array(z.number()) }),
			(input) => {
				const random = sequenceRandom(input.values);
				return { drawn: Array.from({ length: input.draws }, () => random.next()) };
			},
		),
		replay: kind(
			/** Project-rooted trace path. */
			z.object({ file: z.string() }),
			z.object({
				requests: z.array(z.unknown()),
				exchanges: z.array(
					z.object({
						status: z.int(),
						headers: z.record(z.string(), z.string()),
						/** Byte length per chunk. */
						chunks: z.array(z.int()),
						body: z.string(),
					}),
				),
			}),
			async (input, context) => {
				const trace = await readTrace(path.join(context.root, input.file));
				const replay = replayFetch(trace);
				const signal = new AbortController().signal;
				const exchanges = [];
				for (const exchange of trace.exchanges) {
					const response = await replay.fetch(
						{
							method: "POST",
							url: "http://replay.invalid/",
							headers: [],
							body: stringifyJson(exchange.request),
							budget: BUDGETS.stream,
						},
						signal,
					);
					const chunks: Uint8Array[] = [];
					for await (const chunk of response.chunks) chunks.push(chunk);
					exchanges.push({
						status: response.status,
						headers: Object.fromEntries(response.headers),
						chunks: chunks.map((chunk) => chunk.length),
						body: Buffer.concat(chunks).toString("utf8"),
					});
				}
				return {
					requests: replay.requests.map((request) =>
						request.body === undefined ? null : (parseJson(request.body) ?? null),
					),
					exchanges,
				};
			},
			{ context: true },
		),
		/** Handler context contents. */
		context: kind(
			z.object({ folder: z.string() }),
			z.object({
				ids: z.array(z.string()),
				times: z.array(z.int()),
				tmp: z.string(),
				root: z.string(),
				planted: z.boolean(),
			}),
			(input, context) => ({
				ids: [context.ids.newId(), context.ids.newId()],
				times: [context.clock.nowMs(), context.clock.nowMs()],
				tmp: path.join(context.tmp(), input.folder),
				root: path.join(context.root, input.folder),
				planted: existsSync(context.tmp()),
			}),
			{ context: true },
		),
	},
	sources: [
		...testkitPortCases,
		{ kind: "testkit.replay", name: "tool-loop", input: { file: "cases/traces/openrouter.tool-loop.json" } },
		{ kind: "testkit.replay", name: "chunk-edges", input: { file: "cases/traces/synthetic.chunks.json" } },
		{ kind: "testkit.random", name: "cycles", input: { values: [0, 0.5, 0.999999], draws: 7 } },
		{
			kind: "testkit.wheel",
			name: "timers",
			input: {
				startMs: 1000,
				steps: [
					{ op: "after", ms: 50, tag: "b" },
					{ op: "after", ms: 50, tag: "a" },
					{ op: "after", ms: 10, tag: "c" },
					{ op: "after", ms: 200, tag: "late" },
					{ op: "advance", ms: 9 },
					{ op: "advance", ms: 1 },
					{ op: "after", ms: 40, tag: "tie" },
					{ op: "cancel", tag: "a" },
					{ op: "cancel", tag: "a" },
					{ op: "advance", ms: 40 },
					{ op: "after", ms: 0, tag: "now" },
					{ op: "after", ms: -5, tag: "past" },
					{ op: "after", ms: 30, tag: "late" },
					{ op: "advance", ms: 0 },
					{ op: "advance", ms: -10 },
					{ op: "advance", ms: 1000 },
					{ op: "cancel", tag: "none" },
				],
			},
		},
		{
			kind: "testkit.scheduler",
			name: "sleeps-and-calls",
			input: {
				startMs: 5000,
				steps: settled([
					{ op: "sleep", id: "s50", ms: 50 },
					{ op: "schedule", id: "c50", ms: 50 },
					{ op: "sleep", id: "s10", ms: 10 },
					{ op: "schedule", id: "c30", ms: 30 },
					{ op: "sleep", id: "s30", ms: 30 },
					{ op: "sleep", id: "now", ms: 0 },
					{ op: "sleep", id: "past", ms: -5 },
					{ op: "advance", ms: 0 },
					{ op: "advance", ms: 9 },
					{ op: "advance", ms: 1 },
					{ op: "advance", ms: 100 },
					{ op: "advance", ms: 100 },
				]),
			},
		},
		{
			kind: "testkit.scheduler",
			name: "cancels",
			input: {
				startMs: 0,
				steps: settled([
					{ op: "sleep", id: "a", ms: 100 },
					{ op: "sleep", id: "b", ms: 100 },
					{ op: "schedule", id: "c", ms: 100 },
					{ op: "schedule", id: "d", ms: 100 },
					{ op: "sleep", id: "e", ms: 200 },
					{ op: "cancel", id: "a" },
					{ op: "cancel", id: "c" },
					{ op: "cancel", id: "c" },
					{ op: "cancel", id: "none" },
					{ op: "advance", ms: 100 },
					{ op: "cancel", id: "b" },
					{ op: "cancel", id: "d" },
					{ op: "advance", ms: 100 },
					{ op: "cancel", id: "e" },
				]),
			},
		},
		{
			kind: "testkit.scheduler",
			name: "unsettled-cancel",
			input: {
				startMs: 0,
				steps: [
					{ op: "sleep", id: "a", ms: 10 },
					{ op: "cancel", id: "a" },
					{ op: "advance", ms: 10 },
					{ op: "settle" },
				],
			},
		},
		{
			kind: "testkit.scheduler",
			name: "fire-then-cancel",
			input: {
				startMs: 0,
				steps: [
					{ op: "sleep", id: "a", ms: 10 },
					{ op: "advance", ms: 10 },
					{ op: "cancel", id: "a" },
					{ op: "settle" },
				],
			},
		},
		{
			kind: "testkit.scheduler",
			name: "cancelled-id",
			input: {
				startMs: 0,
				steps: [
					{ op: "sleep", id: "a", ms: 10 },
					{ op: "cancel", id: "a" },
					{ op: "sleep", id: "a", ms: 20 },
					{ op: "sleep", id: "b", ms: 5 },
					{ op: "settle" },
					{ op: "advance", ms: 100 },
					{ op: "settle" },
				],
			},
		},
		{
			kind: "testkit.scheduler",
			name: "wheel-cancel",
			input: {
				startMs: 0,
				steps: [
					{ op: "sleep", id: "a", ms: 10 },
					{ op: "wheelCancel", tag: "sleep-1" },
					{ op: "advance", ms: 50 },
					{ op: "settle" },
					{ op: "cancel", id: "a" },
					{ op: "settle" },
				],
			},
		},
		{
			kind: "testkit.scheduler",
			name: "wheel-replace",
			input: {
				startMs: 0,
				steps: [
					{ op: "sleep", id: "a", ms: 10 },
					{ op: "wheelAfter", tag: "sleep-1", ms: 50 },
					{ op: "advance", ms: 50 },
					{ op: "settle" },
					{ op: "cancel", id: "a" },
					{ op: "settle" },
				],
			},
		},
		{ kind: "testkit.compactedSession", name: "rows", input: {} },
		{ kind: "testkit.context", name: "handed", input: { folder: "a/b" } },
	],
});
