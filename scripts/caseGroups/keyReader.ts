import { KeyReader } from "@nyaaai/agentic/health/keyReader";
import { z } from "@nyaaai/contract/shape";
import { KeyState } from "@nyaaai/contract/shapes/provider";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { keyReaderCases } from "../caseSources/keyReader";

////////////////////////////////
//  Shapes

/** A check is held open by the script: only a step, an abort or the deadline ends it. */
const Step = () =>
	z.discriminatedUnion("kind", [
		/** Starts a read, and with it check number `n`, counting from 1. */
		z.object({ kind: z.literal("read") }),
		/** Check `check` answers this state, if it is still open. */
		z.object({ kind: z.literal("answer"), check: z.int().min(1), state: KeyState }),
		/** Check `check` fails with this message, if it is still open. */
		z.object({ kind: z.literal("fail"), check: z.int().min(1), message: z.string() }),
		z.object({ kind: z.literal("advance"), ms: z.int().min(0) }),
		z.object({ kind: z.literal("dispose") }),
	]);

const Outcome = () =>
	z.object({
		/** The wheel's time when the step ended. */
		nowMs: z.int(),
		/** What the reader showed during the step, in order. */
		shown: z.array(KeyState),
		/** Checks whose signal ended during the step, and what ended it. */
		signalled: z.array(z.object({ check: z.int(), by: z.enum(["deadline", "caller"]) })),
		/** Checks still open: no answer, no failure and no signal yet. */
		open: z.array(z.int()),
		/** A check is still running: one that was dropped counts until it ends. */
		busy: z.boolean(),
	});

////////////////////////////////
//  Group

export const keyReader = caseGroup("keyReader", {
	kinds: {
		/** One reader on the timer wheel, over checks the script holds. */
		run: kind(
			z.object({ startMs: z.int(), steps: z.array(Step()).min(1) }),
			z.object({
				steps: z.array(Outcome()),
				/** Timers still waiting at the end, after the reader is disposed. */
				waiting: z.array(z.string()),
			}),
			async (input) => {
				const wheel = new TimerWheel(input.startMs);
				const scheduler = fakeScheduler(wheel);
				interface Check {
					settle: (answer: { state: z.infer<typeof KeyState> } | { failed: string }) => void;
					closed: boolean;
					chain: Promise<void> | undefined;
					awaited: boolean;
				}
				const checks: Check[] = [];
				let shown: Array<z.infer<typeof KeyState>> = [];
				let signalled: Array<{ check: number; by: "deadline" | "caller" }> = [];
				const reader = new KeyReader(
					scheduler,
					(signal) =>
						new Promise((resolve, reject) => {
							const number = checks.length + 1;
							const held: Check = {
								closed: false,
								chain: undefined,
								awaited: false,
								settle: (answer) => {
									if (held.closed) return;
									held.closed = true;
									if ("state" in answer) resolve(answer.state);
									else reject(new Error(answer.failed));
								},
							};
							checks.push(held);
							signal.addEventListener(
								"abort",
								() => {
									if (held.closed) return;
									const reason = signal.reason as { name?: string } | undefined;
									signalled.push({
										check: number,
										by: reason?.name === "TimeoutError" ? "deadline" : "caller",
									});
									held.settle({ failed: "ended" });
								},
								{ once: true },
							);
						}),
					(state) => shown.push(state),
				);
				const steps: Array<z.infer<ReturnType<typeof Outcome>>> = [];
				for (const step of input.steps) {
					if (step.kind === "read") {
						const chain = reader.read();
						const held = checks[checks.length - 1];
						if (held !== undefined) held.chain = chain;
					} else if (step.kind === "answer") checks[step.check - 1]?.settle({ state: step.state });
					else if (step.kind === "fail") checks[step.check - 1]?.settle({ failed: step.message });
					else if (step.kind === "advance") wheel.advance(step.ms);
					else reader.dispose();
					// A closed check's chain has run once awaited.
					for (const held of checks) {
						if (!held.closed || held.awaited || held.chain === undefined) continue;
						held.awaited = true;
						await held.chain;
					}
					steps.push({
						nowMs: wheel.nowMs(),
						shown,
						signalled,
						open: checks.flatMap((held, at) => (held.closed ? [] : [at + 1])),
						busy: reader.busy(),
					});
					shown = [];
					signalled = [];
				}
				reader.dispose();
				await reader.idle();
				return { steps, waiting: wheel.waiting() };
			},
		),
	},
	sources: casesFor("keyReader", keyReaderCases),
});
