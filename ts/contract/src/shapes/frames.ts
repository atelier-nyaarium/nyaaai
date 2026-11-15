// Stdio adapter frames.

import { z } from "zod";
import { shapeSet } from "../shape";

////////////////////////////////
//  Shapes

const run = z.string().min(1);
const id = z.string().min(1);

/** Refusal path component. */
export const PathPart = z.union([z.string(), z.int().min(0)]);
export type PathPart = z.infer<typeof PathPart>;

/** Scripted clock and ids. */
export const Fixed = z.object({
	nowMs: z.int(),
	/** Added per clock read. */
	stepMs: z.int().min(0).default(0),
	/** 1-based ID suffix. */
	idPrefix: z.string(),
});
export type Fixed = z.infer<typeof Fixed>;

export const Hello = z.object({
	type: z.literal("hello"),
	/** Host contract version. */
	version: z.string(),
});
export type Hello = z.infer<typeof Hello>;

export const Start = z.object({
	type: z.literal("start"),
	run,
	command: z.string().min(1),
	input: z.unknown(),
	fixed: Fixed.exactOptional(),
});
export type Start = z.infer<typeof Start>;

export const Steer = z.object({
	type: z.literal("steer"),
	run,
	text: z.string(),
});
export type Steer = z.infer<typeof Steer>;

export const Interrupt = z.object({
	type: z.literal("interrupt"),
	run,
});
export type Interrupt = z.infer<typeof Interrupt>;

export const Reply = z.object({
	type: z.literal("reply"),
	run,
	id,
	value: z.unknown(),
});
export type Reply = z.infer<typeof Reply>;

export const ReplyFailed = z.object({
	type: z.literal("reply-failed"),
	run,
	id,
	message: z.string(),
});
export type ReplyFailed = z.infer<typeof ReplyFailed>;

export const HostFrame = z.discriminatedUnion("type", [Hello, Start, Steer, Interrupt, Reply, ReplyFailed]);
export type HostFrame = z.infer<typeof HostFrame>;

export const Welcome = z.object({
	type: z.literal("welcome"),
	version: z.string(),
});
export type Welcome = z.infer<typeof Welcome>;

export const Reject = z.object({
	type: z.literal("reject"),
	version: z.string(),
	reason: z.string(),
});
export type Reject = z.infer<typeof Reject>;

export const RunEvent = z.object({
	type: z.literal("event"),
	run,
	/** 1-based frame count. */
	seq: z.int().min(1),
	event: z.unknown(),
});
export type RunEvent = z.infer<typeof RunEvent>;

/** Tool, key, or storage request. */
export const Request = z.object({
	type: z.literal("request"),
	run,
	id,
	kind: z.string().min(1),
	input: z.unknown(),
});
export type Request = z.infer<typeof Request>;

/** Run terminal. */
export const Done = z.object({
	type: z.literal("done"),
	run,
	seq: z.int().min(1),
	result: z.unknown(),
});
export type Done = z.infer<typeof Done>;

export const RunFailure = z.enum(["interrupted", "unknown_command", "bad_input", "host_request", "error"]);
export type RunFailure = z.infer<typeof RunFailure>;

/** Run terminal. */
export const Failed = z.object({
	type: z.literal("failed"),
	run,
	seq: z.int().min(1),
	code: RunFailure,
	path: z.array(PathPart).exactOptional(),
	message: z.string(),
});
export type Failed = z.infer<typeof Failed>;

export const FaultCode = z.enum([
	"bad_frame",
	"oversized_frame",
	"not_greeted",
	"unknown_run",
	"duplicate_run",
	"unknown_reply",
]);
export type FaultCode = z.infer<typeof FaultCode>;

/** Refused frame; run continues. */
export const Fault = z.object({
	type: z.literal("fault"),
	code: FaultCode,
	run: run.exactOptional(),
	path: z.array(PathPart).exactOptional(),
	message: z.string(),
});
export type Fault = z.infer<typeof Fault>;

export const AdapterFrame = z.discriminatedUnion("type", [Welcome, Reject, RunEvent, Request, Done, Failed, Fault]);
export type AdapterFrame = z.infer<typeof AdapterFrame>;

export const framesSet = shapeSet("frames", {
	PathPart,
	Fixed,
	Hello,
	Start,
	Steer,
	Interrupt,
	Reply,
	ReplyFailed,
	HostFrame,
	Welcome,
	Reject,
	RunEvent,
	Request,
	Done,
	Failed,
	FaultCode,
	RunFailure,
	Fault,
	AdapterFrame,
});
