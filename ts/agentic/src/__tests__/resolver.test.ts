// What the cases cannot say: identity of a made route, and the order of the table.

import { describe, expect, it } from "bun:test";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedEntryKeys, scriptedModelRegistry, scriptedProviderList } from "@nyaaai/testkit/scriptedPorts";
import { OPENCODE } from "../kinds/opencode/kind";
import { OPENROUTER } from "../kinds/openrouter/kind";
import { KINDS } from "../provider/kind";
import { ModelResolver } from "../provider/resolver";

////////////////////////////////
//  Functions & Helpers

function resolverOver(entries: Parameters<typeof scriptedProviderList>[0]) {
	const list = scriptedProviderList(entries);
	const resolver = new ModelResolver({
		list,
		keys: scriptedEntryKeys(),
		registry: scriptedModelRegistry(),
		kinds: KINDS,
		kindDeps: () => ({
			fetch: async () => {
				throw new Error("No network.");
			},
			scheduler: fakeScheduler(new TimerWheel(0)),
			log: () => {},
			userAgent: "tests/1.0",
		}),
		zdrOnly: () => true,
	});
	return { list, resolver };
}

////////////////////////////////
//  Tests

describe("KINDS", () => {
	it("keeps the dropdown order", () => {
		expect(Object.keys(KINDS)).toEqual(["opencode", "openrouter", "ollama", "lmstudio", "llamacpp", "compatible"]);
		expect(KINDS.opencode).toBe(OPENCODE);
		expect(KINDS.openrouter).toBe(OPENROUTER);
	});
});

describe("ModelResolver", () => {
	const FIRST = { id: "a", kind: "openrouter", label: "First" } as const;
	const SECOND = { id: "b", kind: "openrouter", label: "Second" } as const;

	it("hands out the one route it made while the entry stands", () => {
		const { resolver } = resolverOver([FIRST, SECOND]);
		const route = resolver.route(FIRST);
		expect(resolver.route({ ...FIRST, label: "Renamed" })).toBe(route);
		expect(resolver.route(SECOND)).not.toBe(route);
	});

	it("makes a new route once the list dropped the entry", () => {
		const { list, resolver } = resolverOver([FIRST, SECOND]);
		const route = resolver.route(FIRST);
		list.set([SECOND]);
		list.set([FIRST, SECOND]);
		expect(resolver.route(FIRST)).not.toBe(route);
	});

	it("keeps its routes after a told change that leaves the entry as it was", () => {
		const { list, resolver } = resolverOver([FIRST, SECOND]);
		const route = resolver.route(FIRST);
		list.set([SECOND, FIRST]);
		expect(resolver.route(FIRST)).toBe(route);
	});

	it("stops listening once disposed", () => {
		const { list, resolver } = resolverOver([FIRST]);
		const route = resolver.route(FIRST);
		resolver.dispose();
		list.set([]);
		expect(resolver.route(FIRST)).not.toBe(route);
	});

	it("refuses a kind it does not hold", () => {
		const list = scriptedProviderList([]);
		const resolver = new ModelResolver({
			list,
			keys: scriptedEntryKeys(),
			registry: scriptedModelRegistry(),
			kinds: {},
			kindDeps: () => {
				throw new Error("Unreachable.");
			},
			zdrOnly: () => true,
		});
		expect(() => resolver.route(FIRST)).toThrow("No openrouter provider kind.");
	});
});
