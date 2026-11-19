// Gate, pool and health state shapes.

import { shapeSet, z } from "../shape";
import { Billed, KeyState, NoticeKind, SkipReason } from "./provider";

////////////////////////////////
//  Shapes

/** Identifies a gate pass and probe; tickets with equal fields are the same ticket. */
export const Ticket = z.object({
	/** The issuing gate's name. */
	gate: z.string(),
	model: z.string(),
	/** The key era of its gate rows. */
	era: z.int().min(0),
	/** Bumped by every reopening of the gate. */
	generation: z.int().min(0),
	probe: z.boolean(),
	/** Counted by the gate that issued it. */
	serial: z.int().min(0),
});
export type Ticket = z.infer<typeof Ticket>;

export const Admission = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("go"), ticket: Ticket }),
	z.object({ kind: z.literal("gaveUp") }),
	z.object({ kind: z.literal("stopped") }),
	/** Its incarnation ended, or its era retired. */
	z.object({ kind: z.literal("retired") }),
]);
export type Admission = z.infer<typeof Admission>;

/** A wait and its next probe time. */
export const GateWaiting = z.object({
	model: z.string(),
	/** Null unless paused. */
	resumeAt: z.number().nullable(),
	pause: z.int().min(0),
	of: z.int().min(0),
});
export type GateWaiting = z.infer<typeof GateWaiting>;

/** `again`: admit once more. */
export const GateSettled = z.enum(["done", "again", "gaveUp"]);
export type GateSettled = z.infer<typeof GateSettled>;

export const GateOutcome = z.enum(["ok", "overloaded", "failed", "cancelled"]);
export type GateOutcome = z.infer<typeof GateOutcome>;

/** One entry's pool cooldown. */
export const PoolState = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("clear") }),
	z.object({ kind: z.literal("paused"), reason: z.enum(["allowance", "credit"]), until: z.number() }),
	/** A known allowance reset. */
	z.object({ kind: z.literal("reset"), at: z.number() }),
	/** Billed past the allowance. */
	z.object({
		kind: z.literal("balance"),
		eta: z.number(),
		/** Next probe window. */
		next: z.number(),
		/** The first ticket admitted in the window, whatever its `probe` mark; it holds the entry until `eta`. */
		probe: z.object({ ticket: Ticket, at: z.number() }).nullable(),
	}),
]);
export type PoolState = z.infer<typeof PoolState>;

export const PoolEvent = z.discriminatedUnion("kind", [
	z.object({
		kind: z.literal("billed"),
		source: z.string(),
		plan: z.string().exactOptional(),
		extraUsage: z.boolean(),
	}),
	z.object({
		kind: z.literal("refused"),
		reason: z.enum(["allowance", "credit"]),
		resetsAt: z.number().exactOptional(),
		retryAfterMs: z.number().exactOptional(),
	}),
	z.object({ kind: z.literal("extraUsage"), on: z.boolean(), lastBill: z.string().exactOptional() }),
	/** The pool takes the first ticket admitted in its window as its probe; `Ticket.probe` is the gate's own mark. */
	z.object({ kind: z.literal("admitted"), ticket: Ticket }),
	z.object({ kind: z.literal("settled"), ticket: Ticket }),
]);
export type PoolEvent = z.infer<typeof PoolEvent>;

/** A last-route refusal. */
export const Blocked = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("keyRejected") }),
	z.object({
		kind: z.literal("spendLimit"),
		resetsAt: z.number().exactOptional(),
		nextTryAt: z.number().exactOptional(),
	}),
]);
export type Blocked = z.infer<typeof Blocked>;

export const PoolVerdict = z.object({
	skip: SkipReason,
	/** Set when only a notice can end the last route. */
	blocked: Blocked.exactOptional(),
});
export type PoolVerdict = z.infer<typeof PoolVerdict>;

/** Everything health tracks of one entry. */
export const EntryHealth = z.object({
	entry: z.string(),
	key: KeyState,
	billed: Billed.exactOptional(),
	pool: PoolState,
	/** Models of its gates that are waiting; read as a set, each written once. */
	waiting: z.array(z.string()),
});
export type EntryHealth = z.infer<typeof EntryHealth>;

/** Which parts of an entry's health changed; `changed` is read as a set, written once each in shape order. */
export const HealthChange = z.object({
	entry: z.string(),
	changed: z.array(z.enum(["key", "billed", "pool", "waiting"])).min(1),
});
export type HealthChange = z.infer<typeof HealthChange>;

/** A failure's status: HTTP, or where it failed. */
export const ErrorStatus = z.union([z.number(), z.string()]);
export type ErrorStatus = z.infer<typeof ErrorStatus>;

/** What the user is told when a request ends or a route is passed. */
export const Notice = z.object({
	kind: NoticeKind,
	status: ErrorStatus.exactOptional(),
	message: z.string().exactOptional(),
	reasons: z.array(z.string()).exactOptional(),
	limit: z.number().exactOptional(),
	reset: z.string().exactOptional(),
	/** Epoch ms the allowance resets. */
	resetsAt: z.number().exactOptional(),
	/** Next balance probe time. */
	nextTryAt: z.number().exactOptional(),
	/** Display label of the overloaded or unavailable model. */
	model: z.string().exactOptional(),
	/** The serving entry's label. */
	entry: z.string().exactOptional(),
});
export type Notice = z.infer<typeof Notice>;

/** The wait before a retry is drawn from these. */
export const RetryOptions = z.object({
	tries: z.int().min(0).exactOptional(),
	budgetMs: z.number().min(0).exactOptional(),
	capMs: z.number().min(0).exactOptional(),
	baseMs: z.number().min(0).exactOptional(),
});
export type RetryOptions = z.infer<typeof RetryOptions>;

export const healthSet = shapeSet("health", {
	Ticket,
	Admission,
	GateWaiting,
	GateSettled,
	GateOutcome,
	PoolState,
	PoolEvent,
	PoolVerdict,
	Blocked,
	HealthChange,
	EntryHealth,
	Notice,
	ErrorStatus,
	RetryOptions,
});
