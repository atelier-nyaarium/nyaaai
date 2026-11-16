import { expect, it } from "bun:test";
import { failed, failureOf, ProviderError } from "../failure";
import { StepAssembler } from "../step";

it("a provider error is an Error that carries its failure", () => {
	const error = failed("rateLimited", "Slow down.", { status: 429 });
	expect(error).toBeInstanceOf(Error);
	expect(error.name).toBe("ProviderError");
	expect(error.message).toBe("Slow down.");
	let thrown: unknown;
	try {
		throw error;
	} catch (held) {
		thrown = held;
	}
	expect(thrown).toBeInstanceOf(ProviderError);
	expect(failureOf(thrown)).toEqual({ category: "rateLimited", status: 429, message: "Slow down." });
});

it("a snapshot is a copy, and reading the result changes nothing", () => {
	const assembler = new StepAssembler();
	assembler.add({ kind: "call", index: 0, wireId: "", name: "read", argumentsText: "" });
	assembler.snapshot.pop();
	const first = assembler.result();
	expect(assembler.result()).toEqual(first);
	expect(assembler.snapshot).toEqual([{ kind: "call", index: 0, wireId: "", name: "read", argumentsText: "" }]);
});
