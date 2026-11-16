// The HTTP port.

import type { HeaderList } from "./shapes/step";

////////////////////////////////
//  Interfaces & Types

export interface FetchRequest {
	method: "GET" | "POST";
	url: string;
	headers: HeaderList;
	body?: string;
}

export interface FetchResponse {
	status: number;
	headers: HeaderList;
	/** Read to the end; a read may reject. */
	chunks: AsyncIterable<Uint8Array>;
}

/** Abort rejects with reason. */
export type Fetch = (request: FetchRequest, signal: AbortSignal) => Promise<FetchResponse>;
