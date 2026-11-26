// A Fetch that plays a script on virtual time.

import { budgeted, type Fetch, type FetchRequest, type RawFetch } from "@nyaaai/contract/fetch";
import { responseHeaders } from "@nyaaai/contract/headers";
import type { HeaderList } from "@nyaaai/contract/shapes/step";
import type { FakeScheduler } from "./scheduler";

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
	/** Budgeted on the scheduler's wheel. */
	fetch: Fetch;
	/** Every request given, in order. */
	requests: FetchRequest[];
	/** Chunks handed out, per exchange of the script. */
	read: () => number[];
	/** Requests that found the lane held and had to wait for it. */
	waited: () => number;
}

////////////////////////////////
//  Functions & Helpers

/** Plays exchanges in order on one lane; a body neither read nor cancelled holds it. */
export function scriptedFetch(exchanges: readonly ScriptedExchange[], scheduler: FakeScheduler): ScriptedFetch {
	const { wheel } = scheduler;
	const requests: FetchRequest[] = [];
	const served = exchanges.map(() => 0);
	let next = 0;
	let busy = false;
	let waits = 0;
	const line: Array<{ signal: AbortSignal; go: () => void; refuse: () => void }> = [];

	/** The lane goes to the first live waiter; the others are refused unseen. */
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

	const raw: RawFetch = async (request, signal) => {
		if (busy) {
			waits += 1;
			await wait(signal);
		} else busy = true;
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
	};

	return {
		requests,
		read: () => [...served],
		waited: () => waits,
		fetch: budgeted(raw, scheduler),
	};
}
