// Provider entries: the ids, the URL rules and the lenient reader of a stored list.

import { Pattern } from "@nyaaai/contract/pattern";
import { type ProviderEntry, ProviderKindName, ProviderPlan } from "@nyaaai/contract/shapes/provider";
import { trimText } from "@nyaaai/contract/text";
import { parseUrl, type UrlParts } from "@nyaaai/contract/url";

////////////////////////////////
//  Constants

export const PROVIDER_KINDS: readonly ProviderKindName[] = ProviderKindName.options;

export const PROVIDER_PLANS: readonly ProviderPlan[] = ProviderPlan.options;

const URL_KINDS: readonly ProviderKindName[] = ["ollama", "lmstudio", "llamacpp"];

const ENTRY_ID = Pattern.compile("^[\\w-]{1,64}$");

const LABEL_CHARS = 120;

////////////////////////////////
//  Functions & Helpers

/** A word-character id; `__proto__` is no id. */
export function isEntryId(value: unknown): value is string {
	return typeof value === "string" && value !== "__proto__" && ENTRY_ID.test(value);
}

/** The entry's secret key under the consumer's namespace. */
export function secretKey(id: string, namespace: string): string {
	return `${namespace}.provider.${id}`;
}

/** HTTP(S), anonymous, no query. */
function webUrl(value: unknown): UrlParts | undefined {
	if (typeof value !== "string") return undefined;
	const url = parseUrl(trimText(value));
	if (url === undefined) return undefined;
	const web = url.scheme === "http" || url.scheme === "https";
	const plain = url.query === "" && url.fragment === "";
	const anonymous = url.username === "" && url.password === "";
	return web && plain && anonymous && url.host.length > 0 ? url : undefined;
}

function serverOrigin(value: unknown): string | undefined {
	const url = webUrl(value);
	return url?.path === "/" ? url.origin : undefined;
}

/** Path kept, no trailing slash. */
function serverBase(value: unknown): string | undefined {
	const url = webUrl(value);
	if (url === undefined) return undefined;
	let base = `${url.origin}${url.path}`;
	while (base.endsWith("/")) base = base.slice(0, -1);
	return base;
}

/** Both URLs parse and share an origin. */
export function sameOrigin(left: string | undefined, right: string | undefined): boolean {
	const origin = (url: string | undefined) => webUrl(url)?.origin;
	return origin(left) !== undefined && origin(left) === origin(right);
}

/** Parse a provider entry; the label counts UTF-16 units. */
export function parseEntry(value: unknown): ProviderEntry | undefined {
	if (typeof value !== "object" || value === null || Array.isArray(value)) return undefined;
	const raw = value as Record<string, unknown>;
	const kind = PROVIDER_KINDS.find((each) => each === raw["kind"]);
	const label = typeof raw["label"] === "string" ? trimText(raw["label"]) : "";
	const id = raw["id"];
	if (!isEntryId(id) || kind === undefined || label.length === 0 || label.length > LABEL_CHARS) return undefined;
	const entry: ProviderEntry = { id, kind, label };
	if (kind === "opencode") {
		const plan = PROVIDER_PLANS.find((each) => each === raw["plan"]);
		if (plan === undefined) return undefined;
		entry.plan = plan;
		const extraUsage = raw["extraUsage"];
		if (plan === "go" && typeof extraUsage === "boolean") entry.extraUsage = extraUsage;
	}
	if (URL_KINDS.includes(kind) && raw["url"] !== undefined) {
		const url = serverOrigin(raw["url"]);
		if (url === undefined) return undefined;
		entry.url = url;
	}
	if (kind === "compatible") {
		const url = serverBase(raw["url"]);
		if (url === undefined) return undefined;
		entry.url = url;
	}
	return entry;
}

/** The entries that read, in order; a repeated id keeps its first. */
export function parseEntries(value: unknown): ProviderEntry[] {
	if (!Array.isArray(value)) return [];
	const seen = new Set<string>();
	return value.flatMap((each): ProviderEntry[] => {
		const entry = parseEntry(each);
		if (entry === undefined || seen.has(entry.id)) return [];
		seen.add(entry.id);
		return [entry];
	});
}
