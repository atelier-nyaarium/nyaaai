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
	/** Read to the end; a read may reject. The connection is released at the end, at the iterator's `return`, or at the abort. */
	chunks: AsyncIterable<Uint8Array>;
}

/** No redirect is followed: a 3xx is answered as the status it is. The signal covers the whole exchange: a pending or later chunk read rejects with its reason. */
export type Fetch = (request: FetchRequest, signal: AbortSignal) => Promise<FetchResponse>;
