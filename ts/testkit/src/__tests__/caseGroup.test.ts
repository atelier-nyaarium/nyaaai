// Case kind result tests.

import { describe, expect, it } from "bun:test";
import { command } from "@nyaaai/adapter/runtime";
import { z } from "@nyaaai/contract/shape";
import { kind } from "../caseGroup";
import { rethrown } from "../rejection";

////////////////////////////////
//  Tests

describe("a case kind", () => {
	it("is held to its result shape when served", async () => {
		const held = kind(z.object({ fits: z.boolean() }), z.object({ text: z.string().min(1) }), (input) => ({
			text: input.fits ? "ok" : "",
		}));
		const served = command(held.input, held.result, (input) => (held.run as (input: unknown) => never)(input));
		expect(await served({ fits: true }, {} as never)).toEqual({ text: "ok" });
		const error = await rethrown(served({ fits: false }, {} as never));
		expect(() => error()).toThrow("the result does not fit the command");
	});
});
