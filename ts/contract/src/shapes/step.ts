// Model request and reply shapes.

import { shapeSet, z } from "../shape";

////////////////////////////////
//  Shapes

export const Effort = z.enum(["none", "minimal", "low", "medium", "high", "xhigh", "max"]);
export type Effort = z.infer<typeof Effort>;

export const ToolSpec = z.object({
	name: z.string(),
	description: z.string(),
	/** JSON Schema. */
	parameters: z.record(z.string(), z.unknown()),
});
export type ToolSpec = z.infer<typeof ToolSpec>;

/** Responses names a text's `phase`. */
export const TextPart = z.object({
	kind: z.literal("text"),
	text: z.string(),
	phase: z.string().exactOptional(),
});
export type TextPart = z.infer<typeof TextPart>;

export const ThoughtPart = z.object({
	kind: z.literal("thought"),
	/** Wire type verbatim. */
	type: z.string(),
	segments: z.array(z.object({ kind: z.enum(["summary", "content"]), text: z.string() })).exactOptional(),
	/** Signature or encrypted data. */
	opaque: z.string().exactOptional(),
	id: z.string().exactOptional(),
	format: z.string().exactOptional(),
	index: z.number().exactOptional(),
	anchor: z.string().exactOptional(),
});
export type ThoughtPart = z.infer<typeof ThoughtPart>;

/** Results pair by index. */
export const TurnPart = z.discriminatedUnion("kind", [
	TextPart,
	ThoughtPart,
	z.object({
		kind: z.literal("call"),
		index: z.number(),
		wireId: z.string(),
		name: z.string(),
		arguments: z.string(),
	}),
	z.object({ kind: z.literal("result"), index: z.number(), output: z.string(), isError: z.boolean() }),
]);
export type TurnPart = z.infer<typeof TurnPart>;

export const Turn = z.discriminatedUnion("role", [
	z.object({ role: z.literal("user"), text: z.string() }),
	z.object({
		role: z.literal("step"),
		/** Absent for synthetic steps. */
		producer: z.object({ model: z.string(), host: z.string().exactOptional() }).exactOptional(),
		parts: z.array(TurnPart),
	}),
]);
export type Turn = z.infer<typeof Turn>;

export const CanonicalRequest = z.object({
	/** The binding's wire id. */
	model: z.string(),
	/** Sticky blocks, byte-stable across requests. */
	system: z.array(z.string()),
	/** Folded older history, after the system blocks. */
	summary: z.string().exactOptional(),
	history: z.array(Turn),
	/** Floats last. */
	state: z.string().exactOptional(),
	tools: z.array(ToolSpec).exactOptional(),
	/** A forced tool; codecs that cannot force send the tools as offered. */
	toolChoice: z.object({ name: z.string() }).exactOptional(),
	/** The answer's JSON Schema. */
	schema: z.object({ name: z.string(), json: z.record(z.string(), z.unknown()) }).exactOptional(),
	effort: Effort.exactOptional(),
	cache: z.object({ kind: z.enum(["none", "implicit", "explicit"]), ttl: z.literal("1h").exactOptional() }),
	/** Counts reasoning tokens too. */
	maxTokens: z.int().exactOptional(),
	/** The conversation id, or a fresh id. */
	session: z.string(),
	/** Zero-retention hosts only. */
	zdrOnly: z.boolean(),
});
export type CanonicalRequest = z.infer<typeof CanonicalRequest>;

const thoughtDelta = {
	part: ThoughtPart,
	block: z.string().exactOptional(),
	/** First segment starts anew. */
	split: z.boolean().exactOptional(),
};

/** Carries thought block identity. */
export const ThoughtDelta = z.object(thoughtDelta);
export type ThoughtDelta = z.infer<typeof ThoughtDelta>;

/** The upstream host and the provider's generation id. */
export const Serving = z.object({
	host: z.string().exactOptional(),
	generationId: z.string().exactOptional(),
});
export type Serving = z.infer<typeof Serving>;

/** Chat Completions names; empty when the API gave none. */
export const FinishReason = z.enum(["stop", "tool_calls", "length", "content_filter", "error", ""]);
export type FinishReason = z.infer<typeof FinishReason>;

/** Token counts and reported charge. */
export const NormalizedUsage = z.object({
	/** Prompt tokens, cached included. */
	input: z.number(),
	/** Completion tokens, reasoning included. */
	output: z.number(),
	cached: z.number().exactOptional(),
	cacheWrite: z.number().exactOptional(),
	reasoning: z.number().exactOptional(),
	/** USD the provider reported. */
	cost: z.number().exactOptional(),
	/** BYOK: the upstream provider's USD. */
	upstreamCost: z.number().exactOptional(),
	byok: z.boolean().exactOptional(),
	/** Billing source. */
	billingSource: z.string().exactOptional(),
	/** USD at list price. */
	listCost: z.number().exactOptional(),
	/** Weighted allowance USD. */
	allowance: z.number().exactOptional(),
});
export type NormalizedUsage = z.infer<typeof NormalizedUsage>;

/** Calls arrive in pieces; `index` joins them. */
export const StepEvent = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("text"), text: z.string(), phase: z.string().exactOptional() }),
	z.object({ kind: z.literal("thought"), ...thoughtDelta }),
	z.object({
		kind: z.literal("call"),
		index: z.number(),
		wireId: z.string(),
		name: z.string(),
		argumentsText: z.string(),
	}),
	z.object({ kind: z.literal("serving"), serving: Serving }),
	z.object({ kind: z.literal("usage"), usage: NormalizedUsage }),
	z.object({ kind: z.literal("finish"), reason: FinishReason }),
]);
export type StepEvent = z.infer<typeof StepEvent>;

export const ReplyPart = z.discriminatedUnion("kind", [
	TextPart,
	ThoughtPart,
	z.object({
		kind: z.literal("call"),
		index: z.number(),
		wireId: z.string(),
		name: z.string(),
		argumentsText: z.string(),
	}),
]);
export type ReplyPart = z.infer<typeof ReplyPart>;

export const StepResult = z.object({
	/** Emission order. */
	parts: z.array(ReplyPart),
	finish: FinishReason,
	usage: NormalizedUsage.exactOptional(),
	serving: Serving,
});
export type StepResult = z.infer<typeof StepResult>;

export const FailureCategory = z.enum([
	"keyMissing",
	"keyRejected",
	"outOfCredit",
	/** Plan allowance exhausted. */
	"allowance",
	"refused",
	"notFound",
	"badRequest",
	/** A host refused thought state another host made. */
	"replayRejected",
	"rateLimited",
	"overloaded",
	"unavailable",
	"network",
	"stream",
	"empty",
	"truncated",
	"idle",
]);
export type FailureCategory = z.infer<typeof FailureCategory>;

export const Failure = z.object({
	category: FailureCategory,
	/** HTTP status, or where it failed. */
	status: z.union([z.number(), z.string()]),
	retryAfterMs: z.int().exactOptional(),
	/** Epoch ms reset time. */
	resetsAt: z.int().exactOptional(),
	message: z.string(),
	/** Why a provider refused. */
	reasons: z.array(z.string()).exactOptional(),
	/** Billed before the failure. */
	billed: z
		.object({
			usage: NormalizedUsage.exactOptional(),
			host: z.string().exactOptional(),
			generationId: z.string().exactOptional(),
		})
		.exactOptional(),
});
export type Failure = z.infer<typeof Failure>;

export const ApiFamily = z.enum(["chat-completions", "responses", "messages", "generate-content"]);
export type ApiFamily = z.infer<typeof ApiFamily>;

/** Binding API capabilities. */
export const ApiFacts = z.object({
	family: ApiFamily,
	/** Enforces JSON Schema. */
	structured: z.boolean(),
	/** `tool_choice` values. */
	choice: z.array(z.string()),
	strict: z.boolean(),
	/** The thought text field. */
	reasoning: z.string().exactOptional(),
	/** Binary thinking switch. */
	toggle: z.boolean().exactOptional(),
});
export type ApiFacts = z.infer<typeof ApiFacts>;

/** What a codec reads of a model. */
export const CodecFacts = z.object({
	api: ApiFacts,
	efforts: z.array(Effort),
	outputLimit: z.int().exactOptional(),
});
export type CodecFacts = z.infer<typeof CodecFacts>;

export const Encoded = z.object({
	path: z.string(),
	body: z.unknown(),
	headers: z.record(z.string(), z.string()).exactOptional(),
});
export type Encoded = z.infer<typeof Encoded>;

/** `event` is the `event:` line's name. */
export const SseItem = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("comment"), text: z.string() }),
	z.object({ kind: z.literal("data"), data: z.string(), event: z.string().exactOptional() }),
	z.object({ kind: z.literal("done") }),
]);
export type SseItem = z.infer<typeof SseItem>;

/** Name and value pairs, in arrival order. */
export const HeaderList = z.array(z.tuple([z.string(), z.string()]));
export type HeaderList = z.infer<typeof HeaderList>;

export const stepSet = shapeSet("step", {
	Effort,
	ToolSpec,
	TextPart,
	ThoughtPart,
	TurnPart,
	Turn,
	CanonicalRequest,
	ThoughtDelta,
	StepEvent,
	ReplyPart,
	Serving,
	FinishReason,
	NormalizedUsage,
	StepResult,
	FailureCategory,
	Failure,
	ApiFamily,
	ApiFacts,
	CodecFacts,
	Encoded,
	SseItem,
	HeaderList,
});
