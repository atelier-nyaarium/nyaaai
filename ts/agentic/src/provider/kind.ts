// What a provider kind gives a host: catalogs, a browser and a tester.

import type { Fetch } from "@nyaaai/contract/fetch";
import type { ModelRegistry } from "@nyaaai/contract/modelRegistry";
import type { BrowseDetail, BrowseModel, ModelRow, ModelUse, ModelView } from "@nyaaai/contract/shapes/model";
import type {
	Billed,
	Disposition,
	EntryFields,
	KeyState,
	KindLine,
	Limits,
	ProviderEntry,
	ProviderKindName,
	ProviderPlan,
	ProviderView,
} from "@nyaaai/contract/shapes/provider";
import type { Failure } from "@nyaaai/contract/shapes/step";
import type { Scheduler } from "@nyaaai/contract/time";
import type { Codec } from "../codec";
import { COMPATIBLE } from "../kinds/compatible";
import { LLAMACPP } from "../kinds/llamacpp";
import { LMSTUDIO } from "../kinds/lmstudio";
import { OLLAMA } from "../kinds/ollama";
import { OPENCODE } from "../kinds/opencode/kind";
import { OPENROUTER } from "../kinds/openrouter/kind";
import type { KeySource, Transport } from "../transport";

////////////////////////////////
//  Interfaces & Types

/** What a kind stands on. */
export interface KindDeps {
	fetch: Fetch;
	/** Every date and deadline of a route, its codec and browser included. */
	scheduler: Scheduler;
	log(line: string): void;
	/** The consumer's `name/version`. */
	userAgent: string;
	/** Test loopback URL. */
	baseUrl?: string;
}

/** What a catalog stands on. */
export interface CatalogDeps {
	/** The time, and the deadlines of a fetch. */
	scheduler: Scheduler;
	/** Rows; a catalog that caches a join rejoins when this tells a change. */
	rows: Pick<ModelRegistry, "rows" | "onChange">;
	/** Told after each successful refresh. */
	refreshed(): void;
}

/** Entry-specific bound model data. */
export interface EntryCatalog {
	/** ZDR-scoped bound rows. */
	views(use: ModelUse, zdrOnly: boolean, signal?: AbortSignal): Promise<ModelView[]>;
	rediscover?(): void;
	/** Answers when no refresh or read of this catalog is running, having waited for the one that was. */
	idle(): Promise<void>;
	/** Stops its listening. */
	dispose?(): void;
}

/** An entry's model browser. */
export interface Browser {
	/** Fresh retries failed reads. */
	list(fresh?: boolean): Promise<BrowseModel[] | undefined>;
	/** Optional detail by the kind's own model id. */
	detail(id: string, signal: AbortSignal): Promise<BrowseDetail | undefined>;
}

/** One entry's model facts for the tester. */
export interface Tester {
	/** `id` is the kind's own model id. */
	listed(id: string, signal: AbortSignal): Promise<{ label: string; reasoning: boolean } | undefined>;
	/** ZDR view per use. */
	view(row: ModelRow, use: ModelUse, signal: AbortSignal): Promise<ModelView>;
	/** Fresh per-test view. */
	pins?(): Pins;
}

/** Test host routing. */
export interface Pins {
	/** Two-host ZDR alternation; `id` is the kind's own. */
	hop(id: string, view: ModelView, signal: AbortSignal): Promise<string[]>;
	/** Pin and record streams. */
	wrap(codec: Codec): Codec;
	/** Alternating hosts; empty unpins. */
	pin(hosts: readonly string[]): void;
	/** Detect resent reasoning. */
	passedBack(): boolean;
}

/** An entry's transport and catalog. */
export interface Route {
	transport: Transport;
	catalog: EntryCatalog;
}

/** Provider engine. */
export interface ProviderKind {
	readonly kind: ProviderKindName;
	readonly label: string;
	/** Plan dropdown lines. */
	readonly plans?: readonly { plan: ProviderPlan; label: string }[];
	/** Add-form third field. */
	readonly field: "key" | "url";
	/** URL kinds that also take a key. */
	readonly optionalKey?: true;
	/** Default URL. */
	readonly url?: string;
	/** The balance page. */
	readonly balance?: string;
	/** Plans with extra usage. */
	readonly extraUsage?: readonly ProviderPlan[];
	/** Delayed generation stats. */
	readonly stats?: true;
	/** Provider status facts. */
	facts?(state: KeyState, entry: EntryFields, bill: Billed | undefined): string | undefined;
	/** Key or URL check. */
	check(entry: EntryFields, secret: string | undefined, deps: KindDeps, signal: AbortSignal): Promise<KeyState>;
	transport(entry: EntryFields, key: KeySource, deps: KindDeps): Transport;
	catalog(entry: ProviderEntry, transport: Transport, deps: CatalogDeps): EntryCatalog;
	/** Omitted if unavailable. */
	browse?(entry: ProviderEntry, route: Route, deps: KindDeps): Browser;
	tester?(entry: ProviderEntry, route: Route, deps: KindDeps): Tester;
	/** Model request codec. */
	codec(view: ModelView | undefined, entry: EntryFields, deps: KindDeps): Codec;
	/** Retry, fall-through, or stop. */
	failure(failure: Failure, view: ModelView | undefined): Disposition;
	/** Optional provider limits. */
	limits?(transport: Pick<Transport, "getJson">, signal: AbortSignal): Promise<Limits | undefined>;
}

////////////////////////////////
//  Constants

/** Dropdown order. */
export const KINDS: Partial<Record<ProviderKindName, ProviderKind>> = {
	opencode: OPENCODE,
	openrouter: OPENROUTER,
	ollama: OLLAMA,
	lmstudio: LMSTUDIO,
	llamacpp: LLAMACPP,
	compatible: COMPATIBLE,
};

////////////////////////////////
//  Functions & Helpers

/** One line per plan. */
export function kindLines(kinds: Partial<Record<ProviderKindName, ProviderKind>>): KindLine[] {
	return Object.values(kinds).flatMap((kind): KindLine[] => {
		const line = {
			kind: kind.kind,
			field: kind.field,
			...(kind.url === undefined ? {} : { url: kind.url }),
			...(kind.optionalKey === undefined ? {} : { optionalKey: kind.optionalKey }),
		};
		if (kind.plans === undefined || kind.plans.length === 0) return [{ ...line, label: kind.label }];
		return kind.plans.map(({ plan, label }) => ({ ...line, plan, label }));
	});
}

/** An entry's line of the provider list. */
export function providerView(
	entry: ProviderEntry,
	status: KeyState,
	kinds: Partial<Record<ProviderKindName, ProviderKind>>,
	bill?: Billed,
): ProviderView {
	const kind = kinds[entry.kind];
	const facts = kind?.facts?.(status, entry, bill);
	return {
		id: entry.id,
		kind: entry.kind,
		...(entry.plan === undefined ? {} : { plan: entry.plan }),
		label: entry.label,
		...(entry.url === undefined ? {} : { url: entry.url }),
		status,
		...(facts === undefined ? {} : { facts }),
		...(kind?.balance === undefined ? {} : { balance: true as const }),
		...(entry.plan !== undefined && kind?.extraUsage?.includes(entry.plan) === true
			? { extraUsage: entry.extraUsage === true }
			: {}),
	};
}
