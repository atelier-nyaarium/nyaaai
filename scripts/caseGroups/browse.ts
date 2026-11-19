import { OPENROUTER_CHAT } from "@nyaaai/agentic/dialects/openrouter";
import { OpenCodeCatalog, opencodeBrowser, opencodeTester } from "@nyaaai/agentic/kinds/opencode/catalog";
import { browseModels, parseProviders } from "@nyaaai/agentic/kinds/openrouter/browse";
import { openRouterBrowser, openRouterTester } from "@nyaaai/agentic/kinds/openrouter/browser";
import { type Endpoint, ModelCatalog, parseEndpoints } from "@nyaaai/agentic/kinds/openrouter/catalog";
import { CHAT } from "@nyaaai/agentic/models/uses";
import type { Browser, Tester } from "@nyaaai/agentic/provider/kind";
import { jsString } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { BrowseDetail, BrowseModel, ModelUse, ProviderRow } from "@nyaaai/contract/shapes/model";
import { ProviderPlan } from "@nyaaai/contract/shapes/provider";
import { CanonicalRequest } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { scriptedModelRegistry } from "@nyaaai/testkit/scriptedPorts";
import { browseCases } from "../caseSources/browse";
import { browseOpenCodeCases } from "../caseSources/browseOpenCode";
import { withUses } from "../caseSources/uses";
import { abortOptions, callerSignal, NetExchange, networkOf, rowOfKey, ViewLine, viewLine } from "./catalogKit";

////////////////////////////////
//  Shapes

/** A step of a browser's script. */
const BrowserStep = () =>
	z.discriminatedUnion("kind", [
		z.object({ kind: z.literal("list"), fresh: z.boolean().exactOptional() }),
		z.object({ kind: z.literal("detail"), id: z.string(), ...abortOptions }),
		z.object({ kind: z.literal("listed"), id: z.string(), ...abortOptions }),
		z.object({ kind: z.literal("view"), row: z.string(), use: ModelUse.exactOptional(), ...abortOptions }),
		z.object({ kind: z.literal("advance"), ms: z.int().min(0) }),
	]);

const BrowserAnswer = () =>
	z.discriminatedUnion("kind", [
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

const BrowserResult = () =>
	z.object({
		steps: z.array(z.object({ nowMs: z.int(), answer: BrowserAnswer() })),
		/** Paths asked, in order. */
		requests: z.array(z.string()),
		/** Chunks handed out, per exchange. */
		read: z.array(z.int()),
		/** Timers still waiting at the end. */
		waiting: z.array(z.string()),
	});

type BrowserStepOf = z.infer<ReturnType<typeof BrowserStep>>;
type BrowserAnswerOf = z.infer<ReturnType<typeof BrowserAnswer>>;

////////////////////////////////
//  Functions & Helpers

const messageOf = (error: unknown): string => (error instanceof Error ? error.message : jsString(error));

/** Plays the steps on a browser and its tester, over the network of `net`. */
async function drive(
	net: ReturnType<typeof networkOf>,
	browser: Browser,
	tester: Tester,
	steps: readonly BrowserStepOf[],
): Promise<z.infer<ReturnType<typeof BrowserResult>>> {
	const outcomes: Array<{ nowMs: number; answer: BrowserAnswerOf }> = [];
	let tags = 0;
	for (const step of steps) {
		tags += 1;
		const abort = step.kind === "detail" || step.kind === "listed" || step.kind === "view" ? step : {};
		const signal = callerSignal(net.wheel, `caller-${tags}`, abort);
		let answer: BrowserAnswerOf;
		try {
			if (step.kind === "list") {
				const models = await browser.list(step.fresh);
				answer = { kind: "list", models: models ?? null };
			} else if (step.kind === "detail") {
				answer = { kind: "detail", detail: (await browser.detail(step.id, signal)) ?? null };
			} else if (step.kind === "listed") {
				answer = { kind: "listed", listed: (await tester.listed(step.id, signal)) ?? null };
			} else if (step.kind === "view") {
				const view = await tester.view(rowOfKey(step.row), step.use ?? CHAT, signal);
				answer = { kind: "view", view: viewLine(view) };
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
	return { steps: outcomes, requests: net.asked(), read: net.played.read(), waiting: net.wheel.waiting() };
}

////////////////////////////////
//  Group

export const browse = caseGroup("browse", {
	shapes: { NetExchange, ViewLine },
	kinds: {
		/** The listing of a fetch: raw items and the zero-retention hosts, at a time. */
		openrouterModels: kind(
			z.object({ raw: z.array(z.unknown()), zdr: z.unknown(), nowMs: z.int() }),
			z.object({ models: z.array(BrowseModel) }),
			(input) => ({
				models: browseModels(
					{ raw: input.raw, endpoints: parseEndpoints(input.zdr) ?? new Map() },
					input.nowMs,
				),
			}),
		),
		/** The providers of one model; `zdr` holds the hosts that keep no data, all of them read. */
		openrouterProviders: kind(
			z.object({ raw: z.unknown(), zdr: z.unknown() }),
			z.object({ providers: z.array(ProviderRow).nullable() }),
			(input) => {
				const zdr: Endpoint[] = [...(parseEndpoints(input.zdr)?.values() ?? [])].flat();
				return { providers: parseProviders(input.raw, zdr) ?? null };
			},
		),
		/** The browser and the tester of OpenRouter over a catalog on a scripted network. */
		openrouterBrowser: kind(
			z.object({
				startMs: z.int(),
				exchanges: z.array(NetExchange),
				steps: z.array(BrowserStep()).min(1),
			}),
			BrowserResult(),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges);
				const catalog = new ModelCatalog({
					transport: net.transport,
					scheduler: net.scheduler,
					rows: scriptedModelRegistry([]),
				});
				const browser = openRouterBrowser(catalog, net.transport, net.scheduler);
				const tester = openRouterTester(catalog, net.transport, net.scheduler);
				return drive(net, browser, tester, input.steps);
			},
		),
		/** The two hosts a test pins: the first two serving hosts that are also in the view. */
		openrouterHop: kind(
			z.object({
				startMs: z.int(),
				exchanges: z.array(NetExchange),
				id: z.string(),
				/** The view's zero-retention host tags. */
				viewHosts: z.array(z.string()),
				...abortOptions,
			}),
			z.object({
				hosts: z.array(z.string()).nullable(),
				thrown: z.string().nullable(),
				requests: z.array(z.string()),
				waiting: z.array(z.string()),
				nowMs: z.int(),
			}),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges);
				const catalog = new ModelCatalog({
					transport: net.transport,
					scheduler: net.scheduler,
					rows: scriptedModelRegistry([]),
				});
				const pins = openRouterTester(catalog, net.transport, net.scheduler).pins?.();
				if (pins === undefined) throw new Error("The tester has no pins.");
				const view = {
					row: rowOfKey("openai/gpt-6-luna"),
					status: "ok" as const,
					efforts: [],
					hosts: input.viewHosts,
				};
				const signal = callerSignal(net.wheel, "caller", input);
				let hosts: string[] | null = null;
				let thrown: string | null = null;
				try {
					hosts = await pins.hop(input.id, view, signal);
				} catch (error) {
					thrown = messageOf(error);
				}
				net.wheel.cancel("caller");
				return { hosts, thrown, requests: net.asked(), waiting: net.wheel.waiting(), nowMs: net.wheel.nowMs() };
			},
		),
		/** Host pins on the Chat codec: each call pins hosts, encodes a request, or asks about resent reasoning. */
		openrouterPins: kind(
			z.object({
				calls: z.array(
					z.discriminatedUnion("kind", [
						z.object({ kind: z.literal("pin"), hosts: z.array(z.string()) }),
						z.object({
							kind: z.literal("encode"),
							request: CanonicalRequest,
							mode: z.enum(["stream", "complete"]),
						}),
						z.object({ kind: z.literal("passedBack") }),
					]),
				),
			}),
			z.object({
				answers: z.array(
					z.discriminatedUnion("kind", [
						z.object({ kind: z.literal("pinned") }),
						z.object({
							kind: z.literal("encoded"),
							/** The request's `provider` field. */
							provider: z.unknown(),
							/** An assistant message carries reasoning details. */
							replayed: z.boolean(),
						}),
						z.object({ kind: z.literal("passedBack"), value: z.boolean() }),
					]),
				),
			}),
			(input) => {
				const net = networkOf(0, []);
				const catalog = new ModelCatalog({
					transport: net.transport,
					scheduler: net.scheduler,
					rows: scriptedModelRegistry([]),
				});
				const pins = openRouterTester(catalog, net.transport, net.scheduler).pins?.();
				if (pins === undefined) throw new Error("The tester has no pins.");
				const codec = pins.wrap(OPENROUTER_CHAT);
				const answers: Array<
					| { kind: "pinned" }
					| { kind: "encoded"; provider: unknown; replayed: boolean }
					| { kind: "passedBack"; value: boolean }
				> = [];
				for (const call of input.calls) {
					if (call.kind === "pin") {
						pins.pin(call.hosts);
						answers.push({ kind: "pinned" });
					} else if (call.kind === "encode") {
						const body = codec.encode(call.request, call.mode).body as {
							provider?: unknown;
							messages: Array<{ role: string; reasoning_details?: unknown[] }>;
						};
						answers.push({
							kind: "encoded",
							provider: body.provider ?? null,
							replayed: body.messages.some((message) => (message.reasoning_details?.length ?? 0) > 0),
						});
					} else answers.push({ kind: "passedBack", value: pins.passedBack() });
				}
				return { answers };
			},
		),
		/** The browser and the tester of an OpenCode plan over a catalog on a scripted network. */
		opencodeBrowser: kind(
			z.object({
				startMs: z.int(),
				plan: ProviderPlan,
				exchanges: z.array(NetExchange),
				steps: z.array(BrowserStep()).min(1),
			}),
			BrowserResult(),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges);
				const catalog = new OpenCodeCatalog({
					plan: input.plan,
					transport: net.transport,
					scheduler: net.scheduler,
					rows: scriptedModelRegistry([]),
				});
				const browser = opencodeBrowser(catalog, input.plan, net.scheduler);
				const tester = opencodeTester(catalog);
				return drive(net, browser, tester, input.steps);
			},
		),
	},
	sources: withUses(casesFor("browse", browseCases, browseOpenCodeCases)),
});
