// Byte-form trace replay.

import { readFile } from "node:fs/promises";
import { base64Decode } from "@nyaaai/contract/base64";
import { parseJson } from "@nyaaai/contract/json";
import { decodeText, shapeSet } from "@nyaaai/contract/shape";
import { z } from "zod";

////////////////////////////////
//  Shapes

export const TraceExchange = z.object({
	request: z.unknown(),
	status: z.int().min(100).max(599),
	headers: z.record(z.string(), z.string()),
	/** Ordered base64 body chunks. */
	chunks: z.array(z.string()),
});
export type TraceExchange = z.infer<typeof TraceExchange>;

export const Trace = z.object({
	format: z.literal(1),
	exchanges: z.array(TraceExchange),
});
export type Trace = z.infer<typeof Trace>;

export const traceSet = shapeSet("trace", { TraceExchange, Trace });

////////////////////////////////
//  Interfaces & Types

export interface ReplayTransport {
	/** Returns the next exchange. */
	fetch: (input: string | URL, init?: { body?: string }) => Promise<Response>;
	/** Parsed request bodies. */
	requests: unknown[];
}

////////////////////////////////
//  Functions & Helpers

export async function readTrace(file: string): Promise<Trace> {
	const held = decodeText(Trace, await readFile(file, "utf8"));
	if (!held.ok) throw new Error(`${file} is no trace at ${JSON.stringify(held.path)}: ${held.message}`);
	return held.value;
}

/** Replay preserves body chunks. */
export function replayTransport(trace: Trace): ReplayTransport {
	const left = [...trace.exchanges];
	const requests: unknown[] = [];
	return {
		requests,
		fetch: async (_input, init) => {
			const exchange = left.shift();
			if (!exchange) throw new Error("the trace holds no further exchange");
			requests.push(init?.body === undefined ? null : (parseJson(init.body) ?? null));
			const chunks = exchange.chunks.map((chunk) => {
				const bytes = base64Decode(chunk);
				if (!bytes) throw new Error("a trace chunk is not base64");
				return bytes;
			});
			const body = new ReadableStream<Uint8Array>({
				pull(controller) {
					const chunk = chunks.shift();
					if (chunk) controller.enqueue(chunk);
					else controller.close();
				},
			});
			return new Response(body, { status: exchange.status, headers: exchange.headers });
		},
	};
}
