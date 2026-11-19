// A provider kind for a server named by URL.

import type { EntryFields, KeyState, ProviderKindName } from "@nyaaai/contract/shapes/provider";
import { type LocalDialect, localChat } from "../../dialects/local";
import { failureOf } from "../../failure";
import { codecFactsOf } from "../../models/view";
import { dispositionOf } from "../../provider/disposition";
import type { ProviderKind, Route } from "../../provider/kind";
import { type KeySource, Transport } from "../../transport";
import { LocalCatalog, type LocalServer, type LocalTransport, localBrowser, localTester } from "./catalog";

////////////////////////////////
//  Interfaces & Types

/** An entry's retention and price. */
export interface Terms {
	/** Data may be kept. */
	kept: boolean;
	price: "free" | "unknown";
}

/** One local server type. */
export interface LocalKindSpec extends LocalDialect {
	kind: ProviderKindName;
	/** Default URL. */
	url?: string;
	/** Request root from the URL. */
	base?(url: string): string;
	/** Defaults to `HOME`. */
	terms?(entry: EntryFields): Terms;
	/** Model count; undefined when foreign. */
	reach(transport: LocalTransport, signal: AbortSignal): Promise<number | undefined>;
	server(transport: LocalTransport): LocalServer;
}

////////////////////////////////
//  Constants

/** Data stays home; calls bill nothing. */
export const HOME: Terms = { kept: false, price: "free" };

const DOT = " \u{b7} ";

const NO_KEY: KeySource = { get: async () => undefined };

const KEY_REFUSED: ReadonlySet<string> = new Set(["refused", "keyRejected", "keyMissing"]);

////////////////////////////////
//  Functions & Helpers

function catalogOf(route: Route): LocalCatalog {
	if (!(route.catalog instanceof LocalCatalog)) throw new Error("Not a local catalog.");
	return route.catalog;
}

/** The text without its trailing slashes. */
function withoutSlashes(text: string): string {
	let end = text.length;
	while (end > 0 && text[end - 1] === "/") end -= 1;
	return text.slice(0, end);
}

/** A server named by URL. */
export function localKind(spec: LocalKindSpec): ProviderKind {
	const baseOf = (entry: EntryFields) => {
		const url = withoutSlashes(entry.url ?? spec.url ?? "");
		return spec.base?.(url) ?? url;
	};
	const errors = localChat(undefined, spec.label, spec);

	const self: ProviderKind = {
		kind: spec.kind,
		label: spec.label,
		field: "url",
		...(spec.url === undefined ? {} : { url: spec.url }),
		...(spec.optionalKey === undefined ? {} : { optionalKey: spec.optionalKey }),
		// URL, then model count.
		facts: (state: KeyState, entry) => {
			const url = baseOf(entry);
			if (state.kind !== "ok" || state.models === undefined) return url;
			return `${url}${DOT}${state.models} ${state.models === 1 ? "model" : "models"}`;
		},
		check: async (entry, secret, deps, signal) => {
			try {
				const key = secret === undefined ? NO_KEY : { get: async () => secret };
				const models = await spec.reach(self.transport(entry, key, deps), signal);
				return models === undefined ? { kind: "unreachable" } : { kind: "ok", models };
			} catch (error) {
				if (signal.aborted) throw error;
				return KEY_REFUSED.has(failureOf(error)?.category ?? "")
					? { kind: "rejected" }
					: { kind: "unreachable" };
			}
		},
		transport: (entry, key, deps) =>
			new Transport({
				fetch: deps.fetch,
				key,
				log: deps.log,
				baseUrl: deps.baseUrl ?? baseOf(entry),
				label: spec.label,
				errors,
				auth: spec.optionalKey ? "optional" : "none",
				scheduler: deps.scheduler,
				userAgent: deps.userAgent,
			}),
		catalog: (entry, transport, deps) => {
			const catalog = new LocalCatalog({
				entry: entry.id,
				server: spec.server(transport),
				terms: spec.terms?.(entry) ?? HOME,
				scheduler: deps.scheduler,
				rows: deps.rows,
				refreshed: deps.refreshed,
			});
			transport.onSettled((model) => catalog.settled(model));
			return catalog;
		},
		browse: (_entry, route) => localBrowser(catalogOf(route)),
		tester: (_entry, route) => localTester(catalogOf(route)),
		codec: (view, entry) => localChat(codecFactsOf(view), entry.label, spec),
		// notFound means unavailable.
		failure: (failure, view) =>
			failure.category === "notFound"
				? { act: "fallThrough", notice: "modelUnavailable" }
				: dispositionOf(failure, view),
	};
	return self;
}
