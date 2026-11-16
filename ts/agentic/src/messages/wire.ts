// Anthropic Messages wire types.

import type { Effort } from "@nyaaai/contract/shapes/step";

////////////////////////////////
//  Interfaces & Types

export interface CacheControl {
	type: "ephemeral";
	ttl?: "1h";
}

export type Block =
	| { type: "text"; text: string; cache_control?: CacheControl }
	| { type: "thinking"; thinking: string; signature: string }
	| { type: "redacted_thinking"; data: string }
	| { type: "tool_use"; id: string; name: string; input: Record<string, unknown> }
	| { type: "tool_result"; tool_use_id: string; content: string; is_error?: true; cache_control?: CacheControl };

export interface Message {
	role: "user" | "assistant";
	content: Block[];
}

export interface Tool {
	name: string;
	description: string;
	input_schema: Record<string, unknown>;
}

/** Deliberately omits sampling and stop sequences. */
export interface MessagesRequest {
	model: string;
	/** Counts thinking tokens too. */
	max_tokens: number;
	system?: Extract<Block, { type: "text" }>[];
	messages: Message[];
	tools?: Tool[];
	tool_choice?: { type: "any" } | { type: "tool"; name: string };
	thinking?: { type: "adaptive"; display: "summarized" };
	output_config?: { effort?: Effort; format?: { type: "json_schema"; schema: Record<string, unknown> } };
	stream: boolean;
}
