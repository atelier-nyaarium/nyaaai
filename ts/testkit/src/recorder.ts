// A Fetch that keeps what passed, as a trace.

import { base64Encode } from "@nyaaai/contract/base64";
import type { Fetch } from "@nyaaai/contract/fetch";
import { parseJson } from "@nyaaai/contract/json";
import { asciiLower, encodePathSegment } from "@nyaaai/contract/text";
import type { Trace } from "./trace";

////////////////////////////////
//  Interfaces & Types

export interface RecordingFetch {
	fetch: Fetch;
	/** Throws on a kept secret. */
	trace: () => Trace;
}

interface Held {
	body: string | undefined;
	status?: number;
	headers: Record<string, string>;
	chunks: Uint8Array[];
}

////////////////////////////////
//  Functions & Helpers

/** Headers a replay needs. */
export function keptHeader(name: string): boolean {
	const lower = asciiLower(name);
	return lower === "content-type" || lower === "retry-after" || lower.startsWith("x-ratelimit-");
}

/** One character per UTF-8 byte. */
function binaryOf(bytes: Uint8Array): string {
	return Array.from(bytes, (byte) => String.fromCharCode(byte)).join("");
}

const binaryText = (text: string): string => binaryOf(new TextEncoder().encode(text));

/** As is, path-segment encoded, base64 without padding. */
function formsOf(secret: string): string[] {
	const unpadded = base64Encode(new TextEncoder().encode(secret));
	let end = unpadded.length;
	while (end > 0 && unpadded[end - 1] === "=") end -= 1;
	return [secret, encodePathSegment(secret), unpadded.slice(0, end)].map(binaryText);
}

function joined(chunks: readonly Uint8Array[]): Uint8Array {
	const all = new Uint8Array(chunks.reduce((sum, chunk) => sum + chunk.length, 0));
	let at = 0;
	for (const chunk of chunks) {
		all.set(chunk, at);
		at += chunk.length;
	}
	return all;
}

/** Passes chunks, keeping copies. */
async function* passing(from: AsyncIterable<Uint8Array>, into: Uint8Array[]): AsyncGenerator<Uint8Array> {
	for await (const chunk of from) {
		into.push(chunk.slice());
		yield chunk;
	}
}

/** Keeps no request header. */
export function recordingFetch(inner: Fetch, secrets: readonly string[]): RecordingFetch {
	const held: Held[] = [];
	const forms = secrets.filter((secret) => secret.length > 0).flatMap(formsOf);
	const leaks = (text: string): boolean => forms.some((form) => text.includes(form));

	return {
		fetch: async (request, signal) => {
			const mine: Held = { body: request.body, headers: {}, chunks: [] };
			held.push(mine);
			const response = await inner(request, signal);
			mine.status = response.status;
			for (const [name, value] of response.headers) if (keptHeader(name)) mine.headers[asciiLower(name)] = value;
			return { ...response, chunks: passing(response.chunks, mine.chunks) };
		},
		trace: () => {
			const exchanges = held.map((exchange, at) => {
				const refuse = (place: string): never => {
					throw new Error(`exchange ${at + 1}: a secret occurs in ${place}`);
				};
				if (exchange.status === undefined) throw new Error(`exchange ${at + 1}: no response was kept`);
				if (exchange.body !== undefined && leaks(binaryText(exchange.body))) refuse("the request body");
				for (const [name, value] of Object.entries(exchange.headers)) {
					if (leaks(binaryText(`${name}: ${value}`))) refuse(`the response header ${name}`);
				}
				if (leaks(binaryOf(joined(exchange.chunks)))) refuse("the response body");
				return {
					request: exchange.body === undefined ? null : (parseJson(exchange.body) ?? exchange.body),
					status: exchange.status,
					headers: exchange.headers,
					chunks: exchange.chunks.map(base64Encode),
				};
			});
			return { format: 1, exchanges };
		},
	};
}
