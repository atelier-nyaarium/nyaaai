// OpenAI Responses wire types.

import type { Effort } from "@nyaaai/contract/shapes/step";

////////////////////////////////
//  Interfaces & Types

export interface ReasoningItem {
	type: "reasoning";
	id: string;
	summary: { type: "summary_text"; text: string }[];
	content?: { type: "reasoning_text"; text: string }[];
	encrypted_content: string;
}

export type InputItem =
	| { type: "message"; role: "user" | "developer"; content: string }
	| { type: "message"; role: "assistant"; content: string; phase: string }
	| ReasoningItem
	| { type: "function_call"; call_id: string; name: string; arguments: string }
	| { type: "function_call_output"; call_id: string; output: string };

export interface FunctionTool {
	type: "function";
	name: string;
	description: string;
	parameters: Record<string, unknown>;
}

/** Deliberately omits sampling, parallel calls and stored state. */
export interface ResponsesRequest {
	model: string;
	instructions?: string;
	input: InputItem[];
	tools?: FunctionTool[];
	tool_choice?: { type: "function"; name: string } | "required";
	reasoning?: { effort: Effort; summary?: "auto" };
	text?: { format: { type: "json_schema"; name: string; schema: Record<string, unknown>; strict: true } };
	store: false;
	include: ["reasoning.encrypted_content"];
	/** Counts reasoning tokens too. */
	max_output_tokens?: number;
	stream: boolean;
}
