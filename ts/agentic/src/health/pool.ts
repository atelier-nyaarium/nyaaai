// One entry's pool cooldown: the transition and the skip it asks for.

import type { PoolEvent, PoolState, PoolVerdict, Ticket } from "@nyaaai/contract/shapes/health";

////////////////////////////////
//  Constants

export const CLEAR: PoolState = { kind: "clear" };

/** Go balance reset estimate. */
export const BALANCE_COOL_MS = 5 * 60 * 60_000;

/** ETA floor before halving. */
export const PROBE_FLOOR_MS = 10 * 60_000;

/** A window ends within a month of the reply. */
export const MAX_WINDOW_MS = 31 * 24 * 60 * 60_000;

/** Pool billing sources. */
export const ALLOWANCE_SOURCES: ReadonlySet<string> = new Set(["go", "go-plus"]);

////////////////////////////////
//  Functions & Helpers

/** Two tickets are the same when every field is equal. */
const sameTicket = (left: Ticket, right: Ticket): boolean =>
	left.gate === right.gate &&
	left.model === right.model &&
	left.generation === right.generation &&
	left.probe === right.probe &&
	left.serial === right.serial;

/** Halve remaining ETA. */
export function probeAt(nowMs: number, eta: number): number {
	return nowMs + Math.max(eta - nowMs, PROBE_FLOOR_MS) / 2;
}

/** A known reset wins. */
function billed(state: PoolState, source: string, extraUsage: boolean, nowMs: number): PoolState {
	if (source !== "balance") return ALLOWANCE_SOURCES.has(source) && state.kind === "balance" ? CLEAR : state;
	if (extraUsage || (state.kind === "reset" && nowMs < state.at) || (state.kind === "paused" && nowMs < state.until))
		return state;
	const open = state.kind === "balance" && nowMs < state.eta;
	const eta = open ? state.eta : nowMs + BALANCE_COOL_MS;
	// A renewed window starts without a probe.
	return { kind: "balance", eta, next: probeAt(nowMs, eta), probe: open ? state.probe : null };
}

/** The only transition. */
export function step(state: PoolState, event: PoolEvent, nowMs: number, pauseMs: number): PoolState {
	switch (event.kind) {
		case "billed":
			return event.plan === "go" ? billed(state, event.source, event.extraUsage, nowMs) : state;
		case "refused": {
			const farthest = nowMs + MAX_WINDOW_MS;
			const waited =
				event.retryAfterMs === undefined ? undefined : Math.min(nowMs + event.retryAfterMs, farthest);
			const known = event.resetsAt === undefined ? undefined : Math.min(event.resetsAt, farthest);
			const reset = event.reason === "allowance" ? (known ?? waited) : undefined;
			const until = reset ?? waited ?? nowMs + pauseMs;
			// Pauses never shorten windows.
			if (reset === undefined && state.kind === "balance" && nowMs < Math.max(state.eta, state.next))
				return { ...state, next: Math.max(state.next, until) };
			const refused: PoolState =
				reset === undefined ? { kind: "paused", reason: event.reason, until } : { kind: "reset", at: reset };
			// A refusal never ends a running pause or reset earlier.
			const ends = state.kind === "paused" ? state.until : state.kind === "reset" ? state.at : undefined;
			return ends !== undefined && nowMs < ends && until < ends ? state : refused;
		}
		case "extraUsage":
			if (event.on) return state.kind === "balance" ? CLEAR : state;
			return event.lastBill === "balance" ? billed(state, "balance", false, nowMs) : state;
		case "admitted":
			return state.kind === "balance" && state.probe === null && nowMs >= state.next && nowMs < state.eta
				? { ...state, probe: { ticket: event.ticket, at: nowMs } }
				: state;
		case "settled":
			// Settlement preserves `next`.
			return state.kind === "balance" && state.probe !== null && sameTicket(state.probe.ticket, event.ticket)
				? { ...state, probe: null }
				: state;
	}
}

/** Skip, and the last route's refusal. */
export function verdict(state: PoolState, exempt: boolean, nowMs: number): PoolVerdict | undefined {
	switch (state.kind) {
		case "clear":
			return undefined;
		case "paused":
			return nowMs < state.until ? { skip: state.reason } : undefined;
		case "reset":
			return nowMs < state.at
				? { skip: "allowance", blocked: { kind: "spendLimit", resetsAt: state.at } }
				: undefined;
		case "balance": {
			if (exempt) return undefined;
			const nextTryAt =
				state.probe !== null && nowMs < state.eta
					? Math.max(probeAt(state.probe.at, state.eta), state.next)
					: nowMs < state.next
						? state.next
						: undefined;
			return nextTryAt === undefined
				? undefined
				: { skip: "allowance", blocked: { kind: "spendLimit", nextTryAt } };
		}
	}
}
