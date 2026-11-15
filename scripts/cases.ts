// Case parity runner.
// Run: bun scripts/cases.ts [--refresh]

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, lstatSync, mkdirSync, readFileSync, readlinkSync, writeFileSync } from "node:fs";
import path from "node:path";
import { decode } from "@nyaaai/contract/shape";
import { CASE_FORMAT, CaseFile, type FunctionCase, unhandled } from "@nyaaai/testkit/caseFile";
import type { CaseSource } from "@nyaaai/testkit/caseGroup";
import { nameProblems, type Project, type RunSource } from "@nyaaai/testkit/project";
import { type AdapterCommand, runCommands, rustAdapter, TS_ADAPTER } from "./adapterHost";
import { CASE_PREFIX, runCase } from "./caseAdapter";
import { gitOrNull } from "./child";
import { manifestProblems, removeStale, writeList } from "./manifest";
import { loadProject, PROJECT_ROOT } from "./project";
import { play } from "./runScript";

////////////////////////////////
//  Interfaces & Types

type Source = CaseSource | RunSource;

////////////////////////////////
//  Constants

const FOLDER = "cases";

////////////////////////////////
//  Functions & Helpers

function fileOf(source: Source): string {
	const [group, ...operation] = source.kind.split(".");
	return path.join(FOLDER, group as string, `${operation.join(".")}.${source.name}.json`);
}

const isRun = (source: Source): source is RunSource => "script" in source;

function caseOf(file: string, value: unknown): CaseFile {
	const held = decode(CaseFile, value);
	if (!held.ok) throw new Error(`${file} is no case file at ${JSON.stringify(held.path)}: ${held.message}`);
	return held.value;
}

export const sourcesOf = (project: Project): Source[] => [
	...project.groups.flatMap((group) => group.sources),
	...project.runs,
];

/** Generates TypeScript expectations. */
export async function freshFiles(project: Project, sources = sourcesOf(project)): Promise<Map<string, string>> {
	const files = new Map<string, string>();
	for (const source of sources) {
		const file = fileOf(source);
		if (files.has(file)) throw new Error(`two cases write ${file}`);
		const made = isRun(source)
			? { format: CASE_FORMAT, ...source, expected: await play(TS_ADAPTER, source) }
			: {
					format: CASE_FORMAT,
					type: "function",
					kind: source.kind,
					name: source.name,
					input: source.input,
					expected: await runCase(project, source.kind, source.input),
				};
		files.set(file, `${JSON.stringify(caseOf(file, made), null, "\t")}\n`);
	}
	return files;
}

export function drift(root: string, fresh: Map<string, string>): string[] {
	const problems: string[] = [];
	for (const [file, text] of fresh) {
		const at = path.join(root, file);
		if (!existsSync(at)) problems.push(`${file} is not written`);
		else if (readFileSync(at, "utf8") !== text) problems.push(`${file} differs from a fresh run`);
	}
	return [...problems, ...manifestProblems(root, "cases", fresh.keys())];
}

/** Key order matters. */
const differs = (actual: unknown, expected: unknown): boolean => JSON.stringify(actual) !== JSON.stringify(expected);

const shown = (expected: unknown, actual: unknown): string =>
	`\n  expected ${JSON.stringify(expected)}\n  actual   ${JSON.stringify(actual)}`;

/** Runs the generated case files that exist. */
export async function adapterProblems(root: string, adapter: AdapterCommand, files: string[]): Promise<string[]> {
	const cases = files
		.filter((file) => existsSync(path.join(root, file)))
		.map((file): [string, CaseFile] => [
			file,
			caseOf(file, JSON.parse(readFileSync(path.join(root, file), "utf8"))),
		]);
	const problems: string[] = [];
	const functions = cases.filter((held): held is [string, FunctionCase] => held[1].type === "function");
	const results = await runCommands(
		adapter,
		functions.map(([file, held]) => ({ run: file, command: `${CASE_PREFIX}${held.kind}`, input: held.input })),
	);
	for (const [file, held] of functions) {
		const result = results.get(file);
		if (!result) problems.push(`${file}: ${adapter.name} gave no result`);
		else if ("failed" in result) problems.push(`${file}: ${adapter.name} failed: ${JSON.stringify(result.failed)}`);
		else if (differs(result.result, held.expected))
			problems.push(`${file}: ${adapter.name} differs${shown(held.expected, result.result)}`);
	}
	for (const [file, held] of cases) {
		// Function cases ran together.
		if (held.type === "function") continue;
		if (held.type !== "run") unhandled(held);
		try {
			const played = await play(adapter, held);
			if (differs(played, held.expected))
				problems.push(`${file}: ${adapter.name} differs${shown(held.expected, played)}`);
		} catch (error) {
			problems.push(`${file}: ${error instanceof Error ? error.message : error}`);
		}
	}
	return problems;
}

function refresh(root: string, fresh: Map<string, string>): void {
	removeStale(root, "cases", fresh.keys());
	for (const [file, text] of fresh) {
		mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
		writeFileSync(path.join(root, file), text);
	}
	writeList(root, "cases", fresh.keys());
}

/** Content and mode of tracked and untracked files, ignored ones left out. Null outside git. */
export function treeHash(root: string): string | null {
	if (gitOrNull(root, ["rev-parse", "--is-inside-work-tree"]) !== "true") return null;
	const listed = spawnSync("git", ["ls-files", "-z", "--cached", "--others", "--exclude-standard"], {
		cwd: root,
		maxBuffer: 1 << 30,
	});
	if (listed.status !== 0) return null;
	const hash = createHash("sha256");
	for (const file of listed.stdout.toString("utf8").split("\0").sort()) {
		if (file === "") continue;
		const at = path.join(root, file);
		const held = lstatSync(at, { throwIfNoEntry: false });
		hash.update(`${file}\0`);
		try {
			if (!held) hash.update("missing");
			else if (held.isSymbolicLink()) hash.update(`link ${held.mode} ${readlinkSync(at)}`);
			else if (held.isFile()) hash.update(`file ${held.mode} `).update(readFileSync(at));
			else hash.update(`folder ${held.mode}`);
		} catch {
			hash.update("missing");
		}
		hash.update("\0");
	}
	return hash.digest("hex");
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const project = await loadProject();
	const named = nameProblems(project);
	if (named.length > 0) {
		for (const problem of named) console.error(problem);
		console.log("cases: FAILED");
		process.exit(1);
	}
	const fresh = await freshFiles(project);
	if (process.argv.includes("--refresh")) {
		const [command, ...args] = project.tests ?? [];
		if (command !== undefined && spawnSync(command, args, { cwd: PROJECT_ROOT, stdio: "inherit" }).status !== 0) {
			console.error("cases: not refreshed, the project's tests fail");
			process.exit(1);
		}
		refresh(PROJECT_ROOT, fresh);
		console.log(`cases: ${fresh.size} written`);
	}
	const before = treeHash(PROJECT_ROOT);
	if (before === null) console.log("cases: the project tree check did not run, this is no git checkout");
	const problems = [
		...drift(PROJECT_ROOT, fresh),
		...(await adapterProblems(PROJECT_ROOT, TS_ADAPTER, [...fresh.keys()])),
		...(await adapterProblems(PROJECT_ROOT, rustAdapter(project), [...fresh.keys()])),
	];
	if (treeHash(PROJECT_ROOT) !== before)
		problems.push("a case changed the project tree; cases write under their temp folder");
	for (const problem of problems) console.error(problem);
	console.log(problems.length === 0 ? `cases: ${fresh.size} agree in both languages` : "cases: FAILED");
	process.exit(problems.length === 0 ? 0 : 1);
}
