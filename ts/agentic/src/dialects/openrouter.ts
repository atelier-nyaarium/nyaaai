// OpenRouter Chat dialect.

import type {
	CanonicalRequest,
	Failure,
	FailureCategory,
	NormalizedUsage,
	ReplyPart,
	ThoughtPart,
	TurnPart,
} from "@nyaaai/contract/shapes/step";
import { asciiLower } from "@nyaaai/contract/text";
import { type ChatDialect, chatCodec } from "../chat/codec";
import type { Provider, ReasoningDetail, Usage } from "../chat/wire";
import type { Codec } from "../codec";
import { messageOf, type WireError } from "../failure";
import { strictSchema } from "../schemaProjection";

////////////////////////////////
//  Constants

export const PROVIDER = { zdr: true, data_collection: "deny", require_parameters: true } as const satisfies Provider;

const DETAIL_TYPES: readonly string[] = ["reasoning.text", "reasoning.summary", "reasoning.encrypted"];

const ENCRYPTED = "reasoning.encrypted";

////////////////////////////////
//  Functions & Helpers

export function thoughtsOf(details: readonly ReasoningDetail[]): ThoughtPart[] {
	return details.map((detail) => {
		const segments = [
			...(detail.text == null ? [] : [{ kind: "content" as const, text: detail.text }]),
			...(detail.summary == null ? [] : [{ kind: "summary" as const, text: detail.summary }]),
		];
		const opaque = detail.type === ENCRYPTED ? detail.data : detail.signature;
		return {
			kind: "thought",
			type: detail.type,
			...(segments.length === 0 ? {} : { segments }),
			...(opaque == null ? {} : { opaque }),
			...(detail.id == null ? {} : { id: detail.id }),
			...(detail.format === undefined ? {} : { format: detail.format }),
			...(detail.index === undefined ? {} : { index: detail.index }),
		};
	});
}

/** Thought parts OpenRouter reads back; others drop. */
export function detailsOf(parts: ReadonlyArray<TurnPart | ReplyPart>): ReasoningDetail[] {
	return parts.flatMap((part): ReasoningDetail[] => {
		if (part.kind !== "thought" || !DETAIL_TYPES.includes(part.type)) return [];
		const of = (kind: "content" | "summary") => {
			const texts = (part.segments ?? []).filter((segment) => segment.kind === kind);
			return texts.length === 0 ? undefined : texts.map((segment) => segment.text).join("");
		};
		const text = of("content");
		const summary = of("summary");
		return [
			{
				type: part.type,
				...(part.id === undefined ? {} : { id: part.id }),
				...(part.format === undefined ? {} : { format: part.format }),
				...(part.index === undefined ? {} : { index: part.index }),
				...(text === undefined ? {} : { text }),
				...(summary === undefined ? {} : { summary }),
				...(part.opaque === undefined
					? {}
					: part.type === ENCRYPTED
						? { data: part.opaque }
						: { signature: part.opaque }),
			},
		];
	});
}

/** Off allows hosts that keep or collect data, free models included. */
function providerFor(zdrOnly: boolean): Provider {
	return zdrOnly ? PROVIDER : { require_parameters: true };
}

/** Host of the newest step with thought blocks. */
function affinityOf(request: CanonicalRequest): string | undefined {
	for (const turn of [...request.history].reverse()) {
		if (turn.role === "step" && turn.producer?.model === request.model && detailsOf(turn.parts).length > 0)
			return turn.producer.host;
	}
	return undefined;
}

/** A provider's capacity refusal, never the key's own limits. */
function upstream(error: WireError): boolean {
	const meta = error.metadata ?? {};
	return (
		meta["error_type"] === "rate_limit_exceeded" ||
		typeof meta["provider_name"] === "string" ||
		asciiLower(error.message ?? "").includes("upstream")
	);
}

function categoryOf(code: number | undefined, error: WireError): FailureCategory {
	if (code === undefined) return "stream";
	if (code === 400)
		return asciiLower(error.message ?? "").includes("provider that created it") ? "replayRejected" : "badRequest";
	if (code === 401) return "keyRejected";
	if (code === 402) return "outOfCredit";
	if (code === 403) return "refused";
	if (code === 404) return "notFound";
	if (code === 429) return upstream(error) ? "overloaded" : "rateLimited";
	if (code === 408 || code >= 500) return "unavailable";
	return "badRequest";
}

/** By HTTP status, else by an in-stream error's code. */
function failureOf(status: number | "stream", error: WireError): Failure {
	const code = typeof status === "number" ? status : typeof error.code === "number" ? error.code : undefined;
	const reasons = error.metadata?.["reasons"];
	return {
		category: categoryOf(code, error),
		status,
		message: messageOf(error.message) ?? "OpenRouter reported an error.",
		...(Array.isArray(reasons)
			? { reasons: reasons.filter((reason): reason is string => typeof reason === "string") }
			: {}),
	};
}

function normalized(usage: Usage): NormalizedUsage {
	const cached = usage.prompt_tokens_details?.cached_tokens;
	const cacheWrite = usage.prompt_tokens_details?.cache_write_tokens;
	const reasoning = usage.completion_tokens_details?.reasoning_tokens;
	const upstreamCost = usage.cost_details?.upstream_inference_cost;
	return {
		input: usage.prompt_tokens,
		output: usage.completion_tokens,
		...(cached === undefined ? {} : { cached }),
		...(cacheWrite === undefined ? {} : { cacheWrite }),
		...(reasoning === undefined ? {} : { reasoning }),
		...(usage.cost === undefined ? {} : { cost: usage.cost }),
		...(upstreamCost == null ? {} : { upstreamCost }),
		...(usage.is_byok === undefined ? {} : { byok: usage.is_byok }),
	};
}

////////////////////////////////
//  Dialect

const DIALECT: ChatDialect = {
	fields: (request) => {
		const host = affinityOf(request);
		const provider = providerFor(request.zdrOnly);
		return {
			session_id: request.session,
			provider: host === undefined ? provider : { ...provider, order: [host] },
			...(request.effort === undefined ? {} : { reasoning: { effort: request.effort } }),
		};
	},
	// Preserve model-specific blocks.
	replay: (step, request) => {
		const details = step.producer?.model === request.model ? detailsOf(step.parts) : [];
		return details.length === 0 ? {} : { reasoning_details: details };
	},
	// Block identity: type/index.
	thoughts: (delta) => {
		if (delta.reasoning_details !== undefined && delta.reasoning_details.length > 0)
			return thoughtsOf(delta.reasoning_details).map((part) =>
				part.index === undefined ? { part } : { part, block: `${part.type}#${part.index}` },
			);
		if (typeof delta.reasoning !== "string" || delta.reasoning.length === 0) return [];
		return [
			{ part: { kind: "thought", type: "reasoning", segments: [{ kind: "content", text: delta.reasoning }] } },
		];
	},
	cached: (text, ttl) => [
		{ type: "text", text, cache_control: ttl === "1h" ? { type: "ephemeral", ttl: "1h" } : { type: "ephemeral" } },
	],
	schema: strictSchema,
	usage: normalized,
	serving: (reply) => ({
		...(reply.provider === undefined ? {} : { host: reply.provider }),
		...(reply.id === undefined ? {} : { generationId: reply.id }),
	}),
	failure: failureOf,
};

export const OPENROUTER_CHAT: Codec = chatCodec(DIALECT);
