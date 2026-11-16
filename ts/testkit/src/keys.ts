// A scripted key source.

////////////////////////////////
//  Interfaces & Types

export interface FakeKeys {
	get: () => Promise<string | undefined>;
	/** Times the source was read. */
	reads: () => number;
}

////////////////////////////////
//  Functions & Helpers

/** Answers each read from `values`; the last repeats. `undefined` is no key. */
export function fakeKeys(values: ReadonlyArray<string | undefined>): FakeKeys {
	if (values.length === 0) throw new Error("fakeKeys needs a value");
	let reads = 0;
	return {
		get: async () => {
			const value = values[Math.min(reads, values.length - 1)];
			reads += 1;
			return value;
		},
		reads: () => reads,
	};
}
