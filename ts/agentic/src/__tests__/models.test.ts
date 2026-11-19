import { expect, it } from "bun:test";
import { retentionOf } from "../kinds/opencode/retention";
import { MODELS } from "../models/curated";
import { parseRowKey, rowKey } from "../models/rowKey";
import { raced } from "../models/view";
import { SKIP_REASONS } from "../provider/disposition";
import { PROVIDER_KINDS, PROVIDER_PLANS } from "../provider/entry";

it("the curated list leads with the four main models, with unique keys and families", () => {
	expect(MODELS.slice(0, 4).map((row) => row.key)).toEqual([
		"openai/gpt-6-luna",
		"anthropic/claude-sonnet-5.5",
		"anthropic/claude-opus-5.5",
		"openai/gpt-6.1-sol",
	]);
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
		expect(row.identity).toEqual({ kind: "curated", key: row.key });
		expect(parseRowKey(row.key)).toEqual({ kind: "curated", key: row.key });
		expect(rowKey({ kind: "curated", key: row.key })).toBe(row.key);
	}
});

it("the kind, plan and skip reason lists keep the shapes' order", () => {
	expect([...PROVIDER_KINDS]).toEqual(["openrouter", "opencode", "ollama", "lmstudio", "llamacpp", "compatible"]);
	expect([...PROVIDER_PLANS]).toEqual(["go", "payg"]);
	expect(SKIP_REASONS).toHaveLength(9);
	expect(SKIP_REASONS[0]).toBe("keyMissing");
	expect(SKIP_REASONS[8]).toBe("paused");
});

it("retention answers for table keys only, never for names the platform's objects hold", () => {
	for (const id of ["constructor", "__proto__", "toString", "hasOwnProperty"]) {
		expect(retentionOf("go", id)).toBeUndefined();
		expect(retentionOf("payg", id)).toBeUndefined();
	}
});

it("raced hands the promise back with no signal", async () => {
	const promise = Promise.resolve(5);
	expect(raced(promise, undefined)).toBe(promise);
	expect(await raced(promise, new AbortController().signal)).toBe(5);
});

it("raced rejects with the signal's reason, before or after the start", async () => {
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
});

it("raced passes a rejection through and stops listening once the promise settles", async () => {
	const controller = new AbortController();
	let removed = 0;
	const remove = controller.signal.removeEventListener.bind(controller.signal);
	controller.signal.removeEventListener = ((...args: Parameters<typeof remove>) => {
		removed += 1;
		return remove(...args);
	}) as typeof remove;
	let reason: unknown;
	try {
		await raced(Promise.reject(new Error("boom")), controller.signal);
	} catch (error) {
		reason = error;
	}
	expect((reason as Error).message).toBe("boom");
	await Promise.resolve();
	expect(removed).toBe(1);
	expect(await raced(Promise.resolve(1), controller.signal)).toBe(1);
});
