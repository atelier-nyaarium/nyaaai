// The OpenCode plan catalog: decoder, join, browser and tester.

import type { ModelRegistry } from "@nyaaai/contract/modelRegistry";
import type {
	Binding,
	BrowseDetail,
	BrowseModel,
	ModelRow,
	ModelUse,
	ModelView,
	RowIdentity,
} from "@nyaaai/contract/shapes/model";
import type { ProviderPlan } from "@nyaaai/contract/shapes/provider";
import type { ApiFacts, ApiFamily } from "@nyaaai/contract/shapes/step";
import { type Clock, deadline, type Scheduler } from "@nyaaai/contract/time";
import { EFFORTS } from "../../effort";
import { MODELS } from "../../models/curated";
import { rowKey, shownId } from "../../models/rowKey";
import { CHAT } from "../../models/uses";
import { raced, unknownView } from "../../models/view";
import type { Browser, EntryCatalog, Tester } from "../../provider/kind";
import type { Transport } from "../../transport";
import { told } from "../join";
import {
	booleanField,
	dataList,
	type Fields,
	fieldOf,
	fieldsOf,
	numberField,
	objectField,
	textField,
	textsField,
} from "../readers";
import { isZdr, retentionOf } from "./retention";

////////////////////////////////
//  Interfaces & Types

/** One API a model is served on. A missing, null or mistyped field is null. */
export interface OpenCodeApi {
	family: ApiFamily;
	/** The thought text field. */
	reasoning_field: string | null;
	/** Tool choice modes. */
	choice: string[] | null;
	strict: boolean | null;
}

/** `explicit` is whether the listing holds an object or `true` for it. */
export interface OpenCodeCache {
	implicit: boolean | null;
	explicit: boolean;
}

/** What a plan's list says of one model; a missing, null or mistyped field is null. */
export interface OpenCodeModel {
	id: string;
	name: string | null;
	description: string | null;
	input: string[] | null;
	output: string[] | null;
	tools: boolean | null;
	reasoning: boolean | null;
	efforts: string[] | null;
	toggle: boolean | null;
	structured_output: boolean | null;
	cache: OpenCodeCache | null;
	context: number | null;
	input_limit: number | null;
	output_limit: number | null;
	/** Base text prices, USD per million tokens. */
	price: { input: number; output: number } | null;
	/** The listed families, in the order of the four. */
	api: OpenCodeApi[];
}

export interface OpenCodeCatalogDeps {
	plan: ProviderPlan;
	transport: Pick<Transport, "getJson">;
	scheduler: Scheduler;
	rows: Pick<ModelRegistry, "rows">;
	refreshed?: () => void;
	ttlMs?: number;
}

////////////////////////////////
//  Constants

const TTL_MS = 12 * 60 * 60 * 1000;
const RETRY_MS = 60_000;
const FETCH_MS = 30_000;

const FAMILIES: readonly ApiFamily[] = ["chat-completions", "responses", "messages", "generate-content"];

/** Unbound model suffixes. */
const UNBOUND = ["-free", "-latest"];

////////////////////////////////
//  Functions & Helpers

const into = (fields: Fields | null, key: string): Fields | null => (fields === null ? null : objectField(fields, key));

function apiOf(value: unknown): OpenCodeApi[] {
	const record = fieldsOf(value);
	if (record === undefined) return [];
	return FAMILIES.flatMap((family): OpenCodeApi[] => {
		if (!Object.hasOwn(record, family)) return [];
		const held = fieldsOf(record[family]) ?? null;
		const tools = into(into(held, "capabilities"), "tools");
		const compatibility = into(held, "compatibility");
		return [
			{
				family,
				reasoning_field: compatibility === null ? null : textField(compatibility, "reasoning_field"),
				choice: tools === null ? null : textsField(tools, "choice"),
				strict: tools === null ? null : booleanField(tools, "strict"),
			},
		];
	});
}

/** Both prices are numbers, or the cost is unknown. */
function priceOf(cost: Fields | null): OpenCodeModel["price"] {
	const text = into(cost, "text");
	const input = into(text, "input");
	const output = into(text, "output");
	const prompt = input === null ? null : numberField(input, "price");
	const completion = output === null ? null : numberField(output, "price");
	return prompt === null || completion === null ? null : { input: prompt, output: completion };
}

function cacheField(capabilities: Fields | null): OpenCodeCache | null {
	const cache = into(capabilities, "cache");
	if (cache === null) return null;
	const explicit = fieldOf(cache, "explicit");
	return {
		implicit: booleanField(cache, "implicit"),
		explicit: explicit === true || fieldsOf(explicit) !== undefined,
	};
}

/** An item with a text id; undefined drops the item. */
export function parseOpenCodeModel(item: unknown): OpenCodeModel | undefined {
	const fields = fieldsOf(item);
	const id = fields === undefined ? undefined : fieldOf(fields, "id");
	if (fields === undefined || typeof id !== "string") return undefined;
	const modalities = objectField(fields, "modalities");
	const capabilities = objectField(fields, "capabilities");
	const tools = into(capabilities, "tools");
	const reasoning = into(capabilities, "reasoning");
	const limit = objectField(fields, "limit");
	return {
		id,
		name: textField(fields, "name"),
		description: textField(fields, "description"),
		input: modalities === null ? null : textsField(modalities, "input"),
		output: modalities === null ? null : textsField(modalities, "output"),
		tools: tools === null ? null : booleanField(tools, "supported"),
		reasoning: reasoning === null ? null : booleanField(reasoning, "supported"),
		efforts: reasoning === null ? null : textsField(reasoning, "effort"),
		toggle: reasoning === null ? null : booleanField(reasoning, "toggle"),
		structured_output: capabilities === null ? null : booleanField(capabilities, "structured_output"),
		cache: cacheField(capabilities),
		context: limit === null ? null : numberField(limit, "context"),
		input_limit: limit === null ? null : numberField(limit, "input"),
		output_limit: limit === null ? null : numberField(limit, "output"),
		price: priceOf(objectField(fields, "cost")),
		api: apiOf(fieldOf(fields, "api")),
	};
}

/** Undefined when the body holds no list; a later model of an id replaces the earlier in its place. */
export function parseOpenCode(raw: unknown): Map<string, OpenCodeModel> | undefined {
	const list = dataList(raw);
	if (list === undefined) return undefined;
	const models = new Map<string, OpenCodeModel>();
	for (const item of list) {
		const model = parseOpenCodeModel(item);
		if (model !== undefined) models.set(model.id, model);
	}
	return models;
}

function familyOf(model: OpenCodeModel): ApiFamily | undefined {
	return FAMILIES.find((family) => model.api.some((api) => api.family === family));
}

/** API-specific caching defaults. */
function cacheOf(model: OpenCodeModel, family: ApiFamily): ModelRow["cache"] {
	const cache = model.cache;
	if (cache === null) return family === "messages" ? "explicit" : "implicit";
	return cache.implicit !== true && cache.explicit ? "explicit" : "implicit";
}

function factsOf(model: OpenCodeModel, family: ApiFamily): ApiFacts {
	const api = model.api.find((each) => each.family === family);
	const reasoning = api?.reasoning_field ?? undefined;
	return {
		family,
		structured: model.structured_output === true,
		// Missing choice means `auto`.
		choice: api?.choice ?? ["auto"],
		strict: api?.strict === true,
		...(reasoning === undefined ? {} : { reasoning }),
		...(model.toggle === true ? { toggle: true as const } : {}),
	};
}

/** Base prices only. */
function viewOf(
	row: ModelRow,
	model: OpenCodeModel,
	plan: ProviderPlan,
	zdrOnly: boolean,
	now: number,
): ModelView | undefined {
	const family = familyOf(model);
	if (family === undefined) return undefined;
	if (model.tools !== true) return undefined;
	const offered = model.efforts ?? [];
	const efforts = row.reasoning ? EFFORTS.filter((effort) => offered.includes(effort)) : [];
	const zdr = isZdr(plan, model.id, now);
	if (zdrOnly && !zdr) return { row, status: "noZdrHost", efforts, hosts: [] };
	const limit = model.input_limit ?? model.context;
	const output = model.output_limit;
	const price = model.price;
	return {
		row,
		status: "ok",
		efforts,
		...(limit === null ? {} : { promptLimit: limit }),
		...(output === null ? {} : { outputLimit: output }),
		...(price === null
			? {}
			: {
					price: {
						prompt: [price.input, price.input],
						completion: [price.output, price.output],
						tiers: [{ minPromptTokens: 0, prompt: price.input, completion: price.output }],
					},
				}),
		hosts: [],
		...(zdr ? {} : { dataKept: true as const }),
		api: factsOf(model, family),
		cache: cacheOf(model, family),
	};
}

function bound(row: ModelRow): row is ModelRow & { bindings: { opencode: Binding } } {
	return row.bindings["opencode"] !== undefined;
}

/** Retention survives catalog failure. */
function unlisted(
	row: ModelRow & { bindings: { opencode: Binding } },
	plan: ProviderPlan,
	zdrOnly: boolean,
	now: number,
): ModelView {
	if (isZdr(plan, row.bindings.opencode.id, now)) return unknownView(row);
	return zdrOnly ? { row, status: "noZdrHost", efforts: [], hosts: [] } : { ...unknownView(row), dataKept: true };
}

function mediaOf(types: readonly string[] | null): string[] {
	return [...new Set((types ?? []).map((type) => type.split("/")[0] ?? type))];
}

function capabilitiesOf(model: OpenCodeModel): string[] {
	return [
		...(model.tools === true ? ["tools"] : []),
		...(model.reasoning === true ? ["reasoning"] : []),
		...(model.structured_output === true ? ["structured_output"] : []),
	];
}

function browseOf(model: OpenCodeModel, plan: ProviderPlan, now: number): BrowseModel {
	const family = familyOf(model);
	const zdr = isZdr(plan, model.id, now);
	const price = model.price;
	const context = model.context ?? model.input_limit;
	const identity: RowIdentity = { kind: "hosted", provider: "opencode", id: model.id };
	return {
		id: rowKey(identity),
		display: shownId(identity),
		label: model.name ?? model.id,
		curated: MODELS.some((row) => row.bindings["opencode"]?.id === model.id),
		description: model.description ?? "",
		...(context === null ? {} : { context }),
		inputs: mediaOf(model.input),
		outputs: mediaOf(model.output),
		parameters: capabilitiesOf(model),
		price:
			price === null
				? "unknown"
				: { prompt: [price.input, price.input], completion: [price.output, price.output] },
		tools: !zdr ? "noZdrHost" : family !== undefined && model.tools === true ? "ok" : "none",
		...(family === undefined ? {} : { family }),
	};
}

function detailOf(model: OpenCodeModel, plan: ProviderPlan, now: number): BrowseDetail {
	const family = familyOf(model);
	const until = retentionOf(plan, model.id)?.until;
	return {
		...(model.output_limit === null ? {} : { maxOutput: model.output_limit }),
		efforts: model.efforts ?? [],
		...(family === undefined ? {} : { choice: [...factsOf(model, family).choice], cache: cacheOf(model, family) }),
		retention: { zdr: isZdr(plan, model.id, now), ...(until === undefined ? {} : { until }) },
	};
}

export function opencodeBrowser(catalog: Pick<OpenCodeCatalog, "models">, plan: ProviderPlan, clock: Clock): Browser {
	return {
		list: async (fresh) => {
			const models = await catalog.models(fresh);
			if (models === undefined) return undefined;
			const now = clock.nowMs();
			return [...models.values()].flatMap((model) =>
				UNBOUND.some((suffix) => model.id.endsWith(suffix)) ? [] : [browseOf(model, plan, now)],
			);
		},
		detail: async (id) => {
			const model = (await catalog.models())?.get(id);
			return model === undefined ? undefined : detailOf(model, plan, clock.nowMs());
		},
	};
}

export function opencodeTester(catalog: Pick<OpenCodeCatalog, "models" | "viewFor">): Tester {
	return {
		listed: async (id, signal) => {
			const model = (await catalog.models(false, signal))?.get(id);
			if (model === undefined) return undefined;
			return {
				label: model.name ?? model.id,
				reasoning: (model.efforts ?? []).length > 0,
			};
		},
		view: (row, use, signal) => catalog.viewFor(row, use, signal),
	};
}

/** No per-model fetches. */
export function joinOpenCode(
	rows: readonly ModelRow[],
	models: ReadonlyMap<string, OpenCodeModel> | undefined,
	plan: ProviderPlan,
	use: ModelUse,
	zdrOnly: boolean,
	now: number,
): ModelView[] {
	if (models === undefined) return rows.filter(bound).map((row) => unlisted(row, plan, zdrOnly, now));
	return rows.filter(bound).flatMap((row) => {
		const model = models.get(row.bindings.opencode.id);
		const view = model === undefined ? undefined : viewOf(row, model, plan, zdrOnly, now);
		return view === undefined ? [] : [view];
	});
}

////////////////////////////////
//  Classes

/** One entry's plan catalog; keeps the last list on errors. */
export class OpenCodeCatalog implements EntryCatalog {
	private good: { models: Map<string, OpenCodeModel>; at: number } | undefined;
	private failedAt: number | undefined;
	private inflight: Promise<Map<string, OpenCodeModel> | undefined> | undefined;

	constructor(private readonly d: OpenCodeCatalogDeps) {}

	async views(use: ModelUse = CHAT, zdrOnly = true, signal?: AbortSignal): Promise<ModelView[]> {
		const models = await raced(this.fetched(), signal);
		return joinOpenCode(this.d.rows.rows(), models, this.d.plan, use, zdrOnly, this.d.scheduler.nowMs());
	}

	/** Fresh retries failures. */
	async models(fresh = false, signal?: AbortSignal): Promise<ReadonlyMap<string, OpenCodeModel> | undefined> {
		if (fresh) this.failedAt = undefined;
		return raced(this.fetched(), signal);
	}

	/** Tester ZDR view. */
	async viewFor(row: ModelRow, use: ModelUse = CHAT, signal?: AbortSignal): Promise<ModelView> {
		const models = await raced(this.fetched(), signal);
		const id = row.bindings["opencode"]?.id;
		if (models === undefined || id === undefined) return unknownView(row);
		const model = models.get(id);
		if (model === undefined) return { row, status: "retired", efforts: [], hosts: [] };
		return (
			viewOf(row, model, this.d.plan, true, this.d.scheduler.nowMs()) ?? {
				row,
				status: "noZdrHost",
				efforts: [],
				hosts: [],
			}
		);
	}

	private fetched(): Promise<Map<string, OpenCodeModel> | undefined> {
		const now = this.d.scheduler.nowMs();
		if (this.good !== undefined && now - this.good.at < (this.d.ttlMs ?? TTL_MS))
			return Promise.resolve(this.good.models);
		if (this.failedAt !== undefined && now - this.failedAt < RETRY_MS) return Promise.resolve(this.good?.models);
		this.inflight ??= this.refresh().finally(() => {
			this.inflight = undefined;
		});
		return this.inflight;
	}

	private async refresh(): Promise<Map<string, OpenCodeModel> | undefined> {
		const limit = deadline(this.d.scheduler, FETCH_MS);
		try {
			const models = parseOpenCode(await this.d.transport.getJson("/v1/models", limit.signal, "none"));
			if (models === undefined) throw new Error("The catalog did not parse.");
			this.good = { models, at: this.d.scheduler.nowMs() };
			this.failedAt = undefined;
		} catch {
			this.failedAt = this.d.scheduler.nowMs();
			return this.good?.models;
		} finally {
			limit.clear();
		}
		told(this.d.refreshed);
		return this.good?.models;
	}

	/** Answers when no refresh is running, having waited for the one that was. */
	async idle(): Promise<void> {
		while (this.inflight !== undefined) await this.inflight;
	}
}
