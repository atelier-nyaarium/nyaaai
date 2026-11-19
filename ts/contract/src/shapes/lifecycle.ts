// What a request's lifecycle tells while it runs: one entry per effect, in the order it happened.

import { shapeSet, z } from "../shape";
import { Admission, ErrorStatus, GateOutcome, GateSettled, GateWaiting, Notice } from "./health";
import { Disposition, SkipReason } from "./provider";
import { Failure, NormalizedUsage } from "./step";
import { MeteredCall } from "./usage";

////////////////////////////////
//  Shapes

/**
 * One effect of a run. A run tells its entries to the sink it was given, one at a time, in the
 * order the effects happened; with no sink none is built. An entry holds data only. `entry` is
 * an entry id and `model` a model key.
 */
export const TraceEntry = z.discriminatedUnion("kind", [
	/** A route passed over before it was tried: dropped by the resolver, deferred by the gate, or failed onward. */
	z.object({
		kind: z.literal("skip"),
		cause: z.enum(["passed", "deferred", "fellThrough"]),
		entry: z.string(),
		label: z.string(),
		reason: SkipReason,
	}),
	/** The gate holds the call; told at every pause and probe turn. */
	z.object({ kind: z.literal("held"), entry: z.string(), waiting: GateWaiting }),
	/** The hold ended. */
	z.object({ kind: z.literal("released"), entry: z.string() }),
	/** What the gate answered an admission. */
	z.object({ kind: z.literal("admitted"), entry: z.string(), admission: Admission }),
	/** Zero retention came on after admission and the route keeps data: the route is not sent to. */
	z.object({ kind: z.literal("kept"), entry: z.string(), model: z.string() }),
	/** A send begins. `attempt` counts the route's tries; `again` is an admission after an overload. */
	z.object({
		kind: z.literal("attempt"),
		entry: z.string(),
		model: z.string(),
		attempt: z.int().min(1),
		live: z.boolean(),
		again: z.boolean(),
	}),
	/** The call as the meter was given it. */
	z.object({ kind: z.literal("metered"), call: MeteredCall }),
	/** A reply's billing, told to the gate; also for a failed reply that billed. */
	z.object({
		kind: z.literal("billing"),
		entry: z.string(),
		reply: z.enum(["ok", "failed"]),
		usage: NormalizedUsage.exactOptional(),
	}),
	/**
	 * A send failed. `disposition` is absent for an overload, which the lifecycle handles before it
	 * asks the kind; `fresh` is false after streamed output; `onward` says a route follows.
	 */
	z.object({
		kind: z.literal("failure"),
		entry: z.string(),
		model: z.string(),
		failure: Failure,
		disposition: Disposition.exactOptional(),
		fresh: z.boolean(),
		onward: z.boolean(),
	}),
	/** The gate settled a ticket, and what it answered. */
	z.object({
		kind: z.literal("settled"),
		entry: z.string(),
		outcome: GateOutcome,
		retryAfterMs: z.int().exactOptional(),
		answer: GateSettled,
	}),
	/** The gate was told a route failed. */
	z.object({ kind: z.literal("gateFailed"), entry: z.string(), failure: Failure, onward: z.boolean() }),
	/** A wait before the next try, and why it is that long. */
	z.object({
		kind: z.literal("retry"),
		entry: z.string(),
		attempt: z.int().min(1),
		waitMs: z.int().min(0),
		why: z.enum(["backoff", "retryAfter"]),
		status: ErrorStatus,
	}),
	/** The kind's limits were read for a notice; `read` is false when none came back. */
	z.object({
		kind: z.literal("limits"),
		entry: z.string(),
		read: z.boolean(),
		limit: z.number().exactOptional(),
		reset: z.string().exactOptional(),
	}),
	/** A send threw what is no provider failure; the run throws it on. */
	z.object({ kind: z.literal("threw"), entry: z.string(), message: z.string() }),
	/** The run ended: served by an entry, stopped by the caller, or with a notice. */
	z.object({ kind: z.literal("served"), entry: z.string(), label: z.string(), model: z.string() }),
	z.object({ kind: z.literal("stopped") }),
	z.object({ kind: z.literal("notice"), notice: Notice }),
]);
export type TraceEntry = z.infer<typeof TraceEntry>;

export const lifecycleSet = shapeSet("lifecycle", { TraceEntry });
