// Chat Completions wire types and readers.

import type { Effort } from "@nyaaai/contract/shapes/step";
import type { WireError } from "../failure";
import { isJson, type Json, numberOf } from "../native";

////////////////////////////////
//  Interfaces & Types

export interface TextPart {
	type: "text";
	text: string;
	cache_control?: { type: "ephemeral"; ttl?: "1h" };
}

/** OpenRouter's thought blocks. */
export interface ReasoningDetail {
	type: string;
	id?: string | null;
	format?: string;
	index?: number;
	text?: string;
	summary?: string;
	data?: string;
	signature?: string;
}

export interface WireCall {
	id: string;
	type: "function";
	function: { name: string; arguments: string };
}

export type AssistantMessage = {
	role: "assistant";
	content: string | null;
	tool_calls?: WireCall[];
	reasoning_details?: ReasoningDetail[];
	reasoning_content?: string;
	/** Ollama's thought text. */
	reasoning?: string;
};

export type ChatMessage =
	| { role: "system" | "user"; content: string | TextPart[] }
	| AssistantMessage
	| { role: "tool"; tool_call_id: string; content: string | TextPart[] };

export interface ChatTool {
	type: "function";
	function: { name: string; description: string; parameters: Record<string, unknown>; strict?: boolean };
}

export interface JsonSchemaFormat {
	type: "json_schema";
	json_schema: { name: string; strict: true; schema: Record<string, unknown> };
}

/** OpenRouter routing; `only` pins verification hosts, `preferred_max_latency` is seconds. */
export interface Provider {
	/** Zero-retention hosts only. */
	zdr?: true;
	data_collection?: "deny";
	require_parameters: true;
	only?: readonly string[];
	order?: readonly string[];
	preferred_max_latency?: number;
}

/** Deliberately omits sampling, parallel calls and transforms. */
export interface ChatRequest {
	model: string;
	messages: ChatMessage[];
	stream: boolean;
	session_id?: string;
	provider?: Provider;
	tools?: ChatTool[];
	tool_choice?: { type: "function"; function: { name: string } } | "required";
	reasoning?: { effort: Effort };
	reasoning_effort?: Effort;
	response_format?: JsonSchemaFormat;
	/** Counts reasoning tokens too. */
	max_tokens?: number;
	stream_options?: { include_usage: true };
}

export interface Usage {
	prompt_tokens: number;
	completion_tokens: number;
	total_tokens?: number;
	cost?: number;
	is_byok?: boolean;
	cost_details?: { upstream_inference_cost?: number | null };
	prompt_tokens_details?: { cached_tokens?: number; cache_write_tokens?: number };
	completion_tokens_details?: { reasoning_tokens?: number };
	/** OpenCode's billing. */
	opencode?: {
		billing_source?: string;
		list_cost?: number;
		allowance?: number;
		tokens?: Partial<
			Record<"input" | "output" | "reasoning" | "cache_read" | "cache_write_5m" | "cache_write_1h", number>
		>;
	};
}

export interface ToolCallDelta {
	/** Absent when not a number. */
	index?: number;
	id?: string;
	function?: { name?: string; arguments?: string };
}

/** A streamed delta or a whole reply message. */
export interface Delta {
	role?: string;
	content?: unknown;
	reasoning?: string;
	reasoning_details?: ReasoningDetail[];
	reasoning_content?: string;
}

export interface StreamedDelta extends Delta {
	tool_calls?: ToolCallDelta[];
}

export interface ReplyMessage extends Delta {
	tool_calls?: WireCall[];
}

export interface Chunk {
	id?: string;
	provider?: string;
	choices?: Array<{ delta?: StreamedDelta; finish_reason?: string }>;
	usage?: Usage;
	error?: WireError;
}

export interface Completion {
	id?: string;
	provider?: string;
	choices?: Array<{ message?: ReplyMessage; finish_reason?: string }>;
	usage?: Usage;
	error?: WireError;
}

////////////////////////////////
//  Functions & Helpers

function stringAt(raw: Json, key: string): string | undefined {
	const value = raw[key];
	return typeof value === "string" ? value : undefined;
}

/** A count or amount; negative is mistyped. */
function amountAt(raw: Json, key: string): number | undefined {
	const value = numberOf(raw[key]);
	return value === undefined || value < 0 ? undefined : value + 0;
}

function numberAt(raw: Json, key: string): number | undefined {
	return numberOf(raw[key]);
}

function objectAt(raw: Json, key: string): Json | undefined {
	const value = raw[key];
	return isJson(value) ? value : undefined;
}

/** Empty values are no error; a text is the message. */
function readReplyError(raw: unknown): WireError | undefined {
	if (raw === undefined || raw === null || raw === false || raw === 0 || raw === "") return undefined;
	return isJson(raw) || typeof raw === "string" ? readError(raw) : {};
}

/** A text is the message. */
export function readError(raw: unknown): WireError | undefined {
	if (typeof raw === "string") return { message: raw };
	if (!isJson(raw)) return undefined;
	const held = raw["code"];
	const code = typeof held === "string" ? held : numberOf(held);
	const type = stringAt(raw, "type");
	const message = stringAt(raw, "message");
	const metadata = objectAt(raw, "metadata");
	const resetsAt = raw["resetsAt"];
	return {
		...(code === undefined ? {} : { code }),
		...(type === undefined ? {} : { type }),
		...(message === undefined ? {} : { message }),
		...(metadata === undefined ? {} : { metadata }),
		...(typeof resetsAt === "string" || typeof resetsAt === "number" ? { resetsAt } : {}),
	};
}

/** Mistyped counts become zero. */
export function readUsage(raw: unknown): Usage | undefined {
	if (!isJson(raw)) return undefined;
	const total = amountAt(raw, "total_tokens");
	const cost = amountAt(raw, "cost");
	const byok = raw["is_byok"];
	const costs = objectAt(raw, "cost_details");
	const prompt = objectAt(raw, "prompt_tokens_details");
	const completion = objectAt(raw, "completion_tokens_details");
	const block = objectAt(raw, "opencode");
	const upstream = costs === undefined ? undefined : costs["upstream_inference_cost"];
	const upstreamCost = upstream === null ? null : amountAt({ upstream }, "upstream");
	const cached = prompt === undefined ? undefined : amountAt(prompt, "cached_tokens");
	const written = prompt === undefined ? undefined : amountAt(prompt, "cache_write_tokens");
	const reasoning = completion === undefined ? undefined : amountAt(completion, "reasoning_tokens");
	return {
		prompt_tokens: amountAt(raw, "prompt_tokens") ?? 0,
		completion_tokens: amountAt(raw, "completion_tokens") ?? 0,
		...(total === undefined ? {} : { total_tokens: total }),
		...(cost === undefined ? {} : { cost }),
		...(typeof byok === "boolean" ? { is_byok: byok } : {}),
		...(costs === undefined
			? {}
			: {
					cost_details: upstreamCost === undefined ? {} : { upstream_inference_cost: upstreamCost },
				}),
		...(prompt === undefined
			? {}
			: {
					prompt_tokens_details: {
						...(cached === undefined ? {} : { cached_tokens: cached }),
						...(written === undefined ? {} : { cache_write_tokens: written }),
					},
				}),
		...(completion === undefined
			? {}
			: { completion_tokens_details: reasoning === undefined ? {} : { reasoning_tokens: reasoning } }),
		...(block === undefined ? {} : { opencode: readBilling(block) }),
	};
}

function readBilling(block: Json): NonNullable<Usage["opencode"]> {
	const source = stringAt(block, "billing_source");
	const list = amountAt(block, "list_cost");
	const allowance = amountAt(block, "allowance");
	const tokens = objectAt(block, "tokens");
	const counts = (
		["input", "output", "reasoning", "cache_read", "cache_write_5m", "cache_write_1h"] as const
	).flatMap((key) => {
		const count = tokens === undefined ? undefined : amountAt(tokens, key);
		return count === undefined ? [] : [[key, count] as const];
	});
	return {
		...(source === undefined ? {} : { billing_source: source }),
		...(list === undefined ? {} : { list_cost: list }),
		...(allowance === undefined ? {} : { allowance }),
		...(tokens === undefined ? {} : { tokens: Object.fromEntries(counts) }),
	};
}

function readDetail(raw: unknown): ReasoningDetail[] {
	if (!isJson(raw)) return [];
	const type = stringAt(raw, "type");
	if (type === undefined) return [];
	const id = raw["id"];
	const format = stringAt(raw, "format");
	const index = numberAt(raw, "index");
	const text = stringAt(raw, "text");
	const summary = stringAt(raw, "summary");
	const data = stringAt(raw, "data");
	const signature = stringAt(raw, "signature");
	return [
		{
			type,
			...(typeof id === "string" || id === null ? { id } : {}),
			...(format === undefined ? {} : { format }),
			...(index === undefined ? {} : { index }),
			...(text === undefined ? {} : { text }),
			...(summary === undefined ? {} : { summary }),
			...(data === undefined ? {} : { data }),
			...(signature === undefined ? {} : { signature }),
		},
	];
}

function readDelta(raw: unknown): Delta | undefined {
	if (!isJson(raw)) return undefined;
	const role = stringAt(raw, "role");
	const reasoning = stringAt(raw, "reasoning");
	const reasoningContent = stringAt(raw, "reasoning_content");
	const details = raw["reasoning_details"];
	return {
		...(role === undefined ? {} : { role }),
		...("content" in raw ? { content: raw["content"] } : {}),
		...(reasoning === undefined ? {} : { reasoning }),
		...(Array.isArray(details) ? { reasoning_details: details.flatMap(readDetail) } : {}),
		...(reasoningContent === undefined ? {} : { reasoning_content: reasoningContent }),
	};
}

function readCallDelta(raw: unknown): ToolCallDelta[] {
	if (!isJson(raw)) return [];
	const id = stringAt(raw, "id");
	const held = objectAt(raw, "function");
	const name = held === undefined ? undefined : stringAt(held, "name");
	const args = held === undefined ? undefined : stringAt(held, "arguments");
	const index = numberAt(raw, "index");
	return [
		{
			...(index === undefined ? {} : { index }),
			...(id === undefined ? {} : { id }),
			...(held === undefined
				? {}
				: {
						function: {
							...(name === undefined ? {} : { name }),
							...(args === undefined ? {} : { arguments: args }),
						},
					}),
		},
	];
}

function readWireCall(raw: unknown): WireCall[] {
	if (!isJson(raw)) return [];
	const held = objectAt(raw, "function") ?? {};
	return [
		{
			id: stringAt(raw, "id") ?? "",
			type: "function",
			function: { name: stringAt(held, "name") ?? "", arguments: stringAt(held, "arguments") ?? "" },
		},
	];
}

/** A mistyped reason is an error. */
function readFinish(raw: unknown): string | undefined {
	if (typeof raw === "string") return raw;
	return raw === undefined || raw === null ? undefined : "error";
}

function readServing(raw: Json): { id?: string; provider?: string } {
	const id = stringAt(raw, "id");
	const provider = stringAt(raw, "provider");
	return { ...(id === undefined ? {} : { id }), ...(provider === undefined ? {} : { provider }) };
}

/** One streamed chunk. */
export function readChunk(raw: Json): Chunk {
	const choices = raw["choices"];
	const usage = readUsage(raw["usage"]);
	const error = readReplyError(raw["error"]);
	return {
		...readServing(raw),
		...(Array.isArray(choices)
			? {
					choices: choices.flatMap((each): NonNullable<Chunk["choices"]> => {
						if (!isJson(each)) return [];
						const delta = readDelta(each["delta"]);
						const calls = objectAt(each, "delta")?.["tool_calls"];
						const finish = readFinish(each["finish_reason"]);
						return [
							{
								...(delta === undefined
									? {}
									: {
											delta: {
												...delta,
												...(Array.isArray(calls)
													? { tool_calls: calls.flatMap(readCallDelta) }
													: {}),
											},
										}),
								...(finish === undefined ? {} : { finish_reason: finish }),
							},
						];
					}),
				}
			: {}),
		...(usage === undefined ? {} : { usage }),
		...(error === undefined ? {} : { error }),
	};
}

/** One whole reply. */
export function readCompletion(raw: Json): Completion {
	const choices = raw["choices"];
	const usage = readUsage(raw["usage"]);
	const error = readReplyError(raw["error"]);
	return {
		...readServing(raw),
		...(Array.isArray(choices)
			? {
					choices: choices.flatMap((each): NonNullable<Completion["choices"]> => {
						if (!isJson(each)) return [];
						const message = readDelta(each["message"]);
						const calls = objectAt(each, "message")?.["tool_calls"];
						const finish = readFinish(each["finish_reason"]);
						return [
							{
								...(message === undefined
									? {}
									: {
											message: {
												...message,
												...(Array.isArray(calls)
													? { tool_calls: calls.flatMap(readWireCall) }
													: {}),
											},
										}),
								...(finish === undefined ? {} : { finish_reason: finish }),
							},
						];
					}),
				}
			: {}),
		...(usage === undefined ? {} : { usage }),
		...(error === undefined ? {} : { error }),
	};
}
