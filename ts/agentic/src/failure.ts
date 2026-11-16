// Decoded provider failures.

import type { Failure } from "@nyaaai/contract/shapes/step";
import { trimText } from "@nyaaai/contract/text";

////////////////////////////////
//  Interfaces & Types

/** Decoded API error. */
export interface WireError {
	code?: number | string;
	type?: string;
	message?: string;
	metadata?: Record<string, unknown>;
	/** Limit reset time. */
	resetsAt?: string | number;
}

////////////////////////////////
//  Classes

/** A provider request's decoded failure. */
export class ProviderError extends Error {
	constructor(readonly failure: Failure) {
		super(failure.message);
		this.name = "ProviderError";
	}
}

////////////////////////////////
//  Functions & Helpers

/** Status is the category, for failures with no HTTP status. */
export function failed(category: Failure["category"], message: string, extra: Partial<Failure> = {}): ProviderError {
	return new ProviderError({ category, status: category, message, ...extra });
}

/** Undefined for errors no codec decoded. */
export function failureOf(error: unknown): Failure | undefined {
	return error instanceof ProviderError ? error.failure : undefined;
}

/** Blank counts as missing. */
export function messageOf(value: unknown): string | undefined {
	return typeof value === "string" && trimText(value).length > 0 ? value : undefined;
}
