import type { Codec } from "@nyaaai/agentic/codec";
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
} from "@nyaaai/agentic/provider/resolver";
import type { Transport } from "@nyaaai/agentic/transport";
import type { KeyAnswer } from "@nyaaai/contract/entryKeys";
import type { Fetch } from "@nyaaai/contract/fetch";
import { z } from "@nyaaai/contract/shape";
import { type ModelRow, ModelUse, ModelView, Unresolved, Via } from "@nyaaai/contract/shapes/model";
import { KeyReach, Passed, ProviderEntry, ProviderKindName } from "@nyaaai/contract/shapes/provider";
import type { CanonicalRequest } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedEntryKeys, scriptedModelRegistry, scriptedProviderList } from "@nyaaai/testkit/scriptedPorts";
import { resolverCases, resolverServedCases } from "../caseSources/resolver";
import { resolverKeyCases } from "../caseSources/resolverKey";
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

const RouteStep = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("routes"), model: z.string(), use: ModelUse }),
		z.object({ kind: z.literal("resolve"), model: z.string(), use: ModelUse }),
		z.object({ kind: z.literal("rows"), use: ModelUse }),
		z.object({ kind: z.literal("candidates"), use: ModelUse }),
		/** The list changes and tells. */
		z.object({ kind: z.literal("list"), entries: z.array(ProviderEntry) }),
		/** An entry's key answer changes and tells. */
		z.object({ kind: z.literal("answer"), id: z.string(), answer: KeyAnswerIn }),
		/** The registry's rows change and tell. */
		z.object({ kind: z.literal("registry"), rows: z.array(z.string()) }),
		z.object({ kind: z.literal("zdr"), on: z.boolean() }),
		/** The catalog of an entry's route says it refreshed. */
		z.object({ kind: z.literal("refreshed"), entry: z.string() }),
		/** A request goes through a candidate an earlier `routes` or `resolve` step answered. */
		z.object({ kind: z.literal("send"), from: z.int().min(0), candidate: z.int().min(0) }),
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
		/** What the transport saw: its entry, the switch it was sent, the key it read. */
		z.object({
			kind: z.literal("sent"),
			transport: z.string(),
			zdrOnly: z.boolean(),
			key: z.string().nullable(),
			path: z.string(),
		}),
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

const NO_NETWORK: Fetch = async () => {
	throw new Error("No network.");
};

/** What a stub transport answers to a request. */
interface Sent {
	transport: string;
	zdrOnly: boolean;
	key: string | null;
	path: string;
}

/** A transport that answers each request with what it saw. */
function stubTransport(label: string, key: { get(): Promise<string | undefined> }): Transport {
	return {
		complete: async (codec: Codec, request: CanonicalRequest): Promise<Sent> => ({
			transport: label,
			zdrOnly: request.zdrOnly,
			key: (await key.get()) ?? null,
			path: codec.encode(request, "complete").path,
		}),
		stream: async () => {
			throw new Error("No stream.");
		},
		getJson: async () => {
			throw new Error("No network.");
		},
	} as unknown as Transport;
}

/** The names of the six kinds of NyaaAI. */
const NAMES = ProviderKindName.options;

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
		/** What a key answer says of an entry, for a key kind and a URL kind. */
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
		/** One route's key: it answers only while the list holds the entry at the URL the route was made for. */
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
				const transport = stubTransport(input.entry.label, { get: async () => undefined });
				const plan = servedBy({
					entry: input.entry,
					kind: named,
					transport,
					catalog,
					zdrOnly: () => zdr,
				});
				const routes = await plan(input.key, input.use);
				if (!routes.ok) throw new Error("A plan serves its entry.");
				const [first] = routes.candidates;
				const zdrOnResolve = first.zdrOnly;
				zdr = input.flippedTo;
				const sent = (await first.transport.complete(
					first.codec,
					{ ...REQUEST, model: input.key },
					new AbortController().signal,
				)) as unknown as Sent;
				return {
					key: routes.key,
					entry: first.entry.id,
					binding: first.binding.id,
					view: first.view,
					path: first.codec.encode(REQUEST, "complete").path,
					passed: [...routes.passed],
					candidates: routes.candidates.length,
					zdrOnResolve,
					sent: { zdrOnly: sent.zdrOnly, path: sent.path },
				};
			},
		),
		/** A resolver over scripted ports and stub kinds, driven by a script. */
		routes: kind(
			z.object({
				entries: z.array(ProviderEntry),
				/** The key answers by entry id; an entry with none answers none. */
				answers: z.record(z.string(), KeyAnswerIn),
				/** Entries whose key read fails. */
				failing: z.array(z.string()),
				/** The registry's rows, by key. */
				rows: z.array(z.string()),
				/** The kinds the resolver holds. */
				kinds: z.array(ProviderKindName),
				/** What each entry's catalog lists, per row key; a row not named is listed `ok`. */
				listings: z.record(z.string(), z.record(z.string(), Listing)),
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
				/** The transports made, in order, by label. */
				transports: z.array(z.string()),
				/** The catalogs stopped, in order, by label. */
				stopped: z.array(z.string()),
				/** The entries told a catalog refreshed. */
				refreshed: z.array(z.string()),
				/** The key reads made. */
				keyReads: z.int(),
				/** The list reads made. */
				listReads: z.int(),
			}),
			async (input) => {
				const wheel = new TimerWheel(0);
				const scheduler = fakeScheduler(wheel);
				const scriptedKeys = scriptedEntryKeys(input.answers as Record<string, KeyAnswer>);
				const keys = {
					get: (id: string) =>
						input.failing.includes(id)
							? Promise.reject(new Error("Key read failed."))
							: scriptedKeys.get(id),
					onChange: scriptedKeys.onChange,
				};
				const list = scriptedProviderList(input.entries);
				const registry = scriptedModelRegistry(input.rows.map(rowFor));
				const transports: string[] = [];
				const stopped: string[] = [];
				const refreshed: string[] = [];
				const refreshes = new Map<string, () => void>();
				let zdr = input.zdrOnly;

				const stub = (name: z.infer<typeof ProviderKindName>): ProviderKind => {
					const real = KINDS[name];
					if (real === undefined) throw new Error(`No ${name} kind.`);
					return {
						...real,
						transport: (entry, key) => {
							const label = labelOf(entry);
							transports.push(label);
							return stubTransport(label, key);
						},
						catalog: (entry, _transport, deps) => {
							refreshes.set(entry.id, deps.refreshed);
							const catalog: EntryCatalog & { dispose(): void } = {
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
								dispose: () => void stopped.push(labelOf(entry)),
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
					kindDeps: () => ({ fetch: NO_NETWORK, scheduler, log: () => {}, userAgent: "cases/1.0" }),
					zdrOnly: () => zdr,
					refreshed: (entry) => refreshed.push(entry.id),
				});

				const held: Resolved[][] = [];
				const steps: Array<{ answer: AnswerOf; built: number; disposed: number }> = [];
				const said = async (step: StepOf): Promise<AnswerOf> => {
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
						case "list":
							list.set(step.entries);
							return { kind: "changed" };
						case "answer":
							scriptedKeys.set(step.id, step.answer);
							return { kind: "changed" };
						case "registry":
							registry.setRows(step.rows.map(rowFor));
							return { kind: "changed" };
						case "zdr":
							zdr = step.on;
							return { kind: "changed" };
						case "refreshed":
							refreshes.get(step.entry)?.();
							return { kind: "changed" };
						case "send": {
							const found = held[step.from]?.[step.candidate];
							if (found === undefined) return { kind: "none" };
							const sent = (await found.transport.complete(
								found.codec,
								{ ...REQUEST, model: found.key },
								new AbortController().signal,
							)) as unknown as Sent;
							return {
								kind: "sent",
								transport: sent.transport,
								zdrOnly: sent.zdrOnly,
								key: sent.key,
								path: sent.path,
							};
						}
					}
				};

				for (const step of input.steps) {
					held.push([]);
					const answer = await said(step);
					steps.push({ answer, built: transports.length, disposed: stopped.length });
				}
				resolver.dispose();
				return {
					steps,
					transports,
					stopped,
					refreshed,
					keyReads: scriptedKeys.reads(),
					listReads: list.reads(),
				};
			},
		),
	},
	sources: casesFor("resolver", resolverCases, resolverServedCases, resolverKeyCases),
});
