// A Fetch that plays a script on virtual time.

import type { Fetch, FetchRequest } from "@nyaaai/contract/fetch";
import { responseHeaders } from "@nyaaai/contract/headers";
import type { HeaderList } from "@nyaaai/contract/shapes/step";
import type { TimerWheel } from "./scheduler";

////////////////////////////////
//  Interfaces & Types

export interface ScriptedChunk {
	bytes: Uint8Array;
	/** Wheel time spent first. */
	delayMs?: number;
}

export interface ScriptedExchange {
	status: number;
	/** Raw pairs; answered in `responseHeaders` form. */
	headers: HeaderList;
	chunks: ScriptedChunk[];
	/** Wheel time spent first. */
	headMs?: number;
	/** Default `closed`; `dropped` fails the last read. */
	end?: "closed" | "dropped" | "refused";
	/** Message of a drop or refusal. */
	error?: string;
}

export interface ScriptedFetch {
	fetch: Fetch;
	/** Every request given, in order. */
	requests: FetchRequest[];
	/** Chunks handed out, per exchange of the script. */
	read: () => number[];
}

////////////////////////////////
//  Functions & Helpers

/** Plays exchanges in order, spending wheel time. */
export function scriptedFetch(exchanges: readonly ScriptedExchange[], wheel: TimerWheel): ScriptedFetch {
	const requests: FetchRequest[] = [];
	const served = exchanges.map(() => 0);
	let next = 0;

	async function* body(exchange: ScriptedExchange, at: number, signal: AbortSignal): AsyncGenerator<Uint8Array> {
		for (const chunk of exchange.chunks) {
			wheel.advance(chunk.delayMs ?? 0);
			if (signal.aborted) throw signal.reason;
			served[at] = (served[at] ?? 0) + 1;
			yield chunk.bytes;
		}
		if (signal.aborted) throw signal.reason;
		if (exchange.end === "dropped") throw new Error(exchange.error ?? "connection dropped");
	}

	return {
		requests,
		read: () => [...served],
		fetch: async (request, signal) => {
			requests.push(request);
			if (signal.aborted) throw signal.reason;
			const at = next;
			const exchange = exchanges[at];
			if (exchange === undefined) throw new Error("the script holds no further exchange");
			next += 1;
			wheel.advance(exchange.headMs ?? 0);
			if (signal.aborted) throw signal.reason;
			if (exchange.end === "refused") throw new Error(exchange.error ?? "connection refused");
			return {
				status: exchange.status,
				headers: responseHeaders(exchange.headers),
				chunks: body(exchange, at, signal),
			};
		},
	};
}
