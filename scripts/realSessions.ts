// Compare sessions without transcript content.
// Run: bun scripts/realSessions.ts

import { spawnSync } from "node:child_process";
import { mkdtempSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import path from "node:path";
import { runCase } from "./caseAdapter";
import { cargoBin, ROOT } from "./child";

////////////////////////////////
//  Interfaces & Types

interface IndexResult {
	index: { lines: number; boundaries: number; version: string | null; unparsed: number };
}

////////////////////////////////
//  Constants

const PROVEN = "2.1.323";

////////////////////////////////
//  Functions & Helpers

function sessionFiles(root: string): string[] {
	return readdirSync(root, { withFileTypes: true })
		.filter((project) => project.isDirectory())
		.flatMap((project) =>
			readdirSync(path.join(root, project.name), { withFileTypes: true })
				.filter((file) => file.isFile() && file.name.endsWith(".jsonl"))
				.map((file) => path.join(root, project.name, file.name)),
		);
}

function rustResult(binary: string, caseFile: string): unknown {
	const ran = spawnSync(binary, [caseFile], { encoding: "utf8", maxBuffer: 1 << 30 });
	if (ran.status !== 0) throw new Error(ran.stderr.trim());
	const line = JSON.parse(ran.stdout) as { actual?: unknown; error?: string };
	if (line.error !== undefined) throw new Error(line.error);
	return line.actual;
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const root = process.argv[2] ?? path.join(homedir(), ".claude", "projects");
	const built = spawnSync(
		cargoBin(),
		[
			"build",
			"--quiet",
			"--locked",
			"--manifest-path",
			"rust/Cargo.toml",
			"-p",
			"nyaaai-cli",
			"--features",
			"cases",
		],
		{ cwd: ROOT, stdio: "inherit" },
	);
	if (built.status !== 0) process.exit(1);
	const binary = path.join(ROOT, "rust", "target", "debug", "nyaaai-cases");
	const scratch = mkdtempSync(path.join(tmpdir(), "nyaaai-real-sessions-"));
	let differing = 0;
	try {
		const files = sessionFiles(root);
		for (const file of files) {
			const source = { file };
			const cases = [
				{ kind: "sessionCut.index", input: { source } },
				{ kind: "sessionCut.plan", input: { source, requests: [{ mode: "trim" }], cliVersion: PROVEN } },
			];
			const verdicts: string[] = [];
			let facts = "";
			for (const held of cases) {
				const caseFile = path.join(scratch, "case.json");
				writeFileSync(caseFile, JSON.stringify({ format: 1, name: "real", ...held }));
				const ours = await runCase(held.kind, held.input);
				const same = Bun.deepEquals(ours, rustResult(binary, caseFile), true);
				verdicts.push(same ? "same" : "DIFFERS");
				if (!same) differing += 1;
				if (held.kind === "sessionCut.index") {
					const { index } = ours as IndexResult;
					facts = `lines=${index.lines} boundaries=${index.boundaries} unparsed=${index.unparsed} version=${index.version}`;
				}
			}
			const megabytes = (statSync(file).size / 1_048_576).toFixed(1);
			console.log(`index ${verdicts[0]}  plan ${verdicts[1]}  ${megabytes} MB  ${facts}  ${path.basename(file)}`);
		}
		console.log(`real sessions: ${files.length} files, ${differing} differing results`);
	} finally {
		rmSync(scratch, { recursive: true, force: true });
	}
	process.exit(differing === 0 ? 0 : 1);
}
