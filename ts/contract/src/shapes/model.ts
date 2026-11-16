// Model row, view, registry and browse shapes.

import { matching, preprocessed, shapeSet, z } from "../shape";
import { trimText } from "../text";
import { ProviderKindName } from "./provider";
import { ApiFacts, ApiFamily, Effort } from "./step";

////////////////////////////////
//  Functions & Helpers

const trimmed = (raw: unknown): unknown => (typeof raw === "string" ? trimText(raw) : raw);

/** USD per million tokens, low and high. */
const range = () => z.tuple([z.number(), z.number()]);

////////////////////////////////
//  Shapes

/** The host capability a model must have. */
export const ModelUse = z.enum(["chat", "compaction", "analysis"]);
export type ModelUse = z.infer<typeof ModelUse>;

/** The kind's own model id. */
export const Binding = z.object({ id: z.string() });
export type Binding = z.infer<typeof Binding>;

/** Which row a key names. */
export const RowIdentity = z.discriminatedUnion("kind", [
	/** A curated key, bound per kind. */
	z.object({ kind: z.literal("curated"), key: z.string() }),
	/** A test-added row; every entry of `provider` serves it. */
	z.object({ kind: z.literal("hosted"), provider: ProviderKindName, id: z.string() }),
	/** A discovered tag; only `entry` serves it. */
	z.object({ kind: z.literal("local"), entry: z.string(), tag: z.string() }),
]);
export type RowIdentity = z.infer<typeof RowIdentity>;

export const ModelRow = z.object({
	/** The stable row key. */
	key: z.string(),
	/** By kind name. */
	bindings: z.record(z.string(), Binding),
	/** Catalog id glob. */
	family: z.string(),
	label: z.string(),
	/** The reasoning effort applies. */
	reasoning: z.boolean(),
	schema: z.boolean(),
	cache: z.enum(["implicit", "explicit"]),
	verified: z.string(),
	/** USD per million tokens. */
	verifiedPrice: z.object({ prompt: z.number(), completion: z.number() }),
	/** The key encodes it. */
	identity: RowIdentity,
});
export type ModelRow = z.infer<typeof ModelRow>;

/** USD per million tokens above `minPromptTokens`. */
export const PriceTier = z.object({
	minPromptTokens: z.number(),
	prompt: z.number(),
	completion: z.number(),
	cacheRead: z.number().exactOptional(),
	cacheWrite: z.number().exactOptional(),
	cacheWrite1h: z.number().exactOptional(),
});
export type PriceTier = z.infer<typeof PriceTier>;

/** USD per million tokens, low and high. */
export const Price = z.object({ prompt: range(), completion: range(), tiers: z.array(PriceTier) });
export type Price = z.infer<typeof Price>;

export const ModelView = z.object({
	row: ModelRow,
	status: z.enum(["ok", "retired", "noZdrHost", "noTools", "unknown"]),
	retiredOn: z.string().exactOptional(),
	/** Lowest first. */
	efforts: z.array(Effort),
	promptLimit: z.number().exactOptional(),
	/** Output tokens, reasoning included. */
	outputLimit: z.number().exactOptional(),
	/** `free` bills nothing; `unknown` bills at an unreported price; absent: not listed. */
	price: z.union([Price, z.literal("free"), z.literal("unknown")]).exactOptional(),
	/** The listing prices one-hour cache writes. */
	cacheWrite1h: z.literal(true).exactOptional(),
	/** Tags that honor ZDR-only mode. */
	hosts: z.array(z.string()),
	/** Only hosts that keep data serve it. */
	dataKept: z.literal(true).exactOptional(),
	/** The serving binding's API; absent on OpenRouter. */
	api: ApiFacts.exactOptional(),
	/** The binding's caching, over the row's. */
	cache: z.enum(["implicit", "explicit"]).exactOptional(),
});
export type ModelView = z.infer<typeof ModelView>;

export const TestRecord = z.object({
	label: preprocessed("trimmed", trimmed, z.string().min(1).max(200)),
	date: matching(z.string(), "^\\d{4}-\\d{2}-\\d{2}$"),
	/** USD. */
	cost: z.number().min(0),
	/** Estimated from tokens. */
	estimated: z.boolean().exactOptional(),
	checks: z.object({
		toolPlus: z.enum(["pass", "fail", "n/a"]),
		recovery: z.enum(["pass", "fail", "n/a"]),
		reasoning: z.enum(["pass", "fail", "n/a"]),
		schema: z.enum(["pass", "fail", "n/a"]),
	}),
	/** The effort parameter is supported. */
	reasoning: z.boolean(),
	cache: z.enum(["implicit", "explicit", "none"]),
	/** USD per million tokens. */
	verifiedPrice: z.object({ prompt: z.number().min(0), completion: z.number().min(0) }),
});
export type TestRecord = z.infer<typeof TestRecord>;

/** Records by entry then row key, added keys per entry in order, shared hidden families and keys. */
export const StoredRegistry = z.object({
	tested: z.record(z.string(), z.record(z.string(), TestRecord)),
	added: z.record(z.string(), z.array(z.string())),
	hidden: z.array(z.string()),
});
export type StoredRegistry = z.infer<typeof StoredRegistry>;

/** One upstream host of a browsed model. */
export const ProviderRow = z.object({
	name: z.string(),
	tag: z.string(),
	context: z.number().exactOptional(),
	maxOutput: z.number().exactOptional(),
	quantization: z.string().exactOptional(),
	/** USD per million tokens. */
	prompt: z.number().exactOptional(),
	completion: z.number().exactOptional(),
	cacheRead: z.number().exactOptional(),
	/** Percent. */
	uptime: z.number().exactOptional(),
	zdr: z.boolean(),
});
export type ProviderRow = z.infer<typeof ProviderRow>;

/** A listed model. */
export const BrowseModel = z.object({
	/** The row key. */
	id: z.string(),
	display: z.string(),
	label: z.string(),
	/** A curated row binds it. */
	curated: z.boolean(),
	description: z.string(),
	context: z.number().exactOptional(),
	inputs: z.array(z.string()),
	outputs: z.array(z.string()),
	/** Kind-specific capabilities. */
	parameters: z.array(z.string()),
	/** ZDR price ranges, USD per million tokens. */
	price: z.union([z.object({ prompt: range(), completion: range() }), z.literal("free"), z.literal("unknown")]),
	/** Testable for chat, or why not. */
	tools: z.enum(["ok", "none", "noZdrHost"]),
	/** A failed test can run again. */
	retestable: z.literal(true).exactOptional(),
	family: ApiFamily.exactOptional(),
});
export type BrowseModel = z.infer<typeof BrowseModel>;

/** Kind-specific model detail. */
export const BrowseDetail = z.object({
	hosts: z.array(ProviderRow).exactOptional(),
	/** Output tokens, reasoning included. */
	maxOutput: z.number().exactOptional(),
	efforts: z.array(z.string()).exactOptional(),
	/** Tool choice modes. */
	choice: z.array(z.string()).exactOptional(),
	cache: z.enum(["implicit", "explicit"]).exactOptional(),
	/** The ZDR end date. */
	retention: z.object({ zdr: z.boolean(), until: z.string().exactOptional() }).exactOptional(),
});
export type BrowseDetail = z.infer<typeof BrowseDetail>;

/** The entry serving a key. */
export const Via = z.object({ entry: z.string(), label: z.string() });
export type Via = z.infer<typeof Via>;

/** The notice a request on an unresolved key ends on. */
export const Refusal = z.enum(["keyMissing", "keyRejected", "unreachable", "noZdrHost", "modelRetired", "noModel"]);
export type Refusal = z.infer<typeof Refusal>;

/** A key no entry serves. */
export const Unresolved = z.object({
	ok: z.literal(false),
	key: z.string(),
	refusal: Refusal,
	/** The refusing entry's label. */
	entry: z.string().exactOptional(),
});
export type Unresolved = z.infer<typeof Unresolved>;

export const modelSet = shapeSet("model", {
	ModelUse,
	Binding,
	RowIdentity,
	ModelRow,
	PriceTier,
	Price,
	ModelView,
	TestRecord,
	StoredRegistry,
	ProviderRow,
	BrowseModel,
	BrowseDetail,
	Via,
	Refusal,
	Unresolved,
});
