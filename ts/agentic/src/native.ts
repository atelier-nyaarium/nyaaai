// Helpers shared by the native API codecs.

import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import type {
	CanonicalRequest,
	Failure,
	HeaderList,
	NormalizedUsage,
	Serving,
	StepEvent,
	StepResult,
	Turn,
} from "@nyaaai/contract/shapes/step";
import { failed, messageOf, ProviderError, type WireError } from "./failure";
import { StepAssembler } from "./step";

////////////////////////////////
//  Interfaces & Types

/** Gateway-specific API behavior. */
export interface Host {
	/** API path prefix. */
	prefix: string;
	headers(request: CanonicalRequest): Record<string, string>;
	/** Billing and usage adjustments. */
	usage(own: NormalizedUsage, raw: Record<string, unknown>): NormalizedUsage;
	/** HTTP 200 stream errors. */
	failure(status: number | "stream", error: WireError, headers?: HeaderList): Failure;
}

export type Json = Record<string, unknown>;

////////////////////////////////
//  Functions & Helpers

export function isJson(value: unknown): value is Json {
	return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function stringOf(value: unknown): string {
	return typeof value === "string" ? value : "";
}

export function numberOf(value: unknown): number | undefined {
	return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

export function schemaPrompt(json: Record<string, unknown>): string {
	return `Answer with one JSON object matching this JSON Schema, and nothing else:\n${stringifyJson(json)}`;
}

/** System, summary, prompt schema. */
export function systemTexts(request: CanonicalRequest, prompted: string | undefined): string[] {
	return [
		...request.system,
		...(request.summary === undefined ? [] : [request.summary]),
		...(prompted === undefined ? [] : [prompted]),
	];
}

/** Non-object arguments become `{}`. */
export function argumentsOf(argumentsText: string): Json {
	const parsed = parseJson(argumentsText);
	return isJson(parsed) ? parsed : {};
}

/** Index after latest user. */
export function currentTurnFrom(history: readonly Turn[]): number {
	return history.findLastIndex((turn) => turn.role === "user") + 1;
}

export function parsedChunk(data: string): Json {
	const chunk = parseJson(data);
	if (!isJson(chunk)) throw failed("stream", "A stream chunk was not JSON.");
	return chunk;
}

/** Native errors wrap `error`. */
export function wireError(status: number, body: unknown): WireError {
	const error = isJson(body) && isJson(body["error"]) ? body["error"] : {};
	const type =
		typeof error["type"] === "string"
			? error["type"]
			: typeof error["status"] === "string"
				? error["status"]
				: undefined;
	const message = messageOf(error["message"]) ?? `HTTP ${status}`;
	const resetsAt =
		typeof error["resetsAt"] === "string" || typeof error["resetsAt"] === "number" ? error["resetsAt"] : undefined;
	return {
		message,
		...(type === undefined ? {} : { type }),
		...(resetsAt === undefined ? {} : { resetsAt }),
	};
}

/** Stream failure with billing. */
export function streamFailure(
	host: Host,
	error: WireError,
	usage: NormalizedUsage | undefined,
	serving: Serving,
): ProviderError {
	const failure = host.failure("stream", error);
	if (usage === undefined && serving.generationId === undefined && serving.host === undefined)
		return new ProviderError(failure);
	return new ProviderError({ ...failure, billed: { ...(usage === undefined ? {} : { usage }), ...serving } });
}

/** Assemble whole-reply events. */
export function assembled(events: Iterable<StepEvent>): StepResult {
	const assembler = new StepAssembler();
	for (const event of events) assembler.add(event);
	const result = assembler.result();
	if (result.finish === "") throw failed("stream", "The provider ended the step.");
	return result;
}

export function servingEvents(generationId: unknown): StepEvent[] {
	return typeof generationId === "string" && generationId.length > 0
		? [{ kind: "serving", serving: { generationId } }]
		: [];
}
