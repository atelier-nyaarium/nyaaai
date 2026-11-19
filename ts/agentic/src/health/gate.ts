// Pauses each model while one request probes recovery.

import type { Ids } from "@nyaaai/contract/id";
import { sortedTexts } from "@nyaaai/contract/order";
import type { Admission, GateOutcome, GateSettled, GateWaiting, Ticket } from "@nyaaai/contract/shapes/health";
import type { Scheduler } from "@nyaaai/contract/time";

////////////////////////////////
//  Interfaces & Types

export interface GateDeps {
	/** Pauses are scheduled on it and read from it. */
	scheduler: Scheduler;
	/** Asked once for the gate's name, which every ticket carries. */
	ids: Ids;
	pauseMs?: number;
	maxPauses?: number;
	/** Told each ticket in the gate call that makes it. */
	onTicket?: (ticket: Ticket) => void;
}

/** A model's gate as data. */
export interface GateState {
	phase: "open" | "paused" | "probing";
	/** Bumped by every reopening. */
	generation: number;
	pauses: number;
	/** The end of the last pause; 0 before any. */
	resumeAt: number;
	probe: Ticket | null;
	/** Requests waiting for a turn. */
	queued: number;
}

interface Waiter {
	admit(admission: Admission): void;
	onWait: ((waiting: GateWaiting) => void) | undefined;
}

interface ModelGate {
	phase: GateState["phase"];
	generation: number;
	pauses: number;
	resumeAt: number;
	probe: Ticket | null;
	queue: Waiter[];
	cancel: (() => void) | null;
}

////////////////////////////////
//  Constants

const PAUSE_MS = 60_000;

const MAX_PAUSES = 10;

/** A server's asked wait stops here. */
const MAX_PAUSE_MS = 60 * 60_000;

////////////////////////////////
//  Functions & Helpers

/** Two tickets are the same when every field is equal. */
export function sameTicket(left: Ticket, right: Ticket): boolean {
	return (
		left.gate === right.gate &&
		left.model === right.model &&
		left.generation === right.generation &&
		left.probe === right.probe &&
		left.serial === right.serial
	);
}

/** Open, or probing with no probe out. */
const readyOf = (gate: ModelGate): boolean =>
	gate.phase === "open" || (gate.phase === "probing" && gate.probe === null);

////////////////////////////////
//  Classes

export class ProviderGate {
	private readonly models = new Map<string, ModelGate>();
	private readonly listeners = new Set<(model: string) => void>();
	private readonly id: string;
	private readonly pauseMs: number;
	private readonly maxPauses: number;
	private serial = 0;
	private disposed = false;
	/** Callbacks owed once the gate's own state is whole. */
	private readonly effects: Array<() => void> = [];
	private depth = 0;
	private flushing = false;

	constructor(private readonly d: GateDeps) {
		this.id = d.ids.newId();
		this.pauseMs = d.pauseMs ?? PAUSE_MS;
		this.maxPauses = d.maxPauses ?? MAX_PAUSES;
	}

	/** Retries get front priority. A disposed gate admits nothing. */
	admit(
		model: string,
		signal: AbortSignal,
		onWait?: (waiting: GateWaiting) => void,
		again = false,
	): Promise<Admission> {
		if (signal.aborted || this.disposed) return Promise.resolve({ kind: "stopped" });
		return this.act(() => {
			const gate = this.gateOf(model);
			if (gate.phase === "open")
				return Promise.resolve(this.shifting(model, gate, () => this.go(model, gate, false)));
			if (gate.phase === "probing" && gate.probe === null)
				return Promise.resolve(this.shifting(model, gate, () => this.go(model, gate, true)));
			return new Promise<Admission>((resolve) => {
				const stop = () => {
					gate.queue = gate.queue.filter((queued) => queued !== waiter);
					resolve({ kind: "stopped" });
				};
				const waiter: Waiter = {
					admit: (admission) => {
						signal.removeEventListener("abort", stop);
						resolve(admission);
					},
					onWait,
				};
				signal.addEventListener("abort", stop, { once: true });
				if (again) gate.queue.unshift(waiter);
				else gate.queue.push(waiter);
				const waiting = this.waitOf(model, gate);
				if (onWait !== undefined) this.effects.push(() => onWait(waiting));
			});
		});
	}

	/** Nonblocking, even for probes. */
	tryAdmit(model: string): Ticket | null {
		if (this.disposed) return null;
		return this.act(() => {
			const gate = this.gateOf(model);
			if (!readyOf(gate)) return null;
			const admission = this.shifting(model, gate, () => this.go(model, gate, gate.phase === "probing"));
			return admission.kind === "go" ? admission.ticket : null;
		});
	}

	/** Open or probe-ready. */
	ready(model: string): boolean {
		const gate = this.models.get(model);
		return gate === undefined || readyOf(gate);
	}

	/** Pauses an open model. */
	cool(model: string, retryAfterMs?: number): void {
		if (this.disposed) return;
		this.act(() => {
			const gate = this.gateOf(model);
			if (gate.phase !== "open") return;
			this.shifting(model, gate, () => {
				gate.pauses = 1;
				this.pause(model, gate, retryAfterMs);
			});
		});
	}

	/** Reopens matching gates. */
	clear(matches: (model: string) => boolean): void {
		if (this.disposed) return;
		this.act(() => {
			for (const [model, gate] of this.models) {
				if (!matches(model) || gate.phase === "open") continue;
				this.shifting(model, gate, () => {
					gate.cancel?.();
					gate.cancel = null;
					gate.probe = null;
					this.reopen(gate, (waiter) => waiter.admit(this.go(model, gate, false)));
				});
			}
		});
	}

	/** Settles each ticket once; an earlier request's success resets the give-up count even while paused. */
	settle(ticket: Ticket, outcome: GateOutcome, retryAfterMs?: number): GateSettled {
		if (this.disposed || ticket.gate !== this.id) return "done";
		const gate = this.models.get(ticket.model);
		if (gate === undefined) return "done";
		return this.act(() =>
			this.shifting(ticket.model, gate, () => {
				if (gate.probe !== null && sameTicket(gate.probe, ticket))
					return this.settleProbe(ticket.model, gate, outcome, retryAfterMs);
				if (outcome !== "overloaded") {
					if (outcome === "ok") gate.pauses = 0;
					return "done";
				}
				if (gate.phase !== "open") return "again";
				// Ignore stale probe refusals.
				if (ticket.generation < gate.generation) return "again";
				gate.pauses = 1;
				this.pause(ticket.model, gate, retryAfterMs);
				return "again";
			}),
		);
	}

	/** One model's gate; none until the gate is first used. */
	stateOf(model: string): GateState | undefined {
		const gate = this.models.get(model);
		if (gate === undefined) return undefined;
		return {
			phase: gate.phase,
			generation: gate.generation,
			pauses: gate.pauses,
			resumeAt: gate.resumeAt,
			probe: gate.probe === null ? null : { ...gate.probe },
			queued: gate.queue.length,
		};
	}

	/** Models whose gate is not ready, in code unit order. */
	holding(): string[] {
		const held: string[] = [];
		for (const [model, gate] of this.models) if (!readyOf(gate)) held.push(model);
		return sortedTexts(held);
	}

	/** Told when a model's gate turns ready or not; returns unsubscribe. */
	onChange(listener: (model: string) => void): () => void {
		this.listeners.add(listener);
		return () => {
			this.listeners.delete(listener);
		};
	}

	dispose(): void {
		if (this.disposed) return;
		this.act(() => {
			this.disposed = true;
			const turned: string[] = [];
			for (const [model, gate] of this.models) {
				if (!readyOf(gate)) turned.push(model);
				gate.cancel?.();
				for (const waiter of gate.queue.splice(0)) waiter.admit({ kind: "stopped" });
			}
			this.models.clear();
			for (const model of turned) this.tell(model);
		});
		this.listeners.clear();
	}

	private settleProbe(model: string, gate: ModelGate, outcome: GateOutcome, retryAfterMs?: number): GateSettled {
		gate.probe = null;
		switch (outcome) {
			case "ok":
				this.reopen(gate, (waiter) => waiter.admit(this.go(model, gate, false)));
				return "done";
			case "overloaded":
				if (gate.pauses >= this.maxPauses) {
					this.reopen(gate, (waiter) => waiter.admit({ kind: "gaveUp" }));
					return "gaveUp";
				}
				gate.pauses++;
				this.pause(model, gate, retryAfterMs);
				return "again";
			case "failed":
			case "cancelled":
				this.nextProbe(model, gate);
				return "done";
		}
	}

	private reopen(gate: ModelGate, release: (waiter: Waiter) => void): void {
		gate.phase = "open";
		gate.generation++;
		gate.pauses = 0;
		for (const waiter of gate.queue.splice(0)) release(waiter);
	}

	private pause(model: string, gate: ModelGate, retryAfterMs?: number): void {
		const waitMs = Math.max(this.pauseMs, Math.min(retryAfterMs ?? 0, MAX_PAUSE_MS));
		gate.phase = "paused";
		gate.resumeAt = this.d.scheduler.nowMs() + waitMs;
		gate.cancel?.();
		gate.cancel = this.d.scheduler.schedule(waitMs, () => {
			if (this.disposed) return;
			this.act(() =>
				this.shifting(model, gate, () => {
					gate.cancel = null;
					gate.phase = "probing";
					this.nextProbe(model, gate);
				}),
			);
		});
		this.notify(model, gate);
	}

	/** Oldest waiter probes first. */
	private nextProbe(model: string, gate: ModelGate): void {
		const waiter = gate.queue.shift();
		if (waiter !== undefined) waiter.admit(this.go(model, gate, true));
		this.notify(model, gate);
	}

	private go(model: string, gate: ModelGate, probe: boolean): Admission {
		this.serial += 1;
		const ticket: Ticket = Object.freeze({
			gate: this.id,
			model,
			generation: gate.generation,
			probe,
			serial: this.serial,
		});
		if (probe) gate.probe = ticket;
		this.d.onTicket?.(ticket);
		return { kind: "go", ticket };
	}

	/** Each queued waiter is told, once the gate's state is whole. */
	private notify(model: string, gate: ModelGate): void {
		const waiting = this.waitOf(model, gate);
		for (const waiter of gate.queue) {
			const { onWait } = waiter;
			if (onWait !== undefined) this.effects.push(() => onWait(waiting));
		}
	}

	private waitOf(model: string, gate: ModelGate): GateWaiting {
		return {
			model,
			resumeAt: gate.phase === "paused" ? gate.resumeAt : null,
			pause: gate.pauses,
			of: this.maxPauses,
		};
	}

	/** Runs a gate change and tells if readiness flipped. */
	private shifting<T>(model: string, gate: ModelGate, run: () => T): T {
		const was = readyOf(gate);
		const result = run();
		if (readyOf(gate) !== was) this.tell(model);
		return result;
	}

	private tell(model: string): void {
		for (const listener of [...this.listeners]) {
			this.effects.push(() => {
				try {
					listener(model);
				} catch {
					// A listener cannot change the gate.
				}
			});
		}
	}

	/** Runs a change, then its callbacks in order; nested calls queue behind. */
	private act<T>(run: () => T): T {
		this.depth += 1;
		let result: T;
		try {
			result = run();
		} finally {
			this.depth -= 1;
		}
		if (this.depth === 0 && !this.flushing) {
			this.flushing = true;
			try {
				for (let effect = this.effects.shift(); effect !== undefined; effect = this.effects.shift()) {
					try {
						effect();
					} catch {
						// A callback cannot change the gate.
					}
				}
			} finally {
				this.flushing = false;
			}
		}
		return result;
	}

	private gateOf(model: string): ModelGate {
		let gate = this.models.get(model);
		if (gate === undefined) {
			gate = { phase: "open", generation: 0, pauses: 0, resumeAt: 0, probe: null, queue: [], cancel: null };
			this.models.set(model, gate);
		}
		return gate;
	}
}
