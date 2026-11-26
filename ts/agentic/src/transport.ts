// A codec's calls over the Fetch port.

import {
	BUDGETS,
	BudgetExceeded,
	type Budgets,
	type Fetch,
	type FetchResponse,
	type Span,
	span,
} from "@nyaaai/contract/fetch";
import { headerOf } from "@nyaaai/contract/headers";
import { jsString, parseJson, stringifyJson } from "@nyaaai/contract/json";
import { scrubSecret } from "@nyaaai/contract/secret";
import type {
	CanonicalRequest,
	Encoded,
	Failure,
	HeaderList,
	Serving,
	SseItem,
	StepEvent,
	StepResult,
} from "@nyaaai/contract/shapes/step";
import { asciiLower, trimText, Utf8Decoder } from "@nyaaai/contract/text";
import { parseRetryAfter, type Scheduler } from "@nyaaai/contract/time";
import type { Codec } from "./codec";
import { failed, ProviderError } from "./failure";
import { SseDecoder } from "./sse";

////////////////////////////////
//  Interfaces & Types

export interface KeySource {
	get(): Promise<string | undefined>;
}

export interface TransportDeps {
	fetch: Fetch;
	/** Spans; dates `Retry-After`. */
	scheduler: Scheduler;
	key: KeySource;
	/** Status, generation, host. */
	log: (line: string) => void;
	baseUrl: string;
	/** Provider error/log label. */
	label: string;
	/** The consumer's `name/version`. */
	userAgent: string;
	/** Decodes failed reads. */
	errors: Pick<Codec, "failure">;
	/** `optional`: key auth only when a key exists. */
	auth?: "bearer" | "none" | "optional";
	/** Default `BUDGETS`; `stream` and `reply` bound each call. */
	budgets?: Budgets | undefined;
}

export type KeyAuth = "none" | "stored" | { key: string };

interface Seen {
	status?: number;
	generation?: string | undefined;
	host?: string | undefined;
}

////////////////////////////////
//  Constants

/** Cuts what the budget let through. */
const ERROR_BODY_BYTES = 64 * 1024;
/** The names a codec cannot set. */
const OWNED_HEADERS = ["content-type", "accept", "authorization", "user-agent"];
const TOKEN_SYMBOLS = "!#$%&'*+-.^_`|~";

////////////////////////////////
//  Classes

/** Scrubbed codec transport. */
export class Transport {
	private readonly budgets: Budgets;
	private readonly settled = new Set<(model: string) => void>();

	constructor(private readonly d: TransportDeps) {
		this.budgets = d.budgets ?? BUDGETS;
	}

	/** After each unaborted reply. */
	onSettled(listener: (model: string) => void): () => boolean {
		this.settled.add(listener);
		return () => this.settled.delete(listener);
	}

	/** One `stream` span per call, held until closed. */
	async *stream(codec: Codec, request: CanonicalRequest, signal: AbortSignal): AsyncGenerator<StepEvent> {
		if (signal.aborted) throw signal.reason;
		const encoded = encodedOf(codec, request, "stream");
		this.checkHeaders(encoded);
		const clock = span(this.d.scheduler, this.budgets.stream, signal);
		try {
			const key = await this.requestKey();
			if (signal.aborted) throw signal.reason;
			const seen: Seen = {};
			try {
				const response = await this.post(codec, encoded, key, clock, "text/event-stream", seen);
				const body = response.chunks[Symbol.asyncIterator]();
				try {
					const open: AsyncIterable<Uint8Array> = { [Symbol.asyncIterator]: () => body };
					for await (const event of codec.stream(items(open))) {
						if (event.kind !== "serving") {
							yield event;
							continue;
						}
						const serving = scrubbedServing(event.serving, key);
						seen.generation ??= serving.generationId;
						seen.host ??= serving.host;
						yield { kind: "serving", serving };
					}
				} finally {
					// A body the codec never pulled is still cancelled.
					await body.return?.();
				}
			} catch (error) {
				if (!signal.aborted) this.notify(request.model);
				throw this.mapped(error, signal, key);
			} finally {
				this.log(seen);
			}
			this.notify(request.model);
		} finally {
			clock.clear();
		}
	}

	/** One `reply` span per call. */
	async complete(codec: Codec, request: CanonicalRequest, signal: AbortSignal): Promise<StepResult> {
		if (signal.aborted) throw signal.reason;
		const encoded = encodedOf(codec, request, "complete");
		this.checkHeaders(encoded);
		const clock = span(this.d.scheduler, this.budgets.reply, signal);
		try {
			const key = await this.requestKey();
			if (signal.aborted) throw signal.reason;
			const seen: Seen = {};
			let result: StepResult;
			try {
				const response = await this.post(codec, encoded, key, clock, "application/json", seen);
				const body = parseJson(await readAll(response.chunks));
				if (body === undefined) throw failed("stream", "The reply was not JSON.");
				const decoded = codec.complete(body);
				const serving = scrubbedServing(decoded.serving, key);
				seen.generation = serving.generationId;
				seen.host = serving.host;
				result = { ...decoded, serving };
			} catch (error) {
				if (!signal.aborted) this.notify(request.model);
				throw this.mapped(error, signal, key);
			} finally {
				this.log(seen);
			}
			this.notify(request.model);
			return result;
		} finally {
			clock.clear();
		}
	}

	/** The span's owner clears it. */
	async getJson(path: `/${string}`, clock: Span, auth: KeyAuth): Promise<unknown> {
		return this.json("GET", path, undefined, clock, auth);
	}

	async postJson(path: `/${string}`, body: unknown, clock: Span, auth: KeyAuth): Promise<unknown> {
		return this.json("POST", path, body, clock, auth);
	}

	/** The key first, then the budget. */
	private async json(
		method: "GET" | "POST",
		path: `/${string}`,
		body: unknown,
		clock: Span,
		auth: KeyAuth,
	): Promise<unknown> {
		const signal = clock.caller;
		if (signal.aborted) throw signal.reason;
		checkedHeaders([["User-Agent", this.d.userAgent]]);
		const key = await this.keyFor(auth);
		if (signal.aborted) throw signal.reason;
		const headers: HeaderList = [
			["Accept", "application/json"],
			["User-Agent", this.d.userAgent],
			...(key === undefined ? [] : [["Authorization", `Bearer ${key}`] as [string, string]]),
			...(body === undefined ? [] : [["Content-Type", "application/json"] as [string, string]]),
		];
		try {
			const response = await this.d.fetch(
				{
					method,
					url: `${this.d.baseUrl}${path}`,
					headers,
					...(body === undefined ? {} : { body: stringifyJson(body) }),
					budget: clock.budget(),
				},
				signal,
			);
			if (!ok(response)) throw await this.failure(this.d.errors, response);
			const answer = parseJson(await readAll(response.chunks));
			if (answer === undefined) throw failed("network", `${path} did not answer JSON.`);
			return answer;
		} catch (error) {
			throw this.mapped(error, signal, key);
		}
	}

	/** The one exit; the first cause stands. */
	private mapped(error: unknown, signal: AbortSignal, key: string | undefined): unknown {
		const { label } = this.d;
		if (error instanceof BudgetExceeded) {
			if (error.bound === "idle") return failed("idle", `${label} sent nothing for too long.`);
			if (error.bound === "total") return failed("idle", `${label} did not finish in time.`);
			return failed("network", `The reply is larger than ${error.limit} bytes.`);
		}
		if (signal.aborted) return signal.reason ?? error;
		if (error instanceof ProviderError) return new ProviderError(scrubbedFailure(error.failure, key));
		return failed("network", scrubbed(error instanceof Error ? error.message : jsString(error), key));
	}

	private notify(model: string): void {
		for (const listener of this.settled) {
			try {
				listener(model);
			} catch {
				// A listener cannot change a call's outcome.
			}
		}
	}

	private requestKey(): Promise<string | undefined> {
		return this.keyFor("stored");
	}

	private async keyFor(auth: KeyAuth): Promise<string | undefined> {
		if (auth === "none" || this.d.auth === "none") return undefined;
		const key = auth === "stored" ? await this.d.key.get() : auth.key;
		if (this.d.auth === "optional" && auth === "stored" && !key) return undefined;
		return this.checked(key);
	}

	private checked(key: string | undefined): string {
		if (key === undefined || key.length === 0) throw failed("keyMissing", `No ${this.d.label} key is set.`);
		if (!isKeyShape(key))
			throw new ProviderError({
				category: "keyRejected",
				status: 401,
				message: `The ${this.d.label} key is malformed.`,
			});
		return key;
	}

	private async post(
		codec: Codec,
		encoded: Encoded,
		key: string | undefined,
		clock: Span,
		accept: string,
		seen: Seen,
	): Promise<FetchResponse> {
		const response = await this.d.fetch(
			{
				method: "POST",
				url: `${this.d.baseUrl}${encoded.path}`,
				headers: [
					...codecHeaders(encoded),
					["Content-Type", "application/json"],
					["Accept", accept],
					...(key === undefined ? [] : [["Authorization", `Bearer ${key}`] as [string, string]]),
					["User-Agent", this.d.userAgent],
				],
				body: stringifyJson(encoded.body),
				budget: clock.budget(),
			},
			clock.caller,
		);
		seen.status = response.status;
		if (!ok(response)) throw await this.failure(codec, response);
		return response;
	}

	private async failure(codec: Pick<Codec, "failure">, response: FetchResponse): Promise<ProviderError> {
		const text = await readText(response.chunks, ERROR_BODY_BYTES);
		const decoded = codec.failure(response.status, parseJson(text), response.headers);
		const retryAfterMs =
			decoded.retryAfterMs ??
			parseRetryAfter(headerOf(response.headers, "retry-after"), this.d.scheduler.nowMs());
		return new ProviderError({ ...decoded, ...(retryAfterMs === undefined ? {} : { retryAfterMs }) });
	}

	private checkHeaders(encoded: Encoded): void {
		checkedHeaders([...codecHeaders(encoded), ["User-Agent", this.d.userAgent]]);
	}

	private log(seen: Seen): void {
		try {
			this.d.log(`${this.d.label} ${seen.status ?? "-"} ${seen.generation ?? "-"} ${seen.host ?? "-"}`);
		} catch {
			// A log function cannot change a call's outcome.
		}
	}
}

////////////////////////////////
//  Functions & Helpers

async function* items(chunks: AsyncIterable<Uint8Array>): AsyncGenerator<SseItem> {
	const decoder = new SseDecoder();
	for await (const chunk of chunks) yield* decoder.push(chunk);
	yield* decoder.end();
}

/** Printable ASCII, excluding spaces. */
export function isKeyShape(key: string): boolean {
	if (key.length === 0) return false;
	for (let at = 0; at < key.length; at++) {
		const code = key.charCodeAt(at);
		if (code < 0x21 || code > 0x7e) return false;
	}
	return true;
}

/** Trimmed key, or undefined. */
export function normalizeKey(input: string): string | undefined {
	const key = trimText(input);
	return isKeyShape(key) ? key : undefined;
}

export function staticKey(key: string): KeySource {
	return { get: async () => key };
}

const ok = (response: FetchResponse): boolean => response.status >= 200 && response.status <= 299;

/** Codec refusals stay local. */
function encodedOf(codec: Codec, request: CanonicalRequest, mode: "stream" | "complete"): Encoded {
	try {
		return codec.encode(request, mode);
	} catch (error) {
		throw failed("badRequest", error instanceof Error ? error.message : jsString(error));
	}
}

/** The codec's headers, less the names we own. */
function codecHeaders(encoded: Encoded): Array<[string, string]> {
	return Object.entries(encoded.headers ?? {}).filter(([name]) => !OWNED_HEADERS.includes(asciiLower(name)));
}

function isToken(name: string): boolean {
	if (name.length === 0) return false;
	for (let at = 0; at < name.length; at++) {
		const code = name.charCodeAt(at);
		const alnum =
			(code >= 0x30 && code <= 0x39) || (code >= 0x41 && code <= 0x5a) || (code >= 0x61 && code <= 0x7a);
		if (!alnum && !TOKEN_SYMBOLS.includes(name.charAt(at))) return false;
	}
	return true;
}

/** Visible ASCII, space and tab. */
function isFieldValue(value: string): boolean {
	for (let at = 0; at < value.length; at++) {
		const code = value.charCodeAt(at);
		if (code !== 0x09 && (code < 0x20 || code > 0x7e)) return false;
	}
	return true;
}

/** Refuses a forbidden header, naming it but never its value. */
function checkedHeaders(headers: ReadonlyArray<readonly [string, string]>): void {
	for (const [name, value] of headers) {
		if (!isToken(name))
			throw failed("badRequest", `The request header ${stringifyJson(name)} has a name that is not valid.`);
		if (!isFieldValue(value))
			throw failed("badRequest", `The request header ${name} has a value that is not valid.`);
	}
}

async function readAll(chunks: AsyncIterable<Uint8Array>): Promise<string> {
	const decoder = new Utf8Decoder();
	let text = "";
	for await (const chunk of chunks) text += decoder.push(chunk);
	return text + decoder.end();
}

/** Decodes to the limit, then stops reading. */
async function readText(chunks: AsyncIterable<Uint8Array>, limit: number): Promise<string> {
	const decoder = new Utf8Decoder();
	let text = "";
	let size = 0;
	for await (const chunk of chunks) {
		text += decoder.push(chunk.subarray(0, limit - size));
		size += chunk.byteLength;
		if (size >= limit) break;
	}
	return text + decoder.end();
}

/** Scrub URL/base64 echoes. */
function scrubbed(text: string, key: string | undefined): string {
	return key === undefined ? text : scrubSecret(text, key);
}

/** Sanitize response-derived names. */
function scrubbedServing(serving: Serving, key: string | undefined): Serving {
	return {
		...(serving.host === undefined ? {} : { host: scrubbed(serving.host, key) }),
		...(serving.generationId === undefined ? {} : { generationId: scrubbed(serving.generationId, key) }),
	};
}

function scrubbedFailure(failure: Failure, key: string | undefined): Failure {
	return {
		...failure,
		message: scrubbed(failure.message, key),
		...(failure.reasons === undefined ? {} : { reasons: failure.reasons.map((reason) => scrubbed(reason, key)) }),
		...(failure.billed === undefined
			? {}
			: { billed: { ...failure.billed, ...scrubbedServing(failure.billed, key) } }),
	};
}
