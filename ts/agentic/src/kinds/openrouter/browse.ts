// OpenRouter model browser.

import type { BrowseModel, ProviderRow, RowIdentity } from "@nyaaai/contract/shapes/model";
import { MODELS } from "../../models/curated";
import { isModelId } from "../../models/registry";
import { rowKey, shownId } from "../../models/rowKey";
import { fieldOf, fieldsOf, numberField, objectField, textField, textsField } from "../readers";
import { type CatalogListing, chatStanding, type Endpoint, expiryOf, perMillion, rangeOf } from "./catalog";

////////////////////////////////
//  Interfaces & Types

/** A missing, null or mistyped field is null. */
interface BrowseEntry {
	id: string;
	name: string | null;
	description: string | null;
	context_length: number | null;
	input_modalities: string[] | null;
	output_modalities: string[] | null;
	supported_parameters: string[] | null;
	expiration_date: string | null;
	supported_efforts: string[] | null;
}

interface ProviderEndpoint {
	provider_name: string | null;
	name: string | null;
	tag: string | null;
	context_length: number | null;
	max_completion_tokens: number | null;
	quantization: string | null;
	uptime_last_30m: number | null;
	prices: { prompt: unknown; completion: unknown; input_cache_read: unknown } | null;
}

////////////////////////////////
//  Constants

/** Exclude batch twins. */
const BATCH = ":batch";

const CURATED: ReadonlySet<string> = new Set(MODELS.map((row) => row.key));

/** Cache through expiration. */
const parsed = new WeakMap<readonly unknown[], { models: BrowseModel[]; until: number }>();

////////////////////////////////
//  Functions & Helpers

function entryOf(item: unknown): BrowseEntry | undefined {
	const fields = fieldsOf(item);
	const id = fields === undefined ? undefined : fieldOf(fields, "id");
	if (fields === undefined || typeof id !== "string") return undefined;
	const architecture = objectField(fields, "architecture");
	const reasoning = objectField(fields, "reasoning");
	return {
		id,
		name: textField(fields, "name"),
		description: textField(fields, "description"),
		context_length: numberField(fields, "context_length"),
		input_modalities: architecture === null ? null : textsField(architecture, "input_modalities"),
		output_modalities: architecture === null ? null : textsField(architecture, "output_modalities"),
		supported_parameters: textsField(fields, "supported_parameters"),
		expiration_date: textField(fields, "expiration_date"),
		supported_efforts: reasoning === null ? null : textsField(reasoning, "supported_efforts"),
	};
}

function endpointOf(item: unknown): ProviderEndpoint | undefined {
	const fields = fieldsOf(item);
	if (fields === undefined) return undefined;
	const pricing = objectField(fields, "pricing");
	return {
		provider_name: textField(fields, "provider_name"),
		name: textField(fields, "name"),
		tag: textField(fields, "tag"),
		context_length: numberField(fields, "context_length"),
		max_completion_tokens: numberField(fields, "max_completion_tokens"),
		quantization: textField(fields, "quantization"),
		uptime_last_30m: numberField(fields, "uptime_last_30m"),
		prices:
			pricing === null
				? null
				: {
						prompt: fieldOf(pricing, "prompt"),
						completion: fieldOf(pricing, "completion"),
						input_cache_read: fieldOf(pricing, "input_cache_read"),
					},
	};
}

function chatParams(reasoning: boolean): string[] {
	return reasoning ? ["tools", "reasoning"] : ["tools"];
}

export function browseModels(listing: CatalogListing, now: number): BrowseModel[] {
	const known = parsed.get(listing.raw);
	if (known !== undefined && now < known.until) return known.models;
	const expirations: number[] = [];
	const models = listing.raw.flatMap((item): BrowseModel[] => {
		const entry = entryOf(item);
		// Host-accepted IDs only.
		if (entry === undefined || entry.id.endsWith(BATCH) || !isModelId(entry.id)) return [];
		const { id } = entry;
		const reasoning = (entry.supported_efforts ?? []).length > 0;
		const zdr: readonly Endpoint[] = listing.endpoints.get(id) ?? [];
		const expiration = entry.expiration_date;
		if (expiration != null) expirations.push(expiryOf(expiration));
		const parameters = entry.supported_parameters ?? [];
		const identity: RowIdentity = { kind: "hosted", provider: "openrouter", id };
		return [
			{
				id: rowKey(identity),
				display: shownId(identity),
				label: entry.name ?? id,
				curated: CURATED.has(id),
				description: entry.description ?? "",
				...(entry.context_length === null ? {} : { context: entry.context_length }),
				inputs: entry.input_modalities ?? [],
				outputs: entry.output_modalities ?? [],
				parameters,
				price: priceRanges(zdr),
				tools: chatStanding(expiration, zdr, chatParams(reasoning), now).standing === "ok" ? "ok" : "noZdrHost",
			},
		];
	});
	parsed.set(listing.raw, {
		models,
		until: Math.min(Number.POSITIVE_INFINITY, ...expirations.filter((at) => at > now)),
	});
	return models;
}

function priceRanges(endpoints: readonly Endpoint[]): BrowseModel["price"] {
	const prompt = rangeOf(endpoints.map((endpoint) => perMillion(endpoint.pricing?.prompt)));
	const completion = rangeOf(endpoints.map((endpoint) => perMillion(endpoint.pricing?.completion)));
	return prompt === undefined || completion === undefined ? "unknown" : { prompt, completion };
}

/** Provider endpoints, ZDR status. */
export function parseProviders(raw: unknown, zdr: readonly Endpoint[]): ProviderRow[] | undefined {
	const body = fieldsOf(raw);
	const data = body === undefined ? null : objectField(body, "data");
	const endpoints = data === null ? undefined : fieldOf(data, "endpoints");
	if (!Array.isArray(endpoints)) return undefined;
	const tags = new Set(zdr.flatMap((endpoint) => (endpoint.tag ? [endpoint.tag] : [])));
	return endpoints.flatMap((item): ProviderRow[] => {
		const provider = endpointOf(item);
		if (provider === undefined) return [];
		const tag = provider.tag ?? "";
		const prompt = perMillion(provider.prices?.prompt);
		const completion = perMillion(provider.prices?.completion);
		const cacheRead = perMillion(provider.prices?.input_cache_read);
		return [
			{
				name: provider.provider_name ?? provider.name ?? tag,
				tag,
				...(provider.context_length === null ? {} : { context: provider.context_length }),
				...(provider.max_completion_tokens === null ? {} : { maxOutput: provider.max_completion_tokens }),
				...(provider.quantization === null ? {} : { quantization: provider.quantization }),
				...(prompt === undefined ? {} : { prompt }),
				...(completion === undefined ? {} : { completion }),
				...(cacheRead === undefined ? {} : { cacheRead }),
				...(provider.uptime_last_30m === null ? {} : { uptime: provider.uptime_last_30m }),
				zdr: tags.has(tag),
			},
		];
	});
}
