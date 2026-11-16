// Shared Chat Completions codec.

import { stringifyJson } from "@nyaaai/contract/json";
import type {
	CanonicalRequest,
	Failure,
	FinishReason,
	HeaderList,
	NormalizedUsage,
	Serving,
	SseItem,
	StepEvent,
	StepResult,
	ThoughtDelta,
	ToolSpec,
	Turn,
	TurnPart,
} from "@nyaaai/contract/shapes/step";
import type { Codec, StepTurn } from "../codec";
import { failed, messageOf, ProviderError, type WireError } from "../failure";
import { isJson, parsedChunk, schemaPrompt } from "../native";
import { StepAssembler } from "../step";
import { normalizeResponseValue, textOf } from "./text";
import {
	type AssistantMessage,
	type ChatMessage,
	type ChatRequest,
	type ChatTool,
	type Chunk,
	type Completion,
	type Delta,
	readChunk,
	readCompletion,
	readError,
	type TextPart,
	type Usage,
} from "./wire";

////////////////////////////////
//  Interfaces & Types

/** Host-specific Chat Completions extensions. */
export interface ChatDialect {
	/** Defaults to `/chat/completions`. */
	path?: string;
	headers?(request: CanonicalRequest): Record<string, string>;
	/** Provider-specific request fields. */
	fields(request: CanonicalRequest, mode: "stream" | "complete"): Partial<ChatRequest>;
	/** Thought replay fields. */
	replay(step: StepTurn, request: CanonicalRequest): Partial<AssistantMessage>;
	/** Thought block identities. */
	thoughts(delta: Delta): ThoughtDelta[];
	/** Optional explicit cache control. */
	cached?(text: string, ttl: "1h" | undefined): TextPart[];
	/** Enforced vs prompt-only schema. */
	schema(json: Record<string, unknown>): Record<string, unknown> | undefined;
	/** Enforcement hides descriptions. */
	promptSchema?: true;
	/** Omitted: forced; `undefined`: unforced. */
	toolChoice?(name: string): ChatRequest["tool_choice"];
	/** Minimum `max_tokens`. */
	minTokens?: number;
	usage(usage: Usage): NormalizedUsage;
	serving(reply: { id?: string; provider?: string }): Serving;
	/** HTTP 200 stream errors. */
	failure(status: number | "stream", error: WireError, headers?: HeaderList): Failure;
}

type CallPart = Extract<TurnPart, { kind: "call" }>;

////////////////////////////////
//  Constants

const PATH = "/chat/completions";

const FINISHES: readonly FinishReason[] = ["stop", "tool_calls", "length", "content_filter", "error"];

////////////////////////////////
//  Functions & Helpers

function finishOf(reason: string): FinishReason {
	return FINISHES.find((each) => each === reason) ?? (reason.length === 0 ? "" : "stop");
}

function toolOf(tool: ToolSpec): ChatTool {
	return {
		type: "function",
		function: { name: tool.name, description: tool.description, parameters: tool.parameters },
	};
}

function turnMessages(d: ChatDialect, request: CanonicalRequest, turn: Turn): ChatMessage[] {
	if (turn.role === "user") return [{ role: "user", content: turn.text }];
	const text = turn.parts.flatMap((part) => (part.kind === "text" ? [part.text] : [])).join("");
	const calls = turn.parts.filter((part): part is CallPart => part.kind === "call");
	const wireOf = new Map(calls.map((call) => [call.index, call.wireId]));
	const message: AssistantMessage = {
		role: "assistant",
		content: text.length === 0 && calls.length > 0 ? null : text,
	};
	if (calls.length > 0)
		message.tool_calls = calls.map((call) => ({
			id: call.wireId,
			type: "function",
			function: { name: call.name, arguments: call.arguments },
		}));
	Object.assign(message, d.replay(turn, request));
	const results = turn.parts.flatMap((part): ChatMessage[] => {
		if (part.kind !== "result") return [];
		const wireId = wireOf.get(part.index);
		return wireId === undefined ? [] : [{ role: "tool", tool_call_id: wireId, content: part.output }];
	});
	return [message, ...results];
}

/** Marks the last cacheable message. */
function withLastCached(history: readonly ChatMessage[], mark: (text: string) => string | TextPart[]): ChatMessage[] {
	const marked = [...history];
	for (let at = marked.length - 1; at >= 0; at--) {
		const message = marked[at];
		if (message === undefined || message.role === "assistant" || typeof message.content !== "string") continue;
		marked[at] = { ...message, content: mark(message.content) };
		break;
	}
	return marked;
}

// State stays last.
// Implicit caching marks system.
// Explicit caching marks history.
// Explicit state is user-role.
function messagesOf(d: ChatDialect, request: CanonicalRequest, prompted: string | undefined): ChatMessage[] {
	const explicit = request.cache.kind === "explicit";
	const mark = (text: string): string | TextPart[] =>
		explicit && d.cached !== undefined ? d.cached(text, request.cache.ttl) : text;
	const last = request.system.length - 1;
	const messages: ChatMessage[] = request.system.map((text, at) => ({
		role: "system",
		content: at === last ? mark(text) : text,
	}));
	if (request.summary !== undefined) messages.push({ role: "system", content: mark(request.summary) });
	if (prompted !== undefined) messages.push({ role: "system", content: prompted });
	const history = request.history.flatMap((turn) => turnMessages(d, request, turn));
	messages.push(...(explicit ? withLastCached(history, mark) : history));
	if (request.state !== undefined) messages.push({ role: explicit ? "user" : "system", content: request.state });
	return messages;
}

function bodyOf(d: ChatDialect, request: CanonicalRequest, mode: "stream" | "complete"): ChatRequest {
	const { tools, toolChoice, schema, maxTokens } = request;
	const enforced = schema === undefined ? undefined : d.schema(schema.json);
	const prompted =
		schema !== undefined && (enforced === undefined || d.promptSchema === true)
			? schemaPrompt(schema.json)
			: undefined;
	const forced =
		toolChoice === undefined
			? undefined
			: d.toolChoice === undefined
				? { type: "function" as const, function: { name: toolChoice.name } }
				: d.toolChoice(toolChoice.name);
	return {
		model: request.model,
		messages: messagesOf(d, request, prompted),
		stream: mode === "stream",
		...d.fields(request, mode),
		...(tools === undefined || tools.length === 0 ? {} : { tools: tools.map(toolOf) }),
		...(forced === undefined ? {} : { tool_choice: forced }),
		...(schema === undefined || enforced === undefined
			? {}
			: {
					response_format: {
						type: "json_schema",
						json_schema: { name: schema.name, strict: true, schema: enforced },
					},
				}),
		...(maxTokens === undefined ? {} : { max_tokens: Math.max(d.minTokens ?? 0, maxTokens) }),
	};
}

function servingEvent(d: ChatDialect, reply: { id?: string; provider?: string }): StepEvent[] {
	const serving = d.serving(reply);
	return serving.host === undefined && serving.generationId === undefined ? [] : [{ kind: "serving", serving }];
}

/** Thoughts precede text. */
function* deltaEvents(d: ChatDialect, delta: Delta): Generator<StepEvent> {
	for (const thought of d.thoughts(delta)) yield { kind: "thought", ...thought };
	const value = normalizeResponseValue(delta.content);
	const text = isJson(value) ? stringifyJson(value) : textOf(value);
	if (text.length > 0) yield { kind: "text", text };
}

function* chunkEvents(d: ChatDialect, chunk: Chunk): Generator<StepEvent> {
	yield* servingEvent(d, chunk);
	for (const choice of chunk.choices ?? []) {
		const delta = choice.delta;
		if (delta !== undefined) {
			yield* deltaEvents(d, delta);
			for (const call of delta.tool_calls ?? []) {
				if (call.index === undefined) throw failed("stream", "A tool call had no valid index.");
				yield {
					kind: "call",
					index: call.index,
					wireId: call.id ?? "",
					name: call.function?.name ?? "",
					argumentsText: call.function?.arguments ?? "",
				};
			}
		}
		if (choice.finish_reason !== undefined) yield { kind: "finish", reason: finishOf(choice.finish_reason) };
	}
	if (chunk.usage !== undefined) yield { kind: "usage", usage: d.usage(chunk.usage) };
}

function* replyEvents(d: ChatDialect, reply: Completion): Generator<StepEvent> {
	yield* servingEvent(d, reply);
	const choice = reply.choices?.[0];
	if (choice?.message !== undefined) {
		yield* deltaEvents(d, choice.message);
		for (const [index, call] of (choice.message.tool_calls ?? []).entries())
			yield {
				kind: "call",
				index,
				wireId: call.id,
				name: call.function.name,
				argumentsText: call.function.arguments,
			};
	}
	if (choice?.finish_reason !== undefined) yield { kind: "finish", reason: finishOf(choice.finish_reason) };
	if (reply.usage !== undefined) yield { kind: "usage", usage: d.usage(reply.usage) };
}

/** HTTP 200 error details. */
function replyFailure(d: ChatDialect, reply: Chunk | Completion, error: WireError): ProviderError {
	const failure = d.failure("stream", error);
	const serving = d.serving(reply);
	if (reply.usage === undefined && serving.host === undefined && serving.generationId === undefined)
		return new ProviderError(failure);
	return new ProviderError({
		...failure,
		billed: { ...(reply.usage === undefined ? {} : { usage: d.usage(reply.usage) }), ...serving },
	});
}

async function* streamed(d: ChatDialect, items: AsyncIterable<SseItem>): AsyncGenerator<StepEvent> {
	let finished = false;
	for await (const item of items) {
		if (item.kind === "comment") continue;
		if (item.kind === "done") {
			if (!finished) throw failed("stream", "The provider ended the step.");
			return;
		}
		const chunk = readChunk(parsedChunk(item.data));
		if (chunk.error !== undefined) throw replyFailure(d, chunk, chunk.error);
		for (const event of chunkEvents(d, chunk)) {
			if (event.kind === "finish") finished = true;
			yield event;
		}
	}
	throw failed("truncated", "The stream ended before [DONE].");
}

function completed(d: ChatDialect, body: unknown): StepResult {
	if (body === null || typeof body !== "object") throw failed("stream", "The reply was not JSON.");
	const reply = readCompletion(isJson(body) ? body : {});
	if (reply.error !== undefined) throw replyFailure(d, reply, reply.error);
	const assembler = new StepAssembler();
	for (const event of replyEvents(d, reply)) assembler.add(event);
	const result = assembler.result();
	if (result.finish === "") throw failed("stream", "The provider ended the step.");
	return result;
}

/** Chat Completions codec for one dialect. */
export function chatCodec(d: ChatDialect): Codec {
	return {
		encode: (request, mode) => {
			const headers = d.headers?.(request);
			return {
				path: d.path ?? PATH,
				body: bodyOf(d, request, mode),
				...(headers === undefined ? {} : { headers }),
			};
		},
		stream: (items) => streamed(d, items),
		complete: (body) => completed(d, body),
		failure: (status, body, headers) => {
			const error = (isJson(body) ? readError(body["error"]) : undefined) ?? {};
			return d.failure(status, { ...error, message: messageOf(error.message) ?? `HTTP ${status}` }, headers);
		},
	};
}
