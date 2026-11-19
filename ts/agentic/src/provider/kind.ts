// What a provider kind gives a host: catalogs, a browser and a tester.

import type { ModelRegistry } from "@nyaaai/contract/modelRegistry";
import type { BrowseDetail, BrowseModel, ModelRow, ModelUse, ModelView } from "@nyaaai/contract/shapes/model";
import type { Scheduler } from "@nyaaai/contract/time";
import type { Codec } from "../codec";

////////////////////////////////
//  Interfaces & Types

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
