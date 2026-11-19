// The one way a catalog joins reads that run at once.

////////////////////////////////
//  Functions & Helpers

/** Starts every read at once, in list order, and answers in list order; the first failure fails the join. */
export function joinAll<T>(reads: ReadonlyArray<() => Promise<T>>): Promise<T[]> {
	return Promise.all(reads.map((read) => read()));
}

/**
 * Starts every read at once, in list order and waits for every one to settle, ending none early.
 * Answers the values in list order, or the first failure in list order.
 */
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
