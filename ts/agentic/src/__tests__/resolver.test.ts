// What the cases cannot say: identity of a made route, and the order of the table.

import { describe, expect, it } from "bun:test";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedEntryKeys, scriptedModelRegistry, scriptedProviderList } from "@nyaaai/testkit/scriptedPorts";
import { OPENCODE } from "../kinds/opencode/kind";
import { OPENROUTER } from "../kinds/openrouter/kind";
import { KINDS, kindLines } from "../provider/kind";
import { keySourceOf, ModelResolver } from "../provider/resolver";

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

	it("stops listening and builds nothing once disposed, and unsubscribes once", () => {
		let unsubscribed = 0;
		const list = scriptedProviderList([FIRST]);
		const resolver = new ModelResolver({
			list: {
				entries: () => list.entries(),
				onChange: (listener) => {
					const off = list.onChange(listener);
					return () => {
						unsubscribed += 1;
						off();
					};
				},
			},
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
		resolver.route(FIRST);
		resolver.dispose();
		resolver.dispose();
		expect(unsubscribed).toBe(1);
		expect(() => resolver.route(FIRST)).toThrow("The resolver is disposed.");
	});

	it("reads a key store that rejects as unready and as no key", async () => {
		const keys = {
			get: () => Promise.reject(new Error("The store is unreadable.")),
			onChange: () => () => {},
		};
		const list = scriptedProviderList([FIRST]);
		const resolver = new ModelResolver({
			list,
			keys,
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
		expect(await resolver.reach(FIRST)).toBe("unreachable");
		expect(await keySourceOf(keys, list, FIRST).get()).toBeUndefined();
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

describe("kindLines", () => {
	it("reads an empty list of plans as none", () => {
		const open = KINDS.opencode;
		if (open === undefined) throw new Error("No kind.");
		expect(kindLines({ opencode: { ...open, plans: [] } }).map((line) => [line.plan, line.label])).toEqual([
			[undefined, "OpenCode"],
		]);
	});
});

describe("a catalog that fails to stop", () => {
	it("does not keep the others from going", () => {
		const FIRST = { id: "a", kind: "openrouter", label: "First" } as const;
		const SECOND = { id: "b", kind: "openrouter", label: "Second" } as const;
		const stopped: string[] = [];
		const base = KINDS.openrouter;
		if (base === undefined) throw new Error("No kind.");
		const kind = {
			...base,
			catalog: (entry: { id: string }) => ({
				views: async () => [],
				idle: async () => {},
				dispose: () => {
					stopped.push(entry.id);
					throw new Error("Stopping failed.");
				},
			}),
		};
		const list = scriptedProviderList([FIRST, SECOND]);
		const resolver = new ModelResolver({
			list,
			keys: scriptedEntryKeys(),
			registry: scriptedModelRegistry(),
			kinds: { openrouter: kind },
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
		resolver.route(FIRST);
		resolver.route(SECOND);
		list.set([]);
		expect(stopped).toEqual(["a", "b"]);
	});
});
