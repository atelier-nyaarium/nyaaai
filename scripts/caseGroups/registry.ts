import { MODELS } from "@nyaaai/agentic/models/curated";
import {
	curatedUnder,
	failedGate,
	isModelId,
	modelList,
	offered,
	parseRegistry,
	parseTestRecord,
	passed,
	type RegistrySettings,
	registryFor,
	registryOf,
	registryRows,
	select,
	testedUnder,
} from "@nyaaai/agentic/models/registry";
import { z } from "@nyaaai/contract/shape";
import { ModelRow, ModelUse, type ModelView, Price, StoredRegistry, TestRecord } from "@nyaaai/contract/shapes/model";
import { ProviderKindName } from "@nyaaai/contract/shapes/provider";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { registryCases } from "../caseSources/registry";

////////////////////////////////
//  Shapes

/** The three settings values a registry is read from. */
const Raw = () =>
	z.object({
		records: z.unknown().exactOptional(),
		added: z.unknown().exactOptional(),
		hidden: z.unknown().exactOptional(),
	});

const Listed = () => z.object({ id: z.string(), kind: ProviderKindName });

/** One entry's registry, or the merge of several. */
const Settings = () =>
	z.object({ tested: z.record(z.string(), TestRecord), added: z.array(z.string()), hidden: z.array(z.string()) });

const Verdict = () => z.enum(["pass", "fail", "n/a"]);

////////////////////////////////
//  Functions & Helpers

const asSettings = (settings: RegistrySettings) => ({
	tested: Object.fromEntries(settings.tested),
	added: [...settings.added],
	hidden: [...settings.hidden],
});

function rowsOf(
	raw: { records?: unknown; added?: unknown; hidden?: unknown },
	entries: ReadonlyArray<{ id: string; kind: ProviderKindName }>,
) {
	const stored = parseRegistry(raw, entries);
	return { stored, rows: registryRows(stored, entries) };
}

/** A view of a row the registry holds. */
function viewOf(rows: readonly ModelRow[], key: string, status: ModelView["status"]): ModelView {
	const row = rows.find((each) => each.key === key);
	if (row === undefined) throw new Error(`No row ${key}.`);
	return { row, status, efforts: [], hosts: [] };
}

////////////////////////////////
//  Group

export const registry = caseGroup("registry", {
	kinds: {
		/** Null: the value is not a test record. */
		testRecord: kind(
			z.object({ values: z.array(z.unknown()) }),
			z.object({ records: z.array(TestRecord.nullable()) }),
			(input) => ({ records: input.values.map((value) => parseTestRecord(value) ?? null) }),
		),
		parse: kind(
			z.object({ raw: Raw(), entries: z.array(z.object({ id: z.string() })) }),
			z.object({ stored: StoredRegistry }),
			(input) => ({ stored: parseRegistry(input.raw, input.entries) }),
		),
		/** `scoped` per entry id (null: none), `merged` over all `entries`. */
		scoped: kind(
			z.object({ raw: Raw(), entries: z.array(Listed()), scopes: z.array(z.string().nullable()) }),
			z.object({ scoped: z.array(Settings()), merged: Settings() }),
			(input) => {
				const stored = parseRegistry(input.raw, input.entries);
				return {
					scoped: input.scopes.map((scope) => asSettings(registryFor(stored, scope ?? undefined))),
					merged: asSettings(registryOf(stored, input.entries)),
				};
			},
		),
		/** `curatedFirst`: the curated list leads, whole. `rest`: the rows after it. */
		rows: kind(
			z.object({ raw: Raw(), entries: z.array(Listed()) }),
			z.object({ curatedFirst: z.boolean(), rest: z.array(ModelRow) }),
			(input) => {
				const { rows } = rowsOf(input.raw, input.entries);
				const head = JSON.stringify(rows.slice(0, MODELS.length)) === JSON.stringify(MODELS);
				return { curatedFirst: head, rest: rows.slice(MODELS.length) };
			},
		),
		/** Null: no such row. */
		offered: kind(
			z.object({
				raw: Raw(),
				entries: z.array(Listed()),
				scope: z.string().nullable(),
				keys: z.array(z.string()),
			}),
			z.object({ offered: z.array(z.boolean().nullable()) }),
			(input) => {
				const { stored, rows } = rowsOf(input.raw, input.entries);
				const settings = registryFor(stored, input.scope ?? undefined);
				return {
					offered: input.keys.map((key) => {
						const row = rows.find((each) => each.key === key);
						return row === undefined ? null : offered(row, settings);
					}),
				};
			},
		),
		/** The keys a pick lists, in order. */
		list: kind(
			z.object({
				raw: Raw(),
				entries: z.array(Listed()),
				scope: z.string().nullable(),
				use: ModelUse,
				views: z.array(
					z.object({ key: z.string(), status: z.enum(["ok", "retired", "noZdrHost", "noTools", "unknown"]) }),
				),
			}),
			z.object({ keys: z.array(z.string()) }),
			(input) => {
				const { stored, rows } = rowsOf(input.raw, input.entries);
				const settings = registryFor(stored, input.scope ?? undefined);
				const views = input.views.map((view) => viewOf(rows, view.key, view.status));
				return { keys: modelList(views, input.use, settings).map((view) => view.row.key) };
			},
		),
		/** Each listed row has the verified completion price given; `price` is the catalog's. */
		select: kind(
			z.object({
				wanted: z.string(),
				list: z.array(
					z.object({
						key: z.string(),
						verifiedCompletion: z.number(),
						price: z.union([Price, z.literal("free"), z.literal("unknown")]).exactOptional(),
					}),
				),
			}),
			z.object({ id: z.string().nullable(), marked: z.boolean() }),
			(input) => {
				const views = input.list.map((item): ModelView => {
					const row: ModelRow = {
						key: item.key,
						bindings: {},
						family: item.key,
						label: item.key,
						reasoning: false,
						schema: true,
						cache: "implicit",
						verified: "2026-01-01",
						verifiedPrice: { prompt: 0, completion: item.verifiedCompletion },
						identity: { kind: "curated", key: item.key },
					};
					return {
						row,
						status: "ok",
						efforts: [],
						hosts: [],
						...(item.price === undefined ? {} : { price: item.price }),
					};
				});
				const picked = select(input.wanted, views);
				return { id: picked.id ?? null, marked: picked.marked };
			},
		),
		/** `gates`: the first failed gate, null for none. */
		passed: kind(
			z.object({
				rows: z.array(
					z.object({ toolPlus: Verdict(), recovery: Verdict(), reasoning: Verdict(), schema: Verdict() }),
				),
			}),
			z.object({ passed: z.array(z.boolean()), gates: z.array(z.string().nullable()) }),
			(input) => {
				const record = (checks: (typeof input.rows)[number]) => ({
					label: "L",
					date: "2026-01-01",
					cost: 0,
					checks,
					reasoning: true,
					cache: "none" as const,
					verifiedPrice: { prompt: 0, completion: 0 },
				});
				return {
					passed: input.rows.map((checks) => passed(record(checks))),
					gates: input.rows.map((checks) => failedGate(checks) ?? null),
				};
			},
		),
		ids: kind(
			z.object({
				modelIds: z.array(z.unknown()),
				curated: z.array(z.object({ kind: ProviderKindName, id: z.string() })),
				tested: z.array(z.object({ key: z.string(), entry: z.string() })),
			}),
			z.object({
				modelIds: z.array(z.boolean()),
				curated: z.array(z.boolean()),
				tested: z.array(z.boolean()),
			}),
			(input) => ({
				modelIds: input.modelIds.map(isModelId),
				curated: input.curated.map((each) => curatedUnder(each.kind, each.id)),
				tested: input.tested.map((each) => testedUnder(each.key, each.entry)),
			}),
		),
	},
	sources: casesFor("registry", registryCases),
});
