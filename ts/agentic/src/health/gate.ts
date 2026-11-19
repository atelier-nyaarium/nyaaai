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

	constructor(private readonly d: GateDeps) {
		this.id = d.ids.newId();
		this.pauseMs = d.pauseMs ?? PAUSE_MS;
		this.maxPauses = d.maxPauses ?? MAX_PAUSES;
	}

	/** Retries get front priority. */
	admit(
		model: string,
		signal: AbortSignal,
		onWait?: (waiting: GateWaiting) => void,
		again = false,
	): Promise<Admission> {
		if (signal.aborted) return Promise.resolve({ kind: "stopped" });
		const gate = this.gateOf(model);
		if (gate.phase === "open")
			return Promise.resolve(this.shifting(model, gate, () => this.go(model, gate, false)));
		if (gate.phase === "probing" && gate.probe === null)
			return Promise.resolve(this.shifting(model, gate, () => this.go(model, gate, true)));
		return new Promise((resolve) => {
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
			onWait?.(this.waitOf(model, gate));
		});
	}

	/** Nonblocking, even for probes. */
	tryAdmit(model: string): Ticket | null {
		const gate = this.gateOf(model);
		if (!readyOf(gate)) return null;
		const admission = this.shifting(model, gate, () => this.go(model, gate, gate.phase === "probing"));
		return admission.kind === "go" ? admission.ticket : null;
	}

	/** Open or probe-ready. */
	ready(model: string): boolean {
		const gate = this.models.get(model);
		return gate === undefined || readyOf(gate);
	}

	/** Pauses an open model. */
	cool(model: string, retryAfterMs?: number): void {
		const gate = this.gateOf(model);
		if (gate.phase !== "open") return;
		this.shifting(model, gate, () => {
			gate.pauses = 1;
			this.pause(model, gate, retryAfterMs);
		});
	}

	/** Reopens matching gates. */
	clear(matches: (model: string) => boolean): void {
		for (const [model, gate] of this.models) {
			if (!matches(model) || gate.phase === "open") continue;
			this.shifting(model, gate, () => {
				gate.cancel?.();
				gate.cancel = null;
				gate.probe = null;
				this.reopen(gate, (waiter) => waiter.admit(this.go(model, gate, false)));
			});
		}
	}

	/** Every admitted ticket settles once. */
	settle(ticket: Ticket, outcome: GateOutcome, retryAfterMs?: number): GateSettled {
		const gate = this.gateOf(ticket.model);
		return this.shifting(ticket.model, gate, () => {
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
		});
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

	/** Told a model whose gate turned ready or not ready; the result unsubscribes. */
	onChange(listener: (model: string) => void): () => void {
		this.listeners.add(listener);
		return () => {
			this.listeners.delete(listener);
		};
	}

	dispose(): void {
		const turned: string[] = [];
		for (const [model, gate] of this.models) {
			if (!readyOf(gate)) turned.push(model);
			gate.cancel?.();
			for (const waiter of gate.queue.splice(0)) waiter.admit({ kind: "stopped" });
		}
		this.models.clear();
		for (const model of turned) this.tell(model);
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
		const waitMs = Math.max(this.pauseMs, retryAfterMs ?? 0);
		gate.phase = "paused";
		gate.resumeAt = this.d.scheduler.nowMs() + waitMs;
		gate.cancel?.();
		gate.cancel = this.d.scheduler.schedule(waitMs, () =>
			this.shifting(model, gate, () => {
				gate.cancel = null;
				gate.phase = "probing";
				this.nextProbe(model, gate);
			}),
		);
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
		return { kind: "go", ticket };
	}

	private notify(model: string, gate: ModelGate): void {
		const waiting = this.waitOf(model, gate);
		for (const waiter of gate.queue) waiter.onWait?.(waiting);
	}

	private waitOf(model: string, gate: ModelGate): GateWaiting {
		return {
			model,
			resumeAt: gate.phase === "paused" ? gate.resumeAt : null,
			pause: gate.pauses,
			of: this.maxPauses,
		};
	}

	/** Runs a change of one gate and tells when it turned ready or not. */
	private shifting<T>(model: string, gate: ModelGate, run: () => T): T {
		const was = readyOf(gate);
		const result = run();
		if (readyOf(gate) !== was) this.tell(model);
		return result;
	}

	private tell(model: string): void {
		for (const listener of [...this.listeners]) {
			try {
				listener(model);
			} catch {
				// A listener cannot change the gate.
			}
		}
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
