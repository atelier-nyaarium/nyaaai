// The spellings a secret can take in text.

import { base64Encode } from "./base64";
import { asciiLower, encodePathSegment } from "./text";

////////////////////////////////
//  Interfaces & Types

export interface Spelling {
	text: string;
	/** Positions matched without ASCII case. */
	folded: number[];
}

////////////////////////////////
//  Constants

export const REDACTED = "[key]";

const UTF8 = new TextEncoder();

////////////////////////////////
//  Functions & Helpers

/** Hex digits of each `%XX` triple. */
function hexPositions(text: string): number[] {
	const found: number[] = [];
	for (let at = 0; at < text.length; at++) if (text[at] === "%") found.push(at + 1, at + 2);
	return found;
}

/** Unpadded base64 after `lead` zero bytes, from the first character holding only secret bits. */
function shifted(secret: Uint8Array, lead: number): { core: string; full: string } {
	const padded = new Uint8Array(lead + secret.length);
	padded.set(secret, lead);
	let text = base64Encode(padded);
	let end = text.length;
	while (end > 0 && text[end - 1] === "=") end -= 1;
	text = text.slice(0, end);
	const from = Math.ceil((lead * 8) / 6);
	const to = Math.floor(((lead + secret.length) * 8) / 6);
	return { full: text.slice(from), core: text.slice(from, to) };
}

/** Plain, path-encoded in either hex case, and base64 at each byte alignment. */
export function secretSpellings(secret: string): Spelling[] {
	if (secret.length === 0) return [];
	const bytes = UTF8.encode(secret);
	const encoded = encodePathSegment(secret);
	const found: Spelling[] = [
		{ text: secret, folded: [] },
		{ text: encoded, folded: hexPositions(encoded) },
	];
	for (let lead = 0; lead < 3; lead++) {
		const { full, core } = shifted(bytes, lead);
		found.push({ text: full, folded: [] }, { text: core, folded: [] });
	}
	const seen = new Set<string>();
	return found.filter((spelling) => {
		const id = `${spelling.text}\u{0}${spelling.folded.join(",")}`;
		if (spelling.text.length === 0 || seen.has(id)) return false;
		seen.add(id);
		return true;
	});
}

/** Whether the spelling is in `text` at `at`. */
function matches(text: string, at: number, spelling: Spelling): boolean {
	if (at + spelling.text.length > text.length) return false;
	for (let place = 0; place < spelling.text.length; place++) {
		const held = text[at + place] as string;
		const wanted = spelling.text[place] as string;
		if (held === wanted) continue;
		if (!spelling.folded.includes(place) || asciiLower(held) !== asciiLower(wanted)) return false;
	}
	return true;
}

/** Length of the longest spelling in `text` at `at`; zero when none. */
function longestAt(text: string, at: number, spellings: readonly Spelling[]): number {
	let best = 0;
	for (const spelling of spellings)
		if (spelling.text.length > best && matches(text, at, spelling)) best = spelling.text.length;
	return best;
}

/** Whether any spelling occurs in `text`. */
export function leaksSpelling(text: string, spellings: readonly Spelling[]): boolean {
	for (let at = 0; at < text.length; at++) if (longestAt(text, at, spellings) > 0) return true;
	return false;
}

/** One pass: longest matches become `[key]`; replacements are never rescanned. */
export function scrubSpellings(text: string, spellings: readonly Spelling[]): string {
	let out = "";
	let from = 0;
	for (let at = 0; at < text.length; ) {
		const length = longestAt(text, at, spellings);
		if (length === 0) {
			at += 1;
			continue;
		}
		out += text.slice(from, at) + REDACTED;
		at += length;
		from = at;
	}
	return out + text.slice(from);
}

/** `text` with every spelling of `secret` replaced. */
export const scrubSecret = (text: string, secret: string): string => scrubSpellings(text, secretSpellings(secret));
