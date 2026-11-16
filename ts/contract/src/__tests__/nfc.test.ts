import { describe, expect, it } from "bun:test";
import { CCC, COMPOSE, DECOMP, UNICODE_VERSION } from "../nfcTables";
import { nfc } from "../text";

describe("nfc", () => {
	it("pins the Unicode 17.0 tables", () => {
		expect(UNICODE_VERSION).toBe("17.0.0");
		expect(CCC.length).toBe(968);
		expect(DECOMP.length / 3).toBe(2081);
		expect(COMPOSE.length / 3).toBe(961);
	});

	it("normalizes decomposition, ordering and composition", () => {
		expect(nfc("")).toBe("");
		expect(nfc("plain ASCII")).toBe("plain ASCII");
		expect(nfc("e\u{301}")).toBe("\u{e9}");
		expect(nfc("a\u{301}\u{323}")).toBe(nfc("a\u{323}\u{301}"));
		expect(nfc("\u{1100}\u{1161}")).toBe("\u{ac00}");
		expect(nfc("\u{1100}\u{1161}\u{11a8}")).toBe("\u{ac01}");
		expect(nfc("\u{212a}")).toBe("K");
		expect(nfc("\u{037e}")).toBe(";");
		expect(nfc("\u{0958}")).toBe("\u{0915}\u{093c}");
		expect(nfc("\u{2f800}")).toBe("\u{4e3d}");
		expect(nfc("\u{0378}")).toBe("\u{0378}");
	});

	it("turns a lone surrogate into U+FFFD first", () => {
		expect(nfc("\ud800")).toBe("\u{fffd}");
		expect(nfc("\udc00")).toBe("\u{fffd}");
		expect(nfc("a\ud800b\udc00")).toBe("a\u{fffd}b\u{fffd}");
		expect(nfc("\ud83d\ude00")).toBe("\u{1f600}");
	});
});
