import { expect, it } from "bun:test";
import { patternProblem } from "@nyaaai/contract/pattern";
import { accepted, refused } from "../caseSources/pattern";

it("the dialect case lists say what they hold", () => {
	const wrong = [
		...accepted.filter(({ source, flags }) => patternProblem(source, flags) !== undefined),
		...refused.filter(({ source, flags }) => patternProblem(source, flags) === undefined),
	];
	expect(wrong.map(({ source, flags }) => `/${source.slice(0, 40)}/${flags}`)).toEqual([]);
});
