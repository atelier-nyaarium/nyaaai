// Gate: bun scripts/gate.ts

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { cargoBin, ROOT } from "./child";

////////////////////////////////
//  Interfaces & Types

interface Part {
	name: string;
	command: string;
	args: string[];
}

////////////////////////////////
//  Constants

const BIN = path.join(ROOT, "ts", "node_modules", ".bin");
const MANIFEST = path.join("rust", "Cargo.toml");
const DEMO = path.resolve(ROOT, process.env.NYAAAI_DEMO_DIR ?? "../nyaaai-demo");

////////////////////////////////
//  Functions & Helpers

function parts(): Part[] {
	const cargo = cargoBin();
	const demoManifest = path.join(DEMO, "Cargo.toml");
	const demo = existsSync(demoManifest);
	return [
		{ name: "biome", command: path.join(BIN, "biome"), args: ["ci", "."] },
		{ name: "tsc", command: path.join(BIN, "tsc"), args: ["--build"] },
		{ name: "test", command: "bun", args: ["test"] },
		{ name: "fmt", command: cargo, args: ["fmt", "--manifest-path", MANIFEST, "--all", "--check"] },
		{
			name: "clippy",
			command: cargo,
			args: [
				"clippy",
				"--manifest-path",
				MANIFEST,
				"--workspace",
				"--all-targets",
				"--locked",
				"--",
				"-D",
				"warnings",
			],
		},
		{ name: "cargo-test", command: cargo, args: ["test", "--manifest-path", MANIFEST, "--workspace", "--locked"] },
		{
			name: "crates",
			command: "bun",
			args: [
				"scripts/crates.ts",
				path.join("rust", "Cargo.lock"),
				...(demo ? [path.join(DEMO, "Cargo.lock")] : []),
			],
		},
		{ name: "cases", command: "bun", args: ["scripts/cases.ts"] },
		{ name: "symbols", command: "bun", args: ["scripts/bannedSymbols.ts"] },
		...(demo
			? [{ name: "demo", command: cargo, args: ["test", "--manifest-path", demoManifest, "--locked"] }]
			: []),
	];
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const wanted = new Set(process.argv.slice(2));
	let interrupted = false;
	process.on("SIGINT", () => {
		interrupted = true;
	});
	const results: Array<{ name: string; ok: boolean }> = [];
	const held = parts();
	const unknown = [...wanted].filter((name) => name !== "demo" && !held.some((part) => part.name === name));
	if (unknown.length > 0) {
		console.error(`no such part: ${unknown.join(", ")}`);
		process.exit(1);
	}
	if (!held.some((part) => part.name === "demo")) console.log(`\n[demo] SKIPPED: no ${DEMO}`);
	for (const part of held) {
		if (wanted.size > 0 && !wanted.has(part.name)) continue;
		if (interrupted) {
			results.push({ name: part.name, ok: false });
			continue;
		}
		console.log(`\n[${part.name}]`);
		const ran = spawnSync(part.command, part.args, { cwd: ROOT, stdio: "inherit" });
		results.push({ name: part.name, ok: ran.status === 0 });
	}
	console.log("");
	for (const { name, ok } of results) console.log(`${name}: ${ok ? "ok" : "FAILED"}`);
	process.exit(results.length > 0 && results.every((result) => result.ok) ? 0 : 1);
}
