// Gate runner. Run: bun scripts/gate.ts

import { spawnSync } from "node:child_process";
import { closeSync, existsSync, mkdtempSync, openSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
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
const TAIL_LINES = 60;
const DEMO = path.resolve(ROOT, process.env.NYAAAI_DEMO_DIR ?? "../nyaaai-demo");

////////////////////////////////
//  Functions & Helpers

function parts(): Part[] {
	const cargo = cargoBin();
	const demoManifest = path.join(DEMO, "Cargo.toml");
	const demo = existsSync(demoManifest);
	return [
		{ name: "link", command: "bun", args: ["scripts/consumer.ts", "link"] },
		{ name: "contract", command: "bun", args: ["scripts/contract.ts"] },
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
				"--all-features",
				"--locked",
				"--",
				"-D",
				"warnings",
			],
		},
		{
			name: "clippy-plain",
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
		{
			name: "cargo-test",
			command: cargo,
			args: ["test", "--manifest-path", MANIFEST, "--workspace", "--all-features", "--locked"],
		},
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
		{ name: "samples", command: "bun", args: ["scripts/samples.ts"] },
		{ name: "symbols", command: "bun", args: ["scripts/bannedSymbols.ts"] },
		...(demo
			? [{ name: "demo", command: cargo, args: ["test", "--manifest-path", demoManifest, "--locked"] }]
			: []),
	];
}

/** Runs a part, output kept in order. */
function runQuiet(part: Part): { ok: boolean; output: string } {
	const scratch = mkdtempSync(path.join(tmpdir(), "nyaaai-gate-"));
	const file = path.join(scratch, "output");
	const fd = openSync(file, "w");
	try {
		const ran = spawnSync(part.command, part.args, { cwd: ROOT, stdio: ["ignore", fd, fd] });
		closeSync(fd);
		return { ok: ran.status === 0, output: readFileSync(file, "utf8") };
	} finally {
		rmSync(scratch, { recursive: true, force: true });
	}
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const args = process.argv.slice(2);
	const flags = args.filter((arg) => arg.startsWith("--"));
	const quiet = flags.includes("--quiet");
	const badFlags = flags.filter((flag) => flag !== "--quiet");
	if (badFlags.length > 0) {
		console.error(`no such flag: ${badFlags.join(", ")}`);
		process.exit(1);
	}
	const wanted = new Set(args.filter((arg) => !arg.startsWith("--")));
	let interrupted = false;
	process.on("SIGINT", () => {
		interrupted = true;
	});
	const results: Array<{ name: string; ok: boolean }> = [];
	const held = parts();
	const unknown = [...wanted].filter((name) => !held.some((part) => part.name === name));
	if (unknown.length > 0) {
		console.error(`no such part: ${unknown.join(", ")}`);
		process.exit(1);
	}
	if (!held.some((part) => part.name === "demo")) console.log(`\n[demo] SKIPPED: no ${DEMO}`);
	for (const part of held) {
		if (wanted.size > 0 && !wanted.has(part.name) && part.name !== "link") continue;
		if (interrupted) {
			results.push({ name: part.name, ok: false });
			continue;
		}
		if (quiet) {
			const ran = runQuiet(part);
			if (!ran.ok) {
				console.log(`\n[${part.name}]`);
				console.log(ran.output.trimEnd().split("\n").slice(-TAIL_LINES).join("\n"));
			}
			results.push({ name: part.name, ok: ran.ok });
			continue;
		}
		console.log(`\n[${part.name}]`);
		const ran = spawnSync(part.command, part.args, { cwd: ROOT, stdio: "inherit" });
		results.push({ name: part.name, ok: ran.status === 0 });
	}
	const ok = results.length > 0 && results.every((result) => result.ok);
	if (quiet) {
		console.log("");
		for (const { name } of results.filter((result) => !result.ok)) console.log(`${name}: FAILED`);
		console.log(ok ? `gate: ok (${results.length} parts)` : "gate: FAILED");
	} else {
		console.log("");
		for (const result of results) console.log(`${result.name}: ${result.ok ? "ok" : "FAILED"}`);
	}
	process.exit(ok ? 0 : 1);
}
