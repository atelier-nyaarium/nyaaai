import { expect, it } from "bun:test";
import { retentionOf } from "../kinds/opencode/retention";
import { MODELS } from "../models/curated";
import { raced } from "../models/view";

it("the curated list has unique keys and families", () => {
	expect(new Set(MODELS.map((row) => row.key)).size).toBe(MODELS.length);
	expect(new Set(MODELS.map((row) => row.family)).size).toBe(MODELS.length);
});

it("every curated row binds OpenRouter, never to an alias or a free or batch id", () => {
	for (const row of MODELS) {
		expect(row.bindings["openrouter"], row.key).toBeDefined();
		for (const binding of Object.values(row.bindings)) {
			const id = binding.id;
			const bad = ["~", "latest", "-free", ":free", ":batch"].some((part) => id.includes(part));
			expect(bad, `${row.key}: ${id}`).toBe(false);
		}
	}
});

it("retention answers for table keys only, never for names the platform's objects hold", () => {
	for (const id of ["constructor", "__proto__", "toString", "hasOwnProperty"]) {
		expect(retentionOf("go", id)).toBeUndefined();
		expect(retentionOf("payg", id)).toBeUndefined();
	}
});

it("raced hands the promise back, rejects with the signal's reason, and passes a rejection through", async () => {
	const promise = Promise.resolve(5);
	expect(raced(promise, undefined)).toBe(promise);
	expect(await raced(promise, new AbortController().signal)).toBe(5);

	const before = new AbortController();
	before.abort(new Error("early"));
	expect(() => raced(new Promise(() => {}), before.signal)).toThrow("early");
	const later = new AbortController();
	const pending = raced(new Promise<number>(() => {}), later.signal);
	later.abort("late");
	let reason: unknown;
	try {
		await pending;
	} catch (error) {
		reason = error;
	}
	expect(reason).toBe("late");

	try {
		await raced(Promise.reject(new Error("boom")), new AbortController().signal);
	} catch (error) {
		reason = error;
	}
	expect((reason as Error).message).toBe("boom");
});
