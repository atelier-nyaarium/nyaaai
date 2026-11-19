import { joinOpenCode, OpenCodeCatalog, parseOpenCode } from "@nyaaai/agentic/kinds/opencode/catalog";
import {
	chatStanding,
	type Endpoint,
	expiryOf,
	joinCatalog,
	ModelCatalog,
	type ModelEntry,
	type Pricing,
	parseEndpoints,
	parseModels,
	perMillion,
} from "@nyaaai/agentic/kinds/openrouter/catalog";
import { jsString } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { ModelUse } from "@nyaaai/contract/shapes/model";
import { ProviderPlan } from "@nyaaai/contract/shapes/provider";
import { ApiFamily } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { scriptedModelRegistry } from "@nyaaai/testkit/scriptedPorts";
import { catalogDecodeCases } from "../caseSources/catalogDecode";
import { catalogJoinCases } from "../caseSources/catalogJoin";
import { catalogOpenCodeCases } from "../caseSources/catalogOpenCode";
import { catalogRunCases } from "../caseSources/catalogRun";
import {
	abortOptions,
	callerSignal,
	NetExchange,
	networkOf,
	rowOfKey,
	rowsOfKeys,
	ViewLine,
	viewLine,
} from "./catalogKit";

////////////////////////////////
//  Shapes

/** A price as listed: any JSON; absent is null. */
const OrOverride = () =>
	z.object({
		min_prompt_tokens: z.number(),
		prompt: z.unknown(),
		completion: z.unknown(),
		input_cache_read: z.unknown(),
		input_cache_write: z.unknown(),
		input_cache_write_1h: z.unknown(),
	});

const OrPricing = () =>
	z.object({
		prompt: z.unknown(),
		completion: z.unknown(),
		input_cache_read: z.unknown(),
		input_cache_write: z.unknown(),
		input_cache_write_1h: z.unknown(),
		overrides: z.array(OrOverride()).nullable(),
	});

/** A field missing, null or mistyped is null. */
const OrModel = () =>
	z.object({
		id: z.string(),
		name: z.string().nullable(),
		created: z.number().nullable(),
		context_length: z.number().nullable(),
		expiration_date: z.string().nullable(),
		pricing: OrPricing().nullable(),
		reasoning: z.object({ supported_efforts: z.array(z.string()).nullable() }).nullable(),
		supported_parameters: z.array(z.string()).nullable(),
	});

const OrEndpoint = () =>
	z.object({
		model_id: z.string(),
		tag: z.string().nullable(),
		provider_name: z.string().nullable(),
		context_length: z.number().nullable(),
		max_prompt_tokens: z.number().nullable(),
		supported_parameters: z.array(z.string()).nullable(),
		pricing: OrPricing().nullable(),
	});

const OcApi = () =>
	z.object({
		family: ApiFamily,
		reasoning_field: z.string().nullable(),
		choice: z.array(z.string()).nullable(),
		strict: z.boolean().nullable(),
	});

const OcModel = () =>
	z.object({
		id: z.string(),
		name: z.string().nullable(),
		description: z.string().nullable(),
		input: z.array(z.string()).nullable(),
		output: z.array(z.string()).nullable(),
		tools: z.boolean().nullable(),
		reasoning: z.boolean().nullable(),
		efforts: z.array(z.string()).nullable(),
		toggle: z.boolean().nullable(),
		structured_output: z.boolean().nullable(),
		cache: z.object({ implicit: z.boolean().nullable(), explicit: z.boolean() }).nullable(),
		context: z.number().nullable(),
		input_limit: z.number().nullable(),
		output_limit: z.number().nullable(),
		price: z.object({ input: z.number(), output: z.number() }).nullable(),
		api: z.array(OcApi()),
	});

/** One pass of a join: the use, and whether only zero-retention hosts count. */
const Pass = () => z.object({ use: ModelUse, zdrOnly: z.boolean() });

/** One call on a catalog; a view or a listing, with a cancel of the caller's. */
const OrCall = () =>
	z.discriminatedUnion("kind", [
		z.object({
			kind: z.literal("views"),
			use: ModelUse.exactOptional(),
			zdrOnly: z.boolean().exactOptional(),
			...abortOptions,
		}),
		z.object({
			kind: z.literal("view"),
			key: z.string(),
			use: ModelUse.exactOptional(),
			zdrOnly: z.boolean().exactOptional(),
			...abortOptions,
		}),
		z.object({ kind: z.literal("viewFor"), row: z.string(), use: ModelUse.exactOptional(), ...abortOptions }),
		z.object({ kind: z.literal("entry"), id: z.string(), ...abortOptions }),
		z.object({ kind: z.literal("listing"), retry: z.boolean().exactOptional(), ...abortOptions }),
	]);

const OcCall = () =>
	z.discriminatedUnion("kind", [
		z.object({
			kind: z.literal("views"),
			use: ModelUse.exactOptional(),
			zdrOnly: z.boolean().exactOptional(),
			...abortOptions,
		}),
		z.object({ kind: z.literal("viewFor"), row: z.string(), use: ModelUse.exactOptional(), ...abortOptions }),
		z.object({ kind: z.literal("models"), fresh: z.boolean().exactOptional(), ...abortOptions }),
	]);

/** A step of a script: calls, time, or a change of the registry's rows. */
const Step = <Call extends z.ZodType>(call: () => Call) =>
	z.discriminatedUnion("kind", [
		/** One call. */
		z.object({ kind: z.literal("call"), call: call() }),
		/** Calls at once, in this order. */
		z.object({ kind: z.literal("together"), calls: z.array(call()).min(2) }),
		z.object({ kind: z.literal("advance"), ms: z.int().min(0) }),
		/** The registry's rows change, by key, and it tells. */
		z.object({ kind: z.literal("rows"), rows: z.array(z.string()) }),
	]);

const OrAnswer = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("views"), views: z.array(ViewLine) }),
		z.object({ kind: z.literal("view"), view: ViewLine.nullable() }),
		z.object({
			kind: z.literal("entry"),
			entry: z
				.object({ id: z.string(), name: z.string().nullable(), efforts: z.array(z.string()).nullable() })
				.nullable(),
		}),
		z.object({
			kind: z.literal("listing"),
			listing: z.object({ raw: z.int(), endpoints: z.array(z.string()) }).nullable(),
		}),
		z.object({ kind: z.literal("thrown"), message: z.string() }),
	]);

const OcAnswer = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("views"), views: z.array(ViewLine) }),
		z.object({ kind: z.literal("view"), view: ViewLine }),
		z.object({ kind: z.literal("models"), models: z.array(z.string()).nullable() }),
		z.object({ kind: z.literal("thrown"), message: z.string() }),
	]);

const StepOutcome = <Answer extends z.ZodType>(answer: () => Answer) =>
	z.object({
		/** The wheel's time when the step ended. */
		nowMs: z.int(),
		answers: z.array(answer()),
		/** Times the registry's rows were read during the step. */
		rowReads: z.int(),
	});

type OrCallOf = z.infer<ReturnType<typeof OrCall>>;
type OrAnswerOf = z.infer<ReturnType<typeof OrAnswer>>;
type OcCallOf = z.infer<ReturnType<typeof OcCall>>;
type OcAnswerOf = z.infer<ReturnType<typeof OcAnswer>>;

interface StepOutcomeOf<Answer> {
	nowMs: number;
	answers: Answer[];
	rowReads: number;
}

////////////////////////////////
//  Functions & Helpers

const plain = (value: unknown): unknown => value ?? null;

function pricingResult(pricing: Pricing | null) {
	if (pricing === null) return null;
	return {
		prompt: plain(pricing.prompt),
		completion: plain(pricing.completion),
		input_cache_read: plain(pricing.input_cache_read),
		input_cache_write: plain(pricing.input_cache_write),
		input_cache_write_1h: plain(pricing.input_cache_write_1h),
		overrides:
			pricing.overrides?.map((override) => ({
				min_prompt_tokens: override.min_prompt_tokens,
				prompt: plain(override.prompt),
				completion: plain(override.completion),
				input_cache_read: plain(override.input_cache_read),
				input_cache_write: plain(override.input_cache_write),
				input_cache_write_1h: plain(override.input_cache_write_1h),
			})) ?? null,
	};
}

const modelResult = (entry: ModelEntry) => ({ ...entry, pricing: pricingResult(entry.pricing) });

const endpointResult = (endpoint: Endpoint) => ({ ...endpoint, pricing: pricingResult(endpoint.pricing) });

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : jsString(error));

////////////////////////////////
//  Group

export const catalog = caseGroup("catalog", {
	shapes: { NetExchange, ViewLine },
	kinds: {
		/** Null: the body holds no list. A later model of an id replaces the earlier in its place. */
		openrouterModels: kind(
			z.object({ body: z.unknown() }),
			z.object({ models: z.array(OrModel()).nullable() }),
			(input) => {
				const parsed = parseModels(input.body);
				return { models: parsed === undefined ? null : [...parsed.values()].map(modelResult) };
			},
		),
		/** Hosts by model id, in the order the ids first came. */
		openrouterEndpoints: kind(
			z.object({ body: z.unknown() }),
			z.object({
				endpoints: z.array(z.object({ model: z.string(), hosts: z.array(OrEndpoint()) })).nullable(),
			}),
			(input) => {
				const parsed = parseEndpoints(input.body);
				return {
					endpoints:
						parsed === undefined
							? null
							: [...parsed.entries()].map(([model, hosts]) => ({
									model,
									hosts: hosts.map(endpointResult),
								})),
				};
			},
		),
		opencodeModels: kind(
			z.object({ body: z.unknown() }),
			z.object({ models: z.array(OcModel()).nullable() }),
			(input) => {
				const parsed = parseOpenCode(input.body);
				return { models: parsed === undefined ? null : [...parsed.values()] };
			},
		),
		/** Rows, by key, joined to the two bodies once per pass. */
		openrouterJoin: kind(
			z.object({
				rows: z.array(z.string()),
				models: z.unknown(),
				zdr: z.unknown(),
				nowMs: z.int(),
				passes: z.array(Pass()),
			}),
			z.object({ passes: z.array(z.array(ViewLine)) }),
			(input) => {
				const rows = rowsOfKeys(input.rows);
				return {
					passes: input.passes.map((pass) =>
						joinCatalog(rows, input.models, input.zdr, input.nowMs, pass.use, pass.zdrOnly).map(viewLine),
					),
				};
			},
		),
		/** `models` that holds no list is a plan list that never loaded. */
		opencodeJoin: kind(
			z.object({
				rows: z.array(z.string()),
				plan: ProviderPlan,
				models: z.unknown(),
				nowMs: z.int(),
				passes: z.array(Pass()),
			}),
			z.object({ passes: z.array(z.array(ViewLine)) }),
			(input) => {
				const rows = rowsOfKeys(input.rows);
				const models = parseOpenCode(input.models);
				return {
					passes: input.passes.map((pass) =>
						joinOpenCode(rows, models, input.plan, pass.use, pass.zdrOnly, input.nowMs).map(viewLine),
					),
				};
			},
		),
		/** USD per million tokens of a listed price; null for none. */
		perMillion: kind(
			z.object({ values: z.array(z.unknown()) }),
			z.object({ prices: z.array(z.number().nullable()) }),
			(input) => ({ prices: input.values.map((value) => perMillion(value) ?? null) }),
		),
		/** A removal date in milliseconds; null when it is read as no date. */
		expiry: kind(
			z.object({ texts: z.array(z.string()) }),
			z.object({ ms: z.array(z.int().nullable()) }),
			(input) => ({
				ms: input.texts.map((text) => {
					const at = expiryOf(text);
					return Number.isNaN(at) ? null : at;
				}),
			}),
		),
		/** Whether a model may serve, and which of its hosts count. */
		standing: kind(
			z.object({
				rows: z.array(
					z.object({
						expiration: z.string().nullable(),
						hosts: z.array(z.object({ params: z.array(z.string()).nullable() })),
						/** A text is one parameter; a list is any of its parameters. */
						needs: z.array(z.union([z.string(), z.array(z.string())])),
						nowMs: z.int(),
					}),
				),
			}),
			z.object({
				rows: z.array(
					z.object({
						standing: z.enum(["ok", "retired", "noZdrHost"]),
						/** The places in `hosts` that count. */
						usable: z.array(z.int()),
					}),
				),
			}),
			(input) => ({
				rows: input.rows.map((row) => {
					const hosts: Endpoint[] = row.hosts.map((host, at) => ({
						model_id: "m",
						tag: `h${at}`,
						provider_name: null,
						context_length: null,
						max_prompt_tokens: null,
						supported_parameters: host.params,
						pricing: null,
					}));
					const held = chatStanding(row.expiration, hosts, row.needs, row.nowMs);
					return { standing: held.standing, usable: held.hosts.map((host) => hosts.indexOf(host)) };
				}),
			}),
		),
		/** A catalog on a scripted network and the timer wheel. */
		openrouterRun: kind(
			z.object({
				/** The wheel's first instant. */
				startMs: z.int(),
				/** Replies, in the order the catalog asks. */
				exchanges: z.array(NetExchange),
				/** The registry's first rows, by key. */
				rows: z.array(z.string()),
				ttlMs: z.int().exactOptional(),
				steps: z.array(Step(OrCall)).min(1),
			}),
			z.object({
				steps: z.array(StepOutcome(OrAnswer)),
				/** Paths asked, in order. */
				requests: z.array(z.string()),
				/** Chunks handed out, per exchange. */
				read: z.array(z.int()),
				/** Timers still waiting at the end. */
				waiting: z.array(z.string()),
				/** Times the catalog said it refreshed. */
				refreshed: z.int(),
			}),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges);
				const registry = scriptedModelRegistry(rowsOfKeys(input.rows));
				let refreshed = 0;
				const catalogue = new ModelCatalog({
					transport: net.transport,
					scheduler: net.scheduler,
					rows: registry,
					refreshed: () => {
						refreshed += 1;
					},
					...(input.ttlMs === undefined ? {} : { ttlMs: input.ttlMs }),
				});
				let tags = 0;
				const answerOf = async (call: OrCallOf): Promise<OrAnswerOf> => {
					tags += 1;
					const signal = callerSignal(net.wheel, `caller-${tags}`, call);
					try {
						if (call.kind === "views")
							return {
								kind: "views",
								views: (await catalogue.views(call.use, call.zdrOnly, signal)).map(viewLine),
							};
						if (call.kind === "view") {
							const view = await catalogue.view(call.key, call.use, call.zdrOnly, signal);
							return { kind: "view", view: view === undefined ? null : viewLine(view) };
						}
						if (call.kind === "viewFor")
							return {
								kind: "view",
								view: viewLine(await catalogue.viewFor(rowOfKey(call.row), call.use, signal)),
							};
						if (call.kind === "entry") {
							const entry = await catalogue.entry(call.id, signal);
							return {
								kind: "entry",
								entry:
									entry === undefined
										? null
										: {
												id: entry.id,
												name: entry.name,
												efforts: entry.reasoning?.supported_efforts ?? null,
											},
							};
						}
						const listing = await catalogue.listing(signal, call.retry);
						return {
							kind: "listing",
							listing:
								listing === undefined
									? null
									: { raw: listing.raw.length, endpoints: [...listing.endpoints.keys()] },
						};
					} catch (error) {
						return { kind: "thrown", message: messageOf(error) };
					}
				};
				const steps: StepOutcomeOf<OrAnswerOf>[] = [];
				for (const step of input.steps) {
					const reads = registry.reads();
					let answers: OrAnswerOf[] = [];
					if (step.kind === "call") answers = [await answerOf(step.call)];
					else if (step.kind === "together") answers = await Promise.all(step.calls.map(answerOf));
					else if (step.kind === "advance") net.wheel.advance(step.ms);
					else registry.setRows(rowsOfKeys(step.rows));
					steps.push({ nowMs: net.wheel.nowMs(), answers, rowReads: registry.reads() - reads });
				}
				for (let tag = 1; tag <= tags; tag++) net.wheel.cancel(`caller-${tag}`);
				catalogue.dispose();
				return {
					steps,
					requests: net.asked(),
					read: net.played.read(),
					waiting: net.wheel.waiting(),
					refreshed,
				};
			},
		),
		opencodeRun: kind(
			z.object({
				startMs: z.int(),
				exchanges: z.array(NetExchange),
				rows: z.array(z.string()),
				plan: ProviderPlan,
				ttlMs: z.int().exactOptional(),
				steps: z.array(Step(OcCall)).min(1),
			}),
			z.object({
				steps: z.array(StepOutcome(OcAnswer)),
				requests: z.array(z.string()),
				read: z.array(z.int()),
				waiting: z.array(z.string()),
				refreshed: z.int(),
			}),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges);
				const registry = scriptedModelRegistry(rowsOfKeys(input.rows));
				let refreshed = 0;
				const catalogue = new OpenCodeCatalog({
					plan: input.plan,
					transport: net.transport,
					scheduler: net.scheduler,
					rows: registry,
					refreshed: () => {
						refreshed += 1;
					},
					...(input.ttlMs === undefined ? {} : { ttlMs: input.ttlMs }),
				});
				let tags = 0;
				const answerOf = async (call: OcCallOf): Promise<OcAnswerOf> => {
					tags += 1;
					const signal = callerSignal(net.wheel, `caller-${tags}`, call);
					try {
						if (call.kind === "views")
							return {
								kind: "views",
								views: (await catalogue.views(call.use, call.zdrOnly, signal)).map(viewLine),
							};
						if (call.kind === "viewFor")
							return {
								kind: "view",
								view: viewLine(await catalogue.viewFor(rowOfKey(call.row), call.use, signal)),
							};
						const models = await catalogue.models(call.fresh, signal);
						return { kind: "models", models: models === undefined ? null : [...models.keys()] };
					} catch (error) {
						return { kind: "thrown", message: messageOf(error) };
					}
				};
				const steps: StepOutcomeOf<OcAnswerOf>[] = [];
				for (const step of input.steps) {
					const reads = registry.reads();
					let answers: OcAnswerOf[] = [];
					if (step.kind === "call") answers = [await answerOf(step.call)];
					else if (step.kind === "together") answers = await Promise.all(step.calls.map(answerOf));
					else if (step.kind === "advance") net.wheel.advance(step.ms);
					else registry.setRows(rowsOfKeys(step.rows));
					steps.push({ nowMs: net.wheel.nowMs(), answers, rowReads: registry.reads() - reads });
				}
				for (let tag = 1; tag <= tags; tag++) net.wheel.cancel(`caller-${tag}`);
				return {
					steps,
					requests: net.asked(),
					read: net.played.read(),
					waiting: net.wheel.waiting(),
					refreshed,
				};
			},
		),
	},
	sources: casesFor("catalog", catalogDecodeCases, catalogOpenCodeCases, catalogJoinCases, catalogRunCases),
});
