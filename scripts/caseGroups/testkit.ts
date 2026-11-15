import { existsSync } from "node:fs";
import path from "node:path";
import { stringifyJson } from "@nyaaai/contract/json";
import { compareText } from "@nyaaai/contract/order";
import { sequenceRandom } from "@nyaaai/contract/random";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, kind } from "@nyaaai/testkit/caseGroup";
import { compactedSession } from "@nyaaai/testkit/claudeSession";
import { TimerWheel } from "@nyaaai/testkit/scheduler";
import { readTrace, replayTransport } from "@nyaaai/testkit/trace";

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

////////////////////////////////
//  Group

export const testkit = caseGroup("testkit", {
	shapes: { TestkitConverseInput, TestkitConverseResult },
	kinds: {
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
				const transport = replayTransport(trace);
				const exchanges = [];
				for (const exchange of trace.exchanges) {
					const response = await transport.fetch("http://replay.invalid/", {
						body: stringifyJson(exchange.request),
					});
					const chunks: Uint8Array[] = [];
					for await (const chunk of response.body as unknown as AsyncIterable<Uint8Array>) chunks.push(chunk);
					const headers = [...response.headers].sort(([left], [right]) => compareText(left, right));
					exchanges.push({
						status: response.status,
						headers: Object.fromEntries(headers),
						chunks: chunks.map((chunk) => chunk.length),
						body: Buffer.concat(chunks).toString("utf8"),
					});
				}
				return { requests: transport.requests, exchanges };
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
		{ kind: "testkit.compactedSession", name: "rows", input: {} },
		{ kind: "testkit.context", name: "handed", input: { folder: "a/b" } },
	],
});
