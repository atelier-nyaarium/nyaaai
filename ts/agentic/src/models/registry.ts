// The stored model registry and the lists a pick reads.

import { Pattern } from "@nyaaai/contract/pattern";
import type {
	ModelRow,
	ModelUse,
	ModelView,
	RowIdentity,
	StoredRegistry,
	TestRecord,
} from "@nyaaai/contract/shapes/model";
import type { ProviderEntry, ProviderKindName } from "@nyaaai/contract/shapes/provider";
import { trimText } from "@nyaaai/contract/text";
import { isEntryId } from "../provider/entry";
import { MODELS } from "./curated";
import { parseRowKey, rowKey, servedIdOf } from "./rowKey";

////////////////////////////////
//  Interfaces & Types

/** A row a test can record. */
export type TestedIdentity = Exclude<RowIdentity, { kind: "curated" }>;

/** One entry's registry, or the merge of several. */
export interface RegistrySettings {
	tested: ReadonlyMap<string, TestRecord>;
	/** Manual IDs, ordered. */
	added: readonly string[];
	/** Chat-hidden families and IDs. */
	hidden: ReadonlySet<string>;
}

export interface Selection {
	id: string | undefined;
	/** Configured model unavailable. */
	marked: boolean;
}

/** Chat's gates. */
type Gate = "toolPlus" | "recovery" | "reasoning";

type Verdict = TestRecord["checks"]["toolPlus"];

////////////////////////////////
//  Constants

/** Fallback completion price cap. */
const CHEAP_COMPLETION = 2;

const VERDICTS: readonly Verdict[] = ["pass", "fail", "n/a"];

const CACHES: readonly TestRecord["cache"][] = ["implicit", "explicit", "none"];

const RECORD_DATE = Pattern.compile("^\\d{4}-\\d{2}-\\d{2}$");

/** Reject dot segments. */
const MODEL_ID = Pattern.compile("^\\w[\\w.-]*(?:\\/\\w[\\w.:-]*)+$");

const LABEL_CHARS = 200;

const CURATED: ReadonlySet<string> = new Set(MODELS.map((row) => row.key));

const FAMILIES: ReadonlySet<string> = new Set(MODELS.map((row) => row.family));

////////////////////////////////
//  Functions & Helpers

function strings(value: unknown): string[] {
	return Array.isArray(value) ? value.filter((each): each is string => typeof each === "string") : [];
}

/** The object itself, or undefined for any other value. */
function plain(value: unknown): Record<string, unknown> | undefined {
	return typeof value === "object" && value !== null && !Array.isArray(value)
		? (value as Record<string, unknown>)
		: undefined;
}

const objectOf = (value: unknown): Record<string, unknown> => plain(value) ?? {};

function own<T>(record: Readonly<Record<string, T>>, key: string): T | undefined {
	return Object.hasOwn(record, key) ? record[key] : undefined;
}

/** Unicode scalars; a lone surrogate counts one. */
function codePoints(text: string): number {
	let count = 0;
	for (const _ of text) count += 1;
	return count;
}

const isCost = (value: unknown): value is number => typeof value === "number" && Number.isFinite(value) && value >= 0;

function verdictOf(value: unknown): Verdict | undefined {
	return VERDICTS.find((each) => each === value);
}

/** A stored test record, or undefined; the label counts code points. */
export function parseTestRecord(value: unknown): TestRecord | undefined {
	const raw = plain(value);
	if (raw === undefined) return undefined;
	const label = typeof raw["label"] === "string" ? trimText(raw["label"]) : "";
	const date = raw["date"];
	const cost = raw["cost"];
	const estimated = raw["estimated"];
	const checks = plain(raw["checks"]);
	const toolPlus = verdictOf(checks?.["toolPlus"]);
	const recovery = verdictOf(checks?.["recovery"]);
	const reasoningCheck = verdictOf(checks?.["reasoning"]);
	const schema = verdictOf(checks?.["schema"]);
	const reasoning = raw["reasoning"];
	const cache = CACHES.find((each) => each === raw["cache"]);
	const price = plain(raw["verifiedPrice"]);
	const prompt = price?.["prompt"];
	const completion = price?.["completion"];
	if (label.length === 0 || codePoints(label) > LABEL_CHARS) return undefined;
	if (typeof date !== "string" || !RECORD_DATE.test(date)) return undefined;
	if (!isCost(cost) || (estimated !== undefined && typeof estimated !== "boolean")) return undefined;
	if (toolPlus === undefined || recovery === undefined || reasoningCheck === undefined || schema === undefined)
		return undefined;
	if (typeof reasoning !== "boolean" || cache === undefined || !isCost(prompt) || !isCost(completion))
		return undefined;
	return {
		label,
		date,
		cost,
		...(estimated === undefined ? {} : { estimated }),
		checks: { toolPlus, recovery, reasoning: reasoningCheck, schema },
		reasoning,
		cache,
		verifiedPrice: { prompt, completion },
	};
}

export function isModelId(value: unknown): value is string {
	return typeof value === "string" && MODEL_ID.test(value);
}

export function curatedUnder(kind: ProviderKindName, id: string): boolean {
	return MODELS.some((row) => row.bindings[kind]?.id === id);
}

function localTo(entry: string | undefined, key: string): boolean {
	const identity = parseRowKey(key);
	return identity?.kind === "local" && identity.entry === entry;
}

/** The first gate a Chat model fails; Schema is optional. */
export function failedGate(checks: Pick<TestRecord["checks"], Gate>): Gate | undefined {
	if (checks.toolPlus !== "pass") return "toolPlus";
	if (checks.recovery === "fail") return "recovery";
	if (checks.reasoning === "fail") return "reasoning";
	return undefined;
}

export function passed(record: TestRecord): boolean {
	return failedGate(record.checks) === undefined;
}

/** Hosted rows, or the entry's own local rows. */
export function testedUnder(key: string, entry: string): boolean {
	const identity = parseRowKey(key);
	return identity?.kind === "hosted" || (identity?.kind === "local" && identity.entry === entry);
}

/** Keeps removed entries' records; hidden keeps listed entries' local keys. */
export function parseRegistry(
	raw: { records?: unknown; added?: unknown; hidden?: unknown },
	entries: readonly Pick<ProviderEntry, "id">[] = [],
): StoredRegistry {
	const tested: Record<string, Record<string, TestRecord>> = {};
	for (const [entry, records] of Object.entries(objectOf(raw.records))) {
		const valid: Record<string, TestRecord> = {};
		let count = 0;
		for (const [key, value] of Object.entries(objectOf(records))) {
			const record = parseTestRecord(value);
			if (testedUnder(key, entry) && record !== undefined) {
				valid[key] = record;
				count += 1;
			}
		}
		if (isEntryId(entry) && entry !== "__proto__" && count > 0) tested[entry] = valid;
	}
	const added: Record<string, string[]> = {};
	for (const [entry, keys] of Object.entries(objectOf(raw.added))) {
		const kept = [...new Set(strings(keys))].filter((key) => {
			const record = own(own(tested, entry) ?? {}, key);
			return record !== undefined && passed(record);
		});
		if (kept.length > 0) added[entry] = kept;
	}
	const keys = new Set(Object.values(added).flat());
	const hidden = strings(raw.hidden).filter(
		(each) => FAMILIES.has(each) || keys.has(each) || entries.some((entry) => localTo(entry.id, each)),
	);
	return { tested, added, hidden: [...new Set(hidden)] };
}

/** No-entry result is empty. */
export function registryFor(stored: StoredRegistry, entry: string | undefined): RegistrySettings {
	const records = entry === undefined ? undefined : own(stored.tested, entry);
	const tested = new Map(Object.entries(records ?? {}));
	const added = entry === undefined ? [] : [...(own(stored.added, entry) ?? [])];
	const hidden = stored.hidden.filter((each) => FAMILIES.has(each) || added.includes(each) || localTo(entry, each));
	return { tested, added, hidden: new Set(hidden) };
}

/** Merge with first-entry precedence. */
export function registryOf(stored: StoredRegistry, entries: readonly Pick<ProviderEntry, "id">[]): RegistrySettings {
	const tested = new Map<string, TestRecord>();
	const added: string[] = [];
	for (const entry of entries) {
		const one = registryFor(stored, entry.id);
		for (const [model, record] of one.tested) if (!tested.has(model)) tested.set(model, record);
		for (const model of one.added) if (!added.includes(model)) added.push(model);
	}
	const hidden = stored.hidden.filter(
		(each) => FAMILIES.has(each) || added.includes(each) || entries.some((entry) => localTo(entry.id, each)),
	);
	return { tested, added, hidden: new Set(hidden) };
}

/** Bound to the testing entry's kind. */
export function rowOf(identity: TestedIdentity, record: TestRecord, kind: ProviderKindName): ModelRow {
	const key = rowKey(identity);
	return {
		key,
		bindings: { [kind]: { id: servedIdOf(identity) } },
		family: key,
		label: record.label,
		reasoning: record.reasoning,
		schema: record.checks.schema === "pass",
		cache: record.cache === "explicit" ? "explicit" : "implicit",
		verified: record.date,
		verifiedPrice: record.verifiedPrice,
		identity,
	};
}

/** Retains removed thread models; first entry wins a shared hosted row. */
export function registryRows(
	stored: StoredRegistry,
	entries: readonly Pick<ProviderEntry, "id" | "kind">[],
): ModelRow[] {
	const rows = new Map(MODELS.map((row) => [row.key, row]));
	for (const entry of entries) {
		const settings = registryFor(stored, entry.id);
		const removed = [...settings.tested.keys()].filter((key) => !settings.added.includes(key));
		for (const key of [...settings.added, ...removed]) {
			const record = settings.tested.get(key);
			const identity = parseRowKey(key);
			if (record === undefined || !passed(record) || rows.has(key)) continue;
			const hosted =
				identity?.kind === "hosted" &&
				identity.provider === entry.kind &&
				!curatedUnder(entry.kind, identity.id);
			if (hosted || identity?.kind === "local") rows.set(key, rowOf(identity, record, entry.kind));
		}
	}
	return [...rows.values()];
}

export function offered(row: ModelRow, settings: RegistrySettings): boolean {
	return CURATED.has(row.key) || settings.added.includes(row.key);
}

/** The models a pick can list. */
export function modelList(views: readonly ModelView[], use: ModelUse, settings: RegistrySettings): ModelView[] {
	const fits = (row: ModelRow) =>
		(!use.hides || !settings.hidden.has(row.family)) &&
		use.requires.every((ability) => ability === "schema" && row.schema);
	return views.filter(
		(view) =>
			offered(view.row, settings) && view.status !== "retired" && view.status !== "noZdrHost" && fits(view.row),
	);
}

/** Prefer catalog price; unknown is never cheap. */
function completionPrice({ price, row }: ModelView): number {
	if (price === "free") return 0;
	if (price === "unknown") return Number.POSITIVE_INFINITY;
	return price?.completion[1] ?? row.verifiedPrice.completion;
}

/** Marked cheap fallback. */
export function select(wanted: string, list: readonly ModelView[]): Selection {
	if (list.some((view) => view.row.key === wanted)) return { id: wanted, marked: false };
	return { id: list.find((view) => completionPrice(view) <= CHEAP_COMPLETION)?.row.key, marked: true };
}
