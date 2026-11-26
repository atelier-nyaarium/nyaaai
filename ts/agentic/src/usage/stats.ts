// Delayed generation stats: reading them back, with retries and spacing.

import { BUDGETS, type Budgets, type Span, span } from "@nyaaai/contract/fetch";
import type { GenerationStats } from "@nyaaai/contract/shapes/usage";
import { type Deadline, deadline, type Scheduler } from "@nyaaai/contract/time";
import type { UsageSink } from "@nyaaai/contract/usageSink";
import { booleanField, type Fields, fieldOf, fieldsOf, numberField, objectField, textField } from "../kinds/readers";
import { reportedCost } from "./cost";

////////////////////////////////
//  Interfaces & Types

export interface StatsQueueDeps {
	scheduler: Scheduler;
	/** Where stats are written, and where unfilled records are listed. */
	usage: Pick<UsageSink, "fill" | "filled" | "unfilled">;
	/** Reads through the serving entry's transport only, never another's key; should end on its span. */
	read: (entry: string | undefined, path: `/generation?id=${string}`, span: Span) => Promise<unknown>;
	/** `stats` bounds reads and asks. */
	budgets?: Budgets;
}

/** A record whose stats are still to be read. */
export interface WaitingRead {
	id: string;
	generationId: string;
	/** The serving entry. */
	entry?: string;
	/** Request timestamp. */
	at: number;
	/** Reads that missed. */
	tries: number;
	/** When the next read may start. */
	due: number;
}

type FieldKind = "text" | "number" | "flag";

////////////////////////////////
//  Constants

/** Request-relative retry delays. */
const DELAYS_MS = [7, 15, 30, 60].map((minutes) => minutes * 60_000);

/** Spaces endpoint reads. */
const SPACING_MS = 250;

/** Queue limit. */
const MAX_WAITING = 500;

/** How far back `restore` looks for records without stats. */
export const UNFILLED_MS = 2 * 60 * 60_000;

const GENERATION_PREFIX = "gen-";

/** The part after the prefix is 1 to 124 characters. */
const GENERATION_REST_MAX = 124;

/** Body fields; missing or null is fine, a wrong type means no stats. */
const FIELDS: ReadonlyArray<readonly [string, FieldKind]> = [
	["provider_name", "text"],
	["latency", "number"],
	["generation_time", "number"],
	["total_cost", "number"],
	["is_byok", "flag"],
	["upstream_inference_cost", "number"],
	["tokens_prompt", "number"],
	["tokens_completion", "number"],
	["native_tokens_prompt", "number"],
	["native_tokens_completion", "number"],
	["finish_reason", "text"],
];

////////////////////////////////
//  Functions & Helpers

function fits(fields: Fields, key: string, kind: FieldKind): boolean {
	const value = fieldOf(fields, key);
	if (value === undefined || value === null) return true;
	if (kind === "text") return typeof value === "string";
	if (kind === "number") return typeof value === "number" && Number.isFinite(value);
	return typeof value === "boolean";
}

function isGenerationId(text: string): boolean {
	if (!text.startsWith(GENERATION_PREFIX)) return false;
	const rest = text.length - GENERATION_PREFIX.length;
	if (rest < 1 || rest > GENERATION_REST_MAX) return false;
	for (let at = GENERATION_PREFIX.length; at < text.length; at++) {
		const code = text.charCodeAt(at);
		const digit = code >= 48 && code <= 57;
		const upper = code >= 65 && code <= 90;
		const lower = code >= 97 && code <= 122;
		if (!digit && !upper && !lower && code !== 45) return false;
	}
	return true;
}

/** None for a body that is not a generation's stats. */
export function statsOf(body: unknown): GenerationStats | undefined {
	const top = fieldsOf(body);
	const data = top === undefined ? null : objectField(top, "data");
	if (data === null || !FIELDS.every(([key, kind]) => fits(data, key, kind))) return undefined;
	const provider = textField(data, "provider_name");
	const latency = numberField(data, "latency");
	const generation = numberField(data, "generation_time");
	const account = numberField(data, "total_cost");
	const byok = booleanField(data, "is_byok");
	const upstream = numberField(data, "upstream_inference_cost");
	const finish = textField(data, "finish_reason");
	const stats: GenerationStats = {};
	if (provider !== null) stats.provider = provider;
	if (latency !== null) stats.latencyMs = latency;
	if (generation !== null) stats.generationMs = generation;
	if (account !== null) stats.accountCost = account;
	if (byok !== null) stats.byok = byok;
	if (upstream !== null) stats.upstreamCost = upstream;
	const cost = reportedCost({
		...(account === null ? {} : { cost: account }),
		...(byok === null ? {} : { byok }),
		...(upstream === null ? {} : { upstreamCost: upstream }),
	});
	if (cost !== undefined) stats.cost = cost;
	const inTokens = numberField(data, "native_tokens_prompt") ?? numberField(data, "tokens_prompt");
	const outTokens = numberField(data, "native_tokens_completion") ?? numberField(data, "tokens_completion");
	if (inTokens !== null) stats.in = inTokens;
	if (outTokens !== null) stats.out = outTokens;
	if (finish !== null) stats.finish = finish;
	return stats;
}

////////////////////////////////
//  Classes

/** One read at a time, at 7, 15, 30 and 60 minutes, 250 ms apart at least. */
export class StatsQueue {
	private readonly waiting: WaitingRead[] = [];
	private timer: { cancel: () => void; at: number } | undefined;
	private reading = false;
	/** The id of the record being read. */
	private current: string | undefined;
	private running: Promise<void> | undefined;
	private disposed = false;
	private readonly stop = new AbortController();

	constructor(private readonly d: StatsQueueDeps) {}

	/** Schedules stats reads; delays already past start at once; a record waits once. */
	add(id: string, generationId: string, at = this.d.scheduler.nowMs(), entry?: string): void {
		if (this.disposed || this.current === id || !isGenerationId(generationId)) return;
		const now = this.d.scheduler.nowMs();
		const passed = DELAYS_MS.filter((delay) => at + delay <= now).length;
		const each = { id, generationId, ...(entry === undefined ? {} : { entry }), at };
		const fresh: WaitingRead =
			passed === 0
				? { ...each, tries: 0, due: at + (DELAYS_MS[0] ?? 0) }
				: { ...each, tries: passed - 1, due: now };
		const held = this.waiting.findIndex((other) => other.id === id);
		if (held !== -1) {
			if (fresh.due < (this.waiting[held]?.due ?? Number.POSITIVE_INFINITY)) this.waiting[held] = fresh;
		} else {
			if (this.waiting.length >= MAX_WAITING) this.waiting.shift();
			this.waiting.push(fresh);
		}
		this.arm();
	}

	/** Adds recent records lacking stats; `keep` leaves out gone entries. */
	async restore(keep: (entry: string | undefined) => boolean = () => true): Promise<void> {
		let records: Awaited<ReturnType<UsageSink["unfilled"]>>;
		try {
			records = await this.d.usage.unfilled(this.d.scheduler.nowMs() - UNFILLED_MS);
		} catch {
			return;
		}
		for (const record of records)
			if (keep(record.entry)) this.add(record.id, record.generationId, record.at, record.entry);
	}

	/** Drops waiting reads and ends running ones; results are unused. */
	dispose(): void {
		this.disposed = true;
		this.stop.abort();
		this.timer?.cancel();
		this.timer = undefined;
		this.waiting.length = 0;
	}

	/** Settles when no read is running. */
	async idle(): Promise<void> {
		while (this.running !== undefined) await this.running;
	}

	/** Records that wait, in queue order. */
	queued(): WaitingRead[] {
		return this.waiting.map((each) => ({ ...each }));
	}

	/** The instant the next read is set for. */
	armedAt(): number | undefined {
		return this.timer?.at;
	}

	/** Enforces read spacing. */
	private arm(spacing = 0): void {
		if (this.disposed || this.reading || this.waiting.length === 0) return;
		const due = Math.min(...this.waiting.map((each) => each.due));
		const now = this.d.scheduler.nowMs();
		const at = Math.max(due, now + spacing);
		if (this.timer !== undefined && this.timer.at <= at) return;
		this.timer?.cancel();
		const mine = { cancel: () => {}, at };
		this.timer = mine;
		mine.cancel = this.d.scheduler.schedule(Math.max(0, at - now), () => {
			if (this.timer !== mine) return;
			this.timer = undefined;
			const run: Promise<void> = this.next().finally(() => {
				if (this.running === run) this.running = undefined;
			});
			this.running = run;
		});
	}

	/** A port's answer; none at the window's end, dispose or failure. */
	private async within<T>(limit: Deadline, call: () => Promise<T>): Promise<T | undefined> {
		try {
			const work = new Promise<T>((resolve) => resolve(call()));
			const ended = new Promise<undefined>((resolve) => {
				if (limit.signal.aborted) resolve(undefined);
				else limit.signal.addEventListener("abort", () => resolve(undefined), { once: true });
			});
			return await Promise.race([work, ended]);
		} catch {
			return undefined;
		} finally {
			limit.clear();
		}
	}

	/** Reads or reschedules. */
	private async next(): Promise<void> {
		const now = this.d.scheduler.nowMs();
		const index = this.waiting.findIndex((each) => each.due <= now);
		const each = index === -1 ? undefined : this.waiting.splice(index, 1)[0];
		if (each === undefined || this.disposed) return this.arm();
		this.reading = true;
		this.current = each.id;
		let spacing = SPACING_MS;
		try {
			const budget = (this.d.budgets ?? BUDGETS).stats;
			const asked = deadline(this.d.scheduler, budget.totalMs, this.stop.signal);
			const filled = (await this.within(asked, () => this.d.usage.filled(each.id, each.at))) === true;
			const reading = filled || this.disposed ? undefined : span(this.d.scheduler, budget, this.stop.signal);
			const body =
				reading === undefined
					? undefined
					: await this.within(reading, () =>
							this.d.read(each.entry, `/generation?id=${each.generationId}`, reading),
						);
			if (this.disposed) return;
			if (filled) spacing = 0;
			else {
				const stats = statsOf(body);
				if (stats !== undefined) {
					try {
						this.d.usage.fill(each.id, stats);
					} catch {
						// The record stays without stats.
					}
				} else {
					each.tries += 1;
					const delay = DELAYS_MS[each.tries];
					if (delay !== undefined) {
						if (this.waiting.length >= MAX_WAITING) this.waiting.shift();
						this.waiting.push({ ...each, due: each.at + delay });
					}
				}
			}
		} finally {
			this.reading = false;
			this.current = undefined;
		}
		this.arm(spacing);
	}
}
