// Virtual-time scheduler.

import type { Scheduler } from "@nyaaai/contract/time";

////////////////////////////////
//  Interfaces & Types

export interface Fired {
	tag: string;
	atMs: number;
}

interface Timer {
	tag: string;
	atMs: number;
	run?: () => void;
}

////////////////////////////////
//  Classes

/** Deadline order; insertion order on ties. */
export class TimerWheel {
	private timers: Timer[] = [];

	constructor(private atMs = 0) {}

	nowMs = (): number => this.atMs;

	/** Re-setting a tag replaces its timer. */
	after(ms: number, tag: string, run?: () => void): void {
		this.cancel(tag);
		this.timers.push({ tag, atMs: this.atMs + Math.max(0, ms), ...(run ? { run } : {}) });
	}

	/** Whether a timer was waiting. */
	cancel(tag: string): boolean {
		const before = this.timers.length;
		this.timers = this.timers.filter((timer) => timer.tag !== tag);
		return this.timers.length < before;
	}

	waiting = (): string[] => this.timers.map((timer) => timer.tag);

	/** Newly due timers also fire. */
	advance(ms: number): Fired[] {
		const until = this.atMs + Math.max(0, ms);
		const fired: Fired[] = [];
		for (;;) {
			const due = this.timers.filter((timer) => timer.atMs <= until);
			const next = due.reduce<Timer | undefined>(
				(held, timer) => (held && held.atMs <= timer.atMs ? held : timer),
				undefined,
			);
			if (!next) break;
			this.timers = this.timers.filter((timer) => timer !== next);
			this.atMs = next.atMs;
			fired.push({ tag: next.tag, atMs: next.atMs });
			next.run?.();
		}
		this.atMs = until;
		return fired;
	}
}

////////////////////////////////
//  Functions & Helpers

/** Virtual-time scheduler. */
export function fakeScheduler(wheel: TimerWheel = new TimerWheel()): Scheduler & { wheel: TimerWheel } {
	let sleeps = 0;
	return {
		wheel,
		nowMs: wheel.nowMs,
		sleep: (ms, signal) =>
			new Promise((resolve, reject) => {
				if (signal?.aborted) return reject(signal.reason);
				sleeps += 1;
				const tag = `sleep-${sleeps}`;
				const stop = () => {
					wheel.cancel(tag);
					reject(signal?.reason);
				};
				wheel.after(ms, tag, () => {
					signal?.removeEventListener("abort", stop);
					resolve();
				});
				signal?.addEventListener("abort", stop, { once: true });
			}),
	};
}
