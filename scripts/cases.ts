// Case parity runner.
// Run: bun scripts/cases.ts [--refresh]

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { decode } from "@nyaaai/contract/shape";
import { CASE_FORMAT, CaseFile, type FunctionCase } from "@nyaaai/testkit/caseFile";
import { type AdapterCommand, runCommands, rustAdapter, TS_ADAPTER } from "./adapterHost";
import { CASE_PREFIX, type CaseSource, runCase } from "./caseAdapter";
import { adapterCases } from "./caseSources/adapter";
import { contractCases } from "./caseSources/contract";
import { base64Cases, canonicalCases, jsonCases } from "./caseSources/json";
import { numberCases } from "./caseSources/number";
import { pathCases } from "./caseSources/paths";
import { patternCases } from "./caseSources/pattern";
import { sessionCutCases } from "./caseSources/sessionCut";
import { sessionFileCases } from "./caseSources/sessionFile";
import { strictSchemaCases } from "./caseSources/strictSchema";
import { textCases } from "./caseSources/text";
import { timeCases } from "./caseSources/time";
import { ROOT } from "./child";
import { play, type RunSource } from "./runScript";

////////////////////////////////
//  Interfaces & Types

type Source = CaseSource | RunSource;

////////////////////////////////
//  Constants

const FOLDER = "cases";
const TRACES = "cases/traces";
export const SOURCES: Source[] = [
	...jsonCases,
	...canonicalCases,
	...base64Cases,
	...contractCases,
	...timeCases,
	...numberCases,
	...patternCases,
	...textCases,
	...sessionCutCases,
	...sessionFileCases,
	...strictSchemaCases,
	...pathCases,
	{ kind: "testkit.replay", name: "tool-loop", input: { file: "cases/traces/openrouter.tool-loop.json" } },
	{ kind: "testkit.replay", name: "chunk-edges", input: { file: "cases/traces/synthetic.chunks.json" } },
	{
		kind: "shape.compatibleVersions",
		name: "pairs",
		input: {
			pairs: [
				["1.2.3", "1.9.0"],
				["1.2.3", "2.0.0"],
				["0.1.0", "0.1.9"],
				["0.1.0", "0.2.0"],
				["0.0.1", "0.0.2"],
				["01.2.3", "1.02.003"],
				["000.1.0", "0.01.5"],
				["10.0.0", "1.0.0"],
				["1.2", "1.2.0"],
				["1.2.3.4", "1.2.3"],
				["1.2.x", "1.2.0"],
				["v1.2.3", "1.2.3"],
				["1.2.3 ", "1.2.3"],
				["1..3", "1.0.3"],
				["", ""],
				["latest", "latest"],
				["\u{661}.2.3", "1.2.3"],
				["1.2.3-beta", "1.2.3"],
				["99999999999999999999999.0.0", "99999999999999999999999.5.1"],
			].map(([left, right]) => ({ left, right })),
		},
	},
	{ kind: "testkit.random", name: "cycles", input: { values: [0, 0.5, 0.999999], draws: 7 } },
	{
		kind: "testkit.wheel",
		name: "timers",
		input: {
			startMs: 1000,
			steps: [
				{ op: "after", ms: 50, tag: "b" },
				{ op: "after", ms: 50, tag: "a" },
				{ op: "after", ms: 10, tag: "c" },
				{ op: "after", ms: 200, tag: "late" },
				{ op: "advance", ms: 9 },
				{ op: "advance", ms: 1 },
				{ op: "after", ms: 40, tag: "tie" },
				{ op: "cancel", tag: "a" },
				{ op: "cancel", tag: "a" },
				{ op: "advance", ms: 40 },
				{ op: "after", ms: 0, tag: "now" },
				{ op: "after", ms: -5, tag: "past" },
				{ op: "after", ms: 30, tag: "late" },
				{ op: "advance", ms: 0 },
				{ op: "advance", ms: -10 },
				{ op: "advance", ms: 1000 },
				{ op: "cancel", tag: "none" },
			],
		},
	},
	...adapterCases,
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

/** Generates TypeScript expectations. */
export async function freshFiles(sources: Source[] = SOURCES): Promise<Map<string, string>> {
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
					expected: await runCase(source.kind, source.input),
				};
		files.set(file, `${JSON.stringify(caseOf(file, made), null, "\t")}\n`);
	}
	return files;
}

function committedFiles(root: string, folder = FOLDER): string[] {
	const at = path.join(root, folder);
	if (!existsSync(at)) return [];
	return readdirSync(at, { withFileTypes: true }).flatMap((entry) => {
		const held = path.join(folder, entry.name);
		if (held === TRACES) return [];
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

/** Key order matters. */
const differs = (actual: unknown, expected: unknown): boolean => JSON.stringify(actual) !== JSON.stringify(expected);

const shown = (expected: unknown, actual: unknown): string =>
	`\n  expected ${JSON.stringify(expected)}\n  actual   ${JSON.stringify(actual)}`;

export async function adapterProblems(root: string, adapter: AdapterCommand): Promise<string[]> {
	const cases = committedFiles(root).map((file): [string, CaseFile] => [
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
		if (held.type !== "run") continue;
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
	const problems = [
		...drift(ROOT, fresh),
		...(await adapterProblems(ROOT, TS_ADAPTER)),
		...(await adapterProblems(ROOT, rustAdapter())),
	];
	for (const problem of problems) console.error(problem);
	console.log(problems.length === 0 ? `cases: ${fresh.size} agree in both languages` : "cases: FAILED");
	process.exit(problems.length === 0 ? 0 : 1);
}
