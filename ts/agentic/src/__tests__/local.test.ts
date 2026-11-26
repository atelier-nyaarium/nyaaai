// Local kind tests the cases cannot carry: a listing's reads, bounded.

import { expect, it } from "bun:test";
import { BUDGETS, span } from "@nyaaai/contract/fetch";
import { systemScheduler } from "@nyaaai/contract/time";
import { server as llamacppServer } from "../kinds/llamacpp";
import type { LocalTransport } from "../kinds/local/catalog";
import { server as ollamaServer } from "../kinds/ollama";

////////////////////////////////
//  Functions & Helpers

/** A transport that lists names, counting model reads and how many ran at once. */
function listing(names: string[], key: "models" | "data", field: "name" | "id") {
	const log = { reads: 0, flying: 0, most: 0 };
	const read = async (): Promise<unknown> => {
		log.reads += 1;
		log.flying += 1;
		log.most = Math.max(log.most, log.flying);
		await new Promise((resolve) => setTimeout(resolve, 1));
		log.flying -= 1;
		return { capabilities: ["completion"], default_generation_settings: {} };
	};
	const transport: LocalTransport = {
		getJson: async (path) =>
			path.startsWith("/props") ? read() : { [key]: names.map((name) => ({ [field]: name })) },
		postJson: read,
	};
	return { log, transport };
}

it("a listing is read for its first distinct names, a few reads at once", async () => {
	const signal = span(systemScheduler, BUDGETS.discover);
	const ollama = listing(["a", "b", "a", "c", "d", "e"], "models", "name");
	await ollamaServer(ollama.transport, 3).models([], signal);
	expect(ollama.log.reads).toBe(3);
	const llama = listing(["a", "b", "a", "c", "d", "e"], "data", "id");
	await llamacppServer(llama.transport, 3).models([], signal);
	expect(llama.log.reads).toBe(3);
	const many = listing(
		Array.from({ length: 30 }, (_, at) => `m${at}`),
		"models",
		"name",
	);
	await ollamaServer(many.transport).models([], signal);
	expect(many.log.reads).toBe(30);
	expect(many.log.most).toBe(8);
	signal.clear();
});
