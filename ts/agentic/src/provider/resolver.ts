// Routes a model key across the provider list.

import type { EntryKeys, KeyAnswer } from "@nyaaai/contract/entryKeys";
import type { ModelRegistry } from "@nyaaai/contract/modelRegistry";
import type { Binding, ModelRow, ModelUse, ModelView, Refusal, Unresolved, Via } from "@nyaaai/contract/shapes/model";
import type { KeyReach, Passed, ProviderEntry, ProviderKindName, SkipReason } from "@nyaaai/contract/shapes/provider";
import type { Codec } from "../codec";
import { joinAll } from "../kinds/join";
import { parseRowKey, servedIdOf } from "../models/rowKey";
import { unknownView } from "../models/view";
import type { KeySource, Transport } from "../transport";
import { type EntryRegistry, type Incarnation, identityOf, type RegistryEvent } from "./incarnation";
import type { KindDeps, ProviderKind, Route } from "./kind";

////////////////////////////////
//  Interfaces & Types

/** One model's view by key. */
export interface Catalog {
	view(key: string, use?: ModelUse): Promise<ModelView | undefined>;
}

export interface Resolved {
	ok: true;
	key: string;
	/** The row when resolved. */
	entry: ProviderEntry;
	/** Sends nothing once it ended. */
	incarnation: Incarnation;
	kind: ProviderKind;
	binding: Binding;
	/** The serving entry's price, context and efforts. */
	view: ModelView;
	/** Each request goes with the ZDR switch as it stands then. */
	transport: Pick<Transport, "stream" | "complete" | "getJson">;
	codec: Codec;
	/** Zero-retention routes only; read live. */
	readonly zdrOnly: boolean;
}

/** At least one route. */
export type Candidates = readonly [Resolved, ...Resolved[]];

export type Resolution = Resolved | Unresolved;

/** Eligible routes, priority order. */
export interface Routes {
	ok: true;
	key: string;
	candidates: Candidates;
	/** Dropped entries and reasons. */
	passed: readonly Passed[];
}

export type Plan = (key: string, use: ModelUse) => Promise<Routes | Unresolved>;

/** A row under one use. */
export interface ResolvedOption {
	key: string;
	/** The serving entry's view; unserved, the view behind the refusal, else unknown. */
	view: ModelView;
	/** Absent when no entry serves it. */
	via?: Via;
}

export type Served = ResolvedOption & { via: Via };

/** What a resolver stands on. */
export interface ResolverDeps {
	/** Incarnations in priority order. */
	entries: EntryRegistry;
	/** Key reads go through it. */
	keys: Pick<EntryKeys, "get">;
	registry: Pick<ModelRegistry, "rows" | "onChange">;
	kinds: Partial<Record<ProviderKindName, ProviderKind>>;
	/** What a kind's transport stands on. */
	kindDeps(kind: ProviderKindName): KindDeps;
	zdrOnly(): boolean;
	/** Told after a catalog's refresh. */
	refreshed?(entry: ProviderEntry): void;
}

/** What a resolver made for one incarnation. */
interface Made {
	route: Route;
	deps: KindDeps;
}

/** Top refusal and view. */
interface Unserved {
	refusal: Refusal;
	view?: ModelView;
	entry?: string;
}

type Routed = { ok: true; candidates: Candidates; passed: Passed[] } | ({ ok: false } & Unserved);

////////////////////////////////
//  Constants

const REACH_SKIPS: Record<Exclude<KeyReach, "ok">, SkipReason> = {
	keyMissing: "keyMissing",
	keyRejected: "keyRejected",
	unreachable: "network",
};

/** Higher rank wins. */
const RANK: Record<Refusal, number> = {
	noModel: 1,
	modelRetired: 2,
	noZdrHost: 3,
	unreachable: 4,
	keyRejected: 5,
	keyMissing: 6,
};

/** A later entry's view is waited for this long. */
const LATER_VIEW_MS = 2000;

const OUT = Symbol("out");

interface Bound {
	expired: boolean;
	cancel: () => void;
	late: Promise<void>;
}

////////////////////////////////
//  Functions & Helpers

export function viaOf(entry: ProviderEntry): Via {
	return { entry: entry.id, label: entry.label };
}

/** A URL kind needs no key; a key kind needs one. */
export function reachOf(kind: Pick<ProviderKind, "field">, answer: KeyAnswer): KeyReach {
	switch (answer.kind) {
		case "key":
			return "ok";
		case "none":
			return kind.field === "url" ? "ok" : "keyMissing";
		case "refused":
			return "keyRejected";
		case "unready":
			return "unreachable";
	}
}

/** The key, only while its incarnation lives. */
export function keySourceOf(keys: Pick<EntryKeys, "get">, incarnation: Incarnation): KeySource {
	return {
		get: async () => {
			const answer = await keys.get(incarnation.id).catch((): KeyAnswer => ({ kind: "unready" }));
			return incarnation.live && answer.kind === "key" ? answer.key : undefined;
		},
	};
}

/** Unknown views remain eligible. */
function usable(view: ModelView): boolean {
	return view.status === "ok" || view.status === "unknown";
}

function raised(worst: Unserved, next: Refusal, behind: ModelView | undefined, label: string): Unserved {
	const higher = RANK[next] > RANK[worst.refusal];
	const view = higher ? (behind ?? worst.view) : (worst.view ?? behind);
	const refusal = higher ? next : worst.refusal;
	const entry = higher ? label : (worst.entry ?? label);
	return view === undefined ? { refusal, entry } : { refusal, view, entry };
}

function refusalOf(view: ModelView | undefined): Refusal {
	if (view?.status === "retired") return "modelRetired";
	return view?.status === "noZdrHost" ? "noZdrHost" : "noModel";
}

/** Synthesizes an unknown row; a bare key is the kind's own id. */
function adhocRow(key: string, kind: ProviderKindName): ModelRow {
	const identity = parseRowKey(key) ?? { kind: "hosted", provider: kind, id: key };
	return {
		key,
		identity,
		bindings: { [kind]: { id: identity.kind === "curated" ? key : servedIdOf(identity) } },
		family: key,
		label: key,
		reasoning: false,
		schema: false,
		cache: "implicit",
		verified: "",
		verifiedPrice: { prompt: 0, completion: 0 },
	};
}

function stamped(transport: Resolved["transport"], zdrOnly: () => boolean): Resolved["transport"] {
	return {
		stream: (codec, request, signal) => transport.stream(codec, { ...request, zdrOnly: zdrOnly() }, signal),
		complete: (codec, request, signal) => transport.complete(codec, { ...request, zdrOnly: zdrOnly() }, signal),
		getJson: (path, signal, auth) => transport.getJson(path, signal, auth),
	};
}

/** Stops a catalog that listens; a failure stops nothing else. */
function disposed(catalog: Route["catalog"]): void {
	try {
		catalog.dispose?.();
	} catch {
		// The others still go.
	}
}

/** Resolves through one entry. */
export function servedBy(d: {
	incarnation: Incarnation;
	kind: ProviderKind;
	transport: Resolved["transport"];
	catalog: Catalog;
	deps: KindDeps;
	zdrOnly: () => boolean;
}): Plan {
	const { zdrOnly } = d;
	return async (key, use) => {
		const view = (await d.catalog.view(key, use)) ?? unknownView(adhocRow(key, d.kind.kind));
		const { entry } = d.incarnation;
		const codec = d.kind.codec(view, entry, d.deps);
		const route: Resolved = {
			ok: true,
			key,
			entry,
			incarnation: d.incarnation,
			kind: d.kind,
			binding: view.row.bindings[d.kind.kind] ?? { id: key },
			view,
			transport: stamped(d.transport, zdrOnly),
			codec,
			get zdrOnly() {
				return zdrOnly();
			},
		};
		return { ok: true, key, candidates: [route], passed: [] };
	};
}

////////////////////////////////
//  Classes

/** Routes keys across entries; holds one route per entry as it stands. */
export class ModelResolver {
	/** The routes this resolver made, by incarnation. */
	private readonly made = new Map<Incarnation, Made>();
	private readonly unsubscribe: () => void;
	private closed = false;

	constructor(private readonly d: ResolverDeps) {
		this.unsubscribe = d.entries.watch((events) => this.observe(events));
	}

	/** Stops listening; the routes go; nothing is built after. */
	dispose(): void {
		if (this.closed) return;
		this.closed = true;
		this.unsubscribe();
		for (const incarnation of [...this.made.keys()]) this.release(incarnation);
	}

	/** The incarnation's route, made once. */
	route(entry: ProviderEntry): Route {
		if (this.closed) throw new Error("The resolver is disposed.");
		const incarnation = this.d.entries.incarnation(entry.id);
		if (incarnation === undefined || incarnation.identity !== identityOf(entry))
			throw new Error("The entry is not listed.");
		return this.held(incarnation).route;
	}

	private held(incarnation: Incarnation): Made {
		if (this.closed) throw new Error("The resolver is disposed.");
		const known = this.made.get(incarnation);
		if (known !== undefined) return known;
		const { entry } = incarnation;
		const kind = this.d.kinds[entry.kind];
		if (kind === undefined) throw new Error(`No ${entry.kind} provider kind.`);
		const deps = this.d.kindDeps(entry.kind);
		const transport = kind.transport(entry, keySourceOf(this.d.keys, incarnation), deps);
		const catalog = kind.catalog(entry, transport, {
			scheduler: deps.scheduler,
			rows: this.d.registry,
			refreshed: () => this.d.refreshed?.(incarnation.entry),
		});
		const made = { route: { transport, catalog }, deps };
		this.made.set(incarnation, made);
		return made;
	}

	/** Stops its route. */
	private release(incarnation: Incarnation): void {
		const made = this.made.get(incarnation);
		if (made === undefined) return;
		this.made.delete(incarnation);
		disposed(made.route.catalog);
	}

	/** What a key says of one entry. */
	reach(entry: ProviderEntry): Promise<KeyReach> {
		return this.d.keys.get(entry.id).then(
			(answer) => reachOf({ field: this.d.kinds[entry.kind]?.field ?? "key" }, answer),
			(): KeyReach => "unreachable",
		);
	}

	async resolve(key: string, use: ModelUse): Promise<Resolution> {
		const routes = await this.routes(key, use);
		return routes.ok ? routes.candidates[0] : routes;
	}

	/** Eligible entries, priority order. */
	async routes(key: string, use: ModelUse): Promise<Routes | Unresolved> {
		const row = this.d.registry.rows().find((each) => each.key === key);
		if (row === undefined) return { ok: false, key, refusal: "noModel" };
		const routed = await this.routed(row, use, this.d.zdrOnly(), this.reaches(), false, true);
		return routed.ok
			? { ok: true, key, candidates: routed.candidates, passed: routed.passed }
			: {
					ok: false,
					key,
					refusal: routed.refusal,
					...(routed.entry === undefined ? {} : { entry: routed.entry }),
				};
	}

	/** Settings rows in order. */
	async rows(use: ModelUse): Promise<ResolvedOption[]> {
		const zdrOnly = this.d.zdrOnly();
		const reach = this.reaches();
		return joinAll(
			this.d.registry.rows().map((row) => async (): Promise<ResolvedOption> => {
				const routed = await this.routed(row, use, zdrOnly, reach, true, false);
				if (!routed.ok) return { key: row.key, view: routed.view ?? unknownView(row) };
				const [first] = routed.candidates;
				return { key: row.key, view: first.view, via: viaOf(first.entry) };
			}),
		);
	}

	/** Resolvable keys with their serving entries. */
	async candidates(use: ModelUse): Promise<Served[]> {
		return (await this.rows(use)).filter((option): option is Served => option.via !== undefined);
	}

	/** Drops routes of ended incarnations. */
	private observe(events: RegistryEvent[]): void {
		for (const event of events) if (event.kind === "retired") this.release(event.incarnation);
	}

	/** Key check per read. */
	private reaches(): (entry: ProviderEntry) => Promise<KeyReach> {
		const known = new Map<string, Promise<KeyReach>>();
		return (entry) => {
			let reach = known.get(entry.id);
			if (reach === undefined) {
				reach = this.reach(entry);
				known.set(entry.id, reach);
			}
			return reach;
		};
	}

	private async routed(
		row: ModelRow,
		use: ModelUse,
		zdrOnly: boolean,
		reach: (entry: ProviderEntry) => Promise<KeyReach>,
		standing: boolean,
		all: boolean,
	): Promise<Routed> {
		let worst: Unserved = { refusal: "noModel" };
		let matched = false;
		const found: Resolved[] = [];
		const passed: Passed[] = [];
		const listed = this.closed
			? []
			: this.d.entries.incarnations().map((each) => ({ incarnation: each, entry: each.entry }));
		const owner = row.identity.kind === "local" ? row.identity.entry : undefined;
		if (owner !== undefined && !listed.some((each) => each.entry.id === owner))
			return { ok: false, refusal: "noModel" };
		for (const { incarnation, entry } of listed) {
			if (found.length > 0 && !all) break;
			const kind = this.d.kinds[entry.kind];
			const binding = row.bindings[entry.kind];
			const elsewhere = owner !== undefined && owner !== entry.id;
			if (kind === undefined || binding === undefined || elsewhere) continue;
			// A later entry's cold catalog does not hold the answer.
			const bound = found.length > 0 && all ? this.bound(this.d.kindDeps(entry.kind)) : undefined;
			try {
				const reached = await this.bounded(reach(entry), bound);
				if (reached === OUT) continue;
				if (this.closed || !incarnation.live) continue;
				matched = true;
				if (reached !== "ok") {
					passed.push({
						entry: entry.id,
						label: entry.label,
						reason: REACH_SKIPS[reached],
						before: found.length,
					});
					const held = standing && worst.view === undefined ? this.tried(incarnation) : undefined;
					const behind = held === undefined ? undefined : await this.viewOf(held.route, row, use, zdrOnly);
					worst = raised(worst, reached, behind, entry.label);
					continue;
				}
				const held = this.tried(incarnation);
				if (held === undefined) {
					passed.push({ entry: entry.id, label: entry.label, reason: "network", before: found.length });
					worst = raised(worst, "unreachable", undefined, entry.label);
					continue;
				}
				const asked = this.viewOf(held.route, row, use, zdrOnly);
				asked.catch(() => undefined);
				const view = await this.bounded(asked, bound);
				if (view === OUT) continue;
				if (view === undefined || !usable(view)) {
					worst = raised(worst, refusalOf(view), view, entry.label);
					continue;
				}
				const live = () => this.d.zdrOnly();
				found.push({
					ok: true,
					key: row.key,
					entry,
					incarnation,
					kind,
					binding,
					view,
					transport: stamped(held.route.transport, live),
					codec: kind.codec(view, entry, held.deps),
					get zdrOnly() {
						return live();
					},
				});
			} finally {
				bound?.cancel();
			}
		}
		const [first, ...rest] = found;
		if (first !== undefined) return { ok: true, candidates: [first, ...rest], passed };
		return matched ? { ok: false, ...worst } : { ok: false, refusal: "keyMissing" };
	}

	/** The route, or none when it cannot be made. */
	private tried(incarnation: Incarnation): Made | undefined {
		try {
			return this.held(incarnation);
		} catch {
			return undefined;
		}
	}

	/** A read past the bound goes on and is shared. */
	private bound(deps: KindDeps): Bound {
		const bound = { expired: false, cancel: () => {}, late: Promise.resolve() };
		bound.late = new Promise<void>((resolve) => {
			bound.cancel = deps.scheduler.schedule(LATER_VIEW_MS, () => {
				bound.expired = true;
				resolve();
			});
		});
		return bound;
	}

	/** The read's answer, or `OUT` once the bound ended, even when the read ended with it. */
	private async bounded<T>(read: Promise<T>, bound: Bound | undefined): Promise<T | typeof OUT> {
		if (bound === undefined) return read;
		read.catch(() => undefined);
		const value = await Promise.race([read, bound.late]);
		return bound.expired ? OUT : (value as T);
	}

	private async viewOf(route: Route, row: ModelRow, use: ModelUse, zdrOnly: boolean): Promise<ModelView | undefined> {
		const views = await route.catalog.views(use, zdrOnly);
		return views.find((each) => each.row.key === row.key);
	}
}
