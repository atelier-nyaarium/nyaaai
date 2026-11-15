// Scrubs recorded traces.
// Run: bun scripts/convertTrace.ts <trace.jsonl> <out.json>

import { readFileSync, writeFileSync } from "node:fs";
import { base64Encode } from "@nyaaai/contract/base64";
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
const KEPT_HEADER = /^(?:content-type|retry-after|x-ratelimit-.*)$/;
const GENERATION = /gen-\d+-[A-Za-z0-9]+/g;
const SESSION = /"session_id":"[^"]*"/g;

////////////////////////////////
//  Functions & Helpers

const scrubbed = (text: string): string =>
	text.replace(GENERATION, "gen-0-recorded").replace(SESSION, '"session_id":"00000000-0000-4000-8000-000000000000"');

const bytesOf = (text: string): string => base64Encode(new TextEncoder().encode(text));

/** Old traces lack chunk boundaries. */
export function convertTrace(rows: OldRow[]): Trace {
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
				request: JSON.parse(scrubbed(JSON.stringify(data.request))),
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
			lines.push(scrubbed(data.line));
		} else if (data.body !== undefined) {
			last.chunks.push(bytesOf(scrubbed(JSON.stringify(data.body))));
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
		.map((line) => JSON.parse(line) as OldRow);
	const trace = convertTrace(rows);
	writeFileSync(to, `${JSON.stringify(trace, null, "\t")}\n`);
	console.log(`trace: ${trace.exchanges.length} exchanges written`);
}
