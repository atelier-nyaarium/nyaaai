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
	close: () => Promise<void>;
}

////////////////////////////////
//  Functions & Helpers

/** Replays chunks; returns 500 when exhausted. */
export function loopbackProvider(trace: Trace): Promise<Loopback> {
	const left = [...trace.exchanges];
	const requests: unknown[] = [];
	const server = createServer((request, response) => {
		const parts: Buffer[] = [];
		request.on("data", (part: Buffer) => parts.push(part));
		request.on("end", async () => {
			requests.push(parseJson(Buffer.concat(parts).toString("utf8")) ?? null);
			const exchange = left.shift();
			if (!exchange) {
				response.writeHead(500, { "content-type": "text/plain" });
				response.end("the trace holds no further exchange");
				return;
			}
			response.writeHead(exchange.status, exchange.headers);
			for (const chunk of exchange.chunks) {
				const bytes = base64Decode(chunk) ?? new Uint8Array();
				if (bytes.length === 0) continue;
				// Writes each chunk separately.
				await new Promise<void>((resolve) => response.write(bytes, () => resolve()));
			}
			response.end();
		});
	});
	return new Promise((resolve, reject) => {
		server.once("error", reject);
		server.listen(0, "127.0.0.1", () => {
			const { port } = server.address() as AddressInfo;
			resolve({
				url: `http://127.0.0.1:${port}`,
				requests,
				close: () =>
					new Promise((closed) => {
						server.closeAllConnections();
						server.close(() => closed());
					}),
			});
		});
	});
}
