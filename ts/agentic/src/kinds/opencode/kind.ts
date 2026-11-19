// The OpenCode kind.

import { fixedText } from "@nyaaai/contract/number";
import type { ModelView } from "@nyaaai/contract/shapes/model";
import type { Billed, EntryFields, KeyState, ProviderPlan } from "@nyaaai/contract/shapes/provider";
import { type Clock, systemClock } from "@nyaaai/contract/time";
import type { Codec } from "../../codec";
import { opencodeChat, opencodeErrors } from "../../dialects/opencode";
import { failureOf } from "../../failure";
import { geminiCodec } from "../../gemini/codec";
import { opencodeHost } from "../../hosts/opencode";
import { messagesCodec } from "../../messages/codec";
import { codecFactsOf } from "../../models/view";
import { dispositionOf } from "../../provider/disposition";
import type { ProviderKind, Route } from "../../provider/kind";
import { responsesCodec } from "../../responses/codec";
import { staticKey, Transport } from "../../transport";
import { OpenCodeCatalog, opencodeBrowser, opencodeTester } from "./catalog";

////////////////////////////////
//  Constants

const PLANS = [
	{ plan: "go", label: "OpenCode Go" },
	{ plan: "payg", label: "OpenCode (pay-as-you-go)" },
] as const satisfies readonly { plan: ProviderPlan; label: string }[];

const BASE_URLS: Record<ProviderPlan, string> = {
	go: "https://opencode.ai/inference/go",
	payg: "https://opencode.ai/inference",
};

const DOT = " \u{b7} ";

////////////////////////////////
//  Functions & Helpers

/** Missing plan: Go. */
function planOf(entry: EntryFields): ProviderPlan {
	return entry.plan ?? "go";
}

function labelOf(entry: EntryFields): string {
	return PLANS.find(({ plan }) => plan === planOf(entry))?.label ?? "OpenCode";
}

/** OpenCode route catalog. */
function catalogOf(route: Route): OpenCodeCatalog {
	if (!(route.catalog instanceof OpenCodeCatalog)) throw new Error("Not an OpenCode catalog.");
	return route.catalog;
}

function usd(value: number): string {
	return `$${fixedText(value, 2)}`;
}

/** Extra usage is Go-only. */
function billingFacts(_state: KeyState, entry: EntryFields, bill: Billed | undefined): string | undefined {
	if (bill === undefined) return undefined;
	const spent = bill.cost ?? 0;
	if (planOf(entry) === "payg") return bill.cost === undefined ? undefined : `${usd(spent)} spent`;
	const facts = [
		...(bill.allowance === undefined ? [] : [`Allowance ${usd(bill.allowance)} used`]),
		...(bill.billing === "balance" || spent > 0 ? [`Extra usage ${usd(spent)}`] : []),
	];
	return facts.length === 0 ? undefined : facts.join(DOT);
}

/** The kind; the clock dates what its codecs and browser read. */
export function opencodeKind(clock: Clock): ProviderKind {
	const openai = opencodeHost("/openai/v1", clock);
	const anthropic = opencodeHost("/anthropic/v1", clock);
	const google = opencodeHost("/google/v1beta", clock);

	/** Unknown views use Chat. */
	const codecOf = (view: ModelView | undefined): Codec => {
		switch (view?.api?.family) {
			case "responses":
				return responsesCodec(openai, codecFactsOf(view));
			case "messages":
				return messagesCodec(anthropic, codecFactsOf(view));
			case "generate-content":
				return geminiCodec(google, codecFactsOf(view));
			default:
				return opencodeChat(codecFactsOf(view), clock);
		}
	};

	const self: ProviderKind = {
		kind: "opencode",
		label: "OpenCode",
		plans: PLANS,
		field: "key",
		balance: "https://opencode.ai/console",
		extraUsage: ["go"],
		facts: billingFacts,
		check: async (entry, secret, deps, signal) => {
			if (secret === undefined) return { kind: "none" };
			try {
				// 200 clears rejection only.
				await self.transport(entry, staticKey(secret), deps).getJson("/v1/models", signal, "stored");
				return { kind: "ok" };
			} catch (error) {
				if (signal.aborted) throw error;
				const category = failureOf(error)?.category;
				return { kind: category === "keyRejected" || category === "keyMissing" ? "rejected" : "unreachable" };
			}
		},
		transport: (entry, key, deps) =>
			new Transport({
				fetch: deps.fetch,
				key,
				log: deps.log,
				baseUrl: deps.baseUrl ?? BASE_URLS[planOf(entry)],
				label: labelOf(entry),
				errors: opencodeErrors(deps.scheduler),
				scheduler: deps.scheduler,
				userAgent: deps.userAgent,
			}),
		catalog: (entry, transport, deps) => new OpenCodeCatalog({ plan: planOf(entry), transport, ...deps }),
		browse: (entry, route) => opencodeBrowser(catalogOf(route), planOf(entry), clock),
		tester: (_entry, route) => opencodeTester(catalogOf(route)),
		codec: codecOf,
		// notFound means unavailable.
		failure: (failure, view) => {
			const disposition = dispositionOf(failure, view);
			return disposition.act === "fallThrough" && disposition.notice === "badRequest"
				? { act: "fallThrough", notice: "modelUnavailable" }
				: disposition;
		},
	};
	return self;
}

export const OPENCODE: ProviderKind = opencodeKind(systemClock);
