// Hosts of this network, judged by their text, and where a key may travel.

import { Pattern } from "@nyaaai/contract/pattern";
import { parseUrl } from "@nyaaai/contract/url";

////////////////////////////////
//  Constants

/** Loopback, private ranges and LAN names. */
const PRIVATE_HOST = Pattern.compile(
	"^(localhost|0\\.0\\.0\\.0|(127|10)\\.[\\d.]+|192\\.168\\.[\\d.]+|172\\.(1[6-9]|2\\d|3[01])\\.[\\d.]+|169\\.254\\.[\\d.]+|100\\.(6[4-9]|[7-9]\\d|1[01]\\d|12[0-7])\\.[\\d.]+|\\[(::1?|f[cd][0-9a-f]{2}:[0-9a-f:]*|fe[89ab][0-9a-f]:[0-9a-f:]*)\\]|[^.[]+|.+\\.(localhost|local|lan|internal|home\\.arpa))$",
	"i",
);

const MAPPED_PREFIX = "[::ffff:";

////////////////////////////////
//  Functions & Helpers

/** One to four hex digits. */
function piece(text: string): number | undefined {
	if (text.length === 0 || text.length > 4) return undefined;
	let value = 0;
	for (const char of text) {
		const digit = "0123456789abcdef".indexOf(char);
		if (digit < 0) return undefined;
		value = value * 16 + digit;
	}
	return value;
}

/** The IPv4 address of an IPv4-mapped IPv6 literal. */
function mappedIpv4(host: string): string | undefined {
	if (!host.startsWith(MAPPED_PREFIX) || !host.endsWith("]")) return undefined;
	const [high, low, ...rest] = host.slice(MAPPED_PREFIX.length, -1).split(":").map(piece);
	if (high === undefined || low === undefined || rest.length > 0) return undefined;
	return [high >> 8, high & 255, low >> 8, low & 255].join(".");
}

/** A host of this network, as `parseUrl` writes it; never resolved, a trailing dot cut. */
export function isLocalHost(host: string): boolean {
	const cut = host.endsWith(".") ? host.slice(0, -1) : host;
	return PRIVATE_HOST.test(mappedIpv4(cut) ?? cut);
}

/** https, or http to a host of this network. */
export function keyMayTravel(url: string): boolean {
	const parts = parseUrl(url);
	if (parts === undefined) return false;
	return parts.scheme === "https" || (parts.scheme === "http" && isLocalHost(parts.host));
}
