import path from "node:path";
import type { Codec } from "@nyaaai/agentic/codec";
import { failureOf } from "@nyaaai/agentic/failure";
import { isKeyShape, type KeyAuth, normalizeKey, Transport } from "@nyaaai/agentic/transport";
import { base64Decode } from "@nyaaai/contract/base64";
import { jsString } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { CanonicalRequest, Failure, HeaderList, StepEvent, StepResult } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { fakeKeys } from "@nyaaai/testkit/keys";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { type ScriptedExchange, scriptedFetch } from "@nyaaai/testkit/scriptedFetch";
import { readTrace, replayFetch } from "@nyaaai/testkit/trace";
import { transportCases } from "../caseSources/transport";
import { CodecName, codecOf } from "./codecOf";

////////////////////////////////
//  Shapes

/** One body chunk. `repeat` is `text` written `times` times. */
export const TransportChunk = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("text"), text: z.string(), delayMs: z.int().exactOptional() }),
	z.object({ kind: z.literal("bytes"), base64: z.string(), delayMs: z.int().exactOptional() }),
	z.object({ kind: z.literal("repeat"), text: z.string(), times: z.int().min(0), delayMs: z.int().exactOptional() }),
]);

/** What the network does for one request. */
export const TransportExchange = z.object({
	status: z.int(),
	/** Raw pairs; the fetch answers them lowercase, sorted, repeats joined. */
	headers: HeaderList,
	chunks: z.array(TransportChunk),
	/** Time spent before the head. */
	headMs: z.int().exactOptional(),
	/** `closed` by default. */
	end: z.enum(["closed", "dropped", "refused"]).exactOptional(),
	/** The failure message of `dropped` and `refused`. */
	error: z.string().exactOptional(),
});

/** Where a request's key comes from. */
export const TransportKeyAuth = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("none") }),
	z.object({ kind: z.literal("stored") }),
	z.object({ kind: z.literal("key"), key: z.string() }),
]);

/** The caller's cancel, relative to the call's start. */
const abort = {
	/** Aborts after this much time on the wheel. */
	abortAfterMs: z.int().exactOptional(),
	/** The signal is aborted before the call. */
	abortedBefore: z.boolean().exactOptional(),
};

export const TransportCall = z.discriminatedUnion("kind", [
	z.object({
		kind: z.literal("stream"),
		request: CanonicalRequest,
		/** Takes this many events, then lets go of the stream. */
		stopAfter: z.int().min(1).exactOptional(),
		...abort,
	}),
	z.object({ kind: z.literal("complete"), request: CanonicalRequest, ...abort }),
	/** The path starts with a slash. */
	z.object({ kind: z.literal("getJson"), path: z.string(), keyAuth: TransportKeyAuth.exactOptional(), ...abort }),
	z.object({
		kind: z.literal("postJson"),
		path: z.string(),
		/** Absent sends no body. */
		body: z.unknown().exactOptional(),
		keyAuth: TransportKeyAuth.exactOptional(),
		...abort,
	}),
]);

/** The script is played from the file, with no waiting. */
export const TransportScript = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("inline"), exchanges: z.array(TransportExchange) }),
	z.object({ kind: z.literal("trace"), file: z.string() }),
]);

export const TransportSent = z.object({
	method: z.string(),
	url: z.string(),
	/** In sent order. */
	headers: HeaderList,
	body: z.string().nullable(),
});

export const TransportOutcome = z.object({
	/** A stream's events up to its end or failure. */
	events: z.array(StepEvent),
	result: StepResult.nullable(),
	/** What a JSON call answered; null when it did not. */
	answer: z.unknown(),
	failure: Failure.nullable(),
	/** The message of anything thrown that is no decoded failure. */
	thrown: z.string().nullable(),
	/** Whether the caller's signal was aborted when the call ended. */
	aborted: z.boolean(),
	/** The wheel's time when the call ended. */
	nowMs: z.int(),
});

////////////////////////////////
//  Functions & Helpers

const CALLER_REASON = "caller aborted";

const UTF8 = new TextEncoder();

function bytesOf(chunk: z.infer<typeof TransportChunk>): Uint8Array {
	if (chunk.kind === "text") return UTF8.encode(chunk.text);
	if (chunk.kind === "repeat") return UTF8.encode(chunk.text.repeat(chunk.times));
	const bytes = base64Decode(chunk.base64);
	if (bytes === undefined) throw new Error("a chunk is not base64");
	return bytes;
}

function exchangeOf(held: z.infer<typeof TransportExchange>): ScriptedExchange {
	return {
		status: held.status,
		headers: held.headers,
		chunks: held.chunks.map((chunk) => ({
			bytes: bytesOf(chunk),
			...(chunk.delayMs === undefined ? {} : { delayMs: chunk.delayMs }),
		})),
		...(held.headMs === undefined ? {} : { headMs: held.headMs }),
		...(held.end === undefined ? {} : { end: held.end }),
		...(held.error === undefined ? {} : { error: held.error }),
	};
}

function keyAuthOf(held: z.infer<typeof TransportKeyAuth> | undefined): KeyAuth {
	if (held === undefined || held.kind === "stored") return "stored";
	return held.kind === "none" ? "none" : { key: held.key };
}

function slashed(path: string): `/${string}` {
	if (!path.startsWith("/")) throw new Error(`a JSON path starts with a slash: ${path}`);
	return `/${path.slice(1)}` as const;
}

////////////////////////////////
//  Group

export const transport = caseGroup("transport", {
	shapes: {
		CodecName,
		TransportChunk,
		TransportExchange,
		TransportKeyAuth,
		TransportCall,
		TransportScript,
		TransportSent,
		TransportOutcome,
	},
	kinds: {
		/** Calls in order on one transport over a scripted network and key source. */
		exchange: kind(
			z.object({
				codec: CodecName,
				/** Headers the codec adds to every request it encodes. */
				codecHeaders: z.record(z.string(), z.string()).exactOptional(),
				baseUrl: z.string(),
				label: z.string(),
				userAgent: z.string(),
				auth: z.enum(["bearer", "none", "optional"]).exactOptional(),
				idleMs: z.int().exactOptional(),
				/** The transport's cap on a reply. */
				maxBytes: z.int().exactOptional(),
				/** What each read of the key source answers, the last repeating; null is no key. */
				keys: z.array(z.string().nullable()).min(1),
				/** The wheel's first instant. */
				startMs: z.int(),
				script: TransportScript,
				calls: z.array(TransportCall).min(1),
			}),
			z.object({
				calls: z.array(TransportOutcome),
				/** Every request given to the network. */
				requests: z.array(TransportSent),
				log: z.array(z.string()),
				/** The models `onSettled` was told. */
				settled: z.array(z.string()),
				/** Times the key source was read. */
				keyReads: z.int(),
				/** Chunks handed out, per exchange. */
				read: z.array(z.int()),
				/** Timers still waiting at the end. */
				waiting: z.array(z.string()),
			}),
			async (input, context) => {
				const wheel = new TimerWheel(input.startMs);
				const played =
					input.script.kind === "inline"
						? scriptedFetch(input.script.exchanges.map(exchangeOf), wheel)
						: replayFetch(await readTrace(path.join(context.root, input.script.file)), wheel);
				const keys = fakeKeys(input.keys.map((held) => held ?? undefined));
				const log: string[] = [];
				const settled: string[] = [];
				const base = codecOf(input.codec);
				const added = input.codecHeaders;
				const codec: Codec =
					added === undefined
						? base
						: {
								...base,
								encode: (request, mode) => {
									const encoded = base.encode(request, mode);
									return { ...encoded, headers: { ...encoded.headers, ...added } };
								},
							};
				const made = new Transport({
					fetch: played.fetch,
					scheduler: fakeScheduler(wheel),
					key: keys,
					log: (line) => log.push(line),
					baseUrl: input.baseUrl,
					label: input.label,
					userAgent: input.userAgent,
					errors: base,
					...(input.auth === undefined ? {} : { auth: input.auth }),
					...(input.idleMs === undefined ? {} : { idleMs: input.idleMs }),
					...(input.maxBytes === undefined ? {} : { maxBytes: input.maxBytes }),
				});
				made.onSettled((model) => settled.push(model));
				for (const call of input.calls)
					if (call.kind === "getJson" || call.kind === "postJson") slashed(call.path);
				const calls: z.infer<typeof TransportOutcome>[] = [];
				for (const call of input.calls) {
					const caller = new AbortController();
					if (call.abortedBefore === true) caller.abort(new Error(CALLER_REASON));
					else if (call.abortAfterMs !== undefined)
						wheel.after(call.abortAfterMs, "caller", () => caller.abort(new Error(CALLER_REASON)));
					const outcome: z.infer<typeof TransportOutcome> = {
						events: [],
						result: null,
						answer: null,
						failure: null,
						thrown: null,
						aborted: false,
						nowMs: 0,
					};
					try {
						if (call.kind === "stream") {
							const events = made.stream(codec, call.request, caller.signal);
							try {
								for await (const event of events) {
									outcome.events.push(event);
									if (outcome.events.length === call.stopAfter) break;
								}
							} finally {
								await events.return(undefined);
							}
						} else if (call.kind === "complete") {
							outcome.result = await made.complete(codec, call.request, caller.signal);
						} else if (call.kind === "getJson") {
							outcome.answer = await made.getJson(
								slashed(call.path),
								caller.signal,
								keyAuthOf(call.keyAuth),
							);
						} else {
							outcome.answer = await made.postJson(
								slashed(call.path),
								call.body,
								caller.signal,
								keyAuthOf(call.keyAuth),
							);
						}
					} catch (error) {
						const held = failureOf(error);
						if (held === undefined)
							outcome.thrown = error instanceof Error ? error.message : jsString(error);
						else outcome.failure = held;
					}
					wheel.cancel("caller");
					outcome.aborted = caller.signal.aborted;
					outcome.nowMs = wheel.nowMs();
					calls.push(outcome);
				}
				return {
					calls,
					requests: played.requests.map((sent) => ({
						method: sent.method,
						url: sent.url,
						headers: sent.headers,
						body: sent.body ?? null,
					})),
					log,
					settled,
					keyReads: keys.reads(),
					read: played.read(),
					waiting: wheel.waiting(),
				};
			},
			{ context: true },
		),
		/** The shape a key must have, and the key a pasted text holds. */
		key: kind(
			z.object({ text: z.string() }),
			z.object({ shape: z.boolean(), normalized: z.string().nullable() }),
			(input) => ({ shape: isKeyShape(input.text), normalized: normalizeKey(input.text) ?? null }),
		),
	},
	sources: casesFor("transport", transportCases),
});
