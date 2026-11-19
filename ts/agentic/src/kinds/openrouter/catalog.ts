// The OpenRouter catalog: decoders, the join to rows and the cached fetch.

import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import type { ModelRegistry } from "@nyaaai/contract/modelRegistry";
import { numberOfText, roundHalfUp } from "@nyaaai/contract/number";
import type { Binding, ModelRow, ModelUse, ModelView, PriceTier } from "@nyaaai/contract/shapes/model";
import { deadline, parseIsoTime, type Scheduler } from "@nyaaai/contract/time";
import { EFFORTS } from "../../effort";
import { raced, unknownView } from "../../models/view";
import type { Transport } from "../../transport";
import { joinSettled, told } from "../join";
import { dataList, type Fields, fieldOf, fieldsOf, numberField, objectField, textField, textsField } from "../readers";

////////////////////////////////
//  Interfaces & Types

/** Any listed parameter suffices. */
export type ParamNeed = string | readonly string[];

/** A price override above a prompt size. */
export interface PriceOverride {
	min_prompt_tokens: number;
	prompt: unknown;
	completion: unknown;
	input_cache_read: unknown;
	input_cache_write: unknown;
	input_cache_write_1h: unknown;
}

/** Prices as listed; a price is a number or its text. */
export interface Pricing {
	prompt: unknown;
	completion: unknown;
	input_cache_read: unknown;
	input_cache_write: unknown;
	input_cache_write_1h: unknown;
	overrides: PriceOverride[] | null;
}

/** A missing, null or mistyped field is null. */
export interface ModelEntry {
	id: string;
	name: string | null;
	created: number | null;
	context_length: number | null;
	expiration_date: string | null;
	pricing: Pricing | null;
	reasoning: { supported_efforts: string[] | null } | null;
	supported_parameters: string[] | null;
}

export interface Endpoint {
	model_id: string;
	tag: string | null;
	provider_name: string | null;
	context_length: number | null;
	max_prompt_tokens: number | null;
	supported_parameters: string[] | null;
	pricing: Pricing | null;
}

interface Fetched {
	entries: Map<string, ModelEntry>;
	endpoints: Map<string, Endpoint[]>;
	/** Raw `/models` items for browsing. */
	raw: readonly unknown[];
	at: number;
}

/** One fetch's `/models` items and zero-retention endpoints. */
export interface CatalogListing {
	raw: readonly unknown[];
	endpoints: ReadonlyMap<string, Endpoint[]>;
}

export interface ModelCatalogDeps {
	transport: Pick<Transport, "getJson">;
	scheduler: Scheduler;
	/** Rows; a change rejoins the cached views. */
	rows: Pick<ModelRegistry, "rows" | "onChange">;
	refreshed?: () => void;
	ttlMs?: number;
}

interface Joined {
	/** Row changes told before this join. */
	version: number;
	data: Fetched | undefined;
	zdrOnly: boolean;
	views: ModelView[];
	until: number;
}

////////////////////////////////
//  Constants

const TTL_MS = 12 * 60 * 60 * 1000;
const RETRY_MS = 60_000;
const FETCH_MS = 30_000;
const STRUCTURED_PARAMS = ["response_format", "structured_outputs"];
/** Characters of a date alone. */
const DATE_CHARS = 10;

////////////////////////////////
//  Functions & Helpers

function overrideOf(value: unknown): PriceOverride | undefined {
	const fields = fieldsOf(value);
	if (fields === undefined) return undefined;
	const minimum = numberField(fields, "min_prompt_tokens");
	if (minimum === null) return undefined;
	return {
		min_prompt_tokens: minimum,
		prompt: fieldOf(fields, "prompt"),
		completion: fieldOf(fields, "completion"),
		input_cache_read: fieldOf(fields, "input_cache_read"),
		input_cache_write: fieldOf(fields, "input_cache_write"),
		input_cache_write_1h: fieldOf(fields, "input_cache_write_1h"),
	};
}

/** One bad override drops the list. */
function overridesOf(value: unknown): PriceOverride[] | null {
	if (!Array.isArray(value)) return null;
	const held: PriceOverride[] = [];
	for (const item of value) {
		const override = overrideOf(item);
		if (override === undefined) return null;
		held.push(override);
	}
	return held;
}

function pricingOf(fields: Fields, key: string): Pricing | null {
	const held = objectField(fields, key);
	if (held === null) return null;
	return {
		prompt: fieldOf(held, "prompt"),
		completion: fieldOf(held, "completion"),
		input_cache_read: fieldOf(held, "input_cache_read"),
		input_cache_write: fieldOf(held, "input_cache_write"),
		input_cache_write_1h: fieldOf(held, "input_cache_write_1h"),
		overrides: overridesOf(fieldOf(held, "overrides")),
	};
}

/** USD per million tokens, rounded to six decimals. */
export function perMillion(value: unknown): number | undefined {
	const price = typeof value === "string" ? numberOfText(value) : typeof value === "number" ? value : undefined;
	return price !== undefined && Number.isFinite(price) && price >= 0
		? roundHalfUp(price * 1e12) / 1e6 + 0
		: undefined;
}

/** A removal date: a day, or an ISO time; NaN when neither. */
export function expiryOf(text: string): number {
	return parseIsoTime(text.length === DATE_CHARS ? `${text}T00:00:00Z` : text);
}

export function rangeOf(values: readonly (number | undefined)[]): [number, number] | undefined {
	const known = values.filter((value): value is number => value !== undefined);
	return known.length === 0 ? undefined : [Math.min(...known), Math.max(...known)];
}

function tiersOf(pricing: Pricing | null): PriceTier[] {
	const tiers: PriceTier[] = [];
	const base = { prompt: perMillion(pricing?.prompt), completion: perMillion(pricing?.completion) };
	if (base.prompt === undefined || base.completion === undefined) return tiers;
	tiers.push(
		tier(
			0,
			base.prompt,
			base.completion,
			perMillion(pricing?.input_cache_read),
			perMillion(pricing?.input_cache_write),
			perMillion(pricing?.input_cache_write_1h),
		),
	);
	for (const override of [...(pricing?.overrides ?? [])].sort(
		(left, right) => left.min_prompt_tokens - right.min_prompt_tokens,
	)) {
		tiers.push(
			tier(
				override.min_prompt_tokens,
				perMillion(override.prompt) ?? base.prompt,
				perMillion(override.completion) ?? base.completion,
				perMillion(override.input_cache_read),
				perMillion(override.input_cache_write) ?? perMillion(pricing?.input_cache_write),
				perMillion(override.input_cache_write_1h) ?? perMillion(pricing?.input_cache_write_1h),
			),
		);
	}
	return tiers;
}

/** A tier; a price that is not listed is left out. */
function tier(
	minPromptTokens: number,
	prompt: number,
	completion: number,
	cacheRead: number | undefined,
	cacheWrite: number | undefined,
	cacheWrite1h: number | undefined,
): PriceTier {
	return {
		minPromptTokens,
		prompt,
		completion,
		...(cacheRead === undefined ? {} : { cacheRead }),
		...(cacheWrite === undefined ? {} : { cacheWrite }),
		...(cacheWrite1h === undefined ? {} : { cacheWrite1h }),
	};
}

export function sentParams(row: ModelRow, use: ModelUse): ParamNeed[] {
	const reasoning = row.reasoning ? ["reasoning"] : [];
	if (use === "chat") return ["tools", ...reasoning];
	return ["tools", ...reasoning, ...STRUCTURED_PARAMS];
}

/** An item with a text id; null drops the item. */
export function parseModel(item: unknown): ModelEntry | undefined {
	const fields = fieldsOf(item);
	const id = fields === undefined ? undefined : fieldOf(fields, "id");
	if (fields === undefined || typeof id !== "string") return undefined;
	const reasoning = objectField(fields, "reasoning");
	return {
		id,
		name: textField(fields, "name"),
		created: numberField(fields, "created"),
		context_length: numberField(fields, "context_length"),
		expiration_date: textField(fields, "expiration_date"),
		pricing: pricingOf(fields, "pricing"),
		reasoning: reasoning === null ? null : { supported_efforts: textsField(reasoning, "supported_efforts") },
		supported_parameters: textsField(fields, "supported_parameters"),
	};
}

/** An item with a text model id. */
export function parseEndpoint(item: unknown): Endpoint | undefined {
	const fields = fieldsOf(item);
	const modelId = fields === undefined ? undefined : fieldOf(fields, "model_id");
	if (fields === undefined || typeof modelId !== "string") return undefined;
	return {
		model_id: modelId,
		tag: textField(fields, "tag"),
		provider_name: textField(fields, "provider_name"),
		context_length: numberField(fields, "context_length"),
		max_prompt_tokens: numberField(fields, "max_prompt_tokens"),
		supported_parameters: textsField(fields, "supported_parameters"),
		pricing: pricingOf(fields, "pricing"),
	};
}

/** Undefined when the body holds no list; a later entry of an id replaces the earlier in its place. */
export function parseModels(raw: unknown): Map<string, ModelEntry> | undefined {
	const list = dataList(raw);
	if (list === undefined) return undefined;
	const models = new Map<string, ModelEntry>();
	for (const item of list) {
		const entry = parseModel(item);
		if (entry !== undefined) models.set(entry.id, entry);
	}
	return models;
}

/** Undefined when the body holds no list. */
export function parseEndpoints(raw: unknown): Map<string, Endpoint[]> | undefined {
	const list = dataList(raw);
	if (list === undefined) return undefined;
	const endpoints = new Map<string, Endpoint[]>();
	for (const item of list) {
		const endpoint = parseEndpoint(item);
		if (endpoint === undefined) continue;
		const known = endpoints.get(endpoint.model_id) ?? [];
		known.push(endpoint);
		endpoints.set(endpoint.model_id, known);
	}
	return endpoints;
}

export function usableHosts(endpoints: readonly Endpoint[], params: readonly ParamNeed[]): Endpoint[] {
	return endpoints.filter((endpoint) =>
		params.every((param) =>
			(typeof param === "string" ? [param] : param).some((name) => endpoint.supported_parameters?.includes(name)),
		),
	);
}

/** Checks model eligibility. */
export function chatStanding(
	expiration: string | null | undefined,
	endpoints: readonly Endpoint[],
	params: readonly ParamNeed[],
	now: number,
): { standing: "ok" | "retired" | "noZdrHost"; hosts: Endpoint[] } {
	if (expiration != null && expiryOf(expiration) <= now) return { standing: "retired", hosts: [] };
	const hosts = usableHosts(endpoints, params);
	return { standing: hosts.length === 0 ? "noZdrHost" : "ok", hosts };
}

/** The model's own listing stands for its hosts that keep data. */
function listedHost(entry: ModelEntry): Endpoint {
	return {
		model_id: entry.id,
		tag: null,
		provider_name: null,
		context_length: entry.context_length,
		max_prompt_tokens: null,
		supported_parameters: entry.supported_parameters,
		pricing: entry.pricing,
	};
}

/** Catalog-backed model views. */
export function joinCatalog(
	rows: readonly ModelRow[],
	models: unknown,
	zdr: unknown,
	now: number,
	use: ModelUse = "chat",
	zdrOnly = true,
): ModelView[] {
	const entries = parseModels(models);
	const endpoints = parseEndpoints(zdr);
	if (entries === undefined || endpoints === undefined) return rows.filter(bound).map(unknownView);
	return joinParsed(rows, entries, endpoints, now, use, zdrOnly);
}

function bound(row: ModelRow): row is ModelRow & { bindings: { openrouter: Binding } } {
	return row.bindings["openrouter"] !== undefined;
}

function joinParsed(
	rows: readonly ModelRow[],
	entries: ReadonlyMap<string, ModelEntry>,
	endpoints: ReadonlyMap<string, Endpoint[]>,
	now: number,
	use: ModelUse,
	zdrOnly: boolean,
): ModelView[] {
	return rows.filter(bound).map((row): ModelView => {
		const { id } = row.bindings.openrouter;
		const entry = entries.get(id);
		if (entry === undefined) return { row, status: "retired", efforts: [], hosts: [] };
		const retiredOn = entry.expiration_date ?? undefined;
		const offered = entry.reasoning?.supported_efforts ?? [];
		const efforts = row.reasoning ? EFFORTS.filter((effort) => offered.includes(effort)) : [];
		const base = { row, ...(retiredOn === undefined ? {} : { retiredOn }), efforts };
		const zdr = endpoints.get(id) ?? [];
		const reached = zdrOnly ? zdr : [...zdr, listedHost(entry)];
		const { standing, hosts } = chatStanding(retiredOn, reached, sentParams(row, use), now);
		if (standing !== "ok") return { ...base, status: standing, hosts: [] };
		const limits = hosts
			.map((host) => host.max_prompt_tokens ?? host.context_length ?? entry.context_length)
			.filter((limit): limit is number => typeof limit === "number");
		const prompt = rangeOf(hosts.map((host) => perMillion(host.pricing?.prompt)));
		const completion = rangeOf(hosts.map((host) => perMillion(host.pricing?.completion)));
		return {
			...base,
			status: "ok",
			...(limits.length === 0 ? {} : { promptLimit: Math.min(...limits) }),
			...(prompt === undefined || completion === undefined
				? {}
				: { price: { prompt, completion, tiers: tiersOf(entry.pricing) } }),
			...(perMillion(entry.pricing?.input_cache_write_1h) === undefined ? {} : { cacheWrite1h: true as const }),
			hosts: hosts.map((host) => host.tag ?? host.provider_name ?? "").filter((tag) => tag.length > 0),
			...(hosts.some((host) => zdr.includes(host)) ? {} : { dataKept: true as const }),
		};
	});
}

////////////////////////////////
//  Classes

/** A caller's own views; the cached ones are never handed out. */
function copyOf(views: readonly ModelView[]): ModelView[] {
	return views.map((view) => {
		const { row, ...rest } = view;
		return { ...(parseJson(stringifyJson(rest)) as Omit<ModelView, "row">), row };
	});
}

/** Shared 12-hour cache; failed refreshes keep the last good data. */
export class ModelCatalog {
	private good: Fetched | undefined;
	private failedAt: number | undefined;
	private inflight: Promise<Fetched | undefined> | undefined;
	private readonly joined = new Map<ModelUse, Joined>();
	/** Counts the registry's changes; a join taken before one is stale. */
	private version = 0;
	private readonly unsubscribe: () => void;

	constructor(private readonly d: ModelCatalogDeps) {
		this.unsubscribe = d.rows.onChange(() => {
			this.version += 1;
		});
	}

	/** Stops listening to the registry. */
	dispose(): void {
		this.unsubscribe();
	}

	/** Answers when no refresh is running, having waited for the one that was. */
	async idle(): Promise<void> {
		while (this.inflight !== undefined) await this.inflight;
	}

	/** Rows bound under OpenRouter; off `zdrOnly`, hosts that keep data count. */
	async views(use: ModelUse = "chat", zdrOnly = true, signal?: AbortSignal): Promise<ModelView[]> {
		return this.join(await raced(this.fetched(), signal), use, zdrOnly);
	}

	async view(
		key: string,
		use: ModelUse = "chat",
		zdrOnly = true,
		signal?: AbortSignal,
	): Promise<ModelView | undefined> {
		return (await this.views(use, zdrOnly, signal)).find((view) => view.row.key === key);
	}

	/** Joins a non-registry row on zero-retention hosts, as the tester checks it. */
	async viewFor(row: ModelRow, use: ModelUse = "chat", signal?: AbortSignal): Promise<ModelView> {
		const data = await raced(this.fetched(), signal);
		if (data === undefined) return unknownView(row);
		const [view] = joinParsed([row], data.entries, data.endpoints, this.d.scheduler.nowMs(), use, true);
		return view ?? unknownView(row);
	}

	async entry(id: string, signal?: AbortSignal): Promise<ModelEntry | undefined> {
		return (await raced(this.fetched(), signal))?.entries.get(id);
	}

	/** Undefined if fetching has never succeeded. */
	async listing(signal?: AbortSignal, retry = false): Promise<CatalogListing | undefined> {
		if (retry) this.failedAt = undefined;
		const data = await raced(this.fetched(), signal);
		return data === undefined ? undefined : { raw: data.raw, endpoints: data.endpoints };
	}

	private fetched(): Promise<Fetched | undefined> {
		const now = this.d.scheduler.nowMs();
		if (this.good !== undefined && now - this.good.at < (this.d.ttlMs ?? TTL_MS)) return Promise.resolve(this.good);
		if (this.failedAt !== undefined && now - this.failedAt < RETRY_MS) return Promise.resolve(this.good);
		this.inflight ??= this.refresh().finally(() => {
			this.inflight = undefined;
		});
		return this.inflight;
	}

	/** Reuses the views until the rows, the data, the flag or a removal date changes. */
	private join(data: Fetched | undefined, use: ModelUse, zdrOnly: boolean): ModelView[] {
		const now = this.d.scheduler.nowMs();
		const known = this.joined.get(use);
		if (
			known !== undefined &&
			known.version === this.version &&
			known.data === data &&
			known.zdrOnly === zdrOnly &&
			now < known.until
		)
			return copyOf(known.views);
		const version = this.version;
		const rows = this.d.rows.rows();
		const views =
			data === undefined
				? rows.filter(bound).map(unknownView)
				: joinParsed(rows, data.entries, data.endpoints, now, use, zdrOnly);
		const removals = views.map((view) => expiryOf(view.retiredOn ?? "")).filter((at) => at > now);
		this.joined.set(use, { version, data, zdrOnly, views, until: Math.min(Number.POSITIVE_INFINITY, ...removals) });
		return copyOf(views);
	}

	private async refresh(): Promise<Fetched | undefined> {
		const limit = deadline(this.d.scheduler, FETCH_MS);
		try {
			const [models, zdr] = await joinSettled([
				() => this.d.transport.getJson("/models", limit.signal, "none"),
				() => this.d.transport.getJson("/endpoints/zdr", limit.signal, "none"),
			]);
			const entries = parseModels(models);
			const endpoints = parseEndpoints(zdr);
			const listed = dataList(models);
			if (entries === undefined || endpoints === undefined || listed === undefined)
				throw new Error("The catalog did not parse.");
			this.good = { entries, endpoints, raw: listed, at: this.d.scheduler.nowMs() };
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
