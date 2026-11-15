// Compare real sessions.
// Run: bun scripts/realSessions.ts

import { readdirSync, statSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";
import { runCommands, rustAdapter } from "./adapterHost";
import { CASE_PREFIX, runCase } from "./caseAdapter";
import { loadProject } from "./project";

////////////////////////////////
//  Interfaces & Types

interface IndexResult {
	index: { lines: number; boundaries: number; version: string | null; unparsed: number };
}

////////////////////////////////
//  Constants

const PROVEN = "2.1.323";

/** Large session-file timeout. */
const LARGE_FILE_MS = 600_000;

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

////////////////////////////////
//  Main

if (import.meta.main) {
	const project = await loadProject();
	const root = process.argv[2] ?? path.join(homedir(), ".claude", "projects");
	const files = sessionFiles(root);
	const stamp = (file: string): string => {
		const held = statSync(file, { throwIfNoEntry: false });
		return held ? `${held.size} ${held.mtimeMs}` : "gone";
	};
	const before = new Map(files.map((file) => [file, stamp(file)]));
	const cases = files.flatMap((file) => {
		const source = { file };
		return [
			{ file, kind: "sessionCut.index", input: { source } },
			{ file, kind: "sessionCut.plan", input: { source, requests: [{ mode: "trim" }], cliVersion: PROVEN } },
		];
	});
	const rust = await runCommands(
		rustAdapter(project),
		cases.map((held, at) => ({ run: String(at), command: `${CASE_PREFIX}${held.kind}`, input: held.input })),
		LARGE_FILE_MS,
	);
	let differing = 0;
	// Exclude changed sessions.
	const live = new Set<string>();
	const odd = new Set<string>();
	const lines = new Map<string, string[]>();
	for (const [at, held] of cases.entries()) {
		// Mid-check session changes.
		const ours = await runCase(project, held.kind, held.input).catch(() => undefined);
		const theirs = rust.get(String(at));
		const same =
			ours !== undefined &&
			theirs !== undefined &&
			"result" in theirs &&
			JSON.stringify(ours) === JSON.stringify(theirs.result);
		const moved = !same && before.get(held.file) !== stamp(held.file);
		if (moved) live.add(held.file);
		else if (!same) differing += 1;
		if (!same) odd.add(held.file);
		const said = lines.get(held.file) ?? [];
		said.push(`${held.kind.split(".")[1]} ${same ? "same" : moved ? "live" : "DIFFERS"}`);
		if (held.kind === "sessionCut.index" && ours !== undefined) {
			const { index } = ours as IndexResult;
			const megabytes = ((statSync(held.file, { throwIfNoEntry: false })?.size ?? 0) / 1_048_576).toFixed(1);
			said.push(
				`${megabytes} MB  lines=${index.lines} boundaries=${index.boundaries} unparsed=${index.unparsed} version=${index.version}`,
			);
		}
		lines.set(held.file, said);
	}
	for (const file of odd) console.log(`${lines.get(file)?.join("  ")}  ${path.basename(file)}`);
	console.log(`real sessions: ${files.length} files, ${differing} differing results, ${live.size} live`);
	process.exit(differing === 0 ? 0 : 1);
}
