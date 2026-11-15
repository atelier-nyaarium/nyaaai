// Session cut shapes.

import { z } from "zod";
import { shapeSet } from "../shape";

////////////////////////////////
//  Shapes

/** 1-based line or count. */
const count = z.int().min(0);

/** Trim keeps live; clear and compact cut above a turn. */
export const CutMode = z.enum(["trim", "clear", "compact"]);
export type CutMode = z.infer<typeof CutMode>;

export const CutRefusal = z.enum([
	"no_rows",
	"cli_version",
	"boundary_shape",
	"unreadable_rows",
	"nothing_to_trim",
	"preserved_unresolved",
	"target_missing",
	"target_compacted",
	"target_not_turn_start",
	"target_mid_tool",
	"tail_mid_tool",
	"end_missing",
	"late_message",
	"nothing_above",
	"summary_needed",
]);
export type CutRefusal = z.infer<typeof CutRefusal>;

export const CutRefused = z.object({
	ok: z.literal(false),
	reason: CutRefusal,
	detail: z.string(),
});
export type CutRefused = z.infer<typeof CutRefused>;

/** Post-boundary row metadata. */
export const RowMeta = z.object({
	line: count,
	bytes: count,
	type: z.string(),
	uuid: z.string().exactOptional(),
	at: z.string().exactOptional(),
	/** Unanswered tools before row. */
	openTools: count,
	/** Delivery marks cut points. */
	delivered: z.boolean(),
	/** Delivered message text. */
	text: z.string().exactOptional(),
	/** Referenced rows. */
	points: z.array(z.string()),
});
export type RowMeta = z.infer<typeof RowMeta>;

export const BoundaryMeta = z.object({
	line: count,
	uuid: z.string(),
	/** Preserved rows precede it. */
	preservedLines: z.array(count),
	/** Preserved IDs chain rows tail to head. */
	preservedSound: z.boolean(),
	preserved: z.array(z.string()),
	summaryLine: count.nullable(),
	cumulativeDroppedTokens: z.number(),
	postTokens: z.number(),
	/** Fields match the known cut schema. */
	known: z.boolean(),
});
export type BoundaryMeta = z.infer<typeof BoundaryMeta>;

export const SessionIndex = z.object({
	lines: count,
	unparsed: count,
	/** Unparsed post-boundary lines. */
	unreadable: z.array(count),
	version: z.string().nullable(),
	/** Newest line per type. */
	header: z.record(z.string(), count),
	boundary: BoundaryMeta.nullable(),
	/** Compaction rows in file. */
	boundaries: count,
	/** Live boundary summary. */
	standing: z.string().nullable(),
	/** Tail includes file end. */
	tail: z.array(RowMeta),
});
export type SessionIndex = z.infer<typeof SessionIndex>;

export const CutRequest = z.object({
	mode: CutMode,
	/** Turn start to keep; unused by trim. */
	targetUuid: z.string().exactOptional(),
	/** Last kept row; defaults to file end. */
	throughUuid: z.string().exactOptional(),
	/** Ends kept rows before a later turn holding this. */
	endBefore: z.string().exactOptional(),
	/** Caller-held compact summary. */
	summaryText: z.string().exactOptional(),
	/** Live session; check end at build. */
	liveTail: z.boolean().exactOptional(),
});
export type CutRequest = z.infer<typeof CutRequest>;

/** Turn lookup chat row. */
export const TurnPick = z.object({
	at: z.int(),
	/** Present for sent messages. */
	text: z.string().exactOptional(),
});
export type TurnPick = z.infer<typeof TurnPick>;

export const PickedTurn = z.object({
	uuid: z.string(),
	/** Exact text match; else turn at or before time. */
	exact: z.boolean(),
});
export type PickedTurn = z.infer<typeof PickedTurn>;

/** Boundary-preserved segment. */
export const Preserved = z.object({
	head: z.string(),
	tail: z.string(),
	uuids: z.array(z.string()),
});
export type Preserved = z.infer<typeof Preserved>;

/** Kept turn for confirmation. */
export const KeptFrom = z.object({
	at: z.string().nullable(),
	text: z.string(),
});
export type KeptFrom = z.infer<typeof KeptFrom>;

export const CutPlan = z.object({
	ok: z.literal(true),
	mode: CutMode,
	/** Newest header rows first. */
	headerLines: z.array(count),
	/** Verbatim lines, ascending. */
	keptLines: z.array(count),
	/** Trim preserves none. */
	preserved: Preserved.nullable(),
	/** Rows folded by summarizer. */
	foldLines: z.array(count),
	keptBytes: count,
	droppedBytes: count,
	droppedRows: count,
	from: KeptFrom.nullable(),
});
export type CutPlan = z.infer<typeof CutPlan>;

export const CutDone = z.object({
	ok: z.literal(true),
	sessionId: z.string(),
	plan: CutPlan,
});
export type CutDone = z.infer<typeof CutDone>;

/** Command `sessionCut.plan`. */
export const PlanCommand = z.object({
	source: z.string(),
	request: CutRequest,
	cliVersion: z.string(),
});
export type PlanCommand = z.infer<typeof PlanCommand>;

export const PlanOutcome = z.union([CutPlan, CutRefused]);
export type PlanOutcome = z.infer<typeof PlanOutcome>;

/** Builds beside source, never over it. */
export const ApplyCommand = z.object({
	source: z.string(),
	out: z.string(),
	/** Excluded output folder. */
	keepOutOf: z.string().exactOptional(),
	request: CutRequest,
	cliVersion: z.string(),
});
export type ApplyCommand = z.infer<typeof ApplyCommand>;

export const CutOutcome = z.union([CutDone, CutRefused]);
export type CutOutcome = z.infer<typeof CutOutcome>;

/** Summary request; text reply. */
export const SummarizeRequest = z.object({
	standing: z.string().nullable(),
	rows: z.array(z.string()),
});
export type SummarizeRequest = z.infer<typeof SummarizeRequest>;

export const sessionCutSet = shapeSet("sessionCut", {
	CutMode,
	CutRefusal,
	CutRefused,
	RowMeta,
	BoundaryMeta,
	SessionIndex,
	CutRequest,
	TurnPick,
	PickedTurn,
	Preserved,
	KeptFrom,
	CutPlan,
	CutDone,
	PlanCommand,
	PlanOutcome,
	ApplyCommand,
	CutOutcome,
	SummarizeRequest,
});
