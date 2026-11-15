import { expect, it } from "bun:test";
import { existsSync, mkdtempSync, readdirSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { plant, type TreeEntry } from "../tree";

it("plants a tree and keeps every entry inside it", () => {
	const outer = mkdtempSync(path.join(tmpdir(), "nyaaai-tree-"));
	const folder = path.join(outer, "tree");
	try {
		plant(folder, [
			{ path: "a/b" },
			{ path: "a/f.txt", text: "x" },
			{ path: "a/bytes", base64: "QQ==" },
			{ path: "out", link: "$TMP/.." },
		]);
		expect(readdirSync(path.join(folder, "a")).sort()).toEqual(["b", "bytes", "f.txt"]);
		expect(readFileSync(path.join(folder, "a", "bytes"), "utf8")).toBe("A");
		expect(readdirSync(path.join(folder, "out"))).toEqual(["tree"]);

		const escapes: TreeEntry[] = [
			{ path: "../beside.txt", text: "x" },
			{ path: "a/../../beside.txt", text: "x" },
			{ path: path.join(outer, "absolute.txt"), text: "x" },
			{ path: "out/through.txt", text: "x" },
			{ path: "out", text: "x" },
		];
		for (const entry of escapes) expect(() => plant(folder, [entry])).toThrow();
		for (const base64 of ["!!!", "QQ"]) expect(() => plant(folder, [{ path: "bad", base64 }])).toThrow();
		expect(readdirSync(outer)).toEqual(["tree"]);
		expect(existsSync(path.join(folder, "out"))).toBe(true);
	} finally {
		rmSync(outer, { recursive: true, force: true });
	}
});
