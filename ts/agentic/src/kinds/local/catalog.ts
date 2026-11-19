// The local catalog: one entry's discovered models, their views, browser and tester.

import type { ModelRegistry } from "@nyaaai/contract/modelRegistry";
import type { BrowseModel, ModelRow, ModelUse, ModelView, RowIdentity } from "@nyaaai/contract/shapes/model";
import type { ApiFacts, Effort } from "@nyaaai/contract/shapes/step";
import { headOf, trimText } from "@nyaaai/contract/text";
import { deadline, type Scheduler } from "@nyaaai/contract/time";
import { EFFORTS } from "../../effort";
import { rowKey, shownId } from "../../models/rowKey";
import { CHAT } from "../../models/uses";
import { raced } from "../../models/view";
import type { Browser, EntryCatalog, Tester } from "../../provider/kind";
import type { Transport } from "../../transport";
import { told } from "../join";
import { fieldOf, fieldsOf } from "../readers";
import type { Terms } from "./kind";

////////////////////////////////
//  Interfaces & Types

export type LocalTransport = Pick<Transport, "getJson" | "postJson">;

/** One served model. */
export interface LocalModel {
	tag: string;
	tools: boolean;
	thinking: boolean;
	efforts: Effort[];
	/** Configured context. */
	context?: number;
	/** Current load's context. */
	loaded?: number;
	/** Trained length. */
	trained?: number;
	/** Facts not read this load. */
	probed?: false;
}

/** One server's discovery. */
export interface LocalServer {
	/** Throws when unreadable; `known` is the last list. */
	models(known: readonly LocalModel[], signal: AbortSignal): Promise<LocalModel[]>;
	/** Loaded context by tag, after a reply. */
	loaded?(tag: string, signal: AbortSignal): Promise<ReadonlyMap<string, number>>;
	/** A reload can change context. */
	live?: true;
	/** Loads an unprobed model. */
	warm?(tag: string, signal: AbortSignal): Promise<void>;
}

/** Server-reported model. */
export interface Discovered {
	key: string;
	/** `shownId` text. */
	display: string;
	tag: string;
	tools: boolean;
	reasoning: boolean;
	context?: number;
}

export interface LocalCatalogDeps {
	/** Discovering entry ID. */
	entry: string;
	server: LocalServer;
	terms: Terms;
	/** The time, and the deadlines of a read. */
	scheduler: Scheduler;
	rows: Pick<ModelRegistry, "rows">;
	refreshed?(): void;
}

////////////////////////////////
//  Constants

const RETRY_MS = 60_000;
const DISCOVER_MS = 10_000;

/** Thinking-on efforts. */
const ON: readonly Effort[] = ["low", "medium", "high"];

const LABEL_UNITS = 200;

////////////////////////////////
//  Functions & Helpers

/** The list at `key`; none unless a list. */
export function itemsOf(raw: unknown, key: string): readonly unknown[] | undefined {
	const fields = fieldsOf(raw);
	const list = fields === undefined ? undefined : fieldOf(fields, key);
	return Array.isArray(list) ? list : undefined;
}

/** `false`: none; `true`: all. */
export function effortsOf(values: readonly (boolean | string)[]): Effort[] {
	return EFFORTS.filter(
		(effort) =>
			values.includes(effort) ||
			(effort === "none" && values.includes(false)) ||
			(values.includes(true) && ON.includes(effort)),
	);
}

function apiOf(model: LocalModel): ApiFacts {
	return {
		family: "chat-completions",
		structured: true,
		choice: [],
		strict: false,
		...(model.thinking ? { reasoning: "reasoning" } : {}),
	};
}

/** A length that counts: above zero. */
const positive = (length: number | undefined): number | undefined =>
	length !== undefined && Number.isFinite(length) && length > 0 ? length : undefined;

/** Trained length caps context; a length of zero or less is missing. */
export function contextOf(model: LocalModel, loaded: number | undefined): number | undefined {
	const known = positive(model.context) ?? positive(loaded);
	const trained = positive(model.trained);
	return known === undefined ? undefined : Math.min(known, trained ?? known);
}

/** A model's view for a row under the entry's terms. */
export function localView(
	row: ModelRow,
	model: LocalModel,
	loaded: number | undefined,
	{ terms, zdrOnly }: { terms: Terms; zdrOnly: boolean },
): ModelView {
	const { price } = terms;
	if (!model.tools) return { row, status: "noTools", efforts: model.efforts, price, hosts: [] };
	if (zdrOnly && terms.kept) return { row, status: "noZdrHost", efforts: model.efforts, price, hosts: [] };
	const context = contextOf(model, loaded);
	return {
		row,
		status: "ok",
		efforts: model.efforts,
		...(context === undefined ? {} : { promptLimit: context }),
		price,
		hosts: [],
		api: apiOf(model),
		cache: "implicit",
		...(terms.kept ? { dataKept: true as const } : {}),
	};
}

/** The last non-blank part of a path-like tag; record labels fit 200. */
export function labelOf(tag: string): string {
	let last: string | undefined;
	let from = 0;
	for (let at = 0; at <= tag.length; at++) {
		if (at < tag.length && tag[at] !== "/" && tag[at] !== "\\") continue;
		const part = tag.slice(from, at);
		if (trimText(part).length > 0) last = part;
		from = at + 1;
	}
	return headOf(last ?? tag, LABEL_UNITS);
}

/** Retestable; the server behind a tag can change. */
function browseOf(model: Discovered, terms: Terms): BrowseModel {
	return {
		id: model.key,
		display: model.display,
		label: model.tag,
		curated: false,
		description: "",
		...(model.context === undefined ? {} : { context: model.context }),
		inputs: [],
		outputs: [],
		parameters: [...(model.tools ? ["tools"] : []), ...(model.reasoning ? ["reasoning"] : [])],
		price: terms.price,
		tools: model.tools ? "ok" : "none",
		retestable: true,
		family: "chat-completions",
	};
}

export function localBrowser(catalog: Pick<LocalCatalog, "discovered" | "terms">): Browser {
	return {
		list: async (fresh) => (await catalog.discovered(fresh))?.map((model) => browseOf(model, catalog.terms)),
		detail: async () => ({}),
	};
}

export function localTester(catalog: Pick<LocalCatalog, "discovered" | "viewFor" | "warmed">): Tester {
	return {
		listed: async (id, signal) => {
			await catalog.warmed(id, signal);
			const model = (await catalog.discovered(false, signal))?.find((each) => each.tag === id);
			return model === undefined ? undefined : { label: labelOf(model.tag), reasoning: model.reasoning };
		},
		view: (row, use, signal) => catalog.viewFor(row, use, signal),
	};
}

////////////////////////////////
//  Classes

/** One entry's discovered models; keeps the last list on errors. */
export class LocalCatalog implements EntryCatalog {
	private good: readonly LocalModel[] | undefined;
	private failedAt: number | undefined;
	private inflight: Promise<readonly LocalModel[] | undefined> | undefined;
	/** Loaded context by tag. */
	private readonly loaded = new Map<string, number>();
	/** Last reread by tag. */
	private readonly reread = new Map<string, number>();
	private reading = false;
	/** Every discovery and loaded read that is running. */
	private readonly work = new Set<Promise<unknown>>();

	constructor(private readonly d: LocalCatalogDeps) {}

	get terms(): Terms {
		return this.d.terms;
	}

	/** Tested local model views. */
	async views(use: ModelUse = CHAT, zdrOnly = true, signal?: AbortSignal): Promise<ModelView[]> {
		const found = await raced(this.fetched(), signal);
		if (found === undefined) return [];
		const tested = new Map(
			this.d.rows.rows().flatMap((row) => {
				const tag = this.tagOf(row);
				return tag === undefined ? [] : [[tag, row] as const];
			}),
		);
		return found.flatMap((model) => {
			const row = tested.get(model.tag);
			return row === undefined
				? []
				: [localView(row, model, this.loaded.get(model.tag), { terms: this.d.terms, zdrOnly })];
		});
	}

	/** Refreshes server discovery. */
	async discovered(fresh = false, signal?: AbortSignal): Promise<Discovered[] | undefined> {
		const found = await raced(fresh ? this.discovering() : this.fetched(), signal);
		return found?.map((model) => {
			const context = contextOf(model, this.loaded.get(model.tag));
			const identity: RowIdentity = { kind: "local", entry: this.d.entry, tag: model.tag };
			return {
				key: rowKey(identity),
				display: shownId(identity),
				tag: model.tag,
				tools: model.tools,
				reasoning: model.thinking,
				...(context === undefined ? {} : { context }),
			};
		});
	}

	/** Tester view; its prompts are synthetic. */
	async viewFor(row: ModelRow, use: ModelUse = CHAT, signal?: AbortSignal): Promise<ModelView> {
		const found = await raced(this.fetched(), signal);
		const tag = this.tagOf(row);
		const model = found?.find((each) => each.tag === tag);
		if (model === undefined) throw new Error(`${row.label} is not on the server.`);
		// Synthetic prompts; no retention gate.
		const terms = { ...this.d.terms, kept: false };
		return localView(row, model, this.loaded.get(model.tag), { terms, zdrOnly: false });
	}

	/** This entry's local rows only. */
	private tagOf(row: ModelRow): string | undefined {
		const { identity } = row;
		return identity.kind === "local" && identity.entry === this.d.entry ? identity.tag : undefined;
	}

	rediscover(): void {
		void this.discovering();
	}

	/** Tests read real capabilities. */
	async warmed(tag: string, signal: AbortSignal): Promise<void> {
		const { server } = this.d;
		const model = (await raced(this.fetched(), signal))?.find((each) => each.tag === tag);
		if (server.warm === undefined || model?.probed !== false) return;
		await server.warm(tag, signal);
		await raced(this.discovering(), signal);
	}

	/** Load context after replies. */
	settled(tag: string): void {
		const { server } = this.d;
		const model = this.good?.find((each) => each.tag === tag);
		if (model?.probed === false) {
			const now = this.d.scheduler.nowMs();
			const at = this.reread.get(tag);
			if (at !== undefined && now - at < RETRY_MS) return;
			this.reread.set(tag, now);
			this.rediscover();
			return;
		}
		if (server.loaded === undefined || model === undefined) return;
		if (model.context !== undefined || this.reading || (this.loaded.has(tag) && server.live !== true)) return;
		this.reading = true;
		const limit = deadline(this.d.scheduler, DISCOVER_MS);
		const reading = server
			.loaded(tag, limit.signal)
			.then((contexts) => {
				let changed = false;
				for (const [name, context] of contexts) {
					if (this.loaded.get(name) === context) continue;
					this.loaded.set(name, context);
					changed = true;
				}
				if (changed) told(this.d.refreshed);
			})
			.catch(() => {})
			.finally(() => {
				limit.clear();
				this.reading = false;
				this.work.delete(reading);
			});
		this.work.add(reading);
	}

	/** Settles when no discovery or read is running. */
	async idle(): Promise<void> {
		while (this.work.size > 0) await Promise.allSettled([...this.work]);
	}

	private fetched(): Promise<readonly LocalModel[] | undefined> {
		if (this.good !== undefined) return Promise.resolve(this.good);
		if (this.failedAt !== undefined && this.d.scheduler.nowMs() - this.failedAt < RETRY_MS)
			return Promise.resolve(undefined);
		return this.discovering();
	}

	private discovering(): Promise<readonly LocalModel[] | undefined> {
		if (this.inflight === undefined) {
			const running = this.discover().finally(() => {
				this.inflight = undefined;
				this.work.delete(running);
			});
			this.inflight = running;
			this.work.add(running);
		}
		return this.inflight;
	}

	private async discover(): Promise<readonly LocalModel[] | undefined> {
		const limit = deadline(this.d.scheduler, DISCOVER_MS);
		try {
			const models = await this.d.server.models(this.good ?? [], limit.signal);
			this.good = models;
			for (const tag of this.loaded.keys())
				if (!models.some((model) => model.tag === tag)) this.loaded.delete(tag);
			for (const model of models) {
				if (model.loaded !== undefined) this.loaded.set(model.tag, model.loaded);
				else if (this.d.server.live === true) this.loaded.delete(model.tag);
			}
			this.failedAt = undefined;
		} catch {
			this.failedAt = this.d.scheduler.nowMs();
			return this.good;
		} finally {
			limit.clear();
		}
		told(this.d.refreshed);
		return this.good;
	}
}
