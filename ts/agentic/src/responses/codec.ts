// OpenAI Responses codec.

import { numberText } from "@nyaaai/contract/number";
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
import { amountAt } from "../hosts/opencode";
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

/** What one output item has shown so far. */
interface ItemState {
	call?: CallState;
	thought?: ThoughtState;
	phase?: string;
	texted: boolean;
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
	const prompt = isJson(raw["input_tokens_details"]) ? raw["input_tokens_details"] : {};
	const completion = isJson(raw["output_tokens_details"]) ? raw["output_tokens_details"] : {};
	const cached = amountAt(prompt, "cached_tokens");
	const cacheWrite = amountAt(prompt, "cache_write_tokens");
	const reasoning = amountAt(completion, "reasoning_tokens");
	return host.usage(
		{
			input: amountAt(raw, "input_tokens") ?? 0,
			output: amountAt(raw, "output_tokens") ?? 0,
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

/** An item's id, call id and output place. */
function keysOf(id: string, callId: string, place: number | undefined): string[] {
	return [
		...(id.length === 0 ? [] : [`id:${id}`]),
		...(callId.length === 0 ? [] : [`call:${callId}`]),
		...(place === undefined ? [] : [`at:${numberText(place)}`]),
	];
}

/** A label for a thought block. */
function labelOf(id: string, place: number | undefined): string {
	if (id.length > 0) return id;
	return place === undefined ? "" : `at:${numberText(place)}`;
}

/** Response items keyed by id, else by place. */
class Decoder {
	finished = false;
	private readonly items = new Map<string, ItemState>();
	private called = 0;
	private readonly serving: Serving = {};
	private usage: NormalizedUsage | undefined;

	constructor(private readonly host: Host) {}

	*event(type: unknown, data: Json): Generator<StepEvent> {
		if (errored(data["error"])) throw this.failure(errorBody(data["error"]));
		const itemId = stringOf(data["item_id"]);
		const place = numberOf(data["output_index"]);
		const keys = keysOf(itemId, "", place);
		const label = labelOf(itemId, place);
		const delta = stringOf(data["delta"]);
		switch (type) {
			case "response.created":
			case "response.in_progress":
				yield* this.served(data["response"]);
				return;
			case "response.output_item.added":
				if (isJson(data["item"])) yield* this.added(data["item"], place);
				return;
			case "response.output_item.done":
				if (isJson(data["item"])) yield* this.done(data["item"], place);
				return;
			case "response.output_text.delta":
			case "response.refusal.delta": {
				const state = this.state(keys);
				state.texted = true;
				yield { kind: "text", text: delta, ...phaseOf(state) };
				return;
			}
			case "response.function_call_arguments.delta":
				yield* this.argued(this.state(keys), undefined, delta);
				return;
			case "response.function_call_arguments.done":
				yield* this.argued(this.state(keys), undefined, undefined, finalOf(data["arguments"]));
				return;
			case "response.reasoning_summary_text.delta":
				yield this.thought(
					this.state(keys),
					itemId,
					label,
					"summary",
					numberOf(data["summary_index"]) ?? 0,
					delta,
				);
				return;
			case "response.reasoning_text.delta":
				yield this.thought(
					this.state(keys),
					itemId,
					label,
					"content",
					numberOf(data["content_index"]) ?? 0,
					delta,
				);
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
				throw this.failure(data);
		}
	}

	/** A complete response. */
	*whole(response: Json): Generator<StepEvent> {
		yield* this.served(response);
		const output = Array.isArray(response["output"]) ? response["output"] : [];
		for (const [place, item] of output.entries()) if (isJson(item)) yield* this.done(item, place);
		yield* this.ended(response, "");
	}

	private failure(error: Json): Error {
		return streamFailure(
			this.host,
			{ message: stringOf(error["message"]) || "The response failed.", ...typeOf(error["code"]) },
			this.usage,
			this.serving,
		);
	}

	/** The state every key names, made when none has one. */
	private state(keys: string[]): ItemState {
		const held = keys.map((key) => this.items.get(key)).find((each) => each !== undefined);
		const state = held ?? { texted: false };
		for (const key of keys) if (!this.items.has(key)) this.items.set(key, state);
		return state;
	}

	private *served(response: unknown): Generator<StepEvent> {
		if (!isJson(response) || this.serving.generationId !== undefined) return;
		for (const event of servingEvents(response["id"])) {
			if (event.kind === "serving" && event.serving.generationId !== undefined)
				this.serving.generationId = event.serving.generationId;
			yield event;
		}
	}

	private *added(item: Json, place: number | undefined): Generator<StepEvent> {
		const call = item["type"] === "function_call";
		const state = this.state(keysOf(stringOf(item["id"]), call ? stringOf(item["call_id"]) : "", place));
		if (item["type"] === "message" && typeof item["phase"] === "string") state.phase = item["phase"];
		if (call) yield* this.argued(state, item, stringOf(item["arguments"]));
	}

	private *done(item: Json, place: number | undefined): Generator<StepEvent> {
		const id = stringOf(item["id"]);
		const call = item["type"] === "function_call";
		const state = this.state(keysOf(id, call ? stringOf(item["call_id"]) : "", place));
		if (call) yield* this.argued(state, item, undefined, finalOf(item["arguments"]));
		if (item["type"] === "message" && !state.texted) {
			if (typeof item["phase"] === "string" && state.phase === undefined) state.phase = item["phase"];
			state.texted = true;
			yield { kind: "text", text: messageText(item), ...phaseOf(state) };
		}
		if (item["type"] === REASONING) yield* this.reasoned(state, id, labelOf(id, place), item);
	}

	/** Final arguments backfill deltas. */
	private *argued(state: ItemState, item: Json | undefined, delta?: string, whole?: string): Generator<StepEvent> {
		let call = state.call;
		if (call === undefined) {
			call = { index: this.called++, argumentsText: "" };
			state.call = call;
		}
		let more = delta ?? "";
		if (delta === undefined && whole !== undefined) {
			if (!whole.startsWith(call.argumentsText))
				throw failed("stream", "A tool call's final arguments contradict the ones streamed.");
			more = whole.slice(call.argumentsText.length);
		}
		call.argumentsText += more;
		yield {
			kind: "call",
			index: call.index,
			wireId: stringOf(item?.["call_id"]),
			name: stringOf(item?.["name"]),
			argumentsText: more,
		};
	}

	private thought(
		item: ItemState,
		id: string,
		label: string,
		kind: "summary" | "content",
		at: number,
		delta: string,
	): StepEvent {
		const state = item.thought ?? { streamed: false };
		const split = state[kind] !== undefined && state[kind] !== at;
		state[kind] = at;
		state.streamed = true;
		item.thought = state;
		return {
			kind: "thought",
			part: { kind: "thought", type: REASONING, segments: [{ kind, text: delta }], id },
			block: label,
			...(split ? { split: true as const } : {}),
		};
	}

	/** Done events add reasoning. */
	private *reasoned(state: ItemState, id: string, label: string, item: Json): Generator<StepEvent> {
		const opaque = stringOf(item["encrypted_content"]);
		const segments = state.thought?.streamed ? [] : segmentsOf(item);
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
			block: label,
		};
	}

	/** Status has precedence. */
	private *ended(response: unknown, fallback: string): Generator<StepEvent> {
		const body = isJson(response) ? response : {};
		yield* this.served(body);
		this.usage = usageOf(this.host, body["usage"]) ?? this.usage;
		const status = typeof body["status"] === "string" ? body["status"] : fallback;
		if (status === "failed" || errored(body["error"])) throw this.failure(errorBody(body["error"]));
		if (status !== "completed" && status !== "incomplete") throw failed("stream", "The provider ended the step.");
		if (this.usage !== undefined) yield { kind: "usage", usage: this.usage };
		yield { kind: "finish", reason: finishOf(status, body["incomplete_details"], this.called > 0) };
		this.finished = true;
	}
}

function phaseOf(state: ItemState): { phase?: string } {
	return state.phase === undefined ? {} : { phase: state.phase };
}

/** Final text counts only when there is some. */
function finalOf(value: unknown): string | undefined {
	return typeof value === "string" && value.length > 0 ? value : undefined;
}

/** Empty values are no error. */
function errored(raw: unknown): boolean {
	return !(raw === undefined || raw === null || raw === false || raw === 0 || raw === "");
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
