// HTTP trace replay provider.

import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { base64Decode } from "@nyaaai/contract/base64";
import { parseJson } from "@nyaaai/contract/json";
import type { Trace } from "./trace";

////////////////////////////////
//  Interfaces & Types

export interface Loopback {
	/** Assigned loopback URL. */
	url: string;
	/** Parsed request bodies. */
	requests: unknown[];
	/** Request headers, lowercase names. */
	headers: Array<Record<string, string | string[] | undefined>>;
	/** Resolves at the request. */
	arrived: (at: number) => Promise<void>;
	/** Resolves at the close. */
	closed: (at: number) => Promise<void>;
	close: () => Promise<void>;
}

/** Where an exchange stops. */
export type LoopbackHold = "head" | "chunks";

////////////////////////////////
//  Functions & Helpers

function signal(): { done: Promise<void>; fire: () => void } {
	let fire = () => {};
	const done = new Promise<void>((resolve) => {
		fire = resolve;
	});
	return { done, fire };
}

/** Replays chunks; 500 when exhausted. */
export function loopbackProvider(trace: Trace, holds: readonly (LoopbackHold | undefined)[] = []): Promise<Loopback> {
	const left = trace.exchanges.map((exchange, at) => ({ exchange, at }));
	const requests: unknown[] = [];
	const headers: Loopback["headers"] = [];
	const arrivals = trace.exchanges.map(signal);
	const closes = trace.exchanges.map(signal);
	const server = createServer((request, response) => {
		const parts: Buffer[] = [];
		request.on("data", (part: Buffer) => parts.push(part));
		request.on("end", async () => {
			requests.push(parseJson(Buffer.concat(parts).toString("utf8")) ?? null);
			headers.push(request.headers);
			const next = left.shift();
			if (!next) {
				response.writeHead(500, { "content-type": "text/plain" });
				response.end("the trace holds no further exchange");
				return;
			}
			const { exchange, at } = next;
			response.on("close", () => closes[at]?.fire());
			arrivals[at]?.fire();
			if (holds[at] === "head") return;
			response.writeHead(exchange.status, exchange.headers);
			response.flushHeaders();
			for (const chunk of exchange.chunks) {
				const bytes = base64Decode(chunk) ?? new Uint8Array();
				if (bytes.length === 0) continue;
				// Writes each chunk separately.
				await new Promise<void>((resolve) => response.write(bytes, () => resolve()));
			}
			if (holds[at] !== "chunks") response.end();
		});
	});
	return new Promise((resolve, reject) => {
		server.once("error", reject);
		server.listen(0, "127.0.0.1", () => {
			const { port } = server.address() as AddressInfo;
			resolve({
				url: `http://127.0.0.1:${port}`,
				requests,
				headers,
				arrived: (at) => arrivals[at]?.done ?? Promise.reject(new Error("no such exchange")),
				closed: (at) => closes[at]?.done ?? Promise.reject(new Error("no such exchange")),
				close: () =>
					new Promise((closed) => {
						server.closeAllConnections();
						server.close(() => closed());
					}),
			});
		});
	});
}
