// Anthropic Messages codec.

import { stringifyJson } from "@nyaaai/contract/json";
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
import { failed, type WireError } from "../failure";
import { amountAt } from "../hosts/opencode";
import {
	argumentsOf,
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
import type { Block, CacheControl, Message, MessagesRequest, Tool } from "./wire";

////////////////////////////////
//  Constants

const PATH = "/messages";

const THINKING = "thinking";
const REDACTED = "redacted_thinking";

/** Default token ceiling. */
const MAX_TOKENS = 32_000;

const FINISHES: ReadonlyMap<string, FinishReason> = new Map([
	["tool_use", "tool_calls"],
	["max_tokens", "length"],
	["model_context_window_exceeded", "length"],
	["refusal", "content_filter"],
]);

////////////////////////////////
//  Functions & Helpers

/** A text is the message. */
function errorBody(raw: unknown): Json {
	if (typeof raw === "string") return { message: raw };
	return isJson(raw) ? raw : {};
}

/** A text error is the message. */
function replyError(body: Json): WireError {
	return typeof body["error"] === "string" && body["error"].length > 0
		? { message: body["error"] }
		: wireError(200, body);
}

/** Empty values are no error. */
function errored(raw: unknown): boolean {
	return !(raw === undefined || raw === null || raw === false || raw === 0 || raw === "");
}

function claude(model: string): boolean {
	return model.slice(model.lastIndexOf("/") + 1).startsWith("claude-");
}

/** Allowed tool ID characters. */
function cleanId(wireId: string): string {
	let id = "";
	for (let at = 0; at < wireId.length; at++) {
		const unit = wireId.charCodeAt(at);
		const plain =
			(unit >= 0x30 && unit <= 0x39) ||
			(unit >= 0x41 && unit <= 0x5a) ||
			(unit >= 0x61 && unit <= 0x7a) ||
			unit === 0x5f ||
			unit === 0x2d;
		id += plain ? wireId.charAt(at) : "_";
	}
	return id.length === 0 ? "call" : id;
}

/** A block id per wire id; ids needing no change keep theirs. */
function blockIds(history: readonly Turn[]): Map<string, string> {
	const wireIds = history.flatMap((turn) =>
		turn.role === "step" ? turn.parts.flatMap((part) => (part.kind === "call" ? [part.wireId] : [])) : [],
	);
	const mapped = new Map<string, string>();
	const taken = new Set<string>();
	for (const wireId of wireIds) {
		if (wireId.length > 0 && cleanId(wireId) === wireId && !mapped.has(wireId)) {
			mapped.set(wireId, wireId);
			taken.add(wireId);
		}
	}
	for (const wireId of wireIds) {
		if (mapped.has(wireId)) continue;
		const base = cleanId(wireId);
		let id = base;
		for (let count = 2; taken.has(id); count++) id = `${base}_${numberText(count)}`;
		mapped.set(wireId, id);
		taken.add(id);
	}
	return mapped;
}

function signed(part: ThoughtPart): boolean {
	return (part.type === THINKING || part.type === REDACTED) && part.opaque !== undefined;
}

/** Skip unreadable thoughts locally. */
function readable(step: StepTurn, request: CanonicalRequest): boolean {
	const thoughts = step.parts.filter((part): part is ThoughtPart => part.kind === "thought");
	const producer = step.producer?.model;
	if (producer === undefined || !thoughts.every(signed)) return false;
	return producer === request.model || (claude(producer) && claude(request.model));
}

function thoughtBlock(part: ThoughtPart): Block {
	const opaque = part.opaque ?? "";
	if (part.type === REDACTED) return { type: REDACTED, data: opaque };
	return {
		type: THINKING,
		thinking: (part.segments ?? []).map((segment) => segment.text).join(""),
		signature: opaque,
	};
}

/** User-turn tool results. */
function stepMessages(step: StepTurn, request: CanonicalRequest, blocks: ReadonlyMap<string, string>): Message[] {
	const sent = readable(step, request);
	const ids = new Map(step.parts.flatMap((part) => (part.kind === "call" ? [[part.index, part.wireId]] : [])));
	const content = step.parts.flatMap((part): Block[] => {
		if (part.kind === "thought") return sent ? [thoughtBlock(part)] : [];
		if (part.kind === "text") return part.text.length === 0 ? [] : [{ type: "text", text: part.text }];
		if (part.kind === "call")
			return [
				{
					type: "tool_use",
					id: blocks.get(part.wireId) ?? cleanId(part.wireId),
					name: part.name,
					input: argumentsOf(part.arguments),
				},
			];
		return [];
	});
	const results = step.parts.flatMap((part): Block[] => {
		const wireId = part.kind === "result" ? ids.get(part.index) : undefined;
		if (part.kind !== "result" || wireId === undefined) return [];
		return [
			{
				type: "tool_result",
				tool_use_id: blocks.get(wireId) ?? cleanId(wireId),
				content: part.output,
				...(part.isError ? { is_error: true as const } : {}),
			},
		];
	});
	return [
		...(content.length === 0 ? [] : [{ role: "assistant" as const, content }]),
		...(results.length === 0 ? [] : [{ role: "user" as const, content: results }]),
	];
}

function turnMessages(turn: Turn, request: CanonicalRequest, blocks: ReadonlyMap<string, string>): Message[] {
	return turn.role === "user"
		? [{ role: "user", content: [{ type: "text", text: turn.text }] }]
		: stepMessages(turn, request, blocks);
}

/** Join messages by role. */
function joined(messages: readonly Message[]): Message[] {
	return messages.reduce<Message[]>((out, message) => {
		const last = out.at(-1);
		if (last?.role === message.role)
			out[out.length - 1] = { ...last, content: [...last.content, ...message.content] };
		else out.push(message);
		return out;
	}, []);
}

/** Cache last user block. */
function withLastCached(messages: Message[], control: CacheControl): Message[] {
	const at = messages.findLastIndex((message) => message.role === "user");
	const message = messages[at];
	const last = message?.content.at(-1);
	if (message === undefined || (last?.type !== "text" && last?.type !== "tool_result")) return messages;
	const marked = [...messages];
	marked[at] = { ...message, content: [...message.content.slice(0, -1), { ...last, cache_control: control }] };
	return marked;
}

/** State follows results. */
function withState(messages: Message[], state: string | undefined): Message[] {
	if (state === undefined) return messages;
	return joined([...messages, { role: "user", content: [{ type: "text", text: state }] }]);
}

function toolOf(tool: ToolSpec): Tool {
	return { name: tool.name, description: tool.description, input_schema: tool.parameters };
}

/** Tool-choice mode mapping. */
function choiceOf(choice: readonly string[], name: string): MessagesRequest["tool_choice"] {
	if (choice.includes("tool")) return { type: "tool", name };
	return choice.includes("required") ? { type: "any" } : undefined;
}

/** Explicit prompt caching. */
function bodyOf(
	facts: CodecFacts | undefined,
	request: CanonicalRequest,
	mode: "stream" | "complete",
): MessagesRequest {
	const { tools, toolChoice, schema, effort } = request;
	const api = facts?.api;
	const enforced = schema !== undefined && api?.structured === true ? strictSchema(schema.json) : undefined;
	const prompted = schema !== undefined && enforced === undefined ? schemaPrompt(schema.json) : undefined;
	const control: CacheControl | undefined =
		request.cache.kind !== "explicit"
			? undefined
			: request.cache.ttl === "1h"
				? { type: "ephemeral", ttl: "1h" }
				: { type: "ephemeral" };
	const texts = systemTexts(request, prompted);
	const lastSystem = texts.findLastIndex((text, at) => at < request.system.length && text.length > 0);
	const marked = new Set([lastSystem, ...(request.summary === undefined ? [] : [request.system.length])]);
	const system = texts.flatMap((text, at) =>
		text.length === 0
			? []
			: [
					{
						type: "text" as const,
						text,
						...(control !== undefined && marked.has(at) ? { cache_control: control } : {}),
					},
				],
	);
	const blocks = blockIds(request.history);
	const history = joined(request.history.flatMap((turn) => turnMessages(turn, request, blocks)));
	const forced = toolChoice === undefined ? undefined : choiceOf(api?.choice ?? [], toolChoice.name);
	const efforts = facts?.efforts ?? [];
	const listed = effort !== undefined && efforts.includes(effort) ? effort : undefined;
	const output = {
		...(listed === undefined ? {} : { effort: listed }),
		...(enforced === undefined ? {} : { format: { type: "json_schema" as const, schema: enforced } }),
	};
	return {
		model: request.model,
		max_tokens: request.maxTokens ?? facts?.outputLimit ?? MAX_TOKENS,
		...(system.length === 0 ? {} : { system }),
		messages: withState(control === undefined ? history : withLastCached(history, control), request.state),
		...(tools === undefined || tools.length === 0 ? {} : { tools: tools.map(toolOf) }),
		...(forced === undefined ? {} : { tool_choice: forced }),
		// Claude 5 requires thinking.
		...(efforts.length > 0 && claude(request.model)
			? { thinking: { type: "adaptive", display: "summarized" } }
			: {}),
		...(Object.keys(output).length === 0 ? {} : { output_config: output }),
		stream: mode === "stream",
	};
}

/** Input includes cache. */
function usageOf(host: Host, raw: Json): NormalizedUsage | undefined {
	const cached = amountAt(raw, "cache_read_input_tokens");
	const cacheWrite = amountAt(raw, "cache_creation_input_tokens");
	return host.usage(
		{
			input: (amountAt(raw, "input_tokens") ?? 0) + (cached ?? 0) + (cacheWrite ?? 0),
			output: amountAt(raw, "output_tokens") ?? 0,
			...(cached === undefined ? {} : { cached }),
			...(cacheWrite === undefined ? {} : { cacheWrite }),
		},
		raw,
	);
}

function finishOf(reason: unknown): FinishReason | undefined {
	if (typeof reason !== "string" || reason.length === 0) return undefined;
	return FINISHES.get(reason) ?? "stop";
}

function thoughtOf(block: string, part: Omit<ThoughtPart, "kind">): StepEvent {
	return { kind: "thought", part: { kind: "thought", ...part }, block };
}

////////////////////////////////
//  Classes

/** Block-indexed message events. */
class Decoder {
	finished = false;
	private readonly calls = new Map<number, number>();
	private called = 0;
	private readonly serving: Serving = {};
	private raw: Json = {};
	private usage: NormalizedUsage | undefined;
	private finish: FinishReason | undefined;

	constructor(private readonly host: Host) {}

	*event(type: unknown, data: Json): Generator<StepEvent> {
		if (errored(data["error"])) throw this.failure(errorBody(data["error"]));
		const index = numberOf(data["index"]) ?? 0;
		switch (type) {
			case "message_start":
				if (isJson(data["message"])) yield* this.started(data["message"]);
				return;
			case "content_block_start":
				if (isJson(data["content_block"])) yield* this.block(index, data["content_block"]);
				return;
			case "content_block_delta":
				if (isJson(data["delta"])) yield* this.delta(index, data["delta"]);
				return;
			case "message_delta":
				yield* this.counted(data["usage"]);
				yield* this.stopped(isJson(data["delta"]) ? data["delta"]["stop_reason"] : undefined);
				return;
			case "message_stop":
				if (this.finish === undefined) throw failed("stream", "The provider ended the step.");
				this.finished = true;
				return;
			case "error":
				throw this.failure({});
		}
	}

	private failure(error: Json): Error {
		return streamFailure(
			this.host,
			{
				message: stringOf(error["message"]) || "The message failed.",
				...(typeof error["type"] === "string" ? { type: error["type"] } : {}),
			},
			this.usage,
			this.serving,
		);
	}

	/** A complete message. */
	*whole(message: Json): Generator<StepEvent> {
		yield* this.started({ id: message["id"] });
		const content = Array.isArray(message["content"]) ? message["content"] : [];
		for (const [index, block] of content.entries()) if (isJson(block)) yield* this.block(index, block, true);
		yield* this.counted(message["usage"]);
		yield* this.stopped(message["stop_reason"]);
	}

	private *started(message: Json): Generator<StepEvent> {
		for (const event of servingEvents(message["id"])) {
			if (event.kind === "serving" && event.serving.generationId !== undefined)
				this.serving.generationId = event.serving.generationId;
			yield event;
		}
		yield* this.counted(message["usage"]);
	}

	/** Merge cumulative usage. */
	private *counted(usage: unknown): Generator<StepEvent> {
		if (!isJson(usage)) return;
		this.raw = {
			...this.raw,
			...Object.fromEntries(
				Object.entries(usage).filter(
					([, value]) => isJson(value) || amountAt({ value }, "value") !== undefined,
				),
			),
		};
		this.usage = usageOf(this.host, this.raw) ?? this.usage;
		if (this.usage !== undefined) yield { kind: "usage", usage: this.usage };
	}

	private *stopped(reason: unknown): Generator<StepEvent> {
		const finish = finishOf(reason);
		if (finish === undefined) return;
		this.finish = finish;
		yield { kind: "finish", reason: finish };
	}

	/** Deltas carry streamed content. */
	private *block(index: number, block: Json, whole = false): Generator<StepEvent> {
		const id = numberText(index);
		switch (block["type"]) {
			case "text":
				yield { kind: "text", text: stringOf(block["text"]) };
				return;
			case THINKING: {
				if (!whole) return;
				const text = stringOf(block["thinking"]);
				const opaque = stringOf(block["signature"]);
				yield thoughtOf(id, {
					type: THINKING,
					...(text.length === 0 ? {} : { segments: [{ kind: "content", text }] }),
					...(opaque.length === 0 ? {} : { opaque }),
				});
				return;
			}
			case REDACTED:
				yield thoughtOf(id, { type: REDACTED, opaque: stringOf(block["data"]) });
				return;
			case "tool_use": {
				const call = this.called++;
				this.calls.set(index, call);
				yield {
					kind: "call",
					index: call,
					wireId: stringOf(block["id"]),
					name: stringOf(block["name"]),
					argumentsText: whole ? stringifyJson(isJson(block["input"]) ? block["input"] : {}) : "",
				};
				return;
			}
		}
	}

	private *delta(index: number, delta: Json): Generator<StepEvent> {
		const id = numberText(index);
		switch (delta["type"]) {
			case "text_delta":
				yield { kind: "text", text: stringOf(delta["text"]) };
				return;
			case "thinking_delta": {
				const text = stringOf(delta["thinking"]);
				if (text.length > 0) yield thoughtOf(id, { type: THINKING, segments: [{ kind: "content", text }] });
				return;
			}
			case "signature_delta":
				yield thoughtOf(id, { type: THINKING, opaque: stringOf(delta["signature"]) });
				return;
			case "input_json_delta": {
				const call = this.calls.get(index);
				if (call !== undefined)
					yield {
						kind: "call",
						index: call,
						wireId: "",
						name: "",
						argumentsText: stringOf(delta["partial_json"]),
					};
				return;
			}
		}
	}
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
	throw failed("truncated", "The stream ended before message_stop.");
}

function completed(host: Host, body: unknown): StepResult {
	if (!isJson(body)) throw failed("stream", "The reply was not JSON.");
	if (body["type"] === "error" || errored(body["error"])) throw streamFailure(host, replyError(body), undefined, {});
	return assembled(new Decoder(host).whole(body));
}

////////////////////////////////
//  Codec

export function messagesCodec(host: Host, facts: CodecFacts | undefined): Codec {
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
