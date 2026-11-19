// The one way a catalog joins reads that run at once.

////////////////////////////////
//  Functions & Helpers

/** Runs all reads at once; answers in list order, the first failure fails. */
export function joinAll<T>(reads: ReadonlyArray<() => Promise<T>>): Promise<T[]> {
	return Promise.all(reads.map((read) => read()));
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
