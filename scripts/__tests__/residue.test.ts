// Tree residue tests.

import { describe, expect, it } from "bun:test";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { ROOT } from "../child";

////////////////////////////////
//  Functions & Helpers

/** Operator names from manifests. */
function operatorNames(): string[] {
	const packageName = (
		JSON.parse(readFileSync(path.join(ROOT, "ts", "operator", "package.json"), "utf8")) as { name: string }
	).name;
	const crate = /^name = "([^"]+)"$/m.exec(
		readFileSync(path.join(ROOT, "rust", "operator", "Cargo.toml"), "utf8"),
	)?.[1];
	if (!crate) throw new Error("the operator crate names itself nowhere");
	return [packageName, crate, crate.replaceAll("-", "_"), "../operator/"];
}

function filesUnder(folder: string): string[] {
	return readdirSync(folder, { withFileTypes: true }).flatMap((entry) => {
		const held = path.join(folder, entry.name);
		if (entry.name === "node_modules" || entry.name === "target" || entry.name === ".tsbuild") return [];
		return entry.isDirectory() ? filesUnder(held) : [held];
	});
}

/** Key-backed files referencing operator names. */
function namesOperator(files: Array<[string, string]>): string[] {
	const names = operatorNames();
	return files.filter(([, text]) => names.some((name) => text.includes(name))).map(([file]) => file);
}

////////////////////////////////
//  Tests

describe("residue", () => {
	it("agentic never names operator, in either language", () => {
		const folders = [path.join(ROOT, "ts", "agentic"), path.join(ROOT, "rust", "agentic")];
		const files = folders.flatMap(filesUnder).map((file): [string, string] => [file, readFileSync(file, "utf8")]);
		expect(files.length).toBeGreaterThan(0);
		expect(namesOperator(files)).toEqual([]);

		const [packageName, crate, crateInCode] = operatorNames();
		const probes: Array<[string, string]> = [
			["a.ts", `import { x } from "${packageName}/claude/sessionCut";`],
			["b.ts", 'import { x } from "../../operator/src/index";'],
			["Cargo.toml", `${crate}.workspace = true`],
			["c.rs", `use ${crateInCode}::claude;`],
			["d.ts", 'import { x } from "@nyaaai/contract/json";'],
		];
		expect(namesOperator(probes)).toEqual(["a.ts", "b.ts", "Cargo.toml", "c.rs"]);
	});
});
