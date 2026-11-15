// JSON round-trip cases.

import type { CaseSource } from "../caseAdapter";

const nested = (open: string, close: string, depth: number) => open.repeat(depth) + close.repeat(depth);

const wide = Array.from({ length: 40 }, (_, at) => `"k${at % 30}":${at},"${39 - at}":${at}`).join(",");

const roundTrip = (name: string, texts: string[]): CaseSource => ({ kind: "json.roundTrip", name, input: { texts } });

/** Tied shortest number spellings. */
function dyadics(count: number): number[] {
	let seed = 0x2545f491;
	const next = () => {
		seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
		return seed;
	};
	return Array.from({ length: count }, () => {
		const whole = (next() * 2 ** 21 + (next() >>> 11)) % 2 ** 53;
		return (whole + 1) / 2 ** (1 + (next() % 30));
	});
}

const indexKeys = Array.from({ length: 5000 }, (_, at) => `"${4999 - at}":${at}`).join(",");

export const jsonCases: CaseSource[] = [
	roundTrip("plain", ['{"a":1,"b":[true,null,"x"]}', "null", '"text"', "[]", "{}"]),
	roundTrip("key-order", [
		'{"b":1,"10":2,"2":3,"a":4,"b":5,"01":6,"4294967294":7,"4294967295":8,"-1":9,"0":10}',
		'{"a":1,"a":2,"b":3,"a":{"x":1}}',
		`{${wide}}`,
	]),
	roundTrip("numbers", [
		"[1e21,123456789012345680000,0.000001,1e-7,-0,1.50,5e-324,1.7976931348623157e308]",
		"[1e999,-1e999,0.1,100,1E+2,9007199254740993,0.30000000000000004,123456.789e3,4.35]",
		"[2.5e-10,1e300,12345678901234567890123,-1.5e-7,0.1e1,1e-6,999999999999999900000,1e20,1.0]",
		"[0e0,-0.0,1e-400,123e-20,4.9e-324,2.2250738585072014e-308,0.000001234,1234567.125]",
		"[71692600759833.125,131332132934.515625,-131332132934.515625,0.5,2.5,1.25,9007199254740993]",
		JSON.stringify(dyadics(400)),
		JSON.stringify(dyadics(400).map((value) => -value / 2 ** 40)),
	]),
	roundTrip("index-keys", [`{${indexKeys}}`, `{${indexKeys},"0":"again","named":1,"4999":"last"}`]),
	roundTrip("strings", [
		'"\\u0000\\u001f\\"\\\\\\/\\b\\f\\n\\r\\t\\u2028\\u2029\\u007f\\ud83d\\ude00\\u00e9"',
		'"\u{2028}\u{2029}\u007f\u{1F600}\u{e9} raw"',
		'["\\ud83d","\\ude00"]',
		'["\\ud83d\\ude00","\\ud83d","\\ude00x","\\ud83d\\u0041","\\ud83d\\ud83d\\ude00","\\uD83D\\uDE00"]',
		'{"\\ud83d":1,"\\ude00":2,"__proto__":3}',
		'{"\\ufffd":1,"\\ud800":2,"\\ufffd":3}',
		'{"\\udc00":1,"\\ud800":2,"\\udc00":3,"a":{"\\ud800":4,"\\udfff":5}}',
		'"\\\\ud800 is text, \\ud800\\\\ is not"',
	]),
	roundTrip("refused", [
		"",
		"01",
		"[1,]",
		'{"a":1,}',
		"nul",
		"1 2",
		"\u{feff}1",
		'"a\tb"',
		'{"a"}',
		"[",
		'"\\x"',
		'"\\u12"',
		"+1",
		".5",
		"1.",
		"-",
		"1e",
		"Infinity",
		"NaN",
		"'a'",
		"{a:1}",
		"[1 2]",
		"true false",
		"\u{a0}1",
		"\v1",
		'"\\ud83d\\uZZZZ"',
	]),
	roundTrip("space", [" \t\n\r[ 1 , 2 ] \r\n", '{ "a" : { "b" : [ ] } }']),
	roundTrip("depth", [
		nested("[", "]", 1024),
		nested("[", "]", 1025),
		nested('{"a":', "}", 1024).replace(":}", ":1}"),
		nested('{"a":', "}", 1025).replace(":}", ":1}"),
		`[${nested("[", "]", 1023)},${nested("[", "]", 1024)}]`,
	]),
];
