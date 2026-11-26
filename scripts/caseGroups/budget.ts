import { OPENROUTER_CHAT } from "@nyaaai/agentic/dialects/openrouter";
import { failureOf } from "@nyaaai/agentic/failure";
import { Transport } from "@nyaaai/agentic/transport";
import { BUDGET_NAMES, BudgetExceeded, budgetTable, span } from "@nyaaai/contract/fetch";
import { jsString } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { Budget, Failure } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedFetch } from "@nyaaai/testkit/scriptedFetch";
import { budgetCases } from "../caseSources/budget";
import { BudgetChange, BudgetChanges, changesOf } from "./budgetKit";
import { exchangeOf, TransportExchange } from "./transport";

////////////////////////////////
//  Shapes

/** How an exchange ended. */
export const BudgetEnd = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("budget"), bound: z.enum(["total", "idle", "bytes"]), limit: z.int() }),
	z.object({ kind: z.literal("caller") }),
	z.object({ kind: z.literal("failed"), message: z.string() }),
]);

/** What the consumer saw, in order. */
export const BudgetSeen = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("head"), atMs: z.int(), status: z.int() }),
	z.object({ kind: z.literal("chunk"), atMs: z.int(), size: z.int() }),
	z.object({ kind: z.literal("done"), atMs: z.int() }),
	z.object({ kind: z.literal("ended"), atMs: z.int(), end: BudgetEnd }),
]);

export const BudgetSpanStep = z.discriminatedUnion("kind", [
	/** A `getJson` of the operation. */
	z.object({ kind: z.literal("read"), path: z.string() }),
	z.object({ kind: z.literal("advance"), ms: z.int().min(0) }),
]);

export const BudgetSpanRead = z.object({
	startMs: z.int(),
	endMs: z.int(),
	/** The budget the request carried; null when none was sent. */
	sent: Budget.nullable(),
	answer: z.unknown(),
	failure: Failure.nullable(),
	thrown: z.string().nullable(),
});

////////////////////////////////
//  Functions & Helpers

const CALLER_REASON = "caller aborted";

const BASE_URL = "https://lab.invalid/api";

////////////////////////////////
//  Group

export const budget = caseGroup("budget", {
	shapes: { BudgetChange, BudgetChanges, BudgetEnd, BudgetSeen, BudgetSpanStep, BudgetSpanRead },
	kinds: {
		/** The presets with a host's changes, or the refusal. */
		table: kind(
			z.object({ changes: BudgetChanges }),
			z.object({
				presets: z.array(z.object({ name: z.string(), budget: Budget })).nullable(),
				refused: z.string().nullable(),
			}),
			(input) => {
				try {
					const table = budgetTable(changesOf(input.changes));
					return { presets: BUDGET_NAMES.map((name) => ({ name, budget: table[name] })), refused: null };
				} catch (error) {
					return { presets: null, refused: error instanceof Error ? error.message : jsString(error) };
				}
			},
		),
		/** One budgeted exchange on a scripted network; the consumer holds each chunk its time. */
		fetch: kind(
			z.object({
				budget: Budget,
				exchange: TransportExchange,
				/** Wheel time the consumer holds chunk `i` before its next read. */
				holds: z.array(z.int().min(0)).exactOptional(),
				/** The caller aborts at this wheel time. */
				abortAfterMs: z.int().exactOptional(),
				/** The caller arms that abort at this wheel time, not before the fetch. */
				abortArmedAtMs: z.int().exactOptional(),
				/** After the end, the consumer waits this long and reads once more. */
				afterMs: z.int().min(0).exactOptional(),
			}),
			z.object({
				seen: z.array(BudgetSeen),
				/** Chunks the network handed out. */
				served: z.int(),
				waiting: z.array(z.string()),
				nowMs: z.int(),
			}),
			async (input) => {
				const wheel = new TimerWheel(0);
				const scheduler = fakeScheduler(wheel);
				const played = scriptedFetch([exchangeOf(input.exchange)], scheduler);
				const caller = new AbortController();
				const abortAt = input.abortAfterMs;
				const abort = () => caller.abort(new Error(CALLER_REASON));
				if (abortAt !== undefined && input.abortArmedAtMs !== undefined) {
					const armedAt = input.abortArmedAtMs;
					wheel.after(armedAt, "arm", () => wheel.after(abortAt - armedAt, "caller", abort));
				} else if (abortAt !== undefined) wheel.after(abortAt, "caller", abort);
				const seen: Array<z.infer<typeof BudgetSeen>> = [];
				const ended = (error: unknown) => {
					const end: z.infer<typeof BudgetEnd> =
						error instanceof BudgetExceeded
							? { kind: "budget", bound: error.bound, limit: error.limit }
							: error === caller.signal.reason
								? { kind: "caller" }
								: { kind: "failed", message: error instanceof Error ? error.message : jsString(error) };
					seen.push({ kind: "ended", atMs: wheel.nowMs(), end });
				};
				let reader: AsyncIterator<Uint8Array> | undefined;
				try {
					const response = await played.fetch(
						{ method: "GET", url: `${BASE_URL}/x`, headers: [], budget: input.budget },
						caller.signal,
					);
					seen.push({ kind: "head", atMs: wheel.nowMs(), status: response.status });
					reader = response.chunks[Symbol.asyncIterator]();
					for (let index = 0; ; index++) {
						const step = await reader.next();
						if (step.done === true) {
							seen.push({ kind: "done", atMs: wheel.nowMs() });
							break;
						}
						seen.push({ kind: "chunk", atMs: wheel.nowMs(), size: step.value.byteLength });
						const hold = input.holds?.[index];
						if (hold !== undefined) wheel.advance(hold);
					}
				} catch (error) {
					ended(error);
					if (reader !== undefined && input.afterMs !== undefined) {
						wheel.advance(input.afterMs);
						try {
							const again = await reader.next();
							seen.push(
								again.done === true
									? { kind: "done", atMs: wheel.nowMs() }
									: { kind: "chunk", atMs: wheel.nowMs(), size: again.value.byteLength },
							);
						} catch (later) {
							ended(later);
						}
					}
				}
				await reader?.return?.();
				wheel.cancel("arm");
				wheel.cancel("caller");
				return { seen, served: played.read()[0] ?? 0, waiting: wheel.waiting(), nowMs: wheel.nowMs() };
			},
		),
		/** One span's reads through a transport; the key read spends `keyMs`. */
		span: kind(
			z.object({
				budget: Budget,
				keyMs: z.int().min(0).exactOptional(),
				exchanges: z.array(TransportExchange),
				steps: z.array(BudgetSpanStep).min(1),
			}),
			z.object({
				reads: z.array(BudgetSpanRead),
				/** How the span ended by the last step; null while running. */
				ended: z.enum(["deadline", "caller"]).nullable(),
				requests: z.int(),
				waiting: z.array(z.string()),
			}),
			async (input) => {
				const wheel = new TimerWheel(0);
				const scheduler = fakeScheduler(wheel);
				const played = scriptedFetch(input.exchanges.map(exchangeOf), scheduler);
				const transport = new Transport({
					fetch: played.fetch,
					scheduler,
					key: {
						get: async () => {
							wheel.advance(input.keyMs ?? 0);
							return "fake-key-one";
						},
					},
					log: () => {},
					baseUrl: BASE_URL,
					label: "Lab",
					userAgent: "cases/1.0",
					errors: OPENROUTER_CHAT,
				});
				const clock = span(scheduler, input.budget);
				const reads: Array<z.infer<typeof BudgetSpanRead>> = [];
				for (const step of input.steps) {
					if (step.kind === "advance") {
						wheel.advance(step.ms);
						continue;
					}
					if (!step.path.startsWith("/")) throw new Error(`a path starts with a slash: ${step.path}`);
					const before = played.requests.length;
					const read: z.infer<typeof BudgetSpanRead> = {
						startMs: wheel.nowMs(),
						endMs: 0,
						sent: null,
						answer: null,
						failure: null,
						thrown: null,
					};
					try {
						read.answer = await transport.getJson(`/${step.path.slice(1)}`, clock, "stored");
					} catch (error) {
						const held = failureOf(error);
						if (held === undefined) read.thrown = error instanceof Error ? error.message : jsString(error);
						else read.failure = held;
					}
					read.endMs = wheel.nowMs();
					read.sent = played.requests[before]?.budget ?? null;
					reads.push(read);
				}
				const ended = clock.ended() ?? null;
				clock.clear();
				return { reads, ended, requests: played.requests.length, waiting: wheel.waiting() };
			},
		),
	},
	sources: casesFor("budget", budgetCases),
});
