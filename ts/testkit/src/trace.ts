// Byte-form trace replay.

import { readFile } from "node:fs/promises";
import { base64Decode } from "@nyaaai/contract/base64";
import { decodeText, shapeSet } from "@nyaaai/contract/shape";
import { z } from "zod";
import { type FakeScheduler, fakeScheduler } from "./scheduler";
import { type ScriptedExchange, type ScriptedFetch, scriptedFetch } from "./scriptedFetch";

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
//  Functions & Helpers

export async function readTrace(file: string): Promise<Trace> {
	const held = decodeText(Trace, await readFile(file, "utf8"));
	if (!held.ok) throw new Error(`${file} is no trace at ${JSON.stringify(held.path)}: ${held.message}`);
	return held.value;
}

/** Plays the exchanges in order on the scheduler's wheel, keeping chunk edges. Nothing waits. */
export function replayFetch(trace: Trace, scheduler: FakeScheduler = fakeScheduler()): ScriptedFetch {
	const exchanges = trace.exchanges.map(
		(exchange): ScriptedExchange => ({
			status: exchange.status,
			headers: Object.entries(exchange.headers),
			chunks: exchange.chunks.map((chunk) => {
				const bytes = base64Decode(chunk);
				if (!bytes) throw new Error("a trace chunk is not base64");
				return { bytes };
			}),
		}),
	);
	return scriptedFetch(exchanges, scheduler);
}
