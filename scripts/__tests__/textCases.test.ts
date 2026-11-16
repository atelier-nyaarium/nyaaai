import { expect, it } from "bun:test";
import { base64Decode } from "@nyaaai/contract/base64";
import { encodePathSegment, Utf8Decoder } from "@nyaaai/contract/text";
import {
	SEGMENT_TEXTS,
	UTF8_BOM_RUNS,
	UTF8_EMPTY_RUNS,
	UTF8_MALFORMED_RUNS,
	UTF8_REUSED_RUNS,
	UTF8_SPLIT_RUNS,
} from "../caseSources/text";

const bytesOf = (chunk: string): Uint8Array => base64Decode(chunk) ?? new Uint8Array();

/** Lone surrogates, which a case cannot carry. */
const LONE_SURROGATE_TEXTS = ["\u{D800}", "\u{DC00}", "a\u{D800}b", "\u{DC00}\u{1F600}\u{D83D}"];

const wellFormed = (text: string): string =>
	Array.from(text, (char) => {
		const unit = char.charCodeAt(0);
		return char.length === 1 && unit >= 0xd800 && unit <= 0xdfff ? "\u{fffd}" : char;
	}).join("");

it("the chunk decoder reads every case run as the platform's does", () => {
	const wrong: number[] = [];
	const runs = [
		...UTF8_SPLIT_RUNS,
		...UTF8_MALFORMED_RUNS,
		...UTF8_BOM_RUNS,
		...UTF8_EMPTY_RUNS,
		...UTF8_REUSED_RUNS,
	];
	for (const [at, held] of runs.entries()) {
		const ours = new Utf8Decoder();
		const theirs = new TextDecoder("utf-8");
		const passes: string[][] = [held.chunks, (held as { again?: string[] }).again ?? []];
		for (const chunks of passes) {
			const bytes = chunks.map(bytesOf);
			const pushed = bytes.map((chunk) => ours.push(chunk));
			const platform = bytes.map((chunk) => theirs.decode(chunk, { stream: true }));
			if (JSON.stringify([...pushed, ours.end()]) !== JSON.stringify([...platform, theirs.decode()]))
				wrong.push(at);
		}
	}
	expect(wrong).toEqual([]);
});

it("path-segment encoding is encodeURIComponent on well-formed text", () => {
	for (const held of [...SEGMENT_TEXTS, ...LONE_SURROGATE_TEXTS]) {
		expect(encodePathSegment(held)).toBe(encodeURIComponent(wellFormed(held)));
	}
});
