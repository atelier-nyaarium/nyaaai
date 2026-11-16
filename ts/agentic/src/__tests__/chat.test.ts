import { expect, it } from "bun:test";
import { thoughtText } from "../chat/text";

it("a thought's text is its segments in order", () => {
	expect(
		thoughtText({
			kind: "thought",
			type: "reasoning",
			segments: [
				{ kind: "summary", text: "A" },
				{ kind: "content", text: "B" },
			],
		}),
	).toBe("AB");
	expect(thoughtText({ kind: "thought", type: "reasoning" })).toBe("");
});
