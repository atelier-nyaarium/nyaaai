import { failureOf } from "@nyaaai/agentic/failure";
import {
	listed as compatibleListed,
	modelOf as compatibleModel,
	remote,
	termsOf,
} from "@nyaaai/agentic/kinds/compatible";
import { joinAll } from "@nyaaai/agentic/kinds/join";
import { listed as llamacppListed, modelOf as llamacppModel, propsOfBody } from "@nyaaai/agentic/kinds/llamacpp";
import { chatModels, modelOf as lmstudioModel, loadedContexts } from "@nyaaai/agentic/kinds/lmstudio";
import {
	contextOf,
	effortsOf,
	LocalCatalog,
	type LocalModel,
	type LocalServer,
	labelOf,
	localView,
} from "@nyaaai/agentic/kinds/local/catalog";
import { localTags, numCtxOf, modelOf as ollamaModel, runningOf } from "@nyaaai/agentic/kinds/ollama";
import { MODELS } from "@nyaaai/agentic/models/curated";
import { CHAT } from "@nyaaai/agentic/models/uses";
import { codecFactsOf } from "@nyaaai/agentic/models/view";
import type { ProviderKind } from "@nyaaai/agentic/provider/kind";
import { replyText } from "@nyaaai/agentic/step";
import { jsString } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { BrowseDetail, BrowseModel, ModelUse, type ModelView } from "@nyaaai/contract/shapes/model";
import { Disposition, KeyState } from "@nyaaai/contract/shapes/provider";
import { ApiFacts, CanonicalRequest, Effort, Encoded, Failure } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedModelRegistry } from "@nyaaai/testkit/scriptedPorts";
import { localCatalogCases } from "../caseSources/localCatalog";
import { localDecodeCases } from "../caseSources/localDecode";
import { localFlowCases } from "../caseSources/localFlow";
import { localKindCases } from "../caseSources/localKind";
import { localReachCases } from "../caseSources/localReach";
import { withUses } from "../caseSources/uses";
import { abortOptions, callerSignal, NetExchange, ViewLine, viewLine } from "./catalogKit";
import {
	LOCAL_KINDS,
	LoadedContext,
	LocalEntry,
	LocalKindName,
	localRowOf,
	localRowsOf,
	networkOf,
	requestLine,
	ServedModel,
	ServedTerms,
	serverOf,
	transportOf,
	urlOf,
} from "./localKit";

////////////////////////////////
//  Shapes

const RequestLine = () =>
	z.object({
		method: z.string(),
		url: z.string(),
		authorization: z.string().nullable(),
		body: z.unknown(),
	});

/** What a view reads of its model for a codec. */
const CodecView = () =>
	z.object({
		efforts: z.array(Effort),
		api: ApiFacts.exactOptional(),
		outputLimit: z.number().exactOptional(),
		price: z.enum(["free", "unknown"]).exactOptional(),
	});

/** One call on a server by itself. */
const ServerStep = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("models"), known: z.array(ServedModel), ...abortOptions }),
		z.object({ kind: z.literal("loaded"), tag: z.string(), ...abortOptions }),
		z.object({ kind: z.literal("warm"), tag: z.string(), ...abortOptions }),
	]);

const Discovered = () =>
	z.object({
		key: z.string(),
		display: z.string(),
		tag: z.string(),
		tools: z.boolean(),
		reasoning: z.boolean(),
		context: z.number().exactOptional(),
	});

/** A scripted server reply after `delayMs`: answers or fails. */
const ModelsReply = () =>
	z.object({
		models: z.array(ServedModel),
		delayMs: z.int().exactOptional(),
		fails: z.literal(true).exactOptional(),
	});

const LoadedReply = () =>
	z.object({
		contexts: z.array(LoadedContext),
		delayMs: z.int().exactOptional(),
		fails: z.literal(true).exactOptional(),
	});

const WarmReply = () => z.object({ delayMs: z.int().exactOptional(), fails: z.literal(true).exactOptional() });

/** One call the catalog made on its server. */
const ServerCall = () =>
	z.object({
		method: z.enum(["models", "loaded", "warm"]),
		/** The tag asked about; null for a list. */
		tag: z.string().nullable(),
		/** The tags the catalog already knew, for a list. */
		known: z.array(z.string()),
		startMs: z.int(),
		endMs: z.int(),
		/** The signal the server was given had aborted when it ended. */
		aborted: z.boolean(),
	});

const CatalogCall = () =>
	z.discriminatedUnion("kind", [
		z.object({
			kind: z.literal("views"),
			use: ModelUse.exactOptional(),
			zdrOnly: z.boolean().exactOptional(),
			...abortOptions,
		}),
		z.object({ kind: z.literal("discovered"), fresh: z.boolean().exactOptional(), ...abortOptions }),
		z.object({ kind: z.literal("viewFor"), row: z.string(), use: ModelUse.exactOptional(), ...abortOptions }),
		z.object({ kind: z.literal("warmed"), tag: z.string(), ...abortOptions }),
	]);

const CatalogStep = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("call"), call: CatalogCall() }),
		z.object({ kind: z.literal("together"), calls: z.array(CatalogCall()).min(2) }),
		/** A reply was seen for this tag. */
		z.object({ kind: z.literal("settled"), tag: z.string(), times: z.int().min(1).exactOptional() }),
		z.object({ kind: z.literal("rediscover") }),
		z.object({ kind: z.literal("advance"), ms: z.int().min(0) }),
		z.object({ kind: z.literal("rows"), rows: z.array(z.string()) }),
	]);

const CatalogAnswer = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("views"), views: z.array(ViewLine) }),
		z.object({ kind: z.literal("discovered"), models: z.array(Discovered()).nullable() }),
		z.object({ kind: z.literal("view"), view: ViewLine }),
		z.object({ kind: z.literal("warmed") }),
		z.object({ kind: z.literal("thrown"), message: z.string() }),
	]);

const FlowCall = () =>
	z.discriminatedUnion("kind", [
		z.object({
			kind: z.literal("views"),
			use: ModelUse.exactOptional(),
			zdrOnly: z.boolean().exactOptional(),
			...abortOptions,
		}),
		/** A chat through the transport, on the model's tag. */
		z.object({ kind: z.literal("chat"), model: z.string(), ...abortOptions }),
		z.object({ kind: z.literal("list"), fresh: z.boolean().exactOptional() }),
		z.object({ kind: z.literal("detail"), id: z.string(), ...abortOptions }),
		z.object({ kind: z.literal("listed"), id: z.string(), ...abortOptions }),
		z.object({ kind: z.literal("view"), row: z.string(), use: ModelUse.exactOptional(), ...abortOptions }),
		z.object({ kind: z.literal("advance"), ms: z.int().min(0) }),
		z.object({ kind: z.literal("rows"), rows: z.array(z.string()) }),
	]);

const FlowAnswer = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("views"), views: z.array(ViewLine) }),
		z.object({ kind: z.literal("chat"), text: z.string() }),
		z.object({ kind: z.literal("list"), models: z.array(BrowseModel).nullable() }),
		z.object({ kind: z.literal("detail"), detail: BrowseDetail.nullable() }),
		z.object({
			kind: z.literal("listed"),
			listed: z.object({ label: z.string(), reasoning: z.boolean() }).nullable(),
		}),
		z.object({ kind: z.literal("view"), view: ViewLine }),
		z.object({ kind: z.literal("none") }),
		z.object({ kind: z.literal("thrown"), message: z.string(), category: z.string().nullable() }),
	]);

type CatalogCallOf = z.infer<ReturnType<typeof CatalogCall>>;
type CatalogAnswerOf = z.infer<ReturnType<typeof CatalogAnswer>>;
type ServerCallOf = z.infer<ReturnType<typeof ServerCall>>;
type FlowCallOf = z.infer<ReturnType<typeof FlowCall>>;
type FlowAnswerOf = z.infer<ReturnType<typeof FlowAnswer>>;

////////////////////////////////
//  Functions & Helpers

export const messageOf = (error: unknown): string => (error instanceof Error ? error.message : jsString(error));

/** The Settings line; none when the kind cannot read the entry's URL. */
export function factsOf(
	named: ProviderKind,
	state: z.infer<typeof KeyState>,
	entry: z.infer<typeof LocalEntry> & { kind: LocalKindName },
): string | null {
	try {
		return named.facts?.(state, entry, undefined) ?? null;
	} catch {
		return null;
	}
}

export const contextsOf = (contexts: ReadonlyMap<string, number>) =>
	[...contexts].map(([tag, context]) => ({ tag, context }));

/** A made-up row's view, per the codec's facts. */
export function viewOfFacts(held: z.infer<ReturnType<typeof CodecView>> | null): ModelView | undefined {
	if (held === null) return undefined;
	const row = MODELS[0];
	if (row === undefined) throw new Error("The curated list is empty.");
	return {
		row,
		status: "ok",
		efforts: held.efforts,
		hosts: [],
		...(held.api === undefined ? {} : { api: held.api }),
		...(held.outputLimit === undefined ? {} : { outputLimit: held.outputLimit }),
		...(held.price === undefined ? {} : { price: held.price }),
	};
}

/** A server that answers from a script, spending the wheel's time. */
export function scriptedServer(
	wheel: TimerWheel,
	script: {
		live: boolean;
		loaded: boolean;
		warm: boolean;
		models: ReadonlyArray<z.infer<ReturnType<typeof ModelsReply>>>;
		loads: ReadonlyArray<z.infer<ReturnType<typeof LoadedReply>>>;
		warms: ReadonlyArray<z.infer<ReturnType<typeof WarmReply>>>;
	},
) {
	const calls: ServerCallOf[] = [];
	const taken = { models: 0, loaded: 0, warm: 0 };
	async function answer<T extends { delayMs?: number | undefined; fails?: true | undefined }>(
		method: ServerCallOf["method"],
		replies: readonly T[],
		tag: string | null,
		known: readonly string[],
		signal: AbortSignal,
	): Promise<T> {
		const reply = replies[taken[method]];
		taken[method] += 1;
		const call: ServerCallOf = { method, tag, known: [...known], startMs: wheel.nowMs(), endMs: 0, aborted: false };
		calls.push(call);
		try {
			if (reply === undefined) throw new Error("the script holds no further reply");
			wheel.advance(reply.delayMs ?? 0);
			if (signal.aborted) throw signal.reason;
			if (reply.fails === true) throw new Error("the server failed");
			return reply;
		} finally {
			call.endMs = wheel.nowMs();
			call.aborted = signal.aborted;
		}
	}
	const server: LocalServer = {
		...(script.live ? { live: true as const } : {}),
		models: async (known, signal) =>
			(
				await answer(
					"models",
					script.models,
					null,
					known.map((model) => model.tag),
					signal,
				)
			).models.map((model) => ({ ...model })),
		...(script.loaded
			? {
					loaded: async (tag: string, signal: AbortSignal) => {
						const reply = await answer("loaded", script.loads, tag, [], signal);
						return new Map(reply.contexts.map((each) => [each.tag, each.context] as const));
					},
				}
			: {}),
		...(script.warm
			? {
					warm: async (tag: string, signal: AbortSignal) => {
						await answer("warm", script.warms, tag, [], signal);
					},
				}
			: {}),
	};
	return { server, calls };
}

export const CHAT_REQUEST = (model: string): CanonicalRequest => ({
	model,
	system: ["Rules."],
	history: [{ role: "user", text: "Hi" }],
	cache: { kind: "implicit" },
	session: "root-1",
	zdrOnly: true,
});

////////////////////////////////
//  Group

export const local = caseGroup("local", {
	shapes: { NetExchange, ViewLine, ServedModel, ServedTerms, LocalEntry },
	kinds: {
		/** The efforts a server's thinking values offer. */
		efforts: kind(
			z.object({ values: z.array(z.union([z.boolean(), z.string()])) }),
			z.object({ efforts: z.array(Effort) }),
			(input) => ({ efforts: effortsOf(input.values) }),
		),
		/** The label a tag gives a record. */
		label: kind(z.object({ tags: z.array(z.string()) }), z.object({ labels: z.array(z.string()) }), (input) => ({
			labels: input.tags.map(labelOf),
		})),
		/** A model's view for a local row under the entry's terms. */
		view: kind(
			z.object({
				row: z.string(),
				model: ServedModel,
				loaded: z.number().exactOptional(),
				terms: ServedTerms,
				zdrOnly: z.boolean(),
			}),
			z.object({ view: ViewLine, context: z.number().nullable() }),
			(input) => ({
				view: viewLine(
					localView(localRowOf(input.row), input.model, input.loaded, {
						terms: input.terms,
						zdrOnly: input.zdrOnly,
					}),
				),
				context: contextOf(input.model, input.loaded) ?? null,
			}),
		),
		/** Ollama's tag list: local names only. */
		ollamaTags: kind(
			z.object({ body: z.unknown() }),
			z.object({ tags: z.array(z.string()).nullable() }),
			(input) => ({ tags: localTags(input.body) ?? null }),
		),
		/** One show answer: a completion model, or none. */
		ollamaShow: kind(
			z.object({ tag: z.string(), body: z.unknown() }),
			z.object({ model: ServedModel.nullable() }),
			(input) => ({ model: ollamaModel(input.tag, input.body) ?? null }),
		),
		/** The context the first `num_ctx` line of the parameters text names. */
		ollamaNumCtx: kind(
			z.object({ texts: z.array(z.string()) }),
			z.object({ contexts: z.array(z.number().nullable()) }),
			(input) => ({ contexts: input.texts.map((text) => numCtxOf(text) ?? null) }),
		),
		/** The loaded contexts of the running list, by name. */
		ollamaRunning: kind(
			z.object({ body: z.unknown() }),
			z.object({ contexts: z.array(LoadedContext) }),
			(input) => ({ contexts: contextsOf(runningOf(input.body)) }),
		),
		/** The chat models of LM Studio's list. */
		lmstudioModels: kind(
			z.object({ body: z.unknown() }),
			z.object({ models: z.array(ServedModel).nullable() }),
			(input) => ({ models: chatModels(input.body)?.map(lmstudioModel) ?? null }),
		),
		/** The same list's loaded contexts by key; none when unread. */
		lmstudioLoaded: kind(
			z.object({ body: z.unknown() }),
			z.object({ contexts: z.array(LoadedContext) }),
			(input) => ({ contexts: contextsOf(loadedContexts(chatModels(input.body) ?? [])) }),
		),
		/** The models of llama.cpp's list. */
		llamacppModels: kind(
			z.object({ body: z.unknown() }),
			z.object({ models: z.array(z.object({ id: z.string(), trained: z.number().nullable() })).nullable() }),
			(input) => ({ models: llamacppListed(input.body) ?? null }),
		),
		/** What a props body says. */
		llamacppProps: kind(
			z.object({ body: z.unknown() }),
			z.object({ props: z.object({ context: z.number().nullable(), efforts: z.array(z.string()) }).nullable() }),
			(input) => {
				const props = propsOfBody(input.body);
				return {
					props: props === undefined ? null : { context: props.context ?? null, efforts: [...props.efforts] },
				};
			},
		),
		/** A listed model with its props and catalog knowledge. */
		llamacppModel: kind(
			z.object({
				id: z.string(),
				trained: z.number().exactOptional(),
				props: z.object({ context: z.number().exactOptional(), efforts: z.array(z.string()) }).exactOptional(),
				known: ServedModel.exactOptional(),
			}),
			z.object({ model: ServedModel }),
			(input) => ({
				model: llamacppModel(input.id, input.trained, input.props, input.known),
			}),
		),
		/** The models of an OpenAI-compatible list. */
		compatibleModels: kind(
			z.object({ body: z.unknown() }),
			z.object({ models: z.array(ServedModel).nullable() }),
			(input) => ({ models: compatibleListed(input.body)?.map(compatibleModel) ?? null }),
		),
		/** Whether a URL's host is off this network. */
		compatibleTerms: kind(
			z.object({ urls: z.array(z.string().nullable()) }),
			z.object({
				terms: z.array(
					z.object({ remote: z.boolean(), kept: z.boolean(), price: z.enum(["free", "unknown"]) }),
				),
			}),
			(input) => ({
				terms: input.urls.map((url) => {
					const entry = { kind: "compatible" as const, label: "Host", ...(url === null ? {} : { url }) };
					return { remote: remote(entry), ...termsOf(entry) };
				}),
			}),
		),
		/** A kind's server check: state and requests. */
		reach: kind(
			z.object({
				kind: LocalKindName,
				entry: LocalEntry,
				/** The key given to the check; null is none. */
				secret: z.string().nullable(),
				startMs: z.int(),
				exchanges: z.array(NetExchange),
				...abortOptions,
			}),
			z.object({
				state: KeyState.nullable(),
				thrown: z.string().nullable(),
				/** The Settings line after the check. */
				facts: z.string().nullable(),
				requests: z.array(RequestLine()),
				waiting: z.array(z.string()),
			}),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges);
				const named = LOCAL_KINDS[input.kind];
				const entry = { kind: input.kind, label: input.entry.label, ...urlOf(input.entry) };
				const signal = callerSignal(net.wheel, "caller", input);
				let state: z.infer<typeof KeyState> | null = null;
				let thrown: string | null = null;
				try {
					state = await named.check(entry, input.secret ?? undefined, net.deps, signal);
				} catch (error) {
					thrown = messageOf(error);
				}
				net.wheel.cancel("caller");
				return {
					state,
					thrown,
					facts: state === null ? null : factsOf(named, state, entry),
					requests: net.played.requests.map(requestLine),
					waiting: net.wheel.waiting(),
				};
			},
		),
		/** A kind's server over a scripted network. */
		server: kind(
			z.object({
				kind: LocalKindName,
				entry: LocalEntry,
				/** The stored key; null is none. */
				stored: z.string().nullable(),
				startMs: z.int(),
				exchanges: z.array(NetExchange),
				steps: z.array(ServerStep()).min(1),
			}),
			z.object({
				steps: z.array(
					z.object({
						models: z.array(ServedModel).nullable(),
						contexts: z.array(LoadedContext).nullable(),
						done: z.boolean(),
						thrown: z.string().nullable(),
					}),
				),
				requests: z.array(RequestLine()),
				/** Chunks handed out, per exchange. */
				read: z.array(z.int()),
				waiting: z.array(z.string()),
			}),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges);
				const server = serverOf(input.kind, transportOf(input.kind, input.entry, net.deps, input.stored));
				const steps: Array<{
					models: LocalModel[] | null;
					contexts: Array<{ tag: string; context: number }> | null;
					done: boolean;
					thrown: string | null;
				}> = [];
				let tags = 0;
				for (const step of input.steps) {
					tags += 1;
					const signal = callerSignal(net.wheel, `caller-${tags}`, step);
					const held = {
						models: null as LocalModel[] | null,
						contexts: null as ReturnType<typeof contextsOf> | null,
						done: false,
						thrown: null as string | null,
					};
					try {
						if (step.kind === "models") held.models = await server.models(step.known, signal);
						else if (step.kind === "loaded") {
							if (server.loaded === undefined) throw new Error("This server reads no loaded contexts.");
							held.contexts = contextsOf(await server.loaded(step.tag, signal));
						} else {
							if (server.warm === undefined) throw new Error("This server loads no model.");
							await server.warm(step.tag, signal);
							held.done = true;
						}
					} catch (error) {
						held.thrown = messageOf(error);
					}
					steps.push(held);
				}
				for (let tag = 1; tag <= tags; tag++) net.wheel.cancel(`caller-${tag}`);
				return {
					steps,
					requests: net.played.requests.map(requestLine),
					read: net.played.read(),
					waiting: net.wheel.waiting(),
				};
			},
		),
		/** A `LocalCatalog` over a scripted server on the wheel. */
		catalogRun: kind(
			z.object({
				startMs: z.int(),
				/** The entry the catalog discovers for. */
				entry: z.string(),
				terms: ServedTerms,
				server: z.object({
					live: z.boolean(),
					/** The server can read a loaded context. */
					loaded: z.boolean(),
					/** The server can load a model. */
					warm: z.boolean(),
					models: z.array(ModelsReply()),
					loads: z.array(LoadedReply()),
					warms: z.array(WarmReply()),
				}),
				/** Row keys the registry holds. */
				rows: z.array(z.string()),
				/** What the `refreshed` callback does: reads the catalog, and may throw after. */
				onRefreshed: z.enum(["reads", "readsAndThrows"]).exactOptional(),
				steps: z.array(CatalogStep()).min(1),
			}),
			z.object({
				steps: z.array(
					z.object({
						nowMs: z.int(),
						answers: z.array(CatalogAnswer()),
						/** What the server was asked during the step. */
						served: z.array(ServerCall()),
						/** Times the catalog said it refreshed during the step. */
						refreshed: z.int(),
					}),
				),
				waiting: z.array(z.string()),
				/** What each `refreshed` callback read: the tags, or `none`. */
				saw: z.array(z.string()),
			}),
			async (input) => {
				const wheel = new TimerWheel(input.startMs);
				const scheduler = fakeScheduler(wheel);
				const registry = scriptedModelRegistry(localRowsOf(input.rows));
				const { server, calls } = scriptedServer(wheel, {
					live: input.server.live,
					loaded: input.server.loaded,
					warm: input.server.warm,
					models: input.server.models,
					loads: input.server.loads,
					warms: input.server.warms,
				});
				let refreshed = 0;
				const saw: Array<Promise<string>> = [];
				const catalog = new LocalCatalog({
					entry: input.entry,
					server,
					terms: input.terms,
					scheduler,
					rows: registry,
					refreshed: () => {
						refreshed += 1;
						if (input.onRefreshed === undefined) return;
						saw.push(
							catalog
								.discovered()
								.then((held) =>
									held === undefined ? "none" : held.map((model) => model.tag).join(" "),
								),
						);
						if (input.onRefreshed === "readsAndThrows") throw new Error("told badly");
					},
				});
				let tags = 0;
				const answerOf = async (call: CatalogCallOf): Promise<CatalogAnswerOf> => {
					tags += 1;
					const signal = callerSignal(wheel, `caller-${tags}`, call);
					try {
						if (call.kind === "views")
							return {
								kind: "views",
								views: (await catalog.views(call.use, call.zdrOnly, signal)).map(viewLine),
							};
						if (call.kind === "discovered")
							return {
								kind: "discovered",
								models: (await catalog.discovered(call.fresh, signal)) ?? null,
							};
						if (call.kind === "viewFor")
							return {
								kind: "view",
								view: viewLine(await catalog.viewFor(localRowOf(call.row), call.use, signal)),
							};
						await catalog.warmed(call.tag, signal);
						return { kind: "warmed" };
					} catch (error) {
						return { kind: "thrown", message: messageOf(error) };
					}
				};
				const steps: Array<{
					nowMs: number;
					answers: CatalogAnswerOf[];
					served: ServerCallOf[];
					refreshed: number;
				}> = [];
				for (const step of input.steps) {
					const served = calls.length;
					const before = refreshed;
					let answers: CatalogAnswerOf[] = [];
					if (step.kind === "call") answers = [await answerOf(step.call)];
					else if (step.kind === "together")
						answers = await joinAll(step.calls.map((held) => () => answerOf(held)));
					else if (step.kind === "settled")
						for (let each = 0; each < (step.times ?? 1); each++) catalog.settled(step.tag);
					else if (step.kind === "rediscover") catalog.rediscover();
					else if (step.kind === "advance") wheel.advance(step.ms);
					else registry.setRows(localRowsOf(step.rows));
					await catalog.idle();
					steps.push({
						nowMs: wheel.nowMs(),
						answers,
						served: calls.slice(served).map((call) => ({ ...call })),
						refreshed: refreshed - before,
					});
				}
				for (let tag = 1; tag <= tags; tag++) wheel.cancel(`caller-${tag}`);
				return { steps, waiting: wheel.waiting(), saw: await joinAll(saw.map((held) => () => held)) };
			},
		),
		/** A kind's transport, catalog, browser and tester over a scripted network. */
		flow: kind(
			z.object({
				kind: LocalKindName,
				entry: z.object({ id: z.string(), label: z.string(), url: z.string().exactOptional() }),
				/** The stored key; null is none. */
				stored: z.string().nullable(),
				startMs: z.int(),
				exchanges: z.array(NetExchange),
				/** Row keys the registry holds. */
				rows: z.array(z.string()),
				steps: z.array(FlowCall()).min(1),
			}),
			z.object({
				steps: z.array(z.object({ nowMs: z.int(), answer: FlowAnswer() })),
				requests: z.array(RequestLine()),
				/** Chunks handed out, per exchange. */
				read: z.array(z.int()),
				waiting: z.array(z.string()),
				/** Times the catalog said it refreshed. */
				refreshed: z.int(),
			}),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges);
				const named = LOCAL_KINDS[input.kind];
				const entry = { id: input.entry.id, kind: input.kind, label: input.entry.label, ...urlOf(input.entry) };
				const registry = scriptedModelRegistry(localRowsOf(input.rows));
				let refreshed = 0;
				const transport = transportOf(input.kind, input.entry, net.deps, input.stored);
				const catalog = named.catalog(entry, transport, {
					scheduler: net.scheduler,
					rows: registry,
					refreshed: () => {
						refreshed += 1;
					},
				});
				const route = { transport, catalog };
				const browser = named.browse?.(entry, route, net.deps);
				const tester = named.tester?.(entry, route, net.deps);
				if (browser === undefined || tester === undefined)
					throw new Error("A local kind has a browser and a tester.");
				let tags = 0;
				const answerOf = async (call: FlowCallOf): Promise<FlowAnswerOf> => {
					tags += 1;
					const signal = callerSignal(
						net.wheel,
						`caller-${tags}`,
						"abortAfterMs" in call || "abortedBefore" in call ? call : {},
					);
					try {
						if (call.kind === "views")
							return {
								kind: "views",
								views: (await catalog.views(call.use ?? CHAT, call.zdrOnly ?? true, signal)).map(
									viewLine,
								),
							};
						if (call.kind === "chat") {
							const codec = named.codec(undefined, entry, net.deps);
							return {
								kind: "chat",
								text: replyText(await transport.complete(codec, CHAT_REQUEST(call.model), signal)),
							};
						}
						if (call.kind === "list")
							return { kind: "list", models: (await browser.list(call.fresh)) ?? null };
						if (call.kind === "detail")
							return { kind: "detail", detail: (await browser.detail(call.id, signal)) ?? null };
						if (call.kind === "listed")
							return { kind: "listed", listed: (await tester.listed(call.id, signal)) ?? null };
						if (call.kind === "view")
							return {
								kind: "view",
								view: viewLine(await tester.view(localRowOf(call.row), call.use ?? CHAT, signal)),
							};
						if (call.kind === "advance") net.wheel.advance(call.ms);
						else registry.setRows(localRowsOf(call.rows));
						return { kind: "none" };
					} catch (error) {
						return {
							kind: "thrown",
							message: messageOf(error),
							category: failureOf(error)?.category ?? null,
						};
					}
				};
				const steps: Array<{ nowMs: number; answer: FlowAnswerOf }> = [];
				for (const step of input.steps) {
					const answer = await answerOf(step);
					await catalog.idle();
					steps.push({ nowMs: net.wheel.nowMs(), answer });
				}
				for (let tag = 1; tag <= tags; tag++) net.wheel.cancel(`caller-${tag}`);
				return {
					steps,
					requests: net.played.requests.map(requestLine),
					read: net.played.read(),
					waiting: net.wheel.waiting(),
					refreshed,
				};
			},
		),
		/** The Settings line of a kind's entry. */
		kindFacts: kind(
			z.object({ kind: LocalKindName, url: z.string().exactOptional(), states: z.array(KeyState) }),
			z.object({ facts: z.array(z.string().nullable()) }),
			(input) => {
				const named = LOCAL_KINDS[input.kind];
				const entry = { kind: input.kind, label: "Lab", ...urlOf(input) };
				return { facts: input.states.map((state) => named.facts?.(state, entry, undefined) ?? null) };
			},
		),
		/** A kind's codec: what it sends, how it reads failures. */
		kindCodec: kind(
			z.object({
				kind: LocalKindName,
				label: z.string(),
				view: CodecView().nullable(),
				request: CanonicalRequest,
				mode: z.enum(["stream", "complete"]),
				failures: z.array(z.object({ status: z.int(), body: z.unknown() })),
			}),
			z.object({
				/** What the codec sends, or why it refuses. */
				encoded: Encoded.nullable(),
				refusal: z.string().nullable(),
				/** The facts the view gives the codec. */
				facts: z
					.object({ efforts: z.array(Effort), free: z.boolean(), outputLimit: z.number().nullable() })
					.nullable(),
				failures: z.array(Failure),
			}),
			(input) => {
				const named = LOCAL_KINDS[input.kind];
				const view = viewOfFacts(input.view);
				const codec = named.codec(view, { kind: input.kind, label: input.label }, networkOf(0, []).deps);
				const facts = codecFactsOf(view);
				let encoded: z.infer<typeof Encoded> | null = null;
				let refusal: string | null = null;
				try {
					encoded = codec.encode(input.request, input.mode);
				} catch (error) {
					if (!(error instanceof Error) || Object.getPrototypeOf(error) !== Error.prototype) throw error;
					refusal = error.message;
				}
				return {
					encoded,
					refusal,
					facts:
						facts === undefined
							? null
							: {
									efforts: facts.efforts,
									free: facts.free === true,
									outputLimit: facts.outputLimit ?? null,
								},
					failures: input.failures.map((each) => codec.failure(each.status, each.body, [])),
				};
			},
		),
		/** What a kind does with each failure, against the view it met. */
		kindDisposition: kind(
			z.object({
				kind: LocalKindName,
				rows: z.array(
					z.object({
						failure: Failure,
						/** The view's status the failure met; null is no view. */
						status: z.enum(["ok", "retired", "noZdrHost", "noTools", "unknown"]).nullable(),
					}),
				),
			}),
			z.object({ dispositions: z.array(Disposition) }),
			(input) => {
				const row = MODELS[0];
				if (row === undefined) throw new Error("The curated list is empty.");
				return {
					dispositions: input.rows.map((each) => {
						const view: ModelView | undefined =
							each.status === null ? undefined : { row, status: each.status, efforts: [], hosts: [] };
						return LOCAL_KINDS[input.kind].failure(each.failure, view);
					}),
				};
			},
		),
	},
	sources: withUses(
		casesFor("local", localDecodeCases, localReachCases, localCatalogCases, localFlowCases, localKindCases),
	),
});
