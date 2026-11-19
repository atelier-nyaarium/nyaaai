import { ProviderHealth } from "@nyaaai/agentic/health/health";
import { EntryRegistry, type RegistryEvent } from "@nyaaai/agentic/provider/incarnation";
import { KINDS } from "@nyaaai/agentic/provider/kind";
import type { Resolved } from "@nyaaai/agentic/provider/resolver";
import type { KeyAnswer } from "@nyaaai/contract/entryKeys";
import { countedIds } from "@nyaaai/contract/id";
import { z } from "@nyaaai/contract/shape";
import {
	Admission,
	Blocked,
	EntryHealth,
	GateOutcome,
	GateSettled,
	HealthChange,
	Ticket,
} from "@nyaaai/contract/shapes/health";
import { KeyState, ProviderEntry, SkipReason } from "@nyaaai/contract/shapes/provider";
import { Failure } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { scriptedEntryKeys, scriptedProviderList } from "@nyaaai/testkit/scriptedPorts";
import { healthCases } from "../caseSources/health";
import { NetExchange } from "./catalogKit";
import { leftIncarnation } from "./incarnationKit";
import { networkOf } from "./localKit";

////////////////////////////////
//  Shapes

/** What `EntryKeys.get` answers. */
const HealthKeyAnswer = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("key"), key: z.string() }),
	z.object({ kind: z.literal("none") }),
	z.object({ kind: z.literal("refused") }),
	z.object({ kind: z.literal("unready") }),
]);

/** The fields health reads of a resolved route; `taken` names a route a `take` step held, entry and key still given. */
const HealthRoute = z.object({ entry: ProviderEntry, key: z.string(), taken: z.string().exactOptional() });

/** A script step; a step that starts a check waits for it to end. */
const NoIdle = { idle: z.literal(false).exactOptional() };

const Step = () =>
	z.discriminatedUnion("kind", [
		/** Arms a hold the next check parks at; holds count from 1. `store` is the key read, `check` the kind's check. Does not wait for `idle`, as with `idle: false`. */
		z.object({ kind: z.literal("hold"), at: z.enum(["store", "check"]) }),
		/** Releases a hold: a store answers `answer`, a check answers `state`; each defaults to the real one. */
		z.object({
			kind: z.literal("release"),
			hold: z.int().min(1),
			state: KeyState.exactOptional(),
			answer: HealthKeyAnswer.exactOptional(),
			...NoIdle,
		}),
		/** On this entry's first change told, a listener starts a check. */
		z.object({ kind: z.literal("listenAndCheck"), id: z.string() }),
		/** Holds the route as it stands now, under a name; later routes name it with `taken`. */
		z.object({ kind: z.literal("take"), name: z.string(), route: HealthRoute }),
		/** The list changes, and tells. */
		z.object({ kind: z.literal("list"), entries: z.array(ProviderEntry), ...NoIdle }),
		/** The store's key changes, and tells. */
		z.object({ kind: z.literal("key"), id: z.string(), answer: HealthKeyAnswer, ...NoIdle }),
		z.object({ kind: z.literal("check"), id: z.string(), ...NoIdle }),
		z.object({ kind: z.literal("checkUnread") }),
		/** What a resolver would read for the entry. */
		z.object({ kind: z.literal("keys"), id: z.string() }),
		/** `served` bills a reply under this source; `keep` passes the exemption set. */
		z.object({
			kind: z.literal("served"),
			route: HealthRoute,
			billingSource: z.string().exactOptional(),
			keep: z.boolean().exactOptional(),
		}),
		z.object({
			kind: z.literal("failed"),
			route: HealthRoute,
			failure: Failure,
			onward: z.boolean(),
		}),
		z.object({
			kind: z.literal("billed"),
			id: z.string(),
			reply: z.object({
				source: z.string(),
				allowance: z.number().exactOptional(),
				cost: z.number().exactOptional(),
			}),
		}),
		z.object({ kind: z.literal("deferred"), route: HealthRoute, keep: z.boolean().exactOptional() }),
		z.object({ kind: z.literal("blocked"), route: HealthRoute, keep: z.boolean().exactOptional() }),
		z.object({ kind: z.literal("tryAdmit"), request: z.string(), route: HealthRoute }),
		/** Admits at once; never makes a request wait. */
		z.object({ kind: z.literal("admit"), request: z.string(), route: HealthRoute }),
		/** Settles the ticket the named request was given. */
		z.object({
			kind: z.literal("settle"),
			request: z.string(),
			outcome: GateOutcome,
			retryAfterMs: z.int().min(0).exactOptional(),
		}),
		z.object({ kind: z.literal("advance"), ms: z.int().min(0), ...NoIdle }),
		z.object({ kind: z.literal("dispose") }),
	]);

const RegistryChange = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("list"), entries: z.array(ProviderEntry) }),
	z.object({ kind: z.literal("key"), id: z.string() }),
]);

const RegistryStep = () =>
	z.discriminatedUnion("kind", [
		/** A watcher of the registry; it makes `onFirst` when first told and throws after when `throws`. */
		z.object({
			kind: z.literal("watch"),
			name: z.string(),
			onFirst: RegistryChange.exactOptional(),
			throws: z.boolean().exactOptional(),
		}),
		/** A key listener; it makes `onFirst` when first told. */
		z.object({ kind: z.literal("listen"), name: z.string(), onFirst: RegistryChange.exactOptional() }),
		...RegistryChange.options,
		z.object({ kind: z.literal("dispose") }),
	]);

const Answer = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("none") }),
		z.object({ kind: z.literal("keys"), answer: HealthKeyAnswer }),
		z.object({ kind: z.literal("skip"), skip: SkipReason.nullable() }),
		z.object({ kind: z.literal("blocked"), blocked: Blocked.nullable() }),
		z.object({ kind: z.literal("ticket"), ticket: Ticket.nullable() }),
		z.object({ kind: z.literal("admission"), admission: Admission }),
		z.object({ kind: z.literal("settled"), settled: GateSettled }),
	]);

const Outcome = () =>
	z.object({
		/** The wheel's time when the step ended. */
		nowMs: z.int(),
		answer: Answer(),
		/** What `onChange` told during the step, in order. */
		changes: z.array(HealthChange),
		/** Entries named so far, in first-named order. */
		entries: z.array(EntryHealth),
		/** The entries the run's exemption set holds, in code unit order. */
		run: z.array(z.string()),
		/** Requests the network has seen so far. */
		requests: z.int(),
		/** Holds reached and not released, in order. */
		parked: z.array(z.int()),
	});

////////////////////////////////
//  Constants

const PREPARED = z.enum(["none", "checking", "ok", "rejected", "unreachable"]);

////////////////////////////////
//  Group

export const health = caseGroup("health", {
	shapes: { HealthKeyAnswer, HealthRoute },
	kinds: {
		/** One health over scripted ports, real kinds' checks and the wheel. */
		run: kind(
			z.object({
				startMs: z.int(),
				pauseMs: z.int().min(0).exactOptional(),
				/** The first list. */
				entries: z.array(ProviderEntry),
				/** The store's first answers; an entry with none answers none. */
				keys: z.record(z.string(), HealthKeyAnswer),
				/** Replies, in the order the checks ask. */
				exchanges: z.array(NetExchange),
				steps: z.array(Step()).min(1),
			}),
			z.object({
				steps: z.array(Outcome()),
				/** Every URL the network was asked for, in order. */
				requests: z.array(z.string()),
				/** Timers still waiting at the end, after health is disposed. */
				waiting: z.array(z.string()),
			}),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges);
				const list = scriptedProviderList(input.entries);
				const raw = scriptedEntryKeys(input.keys);
				const entries = new EntryRegistry({ list, keys: raw });
				interface Hold {
					at: "store" | "check";
					reached: Promise<void>;
					arrive: () => void;
					release: (go: {
						state?: z.infer<typeof KeyState>;
						answer?: z.infer<typeof HealthKeyAnswer>;
					}) => void;
					parkedNow: boolean;
					seen: boolean;
					released: Promise<{ state?: z.infer<typeof KeyState>; answer?: z.infer<typeof HealthKeyAnswer> }>;
				}
				const holds: Hold[] = [];
				/** Armed holds per place, not yet taken. */
				const armed: Record<"store" | "check", Hold[]> = { store: [], check: [] };
				const arm = (at: "store" | "check"): Hold => {
					let arrive = () => {};
					let release: Hold["release"] = () => {};
					const held: Hold = {
						at,
						reached: new Promise<void>((resolve) => {
							arrive = resolve;
						}),
						arrive: () => {},
						release: () => {},
						parkedNow: false,
						seen: false,
						released: new Promise((resolve) => {
							release = resolve;
						}),
					};
					held.arrive = () => {
						held.parkedNow = true;
						held.seen = true;
						arrive();
					};
					held.release = (go) => {
						held.parkedNow = false;
						release(go);
					};
					holds.push(held);
					armed[at].push(held);
					return held;
				};
				const keys = {
					get: async (id: string) => {
						const held = armed.store.shift();
						if (held === undefined) return raw.get(id);
						held.arrive();
						const go = await held.released;
						return go.answer === undefined ? raw.get(id) : go.answer;
					},
					onChange: raw.onChange,
				};
				const kinds = Object.fromEntries(
					Object.entries(KINDS).map(([name, real]) => [
						name,
						{
							...real,
							check: (...given: Parameters<typeof real.check>) => {
								const held = armed.check.shift();
								if (held === undefined) return real.check(...given);
								held.arrive();
								return held.released.then((go) => go.state ?? real.check(...given));
							},
						},
					]),
				) as typeof KINDS;
				const health = new ProviderHealth({
					scheduler: net.scheduler,
					ids: countedIds("gate-"),
					entries,
					keys,
					kinds,
					kindDeps: () => net.deps,
					...(input.pauseMs === undefined ? {} : { pauseMs: input.pauseMs }),
				});
				let changes: Array<z.infer<typeof HealthChange>> = [];
				const starters = new Set<string>();
				health.onChange((change) => {
					if (!starters.delete(change.entry)) return;
					name(change.entry);
					health.check(change.entry);
				});
				health.onChange((change) => changes.push(change));
				const named: string[] = [];
				const name = (id: string) => {
					if (!named.includes(id)) named.push(id);
				};
				for (const entry of input.entries) name(entry.id);
				const run = new Set<string>();
				const tickets = new Map<string, z.infer<typeof Ticket>>();
				const held = new Map<string, Resolved>();
				/** The route over the entry's incarnation as it stands now. */
				const standing = (route: { entry: z.infer<typeof ProviderEntry>; key: string }) => {
					name(route.entry.id);
					const incarnation = entries.incarnation(route.entry.id) ?? leftIncarnation(entries, route.entry);
					return { entry: route.entry, key: route.key, incarnation } as unknown as Resolved;
				};
				const routeOf = (route: { entry: z.infer<typeof ProviderEntry>; key: string; taken?: string }) => {
					if (route.taken === undefined) return standing(route);
					name(route.entry.id);
					const taken = held.get(route.taken);
					if (taken === undefined) throw new Error(`No route taken as ${route.taken}.`);
					return taken;
				};
				const steps: Array<z.infer<ReturnType<typeof Outcome>>> = [];
				for (const step of input.steps) {
					let answer: z.infer<ReturnType<typeof Answer>> = { kind: "none" };
					const seenBefore = new Set(holds.filter((held) => held.seen));
					if (step.kind === "hold") {
						arm(step.at);
					} else if (step.kind === "take") {
						held.set(step.name, standing(step.route));
					} else if (step.kind === "release") {
						const held = holds[step.hold - 1];
						if (held === undefined) throw new Error(`No hold ${step.hold}.`);
						held.release({
							...(step.state === undefined ? {} : { state: step.state }),
							...(step.answer === undefined ? {} : { answer: step.answer }),
						});
					} else if (step.kind === "listenAndCheck") {
						starters.add(step.id);
					} else if (step.kind === "list") {
						for (const entry of step.entries) name(entry.id);
						list.set(step.entries);
					} else if (step.kind === "key") {
						name(step.id);
						raw.set(step.id, step.answer);
					} else if (step.kind === "check") {
						name(step.id);
						health.check(step.id);
					} else if (step.kind === "checkUnread") {
						health.checkUnread();
					} else if (step.kind === "keys") {
						name(step.id);
						answer = { kind: "keys", answer: await health.entryKeys.get(step.id) };
					} else if (step.kind === "served") {
						health.served(
							routeOf(step.route),
							step.billingSource === undefined
								? { input: 1, output: 1 }
								: { input: 1, output: 1, billingSource: step.billingSource },
							step.keep === true ? run : undefined,
						);
					} else if (step.kind === "failed") {
						health.failed(routeOf(step.route), step.failure, step.onward);
					} else if (step.kind === "billed") {
						name(step.id);
						health.billed(step.id, step.reply);
					} else if (step.kind === "deferred") {
						answer = {
							kind: "skip",
							skip: health.deferred(routeOf(step.route), step.keep === true ? run : undefined) ?? null,
						};
					} else if (step.kind === "blocked") {
						answer = {
							kind: "blocked",
							blocked: health.blocked(routeOf(step.route), step.keep === true ? run : undefined) ?? null,
						};
					} else if (step.kind === "tryAdmit") {
						const ticket = health.tryAdmit(routeOf(step.route));
						if (ticket !== null) tickets.set(step.request, ticket);
						answer = { kind: "ticket", ticket };
					} else if (step.kind === "admit") {
						const route = routeOf(step.route);
						const gate = health.gateState(route);
						if (
							gate !== undefined &&
							(gate.phase === "paused" || (gate.phase === "probing" && gate.probe !== null))
						)
							throw new Error(`The admission of ${step.request} would wait.`);
						const admission = await health.admit(route, new AbortController().signal);
						if (admission.kind === "go") tickets.set(step.request, admission.ticket);
						answer = { kind: "admission", admission };
					} else if (step.kind === "settle") {
						const ticket = tickets.get(step.request);
						if (ticket === undefined) throw new Error(`No ticket for ${step.request}.`);
						answer = { kind: "settled", settled: health.settle(ticket, step.outcome, step.retryAfterMs) };
					} else if (step.kind === "advance") {
						net.wheel.advance(step.ms);
					} else {
						health.dispose();
					}
					// A check meets earlier-armed holds and parks.
					const starts = ["list", "key", "check", "checkUnread", "keys"].includes(step.kind);
					let met = false;
					if (starts)
						for (const held of holds) {
							if (seenBefore.has(held)) continue;
							met = true;
							await held.reached;
						}
					if (!met && !("idle" in step && step.idle === false)) await health.idle();
					steps.push({
						nowMs: net.wheel.nowMs(),
						answer,
						changes,
						entries: named.map((id) => health.state(id)),
						run: [...run].sort(),
						requests: net.played.requests.length,
						parked: holds.flatMap((held, at) => (held.parkedNow ? [at + 1] : [])),
					});
					changes = [];
				}
				for (const held of holds)
					if (held.parkedNow)
						held.release(held.at === "check" ? { state: { kind: "none" } } : { answer: { kind: "none" } });
				await health.idle();
				health.dispose();
				entries.dispose();
				return {
					steps,
					requests: net.played.requests.map((sent) => sent.url),
					waiting: net.wheel.waiting(),
				};
			},
		),
		/** The registry's changes, in the order told, to watchers that change the list or the keys themselves. */
		registry: kind(
			z.object({
				entries: z.array(ProviderEntry),
				steps: z.array(RegistryStep()).min(1),
			}),
			z.object({
				steps: z.array(
					z.object({
						/** What the watchers and key listeners were told, in order: `name event:id#incarnation` or `name key:id`. */
						told: z.array(z.string()),
						/** Whether the change the step made threw. */
						thrown: z.boolean(),
					}),
				),
			}),
			(input) => {
				const list = scriptedProviderList(input.entries);
				const raw = scriptedEntryKeys({});
				const entries = new EntryRegistry({ list, keys: raw });
				let told: string[] = [];
				const change = (made: z.infer<typeof RegistryChange>) => {
					if (made.kind === "list") list.set(made.entries);
					else raw.set(made.id, { kind: "none" });
				};
				const textOf = (event: RegistryEvent) =>
					`${event.kind}:${event.incarnation.id}#${event.incarnation.number}`;
				const steps: Array<{ told: string[]; thrown: boolean }> = [];
				for (const step of input.steps) {
					let thrown = false;
					if (step.kind === "watch") {
						let first = true;
						entries.watch((events) => {
							told.push(`${step.name} ${events.map(textOf).join(" ")}`);
							if (first && step.onFirst !== undefined) change(step.onFirst);
							first = false;
							if (step.throws === true) throw new Error("A watcher threw.");
						});
					} else if (step.kind === "listen") {
						let first = true;
						entries.onKeyChange((id) => {
							told.push(`${step.name} key:${id}`);
							if (first && step.onFirst !== undefined) change(step.onFirst);
							first = false;
						});
					} else {
						try {
							if (step.kind === "dispose") entries.dispose();
							else change(step);
						} catch {
							thrown = true;
						}
					}
					steps.push({ told, thrown });
					told = [];
				}
				entries.dispose();
				return { steps };
			},
		),
		/** The store's answer, unless health holds the entry out. */
		keys: kind(
			z.object({
				rows: z.array(
					z.object({
						entry: ProviderEntry,
						/** False: the list does not hold the entry. */
						listed: z.boolean().exactOptional(),
						/** What the store answers while the row is prepared. */
						stored: HealthKeyAnswer,
						/** What the store answers when asked; the same when absent. */
						storedThen: HealthKeyAnswer.exactOptional(),
						/** How the entry stands before the ask. */
						prepare: PREPARED,
						exchanges: z.array(NetExchange),
						/** Asks made back to back, before anything else runs; 1 when absent. */
						asks: z.int().min(1).exactOptional(),
						/** Ids the store tells while the keys' `onChange` is held. */
						tell: z.array(z.string()).exactOptional(),
						/** Ids the store tells after the unsubscribe. */
						tellAfter: z.array(z.string()).exactOptional(),
					}),
				),
			}),
			z.object({
				rows: z.array(
					z.object({
						/** The status the entry stood in when asked. */
						status: KeyState,
						/** What each ask answered. */
						answers: z.array(HealthKeyAnswer),
						/** The status once every check has ended. */
						statusAfter: KeyState,
						/** Ids told to the listener while it was held. */
						told: z.array(z.string()),
						/** Ids told to it after it was released. */
						toldAfter: z.array(z.string()),
						/** Requests the network saw. */
						requests: z.int(),
					}),
				),
			}),
			async (input) => {
				const rows: Array<{
					status: z.infer<typeof KeyState>;
					answers: Array<z.infer<typeof HealthKeyAnswer>>;
					statusAfter: z.infer<typeof KeyState>;
					told: string[];
					toldAfter: string[];
					requests: number;
				}> = [];
				for (const row of input.rows) {
					const net = networkOf(0, row.exchanges);
					let current: KeyAnswer = row.stored;
					const listeners = new Set<(id: string) => void>();
					const keys = {
						get: async () => current,
						onChange: (listener: (id: string) => void) => {
							listeners.add(listener);
							return () => {
								listeners.delete(listener);
							};
						},
					};
					const tell = (id: string) => {
						for (const listener of [...listeners]) listener(id);
					};
					const entries = new EntryRegistry({
						list: scriptedProviderList(row.listed === false ? [] : [row.entry]),
						keys,
					});
					const health = new ProviderHealth({
						scheduler: net.scheduler,
						ids: countedIds("gate-"),
						entries,
						keys,
						kinds: KINDS,
						kindDeps: () => net.deps,
					});
					const route = {
						entry: row.entry,
						key: "lab/model-a",
						incarnation: entries.incarnation(row.entry.id) ?? leftIncarnation(entries, row.entry),
					} as unknown as Resolved;
					if (row.prepare === "checking") health.check(row.entry.id);
					else if (row.prepare === "ok") {
						health.check(row.entry.id);
						await health.idle();
					} else if (row.prepare === "rejected") {
						health.failed(route, { category: "keyRejected", status: 401, message: "no" }, false);
					} else if (row.prepare === "unreachable") {
						health.failed(route, { category: "network", status: "network", message: "down" }, false);
					}
					if (row.storedThen !== undefined) current = row.storedThen;
					const status = health.status(row.entry.id);
					// The asks and the tells run before any check gets a turn.
					const asks = Array.from({ length: row.asks ?? 1 }, () => health.entryKeys.get(row.entry.id));
					const heard: string[] = [];
					const stop = health.entryKeys.onChange((id) => heard.push(id));
					for (const id of row.tell ?? []) tell(id);
					const held = heard.length;
					stop();
					for (const id of row.tellAfter ?? []) tell(id);
					const answers = await Promise.all(asks);
					await health.idle();
					const statusAfter = health.status(row.entry.id);
					health.dispose();
					entries.dispose();
					rows.push({
						status,
						answers,
						statusAfter,
						told: heard.slice(0, held),
						toldAfter: heard.slice(held),
						requests: net.played.requests.length,
					});
				}
				return { rows };
			},
		),
	},
	sources: casesFor("health", healthCases),
});
