import { expect, it } from "bun:test";
import { spawnSync } from "node:child_process";
import { systemClock, systemScheduler } from "../time";

it("waits zero for a delay that is negative or not finite", async () => {
	for (const ms of [-5, Number.NaN]) {
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

const JANUARY = 1_768_478_400_000;
const JULY = 1_784_116_800_000;

/** The offsets a child process reads from the system clock and scheduler under `TZ`. */
function offsetsIn(zone: string, instants: number[]): number[][] {
	const time = new URL("../time.ts", import.meta.url).pathname;
	const script = `import { systemClock, systemScheduler } from ${JSON.stringify(time)};
const at = ${JSON.stringify(instants)};
console.log(JSON.stringify([at.map((ms) => systemClock.utcOffsetMinutes(ms)), at.map((ms) => systemScheduler.utcOffsetMinutes(ms))]));`;
	const ran = spawnSync(process.execPath, ["-e", script], { env: { ...process.env, TZ: zone }, encoding: "utf8" });
	return JSON.parse(ran.stdout) as number[][];
}

it("answers the platform's offset under a set zone, for the clock and the scheduler", () => {
	const wanted: Array<[string, number[]]> = [
		["America/New_York", [-300, -240]],
		["Asia/Kolkata", [330, 330]],
		["Asia/Kathmandu", [345, 345]],
		["America/St_Johns", [-210, -150]],
		["UTC", [0, 0]],
	];
	for (const [zone, minutes] of wanted) {
		expect({ zone, rows: offsetsIn(zone, [JANUARY, JULY]) }).toEqual({ zone, rows: [minutes, minutes] });
	}
});

it("answers zero for an instant the platform cannot place", () => {
	for (const ms of [Number.NaN, Number.POSITIVE_INFINITY, 8_640_000_000_000_001]) {
		expect(systemClock.utcOffsetMinutes(ms)).toBe(0);
	}
});
