// What a catalog says of one model, and what a run reads of it.

import type { ModelRow, ModelView, PriceTier } from "@nyaaai/contract/shapes/model";
import type { ApiFacts, CodecFacts, Effort } from "@nyaaai/contract/shapes/step";
import { resolveEffort } from "../effort";

////////////////////////////////
//  Constants

/** A view without an API reads as one with no capabilities. */
const NO_API: ApiFacts = { family: "chat-completions", structured: false, choice: [], strict: false };

////////////////////////////////
//  Functions & Helpers

export function unknownView(row: ModelRow): ModelView {
	return { row, status: "unknown", efforts: [], hosts: [] };
}

/** Listed tiers; none when unpriced. */
export function tiersOf(view: Pick<ModelView, "price"> | undefined): readonly PriceTier[] | undefined {
	return typeof view?.price === "object" ? view.price.tiers : undefined;
}

/** What a Chat codec reads of a view. */
export function codecFactsOf(view: ModelView | undefined): CodecFacts | undefined {
	if (view === undefined) return undefined;
	return {
		api: view.api ?? NO_API,
		efforts: view.efforts,
		...(view.outputLimit === undefined ? {} : { outputLimit: view.outputLimit }),
		...(view.price === "free" ? { free: true } : {}),
	};
}

/** Missing inputs send none; unknown rows send wishes only for reasoning models; other rows resolve offers. */
export function runEffort(view: ModelView | undefined, wanted: Effort | undefined): Effort | undefined {
	if (view === undefined || wanted === undefined) return undefined;
	if (view.status === "unknown") return view.row.reasoning ? wanted : undefined;
	return resolveEffort(view.efforts, wanted);
}

/** The promise, or the signal's reason once it aborts. */
export function raced<T>(promise: Promise<T>, signal: AbortSignal | undefined): Promise<T> {
	if (signal === undefined) return promise;
	signal.throwIfAborted();
	return new Promise<T>((resolve, reject) => {
		const stop = () => reject(signal.reason);
		signal.addEventListener("abort", stop, { once: true });
		promise.then(resolve, reject).finally(() => signal.removeEventListener("abort", stop));
	});
}
