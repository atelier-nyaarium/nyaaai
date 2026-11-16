// Provider entry and status shapes.

import { shapeSet, z } from "../shape";

////////////////////////////////
//  Shapes

export const ProviderKindName = z.enum(["openrouter", "opencode", "ollama", "lmstudio", "llamacpp", "compatible"]);
export type ProviderKindName = z.infer<typeof ProviderKindName>;

export const ProviderPlan = z.enum(["go", "payg"]);
export type ProviderPlan = z.infer<typeof ProviderPlan>;

/** Priority list order. */
export const ProviderEntry = z.object({
	id: z.string(),
	kind: ProviderKindName,
	label: z.string(),
	/** OpenCode only. */
	plan: ProviderPlan.exactOptional(),
	/** Local servers only. */
	url: z.string().exactOptional(),
	/** OpenCode Go only. */
	extraUsage: z.boolean().exactOptional(),
});
export type ProviderEntry = z.infer<typeof ProviderEntry>;

export const EntryFields = ProviderEntry.omit({ id: true });
export type EntryFields = z.infer<typeof EntryFields>;

/** The key stays host-side. */
export const KeyState = z.discriminatedUnion("kind", [
	z.object({ kind: z.literal("none") }),
	z.object({ kind: z.literal("checking") }),
	/** A missing limit is unlimited. */
	z.object({
		kind: z.literal("ok"),
		usage: z.number().exactOptional(),
		limit: z.number().exactOptional(),
		models: z.number().exactOptional(),
	}),
	z.object({ kind: z.literal("rejected") }),
	z.object({ kind: z.literal("unreachable") }),
	/** The field did not parse. */
	z.object({ kind: z.literal("invalid") }),
]);
export type KeyState = z.infer<typeof KeyState>;

/** A kind dropdown option. */
export const KindLine = z.object({
	kind: ProviderKindName,
	plan: ProviderPlan.exactOptional(),
	label: z.string(),
	field: z.enum(["key", "url"]),
	/** URL placeholder. */
	url: z.string().exactOptional(),
	/** URL kinds that also take a key. */
	optionalKey: z.literal(true).exactOptional(),
});
export type KindLine = z.infer<typeof KindLine>;

export const ProviderView = z.object({
	id: z.string(),
	kind: ProviderKindName,
	plan: ProviderPlan.exactOptional(),
	label: z.string(),
	url: z.string().exactOptional(),
	status: KeyState,
	/** Kind-specific facts. */
	facts: z.string().exactOptional(),
	/** The balance page has a link. */
	balance: z.literal(true).exactOptional(),
	/** Only on plans offering it. */
	extraUsage: z.boolean().exactOptional(),
});
export type ProviderView = z.infer<typeof ProviderView>;

export const KeyReach = z.enum(["ok", "keyMissing", "keyRejected", "unreachable"]);
export type KeyReach = z.infer<typeof KeyReach>;

/** A spend-limit notice. */
export const Limits = z.object({ limit: z.number().exactOptional(), reset: z.string().exactOptional() });
export type Limits = z.infer<typeof Limits>;

/** Key billing totals. */
export const Billed = z.object({
	billing: z.string(),
	allowance: z.number().exactOptional(),
	cost: z.number().exactOptional(),
});
export type Billed = z.infer<typeof Billed>;

export const SkipReason = z.enum([
	"keyMissing",
	"keyRejected",
	"credit",
	"allowance",
	"model",
	"overloaded",
	"unavailable",
	"network",
	"paused",
]);
export type SkipReason = z.infer<typeof SkipReason>;

export const Skip = z.object({ entry: z.string(), label: z.string(), reason: SkipReason });
export type Skip = z.infer<typeof Skip>;

/** A dropped route's position. */
export const Passed = z.object({ entry: z.string(), label: z.string(), reason: SkipReason, before: z.number() });
export type Passed = z.infer<typeof Passed>;

export const NoticeKind = z.enum([
	"keyMissing",
	"keyRejected",
	"spendLimit",
	"refused",
	"noZdrHost",
	"modelRetired",
	"modelUnavailable",
	"unreachable",
	"badRequest",
	"retriesExhausted",
	"providerOverloaded",
	"streamFailed",
	"outputCut",
	"noModel",
	"historyDropped",
]);
export type NoticeKind = z.infer<typeof NoticeKind>;

/** Retry, fall through, or stop. */
export const Disposition = z.discriminatedUnion("act", [
	z.object({ act: z.literal("retry") }),
	z.object({ act: z.literal("fallThrough"), notice: NoticeKind }),
	z.object({ act: z.literal("stop"), notice: NoticeKind }),
]);
export type Disposition = z.infer<typeof Disposition>;

export const providerSet = shapeSet("provider", {
	ProviderKindName,
	ProviderPlan,
	ProviderEntry,
	EntryFields,
	KeyState,
	KindLine,
	ProviderView,
	KeyReach,
	Limits,
	Billed,
	SkipReason,
	Skip,
	Passed,
	NoticeKind,
	Disposition,
});
