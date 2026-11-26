// The OpenCode kind.

import { fixedText } from "@nyaaai/contract/number";
import type { ModelView } from "@nyaaai/contract/shapes/model";
import type { Billed, EntryFields, KeyState, ProviderPlan } from "@nyaaai/contract/shapes/provider";
import type { Codec } from "../../codec";
import { opencodeChat, opencodeErrors } from "../../dialects/opencode";
import { failureOf } from "../../failure";
import { geminiCodec } from "../../gemini/codec";
import { opencodeHost } from "../../hosts/opencode";
import { messagesCodec } from "../../messages/codec";
import { codecFactsOf } from "../../models/view";
import { dispositionOf } from "../../provider/disposition";
import type { KindDeps, ProviderKind, Route } from "../../provider/kind";
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

const DEFAULT_PLAN: ProviderPlan = "go";

const DOT = " \u{b7} ";

////////////////////////////////
//  Functions & Helpers

function planOf(entry: EntryFields): ProviderPlan {
	return entry.plan ?? DEFAULT_PLAN;
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

/** Unknown views use Chat. */
function codecOf(view: ModelView | undefined, deps: KindDeps): Codec {
	const clock = deps.scheduler;
	switch (view?.api?.family) {
		case "responses":
			return responsesCodec(opencodeHost("/openai/v1", clock), codecFactsOf(view));
		case "messages":
			return messagesCodec(opencodeHost("/anthropic/v1", clock), codecFactsOf(view));
		case "generate-content":
			return geminiCodec(opencodeHost("/google/v1beta", clock), codecFactsOf(view));
		default:
			return opencodeChat(codecFactsOf(view), clock);
	}
}

export const OPENCODE: ProviderKind = (() => {
	const self: ProviderKind = {
		kind: "opencode",
		label: "OpenCode",
		plans: PLANS,
		defaultPlan: DEFAULT_PLAN,
		field: "key",
		address: (entry) => BASE_URLS[planOf(entry)],
		balance: "https://opencode.ai/console",
		extraUsage: ["go"],
		facts: billingFacts,
		check: async (entry, secret, deps, span) => {
			if (secret === undefined) return { kind: "none" };
			try {
				// 200 clears rejection only.
				await self.transport(entry, staticKey(secret), deps).getJson("/v1/models", span, "stored");
				return { kind: "ok" };
			} catch (error) {
				if (span.signal.aborted) throw error;
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
				budgets: deps.budgets,
			}),
		catalog: (entry, transport, deps) => new OpenCodeCatalog({ plan: planOf(entry), transport, ...deps }),
		browse: (entry, route, deps) => opencodeBrowser(catalogOf(route), planOf(entry), deps.scheduler),
		tester: (_entry, route) => opencodeTester(catalogOf(route)),
		codec: (view, _entry, deps) => codecOf(view, deps),
		// notFound means unavailable.
		failure: (failure, view) => {
			const disposition = dispositionOf(failure, view);
			return disposition.act === "fallThrough" && disposition.notice === "badRequest"
				? { act: "fallThrough", notice: "modelUnavailable" }
				: disposition;
		},
	};
	return self;
})();
