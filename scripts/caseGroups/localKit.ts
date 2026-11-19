// Shared by the local group: shapes, rows, a scripted network and server.

import { COMPATIBLE, server as compatibleServer } from "@nyaaai/agentic/kinds/compatible";
import { LLAMACPP, server as llamacppServer } from "@nyaaai/agentic/kinds/llamacpp";
import { LMSTUDIO, server as lmstudioServer } from "@nyaaai/agentic/kinds/lmstudio";
import type { LocalServer, LocalTransport } from "@nyaaai/agentic/kinds/local/catalog";
import { OLLAMA, server as ollamaServer } from "@nyaaai/agentic/kinds/ollama";
import { rowOf } from "@nyaaai/agentic/models/registry";
import { parseRowKey } from "@nyaaai/agentic/models/rowKey";
import type { KindDeps, ProviderKind } from "@nyaaai/agentic/provider/kind";
import type { FetchRequest } from "@nyaaai/contract/fetch";
import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import type { ModelRow, TestRecord } from "@nyaaai/contract/shapes/model";
import { Effort } from "@nyaaai/contract/shapes/step";
import type { Scheduler } from "@nyaaai/contract/time";
import { fakeKeys } from "@nyaaai/testkit/keys";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { type ScriptedExchange, scriptedFetch } from "@nyaaai/testkit/scriptedFetch";
import type { NetExchange } from "./catalogKit";

////////////////////////////////
//  Shapes

export const LocalKindName = z.enum(["ollama", "lmstudio", "llamacpp", "compatible"]);
export type LocalKindName = z.infer<typeof LocalKindName>;

/** A model a server reported, as the catalog keeps it. */
export const ServedModel = z.object({
	tag: z.string(),
	tools: z.boolean(),
	thinking: z.boolean(),
	efforts: z.array(Effort),
	context: z.number().exactOptional(),
	loaded: z.number().exactOptional(),
	trained: z.number().exactOptional(),
	probed: z.literal(false).exactOptional(),
});

/** An entry's retention and price. */
export const ServedTerms = z.object({ kept: z.boolean(), price: z.enum(["free", "unknown"]) });

/** Entry, as a kind reads it. */
export const LocalEntry = z.object({ label: z.string(), url: z.string().exactOptional() });

/** A loaded context by tag. */
export const LoadedContext = z.object({ tag: z.string(), context: z.number() });

////////////////////////////////
//  Constants

export const LOCAL_KINDS: Record<LocalKindName, ProviderKind> = {
	ollama: OLLAMA,
	lmstudio: LMSTUDIO,
	llamacpp: LLAMACPP,
	compatible: COMPATIBLE,
};

const SERVERS: Record<LocalKindName, (transport: LocalTransport) => LocalServer> = {
	ollama: ollamaServer,
	lmstudio: lmstudioServer,
	llamacpp: llamacppServer,
	compatible: compatibleServer,
};

/** A test record every row of these cases holds. */
const PASSING: TestRecord = {
	label: "Made Up",
	date: "2026-01-01",
	cost: 0,
	checks: { toolPlus: "pass", recovery: "pass", reasoning: "pass", schema: "pass" },
	reasoning: true,
	cache: "none",
	verifiedPrice: { prompt: 0, completion: 0 },
};

export const USER_AGENT = "cases/1.0";

const UTF8 = new TextEncoder();

////////////////////////////////
//  Functions & Helpers

/** A local row by its key, `local:<entry>/<tag>`. */
export function localRowOf(key: string): ModelRow {
	const identity = parseRowKey(key);
	if (identity === undefined || identity.kind !== "local") throw new Error(`No local row ${key}.`);
	return rowOf(identity, { ...PASSING, label: identity.tag }, "ollama");
}

export const localRowsOf = (keys: readonly string[]): ModelRow[] => keys.map(localRowOf);

function exchangeOf(held: z.infer<typeof NetExchange>): ScriptedExchange {
	const body = held.text ?? stringifyJson(Object.hasOwn(held, "json") ? held.json : null);
	return {
		status: held.status,
		headers: [["content-type", "application/json"]],
		chunks: [{ bytes: UTF8.encode(body ?? "null") }],
		...(held.headMs === undefined ? {} : { headMs: held.headMs }),
		...(held.end === undefined ? {} : { end: held.end }),
	};
}

/** A kind's deps over a scripted network on a wheel. */
export function networkOf(startMs: number, exchanges: readonly z.infer<typeof NetExchange>[]) {
	const wheel = new TimerWheel(startMs);
	const played = scriptedFetch(exchanges.map(exchangeOf), wheel);
	const scheduler: Scheduler = fakeScheduler(wheel);
	const deps: KindDeps = { fetch: played.fetch, scheduler, log: () => {}, userAgent: USER_AGENT };
	return { wheel, played, scheduler, deps };
}

/** The key source of a case; null is none. */
export const keysOf = (stored: string | null) => fakeKeys([stored ?? undefined]);

/** A transport of the kind. */
export function transportOf(
	name: LocalKindName,
	entry: z.infer<typeof LocalEntry>,
	deps: KindDeps,
	stored: string | null,
) {
	return LOCAL_KINDS[name].transport({ kind: name, label: entry.label, ...urlOf(entry) }, keysOf(stored), deps);
}

export const urlOf = (entry: { url?: string | undefined }) => (entry.url === undefined ? {} : { url: entry.url });

/** The kind's own server over a transport. */
export const serverOf = (name: LocalKindName, transport: LocalTransport): LocalServer => SERVERS[name](transport);

/** What a request looked like on the wire. */
export function requestLine(sent: FetchRequest) {
	return {
		method: sent.method,
		url: sent.url,
		authorization: sent.headers.find(([name]) => name === "Authorization")?.[1] ?? null,
		body: sent.body === undefined ? null : (parseJson(sent.body) ?? null),
	};
}
