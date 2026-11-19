// The HTTP port.

import type { HeaderList } from "./shapes/step";

////////////////////////////////
//  Interfaces & Types

/** A `GET` carries no body. */
export interface FetchRequest {
	method: "GET" | "POST";
	url: string;
	headers: HeaderList;
	body?: string;
}

export interface FetchResponse {
	status: number;
	/** In the `responseHeaders` form. */
	headers: HeaderList;
	/** A read may reject; the connection frees at end, `return` or abort. */
	chunks: AsyncIterable<Uint8Array>;
}

/** Redirects are not followed; the signal covers chunk reads, which reject with its reason. */
export type Fetch = (request: FetchRequest, signal: AbortSignal) => Promise<FetchResponse>;
