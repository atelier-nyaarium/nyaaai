import { expect, it } from "bun:test";
import { systemScheduler } from "../time";

it("waits zero for a delay that is negative or not finite", async () => {
	for (const ms of [-5, Number.NaN, Number.NEGATIVE_INFINITY, Number.POSITIVE_INFINITY]) {
		await systemScheduler.sleep(ms);
		let ran = false;
		systemScheduler.schedule(ms, () => {
			ran = true;
		});
		await systemScheduler.sleep(5);
		expect(ran).toBe(true);
	}
});

it("holds a delay past the platform timer limit until cancelled", async () => {
	let ran = false;
	const cancel = systemScheduler.schedule(2 ** 31, () => {
		ran = true;
	});
	const stop = new AbortController();
	let slept = false;
	const sleeping = systemScheduler.sleep(2 ** 31 + 5, stop.signal).then(
		() => {
			slept = true;
		},
		() => undefined,
	);
	await systemScheduler.sleep(30);
	expect([ran, slept]).toEqual([false, false]);
	cancel();
	stop.abort();
	await sleeping;
	expect([ran, slept]).toEqual([false, false]);
});
