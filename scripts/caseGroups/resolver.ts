import type { Codec } from "@nyaaai/agentic/codec";
import { failureOf } from "@nyaaai/agentic/failure";
import { parseRowKey } from "@nyaaai/agentic/models/rowKey";
import { type EntryCatalog, KINDS, type ProviderKind } from "@nyaaai/agentic/provider/kind";
import {
	type Catalog,
	keySourceOf,
	ModelResolver,
	type Resolved,
	reachOf,
	routeKey,
	servedBy,
	viaOf,
} from "@nyaaai/agentic/provider/resolver";
import type { KeyAnswer } from "@nyaaai/contract/entryKeys";
import type { Fetch, FetchRequest } from "@nyaaai/contract/fetch";
import { canonicalJson } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { type ModelRow, ModelUse, ModelView, Unresolved, Via } from "@nyaaai/contract/shapes/model";
import { KeyReach, Passed, ProviderEntry, ProviderKindName } from "@nyaaai/contract/shapes/provider";
import { type CanonicalRequest, Failure, type StepResult } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedEntryKeys, scriptedModelRegistry, scriptedProviderList } from "@nyaaai/testkit/scriptedPorts";
import { resolverCases, resolverServedCases, resolverUseCases } from "../caseSources/resolver";
import { resolverKeyCases } from "../caseSources/resolverKey";
import { withUses } from "../caseSources/uses";
import { rowOfKey } from "./catalogKit";
import { localRowOf } from "./localKit";

////////////////////////////////
//  Shapes

const ViewStatus = z.enum(["ok", "retired", "noZdrHost", "noTools", "unknown"]);

/** What `EntryKeys.get` answers. */
const KeyAnswerIn = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("key"), key: z.string() }),
	z.object({ kind: z.literal("none") }),
	z.object({ kind: z.literal("refused") }),
	z.object({ kind: z.literal("unready") }),
]);

/** What a catalog lists of a row for an entry. */
const Listing = z.object({
	/** `unlisted`: the catalog has no view of the row. */
	status: z.union([ViewStatus, z.literal("unlisted")]),
	/** The status with zero retention off; the same when absent. */
	whenZdrOff: z.union([ViewStatus, z.literal("unlisted")]).exactOptional(),
});

/** A route as a request sees it. */
const Candidate = z.object({
	entry: z.string(),
	kind: ProviderKindName,
	binding: z.string(),
	status: ViewStatus,
	/** The switch as it stands when read. */
	zdrOnly: z.boolean(),
	/** The path the codec encodes a request to. */
	path: z.string(),
});

/** A change of a scripted port; it tells its listeners. */
const CHANGES = [
	/** The list changes. */
	z.object({ kind: z.literal("list"), entries: z.array(ProviderEntry) }),
	/** An entry's key answer changes. */
	z.object({ kind: z.literal("answer"), id: z.string(), answer: KeyAnswerIn }),
	/** The registry's rows change. */
	z.object({ kind: z.literal("registry"), rows: z.array(z.string()) }),
	z.object({ kind: z.literal("zdr"), on: z.boolean() }),
] as const;

/** A resolver call, or a request through one of its routes. */
const CALLS = [
	z.object({ kind: z.literal("routes"), model: z.string(), use: ModelUse }),
	z.object({ kind: z.literal("resolve"), model: z.string(), use: ModelUse }),
	z.object({ kind: z.literal("rows"), use: ModelUse }),
	z.object({ kind: z.literal("candidates"), use: ModelUse }),
	/** A request through an earlier candidate; `fail` is the host's answer. */
	z.object({
		kind: z.literal("send"),
		from: z.int().min(0),
		candidate: z.int().min(0),
		mode: z.enum(["complete", "stream"]).exactOptional(),
		fail: z.object({ status: z.int(), body: z.unknown() }).exactOptional(),
	}),
] as const;

const RouteStep = () =>
	z.discriminatedUnion("kind", [
		...CHANGES,
		...CALLS,
		/** The catalog of an entry's route says it refreshed. */
		z.object({ kind: z.literal("refreshed"), entry: z.string() }),
		/** The route of an entry is asked for directly. */
		z.object({ kind: z.literal("route"), entry: ProviderEntry }),
		z.object({ kind: z.literal("dispose") }),
		/** A call whose first key read is held during the changes. */
		z.object({
			kind: z.literal("during"),
			changes: z.array(z.discriminatedUnion("kind", CHANGES)),
			call: z.discriminatedUnion("kind", CALLS),
		}),
	]);

const StepAnswer = () =>
	z.discriminatedUnion("kind", [
		z.object({
			kind: z.literal("routes"),
			unresolved: Unresolved.nullable(),
			candidates: z.array(Candidate),
			passed: z.array(Passed),
		}),
		z.object({ kind: z.literal("resolve"), unresolved: Unresolved.nullable(), candidate: Candidate.nullable() }),
		z.object({
			kind: z.literal("rows"),
			options: z.array(z.object({ key: z.string(), status: ViewStatus, via: Via.nullable() })),
		}),
		z.object({
			kind: z.literal("candidates"),
			served: z.array(z.object({ key: z.string(), status: ViewStatus, via: Via })),
		}),
		z.object({ kind: z.literal("changed") }),
		/** What the host saw: the switch, URL and key. */
		z.object({
			kind: z.literal("sent"),
			zdrOnly: z.boolean().nullable(),
			url: z.string().nullable(),
			authorization: z.string().nullable(),
			failure: Failure.nullable(),
			thrown: z.string().nullable(),
		}),
		/** A direct `route` call. */
		z.object({ kind: z.literal("routed"), thrown: z.string().nullable() }),
		z.object({ kind: z.literal("none") }),
	]);

type AnswerOf = z.infer<ReturnType<typeof StepAnswer>>;
type StepOf = z.infer<ReturnType<typeof RouteStep>>;

////////////////////////////////
//  Functions & Helpers

/** A row by key: curated, hosted `provider:id`, or local `local:<entry>/<tag>`. */
function rowFor(key: string): ModelRow {
	return parseRowKey(key)?.kind === "local" ? localRowOf(key) : rowOfKey(key);
}

/** The name a transport is told by. */
function labelOf(entry: { label: string; plan?: string | undefined; url?: string | undefined }): string {
	return `${entry.label}${entry.plan === undefined ? "" : `/${entry.plan}`}${entry.url === undefined ? "" : ` ${entry.url}`}`;
}

const REQUEST: CanonicalRequest = {
	model: "m",
	system: [],
	history: [],
	cache: { kind: "none" },
	session: "session-1",
	zdrOnly: true,
};

/** What the host saw of a request. */
interface Seen {
	url: string;
	authorization: string | null;
}

const UTF8 = new TextEncoder();

/** A host answering success, or the failure it was told to. */
function echoHost(seen: Seen[], failing: { status: number; body: unknown } | undefined = undefined) {
	const holder = { next: failing };
	const fetch: Fetch = async (request: FetchRequest) => {
		seen.push({
			url: request.url,
			authorization: request.headers.find(([name]) => name === "Authorization")?.[1] ?? null,
		});
		const asked = request.headers.find(([name]) => name === "Accept")?.[1] ?? "";
		const failure = holder.next;
		holder.next = undefined;
		const stream = failure === undefined && asked === "text/event-stream";
		const body = failure === undefined ? (stream ? "data: {}\n\n" : "{}") : JSON.stringify(failure.body);
		return {
			status: failure?.status ?? 200,
			headers: [["content-type", stream ? "text/event-stream" : "application/json"]],
			chunks: (async function* () {
				yield UTF8.encode(body);
			})(),
		};
	};
	return {
		fetch,
		/** The next request answers this. */
		fail: (failure: { status: number; body: unknown } | undefined) => {
			holder.next = failure;
		},
	};
}

const NOTHING: StepResult = { parts: [], finish: "stop", serving: {} };

/** A request through a route: the codec's input and the host's view. */
async function sendThrough(
	found: Resolved,
	seen: Seen[],
	host: ReturnType<typeof echoHost>,
	step: { mode?: "complete" | "stream" | undefined; fail?: { status: number; body: unknown } | undefined },
): Promise<Extract<AnswerOf, { kind: "sent" }>> {
	let zdrOnly: boolean | null = null;
	const codec: Codec = {
		...found.codec,
		encode: (request, mode) => {
			zdrOnly = request.zdrOnly;
			return found.codec.encode(request, mode);
		},
		complete: () => NOTHING,
		stream: async function* (items) {
			for await (const _ of items) {
				// Only the request is asked about.
			}
			yield* [];
		},
	};
	const before = seen.length;
	host.fail(step.fail);
	const request = { ...REQUEST, model: found.key };
	const signal = new AbortController().signal;
	let failure: z.infer<typeof Failure> | null = null;
	let thrown: string | null = null;
	try {
		if (step.mode === "stream") for await (const _ of found.transport.stream(codec, request, signal));
		else await found.transport.complete(codec, request, signal);
	} catch (error) {
		failure = failureOf(error) ?? null;
		thrown = failure === null ? (error instanceof Error ? error.message : String(error)) : null;
	}
	host.fail(undefined);
	const sent = seen.length > before ? seen[seen.length - 1] : undefined;
	return {
		kind: "sent",
		zdrOnly,
		url: sent?.url ?? null,
		authorization: sent?.authorization ?? null,
		failure,
		thrown,
	};
}

/** The names of the six kinds of NyaaAI. */
const NAMES = ProviderKindName.options;

const KEY = "fake-key-one";

const candidateOf = (resolved: Resolved): z.infer<typeof Candidate> => ({
	entry: resolved.entry.id,
	kind: resolved.kind.kind,
	binding: resolved.binding.id,
	status: resolved.view.status,
	zdrOnly: resolved.zdrOnly,
	path: resolved.codec.encode(REQUEST, "complete").path,
});

////////////////////////////////
//  Group

export const resolver = caseGroup("resolver", {
	kinds: {
		/** What a key answer says of an entry, per key kind and URL kind. */
		reach: kind(
			z.object({
				rows: z.array(z.object({ field: z.enum(["key", "url"]), answer: KeyAnswerIn })),
			}),
			z.object({ reaches: z.array(KeyReach) }),
			(input) => ({ reaches: input.rows.map((row) => reachOf({ field: row.field }, row.answer)) }),
		),
		/** What identifies a route. */
		routeKey: kind(
			z.object({ entries: z.array(ProviderEntry) }),
			z.object({ keys: z.array(z.string()) }),
			(input) => ({ keys: input.entries.map(routeKey) }),
		),
		/** A route's key; answers only while the list holds the entry at its URL. */
		keySource: kind(
			z.object({
				/** The entry the route was made for. */
				entry: ProviderEntry,
				list: z.array(ProviderEntry),
				answers: z.record(z.string(), KeyAnswerIn),
				steps: z.array(
					z.discriminatedUnion("kind", [
						z.object({ kind: z.literal("get") }),
						z.object({ kind: z.literal("list"), entries: z.array(ProviderEntry) }),
						z.object({ kind: z.literal("answer"), id: z.string(), answer: KeyAnswerIn }),
					]),
				),
			}),
			z.object({ gets: z.array(z.string().nullable()) }),
			async (input) => {
				const keys = scriptedEntryKeys(input.answers as Record<string, KeyAnswer>);
				const list = scriptedProviderList(input.list);
				const source = keySourceOf(keys, list, input.entry);
				const gets: Array<string | null> = [];
				for (const step of input.steps) {
					if (step.kind === "get") gets.push((await source.get()) ?? null);
					else if (step.kind === "list") list.set(step.entries);
					else keys.set(step.id, step.answer);
				}
				return { gets };
			},
		),
		/** A plan that serves one entry's key. */
		servedBy: kind(
			z.object({
				kind: ProviderKindName,
				entry: ProviderEntry,
				key: z.string(),
				use: ModelUse,
				/** The catalog's view of the key's row; null is no view. */
				view: ViewStatus.nullable(),
				zdrOnly: z.boolean(),
				/** The switch the request meets. */
				flippedTo: z.boolean(),
			}),
			z.object({
				key: z.string(),
				entry: z.string(),
				binding: z.string(),
				view: ModelView,
				path: z.string(),
				passed: z.array(Passed),
				candidates: z.int(),
				zdrOnResolve: z.boolean(),
				sent: z.object({ zdrOnly: z.boolean(), path: z.string() }),
			}),
			async (input) => {
				const named = KINDS[input.kind];
				if (named === undefined) throw new Error(`No ${input.kind} kind.`);
				let zdr = input.zdrOnly;
				const catalog: Catalog = {
					view: async (key) =>
						input.view === null
							? undefined
							: { row: rowFor(key), status: input.view, efforts: [], hosts: [] },
				};
				const seen: Seen[] = [];
				const host = echoHost(seen);
				const deps = {
					fetch: host.fetch,
					scheduler: fakeScheduler(new TimerWheel(0)),
					log: () => {},
					userAgent: "cases/1.0",
				};
				const transport = named.transport(input.entry, { get: async () => KEY }, deps);
				const plan = servedBy({
					entry: input.entry,
					kind: named,
					transport,
					catalog,
					deps,
					zdrOnly: () => zdr,
				});
				const routes = await plan(input.key, input.use);
				if (!routes.ok) throw new Error("A plan serves its entry.");
				const [first] = routes.candidates;
				const zdrOnResolve = first.zdrOnly;
				zdr = input.flippedTo;
				const sent = await sendThrough(first, seen, host, {});
				return {
					key: routes.key,
					entry: first.entry.id,
					binding: first.binding.id,
					view: first.view,
					path: first.codec.encode(REQUEST, "complete").path,
					passed: [...routes.passed],
					candidates: routes.candidates.length,
					zdrOnResolve,
					sent: { zdrOnly: sent.zdrOnly ?? false, path: first.codec.encode(REQUEST, "complete").path },
				};
			},
		),
		/** The uses catalogs were asked with: the value, never a name. */
		uses: kind(
			z.object({ rows: z.array(z.string()), uses: z.array(ModelUse) }),
			z.object({
				calls: z.array(
					z.object({
						call: z.enum(["routes", "resolve", "rows", "candidates"]),
						use: ModelUse,
						/** The distinct uses the catalogs were asked with, in first asked order. */
						asked: z.array(ModelUse),
					}),
				),
			}),
			async (input) => {
				const asked: Array<z.infer<typeof ModelUse>> = [];
				const base = KINDS.openrouter;
				if (base === undefined) throw new Error("No kind.");
				const stub: ProviderKind = {
					...base,
					catalog: (_entry, _transport, deps) => ({
						views: async (use) => {
							if (!asked.some((each) => canonicalJson(each) === canonicalJson(use))) asked.push(use);
							return deps.rows
								.rows()
								.flatMap((row) => [{ row, status: "ok" as const, efforts: [], hosts: [] }]);
						},
						idle: async () => {},
					}),
				};
				const entry = { id: "a", kind: "openrouter" as const, label: "First" };
				const resolver = new ModelResolver({
					list: scriptedProviderList([entry]),
					keys: scriptedEntryKeys({ a: { kind: "key", key: KEY } }),
					registry: scriptedModelRegistry(input.rows.map(rowFor)),
					kinds: { openrouter: stub },
					kindDeps: () => ({
						fetch: echoHost([]).fetch,
						scheduler: fakeScheduler(new TimerWheel(0)),
						log: () => {},
						userAgent: "cases/1.0",
					}),
					zdrOnly: () => true,
				});
				const calls: Array<{
					call: "routes" | "resolve" | "rows" | "candidates";
					use: z.infer<typeof ModelUse>;
					asked: Array<z.infer<typeof ModelUse>>;
				}> = [];
				const model = input.rows[0] ?? "";
				for (const use of input.uses) {
					for (const call of ["routes", "resolve", "rows", "candidates"] as const) {
						asked.length = 0;
						if (call === "routes") await resolver.routes(model, use);
						else if (call === "resolve") await resolver.resolve(model, use);
						else if (call === "rows") await resolver.rows(use);
						else await resolver.candidates(use);
						calls.push({ call, use, asked: [...asked] });
					}
				}
				resolver.dispose();
				return { calls };
			},
		),
		/** A resolver over scripted ports, kinds and stub catalogs. */
		routes: kind(
			z.object({
				entries: z.array(ProviderEntry),
				/** The key answers by entry id; an entry with none answers none. */
				answers: z.record(z.string(), KeyAnswerIn),
				/** Entries whose key store cannot be read: the port answers `unready`. */
				failing: z.array(z.string()),
				/** The registry's rows, by key. */
				rows: z.array(z.string()),
				/** The kinds the resolver holds. */
				kinds: z.array(ProviderKindName),
				/** Each entry's catalog listing by row key; an unnamed row is `ok`. */
				listings: z.record(z.string(), z.record(z.string(), Listing)),
				/** Entries whose catalog fails when stopped. */
				disposeFails: z.array(z.string()).exactOptional(),
				/** The scheduler's first instant. */
				startMs: z.int().exactOptional(),
				zdrOnly: z.boolean(),
				steps: z.array(RouteStep()).min(1),
			}),
			z.object({
				steps: z.array(
					z.object({
						answer: StepAnswer(),
						/** Transports made so far. */
						built: z.int(),
						/** Catalogs stopped so far. */
						disposed: z.int(),
					}),
				),
				/** Transports made, by label; one step's in list order. */
				transports: z.array(z.string()),
				/** The catalogs stopped, in order, by label. */
				stopped: z.array(z.string()),
				/** The entries told a catalog refreshed, as told. */
				refreshed: z.array(Via),
				/** The key reads made. */
				keyReads: z.int(),
			}),
			async (input) => {
				const wheel = new TimerWheel(input.startMs ?? 0);
				const scheduler = fakeScheduler(wheel);
				const scriptedKeys = scriptedEntryKeys(input.answers as Record<string, KeyAnswer>);
				let armed: { reached: () => void; release: Promise<void> } | null = null;
				const keys = {
					get: async (id: string): Promise<KeyAnswer> => {
						const gate = armed;
						if (gate !== null) {
							armed = null;
							gate.reached();
							await gate.release;
						}
						return input.failing.includes(id) ? { kind: "unready" } : scriptedKeys.get(id);
					},
					onChange: scriptedKeys.onChange,
				};
				const list = scriptedProviderList(input.entries);
				const registry = scriptedModelRegistry(input.rows.map(rowFor));
				const transports: string[] = [];
				const madeNow: Array<{ id: string; label: string }> = [];
				const stopped: string[] = [];
				const refreshed: Array<z.infer<typeof Via>> = [];
				const refreshes = new Map<string, () => void>();
				const seen: Seen[] = [];
				const host = echoHost(seen);
				let zdr = input.zdrOnly;

				const stub = (name: z.infer<typeof ProviderKindName>): ProviderKind => {
					const real = KINDS[name];
					if (real === undefined) throw new Error(`No ${name} kind.`);
					return {
						...real,
						transport: (entry, key, deps) => {
							const made = real.transport(entry, key, deps);
							madeNow.push({ id: (entry as ProviderEntry).id, label: labelOf(entry) });
							return made;
						},
						catalog: (entry, _transport, deps) => {
							refreshes.set(entry.id, deps.refreshed);
							const catalog: EntryCatalog = {
								views: async (_use, zdrOnly) =>
									deps.rows.rows().flatMap((row): ModelView[] => {
										const bound = row.bindings[name] !== undefined;
										const elsewhere =
											row.identity.kind === "local" && row.identity.entry !== entry.id;
										if (!bound || elsewhere) return [];
										const held = input.listings[entry.id]?.[row.key];
										const status = zdrOnly ? held?.status : (held?.whenZdrOff ?? held?.status);
										if (status === "unlisted") return [];
										return [{ row, status: status ?? "ok", efforts: [], hosts: [] }];
									}),
								idle: async () => {},
								dispose: () => {
									stopped.push(labelOf(entry));
									if (input.disposeFails?.includes(entry.id) === true)
										throw new Error("Stopping failed.");
								},
							};
							return catalog;
						},
					};
				};

				const kinds: Partial<Record<z.infer<typeof ProviderKindName>, ProviderKind>> = {};
				for (const name of NAMES) if (input.kinds.includes(name)) kinds[name] = stub(name);

				const resolver = new ModelResolver({
					list,
					keys,
					registry,
					kinds,
					kindDeps: () => ({ fetch: host.fetch, scheduler, log: () => {}, userAgent: "cases/1.0" }),
					zdrOnly: () => zdr,
					refreshed: (entry) => refreshed.push(viaOf(entry)),
				});

				const held: Resolved[][] = [];
				const steps: Array<{ answer: AnswerOf; built: number; disposed: number }> = [];

				const change = (step: z.infer<(typeof CHANGES)[number]>): void => {
					if (step.kind === "list") list.set(step.entries);
					else if (step.kind === "answer") scriptedKeys.set(step.id, step.answer);
					else if (step.kind === "registry") registry.setRows(step.rows.map(rowFor));
					else zdr = step.on;
				};

				const called = async (step: z.infer<(typeof CALLS)[number]>): Promise<AnswerOf> => {
					switch (step.kind) {
						case "routes": {
							const routes = await resolver.routes(step.model, step.use);
							held[held.length - 1] = routes.ok ? [...routes.candidates] : [];
							return routes.ok
								? {
										kind: "routes",
										unresolved: null,
										candidates: routes.candidates.map(candidateOf),
										passed: [...routes.passed],
									}
								: { kind: "routes", unresolved: routes, candidates: [], passed: [] };
						}
						case "resolve": {
							const resolved = await resolver.resolve(step.model, step.use);
							held[held.length - 1] = resolved.ok ? [resolved] : [];
							return resolved.ok
								? { kind: "resolve", unresolved: null, candidate: candidateOf(resolved) }
								: { kind: "resolve", unresolved: resolved, candidate: null };
						}
						case "rows":
							return {
								kind: "rows",
								options: (await resolver.rows(step.use)).map((option) => ({
									key: option.key,
									status: option.view.status,
									via: option.via ?? null,
								})),
							};
						case "candidates":
							return {
								kind: "candidates",
								served: (await resolver.candidates(step.use)).map((option) => ({
									key: option.key,
									status: option.view.status,
									via: option.via,
								})),
							};
						case "send": {
							const found = held[step.from]?.[step.candidate];
							if (found === undefined) return { kind: "none" };
							return sendThrough(found, seen, host, step);
						}
					}
				};

				const said = async (step: StepOf): Promise<AnswerOf> => {
					switch (step.kind) {
						case "list":
						case "answer":
						case "registry":
						case "zdr":
							change(step);
							return { kind: "changed" };
						case "refreshed":
							refreshes.get(step.entry)?.();
							return { kind: "changed" };
						case "route":
							try {
								resolver.route(step.entry);
								return { kind: "routed", thrown: null };
							} catch (error) {
								return {
									kind: "routed",
									thrown: error instanceof Error ? error.message : String(error),
								};
							}
						case "dispose":
							resolver.dispose();
							return { kind: "changed" };
						case "during": {
							let reached = () => {};
							const arrived = new Promise<void>((resolve) => {
								reached = resolve;
							});
							let release = () => {};
							armed = {
								reached: () => reached(),
								release: new Promise<void>((resolve) => {
									release = resolve;
								}),
							};
							const call = called(step.call);
							await Promise.race([
								arrived,
								call.then(
									() => undefined,
									() => undefined,
								),
							]);
							for (const each of step.changes) change(each);
							armed = null;
							release();
							return call;
						}
						default:
							return called(step);
					}
				};

				for (const step of input.steps) {
					held.push([]);
					const answer = await said(step);
					const position = (id: string) => {
						const at = list.entries().findIndex((each) => each.id === id);
						return at < 0 ? Number.MAX_SAFE_INTEGER : at;
					};
					madeNow.sort((left, right) => position(left.id) - position(right.id));
					transports.push(...madeNow.splice(0).map((each) => each.label));
					steps.push({ answer, built: transports.length, disposed: stopped.length });
				}
				resolver.dispose();
				return {
					steps,
					transports,
					stopped,
					refreshed,
					keyReads: scriptedKeys.reads(),
				};
			},
		),
	},
	sources: withUses(casesFor("resolver", resolverCases, resolverServedCases, resolverKeyCases, resolverUseCases)),
});
