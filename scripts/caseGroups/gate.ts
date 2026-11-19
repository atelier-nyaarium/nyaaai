import { GateBook, type GateState, ProviderGate } from "@nyaaai/agentic/health/gate";
import { countedIds } from "@nyaaai/contract/id";
import { z } from "@nyaaai/contract/shape";
import { Admission, GateOutcome, GateSettled, GateWaiting, Ticket } from "@nyaaai/contract/shapes/health";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { gateCases } from "../caseSources/gate";

////////////////////////////////
//  Shapes

/** The era's book a step uses, `main` when absent; a book's era is counted from 1 as books are first used. */
const InBook = { book: z.string().exactOptional() };

/** A script step; a request is named by the case. */
const Step = () =>
	z.discriminatedUnion("kind", [
		/** Starts an admission: it resolves at once or queues. */
		z.object({
			kind: z.literal("admit"),
			request: z.string(),
			model: z.string(),
			/** A retry goes to the front of the queue. */
			again: z.boolean().exactOptional(),
			/** The caller's abort, this long after the step. */
			abortAfterMs: z.int().min(0).exactOptional(),
			abortedBefore: z.boolean().exactOptional(),
			/** What the waiter's callback does after it records the gate it sees. */
			inside: z.enum(["cool", "tryAdmit", "settleTrigger"]).exactOptional(),
			...InBook,
		}),
		z.object({ kind: z.literal("tryAdmit"), request: z.string(), model: z.string(), ...InBook }),
		/** Settles the ticket the named request was given. */
		z.object({
			kind: z.literal("settle"),
			request: z.string(),
			outcome: GateOutcome,
			retryAfterMs: z.int().min(0).exactOptional(),
		}),
		/** Settles a ticket given as data, such as another gate's. */
		z.object({ kind: z.literal("settleTicket"), ticket: Ticket, outcome: GateOutcome }),
		z.object({
			kind: z.literal("cool"),
			model: z.string(),
			retryAfterMs: z.int().min(0).exactOptional(),
			...InBook,
		}),
		/** Retires the named book; the next use of its name starts a new era. */
		z.object({ kind: z.literal("retire"), ...InBook }),
		z.object({ kind: z.literal("advance"), ms: z.int().min(0) }),
		/** The caller of the named request aborts. */
		z.object({ kind: z.literal("abort"), request: z.string() }),
		z.object({ kind: z.literal("dispose") }),
	]);

const GateData = () =>
	z.object({
		model: z.string(),
		/** None until the gate is first used. */
		gate: z
			.object({
				phase: z.enum(["open", "paused", "probing"]),
				generation: z.int(),
				pauses: z.int(),
				resumeAt: z.number(),
				probe: Ticket.nullable(),
				queued: z.int(),
			})
			.nullable(),
	});

const Outcome = () =>
	z.object({
		/** The wheel's time when the step ended. */
		nowMs: z.int(),
		/** What `settle` answered. */
		settled: GateSettled.nullable(),
		/** What `tryAdmit` answered. */
		ticket: Ticket.nullable(),
		/** Admissions that resolved during the step, in order. */
		resolved: z.array(z.object({ request: z.string(), admission: Admission })),
		/** The gate a waiter's callback saw when it ran, in order. */
		inside: z.array(z.object({ request: z.string(), gate: GateData().shape.gate })),
		/** What waiting requests were told, in order. */
		waits: z.array(z.object({ request: z.string(), waiting: GateWaiting })),
		/** Models whose gate turned ready or not ready, in order. */
		told: z.array(z.string()),
		/** Every model used so far, in the order first used. */
		gates: z.array(GateData()),
		/** Models whose gate is not ready, in code unit order. */
		holding: z.array(z.string()),
	});

////////////////////////////////
//  Constants

const REASON = "caller aborted";

////////////////////////////////
//  Group

export const gate = caseGroup("gate", {
	kinds: {
		/** One gate on the timer wheel, driven by a script of steps. */
		run: kind(
			z.object({
				startMs: z.int(),
				pauseMs: z.int().min(0).exactOptional(),
				maxPauses: z.int().min(0).exactOptional(),
				steps: z.array(Step()).min(1),
			}),
			z.object({
				steps: z.array(Outcome()),
				/** Timers still waiting at the end, after the gate is disposed. */
				waiting: z.array(z.string()),
			}),
			async (input) => {
				const wheel = new TimerWheel(input.startMs);
				const scheduler = fakeScheduler(wheel);
				const gate = new ProviderGate({
					scheduler,
					ids: countedIds("gate-"),
					...(input.pauseMs === undefined ? {} : { pauseMs: input.pauseMs }),
					...(input.maxPauses === undefined ? {} : { maxPauses: input.maxPauses }),
				});
				interface Request {
					id: string;
					controller: AbortController;
					done: Promise<void>;
				}
				type Resolved = Array<{ request: string; admission: z.infer<typeof Admission> }>;
				let resolved: Resolved = [];
				let waits: Array<{ request: string; waiting: z.infer<typeof GateWaiting> }> = [];
				let inside: Array<{ request: string; gate: GateState | null }> = [];
				let told: string[] = [];
				gate.onChange((_owner, model) => told.push(model));
				const books = new Map<string, GateBook>();
				const byEra = new Map<number, GateBook>();
				let eras = 0;
				const bookOf = (name = "main"): GateBook => {
					let book = books.get(name);
					if (book === undefined) {
						eras += 1;
						book = new GateBook(eras, name);
						books.set(name, book);
						byEra.set(eras, book);
					}
					return book;
				};
				/** The book each model was first used in. */
				const homes = new Map<string, string>();
				const tickets = new Map<string, z.infer<typeof Ticket>>();
				const requests = new Map<string, Request>();
				/** Each model's waiting requests in the gate's queue order. */
				const queues = new Map<string, Request[]>();
				const models: string[] = [];
				const use = (model: string, book = "main") => {
					if (!models.includes(model)) models.push(model);
					homes.set(model, book);
				};
				/** Requests whose abort just resolved them. */
				let stopped: Request[] = [];
				const abortOf = (id: string) => {
					const request = requests.get(id);
					if (request === undefined) throw new Error(`No request ${id}.`);
					for (const queue of queues.values()) {
						const at = queue.indexOf(request);
						if (at >= 0) queue.splice(at, 1);
					}
					stopped.push(request);
					request.controller.abort(new Error(REASON));
				};
				const stateIn = (model: string, name = "main") => {
					const book = books.get(name);
					return book === undefined ? null : (gate.stateOf(book, model) ?? null);
				};
				const queuedOf = (model: string, name?: string) => stateIn(model, name)?.queued ?? 0;
				const stateOf = (model: string) => stateIn(model, homes.get(model));
				/** The book a ticket's era names, else the main one. */
				const bookFor = (ticket: z.infer<typeof Ticket>) => byEra.get(ticket.era) ?? bookOf();
				const ticketOf = (id: string) => {
					const held = tickets.get(id);
					if (held === undefined) throw new Error(`No ticket for ${id}.`);
					return held;
				};
				let disposed = false;
				const steps: Array<z.infer<ReturnType<typeof Outcome>>> = [];
				for (const step of input.steps) {
					let settled: z.infer<typeof GateSettled> | null = null;
					let ticket: z.infer<typeof Ticket> | null = null;
					if (step.kind === "admit") {
						use(step.model, step.book);
						const controller = new AbortController();
						if (step.abortedBefore === true) controller.abort(new Error(REASON));
						else if (step.abortAfterMs !== undefined)
							wheel.after(step.abortAfterMs, `abort-${step.request}`, () => abortOf(step.request));
						const before = queuedOf(step.model, step.book);
						const done = gate
							.admit(
								bookOf(step.book),
								step.model,
								controller.signal,
								(waiting) => {
									waits.push({ request: step.request, waiting });
									inside.push({ request: step.request, gate: stateOf(step.model) });
									if (step.inside === "cool") gate.cool(bookOf(step.book), step.model);
									else if (step.inside === "tryAdmit") gate.tryAdmit(bookOf(step.book), step.model);
									else if (step.inside === "settleTrigger") {
										const held = tickets.get("trigger");
										if (held !== undefined) gate.settle(bookFor(held), held, "overloaded");
									}
								},
								step.again === true,
							)
							.then((admission) => {
								resolved.push({ request: step.request, admission });
								if (admission.kind === "go") tickets.set(step.request, admission.ticket);
							});
						const request: Request = { id: step.request, controller, done };
						requests.set(step.request, request);
						if (queuedOf(step.model, step.book) > before) {
							const queue = queues.get(step.model) ?? [];
							if (step.again === true) queue.unshift(request);
							else queue.push(request);
							queues.set(step.model, queue);
						} else await done;
					} else if (step.kind === "tryAdmit") {
						use(step.model, step.book);
						ticket = gate.tryAdmit(bookOf(step.book), step.model);
						if (ticket !== null) tickets.set(step.request, ticket);
					} else if (step.kind === "settle") {
						const held = ticketOf(step.request);
						use(held.model, homes.get(held.model));
						settled = gate.settle(bookFor(held), held, step.outcome, step.retryAfterMs);
					} else if (step.kind === "settleTicket") {
						use(step.ticket.model, homes.get(step.ticket.model));
						settled = gate.settle(bookFor(step.ticket), step.ticket, step.outcome);
					} else if (step.kind === "cool") {
						use(step.model, step.book);
						gate.cool(bookOf(step.book), step.model, step.retryAfterMs);
					} else if (step.kind === "retire") {
						gate.retire(bookOf(step.book));
						books.delete(step.book ?? "main");
					} else if (step.kind === "advance") {
						wheel.advance(step.ms);
					} else if (step.kind === "abort") {
						abortOf(step.request);
					} else {
						gate.dispose();
						disposed = true;
					}
					// Released calls left the queue from the front.
					const released: Request[] = [];
					for (const [model, queue] of queues)
						released.push(...queue.splice(0, queue.length - queuedOf(model, homes.get(model))));
					for (const request of [...stopped, ...released]) await request.done;
					stopped = [];
					steps.push({
						nowMs: wheel.nowMs(),
						settled,
						ticket,
						resolved,
						inside,
						waits,
						told,
						gates: models.map((model) => ({ model, gate: stateOf(model) })),
						holding: [...books.values()].flatMap((book) => gate.holding(book)).sort(),
					});
					resolved = [];
					waits = [];
					inside = [];
					told = [];
				}
				if (!disposed) gate.dispose();
				for (const request of requests.values()) await request.done;
				for (const id of requests.keys()) wheel.cancel(`abort-${id}`);
				return { steps, waiting: wheel.waiting() };
			},
		),
	},
	sources: casesFor("gate", gateCases),
});
