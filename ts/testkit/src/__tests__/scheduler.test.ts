// Fake scheduler tests.

import { describe, expect, it } from "bun:test";
import { fakeScheduler } from "../scheduler";

////////////////////////////////
//  Tests

describe("the fake scheduler", () => {
	it("ends sleeps in time order as the clock advances, and never on its own", async () => {
		const scheduler = fakeScheduler();
		const woke: string[] = [];
		const late = scheduler.sleep(300).then(() => woke.push(`late@${scheduler.nowMs()}`));
		const early = scheduler.sleep(100).then(async () => {
			woke.push(`early@${scheduler.nowMs()}`);
			await scheduler.sleep(50);
			woke.push(`chained@${scheduler.nowMs()}`);
		});
		await Promise.resolve();
		expect(woke).toEqual([]);

		scheduler.wheel.advance(100);
		await Bun.sleep(0);
		expect(woke).toEqual(["early@100"]);

		scheduler.wheel.advance(1000);
		await Promise.all([late, early]);
		expect(woke).toEqual(["early@100", "chained@1100", "late@1100"]);
		expect(scheduler.wheel.waiting()).toEqual([]);
	});

	it("fails an aborted sleep and drops its timer", async () => {
		const scheduler = fakeScheduler();
		const stop = new AbortController();
		const slept = scheduler.sleep(100, stop.signal).then(
			() => "woke",
			(reason) => `stopped: ${reason}`,
		);
		stop.abort("deadline");
		expect(await slept).toBe("stopped: deadline");
		expect(scheduler.wheel.advance(500)).toEqual([]);

		const already = await scheduler.sleep(1, stop.signal).then(
			() => "woke",
			() => "stopped",
		);
		expect(already).toBe("stopped");
	});
});
