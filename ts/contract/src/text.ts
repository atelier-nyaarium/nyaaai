// UTF-16 text helpers.

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

export const indexOfText = (text: string, needle: string, from = 0): number => text.indexOf(needle, within(text, from));

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

function mapAscii(text: string, from: number, to: number, shift: number): string {
	let out = "";
	for (let at = 0; at < text.length; at++) {
		const unit = text.charCodeAt(at);
		out += unit >= from && unit <= to ? String.fromCharCode(unit + shift) : text[at];
	}
	return out;
}

export const asciiLower = (text: string): string => mapAscii(text, 0x41, 0x5a, 0x20);

export const asciiUpper = (text: string): string => mapAscii(text, 0x61, 0x7a, -0x20);

/** ECMAScript whitespace. */
export function isSpace(unit: number): boolean {
	return (
		(unit >= 0x09 && unit <= 0x0d) ||
		unit === 0x20 ||
		unit === 0xa0 ||
		unit === 0x1680 ||
		(unit >= 0x2000 && unit <= 0x200a) ||
		unit === 0x2028 ||
		unit === 0x2029 ||
		unit === 0x202f ||
		unit === 0x205f ||
		unit === 0x3000 ||
		unit === 0xfeff
	);
}

/** Trims ECMAScript whitespace. */
export function trimText(text: string): string {
	let start = 0;
	let end = text.length;
	while (start < end && isSpace(text.charCodeAt(start))) start += 1;
	while (end > start && isSpace(text.charCodeAt(end - 1))) end -= 1;
	return text.slice(start, end);
}

/** Collapses whitespace runs to spaces. */
export function collapseSpace(text: string): string {
	const words: string[] = [];
	let from = -1;
	for (let at = 0; at <= text.length; at++) {
		if (at === text.length || isSpace(text.charCodeAt(at))) {
			if (from >= 0) words.push(text.slice(from, at));
			from = -1;
		} else if (from < 0) {
			from = at;
		}
	}
	return words.join(" ");
}
