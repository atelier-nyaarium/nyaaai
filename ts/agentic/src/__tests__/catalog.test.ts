// Catalog tests the cases cannot carry: a disposed catalog, and copies of views.

import { expect, it } from "bun:test";
import type { ModelRow } from "@nyaaai/contract/shapes/model";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedModelRegistry } from "@nyaaai/testkit/scriptedPorts";
import { ModelCatalog } from "../kinds/openrouter/catalog";
import { MODELS } from "../models/curated";

////////////////////////////////
//  Fixtures

const START = 1_790_337_600_000;

const ROWS: ModelRow[] = MODELS.filter((row) => ["openai/gpt-6-luna", "anthropic/claude-opus-5.5"].includes(row.key));

const MODELS_BODY = {
	data: [{ id: "openai/gpt-6-luna", reasoning: { supported_efforts: ["low"] } }, { id: "anthropic/claude-opus-5.5" }],
};

const ZDR_BODY = {
	data: [
		{ model_id: "openai/gpt-6-luna", tag: "alpha", supported_parameters: ["tools", "reasoning"] },
		{ model_id: "anthropic/claude-opus-5.5", tag: "beta", supported_parameters: ["tools", "reasoning"] },
	],
};

function catalogOf(rows: ReturnType<typeof scriptedModelRegistry>, wheel: TimerWheel) {
	return new ModelCatalog({
		transport: { getJson: async (path) => (path === "/models" ? MODELS_BODY : ZDR_BODY) },
		scheduler: fakeScheduler(wheel),
		rows,
	});
}

////////////////////////////////
//  Tests

it("a catalog that is disposed no longer hears the registry", async () => {
	const registry = scriptedModelRegistry(ROWS);
	const catalog = catalogOf(registry, new TimerWheel(START));
	const first = await catalog.views();
	catalog.dispose();
	registry.setRows(ROWS.slice(0, 1));
	expect(await catalog.views()).toEqual(first);
	expect(registry.reads()).toBe(1);
});

it("a caller's edit of its views never reaches the next caller", async () => {
	const catalog = catalogOf(scriptedModelRegistry(ROWS), new TimerWheel(START));
	const first = await catalog.views();
	const pristine = JSON.stringify(await catalogOf(scriptedModelRegistry(ROWS), new TimerWheel(START)).views());
	first.reverse();
	for (const view of first) {
		view.efforts.push("max");
		view.hosts.length = 0;
		view.status = "retired";
	}
	const second = await catalog.views();
	expect(JSON.stringify(second)).toBe(pristine);
});
