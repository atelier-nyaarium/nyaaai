// Segment flattening for chat content.

import type { ThoughtPart } from "@nyaaai/contract/shapes/step";
import { trimText } from "@nyaaai/contract/text";
import { isJson } from "../native";

////////////////////////////////
//  Functions & Helpers

/** Responses can hold any JSON value. */
export function normalizeResponseValue(value: unknown): unknown {
	if (Array.isArray(value)) return value.map(normalizeResponseValue);
	if (!isJson(value)) return value;
	const content = value["content"];
	return Array.isArray(content) ? extractTextFromSegments(content) : value;
}

/** Streaming segments can hold any entry. */
export function extractTextFromSegments(segments: unknown[]): string | undefined {
	const parts: string[] = [];
	for (const segment of segments) {
		if (typeof segment === "string") {
			if (trimText(segment).length > 0) parts.push(segment);
		} else if (isJson(segment)) {
			const text = segment["text"];
			if (typeof text === "string" && trimText(text).length > 0) {
				parts.push(text);
				continue;
			}
			const nested = segment["content"];
			if (!Array.isArray(nested)) continue;
			const nestedText = extractTextFromSegments(nested);
			if (typeof nestedText === "string" && trimText(nestedText).length > 0) parts.push(nestedText);
		}
	}
	return parts.length === 0 ? undefined : parts.join(" ");
}

export function textOf(content: unknown): string {
	if (typeof content === "string") return content;
	if (Array.isArray(content)) return extractTextFromSegments(content) ?? "";
	return "";
}

/** The visible text of a thought, in order. */
export function thoughtText(part: ThoughtPart): string {
	return (part.segments ?? []).map((segment) => segment.text).join("");
}
