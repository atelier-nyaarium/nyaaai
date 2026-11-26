// The HTTP port, and the budgets every exchange runs on.

import { type Budget, type HeaderList, MAX_BYTES, MAX_IDLE_MS, MAX_TOTAL_MS } from "./shapes/step";
import { type Deadline, deadline, type Scheduler } from "./time";

////////////////////////////////
//  Interfaces & Types

/** A `GET` carries no body. */
export interface FetchRequest {
	method: "GET" | "POST";
	url: string;
	headers: HeaderList;
	body?: string;
	budget: Budget;
}

export interface FetchResponse {
	status: number;
	/** In the `responseHeaders` form. */
	headers: HeaderList;
	/** A read may reject; the connection frees at end, `return` or abort. */
	chunks: AsyncIterable<Uint8Array>;
}

/** Redirects are not followed; the signal covers chunk reads, which reject with its reason. */
export type Fetch = (request: FetchRequest, signal: AbortSignal) => Promise<FetchResponse>;

/** Enforces no budget; only `budgeted` takes one. */
export type RawFetch = (request: FetchRequest, signal: AbortSignal) => Promise<FetchResponse>;

export type BudgetBound = "total" | "idle" | "bytes";

export type BudgetName = (typeof BUDGET_NAMES)[number];

export type Budgets = Readonly<Record<BudgetName, Budget>>;

/** A host's changes to the presets. */
export type BudgetChanges = Partial<Record<BudgetName, Partial<Budget>>>;

/** An operation's clock. */
export interface Span extends Deadline {
	/** The caller's signal alone. */
	caller: AbortSignal;
	/** What is left; refuses once ended. */
	budget(): Budget;
}

////////////////////////////////
//  Constants

export const BUDGET_NAMES = [
	"keyCheck",
	"limits",
	"stats",
	"catalog",
	"detail",
	"discover",
	"warm",
	"reply",
	"stream",
] as const;

const MIB = 1024 * 1024;

/** Even total and idle, 16 MiB. */
const plain = (ms: number): Budget => ({ totalMs: ms, idleMs: ms, maxBytes: 16 * MIB });

/** Every preset the engine uses. */
export const BUDGETS: Budgets = {
	keyCheck: plain(30_000),
	limits: plain(5000),
	stats: plain(10_000),
	catalog: plain(30_000),
	detail: plain(30_000),
	discover: plain(10_000),
	warm: plain(10 * 60_000),
	reply: { totalMs: 240_000, idleMs: 120_000, maxBytes: 16 * MIB },
	stream: { totalMs: 2 * 60 * 60_000, idleMs: 120_000, maxBytes: 256 * MIB },
};

export const BUDGET_CEILINGS: Budget = { totalMs: MAX_TOTAL_MS, idleMs: MAX_IDLE_MS, maxBytes: MAX_BYTES };

const FIELDS = ["totalMs", "idleMs", "maxBytes"] as const;

/** Never ends. */
const NEVER = new AbortController().signal;

////////////////////////////////
//  Classes

/** A bound of an exchange ran out. */
export class BudgetExceeded extends Error {
	constructor(
		readonly bound: BudgetBound,
		readonly limit: number,
	) {
		super(`The ${bound} budget of ${limit} ran out.`);
		this.name = "BudgetExceeded";
	}
}

/** One exchange's clocks and count. */
class Exchange {
	private readonly raw = new AbortController();
	/** Marked at a caller's abort. */
	readonly signal: AbortSignal;
	/** The first end; it stands. */
	private end: { error: unknown } | undefined;
	/** Rejects the network wait in progress. */
	private refuse: ((error: unknown) => void) | undefined;
	/** The total passed while no wait ran. */
	private overdue = false;
	/** The clocks have stopped. */
	private over = false;
	private finished = false;
	private bytes = 0;
	private stopTotal = () => {};
	private stopIdle = () => {};

	constructor(
		private readonly scheduler: Scheduler,
		private readonly budget: Budget,
		private readonly caller: AbortSignal,
	) {
		this.signal = AbortSignal.any([caller, this.raw.signal]);
	}

	/** Arms the total clock. */
	start(): void {
		this.caller.addEventListener("abort", this.onCaller, { once: true });
		this.stopTotal = this.scheduler.schedule(this.budget.totalMs, () => {
			if (this.refuse === undefined) this.overdue = true;
			else this.breach("total");
		});
	}

	/** A wait on the network, under the idle clock. */
	wait<T>(read: () => Promise<T>): Promise<T> {
		if (this.end !== undefined) return Promise.reject(this.end.error);
		if (this.over) return read();
		if (this.overdue) return Promise.reject(this.breach("total"));
		return new Promise<T>((resolve, reject) => {
			this.refuse = reject;
			// A chunk at the bound counts.
			this.stopIdle = this.scheduler.schedule(this.budget.idleMs + 1, () => this.breach("idle"));
			let pending: Promise<T>;
			try {
				pending = read();
			} catch (error) {
				pending = Promise.reject(error);
			}
			pending.then(
				(value) => {
					this.waited(reject);
					resolve(value);
				},
				(error: unknown) => {
					this.waited(reject);
					if (this.end === undefined) this.settle();
					reject(this.end === undefined ? error : this.end.error);
				},
			);
		});
	}

	/** The body, counted and clocked. */
	body(chunks: AsyncIterable<Uint8Array>): AsyncIterable<Uint8Array> {
		let iterator: AsyncIterator<Uint8Array> | undefined;
		return {
			[Symbol.asyncIterator]: () => ({
				next: async () => {
					const step = await this.wait(() => {
						iterator ??= chunks[Symbol.asyncIterator]();
						return iterator.next();
					});
					if (step.done === true) {
						this.finished = true;
						this.settle();
						return step;
					}
					this.bytes += step.value.byteLength;
					if (this.bytes > this.budget.maxBytes) throw this.breach("bytes");
					return step;
				},
				return: async () => {
					this.settle();
					if (!this.finished && !this.signal.aborted) this.raw.abort();
					await iterator?.return?.();
					return { done: true, value: undefined };
				},
			}),
		};
	}

	private readonly onCaller = () => {
		this.finish(this.caller.reason);
	};

	/** The end that stands. */
	private breach(bound: BudgetBound): unknown {
		const limit =
			bound === "total" ? this.budget.totalMs : bound === "idle" ? this.budget.idleMs : this.budget.maxBytes;
		return this.finish(new BudgetExceeded(bound, limit));
	}

	/** The first end stands. */
	private finish(error: unknown): unknown {
		if (this.end === undefined) {
			this.end = { error };
			this.settle();
			this.raw.abort(error);
			const refuse = this.refuse;
			this.refuse = undefined;
			refuse?.(error);
		}
		return this.end.error;
	}

	private waited(reject: (error: unknown) => void): void {
		if (this.refuse !== reject) return;
		this.refuse = undefined;
		this.stopIdle();
	}

	/** Stops the clocks. */
	private settle(): void {
		this.over = true;
		this.stopTotal();
		this.stopIdle();
		this.caller.removeEventListener("abort", this.onCaller);
	}
}

////////////////////////////////
//  Functions & Helpers

/** The presets with a host's changes. */
export function budgetTable(changes: BudgetChanges = {}): Budgets {
	for (const name of Object.keys(changes))
		if (!(BUDGET_NAMES as readonly string[]).includes(name)) throw new Error(`There is no ${name} budget.`);
	const checked = (name: BudgetName): Budget => {
		const change: Record<string, unknown> = { ...changes[name] };
		for (const field of Object.keys(change))
			if (!(FIELDS as readonly string[]).includes(field)) throw new Error(`A budget has no ${field}.`);
		const value = (field: (typeof FIELDS)[number]): number => {
			const held = field in change ? change[field] : BUDGETS[name][field];
			const ceiling = BUDGET_CEILINGS[field];
			if (typeof held !== "number" || !Number.isSafeInteger(held) || held < 1 || held > ceiling)
				throw new Error(`The ${name} budget's ${field} is not a whole number from 1 to ${ceiling}.`);
			return held;
		};
		return { totalMs: value("totalMs"), idleMs: value("idleMs"), maxBytes: value("maxBytes") };
	};
	return Object.fromEntries(BUDGET_NAMES.map((name) => [name, checked(name)])) as Record<BudgetName, Budget>;
}

/** The one enforcer of a budget. */
export function budgeted(raw: RawFetch, scheduler: Scheduler): Fetch {
	return async (request, signal) => {
		if (signal.aborted) {
			// The raw fetch refuses it.
			const answer = await raw(request, signal).catch(() => undefined);
			await answer?.chunks[Symbol.asyncIterator]().return?.();
			throw signal.reason;
		}
		const exchange = new Exchange(scheduler, request.budget, signal);
		exchange.start();
		const response = await exchange.wait(() => raw(request, exchange.signal));
		return { ...response, chunks: exchange.body(response.chunks) };
	};
}

/** An operation's span; clear it when done. */
export function span(scheduler: Scheduler, budget: Budget, caller?: AbortSignal): Span {
	const start = scheduler.nowMs();
	const limit = deadline(scheduler, budget.totalMs, caller);
	return {
		...limit,
		caller: caller ?? NEVER,
		budget: () => {
			const totalMs = Math.min(budget.totalMs, budget.totalMs - (scheduler.nowMs() - start));
			if (limit.ended() === "deadline" || totalMs < 1) throw new BudgetExceeded("total", budget.totalMs);
			return { totalMs, idleMs: Math.min(budget.idleMs, totalMs), maxBytes: budget.maxBytes };
		},
	};
}
