// Sentences for model notices.

////////////////////////////////
//  Functions & Helpers

/** Undefined: the label is unknown. */
export function overloadedText(model: string | undefined): string {
	return `${model ?? "The model"} is overloaded at its provider.`;
}

/** Undefined: the label is unknown. */
export function unavailableText(model: string | undefined, entry: string | undefined): string {
	return `${model ?? "The model"} is unavailable on ${entry ?? "its provider"}.`;
}
