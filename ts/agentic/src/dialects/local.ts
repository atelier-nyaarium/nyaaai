// Chat dialect for URL servers.

import type {
	CanonicalRequest,
	CodecFacts,
	Failure,
	FailureCategory,
	ThoughtDelta,
} from "@nyaaai/contract/shapes/step";
import { type ChatDialect, chatCodec } from "../chat/codec";
import { thoughtText } from "../chat/text";
import type { AssistantMessage, Delta } from "../chat/wire";
import type { Codec, StepTurn } from "../codec";
import { messageOf, type WireError } from "../failure";

////////////////////////////////
//  Interfaces & Types

/** One server's chat differences. */
export interface LocalDialect {
	/** Failure text name. */
	label: string;
	/** Omitted: thoughts stay out. */
	replay?: "reasoning" | "reasoning_content";
	/** Defaults to `/v1/chat/completions`. */
	chatPath?: `/${string}`;
	/** Requests carry a key when set. */
	optionalKey?: true;
}

////////////////////////////////
//  Constants

const PATH = "/v1/chat/completions";

const REASONING = "reasoning";

////////////////////////////////
//  Functions & Helpers

/** Thoughts stay model-local. */
function replayOf(local: LocalDialect, step: StepTurn, request: CanonicalRequest): Partial<AssistantMessage> {
	if (local.replay === undefined || step.producer?.model !== request.model) return {};
	const text = step.parts
		.flatMap((part) => (part.kind === "thought" && part.type === REASONING ? [thoughtText(part)] : []))
		.join("");
	return text.length === 0 ? {} : { [local.replay]: text };
}

function thoughtsOf(delta: Delta): ThoughtDelta[] {
	const text = delta.reasoning_content ?? delta.reasoning;
	if (typeof text !== "string" || text.length === 0) return [];
	return [{ part: { kind: "thought", type: REASONING, segments: [{ kind: "content", text }] } }];
}

function categoryOf(code: number | undefined, keyed: boolean): FailureCategory {
	if (code === undefined) return "stream";
	if (keyed && code === 401) return "keyRejected";
	if (keyed && code === 402) return "outOfCredit";
	if (code === 404) return "notFound";
	if (code === 401 || code === 403) return "refused";
	if (code === 429) return "rateLimited";
	if (code === 408 || code >= 500) return "unavailable";
	return "badRequest";
}

/** Schema for forced tools. */
function forcedAsSchema(request: CanonicalRequest): CanonicalRequest {
	const { toolChoice, tools, ...rest } = request;
	const forced = tools?.find((tool) => tool.name === toolChoice?.name);
	if (forced === undefined || request.schema !== undefined) return request;
	return { ...rest, schema: { name: forced.name, json: forced.parameters } };
}

/** Serving entry label. */
function dialectOf(facts: CodecFacts | undefined, host: string, local: LocalDialect): ChatDialect {
	const efforts = facts?.efforts ?? [];
	return {
		path: local.chatPath ?? PATH,
		fields: (request, mode) => ({
			...(mode === "stream" ? { stream_options: { include_usage: true } } : {}),
			...(request.effort !== undefined && efforts.includes(request.effort)
				? { reasoning_effort: request.effort }
				: {}),
		}),
		replay: (step, request) => replayOf(local, step, request),
		thoughts: thoughtsOf,
		// Grammar ignores `strict` and descriptions.
		schema: (json) => json,
		promptSchema: true,
		toolChoice: () => undefined,
		usage: (usage) => ({
			input: usage.prompt_tokens,
			output: usage.completion_tokens,
			...(facts?.free === true ? { cost: 0 } : {}),
		}),
		serving: (reply) => ({ host, ...(reply.id === undefined ? {} : { generationId: reply.id }) }),
		// 400: unsupported features possible.
		failure: (status: number | "stream", error: WireError): Failure => {
			const code = typeof status === "number" ? status : typeof error.code === "number" ? error.code : undefined;
			return {
				category: categoryOf(code, local.optionalKey === true),
				status,
				message: messageOf(error.message) ?? `${local.label} reported an error.`,
			};
		},
	};
}

////////////////////////////////
//  Codecs

export function localChat(facts: CodecFacts | undefined, host: string, local: LocalDialect): Codec {
	const codec = chatCodec(dialectOf(facts, host, local));
	return { ...codec, encode: (request, mode) => codec.encode(forcedAsSchema(request), mode) };
}
