// OpenRouter model browser and tester.

import { BUDGETS, type Budgets, span } from "@nyaaai/contract/fetch";
import { asciiLower, encodePathSegment, trimText } from "@nyaaai/contract/text";
import type { Scheduler } from "@nyaaai/contract/time";
import type { ChatRequest } from "../../chat/wire";
import type { Codec } from "../../codec";
import type { Browser, Pins, Tester } from "../../provider/kind";
import type { Transport } from "../../transport";
import { browseModels, parseProviders } from "./browse";
import type { ModelCatalog } from "./catalog";

////////////////////////////////
//  Functions & Helpers

/** Provider filter tag; one rule for tags and host names. */
function slugOf(tag: string): string {
	return asciiLower(trimText(tag.split("/")[0] ?? tag));
}

/** A path segment; dot segments are escaped so none climbs. */
function segmentOf(part: string): string {
	if (part === ".") return "%2E";
	return part === ".." ? "%2E%2E" : encodePathSegment(part);
}

/** An id's endpoints path, its slashes kept. */
export function endpointsPath(id: string): `/${string}` {
	return `/models/${id.split("/").map(segmentOf).join("/")}/endpoints`;
}

/** An endpoints read under a span from `detail`. */
async function endpointsOf(
	transport: Pick<Transport, "getJson">,
	scheduler: Scheduler,
	budgets: Budgets,
	id: string,
	signal: AbortSignal,
): Promise<unknown> {
	const clock = span(scheduler, budgets.detail, signal);
	try {
		return await transport.getJson(endpointsPath(id), clock, "none");
	} finally {
		clock.clear();
	}
}

/** Catalog and provider endpoints. */
export function openRouterBrowser(
	catalog: Pick<ModelCatalog, "listing">,
	transport: Pick<Transport, "getJson">,
	scheduler: Scheduler,
	budgets: Budgets = BUDGETS,
): Browser {
	return {
		list: async (fresh) => {
			const listing = await catalog.listing(undefined, fresh);
			return listing === undefined ? undefined : browseModels(listing, scheduler.nowMs());
		},
		detail: async (id, signal) => {
			const listing = await catalog.listing(signal);
			const raw = await endpointsOf(transport, scheduler, budgets, id, signal);
			const hosts = parseProviders(raw, listing?.endpoints.get(id) ?? []);
			return hosts === undefined ? undefined : { hosts };
		},
	};
}

export function openRouterTester(
	catalog: Pick<ModelCatalog, "entry" | "viewFor">,
	transport: Pick<Transport, "getJson">,
	scheduler: Scheduler,
	budgets: Budgets = BUDGETS,
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
		pins: () => pinsOf(transport, scheduler, budgets),
	};
}

/** Pin and record tests. */
function pinsOf(transport: Pick<Transport, "getJson">, scheduler: Scheduler, budgets: Budgets): Pins {
	let hosts: string[] = [];
	let step = 0;
	let sent: ChatRequest[] = [];
	return {
		hop: async (id, view, signal) => {
			const raw = await endpointsOf(transport, scheduler, budgets, id, signal);
			const serving = parseProviders(raw, []) ?? [];
			const zdr = new Set(view.hosts.map(slugOf));
			const both = [...new Set(serving.map((provider) => slugOf(provider.tag)))].filter(
				(slug) => slug.length > 0 && zdr.has(slug),
			);
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
