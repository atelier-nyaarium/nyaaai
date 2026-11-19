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
	book: GateBook;
	model: string;
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
		left.era === right.era &&
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

/** One key era's gate rows. */
export class GateBook {
	/** Set by `retire`. */
	retired = false;
	/** Rows by model. */
	readonly models = new Map<string, ModelGate>();

	constructor(
		/** Stamped on its tickets. */
		readonly era: number,
		/** Told with each readiness change. */
		readonly owner: string,
	) {}
}

export class ProviderGate {
	private readonly books = new Set<GateBook>();
	private readonly listeners = new Set<(owner: string, model: string) => void>();
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

	/** Retries go first. A retired book answers `retired`. */
	admit(
		book: GateBook,
		model: string,
		signal: AbortSignal,
		onWait?: (waiting: GateWaiting) => void,
		again = false,
	): Promise<Admission> {
		if (signal.aborted || this.disposed) return Promise.resolve({ kind: "stopped" });
		if (book.retired) return Promise.resolve({ kind: "retired" });
		return this.act(() => {
			const gate = this.gateOf(book, model);
			if (gate.phase === "open") return Promise.resolve(this.shifting(gate, () => this.go(gate, false)));
			if (gate.phase === "probing" && gate.probe === null)
				return Promise.resolve(this.shifting(gate, () => this.go(gate, true)));
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
				const waiting = this.waitOf(gate);
				if (onWait !== undefined) this.effects.push(() => onWait(waiting));
			});
		});
	}

	/** Nonblocking, even for probes. */
	tryAdmit(book: GateBook, model: string): Ticket | null {
		if (this.disposed || book.retired) return null;
		return this.act(() => {
			const gate = this.gateOf(book, model);
			if (!readyOf(gate)) return null;
			const admission = this.shifting(gate, () => this.go(gate, gate.phase === "probing"));
			return admission.kind === "go" ? admission.ticket : null;
		});
	}

	/** Open or probe-ready. */
	ready(book: GateBook, model: string): boolean {
		const gate = book.models.get(model);
		return gate === undefined || readyOf(gate);
	}

	/** Pauses an open model. */
	cool(book: GateBook, model: string, retryAfterMs?: number): void {
		if (this.disposed || book.retired) return;
		this.act(() => {
			const gate = this.gateOf(book, model);
			if (gate.phase !== "open") return;
			this.shifting(gate, () => {
				gate.pauses = 1;
				this.pause(gate, retryAfterMs);
			});
		});
	}

	/** Waiters are answered `retired`. */
	retire(book: GateBook): void {
		if (book.retired) return;
		book.retired = true;
		this.books.delete(book);
		if (this.disposed) return;
		this.act(() => {
			const turned: ModelGate[] = [];
			for (const gate of book.models.values()) {
				if (!readyOf(gate)) turned.push(gate);
				gate.cancel?.();
				gate.cancel = null;
				for (const waiter of gate.queue.splice(0)) waiter.admit({ kind: "retired" });
			}
			book.models.clear();
			for (const gate of turned) this.tell(gate);
		});
	}

	/** Settles each ticket once; a success resets the give-up count even while paused. */
	settle(book: GateBook, ticket: Ticket, outcome: GateOutcome, retryAfterMs?: number): GateSettled {
		if (this.disposed || book.retired || ticket.gate !== this.id || ticket.era !== book.era) return "done";
		const gate = book.models.get(ticket.model);
		if (gate === undefined) return "done";
		return this.act(() =>
			this.shifting(gate, () => {
				if (gate.probe !== null && sameTicket(gate.probe, ticket))
					return this.settleProbe(gate, outcome, retryAfterMs);
				if (outcome !== "overloaded") {
					if (outcome === "ok") gate.pauses = 0;
					return "done";
				}
				if (gate.phase !== "open") return "again";
				// Ignore stale probe refusals.
				if (ticket.generation < gate.generation) return "again";
				gate.pauses = 1;
				this.pause(gate, retryAfterMs);
				return "again";
			}),
		);
	}

	/** One model's gate; none until the gate is first used. */
	stateOf(book: GateBook, model: string): GateState | undefined {
		const gate = book.models.get(model);
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

	/** The book's models not ready, in code unit order. */
	holding(book: GateBook): string[] {
		const held: string[] = [];
		for (const [model, gate] of book.models) if (!readyOf(gate)) held.push(model);
		return sortedTexts(held);
	}

	/** Told when a model turns ready or not; returns unsubscribe. */
	onChange(listener: (owner: string, model: string) => void): () => void {
		this.listeners.add(listener);
		return () => {
			this.listeners.delete(listener);
		};
	}

	dispose(): void {
		if (this.disposed) return;
		this.act(() => {
			this.disposed = true;
			const turned: ModelGate[] = [];
			for (const book of this.books) {
				for (const gate of book.models.values()) {
					if (!readyOf(gate)) turned.push(gate);
					gate.cancel?.();
					for (const waiter of gate.queue.splice(0)) waiter.admit({ kind: "stopped" });
				}
				book.models.clear();
			}
			this.books.clear();
			for (const gate of turned) this.tell(gate);
		});
		this.listeners.clear();
	}

	private settleProbe(gate: ModelGate, outcome: GateOutcome, retryAfterMs?: number): GateSettled {
		gate.probe = null;
		switch (outcome) {
			case "ok":
				this.reopen(gate, (waiter) => waiter.admit(this.go(gate, false)));
				return "done";
			case "overloaded":
				if (gate.pauses >= this.maxPauses) {
					this.reopen(gate, (waiter) => waiter.admit({ kind: "gaveUp" }));
					return "gaveUp";
				}
				gate.pauses++;
				this.pause(gate, retryAfterMs);
				return "again";
			case "failed":
			case "cancelled":
				this.nextProbe(gate);
				return "done";
		}
	}

	private reopen(gate: ModelGate, release: (waiter: Waiter) => void): void {
		gate.phase = "open";
		gate.generation++;
		gate.pauses = 0;
		for (const waiter of gate.queue.splice(0)) release(waiter);
	}

	private pause(gate: ModelGate, retryAfterMs?: number): void {
		const waitMs = Math.max(this.pauseMs, Math.min(retryAfterMs ?? 0, MAX_PAUSE_MS));
		gate.phase = "paused";
		gate.resumeAt = this.d.scheduler.nowMs() + waitMs;
		gate.cancel?.();
		gate.cancel = this.d.scheduler.schedule(waitMs, () => {
			if (this.disposed || gate.book.retired) return;
			this.act(() =>
				this.shifting(gate, () => {
					gate.cancel = null;
					gate.phase = "probing";
					this.nextProbe(gate);
				}),
			);
		});
		this.notify(gate);
	}

	/** Oldest waiter probes first. */
	private nextProbe(gate: ModelGate): void {
		const waiter = gate.queue.shift();
		if (waiter !== undefined) waiter.admit(this.go(gate, true));
		this.notify(gate);
	}

	private go(gate: ModelGate, probe: boolean): Admission {
		this.serial += 1;
		const ticket: Ticket = Object.freeze({
			gate: this.id,
			model: gate.model,
			era: gate.book.era,
			generation: gate.generation,
			probe,
			serial: this.serial,
		});
		if (probe) gate.probe = ticket;
		this.d.onTicket?.(ticket);
		return { kind: "go", ticket };
	}

	/** Each queued waiter is told, once the gate's state is whole. */
	private notify(gate: ModelGate): void {
		const waiting = this.waitOf(gate);
		for (const waiter of gate.queue) {
			const { onWait } = waiter;
			if (onWait !== undefined) this.effects.push(() => onWait(waiting));
		}
	}

	private waitOf(gate: ModelGate): GateWaiting {
		return {
			model: gate.model,
			resumeAt: gate.phase === "paused" ? gate.resumeAt : null,
			pause: gate.pauses,
			of: this.maxPauses,
		};
	}

	/** Runs a gate change and tells if readiness flipped. */
	private shifting<T>(gate: ModelGate, run: () => T): T {
		const was = readyOf(gate);
		const result = run();
		if (readyOf(gate) !== was) this.tell(gate);
		return result;
	}

	private tell(gate: ModelGate): void {
		const { owner } = gate.book;
		for (const listener of [...this.listeners]) {
			this.effects.push(() => {
				try {
					listener(owner, gate.model);
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

	private gateOf(book: GateBook, model: string): ModelGate {
		let gate = book.models.get(model);
		if (gate === undefined) {
			gate = {
				book,
				model,
				phase: "open",
				generation: 0,
				pauses: 0,
				resumeAt: 0,
				probe: null,
				queue: [],
				cancel: null,
			};
			book.models.set(model, gate);
			this.books.add(book);
		}
		return gate;
	}
}
