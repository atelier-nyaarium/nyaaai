// OpenCode Chat dialect.

import type { CanonicalRequest, CodecFacts, NormalizedUsage, ThoughtDelta } from "@nyaaai/contract/shapes/step";
import type { Clock } from "@nyaaai/contract/time";
import { type ChatDialect, chatCodec } from "../chat/codec";
import { thoughtText } from "../chat/text";
import type { AssistantMessage, ChatRequest, Delta, Usage } from "../chat/wire";
import type { Codec, StepTurn } from "../codec";
import { opencodeFailure, opencodeUsage, SESSION_HEADER } from "../hosts/opencode";
import { strictSchema } from "../schemaProjection";

////////////////////////////////
//  Constants

const PATH = "/openai/v1/chat/completions";

const REASONING = "reasoning_content";

/** Post-translation token floor. */
const MIN_TOKENS = 16;

////////////////////////////////
//  Functions & Helpers

/** DeepSeek V4 aliases. */
function familyOf(model: string): string {
	const name = model.slice(model.lastIndexOf("/") + 1);
	const dash = name.indexOf("-");
	return dash < 0 ? name : name.slice(0, dash);
}

/** Preserve tool-turn reasoning. */
function replayOf(step: StepTurn, request: CanonicalRequest): Partial<AssistantMessage> {
	const own =
		step.producer !== undefined && familyOf(step.producer.model) === familyOf(request.model)
			? step.parts
					.flatMap((part) => (part.kind === "thought" && part.type === REASONING ? [thoughtText(part)] : []))
					.join("")
			: "";
	if (own.length > 0) return { reasoning_content: own };
	return step.parts.some((part) => part.kind === "call") ? { reasoning_content: "" } : {};
}

function thoughtsOf(delta: Delta): ThoughtDelta[] {
	const text = delta.reasoning_content;
	if (typeof text !== "string" || text.length === 0) return [];
	return [{ part: { kind: "thought", type: REASONING, segments: [{ kind: "content", text }] } }];
}

/** Map named and any-tool. */
function choiceOf(choice: readonly string[], name: string): ChatRequest["tool_choice"] {
	if (choice.includes("tool")) return { type: "function", function: { name } };
	return choice.includes("required") ? "required" : undefined;
}

function normalized(usage: Usage): NormalizedUsage {
	const cached = usage.prompt_tokens_details?.cached_tokens;
	const cacheWrite = usage.prompt_tokens_details?.cache_write_tokens;
	const reasoning = usage.completion_tokens_details?.reasoning_tokens;
	return opencodeUsage(
		{
			input: usage.prompt_tokens,
			output: usage.completion_tokens,
			...(cached === undefined ? {} : { cached }),
			...(cacheWrite === undefined ? {} : { cacheWrite }),
			...(reasoning === undefined ? {} : { reasoning }),
		},
		{ ...usage },
	);
}

/** Catalog gates optional fields. */
function dialectOf(facts: CodecFacts | undefined, clock: Clock): ChatDialect {
	const api = facts?.api;
	const efforts = facts?.efforts ?? [];
	return {
		path: PATH,
		headers: (request) => ({ [SESSION_HEADER]: request.session }),
		fields: (request, mode) => ({
			...(mode === "stream" ? { stream_options: { include_usage: true } } : {}),
			...(request.effort !== undefined && efforts.includes(request.effort)
				? { reasoning_effort: request.effort }
				: {}),
		}),
		replay: (step, request) => (api?.reasoning === REASONING ? replayOf(step, request) : {}),
		thoughts: thoughtsOf,
		schema: (json) => (api?.structured === true ? strictSchema(json) : undefined),
		toolChoice: (name) => choiceOf(api?.choice ?? [], name),
		minTokens: MIN_TOKENS,
		usage: normalized,
		serving: (reply) => (reply.id === undefined ? {} : { generationId: reply.id }),
		failure: (status, error, headers) => opencodeFailure(clock, status, error, headers),
	};
}

////////////////////////////////
//  Codecs

/** Reset stamps count from the clock. */
export function opencodeChat(facts: CodecFacts | undefined, clock: Clock): Codec {
	return chatCodec(dialectOf(facts, clock));
}

/** Failures before model lookup. */
export function opencodeErrors(clock: Clock): Pick<Codec, "failure"> {
	return opencodeChat(undefined, clock);
}
