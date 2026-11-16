import { expect, it } from "bun:test";
import { base64Decode } from "@nyaaai/contract/base64";
import { encodePathSegment, Utf8Decoder } from "@nyaaai/contract/text";
import {
	SEGMENT_TEXTS,
	UTF8_BOM_RUNS,
	UTF8_EMPTY_RUNS,
	UTF8_MALFORMED_RUNS,
	UTF8_SPLIT_RUNS,
} from "../caseSources/text";

const bytesOf = (chunk: string): Uint8Array => base64Decode(chunk) ?? new Uint8Array();

it("the chunk decoder reads every case run as the platform's does", () => {
	const wrong: number[] = [];
	const runs = [...UTF8_SPLIT_RUNS, ...UTF8_MALFORMED_RUNS, ...UTF8_BOM_RUNS, ...UTF8_EMPTY_RUNS];
	for (const [at, held] of runs.entries()) {
		const ours = new Utf8Decoder();
		const theirs = new TextDecoder("utf-8");
		const chunks = held.chunks.map(bytesOf);
		const pushed = chunks.map((chunk) => ours.push(chunk));
		const platform = chunks.map((chunk) => theirs.decode(chunk, { stream: true }));
		if (JSON.stringify([...pushed, ours.end()]) !== JSON.stringify([...platform, theirs.decode()])) wrong.push(at);
	}
	expect(wrong).toEqual([]);
});

it("path-segment encoding is encodeURIComponent on well-formed text", () => {
	for (const held of SEGMENT_TEXTS) {
		const formed = Array.from(held, (char) => {
			const unit = char.charCodeAt(0);
			return char.length === 1 && unit >= 0xd800 && unit <= 0xdfff ? "\u{fffd}" : char;
		}).join("");
		expect(encodePathSegment(held)).toBe(encodeURIComponent(formed));
	}
});
