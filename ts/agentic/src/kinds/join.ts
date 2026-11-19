// The one way a catalog joins reads that run at once.

////////////////////////////////
//  Constants

/** A listing's rows read at most this many. */
export const MAX_LISTED = 2000;

/** Reads in flight at once, for a read per listed row. */
export const READS_AT_ONCE = 8;

////////////////////////////////
//  Functions & Helpers

/** Runs all reads at once; answers in list order, the first failure fails. */
export function joinAll<T>(reads: ReadonlyArray<() => Promise<T>>): Promise<T[]> {
	return Promise.all(reads.map((read) => read()));
}

/** At most `limit` reads at once, in list order; a failure starts no more, waits for those started, and the earliest is thrown. */
export async function joinLimited<T>(reads: ReadonlyArray<() => Promise<T>>, limit = READS_AT_ONCE): Promise<T[]> {
	const answers: T[] = [];
	const failures = new Map<number, unknown>();
	let next = 0;
	const lane = async (): Promise<void> => {
		while (failures.size === 0 && next < reads.length) {
			const at = next++;
			const read = reads[at];
			if (read === undefined) return;
			try {
				answers[at] = await read();
			} catch (error) {
				failures.set(at, error);
			}
		}
	};
	await Promise.all(Array.from({ length: Math.min(Math.max(limit, 1), reads.length) }, lane));
	const first = Math.min(...failures.keys());
	if (failures.size > 0) throw failures.get(first);
	return answers;
}

/** The first `max` items with distinct keys, in order. */
export function firstDistinct<T>(items: ReadonlyArray<T>, keyOf: (item: T) => string, max = MAX_LISTED): T[] {
	const seen = new Set<string>();
	const kept: T[] = [];
	for (const item of items) {
		if (kept.length >= max) break;
		const key = keyOf(item);
		if (seen.has(key)) continue;
		seen.add(key);
		kept.push(item);
	}
	return kept;
}

/** Runs all reads at once, waits for all; answers in list order, or the first failure. */
export async function joinSettled<T>(reads: ReadonlyArray<() => Promise<T>>): Promise<T[]> {
	const settled = await Promise.allSettled(reads.map((read) => read()));
	const failed = settled.find((each): each is PromiseRejectedResult => each.status === "rejected");
	if (failed !== undefined) throw failed.reason;
	return settled.map((each) => (each as PromiseFulfilledResult<T>).value);
}

/** Tells a listener; what it throws changes nothing. */
export function told(listener: (() => void) | undefined): void {
	try {
		listener?.();
	} catch {
		// A listener cannot change a refresh's outcome.
	}
}
