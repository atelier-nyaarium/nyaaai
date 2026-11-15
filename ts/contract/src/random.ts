// Randomness port.

////////////////////////////////
//  Interfaces & Types

export interface Random {
	/** In [0, 1). */
	next: () => number;
}

////////////////////////////////
//  Functions & Helpers

export const systemRandom: Random = { next: () => Math.random() };

/** Cycles supplied values. */
export function sequenceRandom(values: readonly number[]): Random {
	if (values.length === 0) throw new Error("a random sequence holds a value");
	let drawn = 0;
	return { next: () => values[drawn++ % values.length] as number };
}
