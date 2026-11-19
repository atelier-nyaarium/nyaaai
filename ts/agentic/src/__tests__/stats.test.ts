// The stats queue on a scheduler that runs a zero-delay timer before it returns.

import { expect, it } from "bun:test";
import type { Scheduler } from "@nyaaai/contract/time";
import { StatsQueue } from "../usage/stats";

////////////////////////////////
//  Tests

it("a timer that fires before schedule returns leaves the queue working", async () => {
	let now = 600_000;
	const later: Array<{ at: number; run: () => void }> = [];
	const scheduler: Scheduler = {
		nowMs: () => now,
		utcOffsetMinutes: () => 0,
		sleep: async () => {},
		schedule: (ms, run) => {
			if (ms === 0) {
				run();
				return () => {};
			}
			const held = { at: now + ms, run };
			later.push(held);
			return () => {
				later.splice(later.indexOf(held), 1);
			};
		},
	};
	const read: string[] = [];
	const queue = new StatsQueue({
		scheduler,
		usage: { fill: () => {}, filled: async () => false, unfilled: async () => [] },
		read: async (_entry, path) => {
			read.push(path);
			return { data: { latency: 1 } };
		},
	});
	queue.add("a1", "gen-a1", 0);
	await queue.idle();
	expect(read).toEqual(["/generation?id=gen-a1"]);
	queue.add("a2", "gen-a2", 590_000);
	now = 590_000 + 7 * 60_000;
	later[0]?.run();
	await queue.idle();
	expect(read).toEqual(["/generation?id=gen-a1", "/generation?id=gen-a2"]);
});
