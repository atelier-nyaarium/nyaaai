// OpenAI Responses codec.

import type {
	CanonicalRequest,
	CodecFacts,
	FinishReason,
	NormalizedUsage,
	Serving,
	SseItem,
	StepEvent,
	StepResult,
	ThoughtPart,
	ToolSpec,
	Turn,
} from "@nyaaai/contract/shapes/step";
import type { Codec, StepTurn } from "../codec";
import { failed } from "../failure";
import {
	assembled,
	type Host,
	isJson,
	type Json,
	numberOf,
	parsedChunk,
	schemaPrompt,
	servingEvents,
	streamFailure,
	stringOf,
	systemTexts,
	wireError,
} from "../native";
import { strictSchema } from "../schemaProjection";
import type { FunctionTool, InputItem, ReasoningItem, ResponsesRequest } from "./wire";

////////////////////////////////
//  Interfaces & Types

interface CallState {
	index: number;
	argumentsText: string;
}

interface ThoughtState {
	summary?: number;
	content?: number;
	streamed: boolean;
}

////////////////////////////////
//  Constants

const PATH = "/responses";

const REASONING = "reasoning";

const MIN_TOKENS = 16;

////////////////////////////////
//  Functions & Helpers

/** Shared model family IDs. */
function familyOf(model: string): string {
	return model
		.slice(model.lastIndexOf("/") + 1)
		.split("-")
		.slice(0, 2)
		.join("-");
}

function replayable(part: ThoughtPart): part is ThoughtPart & { id: string; opaque: string } {
	return part.type === REASONING && part.id !== undefined && part.opaque !== undefined;
}

function reasoningItem(part: ThoughtPart & { id: string; opaque: string }): ReasoningItem {
	const segments = part.segments ?? [];
	const content = segments.filter((segment) => segment.kind === "content");
	return {
		type: "reasoning",
		id: part.id,
		summary: segments
			.filter((segment) => segment.kind === "summary")
			.map((segment) => ({ type: "summary_text", text: segment.text })),
		...(content.length === 0
			? {}
			: { content: content.map((segment) => ({ type: "reasoning_text" as const, text: segment.text })) }),
		encrypted_content: part.opaque,
	};
}

/** Same-family reasoning replay. */
function stepItems(step: StepTurn, request: CanonicalRequest): InputItem[] {
	const thoughts = step.parts.filter((part): part is ThoughtPart => part.kind === "thought");
	const sent =
		step.producer !== undefined &&
		familyOf(step.producer.model) === familyOf(request.model) &&
		thoughts.every(replayable);
	const wireOf = new Map(step.parts.flatMap((part) => (part.kind === "call" ? [[part.index, part.wireId]] : [])));
	const phase = wireOf.size > 0 ? "commentary" : "final_answer";
	const items = step.parts.flatMap((part): InputItem[] => {
		if (part.kind === "thought") return sent && replayable(part) ? [reasoningItem(part)] : [];
		if (part.kind === "text")
			return part.text.length === 0
				? []
				: [{ type: "message", role: "assistant", content: part.text, phase: part.phase ?? phase }];
		if (part.kind === "call")
			return [{ type: "function_call", call_id: part.wireId, name: part.name, arguments: part.arguments }];
		return [];
	});
	const outputs = step.parts.flatMap((part): InputItem[] => {
		const callId = part.kind === "result" ? wireOf.get(part.index) : undefined;
		return part.kind !== "result" || callId === undefined
			? []
			: [{ type: "function_call_output", call_id: callId, output: part.output }];
	});
	return [...items, ...outputs];
}

function turnItems(turn: Turn, request: CanonicalRequest): InputItem[] {
	return turn.role === "user" ? [{ type: "message", role: "user", content: turn.text }] : stepItems(turn, request);
}

function toolOf(tool: ToolSpec): FunctionTool {
	return { type: "function", name: tool.name, description: tool.description, parameters: tool.parameters };
}

/** Named or any-tool choice. */
function choiceOf(choice: readonly string[], name: string): ResponsesRequest["tool_choice"] {
	if (choice.includes("tool")) return { type: "function", name };
	return choice.includes("required") ? "required" : undefined;
}

/** Catalog gates optional fields. */
function bodyOf(
	facts: CodecFacts | undefined,
	request: CanonicalRequest,
	mode: "stream" | "complete",
): ResponsesRequest {
	const { tools, toolChoice, schema, effort, maxTokens } = request;
	const api = facts?.api;
	const enforced = schema !== undefined && api?.structured === true ? strictSchema(schema.json) : undefined;
	const prompted = schema !== undefined && enforced === undefined ? schemaPrompt(schema.json) : undefined;
	const instructions = systemTexts(request, prompted).join("\n\n");
	const forced = toolChoice === undefined ? undefined : choiceOf(api?.choice ?? [], toolChoice.name);
	const reasoning: ResponsesRequest["reasoning"] =
		effort === undefined || !(facts?.efforts ?? []).includes(effort)
			? undefined
			: { effort, ...(effort === "none" ? {} : { summary: "auto" }) };
	return {
		model: request.model,
		...(instructions.length === 0 ? {} : { instructions }),
		input: [
			...request.history.flatMap((turn) => turnItems(turn, request)),
			...(request.state === undefined
				? []
				: [{ type: "message" as const, role: "developer" as const, content: request.state }]),
		],
		...(tools === undefined || tools.length === 0 ? {} : { tools: tools.map(toolOf) }),
		...(forced === undefined ? {} : { tool_choice: forced }),
		...(reasoning === undefined ? {} : { reasoning }),
		...(schema === undefined || enforced === undefined
			? {}
			: { text: { format: { type: "json_schema", name: schema.name, schema: enforced, strict: true } } }),
		store: false,
		include: ["reasoning.encrypted_content"],
		...(maxTokens === undefined ? {} : { max_output_tokens: Math.max(MIN_TOKENS, maxTokens) }),
		stream: mode === "stream",
	};
}

function usageOf(host: Host, raw: unknown): NormalizedUsage | undefined {
	if (!isJson(raw)) return undefined;
	const input = numberOf(raw["input_tokens"]);
	const output = numberOf(raw["output_tokens"]);
	if (input === undefined || output === undefined) return undefined;
	const prompt = isJson(raw["input_tokens_details"]) ? raw["input_tokens_details"] : {};
	const completion = isJson(raw["output_tokens_details"]) ? raw["output_tokens_details"] : {};
	const cached = numberOf(prompt["cached_tokens"]);
	const cacheWrite = numberOf(prompt["cache_write_tokens"]);
	const reasoning = numberOf(completion["reasoning_tokens"]);
	return host.usage(
		{
			input,
			output,
			...(cached === undefined ? {} : { cached }),
			...(cacheWrite === undefined ? {} : { cacheWrite }),
			...(reasoning === undefined ? {} : { reasoning }),
		},
		raw,
	);
}

/** Output text and refusals. */
function messageText(item: Json): string {
	const content = Array.isArray(item["content"]) ? item["content"] : [];
	return content.map((part) => (isJson(part) ? stringOf(part["text"]) || stringOf(part["refusal"]) : "")).join("");
}

function segmentsOf(item: Json): NonNullable<ThoughtPart["segments"]> {
	const of = (field: unknown, kind: "summary" | "content") =>
		(Array.isArray(field) ? field : []).flatMap((part) =>
			isJson(part) && typeof part["text"] === "string" ? [{ kind, text: part["text"] }] : [],
		);
	return [...of(item["summary"], "summary"), ...of(item["content"], "content")];
}

////////////////////////////////
//  Classes

/** Response items keyed by ID. */
class Decoder {
	finished = false;
	private readonly calls = new Map<string, CallState>();
	private readonly thoughts = new Map<string, ThoughtState>();
	private readonly phases = new Map<string, string>();
	private readonly texts = new Set<string>();
	private readonly serving: Serving = {};
	private usage: NormalizedUsage | undefined;

	constructor(private readonly host: Host) {}

	*event(type: unknown, data: Json): Generator<StepEvent> {
		const itemId = stringOf(data["item_id"]);
		const delta = stringOf(data["delta"]);
		switch (type) {
			case "response.created":
			case "response.in_progress":
				yield* this.served(data["response"]);
				return;
			case "response.output_item.added":
				if (isJson(data["item"])) yield* this.added(data["item"]);
				return;
			case "response.output_item.done":
				if (isJson(data["item"])) yield* this.done(data["item"]);
				return;
			case "response.output_text.delta":
			case "response.refusal.delta":
				this.texts.add(itemId);
				yield { kind: "text", text: delta, ...this.phaseOf(itemId) };
				return;
			case "response.function_call_arguments.delta":
				yield* this.argued(itemId, undefined, delta);
				return;
			case "response.function_call_arguments.done":
				yield* this.argued(itemId, undefined, undefined, stringOf(data["arguments"]));
				return;
			case "response.reasoning_summary_text.delta":
				yield this.thought(itemId, "summary", numberOf(data["summary_index"]) ?? 0, delta);
				return;
			case "response.reasoning_text.delta":
				yield this.thought(itemId, "content", numberOf(data["content_index"]) ?? 0, delta);
				return;
			case "response.completed":
				yield* this.ended(data["response"], "completed");
				return;
			case "response.incomplete":
				yield* this.ended(data["response"], "incomplete");
				return;
			case "response.failed":
				yield* this.ended(data["response"], "failed");
				return;
			case "error":
				throw streamFailure(
					this.host,
					{ message: stringOf(data["message"]) || "The response failed.", ...typeOf(data["code"]) },
					this.usage,
					this.serving,
				);
		}
	}

	/** Decode a complete response. */
	*whole(response: Json): Generator<StepEvent> {
		yield* this.served(response);
		for (const item of Array.isArray(response["output"]) ? response["output"] : [])
			if (isJson(item)) yield* this.done(item);
		yield* this.ended(response, "");
	}

	private *served(response: unknown): Generator<StepEvent> {
		if (!isJson(response) || this.serving.generationId !== undefined) return;
		for (const event of servingEvents(response["id"])) {
			if (event.kind === "serving" && event.serving.generationId !== undefined)
				this.serving.generationId = event.serving.generationId;
			yield event;
		}
	}

	private phaseOf(itemId: string): { phase?: string } {
		const phase = this.phases.get(itemId);
		return phase === undefined ? {} : { phase };
	}

	private *added(item: Json): Generator<StepEvent> {
		const id = stringOf(item["id"]);
		if (item["type"] === "message" && typeof item["phase"] === "string") this.phases.set(id, item["phase"]);
		if (item["type"] === "function_call") yield* this.argued(id, item, stringOf(item["arguments"]));
	}

	private *done(item: Json): Generator<StepEvent> {
		const id = stringOf(item["id"]);
		if (item["type"] === "function_call") yield* this.argued(id, item, undefined, stringOf(item["arguments"]));
		if (item["type"] === "message" && !this.texts.has(id)) {
			if (typeof item["phase"] === "string" && !this.phases.has(id)) this.phases.set(id, item["phase"]);
			this.texts.add(id);
			yield { kind: "text", text: messageText(item), ...this.phaseOf(id) };
		}
		if (item["type"] === REASONING) yield* this.reasoned(id, item);
	}

	/** Final arguments backfill deltas. */
	private *argued(id: string, item: Json | undefined, delta?: string, whole?: string): Generator<StepEvent> {
		let call = this.calls.get(id);
		if (call === undefined) {
			call = { index: this.calls.size, argumentsText: "" };
			this.calls.set(id, call);
		}
		const more = delta ?? (whole?.startsWith(call.argumentsText) ? whole.slice(call.argumentsText.length) : "");
		call.argumentsText += more;
		yield {
			kind: "call",
			index: call.index,
			wireId: stringOf(item?.["call_id"]),
			name: stringOf(item?.["name"]),
			argumentsText: more,
		};
	}

	private thought(id: string, kind: "summary" | "content", at: number, delta: string): StepEvent {
		const state = this.thoughts.get(id) ?? { streamed: false };
		const split = state[kind] !== undefined && state[kind] !== at;
		state[kind] = at;
		state.streamed = true;
		this.thoughts.set(id, state);
		return {
			kind: "thought",
			part: { kind: "thought", type: REASONING, segments: [{ kind, text: delta }], id },
			block: id,
			...(split ? { split: true as const } : {}),
		};
	}

	/** Done events add reasoning. */
	private *reasoned(id: string, item: Json): Generator<StepEvent> {
		const opaque = stringOf(item["encrypted_content"]);
		const segments = this.thoughts.get(id)?.streamed ? [] : segmentsOf(item);
		if (segments.length === 0 && opaque.length === 0) return;
		yield {
			kind: "thought",
			part: {
				kind: "thought",
				type: REASONING,
				...(segments.length === 0 ? {} : { segments }),
				...(opaque.length === 0 ? {} : { opaque }),
				id,
			},
			block: id,
		};
	}

	/** Status has precedence. */
	private *ended(response: unknown, fallback: string): Generator<StepEvent> {
		const body = isJson(response) ? response : {};
		yield* this.served(body);
		this.usage = usageOf(this.host, body["usage"]) ?? this.usage;
		const status = typeof body["status"] === "string" ? body["status"] : fallback;
		if (status === "failed") {
			const error = errorBody(body["error"]);
			throw streamFailure(
				this.host,
				{ message: stringOf(error["message"]) || "The response failed.", ...typeOf(error["code"]) },
				this.usage,
				this.serving,
			);
		}
		if (status !== "completed" && status !== "incomplete") throw failed("stream", "The provider ended the step.");
		if (this.usage !== undefined) yield { kind: "usage", usage: this.usage };
		yield { kind: "finish", reason: finishOf(status, body["incomplete_details"], this.calls.size > 0) };
		this.finished = true;
	}
}

/** A text is the message. */
function errorBody(raw: unknown): Json {
	if (typeof raw === "string") return { message: raw };
	return isJson(raw) ? raw : {};
}

function typeOf(code: unknown): { type?: string } {
	return typeof code === "string" && code.length > 0 ? { type: code } : {};
}

function finishOf(status: string, details: unknown, called: boolean): FinishReason {
	if (status === "completed") return called ? "tool_calls" : "stop";
	return isJson(details) && details["reason"] === "content_filter" ? "content_filter" : "length";
}

async function* streamed(host: Host, items: AsyncIterable<SseItem>): AsyncGenerator<StepEvent> {
	const decoder = new Decoder(host);
	for await (const item of items) {
		if (item.kind === "comment") continue;
		if (item.kind === "done") break;
		const data = parsedChunk(item.data);
		yield* decoder.event(data["type"] ?? item.event, data);
		if (decoder.finished) return;
	}
	throw failed("truncated", "The stream ended before the response completed.");
}

function completed(host: Host, body: unknown): StepResult {
	if (!isJson(body)) throw failed("stream", "The reply was not JSON.");
	return assembled(new Decoder(host).whole(body));
}

////////////////////////////////
//  Codec

export function responsesCodec(host: Host, facts: CodecFacts | undefined): Codec {
	return {
		encode: (request, mode) => ({
			path: `${host.prefix}${PATH}`,
			body: bodyOf(facts, request, mode),
			headers: host.headers(request),
		}),
		stream: (items) => streamed(host, items),
		complete: (body) => completed(host, body),
		failure: (status, body, headers) => host.failure(status, wireError(status, body), headers),
	};
}
