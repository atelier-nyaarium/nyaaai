import { expect, it } from "bun:test";
import { patternProblem } from "@nyaaai/contract/pattern";
import { accepted, acceptedCaptures, refused, refusedCaptures } from "../caseSources/pattern";

it("the dialect case lists say what they hold", () => {
	const wrong = [
		...[...accepted, acceptedCaptures].filter(({ source, flags }) => patternProblem(source, flags) !== undefined),
		...[...refused, refusedCaptures].filter(({ source, flags }) => patternProblem(source, flags) === undefined),
	];
	expect(wrong.map(({ source, flags }) => `/${source.slice(0, 40)}/${flags}`)).toEqual([]);
});
