// Rust names and literals for generated code.

////////////////////////////////
//  Constants

const KEYWORDS = new Set([
	"as",
	"async",
	"await",
	"break",
	"const",
	"continue",
	"crate",
	"dyn",
	"else",
	"enum",
	"extern",
	"false",
	"fn",
	"for",
	"gen",
	"if",
	"impl",
	"in",
	"let",
	"loop",
	"match",
	"mod",
	"move",
	"mut",
	"pub",
	"ref",
	"return",
	"self",
	"Self",
	"static",
	"struct",
	"super",
	"trait",
	"true",
	"try",
	"type",
	"unsafe",
	"use",
	"where",
	"while",
	"abstract",
	"become",
	"box",
	"do",
	"final",
	"macro",
	"override",
	"priv",
	"typeof",
	"unsized",
	"virtual",
	"yield",
]);

////////////////////////////////
//  Functions & Helpers

const isLower = (unit: number): boolean => unit >= 0x61 && unit <= 0x7a;
const isUpper = (unit: number): boolean => unit >= 0x41 && unit <= 0x5a;
const isDigit = (unit: number): boolean => unit >= 0x30 && unit <= 0x39;
const isAlphanumeric = (unit: number): boolean => isLower(unit) || isUpper(unit) || isDigit(unit);

/** Splits at non-alphanumerics, and before an uppercase letter after a lowercase letter or digit. */
function words(text: string): string[] {
	const found: string[] = [];
	let start = -1;
	for (let at = 0; at < text.length; at += 1) {
		const unit = text.charCodeAt(at);
		if (!isAlphanumeric(unit)) {
			if (start >= 0) found.push(text.slice(start, at));
			start = -1;
			continue;
		}
		const before = at > 0 ? text.charCodeAt(at - 1) : 0;
		if (start >= 0 && isUpper(unit) && (isLower(before) || isDigit(before))) {
			found.push(text.slice(start, at));
			start = -1;
		}
		if (start < 0) start = at;
	}
	if (start >= 0) found.push(text.slice(start));
	return found;
}

/** The name when it is a Rust identifier. */
function identifierOf(made: string): string | undefined {
	let valid = made.length > 0 && !isDigit(made.charCodeAt(0));
	for (let at = 0; at < made.length; at += 1) {
		const unit = made.charCodeAt(at);
		if (!isAlphanumeric(unit) && unit !== 0x5f) valid = false;
	}
	if (!valid) return undefined;
	return KEYWORDS.has(made) ? `${made}_` : made;
}

/** Snake case, or undefined when no Rust name comes from the text. */
export const trySnake = (text: string): string | undefined =>
	identifierOf(
		words(text)
			.map((word) => word.toLowerCase())
			.join("_"),
	);

/** Pascal case, or undefined when no Rust name comes from the text. */
export const tryPascal = (text: string): string | undefined =>
	identifierOf(
		words(text)
			.map((word) => `${word.charAt(0).toUpperCase()}${word.slice(1)}`)
			.join(""),
	);

/** A variant name; the empty text is `Empty`. */
export const tryVariant = (text: string): string | undefined => (text === "" ? "Empty" : tryPascal(text));

function named(text: string, made: string | undefined): string {
	if (made === undefined) throw new Error(`${JSON.stringify(text)} names no Rust identifier`);
	return made;
}

export const snake = (text: string): string => named(text, trySnake(text));

export const pascal = (text: string): string => named(text, tryPascal(text));

export const variant = (text: string): string => named(text, tryVariant(text));

/** ASCII-only Rust literal. */
export function quote(text: string): string {
	let made = '"';
	for (const char of text) {
		const point = char.codePointAt(0) as number;
		if (char === '"' || char === "\\") made += `\\${char}`;
		else if (point >= 0x20 && point < 0x7f) made += char;
		else made += `\\u{${point.toString(16)}}`;
	}
	return `${made}"`;
}
