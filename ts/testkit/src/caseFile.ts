// Tagged case-file shapes.

import { shapeSet } from "@nyaaai/contract/shape";
import { z } from "zod";

////////////////////////////////
//  Shapes

/** Bump on layout changes. */
export const CASE_FORMAT = 2;

const Frame = z.record(z.string(), z.unknown());

export const FunctionCase = z.object({
	format: z.literal(CASE_FORMAT),
	type: z.literal("function"),
	/** A `CASE_KINDS` key. */
	kind: z.string(),
	name: z.string(),
	input: z.unknown(),
	expected: z.unknown(),
});
export type FunctionCase = z.infer<typeof FunctionCase>;

/** `$TMP` means run folder. */
export const RunStep = z.union([
	z.object({ send: z.unknown() }),
	z.object({ sendText: z.string() }),
	z.object({ sendBase64: z.string() }),
	/** No trailing line ending. */
	z.object({ sendTail: z.string() }),
	/** Bytes beyond frame cap. */
	z.object({ sendOverCap: z.int().min(0) }),
	/** Partial frame match. */
	z.object({ await: z.union([Frame, z.array(Frame)]) }),
	z.object({ close: z.literal(true) }),
]);
export type RunStep = z.infer<typeof RunStep>;

export const Played = z.object({
	exit: z.int(),
	/** Handshake and fault order. */
	connection: z.array(Frame),
	/** Ordered per-run frames. */
	runs: z.record(z.string(), z.array(Frame)),
	/** LF-separated file rows. */
	files: z.record(z.string(), z.array(z.string())),
});
export type Played = z.infer<typeof Played>;

export const RunCase = z.object({
	format: z.literal(CASE_FORMAT),
	type: z.literal("run"),
	/** The `<group>.run` kind. */
	kind: z.string(),
	name: z.string(),
	/** LF rows under `$TMP`. */
	files: z.record(z.string(), z.array(z.string())).exactOptional(),
	script: z.array(RunStep),
	read: z.array(z.string()).exactOptional(),
	expected: Played,
});
export type RunCase = z.infer<typeof RunCase>;

export const CaseFile = z.discriminatedUnion("type", [FunctionCase, RunCase]);
export type CaseFile = z.infer<typeof CaseFile>;

/** Union exhaustiveness guard. */
export function unhandled(value: never): never {
	throw new Error(`unhandled ${JSON.stringify(value)}`);
}

export const caseFileSet = shapeSet("caseFile", { FunctionCase, RunStep, Played, RunCase, CaseFile });
