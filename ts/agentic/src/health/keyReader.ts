// One entry's key status; a newer read drops the last.

import type { KeyState } from "@nyaaai/contract/shapes/provider";
import { deadline, type Scheduler } from "@nyaaai/contract/time";

////////////////////////////////
//  Constants

export const KEY_CHECK_MS = 30_000;

////////////////////////////////
//  Classes

export class KeyReader {
	private abort: AbortController | undefined;
	private readonly running = new Set<Promise<void>>();

	constructor(
		/** The deadline of each check comes from it. */
		private readonly scheduler: Scheduler,
		/** Undefined: the check ended without effect. */
		private readonly check: (signal: AbortSignal) => Promise<KeyState | undefined>,
		private readonly shown: (state: KeyState) => void,
	) {}

	/** Shows `checking`, then the state; a failed or late check shows `unreachable`. */
	read(): Promise<void> {
		this.abort?.abort();
		const abort = new AbortController();
		this.abort = abort;
		this.shown({ kind: "checking" });
		const limit = deadline(this.scheduler, KEY_CHECK_MS, abort.signal);
		const raced = new Promise<KeyState | undefined>((resolve, reject) => {
			const end = () => reject(limit.signal.reason);
			if (limit.signal.aborted) return end();
			limit.signal.addEventListener("abort", end, { once: true });
			try {
				this.check(limit.signal).then(resolve, reject);
			} catch (error) {
				reject(error);
			}
		});
		const ended = raced.then(
			(state) => {
				if (state !== undefined && !abort.signal.aborted) this.shown(state);
			},
			() => {
				if (!abort.signal.aborted) this.shown({ kind: "unreachable" });
			},
		);
		const running = ended.finally(() => {
			limit.clear();
			this.running.delete(running);
		});
		this.running.add(running);
		return running;
	}

	/** A check is running, including dropped ones until they end. */
	busy(): boolean {
		return this.running.size > 0;
	}

	/** Answers when no check is running, having waited for those that were. */
	async idle(): Promise<void> {
		while (this.running.size > 0) await Promise.allSettled([...this.running]);
	}

	dispose(): void {
		this.abort?.abort();
	}
}
