// Scrubs recorded traces.
// Run: bun scripts/convertTrace.ts <trace.jsonl> <out.json>

import { readFileSync, writeFileSync } from "node:fs";
import { base64Encode } from "@nyaaai/contract/base64";
import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import { Pattern } from "@nyaaai/contract/pattern";
import type { Trace, TraceExchange } from "@nyaaai/testkit/trace";

////////////////////////////////
//  Interfaces & Types

interface OldRow {
	source: string;
	data: {
		request?: unknown;
		status?: number;
		headers?: Record<string, string>;
		line?: string;
		body?: unknown;
	};
}

////////////////////////////////
//  Constants

/** Keep only replay headers. */
const KEPT_HEADER = Pattern.compile("^(?:content-type|retry-after|x-ratelimit-.*)$");
const GENERATION = Pattern.compile("gen-[0-9]+-[A-Za-z0-9]+");
const SESSION_KEY = "session_id";
const SESSION = "00000000-0000-4000-8000-000000000000";
const DATA_PREFIX = "data:";

////////////////////////////////
//  Functions & Helpers

/** The JSON after `data:` and the prefix before it; undefined when the line holds none. */
function dataOf(line: string): { prefix: string; payload: string; value: unknown } | undefined {
	if (!line.startsWith(DATA_PREFIX)) return undefined;
	const gap = line.startsWith(" ", DATA_PREFIX.length) ? 1 : 0;
	const prefix = line.slice(0, DATA_PREFIX.length + gap);
	const payload = line.slice(prefix.length);
	const value = parseJson(payload);
	return value === undefined ? undefined : { prefix, payload, value };
}

/** Every string found under a session key. */
function collect(value: unknown, ids: Set<string>): void {
	if (Array.isArray(value)) for (const item of value) collect(item, ids);
	else if (value !== null && typeof value === "object") {
		for (const [key, held] of Object.entries(value)) {
			if (key === SESSION_KEY && typeof held === "string" && held !== "") ids.add(held);
			else collect(held, ids);
		}
	}
}

function sessionIds(rows: OldRow[]): string[] {
	const ids = new Set<string>();
	for (const { source, data } of rows) {
		if (source !== "http") continue;
		collect(data.request, ids);
		collect(data.body, ids);
		if (data.line !== undefined) collect(dataOf(data.line)?.value, ids);
	}
	// Longest first, so one id inside another is replaced whole.
	return [...ids].sort((left, right) => right.length - left.length);
}

/** Scrubs by known id, by key, and every string. */
function scrubberOf(ids: string[]) {
	const text = (held: string): string => {
		let made = held;
		for (const id of ids) made = made.split(id).join(SESSION);
		let from = 0;
		let out = "";
		for (const found of GENERATION.findAll(made)) {
			out += `${made.slice(from, found.start)}gen-0-recorded`;
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
	return { value, line };
}

const bytesOf = (text: string): string => base64Encode(new TextEncoder().encode(text));

/** Converts rows lacking chunk boundaries. */
export function convertTrace(rows: OldRow[]): Trace {
	const scrub = scrubberOf(sessionIds(rows));
	const exchanges: TraceExchange[] = [];
	let lines: string[] = [];
	const close = () => {
		const last = exchanges.at(-1);
		if (last && lines.length > 0) last.chunks.push(bytesOf(lines.map((line) => `${line}\n\n`).join("")));
		lines = [];
	};
	for (const { source, data } of rows) {
		if (source !== "http") continue;
		const last = exchanges.at(-1);
		if (data.request !== undefined) {
			close();
			exchanges.push({
				request: scrub.value(data.request),
				status: 0,
				headers: {},
				chunks: [],
			});
		} else if (!last) {
			throw new Error("a response row comes before any request");
		} else if (data.status !== undefined) {
			last.status = data.status;
			last.headers = Object.fromEntries(
				Object.entries(data.headers ?? {}).filter(([name]) => KEPT_HEADER.test(name)),
			);
		} else if (data.line !== undefined) {
			lines.push(scrub.line(data.line));
		} else if (data.body !== undefined) {
			last.chunks.push(bytesOf(stringifyJson(scrub.value(data.body))));
		}
	}
	close();
	return { format: 1, exchanges };
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const [from, to] = process.argv.slice(2);
	if (!from || !to) {
		console.error("usage: bun scripts/convertTrace.ts <trace.jsonl> <out.json>");
		process.exit(1);
	}
	const rows = readFileSync(from, "utf8")
		.split("\n")
		.filter((line) => line.length > 0)
		.map((line) => {
			const row = parseJson(line);
			if (row === undefined) throw new Error("a trace line is no JSON");
			return row as OldRow;
		});
	const trace = convertTrace(rows);
	writeFileSync(to, `${JSON.stringify(trace, null, "\t")}\n`);
	console.log(`trace: ${trace.exchanges.length} exchanges written`);
}
