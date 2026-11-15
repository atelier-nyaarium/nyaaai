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
});
