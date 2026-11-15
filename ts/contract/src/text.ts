// UTF-16 cuts preserve pairs.

////////////////////////////////
//  Functions & Helpers

const isHigh = (unit: number) => unit >= 0xd800 && unit <= 0xdbff;
const isLow = (unit: number) => unit >= 0xdc00 && unit <= 0xdfff;

const splits = (text: string, at: number) =>
	at > 0 && at < text.length && isHigh(text.charCodeAt(at - 1)) && isLow(text.charCodeAt(at));

const within = (text: string, at: number) => Math.min(text.length, Math.max(0, Math.trunc(at) || 0));

export const textLength = (text: string): number => text.length;

/** Clamps, then rounds down. */
export function cutBefore(text: string, at: number): number {
	const held = within(text, at);
	return splits(text, held) ? held - 1 : held;
}

/** Clamps, then rounds up. */
export function cutAfter(text: string, at: number): number {
	const held = within(text, at);
	return splits(text, held) ? held + 1 : held;
}

export const headOf = (text: string, max: number): string => text.slice(0, cutBefore(text, max));

export const tailOf = (text: string, max: number): string =>
	text.slice(cutAfter(text, text.length - within(text, max)));

/** Bounds avoid pair splits. */
export function sliceText(text: string, start: number, end: number = text.length): string {
	const from = cutBefore(text, start);
	return text.slice(from, Math.max(from, cutBefore(text, end)));
}

/** Empty needles match at `from`. */
export const indexOfText = (text: string, needle: string, from = 0): number => text.indexOf(needle, within(text, from));

/** Empty needles match at `from`. */
export const lastIndexOfText = (text: string, needle: string, from: number = text.length): number =>
	text.lastIndexOf(needle, within(text, from));

const UTF8 = new TextEncoder();

export const utf8Length = (text: string): number => UTF8.encode(text).length;

/** Preserves UTF-8 boundaries. */
export function cutUtf8(text: string, maxBytes: number): string {
	let bytes = 0;
	let end = 0;
	for (const char of text) {
		bytes += utf8Length(char);
		if (bytes > maxBytes) break;
		end += char.length;
	}
	return text.slice(0, end);
}

export const asciiLower = (text: string): string => text.replace(/[A-Z]/g, (char) => char.toLowerCase());

export const asciiUpper = (text: string): string => text.replace(/[a-z]/g, (char) => char.toUpperCase());
