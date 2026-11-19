// Routes a model key across the provider list.

import type { EntryKeys, KeyAnswer } from "@nyaaai/contract/entryKeys";
import type { ModelRegistry } from "@nyaaai/contract/modelRegistry";
import type { ProviderList } from "@nyaaai/contract/providerList";
import type { Binding, ModelRow, ModelUse, ModelView, Refusal, Unresolved, Via } from "@nyaaai/contract/shapes/model";
import type { KeyReach, Passed, ProviderEntry, ProviderKindName, SkipReason } from "@nyaaai/contract/shapes/provider";
import type { Codec } from "../codec";
import { parseRowKey, servedIdOf } from "../models/rowKey";
import { unknownView } from "../models/view";
import type { KeySource, Transport } from "../transport";
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
	entry: ProviderEntry;
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
	/** List order is priority order. */
	list: ProviderList;
	keys: EntryKeys;
	registry: Pick<ModelRegistry, "rows" | "onChange">;
	kinds: Partial<Record<ProviderKindName, ProviderKind>>;
	/** What a kind's transport stands on. */
	kindDeps(kind: ProviderKindName): KindDeps;
	zdrOnly(): boolean;
	/** Told after a catalog's refresh. */
	refreshed?(entry: ProviderEntry): void;
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

////////////////////////////////
//  Functions & Helpers

export function viaOf(entry: ProviderEntry): Via {
	return { entry: entry.id, label: entry.label };
}

/** What identifies a route: id, plan and URL. */
export function routeKey(entry: ProviderEntry): string {
	return [entry.id, entry.plan ?? "", entry.url ?? ""].join("\n");
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

/** One route's key: a key stays with its URL. */
export function keySourceOf(keys: EntryKeys, list: ProviderList, entry: ProviderEntry): KeySource {
	return {
		get: async () => {
			if (list.entries().find((each) => each.id === entry.id)?.url !== entry.url) return undefined;
			const answer = await keys.get(entry.id);
			return answer.kind === "key" ? answer.key : undefined;
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

/** Stops a catalog that listens, when it does. */
function disposed(catalog: Route["catalog"]): void {
	const held = catalog as { dispose?: unknown };
	if (typeof held.dispose === "function") held.dispose();
}

/** Resolves through one entry. */
export function servedBy(d: {
	entry: ProviderEntry;
	kind: ProviderKind;
	transport: Resolved["transport"];
	catalog: Catalog;
	zdrOnly: () => boolean;
}): Plan {
	const { zdrOnly } = d;
	return async (key, use) => {
		const view = (await d.catalog.view(key, use)) ?? unknownView(adhocRow(key, d.kind.kind));
		const codec = d.kind.codec(view, d.entry);
		const route: Resolved = {
			ok: true,
			key,
			entry: d.entry,
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
	private readonly made = new Map<string, Route>();
	private readonly unsubscribe: () => void;

	constructor(private readonly d: ResolverDeps) {
		this.unsubscribe = d.list.onChange(() => this.drop());
	}

	/** Stops listening; the routes go. */
	dispose(): void {
		this.unsubscribe();
		for (const route of this.made.values()) disposed(route.catalog);
		this.made.clear();
	}

	/** The entry's route, made once per id, plan and URL. */
	route(entry: ProviderEntry): Route {
		const at = routeKey(entry);
		const known = this.made.get(at);
		if (known !== undefined) return known;
		const kind = this.d.kinds[entry.kind];
		if (kind === undefined) throw new Error(`No ${entry.kind} provider kind.`);
		const deps = this.d.kindDeps(entry.kind);
		const transport = kind.transport(entry, keySourceOf(this.d.keys, this.d.list, entry), deps);
		const catalog = kind.catalog(entry, transport, {
			scheduler: deps.scheduler,
			rows: this.d.registry,
			refreshed: () => this.d.refreshed?.(entry),
		});
		const made = { transport, catalog };
		this.made.set(at, made);
		return made;
	}

	/** What a key says of one entry. */
	reach(entry: ProviderEntry): Promise<KeyReach> {
		return this.d.keys.get(entry.id).then(
			(answer) => reachOf({ field: this.d.kinds[entry.kind]?.field ?? "key" }, answer),
			(): KeyReach => "keyMissing",
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
		return Promise.all(
			this.d.registry.rows().map(async (row): Promise<ResolvedOption> => {
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

	/** Routes of entries the list no longer holds go. */
	private drop(): void {
		const listed = new Set(this.d.list.entries().map(routeKey));
		for (const [at, route] of this.made) {
			if (listed.has(at)) continue;
			this.made.delete(at);
			disposed(route.catalog);
		}
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
		for (const entry of this.d.list.entries()) {
			if (found.length > 0 && !all) break;
			const kind = this.d.kinds[entry.kind];
			const binding = row.bindings[entry.kind];
			const elsewhere = row.identity.kind === "local" && row.identity.entry !== entry.id;
			if (kind === undefined || binding === undefined || elsewhere) continue;
			matched = true;
			const reached = await reach(entry);
			if (reached !== "ok") {
				passed.push({
					entry: entry.id,
					label: entry.label,
					reason: REACH_SKIPS[reached],
					before: found.length,
				});
				const behind =
					standing && worst.view === undefined ? await this.viewOf(entry, row, use, zdrOnly) : undefined;
				worst = raised(worst, reached, behind, entry.label);
				continue;
			}
			const view = await this.viewOf(entry, row, use, zdrOnly);
			if (view === undefined || !usable(view)) {
				worst = raised(worst, refusalOf(view), view, entry.label);
				continue;
			}
			const live = () => this.d.zdrOnly();
			found.push({
				ok: true,
				key: row.key,
				entry,
				kind,
				binding,
				view,
				transport: stamped(this.route(entry).transport, live),
				codec: kind.codec(view, entry),
				get zdrOnly() {
					return live();
				},
			});
		}
		const [first, ...rest] = found;
		if (first !== undefined) return { ok: true, candidates: [first, ...rest], passed };
		return matched ? { ok: false, ...worst } : { ok: false, refusal: "keyMissing" };
	}

	private async viewOf(
		entry: ProviderEntry,
		row: ModelRow,
		use: ModelUse,
		zdrOnly: boolean,
	): Promise<ModelView | undefined> {
		const views = await this.route(entry).catalog.views(use, zdrOnly);
		return views.find((each) => each.row.key === row.key);
	}
}
