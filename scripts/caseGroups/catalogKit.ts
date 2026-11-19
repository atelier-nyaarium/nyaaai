// What the catalog, browse and kind groups share: rows by key, a scripted network and a view line.

import { OPENROUTER_CHAT } from "@nyaaai/agentic/dialects/openrouter";
import { MODELS } from "@nyaaai/agentic/models/curated";
import { rowOf } from "@nyaaai/agentic/models/registry";
import { parseRowKey } from "@nyaaai/agentic/models/rowKey";
import { Transport } from "@nyaaai/agentic/transport";
import { stringifyJson } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import type { ModelView } from "@nyaaai/contract/shapes/model";
import { type ModelRow, Price, type TestRecord } from "@nyaaai/contract/shapes/model";
import { ApiFacts, Effort } from "@nyaaai/contract/shapes/step";
import { fakeKeys } from "@nyaaai/testkit/keys";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { type ScriptedExchange, scriptedFetch } from "@nyaaai/testkit/scriptedFetch";

////////////////////////////////
//  Shapes

/** One reply of the scripted network. `json` is the body, `text` raw text instead. */
export const NetExchange = z.object({
	status: z.int(),
	json: z.unknown().exactOptional(),
	text: z.string().exactOptional(),
	/** Wheel time spent before the head. */
	headMs: z.int().exactOptional(),
	/** `closed` by default. */
	end: z.enum(["closed", "dropped", "refused"]).exactOptional(),
});

/** A view without its row, named by the row's key. */
export const ViewLine = z.object({
	key: z.string(),
	status: z.enum(["ok", "retired", "noZdrHost", "noTools", "unknown"]),
	retiredOn: z.string().exactOptional(),
	efforts: z.array(Effort),
	promptLimit: z.number().exactOptional(),
	outputLimit: z.number().exactOptional(),
	price: z.union([Price, z.literal("free"), z.literal("unknown")]).exactOptional(),
	cacheWrite1h: z.literal(true).exactOptional(),
	hosts: z.array(z.string()),
	dataKept: z.literal(true).exactOptional(),
	api: ApiFacts.exactOptional(),
	cache: z.enum(["implicit", "explicit"]).exactOptional(),
});
export type ViewLine = z.infer<typeof ViewLine>;

/** The caller's cancel, relative to the call's start. */
export const abortOptions = {
	/** Aborts after this much time on the wheel. */
	abortAfterMs: z.int().exactOptional(),
	/** The signal is aborted before the call. */
	abortedBefore: z.boolean().exactOptional(),
};

////////////////////////////////
//  Constants

export const BASE_URL = "https://catalog.test/v1";

export const CALLER_REASON = "caller aborted";

/** A passing record for a row made up from its key. */
const PASSING: TestRecord = {
	label: "Made Up",
	date: "2026-01-01",
	cost: 0,
	checks: { toolPlus: "pass", recovery: "pass", reasoning: "pass", schema: "pass" },
	reasoning: true,
	cache: "none",
	verifiedPrice: { prompt: 1, completion: 2 },
};

////////////////////////////////
//  Functions & Helpers

/** A curated row by key, or a hosted row (`provider:id`) made up. */
export function rowOfKey(key: string): ModelRow {
	const curated = MODELS.find((row) => row.key === key);
	if (curated !== undefined) return curated;
	const identity = parseRowKey(key);
	if (identity === undefined || identity.kind !== "hosted") throw new Error(`No row ${key}.`);
	return rowOf(identity, PASSING, identity.provider);
}

export const rowsOfKeys = (keys: readonly string[]): ModelRow[] => keys.map(rowOfKey);

export function viewLine(view: ModelView): ViewLine {
	const { row, ...rest } = view;
	return { key: row.key, ...rest };
}

const UTF8 = new TextEncoder();

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

/** A transport on a wheel, over the exchanges in order. */
export function networkOf(
	startMs: number,
	exchanges: readonly z.infer<typeof NetExchange>[],
	stored: string | undefined = "key-0123456789abcdef",
) {
	const wheel = new TimerWheel(startMs);
	const played = scriptedFetch(exchanges.map(exchangeOf), wheel);
	const scheduler = fakeScheduler(wheel);
	const keys = fakeKeys([stored]);
	const transport = new Transport({
		fetch: played.fetch,
		scheduler,
		key: keys,
		log: () => {},
		baseUrl: BASE_URL,
		label: "Catalog",
		userAgent: "cases/1.0",
		errors: OPENROUTER_CHAT,
	});
	/** Paths asked, in order. */
	const asked = () => played.requests.map((sent) => sent.url.slice(BASE_URL.length));
	return { wheel, scheduler, transport, played, keys, asked };
}

/** A signal of the caller's, aborted at its time on the wheel. */
export function callerSignal(
	wheel: TimerWheel,
	tag: string,
	options: { abortAfterMs?: number | undefined; abortedBefore?: boolean | undefined },
): AbortSignal {
	const caller = new AbortController();
	if (options.abortedBefore === true) caller.abort(new Error(CALLER_REASON));
	else if (options.abortAfterMs !== undefined)
		wheel.after(options.abortAfterMs, tag, () => caller.abort(new Error(CALLER_REASON)));
	return caller.signal;
}
