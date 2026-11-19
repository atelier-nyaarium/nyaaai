// Scrubs recorded traces. Run: bun scripts/convertTrace.ts <trace.jsonl> <out.json>

import { readFileSync, writeFileSync } from "node:fs";
import { base64Encode } from "@nyaaai/contract/base64";
import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import { keptHeader } from "@nyaaai/testkit/recorder";
import type { Trace, TraceExchange } from "@nyaaai/testkit/trace";
import { collect, dataOf, longestFirst, scrubberOf } from "./traceIds";

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
//  Functions & Helpers

function sessionIds(rows: OldRow[]): string[] {
	const ids = new Set<string>();
	for (const { source, data } of rows) {
		if (source !== "http") continue;
		collect(data.request, ids);
		collect(data.body, ids);
		if (data.line !== undefined) collect(dataOf(data.line)?.value, ids);
	}
	return longestFirst(ids);
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
			last.headers = Object.fromEntries(Object.entries(data.headers ?? {}).filter(([name]) => keptHeader(name)));
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
