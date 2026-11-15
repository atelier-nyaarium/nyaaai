// Promise helpers avoid Bun's stalled matcher.

////////////////////////////////
//  Functions & Helpers

/** Throws on resolution. */
export async function rejection(promise: Promise<unknown>): Promise<unknown> {
	let reason: unknown;
	const rejected = await promise.then(
		() => false,
		(held: unknown) => {
			reason = held;
			return true;
		},
	);
	if (!rejected) throw new Error("the promise resolved");
	return reason;
}

/** Use with `toThrow`. */
export async function rethrown(promise: Promise<unknown>): Promise<() => never> {
	const reason = await rejection(promise);
	return () => {
		throw reason;
	};
}
