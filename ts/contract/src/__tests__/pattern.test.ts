import { describe, expect, it } from "bun:test";
import { Pattern } from "../pattern";

// Shared-engine inputs only.
const SOURCES: Array<[source: string, flags: string]> = [
	["", ""],
	["a", ""],
	["a*", ""],
	["a*?", ""],
	["a+?", ""],
	["a|", ""],
	["(a)|b", ""],
	["(a)?", ""],
	["(x*)", ""],
	[",\\s*", ""],
	["(,)(\\s*)", ""],
	["(a)|(b)|c", ""],
	["(\\w)(\\d)?", ""],
	["\\d+", ""],
	["[^a]", ""],
	[".", ""],
	[".", "s"],
	["\\b", ""],
	["\\B", ""],
	["^", ""],
	["$", ""],
	["^", "m"],
	["$", "m"],
	["A", "i"],
	["a|ab", ""],
	["(a|ab)(c|bcd)(d*)", ""],
	["a+?b*", ""],
	["(?:ab){2,3}?", ""],
	["(?:a+)+b", ""],
	["a{2,4}", ""],
	["(?:^a|b$|\\ba)", "m"],
	["(a|b)*c", ""],
	["(\\u{1f600}|x)+", ""],
];

const TEXTS = [
	"",
	"a",
	"aaa",
	"abc",
	"baaac",
	",a, b,,c",
	"ab cd",
	" a ",
	"a1b22c",
	"a\nb\n",
	"\n\n",
	"\u{1f600}",
	"x\u{1f600}a\u{1f600}",
	"abcd",
	"aaabbb",
	"abababx",
	"aaabaaaab",
	"x\na\nb\nx",
	"\u{1f600}x\u{1f600}",
];

describe("Pattern", () => {
	it("splits, finds and replaces as the platform does", () => {
		for (const [source, flags] of SOURCES) {
			const pattern = Pattern.compile(source, flags);
			const native = () => new RegExp(source, `gu${flags}`);
			const groups = (new RegExp(`${source}|`, "u").exec("")?.length ?? 1) - 1;
			const template = groups > 0 ? "<$1:$&:$$>" : "<$&:$$>";
			for (const text of TEXTS) {
				const told = `${JSON.stringify(source)} ${flags} on ${JSON.stringify(text)}`;
				expect(pattern.split(text), told).toEqual(text.split(native()).map((piece) => piece ?? ""));
				expect(pattern.findAll(text), told).toEqual(
					[...text.matchAll(native())].map((found) => ({
						start: found.index,
						end: found.index + found[0].length,
						text: found[0],
						captures: found.slice(1).map((capture) => capture ?? null),
					})),
				);
				expect(pattern.replaceAll(text, template), told).toBe(text.replaceAll(native(), template));
			}
		}
	});

	it("handles ambiguous input within a second", () => {
		const pattern = Pattern.compile("^(a|aa)+$");
		const text = `${"a".repeat(40)}!`;
		const started = performance.now();
		expect(pattern.test(text)).toBe(false);
		expect(performance.now() - started).toBeLessThan(1000);
	});

	it("matches a lone surrogate as U+FFFD at the original offsets", () => {
		const text = "a\ud800b";
		const found = Pattern.compile(".", "s").findAll(text.slice(1, 2));
		expect(found).toEqual([{ start: 0, end: 1, text: "\ud800", captures: [] }]);
		expect(Pattern.compile("a.b", "s").findAll(text)).toEqual([{ start: 0, end: 3, text, captures: [] }]);
	});
});
