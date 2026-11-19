import { expect, it } from "bun:test";
import { failed, failureOf, ProviderError } from "../failure";
import { StepAssembler } from "../step";

const CALL = { kind: "call", index: 0, wireId: "", name: "read", argumentsText: "" } as const;

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

it("a failure is read from provider errors alone", () => {
	const look = { failure: { category: "idle", status: "idle", message: "x" } };
	expect([undefined, null, "text", 5, ["failure"], look, new Error("plain")].map(failureOf)).toEqual([
		undefined,
		undefined,
		undefined,
		undefined,
		undefined,
		undefined,
		undefined,
	]);
});

it("the status falls back to the category, and extra cannot set category or message", () => {
	expect(failed("stream", "m", { status: undefined }).failure.status).toBe("stream");
	expect(failed("stream", "m", { status: 502 }).failure.status).toBe(502);
	const sneaky = { category: "auth", message: "other", retryAfterMs: 5 } as object;
	expect(failed("stream", "m", sneaky).failure).toEqual({
		category: "stream",
		status: "stream",
		message: "m",
		retryAfterMs: 5,
	});
});

it("an assembler keeps nothing it was handed, hands out nothing it holds and keeps a call to its own fields", () => {
	const assembler = new StepAssembler();
	const segment = { kind: "content" as const, text: "a" };
	assembler.add({ kind: "thought", part: { kind: "thought", type: "t", segments: [segment] } });
	segment.text = "MUTATED";
	expect(assembler.snapshot[0]).toMatchObject({ segments: [{ text: "a" }] });

	const joined = { kind: "content" as const, text: "b" };
	assembler.add({ kind: "thought", part: { kind: "thought", type: "t", segments: [joined] }, split: true });
	joined.text = "MUTATED";
	expect(assembler.snapshot[0]).toMatchObject({ segments: [{ text: "a" }, { text: "b" }] });

	const first = assembler.snapshot;
	(first[0] as { type: string }).type = "changed";
	const result = assembler.result();
	(result.parts[0] as { type: string }).type = "changed";
	(result.parts[0] as { segments: Array<{ text: string }> }).segments[0]!.text = "changed";
	expect(assembler.snapshot[0]).toMatchObject({ type: "t", segments: [{ text: "a" }, { text: "b" }] });
	expect(assembler.result().parts[0]).toMatchObject({ type: "t", segments: [{ text: "a" }, { text: "b" }] });

	const calls = new StepAssembler();
	calls.add({ ...CALL, extra: 1 } as typeof CALL);
	calls.snapshot.pop();
	const held = calls.result();
	expect(calls.result()).toEqual(held);
	expect(calls.snapshot).toEqual([CALL]);
});
