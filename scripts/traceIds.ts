// The id rules shared by the trace converter and the recorder.

import { base64Decode, base64Encode } from "@nyaaai/contract/base64";
import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import { Pattern } from "@nyaaai/contract/pattern";
import type { Trace } from "@nyaaai/testkit/trace";

////////////////////////////////
//  Constants

const GENERATION = Pattern.compile("gen-[0-9]+-[A-Za-z0-9]+");
const GENERATION_STAND_IN = "gen-0-recorded";
export const SESSION_KEY = "session_id";
export const SESSION = "00000000-0000-4000-8000-000000000000";
const DATA_PREFIX = "data:";

////////////////////////////////
//  Functions & Helpers

/** The JSON after `data:`. */
export function dataOf(line: string): { prefix: string; payload: string; value: unknown } | undefined {
	if (!line.startsWith(DATA_PREFIX)) return undefined;
	const gap = line.startsWith(" ", DATA_PREFIX.length) ? 1 : 0;
	const prefix = line.slice(0, DATA_PREFIX.length + gap);
	const payload = line.slice(prefix.length);
	const value = parseJson(payload);
	return value === undefined ? undefined : { prefix, payload, value };
}

/** Every string found under a session key. */
export function collect(value: unknown, ids: Set<string>): void {
	if (Array.isArray(value)) for (const item of value) collect(item, ids);
	else if (value !== null && typeof value === "object") {
		for (const [key, held] of Object.entries(value)) {
			if (key === SESSION_KEY && typeof held === "string" && held !== "") ids.add(held);
			else collect(held, ids);
		}
	}
}

/** Longest first. */
export function longestFirst(ids: ReadonlySet<string>): string[] {
	return [...ids].sort((left, right) => right.length - left.length);
}

/** Scrubs by known id, by key, and every string. */
export function scrubberOf(ids: readonly string[]) {
	const text = (held: string): string => {
		let made = held;
		for (const id of ids) made = made.split(id).join(SESSION);
		let from = 0;
		let out = "";
		for (const found of GENERATION.findAll(made)) {
			out += `${made.slice(from, found.start)}${GENERATION_STAND_IN}`;
			from = found.end;
		}
		return out + made.slice(from);
	};
	const value = (held: unknown): unknown => {
		if (typeof held === "string") return text(held);
		if (Array.isArray(held)) return held.map(value);
		if (held !== null && typeof held === "object") {
			return Object.fromEntries(
				Object.entries(held).map(([key, item]) => [
					text(key),
					key === SESSION_KEY && typeof item === "string" ? SESSION : value(item),
				]),
			);
		}
		return held;
	};
	/** A stream line: JSON after `data:` is parsed, anything else is one string. */
	const line = (held: string): string => {
		const data = dataOf(held);
		return data === undefined ? text(held) : data.prefix + stringifyJson(value(data.value));
	};
	return { text, value, line };
}

/** One character per byte. */
function binaryOf(bytes: Uint8Array): string {
	return Array.from(bytes, (byte) => String.fromCharCode(byte)).join("");
}

function bytesOfBinary(text: string): Uint8Array {
	return Uint8Array.from(text, (char) => char.charCodeAt(0));
}

function chunkBytes(chunk: string): Uint8Array {
	const bytes = base64Decode(chunk);
	if (!bytes) throw new Error("a trace chunk is not base64");
	return bytes;
}

/** Session ids in a body. */
function bodySessionIds(joined: Uint8Array, ids: Set<string>): void {
	const text = new TextDecoder().decode(joined);
	const whole = parseJson(text);
	if (whole !== undefined) {
		collect(whole, ids);
		return;
	}
	for (const line of text.split("\n")) collect(dataOf(line)?.value, ids);
}

/** Chunk spans in the joined body. */
function edgesOf(sizes: readonly number[]): Array<[number, number]> {
	let at = 0;
	return sizes.map((size): [number, number] => {
		const held: [number, number] = [at, at + size];
		at += size;
		return held;
	});
}

function crossesEdge(edges: ReadonlyArray<[number, number]>, start: number, end: number): boolean {
	return !edges.some(([from, to]) => start >= from && end <= to);
}

/** Replaces ids in place; an id across an edge throws. */
export function replaceTraceIds(trace: Trace): Trace {
	const found = new Set<string>();
	const joined = trace.exchanges.map((exchange) => {
		collect(exchange.request, found);
		const bytes = exchange.chunks.map(chunkBytes);
		const all = new Uint8Array(bytes.reduce((sum, part) => sum + part.length, 0));
		let at = 0;
		for (const part of bytes) {
			all.set(part, at);
			at += part.length;
		}
		bodySessionIds(all, found);
		return { bytes, text: binaryOf(all) };
	});
	const ids = longestFirst(found);
	const plain = scrubberOf(ids);
	const binary = scrubberOf(ids.map((id) => binaryOf(new TextEncoder().encode(id))));
	const exchanges = trace.exchanges.map((exchange, at) => {
		const body = joined[at] as (typeof joined)[number];
		const edges = edgesOf(body.bytes.map((part) => part.length));
		const spans: Array<[number, number]> = [];
		for (const id of ids) {
			const wanted = binaryOf(new TextEncoder().encode(id));
			for (let from = body.text.indexOf(wanted); from !== -1; from = body.text.indexOf(wanted, from + 1))
				spans.push([from, from + wanted.length]);
		}
		for (const match of GENERATION.findAll(body.text)) spans.push([match.start, match.end]);
		if (spans.some(([start, end]) => crossesEdge(edges, start, end)))
			throw new Error(`exchange ${at + 1}: an id crosses a chunk edge, record again`);
		return {
			...exchange,
			request: plain.value(exchange.request),
			chunks: body.bytes.map((part) => base64Encode(bytesOfBinary(binary.text(binaryOf(part))))),
		};
	});
	return { format: 1, exchanges };
}
