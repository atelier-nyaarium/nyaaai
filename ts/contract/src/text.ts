// UTF-16 text helpers.

import { CCC, COMPOSE, DECOMP } from "./nfcTables";

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

////////////////////////////////
//  NFC normalization

const S_BASE = 0xac00;
const L_BASE = 0x1100;
const V_BASE = 0x1161;
const T_BASE = 0x11a7;
const L_COUNT = 19;
const V_COUNT = 21;
const T_COUNT = 28;
const N_COUNT = V_COUNT * T_COUNT;
const S_COUNT = L_COUNT * N_COUNT;

const cccOf = (cp: number): number => {
	let lo = 0;
	let hi = CCC.length;
	while (lo < hi) {
		const mid = (lo + hi) >>> 1;
		if (CCC[mid]! >>> 8 < cp) lo = mid + 1;
		else hi = mid;
	}
	return lo < CCC.length && CCC[lo]! >>> 8 === cp ? CCC[lo]! & 0xff : 0;
};

const decompOf = (cp: number): number[] | undefined => {
	let lo = 0;
	let hi = DECOMP.length / 3;
	while (lo < hi) {
		const mid = (lo + hi) >>> 1;
		if (DECOMP[mid * 3]! < cp) lo = mid + 1;
		else hi = mid;
	}
	if (lo >= DECOMP.length / 3 || DECOMP[lo * 3]! !== cp) return undefined;
	const second = DECOMP[lo * 3 + 2]!;
	return second === 0 ? [DECOMP[lo * 3 + 1]!] : [DECOMP[lo * 3 + 1]!, second];
};

const composeOf = (first: number, second: number): number | undefined => {
	let lo = 0;
	let hi = COMPOSE.length / 3;
	while (lo < hi) {
		const mid = (lo + hi) >>> 1;
		const held = COMPOSE[mid * 3]!;
		if (held < first || (held === first && COMPOSE[mid * 3 + 1]! < second)) lo = mid + 1;
		else hi = mid;
	}
	if (lo >= COMPOSE.length / 3) return undefined;
	const held = COMPOSE[lo * 3]!;
	return held === first && COMPOSE[lo * 3 + 1]! === second ? COMPOSE[lo * 3 + 2]! : undefined;
};

const appendDecomposed = (cp: number, out: number[]): void => {
	if (cp >= S_BASE && cp < S_BASE + S_COUNT) {
		const index = cp - S_BASE;
		out.push(L_BASE + Math.floor(index / N_COUNT), V_BASE + Math.floor((index % N_COUNT) / T_COUNT));
		const tail = T_BASE + (index % T_COUNT);
		if (tail !== T_BASE) out.push(tail);
		return;
	}
	const parts = decompOf(cp);
	if (parts === undefined) {
		out.push(cp);
		return;
	}
	for (const part of parts) appendDecomposed(part, out);
};

const isAscii = (text: string): boolean => {
	for (let at = 0; at < text.length; at++) if (text.charCodeAt(at) >= 0x80) return false;
	return true;
};

/** Unicode 17.0 NFC. */
export function nfc(text: string): string {
	if (isAscii(text)) return text;
	const codes: number[] = [];
	for (let at = 0; at < text.length; at++) {
		const unit = text.charCodeAt(at);
		if (unit >= 0xd800 && unit <= 0xdbff) {
			const next = text.charCodeAt(at + 1);
			if (at + 1 < text.length && next >= 0xdc00 && next <= 0xdfff) {
				codes.push(0x10000 + (unit - 0xd800) * 0x400 + (next - 0xdc00));
				at += 1;
			} else {
				codes.push(0xfffd);
			}
		} else if (unit >= 0xdc00 && unit <= 0xdfff) {
			codes.push(0xfffd);
		} else {
			codes.push(unit);
		}
	}
	const decomposed: number[] = [];
	for (const cp of codes) appendDecomposed(cp, decomposed);
	let at = 0;
	while (at < decomposed.length) {
		if (cccOf(decomposed[at]!) === 0) {
			at += 1;
			continue;
		}
		let end = at;
		while (end < decomposed.length && cccOf(decomposed[end]!) !== 0) end += 1;
		const run = decomposed.slice(at, end).map((cp) => [cp, cccOf(cp)] as [number, number]);
		run.sort((left, right) => left[1] - right[1]);
		for (let cursor = 0; cursor < run.length; cursor++) decomposed[at + cursor] = run[cursor]![0];
		at = end;
	}
	const composed: number[] = [decomposed[0]!];
	let starter = 0;
	let lastClass = -1;
	for (let cursor = 1; cursor < decomposed.length; cursor++) {
		const cp = decomposed[cursor]!;
		const previous = composed[composed.length - 1]!;
		if (previous >= L_BASE && previous < L_BASE + L_COUNT && cp >= V_BASE && cp < V_BASE + V_COUNT) {
			composed[composed.length - 1] = S_BASE + (previous - L_BASE) * N_COUNT + (cp - V_BASE) * T_COUNT;
			starter = composed.length - 1;
			lastClass = -1;
			continue;
		}
		if (
			previous >= S_BASE &&
			previous < S_BASE + S_COUNT &&
			(previous - S_BASE) % T_COUNT === 0 &&
			cp > T_BASE &&
			cp < T_BASE + T_COUNT
		) {
			composed[composed.length - 1] = previous + (cp - T_BASE);
			starter = composed.length - 1;
			lastClass = -1;
			continue;
		}
		const class_ = cccOf(cp);
		if (lastClass < class_) {
			const pair = composeOf(composed[starter]!, cp);
			if (pair !== undefined) {
				composed[starter] = pair;
				continue;
			}
		}
		composed.push(cp);
		if (class_ === 0) {
			starter = composed.length - 1;
			lastClass = -1;
		} else {
			lastClass = class_;
		}
	}
	return composed.map((cp) => String.fromCodePoint(cp)).join("");
}
