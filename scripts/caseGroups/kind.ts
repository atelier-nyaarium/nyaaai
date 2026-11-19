import { checkKey } from "@nyaaai/agentic/kinds/openrouter/key";
import { CHAT } from "@nyaaai/agentic/models/uses";
import { codecFactsOf } from "@nyaaai/agentic/models/view";
import { KINDS, kindLines, type ProviderKind, providerView } from "@nyaaai/agentic/provider/kind";
import type { KeyAuth } from "@nyaaai/agentic/transport";
import { jsString } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { BrowseDetail, BrowseModel, ModelUse, type ModelView } from "@nyaaai/contract/shapes/model";
import {
	Billed,
	Disposition,
	KeyState,
	KindLine,
	Limits,
	ProviderEntry,
	ProviderKindName,
	ProviderPlan,
	ProviderView,
} from "@nyaaai/contract/shapes/provider";
import { ApiFacts, CanonicalRequest, Effort, Encoded, Failure } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { scriptedModelRegistry } from "@nyaaai/testkit/scriptedPorts";
import { kindCases } from "../caseSources/kindCases";
import { kindCheckCases } from "../caseSources/kindCheck";
import { kindKeyCases } from "../caseSources/kindKey";
import { kindRouteCases } from "../caseSources/kindRoute";
import { withUses } from "../caseSources/uses";
import {
	abortOptions,
	BASE_URL,
	callerSignal,
	NetExchange,
	networkOf,
	rowsOfKeys,
	ViewLine,
	viewLine,
} from "./catalogKit";
import { keysOf, networkOf as kindNetworkOf } from "./localKit";

////////////////////////////////
//  Shapes

/** The kinds of this group. */
const HostedKindName = z.enum(["openrouter", "opencode"]);
type HostedKindName = z.infer<typeof HostedKindName>;

/** An entry as a hosted kind reads it. */
const HostedEntry = z.object({ label: z.string(), plan: ProviderPlan.exactOptional() });

const RequestLine = () =>
	z.object({
		method: z.string(),
		url: z.string(),
		authorization: z.string().nullable(),
		userAgent: z.string().nullable(),
	});

/** What a view reads of its model for a codec. */
const CodecView = () =>
	z.object({
		efforts: z.array(Effort),
		api: ApiFacts.exactOptional(),
		outputLimit: z.number().exactOptional(),
		price: z.enum(["free", "unknown"]).exactOptional(),
	});

/** A step of a kind's route. */
const RouteStep = () =>
	z.discriminatedUnion("kind", [
		z.object({
			kind: z.literal("views"),
			use: ModelUse.exactOptional(),
			zdrOnly: z.boolean().exactOptional(),
			...abortOptions,
		}),
		z.object({ kind: z.literal("list"), fresh: z.boolean().exactOptional() }),
		z.object({ kind: z.literal("detail"), id: z.string(), ...abortOptions }),
		z.object({ kind: z.literal("listed"), id: z.string(), ...abortOptions }),
		z.object({ kind: z.literal("view"), row: z.string(), use: ModelUse.exactOptional(), ...abortOptions }),
		z.object({ kind: z.literal("advance"), ms: z.int().min(0) }),
	]);

const RouteAnswer = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("views"), views: z.array(ViewLine) }),
		z.object({ kind: z.literal("list"), models: z.array(BrowseModel).nullable() }),
		z.object({ kind: z.literal("detail"), detail: BrowseDetail.nullable() }),
		z.object({
			kind: z.literal("listed"),
			listed: z.object({ label: z.string(), reasoning: z.boolean() }).nullable(),
		}),
		z.object({ kind: z.literal("view"), view: ViewLine }),
		z.object({ kind: z.literal("advanced") }),
		z.object({ kind: z.literal("thrown"), message: z.string() }),
	]);

type RouteAnswerOf = z.infer<ReturnType<typeof RouteAnswer>>;

////////////////////////////////
//  Functions & Helpers

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : jsString(error));

/** A hosted kind of the table. */
function hostedKind(name: HostedKindName): ProviderKind {
	const held = KINDS[name];
	if (held === undefined) throw new Error(`No ${name} kind.`);
	return held;
}

/** The entry of a case. */
const entryOf = (name: HostedKindName, entry: z.infer<typeof HostedEntry>) => ({
	kind: name,
	label: entry.label,
	...(entry.plan === undefined ? {} : { plan: entry.plan }),
});

/** What a request looked like on the wire. */
function requestLine(sent: { method: string; url: string; headers: readonly (readonly [string, string])[] }) {
	const header = (name: string) => sent.headers.find(([each]) => each === name)?.[1] ?? null;
	return {
		method: sent.method,
		url: sent.url,
		authorization: header("Authorization"),
		userAgent: header("User-Agent"),
	};
}

/** The kinds of a table, in the order given. */
function tableOf(names: readonly z.infer<typeof ProviderKindName>[] | undefined) {
	if (names === undefined) return KINDS;
	const table: Partial<Record<z.infer<typeof ProviderKindName>, ProviderKind>> = {};
	for (const name of names) {
		const held = KINDS[name];
		if (held !== undefined) table[name] = held;
	}
	return table;
}

////////////////////////////////
//  Group

export const kindGroup = caseGroup("kind", {
	shapes: { NetExchange, ViewLine },
	kinds: {
		/** `checkKey` over a scripted network; each check reads one reply. */
		openrouterKey: kind(
			z.object({
				startMs: z.int(),
				exchanges: z.array(NetExchange),
				/** The stored key; null is none. */
				stored: z.string().nullable(),
				checks: z.array(
					z.discriminatedUnion("auth", [
						z.object({ auth: z.literal("stored"), ...abortOptions }),
						z.object({ auth: z.literal("key"), key: z.string(), ...abortOptions }),
					]),
				),
			}),
			z.object({
				checks: z.array(
					z.object({
						/** Null when the check threw. */
						ok: z.boolean().nullable(),
						reason: z.enum(["rejected", "unreachable"]).nullable(),
						usage: z.number().nullable(),
						limit: z.number().nullable(),
						reset: z.string().nullable(),
						thrown: z.string().nullable(),
					}),
				),
				/** Paths asked with the authorization sent, in order. */
				requests: z.array(z.object({ path: z.string(), authorization: z.string().nullable() })),
				waiting: z.array(z.string()),
			}),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges, input.stored ?? undefined);
				const checks: Array<{
					ok: boolean | null;
					reason: "rejected" | "unreachable" | null;
					usage: number | null;
					limit: number | null;
					reset: string | null;
					thrown: string | null;
				}> = [];
				let tags = 0;
				for (const held of input.checks) {
					tags += 1;
					const signal = callerSignal(net.wheel, `caller-${tags}`, held);
					const auth: KeyAuth = held.auth === "stored" ? "stored" : { key: held.key };
					try {
						const result = await checkKey(net.transport, auth, signal);
						checks.push(
							result.ok
								? {
										ok: true,
										reason: null,
										usage: result.usage ?? null,
										limit: result.limit ?? null,
										reset: result.reset ?? null,
										thrown: null,
									}
								: {
										ok: false,
										reason: result.reason,
										usage: null,
										limit: null,
										reset: null,
										thrown: null,
									},
						);
					} catch (error) {
						checks.push({
							ok: null,
							reason: null,
							usage: null,
							limit: null,
							reset: null,
							thrown: error instanceof Error ? error.message : jsString(error),
						});
					}
					net.wheel.cancel(`caller-${tags}`);
				}
				return {
					checks,
					requests: net.played.requests.map((sent) => ({
						path: sent.url.slice(BASE_URL.length),
						authorization: sent.headers.find(([name]) => name === "Authorization")?.[1] ?? null,
					})),
					waiting: net.wheel.waiting(),
				};
			},
		),
		/** The dropdown lines of a table of kinds; none given reads `KINDS`. */
		lines: kind(
			z.object({ kinds: z.array(ProviderKindName).exactOptional() }),
			z.object({ lines: z.array(KindLine) }),
			(input) => ({ lines: kindLines(tableOf(input.kinds)) }),
		),
		/** An entry's line of the provider list. */
		providerView: kind(
			z.object({
				entry: ProviderEntry,
				status: KeyState,
				/** The table's kinds; none given reads `KINDS`. */
				kinds: z.array(ProviderKindName).exactOptional(),
				bill: Billed.exactOptional(),
			}),
			z.object({ view: ProviderView }),
			(input) => ({ view: providerView(input.entry, input.status, tableOf(input.kinds), input.bill) }),
		),
		/** A kind's Settings lines for states and bills. */
		facts: kind(
			z.object({
				kind: HostedKindName,
				entry: HostedEntry,
				rows: z.array(z.object({ state: KeyState, bill: Billed.exactOptional() })),
			}),
			z.object({ facts: z.array(z.string().nullable()) }),
			(input) => {
				const named = hostedKind(input.kind);
				const entry = entryOf(input.kind, input.entry);
				return { facts: input.rows.map((row) => named.facts?.(row.state, entry, row.bill) ?? null) };
			},
		),
		/** A kind's check of its key on a scripted network: the state and the requests. */
		check: kind(
			z.object({
				kind: HostedKindName,
				entry: HostedEntry,
				/** The key given to the check; null is none. */
				secret: z.string().nullable(),
				baseUrl: z.string().exactOptional(),
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
				const net = kindNetworkOf(input.startMs, input.exchanges);
				const named = hostedKind(input.kind);
				const entry = entryOf(input.kind, input.entry);
				const deps = input.baseUrl === undefined ? net.deps : { ...net.deps, baseUrl: input.baseUrl };
				const signal = callerSignal(net.wheel, "caller", input);
				let state: z.infer<typeof KeyState> | null = null;
				let thrown: string | null = null;
				try {
					state = await named.check(entry, input.secret ?? undefined, deps, signal);
				} catch (error) {
					thrown = messageOf(error);
				}
				net.wheel.cancel("caller");
				return {
					state,
					thrown,
					facts: state === null ? null : (named.facts?.(state, entry, undefined) ?? null),
					requests: net.played.requests.map(requestLine),
					waiting: net.wheel.waiting(),
				};
			},
		),
		/** A kind's spend limits from its key, on a scripted network. */
		limits: kind(
			z.object({
				kind: HostedKindName,
				entry: HostedEntry,
				/** The stored key; null is none. */
				stored: z.string().nullable(),
				startMs: z.int(),
				exchanges: z.array(NetExchange),
				...abortOptions,
			}),
			z.object({
				/** The kind has limits to read. */
				present: z.boolean(),
				limits: Limits.nullable(),
				requests: z.array(RequestLine()),
				waiting: z.array(z.string()),
			}),
			async (input) => {
				const net = kindNetworkOf(input.startMs, input.exchanges);
				const named = hostedKind(input.kind);
				const transport = named.transport(entryOf(input.kind, input.entry), keysOf(input.stored), net.deps);
				const signal = callerSignal(net.wheel, "caller", input);
				const limits = named.limits === undefined ? null : ((await named.limits(transport, signal)) ?? null);
				net.wheel.cancel("caller");
				return {
					present: named.limits !== undefined,
					limits,
					requests: net.played.requests.map(requestLine),
					waiting: net.wheel.waiting(),
				};
			},
		),
		/** What a kind's codec sends for a view, and how it reads failures. */
		codec: kind(
			z.object({
				kind: HostedKindName,
				label: z.string(),
				view: CodecView().nullable(),
				request: CanonicalRequest,
				mode: z.enum(["stream", "complete"]),
				/** The clock the codec dates resets by. */
				nowMs: z.int(),
				failures: z.array(
					z.object({
						status: z.int(),
						body: z.unknown(),
						headers: z.array(z.tuple([z.string(), z.string()])).exactOptional(),
					}),
				),
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
				const named = hostedKind(input.kind);
				const view: ModelView | undefined =
					input.view === null
						? undefined
						: {
								row: rowsOfKeys(["openai/gpt-6-luna"])[0] as ModelView["row"],
								status: "ok",
								efforts: input.view.efforts,
								hosts: [],
								...(input.view.api === undefined ? {} : { api: input.view.api }),
								...(input.view.outputLimit === undefined
									? {}
									: { outputLimit: input.view.outputLimit }),
								...(input.view.price === undefined ? {} : { price: input.view.price }),
							};
				const codec = named.codec(
					view,
					{ kind: input.kind, label: input.label },
					kindNetworkOf(input.nowMs, []).deps,
				);
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
					failures: input.failures.map((each) => codec.failure(each.status, each.body, each.headers ?? [])),
				};
			},
		),
		/** What a kind does with each failure, against the view it met. */
		disposition: kind(
			z.object({
				kind: HostedKindName,
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
				const named = hostedKind(input.kind);
				const row = rowsOfKeys(["openai/gpt-6-luna"])[0] as ModelView["row"];
				return {
					dispositions: input.rows.map((each) => {
						const view: ModelView | undefined =
							each.status === null ? undefined : { row, status: each.status, efforts: [], hosts: [] };
						return named.failure(each.failure, view);
					}),
				};
			},
		),
		/** A kind's route on a scripted network: its catalog, browser and tester as the kind wires them. */
		route: kind(
			z.object({
				kind: HostedKindName,
				entry: HostedEntry,
				startMs: z.int(),
				exchanges: z.array(NetExchange),
				/** The registry's rows, by key. */
				rows: z.array(z.string()),
				steps: z.array(RouteStep()).min(1),
			}),
			z.object({
				steps: z.array(z.object({ nowMs: z.int(), answer: RouteAnswer() })),
				/** Paths asked, in order. */
				requests: z.array(z.string()),
				/** Chunks handed out, per exchange. */
				read: z.array(z.int()),
				waiting: z.array(z.string()),
			}),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges);
				const named = hostedKind(input.kind);
				const entry = { id: "e1", ...entryOf(input.kind, input.entry) };
				const deps = {
					fetch: net.played.fetch,
					scheduler: net.scheduler,
					log: () => {},
					userAgent: "cases/1.0",
					baseUrl: BASE_URL,
				};
				const transport = named.transport(entry, net.keys, deps);
				const registry = scriptedModelRegistry(rowsOfKeys(input.rows));
				const catalog = named.catalog(entry, transport, {
					scheduler: net.scheduler,
					rows: registry,
					refreshed: () => {},
				});
				const route = { transport, catalog };
				const browser = named.browse?.(entry, route, deps);
				const tester = named.tester?.(entry, route, deps);
				if (browser === undefined || tester === undefined)
					throw new Error("The kind has no browser or tester.");
				const outcomes: Array<{ nowMs: number; answer: RouteAnswerOf }> = [];
				let tags = 0;
				for (const step of input.steps) {
					tags += 1;
					const abort = "abortAfterMs" in step || "abortedBefore" in step ? step : {};
					const signal = callerSignal(net.wheel, `caller-${tags}`, abort);
					let answer: RouteAnswerOf;
					try {
						if (step.kind === "views") {
							const views = await catalog.views(step.use ?? CHAT, step.zdrOnly ?? true, signal);
							answer = { kind: "views", views: views.map(viewLine) };
						} else if (step.kind === "list") {
							answer = { kind: "list", models: (await browser.list(step.fresh)) ?? null };
						} else if (step.kind === "detail") {
							answer = { kind: "detail", detail: (await browser.detail(step.id, signal)) ?? null };
						} else if (step.kind === "listed") {
							answer = { kind: "listed", listed: (await tester.listed(step.id, signal)) ?? null };
						} else if (step.kind === "view") {
							const row = rowsOfKeys([step.row])[0] as ModelView["row"];
							answer = {
								kind: "view",
								view: viewLine(await tester.view(row, step.use ?? CHAT, signal)),
							};
						} else {
							net.wheel.advance(step.ms);
							answer = { kind: "advanced" };
						}
					} catch (error) {
						answer = { kind: "thrown", message: messageOf(error) };
					}
					net.wheel.cancel(`caller-${tags}`);
					outcomes.push({ nowMs: net.wheel.nowMs(), answer });
				}
				return {
					steps: outcomes,
					requests: net.asked(),
					read: net.played.read(),
					waiting: net.wheel.waiting(),
				};
			},
		),
	},
	sources: withUses(casesFor("kind", kindKeyCases, kindCases, kindCheckCases, kindRouteCases)),
});
