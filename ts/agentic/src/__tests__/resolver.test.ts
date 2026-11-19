// What the cases cannot say: a store that rejects, and the table's edges.

import { describe, expect, it } from "bun:test";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedEntryKeys, scriptedModelRegistry, scriptedProviderList } from "@nyaaai/testkit/scriptedPorts";
import { KINDS, kindLines } from "../provider/kind";
import { keySourceOf, ModelResolver } from "../provider/resolver";

////////////////////////////////
//  Tests

describe("ModelResolver", () => {
	const FIRST = { id: "a", kind: "openrouter", label: "First" } as const;

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
		expect(() => resolver.route(FIRST)).toThrow();
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
