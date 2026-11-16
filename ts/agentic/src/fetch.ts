// The Fetch port over the platform's fetch.

import type { Fetch } from "@nyaaai/contract/fetch";
import { responseHeaders } from "@nyaaai/contract/headers";

////////////////////////////////
//  Functions & Helpers

/** Reads under the signal. */
async function* chunksOf(body: ReadableStream<Uint8Array> | null, signal: AbortSignal): AsyncGenerator<Uint8Array> {
	if (body === null) return;
	const reader = body.getReader();
	let stop: (() => void) | undefined;
	const aborted = new Promise<never>((_, reject) => {
		stop = () => reject(signal.reason);
		signal.addEventListener("abort", stop, { once: true });
	});
	aborted.catch(() => undefined);
	try {
		for (;;) {
			if (signal.aborted) throw signal.reason;
			const held = await Promise.race([reader.read(), aborted]).catch((error: unknown) => {
				throw signal.aborted ? signal.reason : error;
			});
			if (held.done) return;
			yield held.value;
		}
	} finally {
		if (stop) signal.removeEventListener("abort", stop);
		await reader.cancel().catch(() => undefined);
	}
}

/** The platform's fetch as a port. */
export function platformFetch(platform: typeof fetch = fetch): Fetch {
	return async (request, signal) => {
		if (signal.aborted) throw signal.reason;
		const response = await platform(request.url, {
			method: request.method,
			headers: request.headers.map(([name, value]) => [name, value]),
			...(request.method === "GET" || request.body === undefined ? {} : { body: request.body }),
			redirect: "manual",
			signal,
		}).catch((error: unknown) => {
			throw signal.aborted ? signal.reason : error;
		});
		return {
			status: response.status,
			headers: responseHeaders([...response.headers]),
			chunks: chunksOf(response.body, signal),
		};
	};
}
