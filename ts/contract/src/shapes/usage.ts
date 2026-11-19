// Usage record shapes.

import { shapeSet, z } from "../shape";
import { NormalizedUsage } from "./step";

////////////////////////////////
//  Shapes

/** What a request cost. */
export const Cost = z.object({
	usd: z.number(),
	/** Tier-price estimate. */
	estimated: z.boolean(),
	/** No price; shown blank. */
	unpriced: z.literal(true).exactOptional(),
});
export type Cost = z.infer<typeof Cost>;

/** Who a model request is for. */
export const CallTag = z.object({
	/** Any text; an empty one is refused at recording. */
	feature: z.string(),
	/** The Analysis purpose or the tester check. */
	lane: z.string().exactOptional(),
	/** The workspace folder's path. */
	folder: z.string().exactOptional(),
});
export type CallTag = z.infer<typeof CallTag>;

/** One metered request, including its outcome. */
export const MeteredCall = z.object({
	tag: CallTag,
	model: z.string(),
	/** The serving entry's id. */
	entry: z.string().exactOptional(),
	/** The serving entry's label. */
	label: z.string().exactOptional(),
	at: z.number(),
	ms: z.number(),
	/** First streamed chunk; a JSON request streams none. */
	firstTokenMs: z.number().exactOptional(),
	usage: NormalizedUsage.exactOptional(),
	cost: Cost,
	provider: z.string().exactOptional(),
	generationId: z.string().exactOptional(),
	finishReason: z.string().exactOptional(),
	failure: z.string().exactOptional(),
	cancelled: z.boolean().exactOptional(),
});
export type MeteredCall = z.infer<typeof MeteredCall>;

/** A call as a day file holds it. */
export const CallRecord = z.object({
	id: z.string(),
	at: z.number(),
	feature: z.string().min(1),
	lane: z.string().exactOptional(),
	model: z.string(),
	/** The serving entry's id. */
	entry: z.string().exactOptional(),
	/** The serving entry's label then. */
	label: z.string().exactOptional(),
	/** The workspace folder, home as `~`. */
	app: z.string().exactOptional(),
	ms: z.number(),
	first: z.number().exactOptional(),
	in: z.number().exactOptional(),
	out: z.number().exactOptional(),
	cached: z.number().exactOptional(),
	cacheWrite: z.number().exactOptional(),
	reasoning: z.number().exactOptional(),
	cost: z.number(),
	accountCost: z.number().exactOptional(),
	upstreamCost: z.number().exactOptional(),
	byok: z.boolean().exactOptional(),
	/** Who paid, such as `go` or `balance`. */
	billingSource: z.string().exactOptional(),
	estimated: z.boolean().exactOptional(),
	/** No price; shown blank. */
	unpriced: z.literal(true).exactOptional(),
	provider: z.string().exactOptional(),
	gen: z.string().exactOptional(),
	finish: z.string().exactOptional(),
	failure: z.string().exactOptional(),
	cancelled: z.boolean().exactOptional(),
});
export type CallRecord = z.infer<typeof CallRecord>;

/** OpenRouter's word on a generation, read after it ended. */
export const GenerationStats = z.object({
	provider: z.string().exactOptional(),
	/** Time to the first token at the provider. */
	latencyMs: z.number().exactOptional(),
	generationMs: z.number().exactOptional(),
	cost: z.number().exactOptional(),
	accountCost: z.number().exactOptional(),
	upstreamCost: z.number().exactOptional(),
	byok: z.boolean().exactOptional(),
	in: z.number().exactOptional(),
	out: z.number().exactOptional(),
	finish: z.string().exactOptional(),
});
export type GenerationStats = z.infer<typeof GenerationStats>;

/** A line holding a call. */
export const CallLine = z.object({ call: CallRecord });
export type CallLine = z.infer<typeof CallLine>;

/** Stats for the call with this id. */
export const StatsRecord = z.object({ id: z.string(), of: GenerationStats });
export type StatsRecord = z.infer<typeof StatsRecord>;

/** A line holding stats. */
export const StatsLine = z.object({ stats: StatsRecord });
export type StatsLine = z.infer<typeof StatsLine>;

/** One line of a day file; a line holding both keys reads as a call. */
export const UsageLine = z.union([CallLine, StatsLine]);
export type UsageLine = z.infer<typeof UsageLine>;

/** A record that names a generation whose stats have not joined it. */
export const UnfilledRecord = z.object({
	id: z.string(),
	generationId: z.string(),
	at: z.number(),
	entry: z.string().exactOptional(),
});
export type UnfilledRecord = z.infer<typeof UnfilledRecord>;

export const usageSet = shapeSet("usage", {
	Cost,
	CallTag,
	MeteredCall,
	CallRecord,
	GenerationStats,
	CallLine,
	StatsRecord,
	StatsLine,
	UsageLine,
	UnfilledRecord,
});
