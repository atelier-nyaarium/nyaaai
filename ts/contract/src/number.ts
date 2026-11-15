// ECMAScript number conversion.

////////////////////////////////
//  Constants

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

function digitsEnd(text: string, at: number): number {
	let end = at;
	while (text.charCodeAt(end) >= 0x30 && text.charCodeAt(end) <= 0x39) end += 1;
	return end;
}

/** Canonical signed integer text. */
function integerEnd(text: string): number | undefined {
	const from = text[0] === "-" ? 1 : 0;
	const end = digitsEnd(text, from);
	const leadingZero = text[from] === "0" && end - from > 1;
	return end > from && !leadingZero ? end : undefined;
}

/** Finite JSON number text. */
export function numberOfText(text: string): number | undefined {
	let at = integerEnd(text);
	if (at === undefined) return undefined;
	if (text[at] === ".") {
		const end = digitsEnd(text, at + 1);
		if (end === at + 1) return undefined;
		at = end;
	}
	if (text[at] === "e" || text[at] === "E") {
		const from = at + 1 + (text[at + 1] === "+" || text[at + 1] === "-" ? 1 : 0);
		const end = digitsEnd(text, from);
		if (end === from) return undefined;
		at = end;
	}
	if (at !== text.length) return undefined;
	const value = Number(text);
	return Number.isFinite(value) ? value : undefined;
}

/** Safe integer text. */
export function integerOfText(text: string): number | undefined {
	if (integerEnd(text) !== text.length) return undefined;
	const value = Number(text);
	return Number.isSafeInteger(value) ? value + 0 : undefined;
}
