// ID generation.

////////////////////////////////
//  Interfaces & Types

export interface Ids {
	newId: () => string;
}

////////////////////////////////
//  Functions & Helpers

/** Version 4 UUID. */
export const randomUuid = (): string => crypto.randomUUID();

export const randomIds: Ids = { newId: randomUuid };

/** IDs start at 1. */
export function countedIds(prefix: string): Ids {
	let minted = 0;
	return { newId: () => `${prefix}${++minted}` };
}
