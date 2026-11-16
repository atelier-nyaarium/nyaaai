// Google generateContent codec.

import { stringifyJson } from "@nyaaai/contract/json";
import type {
	CanonicalRequest,
	CodecFacts,
	Effort,
	FinishReason,
	NormalizedUsage,
	Serving,
	SseItem,
	StepEvent,
	StepResult,
	ThoughtPart,
	ToolSpec,
} from "@nyaaai/contract/shapes/step";
import { encodePathSegment } from "@nyaaai/contract/text";
import type { Codec, StepTurn } from "../codec";
import { failed } from "../failure";
import {
	argumentsOf,
	assembled,
	currentTurnFrom,
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
import type { Content, FunctionDeclaration, GenerateContentRequest, Part, ThinkingLevel } from "./wire";

////////////////////////////////
//  Constants

const THOUGHT = "thought";

/** Text signature anchor. */
const ON_TEXT = "text";

/** Sentinel bypasses signature validation. */
export const SKIP_SIGNATURE = "skip_thought_signature_validator";

const LEVELS: Partial<Record<Effort, ThinkingLevel>> = {
	minimal: "MINIMAL",
	low: "LOW",
	medium: "MEDIUM",
	high: "HIGH",
};

const FILTERED: ReadonlySet<string> = new Set([
	"SAFETY",
	"RECITATION",
	"LANGUAGE",
	"BLOCKLIST",
	"PROHIBITED_CONTENT",
	"SPII",
	"IMAGE_SAFETY",
	"IMAGE_PROHIBITED_CONTENT",
	"IMAGE_RECITATION",
]);

////////////////////////////////
//  Functions & Helpers

/** Empty values are no error. */
function errored(raw: unknown): boolean {
	return !(raw === undefined || raw === null || raw === false || raw === 0 || raw === "");
}

/** A text is the message. */
function errorBody(raw: unknown): Json {
	if (typeof raw === "string") return { message: raw };
	return isJson(raw) ? raw : {};
}

function onCall(index: number): string {
	return `call:${index}`;
}

/** Replay only to producer. */
function readable(step: StepTurn, request: CanonicalRequest): boolean {
	return (
		step.producer?.model === request.model &&
		step.parts.every((part) => part.kind !== "thought" || part.type === THOUGHT)
	);
}

function thoughtPart(part: ThoughtPart): Part[] {
	if (part.anchor === ON_TEXT) return part.opaque === undefined ? [] : [{ text: "", thoughtSignature: part.opaque }];
	if (part.anchor !== undefined) return [];
	const text = (part.segments ?? []).map((segment) => segment.text).join("");
	return [{ text, thought: true, ...(part.opaque === undefined ? {} : { thoughtSignature: part.opaque }) }];
}

/** First unsigned current call gets sentinel. */
function stepContents(step: StepTurn, request: CanonicalRequest, current: boolean): Content[] {
	const sent = readable(step, request);
	const signatures = new Map(
		step.parts.flatMap((part) =>
			sent && part.kind === "thought" && part.anchor !== undefined && part.opaque !== undefined
				? [[part.anchor, part.opaque]]
				: [],
		),
	);
	const names = new Map(step.parts.flatMap((part) => (part.kind === "call" ? [[part.index, part.name]] : [])));
	const parts = step.parts.flatMap((part): Part[] => {
		if (part.kind === "thought") return sent ? thoughtPart(part) : [];
		if (part.kind === "text") return part.text.length === 0 ? [] : [{ text: part.text }];
		if (part.kind !== "call") return [];
		const signature = signatures.get(onCall(part.index));
		return [
			{
				functionCall: { name: part.name, args: argumentsOf(part.arguments) },
				...(signature === undefined ? {} : { thoughtSignature: signature }),
			},
		];
	});
	const first = parts.findIndex((part) => "functionCall" in part);
	const call = parts[first];
	if (current && call !== undefined && "functionCall" in call && call.thoughtSignature === undefined)
		parts[first] = { ...call, thoughtSignature: SKIP_SIGNATURE };
	const responses = step.parts.flatMap((part): Part[] => {
		const name = part.kind === "result" ? names.get(part.index) : undefined;
		if (part.kind !== "result" || name === undefined) return [];
		return [
			{ functionResponse: { name, response: part.isError ? { error: part.output } : { output: part.output } } },
		];
	});
	return [
		...(parts.length === 0 ? [] : [{ role: "model" as const, parts }]),
		...(responses.length === 0 ? [] : [{ role: "user" as const, parts: responses }]),
	];
}

function contentsOf(request: CanonicalRequest): Content[] {
	const current = currentTurnFrom(request.history);
	return [
		...request.history.flatMap((turn, at): Content[] =>
			turn.role === "user"
				? [{ role: "user", parts: [{ text: turn.text }] }]
				: stepContents(turn, request, at >= current),
		),
		...(request.state === undefined ? [] : [{ role: "user" as const, parts: [{ text: request.state }] }]),
	];
}

function declarationOf(tool: ToolSpec): FunctionDeclaration {
	return { name: tool.name, description: tool.description, parametersJsonSchema: tool.parameters };
}

/** Named or any-tool choice. */
function choiceOf(choice: readonly string[], name: string): GenerateContentRequest["toolConfig"] {
	if (choice.includes("tool")) return { functionCallingConfig: { mode: "ANY", allowedFunctionNames: [name] } };
	return choice.includes("required") ? { functionCallingConfig: { mode: "ANY" } } : undefined;
}

/** Catalog gates optional fields. */
function bodyOf(facts: CodecFacts | undefined, request: CanonicalRequest): GenerateContentRequest {
	const { tools, toolChoice, schema, effort, maxTokens } = request;
	const api = facts?.api;
	const efforts = facts?.efforts ?? [];
	const enforced = schema !== undefined && api?.structured === true ? strictSchema(schema.json) : undefined;
	const prompted = schema !== undefined && enforced === undefined ? schemaPrompt(schema.json) : undefined;
	const system = systemTexts(request, prompted).filter((text) => text.length > 0);
	const forced = toolChoice === undefined ? undefined : choiceOf(api?.choice ?? [], toolChoice.name);
	const level = effort !== undefined && efforts.includes(effort) ? LEVELS[effort] : undefined;
	const generation = {
		...(enforced === undefined
			? {}
			: { responseMimeType: "application/json" as const, responseJsonSchema: enforced }),
		...(efforts.length === 0
			? {}
			: {
					thinkingConfig: {
						includeThoughts: true as const,
						...(level === undefined ? {} : { thinkingLevel: level }),
					},
				}),
		...(maxTokens === undefined ? {} : { maxOutputTokens: maxTokens }),
	};
	return {
		contents: contentsOf(request),
		...(system.length === 0 ? {} : { systemInstruction: { parts: system.map((text) => ({ text })) } }),
		...(tools === undefined || tools.length === 0
			? {}
			: { tools: [{ functionDeclarations: tools.map(declarationOf) }] }),
		...(forced === undefined ? {} : { toolConfig: forced }),
		...(Object.keys(generation).length === 0 ? {} : { generationConfig: generation }),
	};
}

/** Output includes thoughts. */
function usageOf(host: Host, raw: unknown): NormalizedUsage | undefined {
	if (!isJson(raw)) return undefined;
	const prompt = numberOf(raw["promptTokenCount"]);
	if (prompt === undefined) return undefined;
	const reasoning = numberOf(raw["thoughtsTokenCount"]);
	const cached = numberOf(raw["cachedContentTokenCount"]);
	return host.usage(
		{
			input: prompt + (numberOf(raw["toolUsePromptTokenCount"]) ?? 0),
			output: (numberOf(raw["candidatesTokenCount"]) ?? 0) + (reasoning ?? 0),
			...(cached === undefined ? {} : { cached }),
			...(reasoning === undefined ? {} : { reasoning }),
		},
		raw,
	);
}

function finishOf(reason: string, called: boolean): FinishReason {
	if (reason === "STOP") return called ? "tool_calls" : "stop";
	if (reason === "MAX_TOKENS" || reason === "CONTINUATION") return "length";
	return FILTERED.has(reason) ? "content_filter" : "error";
}

function thoughtOf(part: Omit<ThoughtPart, "kind">): StepEvent {
	return { kind: "thought", part: { kind: "thought", ...part } };
}

////////////////////////////////
//  Classes

/** Calls arrive whole. */
class Decoder {
	finished = false;
	private calls = 0;
	private readonly serving: Serving = {};
	private usage: NormalizedUsage | undefined;

	constructor(private readonly host: Host) {}

	*chunk(data: Json): Generator<StepEvent> {
		this.usage = usageOf(this.host, data["usageMetadata"]) ?? this.usage;
		const usage = isJson(data["usageMetadata"]) ? this.usage : undefined;
		if (this.finished) {
			if (usage !== undefined) yield { kind: "usage", usage };
			return;
		}
		if (this.serving.generationId === undefined)
			for (const event of servingEvents(data["responseId"])) {
				if (event.kind === "serving" && event.serving.generationId !== undefined)
					this.serving.generationId = event.serving.generationId;
				yield event;
			}
		if (errored(data["error"])) {
			const error = errorBody(data["error"]);
			const code = numberOf(error["code"]);
			throw streamFailure(
				this.host,
				{
					message: stringOf(error["message"]) || "The response failed.",
					...(code === undefined ? {} : { code }),
					...(typeof error["status"] === "string" ? { type: error["status"] } : {}),
				},
				this.usage,
				this.serving,
			);
		}
		const candidate =
			Array.isArray(data["candidates"]) && isJson(data["candidates"][0]) ? data["candidates"][0] : undefined;
		const feedback = isJson(data["promptFeedback"]) ? data["promptFeedback"] : {};
		if (candidate === undefined && typeof feedback["blockReason"] === "string")
			throw failed("refused", `The prompt was blocked: ${feedback["blockReason"]}.`, { status: "stream" });
		const content = isJson(candidate?.["content"]) ? candidate["content"] : {};
		for (const part of Array.isArray(content["parts"]) ? content["parts"] : [])
			if (isJson(part)) yield* this.part(part);
		if (usage !== undefined) yield { kind: "usage", usage };
		const reason = stringOf(candidate?.["finishReason"]);
		if (reason.length === 0) return;
		if (reason === "MISSING_THOUGHT_SIGNATURE")
			throw failed("replayRejected", "A thought signature was missing.", { status: "stream" });
		this.finished = true;
		yield { kind: "finish", reason: finishOf(reason, this.calls > 0) };
	}

	/** Call/text signature anchors. */
	private *part(part: Json): Generator<StepEvent> {
		const signature = stringOf(part["thoughtSignature"]);
		if (isJson(part["functionCall"])) {
			const call = part["functionCall"];
			const index = this.calls++;
			if (signature.length > 0) yield thoughtOf({ type: THOUGHT, opaque: signature, anchor: onCall(index) });
			yield {
				kind: "call",
				index,
				wireId: stringOf(call["id"]),
				name: stringOf(call["name"]),
				argumentsText: stringifyJson(isJson(call["args"]) ? call["args"] : {}),
			};
			return;
		}
		if (typeof part["text"] !== "string") return;
		const text = part["text"];
		if (part["thought"] === true) {
			if (text.length > 0 || signature.length > 0)
				yield thoughtOf({
					type: THOUGHT,
					...(text.length === 0 ? {} : { segments: [{ kind: "content", text }] }),
					...(signature.length === 0 ? {} : { opaque: signature }),
				});
			return;
		}
		if (text.length > 0) yield { kind: "text", text };
		if (signature.length > 0) yield thoughtOf({ type: THOUGHT, opaque: signature, anchor: ON_TEXT });
	}
}

/** Finish arrives last. */
async function* streamed(host: Host, items: AsyncIterable<SseItem>): AsyncGenerator<StepEvent> {
	const decoder = new Decoder(host);
	for await (const item of items) {
		if (item.kind === "comment") continue;
		if (item.kind === "done") break;
		yield* decoder.chunk(parsedChunk(item.data));
	}
	if (!decoder.finished) throw failed("truncated", "The stream ended before a finish reason.");
}

function completed(host: Host, body: unknown): StepResult {
	if (!isJson(body)) throw failed("stream", "The reply was not JSON.");
	return assembled(new Decoder(host).chunk(body));
}

////////////////////////////////
//  Codec

export function geminiCodec(host: Host, facts: CodecFacts | undefined): Codec {
	return {
		encode: (request, mode) => {
			const model = encodePathSegment(request.model);
			return {
				path: `${host.prefix}/models/${model}:${mode === "stream" ? "streamGenerateContent?alt=sse" : "generateContent"}`,
				body: bodyOf(facts, request),
				headers: host.headers(request),
			};
		},
		stream: (items) => streamed(host, items),
		complete: (body) => completed(host, body),
		failure: (status, body, headers) => host.failure(status, wireError(status, body), headers),
	};
}
