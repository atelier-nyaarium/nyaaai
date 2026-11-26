// The OpenRouter kind.

import { fixedText } from "@nyaaai/contract/number";
import type { EntryFields, KeyState } from "@nyaaai/contract/shapes/provider";
import { OPENROUTER_CHAT } from "../../dialects/openrouter";
import { dispositionOf } from "../../provider/disposition";
import type { ProviderKind, Route } from "../../provider/kind";
import { staticKey, Transport } from "../../transport";
import { openRouterBrowser, openRouterTester } from "./browser";
import { ModelCatalog } from "./catalog";
import { checkKey, type KeyCheck } from "./key";

////////////////////////////////
//  Constants

/** The entry scripts and tests use. */
export const OPENROUTER_ENTRY: EntryFields = { kind: "openrouter", label: "OpenRouter" };

const BASE_URL = "https://openrouter.ai/api/v1";

const DOT = " \u{b7} ";

////////////////////////////////
//  Functions & Helpers

function usd(value: number): string {
	return `$${fixedText(value, 2)}`;
}

/** No limit means unlimited. */
function usageFacts(state: KeyState): string | undefined {
	if (state.kind !== "ok" || state.usage === undefined) return undefined;
	const used = `${usd(state.usage)} used`;
	return state.limit === undefined ? used : `${used}${DOT}${usd(state.limit)} key limit`;
}

/** Every OpenRouter route holds one. */
function catalogOf(route: Route): ModelCatalog {
	if (!(route.catalog instanceof ModelCatalog)) throw new Error("Not an OpenRouter catalog.");
	return route.catalog;
}

function keyStateOf(check: KeyCheck): KeyState {
	if (!check.ok) return { kind: check.reason };
	return {
		kind: "ok",
		...(check.usage === undefined ? {} : { usage: check.usage }),
		...(check.limit === undefined ? {} : { limit: check.limit }),
	};
}

export const OPENROUTER: ProviderKind = (() => {
	const self: ProviderKind = {
		kind: "openrouter",
		label: "OpenRouter",
		field: "key",
		address: () => BASE_URL,
		balance: "https://openrouter.ai/settings/credits",
		stats: true,
		facts: usageFacts,
		check: async (entry, secret, deps, span) => {
			if (secret === undefined) return { kind: "none" };
			return keyStateOf(await checkKey(self.transport(entry, staticKey(secret), deps), "stored", span));
		},
		transport: (_entry, key, deps) =>
			new Transport({
				fetch: deps.fetch,
				key,
				log: deps.log,
				baseUrl: deps.baseUrl ?? BASE_URL,
				label: "OpenRouter",
				errors: OPENROUTER_CHAT,
				scheduler: deps.scheduler,
				userAgent: deps.userAgent,
				budgets: deps.budgets,
			}),
		catalog: (_entry, transport, deps) => new ModelCatalog({ transport, ...deps }),
		browse: (_entry, route, deps) =>
			openRouterBrowser(catalogOf(route), route.transport, deps.scheduler, deps.budgets),
		tester: (_entry, route, deps) =>
			openRouterTester(catalogOf(route), route.transport, deps.scheduler, deps.budgets),
		codec: () => OPENROUTER_CHAT,
		failure: dispositionOf,
		limits: async (transport, span) => {
			const key = await checkKey(transport, "stored", span).catch(() => undefined);
			if (key?.ok !== true) return undefined;
			return {
				...(key.limit === undefined ? {} : { limit: key.limit }),
				...(key.reset === undefined ? {} : { reset: key.reset }),
			};
		},
	};
	return self;
})();
