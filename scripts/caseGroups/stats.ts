import { StatsQueue, statsOf } from "@nyaaai/agentic/usage/stats";
import { z } from "@nyaaai/contract/shape";
import { GenerationStats, type MeteredCall, UnfilledRecord } from "@nyaaai/contract/shapes/usage";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { scriptedUsageSink } from "@nyaaai/testkit/scriptedPorts";
import { statsCases } from "../caseSources/stats";
import { NetExchange, networkOf } from "./catalogKit";

////////////////////////////////
//  Shapes

/** A record handed to the queue. A missing time is the wheel's now. */
const Added = z.object({
	id: z.string(),
	generationId: z.string(),
	at: z.int().exactOptional(),
	entry: z.string().exactOptional(),
});

/** What happens at a wheel time inside a step, whether or not a read is running then. */
const Action = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("add"), ...Added.shape }),
	z.object({
		kind: z.literal("addMany"),
		count: z.int().min(1),
		at: z.int().exactOptional(),
		entry: z.string().exactOptional(),
	}),
	z.object({ kind: z.literal("dispose") }),
]);
type ActionOf = z.infer<typeof Action>;

const Step = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("add"), ...Added.shape }),
	/** `count` records `r1`, `r2` and on, named `gen-r1`, `gen-r2` and on. */
	z.object({
		kind: z.literal("addMany"),
		count: z.int().min(1),
		at: z.int().exactOptional(),
		entry: z.string().exactOptional(),
	}),
	/** Time passes, a read at a time; each `during` action runs at its wheel time from the step's start. */
	z.object({
		kind: z.literal("advance"),
		ms: z.int().min(0),
		during: z.array(z.object({ afterMs: z.int().min(0), do: Action })).exactOptional(),
	}),
	/** Adds the recent unfilled records, but those of the entries named. */
	z.object({ kind: z.literal("restore"), skip: z.array(z.string()).exactOptional() }),
	z.object({ kind: z.literal("dispose") }),
]);
type StepOf = z.infer<typeof Step>;

const ReadLine = z.object({
	/** The wheel's time when the read began and when it ended. */
	atMs: z.int(),
	endedMs: z.int(),
	entry: z.string().exactOptional(),
	path: z.string(),
	failed: z.boolean(),
});
type ReadLine = z.infer<typeof ReadLine>;

const WaitingLine = z.object({
	id: z.string(),
	generationId: z.string(),
	entry: z.string().exactOptional(),
	at: z.int(),
	tries: z.int(),
	due: z.int(),
});

////////////////////////////////
//  Functions & Helpers

/** A call as a log would hold it, made at `at`. */
function callAt(at: number): MeteredCall {
	return { tag: { feature: "chat" }, model: "m", at, ms: 0, cost: { usd: 0, estimated: false } };
}

////////////////////////////////
//  Group

export const stats = caseGroup("stats", {
	kinds: {
		/** None when the body is not a generation's stats. */
		decode: kind(
			z.object({ bodies: z.array(z.unknown()) }),
			z.object({ stats: z.array(GenerationStats.nullable()) }),
			(input) => ({ stats: input.bodies.map((body) => statsOf(body) ?? null) }),
		),
		/**
		 * One queue on the timer wheel, over a scripted sink and a scripted network that answers each
		 * read in order. `known` are ids another window filled, with their request time; `unfilled`
		 * what the sink lists; `fails` makes `filled` or `fill` fail for the ids named, or `unfilled`.
		 */
		run: kind(
			z.object({
				startMs: z.int(),
				exchanges: z.array(NetExchange),
				known: z.array(z.object({ id: z.string(), at: z.int() })).exactOptional(),
				unfilled: z.array(UnfilledRecord).exactOptional(),
				fails: z
					.object({
						filled: z.array(z.string()).exactOptional(),
						fill: z.array(z.string()).exactOptional(),
						unfilled: z.boolean().exactOptional(),
					})
					.exactOptional(),
				/** The sink's `filled` answers for these ids only after `ms` on the wheel, or never. */
				holdFilled: z.array(z.object({ id: z.string(), ms: z.int().min(0).exactOptional() })).exactOptional(),
				/** A read of this generation ignores its signal and answers `body` after `ms`; the network is not asked. */
				late: z
					.array(z.object({ generationId: z.string(), ms: z.int().min(0), body: z.unknown() }))
					.exactOptional(),
				steps: z.array(Step).min(1),
			}),
			z.object({
				steps: z.array(
					z.object({
						/** The wheel's time when the step ended. */
						nowMs: z.int(),
						/** Reads that began during the step. */
						reads: z.array(ReadLine),
						/** Stats written during the step. */
						fills: z.array(z.object({ id: z.string(), stats: GenerationStats })),
						/** The queue after the step, in order. */
						waiting: z.array(WaitingLine),
						/** When the next read is set for. */
						armedAt: z.int().nullable(),
					}),
				),
				/** Paths the network got, in order. */
				requests: z.array(z.string()),
				read: z.array(z.int()),
				filledAsks: z.array(z.object({ id: z.string(), at: z.int() })),
				unfilledAsks: z.array(z.int()),
				/** Timers left on the wheel. */
				timers: z.array(z.string()),
			}),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges);
				const sink = scriptedUsageSink();
				const known = input.known ?? [];
				for (const held of known) {
					sink.setIds([held.id]);
					sink.record(callAt(held.at));
				}
				sink.setFilled(known.map((held) => held.id));
				sink.setUnfilled(input.unfilled ?? []);
				const fails = input.fails ?? {};
				const reads: ReadLine[] = [];
				let timers = 0;
				/** Timers of the queue's scheduler, as the wheel holds them. */
				const live: Array<{ at: number; on: boolean }> = [];
				/** Timers of the script: actions, held answers, late reads. */
				const waits: Array<{ at: number; done: boolean }> = [];
				const waitOn = (ms: number, name: string, run: () => void) => {
					timers += 1;
					const each = { at: net.wheel.nowMs() + ms, done: false };
					waits.push(each);
					net.wheel.after(ms, `${name}-${timers}`, () => {
						each.done = true;
						run();
					});
				};
				const scheduler = {
					...net.scheduler,
					schedule: (ms: number, run: () => void) => {
						const each = { at: net.wheel.nowMs() + ms, on: true };
						live.push(each);
						const cancel = net.scheduler.schedule(ms, () => {
							each.on = false;
							run();
						});
						return () => {
							each.on = false;
							cancel();
						};
					},
				};
				let wake = () => {};
				/** A port's answer is held: the queue waits on the wheel, which only the script moves. */
				const park = async (held: Promise<void>) => {
					wake();
					await held;
				};
				const queue: StatsQueue = new StatsQueue({
					scheduler,
					usage: {
						fill: (id, held) => {
							sink.fill(id, held);
							if (fails.fill?.includes(id)) throw new Error("fill failed");
						},
						filled: async (id, at) => {
							const answer = await sink.filled(id, at);
							if (fails.filled?.includes(id)) throw new Error("filled failed");
							const hold = input.holdFilled?.find((held) => held.id === id);
							if (hold !== undefined && hold.ms !== undefined) {
								const { ms } = hold;
								await park(new Promise<void>((resolve) => waitOn(ms, "hold", resolve)));
							} else if (hold !== undefined) await park(new Promise<void>(() => {}));
							return answer;
						},
						unfilled: async (since) => {
							const answer = await sink.unfilled(since);
							if (fails.unfilled === true) throw new Error("unfilled failed");
							return answer;
						},
					},
					read: async (entry, path, signal) => {
						const line = { atMs: net.wheel.nowMs(), ...(entry === undefined ? {} : { entry }), path };
						const late = input.late?.find((held) => path.endsWith(`=${held.generationId}`));
						if (late !== undefined) {
							await park(new Promise<void>((resolve) => waitOn(late.ms, "late", resolve)));
							reads.push({ ...line, endedMs: net.wheel.nowMs(), failed: false });
							return late.body;
						}
						try {
							const body = await net.transport.getJson(path, signal, "stored");
							reads.push({ ...line, endedMs: net.wheel.nowMs(), failed: false });
							return body;
						} catch (error) {
							reads.push({ ...line, endedMs: net.wheel.nowMs(), failed: true });
							throw error;
						}
					},
				});
				let made = 0;
				const many = (count: number, at: number | undefined, entry: string | undefined) => {
					for (let each = 0; each < count; each++) {
						made += 1;
						queue.add(`r${made}`, `gen-r${made}`, at, entry);
					}
				};
				/** Waits for the queue to end its read, or to wait on a held answer. */
				const settle = async (): Promise<"idle" | "parked"> => {
					const parked = new Promise<"parked">((resolve) => {
						wake = () => resolve("parked");
					});
					return Promise.race([queue.idle().then(() => "idle" as const), parked]);
				};
				const act = (held: ActionOf) => {
					if (held.kind === "add") queue.add(held.id, held.generationId, held.at, held.entry);
					else if (held.kind === "addMany") many(held.count, held.at, held.entry);
					else queue.dispose();
				};
				/** A read at a time: stops at each instant the queue or an action is set for. */
				const advance = async (
					ms: number,
					during: NonNullable<Extract<StepOf, { kind: "advance" }>["during"]>,
				) => {
					const start = net.wheel.nowMs();
					const target = start + ms;
					for (const held of during)
						if (held.afterMs <= ms) waitOn(held.afterMs, "during", () => act(held.do));
					let last: "idle" | "parked" = "idle";
					for (;;) {
						const times = [...live.filter((each) => each.on), ...waits.filter((each) => !each.done)]
							.map((each) => each.at)
							.filter((at) => at <= target)
							.sort((a, b) => a - b);
						const stop = times.length === 0 ? target : (times[0] ?? target);
						const progress = () => JSON.stringify([net.wheel.nowMs(), sink.calls(), queue.queued().length]);
						const before = JSON.stringify([progress(), times]);
						net.wheel.advance(Math.max(0, stop - net.wheel.nowMs()));
						last = await settle();
						if (times.length === 0) break;
						const after = JSON.stringify([
							progress(),
							[...live.filter((each) => each.on), ...waits.filter((each) => !each.done)]
								.map((each) => each.at)
								.filter((at) => at <= target)
								.sort((a, b) => a - b),
						]);
						if (after === before) throw new Error("the advance made no progress");
					}
					if (last === "parked") throw new Error("the queue waits on a held answer past the end of the step");
				};
				const steps = [];
				for (const step of input.steps) {
					const readsBefore = reads.length;
					const fillsBefore = sink.fills().length;
					if (step.kind === "add") act(step);
					else if (step.kind === "addMany") many(step.count, step.at, step.entry);
					else if (step.kind === "advance") await advance(step.ms, step.during ?? []);
					else if (step.kind === "restore") {
						const skip = step.skip ?? [];
						await queue.restore((entry) => entry === undefined || !skip.includes(entry));
					} else queue.dispose();
					if ((await settle()) === "parked")
						throw new Error("the queue waits on a held answer past the end of the step");
					const armed = queue.armedAt();
					const on = live.filter((each) => each.on);
					if (armed === undefined ? on.length !== 0 : on.length !== 1 || on[0]?.at !== armed)
						throw new Error("the queue's timer and the wheel disagree");
					steps.push({
						nowMs: net.wheel.nowMs(),
						reads: reads.slice(readsBefore),
						fills: sink.fills().slice(fillsBefore),
						waiting: queue.queued(),
						armedAt: queue.armedAt() ?? null,
					});
				}
				const result = {
					steps,
					requests: net.asked(),
					read: net.played.read(),
					filledAsks: sink.filledAsks(),
					unfilledAsks: sink.unfilledAsks(),
					timers: net.wheel.waiting(),
				};
				queue.dispose();
				return result;
			},
		),
	},
	sources: casesFor("stats", statsCases),
});
