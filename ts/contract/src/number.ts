// ECMAScript number conversion.

////////////////////////////////
//  Constants

/** JSON number syntax. */
const NUMBER = /^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?$/;
const INTEGER = /^-?(?:0|[1-9]\d*)$/;
const MOST_DIGITS = 100;

////////////////////////////////
//  Functions & Helpers

/** Shortest round-trip text. */
export const numberText = (value: number): string => String(value);

/** Ties round toward +infinity. */
export const roundHalfUp = (value: number): number => Math.round(value);

/** ECMAScript toFixed semantics. */
export function fixedText(value: number, digits: number): string {
	return value.toFixed(Math.min(MOST_DIGITS, Math.max(0, Math.trunc(digits))));
}

/** Finite JSON number text. */
export function numberOfText(text: string): number | undefined {
	if (!NUMBER.test(text)) return undefined;
	const value = Number(text);
	return Number.isFinite(value) ? value : undefined;
}

/** Safe integer text. */
export function integerOfText(text: string): number | undefined {
	if (!INTEGER.test(text)) return undefined;
	const value = Number(text);
	return Number.isSafeInteger(value) ? value + 0 : undefined;
}
