// Rust matches TypeScript case expectations.
// Run: bun scripts/cases.ts [--refresh]

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { type CaseSource, runCase } from "./caseAdapter";
import { jsonCases } from "./caseSources/json";
import { sessionCutCases } from "./caseSources/sessionCut";
import { sessionFileCases } from "./caseSources/sessionFile";
import { timeCases } from "./caseSources/time";
import { cargoBin, ROOT } from "./child";

////////////////////////////////
//  Interfaces & Types

interface CaseFile {
	format: number;
	kind: string;
	name: string;
	input: unknown;
	expected: unknown;
}

////////////////////////////////
//  Constants

const FORMAT = 1;
const FOLDER = "cases";
const SOURCES: CaseSource[] = [
	...jsonCases,
	...timeCases,
	...sessionCutCases,
	...sessionFileCases,
	{ kind: "testkit.compactedSession", name: "rows", input: {} },
	{
		kind: "claude.versionIn",
		name: "answers",
		input: {
			texts: [
				"2.1.323 (Claude Code)",
				"v10.20.3-beta",
				"1.2 then 3.4.5",
				"no version 1.",
				"",
				"1.2.3.4",
				"1..2.3 4.5.6",
				"\u{663}.\u{664}.\u{665} 7.8.9",
				"1.2.x 3.4.5",
				"007.08.9",
				"1.2.\n3 4.5.6",
			],
		},
	},
];

////////////////////////////////
//  Functions & Helpers

/** `cases/<group>/<operation>.<name>.json` */
function fileOf(source: CaseSource): string {
	const [group, ...operation] = source.kind.split(".");
	return path.join(FOLDER, group as string, `${operation.join(".")}.${source.name}.json`);
}

export async function freshFiles(sources: CaseSource[] = SOURCES): Promise<Map<string, string>> {
	const files = new Map<string, string>();
	for (const source of sources) {
		const file = fileOf(source);
		if (files.has(file)) throw new Error(`two cases write ${file}`);
		const held: CaseFile = {
			format: FORMAT,
			kind: source.kind,
			name: source.name,
			input: source.input,
			expected: await runCase(source.kind, source.input),
		};
		files.set(file, `${JSON.stringify(held, null, "\t")}\n`);
	}
	return files;
}

function committedFiles(root: string, folder = FOLDER): string[] {
	const at = path.join(root, folder);
	if (!existsSync(at)) return [];
	return readdirSync(at, { withFileTypes: true }).flatMap((entry) => {
		const held = path.join(folder, entry.name);
		if (entry.isDirectory()) return committedFiles(root, held);
		return entry.name.endsWith(".json") ? [held] : [];
	});
}

export function drift(root: string, fresh: Map<string, string>): string[] {
	const problems: string[] = [];
	const committed = new Set(committedFiles(root));
	for (const [file, text] of fresh) {
		if (!committed.has(file)) problems.push(`${file} is not written`);
		else if (readFileSync(path.join(root, file), "utf8") !== text)
			problems.push(`${file} differs from a fresh run`);
	}
	for (const file of committed) if (!fresh.has(file)) problems.push(`${file} has no source`);
	return problems;
}

/** Finds Rust case mismatches. */
export function rustProblems(root: string): string[] {
	const files = committedFiles(root);
	if (files.length === 0) return [];
	const ran = spawnSync(
		cargoBin(),
		["run", "--quiet", "--locked", "--manifest-path", "rust/Cargo.toml", "-p", "nyaaai-cli", "--", "cases", FOLDER],
		{ cwd: root, encoding: "utf8", stdio: ["ignore", "pipe", "inherit"], maxBuffer: 1 << 30 },
	);
	if (ran.status !== 0) return ["the Rust adapter failed"];
	const problems: string[] = [];
	const seen = new Set<string>();
	for (const line of ran.stdout.split("\n")) {
		if (line.length === 0) continue;
		const result = JSON.parse(line) as { file: string; actual?: unknown; error?: string };
		seen.add(result.file);
		if (result.error !== undefined) {
			problems.push(`${result.file}: Rust failed: ${result.error}`);
			continue;
		}
		const held = JSON.parse(readFileSync(path.join(root, result.file), "utf8")) as CaseFile;
		if (!Bun.deepEquals(result.actual, held.expected, true)) {
			problems.push(
				`${result.file}: Rust differs\n  expected ${JSON.stringify(held.expected)}\n  actual   ${JSON.stringify(result.actual)}`,
			);
		}
	}
	for (const file of files) if (!seen.has(file)) problems.push(`${file}: Rust gave no result`);
	return problems;
}

function refresh(root: string, fresh: Map<string, string>): void {
	for (const file of committedFiles(root)) if (!fresh.has(file)) rmSync(path.join(root, file));
	for (const [file, text] of fresh) {
		mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
		writeFileSync(path.join(root, file), text);
	}
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const fresh = await freshFiles();
	if (process.argv.includes("--refresh")) {
		const tests = spawnSync("bun", ["test"], { cwd: ROOT, stdio: "inherit" });
		if (tests.status !== 0) {
			console.error("cases: not refreshed, the TypeScript tests fail");
			process.exit(1);
		}
		refresh(ROOT, fresh);
		console.log(`cases: ${fresh.size} written`);
	}
	const problems = [...drift(ROOT, fresh), ...rustProblems(ROOT)];
	for (const problem of problems) console.error(problem);
	console.log(problems.length === 0 ? `cases: ${fresh.size} agree in both languages` : "cases: FAILED");
	process.exit(problems.length === 0 ? 0 : 1);
}
