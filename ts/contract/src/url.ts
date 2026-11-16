// URL origins.

import { asciiLower } from "./text";

////////////////////////////////
//  Constants

/** Special schemes and their default ports. */
const SPECIAL_PORTS: Record<string, number> = { http: 80, https: 443, ws: 80, wss: 443, ftp: 21 };

const FORBIDDEN_HOST = " #/:<>?@[\\]^|%";

////////////////////////////////
//  Functions & Helpers

const isAlpha = (unit: number) => (unit >= 0x41 && unit <= 0x5a) || (unit >= 0x61 && unit <= 0x7a);
const isDigit = (unit: number) => unit >= 0x30 && unit <= 0x39;
const isSlash = (char: string) => char === "/" || char === "\\";

/** Hex digits of any case, or undefined. */
function hexValue(text: string): number | undefined {
	if (text.length === 0) return undefined;
	let value = 0;
	for (let at = 0; at < text.length; at++) {
		const unit = text.charCodeAt(at);
		const digit = isDigit(unit)
			? unit - 0x30
			: unit >= 0x61 && unit <= 0x66
				? unit - 0x57
				: unit >= 0x41 && unit <= 0x46
					? unit - 0x37
					: -1;
		if (digit < 0) return undefined;
		value = value * 16 + digit;
	}
	return value;
}

function decimalValue(text: string): number | undefined {
	if (text.length === 0) return undefined;
	let value = 0;
	for (let at = 0; at < text.length; at++) {
		if (!isDigit(text.charCodeAt(at))) return undefined;
		value = value * 10 + (text.charCodeAt(at) - 0x30);
	}
	return value;
}

/** One IPv4 number: `0x` hex, leading-zero octal, or decimal. */
function ipv4Number(part: string): number | undefined {
	if (part.length >= 2 && part[0] === "0" && (part[1] === "x" || part[1] === "X")) {
		return part.length === 2 ? 0 : hexValue(part.slice(2));
	}
	if (part.length >= 2 && part[0] === "0") {
		let value = 0;
		for (let at = 1; at < part.length; at++) {
			const unit = part.charCodeAt(at);
			if (unit < 0x30 || unit > 0x37) return undefined;
			value = value * 8 + (unit - 0x30);
		}
		return value;
	}
	return decimalValue(part);
}

/** Whether the last label reads as a number. */
function endsInNumber(labels: string[]): boolean {
	const parts = labels.length > 1 && labels[labels.length - 1] === "" ? labels.slice(0, -1) : labels;
	const last = parts[parts.length - 1] ?? "";
	if (last === "") return false;
	if (decimalValue(last) !== undefined) return true;
	return (
		last.length >= 2 &&
		last[0] === "0" &&
		(last[1] === "x" || last[1] === "X") &&
		(last.length === 2 || hexValue(last.slice(2)) !== undefined)
	);
}

/** Dotted decimal, or undefined when the numbers do not fit. */
function ipv4Of(labels: string[]): string | undefined {
	const parts = labels.length > 1 && labels[labels.length - 1] === "" ? labels.slice(0, -1) : labels;
	if (parts.length > 4) return undefined;
	const numbers: number[] = [];
	for (const part of parts) {
		const value = ipv4Number(part);
		if (value === undefined) return undefined;
		numbers.push(value);
	}
	const last = numbers[numbers.length - 1] as number;
	for (const value of numbers.slice(0, -1)) if (value > 255) return undefined;
	if (last >= 256 ** (5 - numbers.length)) return undefined;
	let address = last;
	for (const [at, value] of numbers.slice(0, -1).entries()) address += value * 256 ** (3 - at);
	return [24, 16, 8, 0].map((shift) => Math.floor(address / 2 ** shift) % 256).join(".");
}

/** An embedded IPv4 in an IPv6 literal: four plain decimals. */
function embeddedIpv4(text: string): number[] | undefined {
	const parts = text.split(".");
	if (parts.length !== 4) return undefined;
	const bytes: number[] = [];
	for (const part of parts) {
		const value = decimalValue(part);
		if (value === undefined || value > 255 || (part.length > 1 && part[0] === "0")) return undefined;
		bytes.push(value);
	}
	const [first = 0, second = 0, third = 0, fourth = 0] = bytes;
	return [first * 256 + second, third * 256 + fourth];
}

/** Eight 16-bit pieces, or undefined. */
function ipv6Pieces(text: string): number[] | undefined {
	const gap = text.indexOf("::");
	if (gap >= 0 && text.indexOf("::", gap + 1) >= 0) return undefined;
	const groups = (part: string, ends: boolean): number[] | undefined => {
		if (part === "") return [];
		const names = part.split(":");
		const pieces: number[] = [];
		for (const [at, name] of names.entries()) {
			if (name.includes(".")) {
				if (!ends || at !== names.length - 1) return undefined;
				const tail = embeddedIpv4(name);
				if (tail === undefined) return undefined;
				pieces.push(...tail);
				continue;
			}
			const value = name.length <= 4 ? hexValue(name) : undefined;
			if (value === undefined) return undefined;
			pieces.push(value);
		}
		return pieces;
	};
	if (gap < 0) {
		const pieces = groups(text, true);
		return pieces?.length === 8 ? pieces : undefined;
	}
	const head = groups(text.slice(0, gap), false);
	const tail = groups(text.slice(gap + 2), true);
	if (head === undefined || tail === undefined || head.length + tail.length > 7) return undefined;
	return [...head, ...Array.from({ length: 8 - head.length - tail.length }, () => 0), ...tail];
}

/** Lowercase hex, the first longest run of zeros compressed. */
function ipv6Text(pieces: number[]): string {
	let from = -1;
	let length = 0;
	for (let at = 0; at < 8; at++) {
		if (pieces[at] !== 0) continue;
		let end = at;
		while (end < 8 && pieces[end] === 0) end += 1;
		if (end - at > length) {
			from = at;
			length = end - at;
		}
		at = end;
	}
	const hex = (piece: number | undefined) => (piece as number).toString(16);
	if (length < 2) return pieces.map(hex).join(":");
	return `${pieces.slice(0, from).map(hex).join(":")}::${pieces
		.slice(from + length)
		.map(hex)
		.join(":")}`;
}

/** The host as URLs write it, or undefined. */
function hostOf(raw: string): string | undefined {
	if (raw.startsWith("[")) {
		if (!raw.endsWith("]")) return undefined;
		const pieces = ipv6Pieces(raw.slice(1, -1));
		return pieces === undefined ? undefined : `[${ipv6Text(pieces)}]`;
	}
	if (raw === "") return undefined;
	for (let at = 0; at < raw.length; at++) {
		const unit = raw.charCodeAt(at);
		if (unit <= 0x20 || unit >= 0x7f || FORBIDDEN_HOST.includes(raw[at] as string)) return undefined;
	}
	const host = asciiLower(raw);
	const labels = host.split(".");
	return endsInNumber(labels) ? ipv4Of(labels) : host;
}

/**
 * The origin of an http, https, ws, wss or ftp URL as the WHATWG parser writes it:
 * lowercase scheme and host, default port dropped, credentials ignored.
 * Undefined for any other scheme, a malformed URL, or a host with non-ASCII text.
 */
export function originOf(text: string): string | undefined {
	let start = 0;
	let end = text.length;
	while (start < end && text.charCodeAt(start) <= 0x20) start += 1;
	while (end > start && text.charCodeAt(end - 1) <= 0x20) end -= 1;
	let url = "";
	for (let at = start; at < end; at++) {
		const char = text[at] as string;
		if (char !== "\t" && char !== "\n" && char !== "\r") url += char;
	}
	if (url.length === 0 || !isAlpha(url.charCodeAt(0))) return undefined;
	const colon = url.indexOf(":");
	if (colon < 0) return undefined;
	const scheme = asciiLower(url.slice(0, colon));
	for (let at = 1; at < scheme.length; at++) {
		const unit = scheme.charCodeAt(at);
		if (!isAlpha(unit) && !isDigit(unit) && scheme[at] !== "+" && scheme[at] !== "-" && scheme[at] !== ".")
			return undefined;
	}
	const defaultPort = SPECIAL_PORTS[scheme];
	if (defaultPort === undefined) return undefined;
	let at = colon + 1;
	while (at < url.length && isSlash(url[at] as string)) at += 1;
	let stop = at;
	while (stop < url.length && !isSlash(url[stop] as string) && url[stop] !== "?" && url[stop] !== "#") stop += 1;
	let authority = url.slice(at, stop);
	authority = authority.slice(authority.lastIndexOf("@") + 1);
	let hostText = authority;
	let portText = "";
	if (authority.startsWith("[")) {
		const close = authority.indexOf("]");
		if (close < 0) return undefined;
		hostText = authority.slice(0, close + 1);
		const rest = authority.slice(close + 1);
		if (rest !== "") {
			if (rest[0] !== ":") return undefined;
			portText = rest.slice(1);
		}
	} else {
		const port = authority.indexOf(":");
		if (port >= 0) {
			hostText = authority.slice(0, port);
			portText = authority.slice(port + 1);
		}
	}
	const host = hostOf(hostText);
	if (host === undefined) return undefined;
	let port: number | undefined;
	if (portText !== "") {
		port = decimalValue(portText);
		if (port === undefined || port > 65535) return undefined;
	}
	return `${scheme}://${host}${port === undefined || port === defaultPort ? "" : `:${port}`}`;
}
