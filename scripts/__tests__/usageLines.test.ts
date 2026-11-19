import { expect, it } from "bun:test";
import { stringifyJson } from "@nyaaai/contract/json";
import { decodeText } from "@nyaaai/contract/shape";
import { UsageLine } from "@nyaaai/contract/shapes/usage";
import { CALL_LINES, STATS_LINES, SURROGATE_LINE } from "../caseSources/usageShapes";

it("a day file's lines decode and encode to the same bytes", () => {
	const changed: string[] = [];
	for (const line of [...CALL_LINES, ...STATS_LINES]) {
		const decoded = decodeText(UsageLine, line);
		if (!decoded.ok || stringifyJson(decoded.value) !== line) changed.push(line);
	}
	expect(changed).toEqual([]);
});

it("a stats line is not a call line", () => {
	const kinds = [...CALL_LINES, ...STATS_LINES].map((line) => {
		const decoded = decodeText(UsageLine, line);
		return decoded.ok ? ("call" in decoded.value ? "call" : "stats") : "refused";
	});
	expect(kinds).toEqual(["call", "call", "call", "stats", "stats"]);
});

it("a lone surrogate in a line reads as U+FFFD and is written as that character", () => {
	const decoded = decodeText(UsageLine, SURROGATE_LINE);
	expect(decoded.ok).toBe(true);
	if (!decoded.ok || !("call" in decoded.value)) return;
	expect([decoded.value.call.feature, decoded.value.call.model]).toEqual(["\u{fffd}", "m\u{fffd}x"]);
	const written = stringifyJson(decoded.value);
	expect(written.includes("\u{fffd}")).toBe(true);
	expect(written.includes("ud800")).toBe(false);
});
