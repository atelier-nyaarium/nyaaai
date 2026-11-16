// URL origin cases.

import type { CaseSource } from "../caseAdapter";

/** Hosts the primitive leaves to the platform's IDNA and percent decoding. */
export const UNREAD_URLS = ["http://\u{e9}xample.com/", "http://ex%41mple.com/"];

export const ORIGIN_URLS = [
	// Default ports.
	"http://example.com",
	"http://example.com:80/",
	"http://example.com:81",
	"https://example.com:443",
	"https://example.com:80",
	"ws://example.com:80",
	"wss://example.com:443",
	"wss://example.com:80",
	"ftp://example.com:21",
	"ftp://example.com:22",
	"http://example.com:0080",
	"http://example.com:00000000000000000080",
	"http://example.com:65535",
	"http://example.com:0",
	"http://example.com:",
	// Case.
	"HTTP://EXAMPLE.COM/Path",
	"Http://ExAmPlE.CoM:8080",
	"hTTps://Example.com",
	// A trailing dot stays.
	"http://example.com./",
	"http://example.com.:80",
	"http://example.com..",
	// IPv6.
	"http://[::1]/",
	"http://[::1]:8080/",
	"http://[::1]:80",
	"http://[0:0:0:0:0:0:0:1]/",
	"http://[2001:DB8:0:0:1:0:0:1]/",
	"http://[2001:db8::1]:443",
	"https://[2001:db8::1]:443",
	"https://[::ffff:1.2.3.4]/",
	"http://[::1.2.3.4]/",
	"http://[1:2:3:4:5:6:7:8]/",
	"http://[::]/",
	"http://[1::]/",
	"http://[1:2:3:4:5:6:7::]/",
	"http://[0:0:1:0:0:1:0:0]/",
	"http://[1:0:0:2:0:0:0:3]/",
	"http://[::1",
	"http://[1:2:3]/",
	"http://[::1::2]/",
	"http://[1:2:3:4:5:6:7:8:9]/",
	"http://[1:2:3:4:5:6:7:8::]/",
	"http://[g::1]/",
	"http://[::12345]/",
	"http://[::1.2.3]/",
	"http://[::01.2.3.4]/",
	"http://[]/",
	"http://[::1]x/",
	"http://[:1]/",
	"http://[::1%25eth0]/",
	// A user and a password.
	"http://user:pass@example.com/",
	"http://user@example.com:8080",
	"http://a@b@example.com/",
	"http://user:pa:ss@host:81",
	"http://@example.com",
	"http://user:pass@[::1]:81/",
	// IPv4.
	"http://127.0.0.1:80",
	"http://0x7f.1/",
	"http://127.1/",
	"http://2130706433/",
	"http://0177.0.0.1",
	"http://0x/",
	"http://256.0.0.1/",
	"http://1.2.3.4.5/",
	"http://1.2.3.4./",
	"http://a.1/",
	"http://example.123/",
	"http://999999999999/",
	"http://08.0.0.1/",
	"http://1.2.3/",
	"http://1.2.65536/",
	"http://example.0x1f/",
	// Whitespace and slashes.
	"  http://example.com/ ",
	"http://exa\tmple.com",
	"ht\ntp://example.com",
	"http:\\\\example.com\\path",
	"http:/example.com",
	"http:example.com",
	"http:////example.com",
	// The rest of the URL.
	"http://example.com/a?b#c",
	"http://example.com?x",
	"http://example.com#x",
	"http://example.com:8080?x",
	"http://example.com:8080\\x",
	// Not read.
	"",
	" ",
	"not a url",
	"example.com",
	"//example.com",
	"http://",
	"http:///",
	"http://:80",
	"http://host:abc",
	"http://host:65536",
	"http://host:-1",
	"http://host:80:80",
	"http://exa%2Fmple.com/",
	"http://exa mple.com",
	"http://exa<mple.com",
	"http://exa^mple.com",
	"http://exa|mple.com",
	"mailto:a@b.c",
	"file:///etc/passwd",
	"javascript:alert(1)",
	"data:text/plain,hi",
	"foo://bar",
	"1http://example.com",
	"http//example.com",
	"-http://example.com",
	"ht+tp://example.com",
	...UNREAD_URLS,
];

const PIECES = [
	"http",
	"https",
	"ws",
	"ftp",
	"file",
	"://",
	":/",
	"/",
	"\\",
	"@",
	"[",
	"]",
	":",
	"::",
	"1",
	"0",
	"a",
	"B",
	"x",
	".",
	"..",
	"80",
	"443",
	"0x",
	"ff",
	"-",
	"?",
	"#",
	" ",
	"255",
	"256",
	"u:p",
	"e",
	"8",
	"9",
];
const SCHEMES = ["http", "https", "ws", "wss", "ftp"];

const GROUPS = ["0", "1", "a", "ff", "fe80", "0000", "1234", "12345", "g"];

/** Seeded pieces of URLs; the platform reads every one as these do. */
function fuzzed(count: number, v6 = false): string[] {
	let seed = 12345;
	const next = (below: number): number => {
		seed = (seed * 1103515245 + 12345) % 2147483648;
		return Math.floor((seed / 2147483648) * below);
	};
	return Array.from({ length: count }, () => {
		if (v6) {
			let inner = GROUPS[next(GROUPS.length)] as string;
			for (let at = 0, length = next(8); at < length; at++) {
				inner += `${next(4) === 0 ? "::" : ":"}${GROUPS[next(GROUPS.length)]}`;
			}
			return `http://[${inner}]${next(2) === 0 ? ":80" : ""}`;
		}
		let text = next(10) < 7 ? `${SCHEMES[next(SCHEMES.length)]}://` : "";
		for (let at = 0, length = 2 + next(9); at < length; at++) text += PIECES[next(PIECES.length)];
		return text;
	});
}

export const FUZZED_URLS = [...fuzzed(1200), ...fuzzed(600, true)];

export const urlCases: CaseSource[] = [
	{ kind: "url.origin", name: "origins", input: { urls: ORIGIN_URLS } },
	{ kind: "url.origin", name: "fuzzed", input: { urls: FUZZED_URLS } },
];
