// Builders for the transport cases. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { comment, data, finish, type Json, NOW, OPENROUTER, request, text } from "./codecFixtures";

////////////////////////////////
//  Constants

/** A key-shaped made-up constant. */
export const KEY = "fake-key-one";

/** Base64 of this has `+` and `/`, and its path encoding changes every symbol. */
export const ODD_KEY = "fake+key/v1==";

export const BASE_URL = "https://openrouter.ai/api/v1";

export const AGENT = "lab-app/1.0";

export const SSE_HEAD: Array<[string, string]> = [["content-type", "text/event-stream"]];

export const JSON_HEAD: Array<[string, string]> = [["content-type", "application/json"]];

export const DONE_TEXT = "data: [DONE]\n\n";

/** The standard request. */
export const REQUEST = request({ system: [], cache: { kind: "none" } });

////////////////////////////////
//  Functions & Helpers

/** One SSE frame of JSON. */
export const frame = (chunk: Json | string): string =>
	`data: ${typeof chunk === "string" ? chunk : JSON.stringify(chunk)}\n\n`;

export const commentFrame = (note: string): string => `:${note}\n\n`;

/** A text chunk of the body. */
export const piece = (body: string, delayMs?: number): Json => ({
	kind: "text",
	text: body,
	...(delayMs === undefined ? {} : { delayMs }),
});

/** A chunk of raw bytes. */
export const bytes = (base64: string, delayMs?: number): Json => ({
	kind: "bytes",
	base64,
	...(delayMs === undefined ? {} : { delayMs }),
});

/** The text's UTF-8 bytes cut at these offsets, as byte chunks. */
export function splitAt(body: string, ...cuts: number[]): Json[] {
	const raw = Buffer.from(body, "utf8");
	const edges = [0, ...cuts, raw.length];
	return edges.slice(1).map((end, at) => bytes(raw.subarray(edges[at], end).toString("base64")));
}

/** `text` written `times` times. */
export const repeat = (body: string, times: number, delayMs?: number): Json => ({
	kind: "repeat",
	text: body,
	times,
	...(delayMs === undefined ? {} : { delayMs }),
});

/** A streamed answer: the frames as one chunk, or the chunks given. */
export function sseReply(chunks: Json[] | string, over: Json = {}): Json {
	return {
		status: 200,
		headers: SSE_HEAD,
		chunks: typeof chunks === "string" ? [piece(chunks)] : chunks,
		...over,
	};
}

/** A whole JSON answer, as one chunk. */
export function jsonReply(body: unknown, status = 200, headers: Array<[string, string]> = [], over: Json = {}): Json {
	return {
		status,
		headers: [...JSON_HEAD, ...headers],
		chunks: [piece(typeof body === "string" ? body : JSON.stringify(body))],
		...over,
	};
}

/** The frames of a short answer. */
export const HI_FRAMES = [frame(text("Hi")), frame(finish("stop")), DONE_TEXT].join("");

export const streamCall = (over: Json = {}, req: Json = REQUEST): Json => ({ kind: "stream", request: req, ...over });

export const completeCall = (over: Json = {}, req: Json = REQUEST): Json => ({
	kind: "complete",
	request: req,
	...over,
});

export const getCall = (path: string, over: Json = {}): Json => ({ kind: "getJson", path, ...over });

export const postCall = (path: string, body: unknown, over: Json = {}): Json => ({
	kind: "postJson",
	path,
	...(body === undefined ? {} : { body }),
	...over,
});

/** A host's change of one preset's idle bound. */
export const idle = (ms: number, preset = "stream"): Json => ({ budgets: { [preset]: { idleMs: ms } } });

/** A host's change of one preset's size. */
export const capped = (bytes: number, preset = "stream"): Json => ({ budgets: { [preset]: { maxBytes: bytes } } });

/** One transport run. `over` replaces any input field. */
export function exchange(name: string, calls: Json[], exchanges: Json[], over: Json = {}): CaseSource {
	return {
		kind: "transport.exchange",
		name,
		input: {
			codec: OPENROUTER,
			baseUrl: BASE_URL,
			label: "OpenRouter",
			userAgent: AGENT,
			keys: [KEY],
			startMs: NOW,
			script: { kind: "inline", exchanges },
			calls,
			...over,
		},
	};
}

/** A run over a recorded trace. */
export function replay(name: string, file: string, calls: Json[], over: Json = {}): CaseSource {
	return exchange(name, calls, [], { script: { kind: "trace", file }, ...over });
}

export const keyed = (name: string, text: string): CaseSource => ({
	kind: "transport.key",
	name,
	input: { text },
});

export { comment, data, finish, NOW, text };
