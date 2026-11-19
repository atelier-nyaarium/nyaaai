// Entry health: key status, bills, pool cooldowns and gates, read through ports.

import type { EntryKeys } from "@nyaaai/contract/entryKeys";
import type { Ids } from "@nyaaai/contract/id";
import { canonicalJson } from "@nyaaai/contract/json";
import type { ProviderList } from "@nyaaai/contract/providerList";
import type {
	Admission,
	Blocked,
	EntryHealth,
	GateOutcome,
	GateSettled,
	GateWaiting,
	HealthChange,
	PoolEvent,
	PoolState,
	PoolVerdict,
	Ticket,
} from "@nyaaai/contract/shapes/health";
import type { Billed, KeyState, ProviderEntry, ProviderKindName, SkipReason } from "@nyaaai/contract/shapes/provider";
import type { Failure, NormalizedUsage } from "@nyaaai/contract/shapes/step";
import type { Scheduler } from "@nyaaai/contract/time";
import type { KindDeps, ProviderKind } from "../provider/kind";
import type { Resolved } from "../provider/resolver";
import { type GateState, ProviderGate } from "./gate";
import { KEY_CHECK_MS, KeyReader } from "./keyReader";
import { CLEAR, step, verdict } from "./pool";

////////////////////////////////
//  Interfaces & Types

/** What the lifecycle calls: per-route admission, cooldowns and skips. */
export interface Gate {
	/** Waits through route pauses. */
	admit(
		route: Resolved,
		signal: AbortSignal,
		onWait?: (waiting: GateWaiting) => void,
		again?: boolean,
	): Promise<Admission>;
	/** Never waits. */
	tryAdmit(route: Resolved): Ticket | null;
	settle(ticket: Ticket, outcome: GateOutcome, retryAfterMs?: number): GateSettled;
	/** Balance-cooldown exemption. */
	deferred(route: Resolved, keep?: ReadonlySet<string>): SkipReason | undefined;
	/** Last-route refusal. */
	blocked(route: Resolved, keep?: ReadonlySet<string>): Blocked | undefined;
	failed(route: Resolved, failure: Failure, onward: boolean): void;
	/** Balance replies join `run`. */
	served(route: Resolved, usage: NormalizedUsage | undefined, run?: Set<string>): void;
}

/** One reply's billing. */
export interface Billing {
	source: string;
	/** Weighted USD. */
	allowance?: number;
	cost?: number;
}

export interface HealthDeps {
	/** Gate pauses, rechecks and the time of every pool event. */
	scheduler: Scheduler;
	/** Asked once, for the gate's name. */
	ids: Ids;
	/** The entries in priority order. */
	list: ProviderList;
	/** The store's keys; health offers `entryKeys` over them. */
	keys: EntryKeys;
	kinds: Partial<Record<ProviderKindName, ProviderKind>>;
	/** What a kind's check stands on. */
	kindDeps(kind: ProviderKindName): KindDeps;
	pauseMs?: number;
	maxPauses?: number;
}

type Part = HealthChange["changed"][number];

interface Status {
	state: KeyState;
	reader: KeyReader;
}

/** Tracked provider fields. */
interface Identity {
	plan?: string;
	url?: string;
	extraUsage: boolean;
}

////////////////////////////////
//  Constants

const PAUSE_MS = 60_000;

const RECHECK_MS = 60_000;

/** Declaration order of the shape's parts. */
const PARTS: readonly Part[] = ["key", "billed", "pool", "waiting"];

////////////////////////////////
//  Functions & Helpers

const none = (): KeyState => ({ kind: "none" });

function keyOf(route: Pick<Resolved, "entry" | "key">): string {
	return `${route.entry.id}\n${route.key}`;
}

/** The entry a gate's model key belongs to. */
function entryOfModel(model: string): string {
	const end = model.indexOf("\n");
	return end < 0 ? model : model.slice(0, end);
}

const ticketKey = (ticket: Ticket): string => canonicalJson(ticket);

function total(sum: number | undefined, more: number | undefined): number | undefined {
	return sum === undefined && more === undefined ? undefined : (sum ?? 0) + (more ?? 0);
}

function billedWith(was: Billed | undefined, reply: Billing): Billed {
	const allowance = total(was?.allowance, reply.allowance);
	const cost = total(was?.cost, reply.cost);
	return {
		billing: reply.source,
		...(allowance === undefined ? {} : { allowance }),
		...(cost === undefined ? {} : { cost }),
	};
}

function identityOf(entry: ProviderEntry): Identity {
	return {
		...(entry.plan === undefined ? {} : { plan: entry.plan }),
		...(entry.url === undefined ? {} : { url: entry.url }),
		extraUsage: entry.extraUsage === true,
	};
}

////////////////////////////////
//  Classes

/** Entry health and routing. */
export class ProviderHealth implements Gate {
	/** The keys a resolver reads: the store's, unless health holds the entry out. */
	readonly entryKeys: EntryKeys;
	private readonly gate: ProviderGate;
	/** Absent pool means clear. */
	private readonly pools = new Map<string, PoolState>();
	/** Admitted tickets' entries, by the ticket's data. */
	private readonly probes = new Map<string, string>();
	private readonly statuses = new Map<string, Status>();
	/** Readers dropped while a check ran. */
	private retired: KeyReader[] = [];
	private readonly rechecked = new Map<string, number>();
	private readonly bills = new Map<string, Billed>();
	private readonly listeners = new Set<(change: HealthChange) => void>();
	private readonly unsubscribe: (() => void)[] = [];
	private identities: Map<string, Identity>;
	/** Depth of the running changes whose telling waits for their end. */
	private batching = 0;
	private readonly pending = new Map<string, Set<Part>>();

	/** Defers initial key checks. */
	constructor(private readonly d: HealthDeps) {
		this.gate = new ProviderGate({
			scheduler: d.scheduler,
			ids: d.ids,
			...(d.pauseMs === undefined ? {} : { pauseMs: d.pauseMs }),
			...(d.maxPauses === undefined ? {} : { maxPauses: d.maxPauses }),
		});
		this.entryKeys = {
			get: async (id) => {
				try {
					const entry = this.entryOf(id);
					if (entry === undefined) return { kind: "none" };
					const standing = this.standing(entry);
					if (standing !== undefined) return { kind: standing };
					return await d.keys.get(id);
				} catch {
					return { kind: "unready" };
				}
			},
			onChange: (listener) => d.keys.onChange(listener),
		};
		this.identities = this.snapshot();
		this.unsubscribe.push(
			d.keys.onChange((id) =>
				this.batch(() => {
					this.invalidate(id, true);
					this.check(id);
				}),
			),
			d.list.onChange(() => this.diff()),
			this.gate.onChange((model) => this.touch(entryOfModel(model), "waiting")),
		);
	}

	/** Cached key state. */
	status(id: string): KeyState {
		return { ...(this.statuses.get(id)?.state ?? none()) };
	}

	/** Listed replies create bills. */
	bill(id: string): Billed | undefined {
		const held = this.bills.get(id);
		return held === undefined ? undefined : { ...held };
	}

	/** Everything health tracks of one entry. */
	state(id: string): EntryHealth {
		const prefix = `${id}\n`;
		const billed = this.bill(id);
		return {
			entry: id,
			key: this.status(id),
			...(billed === undefined ? {} : { billed }),
			pool: structuredClone(this.pools.get(id) ?? CLEAR),
			waiting: this.gate
				.holding()
				.filter((model) => model.startsWith(prefix))
				.map((model) => model.slice(prefix.length)),
		};
	}

	/** One model's gate as data; none until it is first used. */
	gateState(route: Pick<Resolved, "entry" | "key">): GateState | undefined {
		return this.gate.stateOf(keyOf(route));
	}

	/** Accumulates reply billing. */
	billed(id: string, reply: Billing): void {
		if (!this.listed(id)) return;
		const was = this.bills.get(id);
		const next = billedWith(was, reply);
		if (was?.billing === next.billing && was.allowance === next.allowance && was.cost === next.cost) return;
		this.bills.set(id, next);
		this.touch(id, "billed");
	}

	/** Replaces pending checks. */
	check(id: string): void {
		if (!this.listed(id)) return;
		this.batch(() => {
			void this.statusOf(id).reader.read();
		});
	}

	checkUnread(): void {
		for (const { id } of this.d.list.entries()) if (!this.statuses.has(id)) this.check(id);
	}

	/** Answers when no key check is running, having waited for those that were. */
	async idle(): Promise<void> {
		for (;;) {
			const busy = [...[...this.statuses.values()].map((status) => status.reader), ...this.retired].filter(
				(reader) => reader.busy(),
			);
			if (busy.length === 0) {
				this.retired = [];
				return;
			}
			await Promise.all(busy.map((reader) => reader.idle()));
		}
	}

	/** Told each change once, with its parts in the shape's order; the result unsubscribes. */
	onChange(listener: (change: HealthChange) => void): () => void {
		this.listeners.add(listener);
		return () => {
			this.listeners.delete(listener);
		};
	}

	/** Probes require admitted tickets. */
	async admit(
		route: Resolved,
		signal: AbortSignal,
		onWait?: (waiting: GateWaiting) => void,
		again = false,
	): Promise<Admission> {
		const admission = await this.batch(() => this.gate.admit(keyOf(route), signal, onWait, again));
		if (admission.kind === "go") this.admitted(route.entry.id, admission.ticket);
		return admission;
	}

	tryAdmit(route: Resolved): Ticket | null {
		return this.batch(() => {
			const ticket = this.gate.tryAdmit(keyOf(route));
			if (ticket !== null) this.admitted(route.entry.id, ticket);
			return ticket;
		});
	}

	settle(ticket: Ticket, outcome: GateOutcome, retryAfterMs?: number): GateSettled {
		return this.batch(() => {
			const key = ticketKey(ticket);
			const id = this.probes.get(key);
			if (id !== undefined) {
				this.probes.delete(key);
				this.apply(id, { kind: "settled", ticket });
			}
			return this.gate.settle(ticket, outcome, retryAfterMs);
		});
	}

	deferred(route: Resolved, keep?: ReadonlySet<string>): SkipReason | undefined {
		if (this.rejected(route.entry)) return "keyRejected";
		const pool = this.verdict(route.entry, keep);
		if (pool !== undefined) return pool.skip;
		return this.gate.ready(keyOf(route)) ? undefined : "paused";
	}

	blocked(route: Resolved, keep?: ReadonlySet<string>): Blocked | undefined {
		if (this.rejected(route.entry)) return { kind: "keyRejected" };
		return this.verdict(route.entry, keep)?.blocked;
	}

	/** Balance replies extend `run`. */
	served(route: Resolved, usage: NormalizedUsage | undefined, run?: Set<string>): void {
		const source = usage?.billingSource;
		if (source === undefined) return;
		// Settings may change mid-request.
		const entry = this.entryOf(route.entry.id) ?? route.entry;
		this.apply(entry.id, {
			kind: "billed",
			source,
			...(entry.plan === undefined ? {} : { plan: entry.plan }),
			extraUsage: entry.extraUsage === true,
		});
		if (source === "balance") run?.add(entry.id);
	}

	failed(route: Resolved, failure: Failure, onward: boolean): void {
		const { entry } = route;
		this.batch(() => {
			switch (failure.category) {
				case "keyRejected":
					this.mark(entry.id, { kind: "rejected" });
					return;
				// One pool per entry.
				case "allowance":
				case "outOfCredit":
					this.apply(entry.id, {
						kind: "refused",
						reason: failure.category === "allowance" ? "allowance" : "credit",
						...(failure.resetsAt === undefined ? {} : { resetsAt: failure.resetsAt }),
						...(failure.retryAfterMs === undefined ? {} : { retryAfterMs: failure.retryAfterMs }),
					});
					return;
				case "network":
					if (this.byUrl(entry)) this.mark(entry.id, { kind: "unreachable" });
					else if (onward) this.gate.cool(keyOf(route), failure.retryAfterMs);
					return;
				// Settlement owns overload pauses.
				case "overloaded":
				case "keyMissing":
				case "refused":
				case "badRequest":
				case "replayRejected":
					return;
				default:
					if (onward) this.gate.cool(keyOf(route), failure.retryAfterMs);
			}
		});
	}

	dispose(): void {
		for (const off of this.unsubscribe) off();
		for (const status of this.statuses.values()) status.reader.dispose();
		this.statuses.clear();
		this.bills.clear();
		this.pools.clear();
		this.probes.clear();
		this.listeners.clear();
		this.gate.dispose();
	}

	/** Unhealthy entries are held out; asking rechecks an unreachable server. */
	private standing(entry: ProviderEntry): "refused" | "unready" | undefined {
		const status = this.status(entry.id).kind;
		if (status === "rejected") return "refused";
		if (!this.byUrl(entry)) return undefined;
		if (status === "unreachable") this.recheck(entry.id);
		// Out until the recheck passes.
		return status === "unreachable" || (status === "checking" && this.rechecking(entry.id)) ? "unready" : undefined;
	}

	/** A later turn may find it back. */
	private recheck(id: string): void {
		const now = this.d.scheduler.nowMs();
		const last = this.rechecked.get(id);
		if (last !== undefined && now - last < RECHECK_MS) return;
		this.rechecked.set(id, now);
		this.check(id);
	}

	private rechecking(id: string): boolean {
		const last = this.rechecked.get(id);
		return last !== undefined && this.d.scheduler.nowMs() - last < KEY_CHECK_MS;
	}

	/** The only pool writer. */
	private apply(id: string, event: PoolEvent): void {
		const was = this.pools.get(id) ?? CLEAR;
		const next = step(was, event, this.d.scheduler.nowMs(), this.d.pauseMs ?? PAUSE_MS);
		if (next.kind === "clear") this.pools.delete(id);
		else this.pools.set(id, next);
		if (canonicalJson(next) !== canonicalJson(was)) this.touch(id, "pool");
	}

	private admitted(id: string, ticket: Ticket): void {
		this.batch(() => {
			this.probes.set(ticketKey(ticket), id);
			this.apply(id, { kind: "admitted", ticket });
		});
	}

	/** Extra usage or `keep` exempts balance. */
	private verdict(entry: ProviderEntry, keep: ReadonlySet<string> | undefined): PoolVerdict | undefined {
		const exempt = entry.extraUsage === true || keep?.has(entry.id) === true;
		return verdict(this.pools.get(entry.id) ?? CLEAR, exempt, this.d.scheduler.nowMs());
	}

	/** Inference stands until recheck. */
	private mark(id: string, state: KeyState): void {
		if (!this.listed(id)) return;
		const status = this.statusOf(id);
		if (status.state.kind === state.kind) return;
		status.state = state;
		this.touch(id, "key");
	}

	private statusOf(id: string): Status {
		const known = this.statuses.get(id);
		if (known !== undefined) return known;
		const fresh: Status = {
			state: none(),
			reader: new KeyReader(
				this.d.scheduler,
				(signal) => this.keyCheck(id, signal),
				(state) => {
					fresh.state = state;
					this.touch(id, "key");
				},
			),
		};
		this.statuses.set(id, fresh);
		return fresh;
	}

	private async keyCheck(id: string, signal: AbortSignal): Promise<KeyState> {
		const entry = this.entryOf(id);
		const kind = entry === undefined ? undefined : this.d.kinds[entry.kind];
		if (entry === undefined || kind === undefined) return none();
		const answer = await this.d.keys.get(id);
		if (answer.kind === "refused") return { kind: "rejected" };
		if (answer.kind === "unready") return { kind: "unreachable" };
		const secret = answer.kind === "key" ? answer.key : undefined;
		if (kind.field === "key" && secret === undefined) return none();
		return kind.check(entry, secret, this.d.kindDeps(entry.kind), signal);
	}

	private invalidate(id: string, unbill: boolean): void {
		this.batch(() => {
			const status = this.statuses.get(id);
			if (status !== undefined) {
				status.reader.dispose();
				if (status.reader.busy()) this.retired.push(status.reader);
				this.statuses.delete(id);
				if (status.state.kind !== "none") this.touch(id, "key");
			}
			if (unbill && this.bills.delete(id)) this.touch(id, "billed");
			if (this.pools.delete(id)) this.touch(id, "pool");
			this.gate.clear((key) => key.startsWith(`${id}\n`));
		});
	}

	private diff(): void {
		this.batch(() => {
			const next = this.snapshot();
			for (const [id, was] of this.identities) {
				const now = next.get(id);
				if (now === undefined) {
					this.invalidate(id, true);
				} else if (now.plan !== was.plan || now.url !== was.url) {
					this.invalidate(id, false);
					this.check(id);
				} else if (now.extraUsage !== was.extraUsage) {
					const lastBill = this.bills.get(id)?.billing;
					this.apply(id, {
						kind: "extraUsage",
						on: now.extraUsage,
						...(lastBill === undefined ? {} : { lastBill }),
					});
				}
			}
			const added = [...next.keys()].filter((id) => !this.identities.has(id));
			this.identities = next;
			for (const id of added) this.check(id);
		});
	}

	private snapshot(): Map<string, Identity> {
		return new Map(this.d.list.entries().map((entry) => [entry.id, identityOf(entry)]));
	}

	/** Runs a change whose parts are told together, once it ends. */
	private batch<T>(run: () => T): T {
		this.batching += 1;
		try {
			return run();
		} finally {
			this.batching -= 1;
			if (this.batching === 0) this.flush();
		}
	}

	private touch(id: string, part: Part): void {
		let parts = this.pending.get(id);
		if (parts === undefined) {
			parts = new Set();
			this.pending.set(id, parts);
		}
		parts.add(part);
		if (this.batching === 0) this.flush();
	}

	private flush(): void {
		const held = [...this.pending];
		this.pending.clear();
		for (const [entry, parts] of held) {
			const changed = PARTS.filter((part) => parts.has(part));
			for (const listener of [...this.listeners]) {
				try {
					listener({ entry, changed });
				} catch {
					// A listener cannot change health.
				}
			}
		}
	}

	private entryOf(id: string): ProviderEntry | undefined {
		return this.d.list.entries().find((entry) => entry.id === id);
	}

	private listed(id: string): boolean {
		return this.entryOf(id) !== undefined;
	}

	private rejected(entry: ProviderEntry): boolean {
		return this.status(entry.id).kind === "rejected";
	}

	private byUrl(entry: ProviderEntry): boolean {
		return this.d.kinds[entry.kind]?.field === "url";
	}
}
