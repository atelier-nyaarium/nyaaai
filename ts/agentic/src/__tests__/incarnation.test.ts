// What the cases cannot say: one health, two registries.

import { describe, expect, it } from "bun:test";
import { countedIds } from "@nyaaai/contract/id";
import { fakeScheduler, TimerWheel } from "@nyaaai/testkit/scheduler";
import { scriptedEntryKeys, scriptedProviderList } from "@nyaaai/testkit/scriptedPorts";
import { ProviderHealth } from "../health/health";
import { EntryRegistry } from "../provider/incarnation";
import { KINDS } from "../provider/kind";
import type { Resolved } from "../provider/resolver";

describe("ProviderHealth", () => {
	const FIRST = { id: "a", kind: "openrouter", label: "First" } as const;

	it("refuses a route of another registry at every door", () => {
		const keys = scriptedEntryKeys();
		const own = new EntryRegistry({ list: scriptedProviderList([FIRST]), keys });
		const other = new EntryRegistry({ list: scriptedProviderList([FIRST]), keys });
		const health = new ProviderHealth({
			scheduler: fakeScheduler(new TimerWheel(0)),
			ids: countedIds("gate-"),
			entries: own,
			keys,
			kinds: KINDS,
			kindDeps: () => {
				throw new Error("Unreachable.");
			},
		});
		const incarnation = other.incarnations()[0];
		if (incarnation === undefined) throw new Error("The entry is listed.");
		const route = { entry: FIRST, key: "m", incarnation } as unknown as Resolved;
		const signal = new AbortController().signal;
		const doors = [
			() => health.admit(route, signal),
			() => health.tryAdmit(route),
			() => health.deferred(route),
			() => health.blocked(route),
			() => health.served(route, { input: 1, output: 1, billingSource: "balance" }),
			() => health.failed(route, { category: "network", status: "network", message: "down" }, false),
			() => health.gateState(route),
			() => health.billed("a", { source: "go" }, undefined, incarnation),
		];
		for (const door of doors) expect(door).toThrow();
		health.dispose();
	});
});
