// Catalog tests the cases cannot carry: identity of reused views, and the platform's own reading of a day.

import { expect, it } from "bun:test";
import type { ModelRow } from "@nyaaai/contract/shapes/model";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedModelRegistry } from "@nyaaai/testkit/scriptedPorts";
import { browseModels } from "../kinds/openrouter/browse";
import { expiryOf, ModelCatalog, parseEndpoints } from "../kinds/openrouter/catalog";
import { MODELS } from "../models/curated";
import { ANALYSIS, CHAT } from "../models/uses";

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

it("a join is reused, not rebuilt, while rows, data, the flag and the removal dates stand", async () => {
	const registry = scriptedModelRegistry(ROWS);
	const wheel = new TimerWheel(START);
	const catalog = catalogOf(registry, wheel);
	const first = await catalog.views();
	expect(await catalog.views()).toEqual(first);
	expect(registry.reads()).toBe(1);
	await catalog.views(CHAT, false);
	expect(registry.reads()).toBe(2);
	registry.setRows(ROWS);
	await catalog.views(CHAT, false);
	expect(registry.reads()).toBe(3);
	await catalog.views(CHAT, false);
	expect(registry.reads()).toBe(3);
	await catalog.views(ANALYSIS);
	expect(registry.reads()).toBe(4);
});

it("a catalog that is disposed no longer hears the registry", async () => {
	const registry = scriptedModelRegistry(ROWS);
	const catalog = catalogOf(registry, new TimerWheel(START));
	const first = await catalog.views();
	catalog.dispose();
	registry.setRows(ROWS.slice(0, 1));
	expect(await catalog.views()).toEqual(first);
	expect(registry.reads()).toBe(1);
});

it("the browse list of one fetch is the same list until a removal day passes", () => {
	const endpoints = parseEndpoints(ZDR_BODY) ?? new Map();
	const raw = [{ id: "openai/gpt-6-luna", expiration_date: "2026-10-01" }];
	const before = browseModels({ raw, endpoints }, START);
	expect(browseModels({ raw, endpoints }, START + 1000)).toBe(before);
	expect(browseModels({ raw: [...raw], endpoints }, START)).not.toBe(before);
	expect(browseModels({ raw, endpoints }, Date.UTC(2026, 9, 1))).not.toBe(before);
});

it("a day or an ISO time reads as the platform reads it", () => {
	const spellings = [
		"2026-10-01",
		"1970-01-01",
		"2028-02-29",
		"2026-10-01T00:00:00Z",
		"2026-10-01T00:00:00.5Z",
		"2026-10-01T12:30:45.123456Z",
		"2026-10-01T12:30:45+02:00",
		"2026-10-01T12:30:45-08:00",
		"2026-12-31T23:59:59Z",
	];
	expect(spellings.map(expiryOf)).toEqual(spellings.map((text) => Date.parse(text)));
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
	expect(second.map((view) => view.row.key)).toEqual(ROWS.map((row) => row.key));
	expect(second.every((view) => view.status !== "retired")).toBe(true);
	expect(second.some((view) => view.efforts.includes("max"))).toBe(false);
});
