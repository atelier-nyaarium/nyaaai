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

/**
 * Plays exchanges in order, spending wheel time, on one lane: a request holds it from its turn
 * until its body ends, fails or is cancelled (the iterator's `return`, or the signal's abort).
 * Later requests wait in arrival order and are seen (recorded, timed) at their turn; a request
 * whose signal has ended when its turn comes, or ends while it waits, is refused with its reason,
 * unrecorded and with no time spent. A body neither read to its end nor cancelled holds the lane.
 */
export function scriptedFetch(exchanges: readonly ScriptedExchange[], wheel: TimerWheel): ScriptedFetch {
	const requests: FetchRequest[] = [];
	const served = exchanges.map(() => 0);
	let next = 0;
	let busy = false;
	const line: Array<{ signal: AbortSignal; go: () => void; refuse: () => void }> = [];

	/** The lane goes to the first waiter whose signal has not ended; the others are refused unseen. */
	const hand = () => {
		for (;;) {
			const waiter = line.shift();
			if (waiter === undefined) {
				busy = false;
				return;
			}
			if (waiter.signal.aborted) {
				waiter.refuse();
				continue;
			}
			waiter.go();
			return;
		}
	};

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

	/** The body, freeing the lane once at its end, failure or cancel. */
	function held(inner: AsyncGenerator<Uint8Array>, signal: AbortSignal): AsyncIterable<Uint8Array> {
		let freed = false;
		const free = () => {
			if (freed) return;
			freed = true;
			signal.removeEventListener("abort", free);
			hand();
		};
		signal.addEventListener("abort", free, { once: true });
		return {
			[Symbol.asyncIterator]: () => ({
				next: async () => {
					try {
						const step = await inner.next();
						if (step.done === true) free();
						return step;
					} catch (error) {
						free();
						throw error;
					}
				},
				return: async () => {
					free();
					await inner.return(undefined);
					return { done: true, value: undefined };
				},
			}),
		};
	}

	async function wait(signal: AbortSignal): Promise<void> {
		await new Promise<void>((resolve, reject) => {
			if (signal.aborted) return reject(signal.reason);
			const waiter = {
				signal,
				go: () => {
					signal.removeEventListener("abort", leave);
					resolve();
				},
				refuse: () => {
					signal.removeEventListener("abort", leave);
					reject(signal.reason);
				},
			};
			const leave = () => {
				const at = line.indexOf(waiter);
				if (at >= 0) line.splice(at, 1);
				reject(signal.reason);
			};
			line.push(waiter);
			signal.addEventListener("abort", leave, { once: true });
		});
	}

	return {
		requests,
		read: () => [...served],
		fetch: async (request, signal) => {
			if (busy) await wait(signal);
			else busy = true;
			try {
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
					chunks: held(body(exchange, at, signal), signal),
				};
			} catch (error) {
				hand();
				throw error;
			}
		},
	};
}
