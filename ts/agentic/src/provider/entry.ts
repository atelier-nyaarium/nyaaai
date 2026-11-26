// Provider entries and the reader of a stored list.

import type { KeyAnswer } from "@nyaaai/contract/entryKeys";
import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import { Pattern } from "@nyaaai/contract/pattern";
import { type EntryFields, type ProviderEntry, ProviderKindName, ProviderPlan } from "@nyaaai/contract/shapes/provider";
import { trimText } from "@nyaaai/contract/text";
import { parseUrl, type UrlParts } from "@nyaaai/contract/url";

////////////////////////////////
//  Interfaces & Types

/** What a kind tells of its key. */
export interface KeyKind {
	readonly label: string;
	/** Plan dropdown lines. */
	readonly plans?: readonly { plan: ProviderPlan; label: string }[];
	/** The plan of an entry that names none. */
	readonly defaultPlan?: ProviderPlan;
	/** Add-form third field: `key` for the kind's own address. */
	readonly field: "key" | "url";
	/** A URL kind that takes a key at the entry's address. */
	readonly optionalKey?: true;
}

export type KeyKinds = Partial<Record<ProviderKindName, KeyKind>>;

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

/** A key for the kind's own address, or for the entry's. */
export function takesKey(kind: KeyKind): boolean {
	return kind.field === "key" || kind.optionalKey === true;
}

/** What a key is for: the kind, its effective plan, and the origin of an address the person gives. */
export function keyBindingOf(fields: EntryFields, kinds: KeyKinds): string | undefined {
	const kind = kinds[fields.kind];
	if (kind === undefined || !takesKey(kind)) return undefined;
	let binding: string = fields.kind;
	if (kind.plans !== undefined && kind.plans.length > 0) {
		const plan = fields.plan ?? kind.defaultPlan;
		if (plan === undefined) return undefined;
		binding += ` ${plan}`;
	}
	if (kind.field === "url") {
		const origin = webUrl(fields.url)?.origin;
		if (origin === undefined) return undefined;
		binding += ` ${origin}`;
	}
	return binding;
}

/** The text a host stores for a key; none when the fields bind no key. */
export function boundKey(fields: EntryFields, key: string, kinds: KeyKinds): string | undefined {
	const binding = keyBindingOf(fields, kinds);
	return binding === undefined ? undefined : stringifyJson({ v: 1, for: binding, key });
}

/** A stored text's key and binding; none for any other text. */
export function readBoundKey(text: string): Extract<KeyAnswer, { kind: "key" | "none" }> {
	const value = parseJson(text);
	if (typeof value !== "object" || value === null || Array.isArray(value)) return { kind: "none" };
	const held = value as Record<string, unknown>;
	const key = held["key"];
	const binding = held["for"];
	const exact = Object.keys(held).length === 3 && held["v"] === 1;
	return exact && typeof key === "string" && typeof binding === "string"
		? { kind: "key", key, boundTo: binding }
		: { kind: "none" };
}

/** Whom a binding names: its origin's host, else its kind's label, else its kind name. */
export function boundFor(binding: string, kinds: KeyKinds): string {
	const parts = binding.split(" ");
	const host = parseUrl(parts[parts.length - 1] ?? "")?.host;
	if (host !== undefined) return host;
	const name = parts[0] ?? "";
	const kind = PROVIDER_KINDS.find((each) => each === name);
	return (kind === undefined ? undefined : kinds[kind]?.label) ?? name;
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
