// OpenRouter model browser and tester.

import type { Clock, Scheduler } from "@nyaaai/contract/time";
import { deadline } from "@nyaaai/contract/time";
import type { ChatRequest } from "../../chat/wire";
import type { Codec } from "../../codec";
import type { Browser, Pins, Tester } from "../../provider/kind";
import type { Transport } from "../../transport";
import { browseModels, parseProviders } from "./browse";
import type { ModelCatalog } from "./catalog";

////////////////////////////////
//  Constants

const DETAIL_MS = 30_000;

////////////////////////////////
//  Functions & Helpers

/** Provider filter tag. */
function slugOf(tag: string): string {
	return tag.split("/")[0] ?? tag;
}

/** Catalog and provider endpoints. */
export function openRouterBrowser(
	catalog: Pick<ModelCatalog, "listing">,
	transport: Pick<Transport, "getJson">,
	clock: Clock,
): Browser {
	return {
		list: async (fresh) => {
			const listing = await catalog.listing(undefined, fresh);
			return listing === undefined ? undefined : browseModels(listing, clock.nowMs());
		},
		detail: async (id, signal) => {
			const listing = await catalog.listing(signal);
			const raw = await transport.getJson(`/models/${id}/endpoints`, signal, "none");
			const hosts = parseProviders(raw, listing?.endpoints.get(id) ?? []);
			return hosts === undefined ? undefined : { hosts };
		},
	};
}

export function openRouterTester(
	catalog: Pick<ModelCatalog, "entry" | "viewFor">,
	transport: Pick<Transport, "getJson">,
	scheduler: Scheduler,
): Tester {
	return {
		listed: async (id, signal) => {
			const entry = await catalog.entry(id, signal);
			if (entry === undefined) return undefined;
			return {
				label: entry.name ?? id,
				reasoning: (entry.reasoning?.supported_efforts ?? []).length > 0,
			};
		},
		view: (row, use, signal) => catalog.viewFor(row, use, signal),
		pins: () => pinsOf(transport, scheduler),
	};
}

/** Pin and record tests. */
function pinsOf(transport: Pick<Transport, "getJson">, scheduler: Scheduler): Pins {
	let hosts: string[] = [];
	let step = 0;
	let sent: ChatRequest[] = [];
	return {
		hop: async (id, view, signal) => {
			const limit = deadline(scheduler, DETAIL_MS, signal);
			let raw: unknown;
			try {
				raw = await transport.getJson(`/models/${id}/endpoints`, limit.signal, "none");
			} finally {
				limit.clear();
			}
			const serving = parseProviders(raw, []) ?? [];
			const zdr = new Set(view.hosts.map(slugOf));
			const both = [...new Set(serving.map((provider) => slugOf(provider.tag)))].filter((slug) => zdr.has(slug));
			return both.length < 2 ? [] : both.slice(0, 2);
		},
		wrap: (base) =>
			({
				...base,
				encode: (request, mode) => {
					const encoded = base.encode(request, mode);
					if (mode !== "stream") return encoded;
					const body = encoded.body as ChatRequest;
					sent.push(body);
					const host = hosts[step++ % Math.max(1, hosts.length)];
					if (host === undefined) return encoded;
					return {
						...encoded,
						body: { ...body, provider: { require_parameters: true, ...body.provider, only: [host] } },
					};
				},
			}) satisfies Codec,
		pin: (next) => {
			hosts = [...next];
			step = 0;
			sent = [];
		},
		passedBack: () =>
			sent
				.slice(1)
				.some((request) =>
					request.messages.some(
						(message) => message.role === "assistant" && (message.reasoning_details?.length ?? 0) > 0,
					),
				),
	};
}
