// Contract generator. Run: bun scripts/contract.ts [--refresh]

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { exportSchemas, SCHEMA_FORMS, type SetSchema } from "@nyaaai/contract/schemaExport";
import { type CaseGroup, kindsOf } from "@nyaaai/testkit/caseGroup";
import { CASES_SET, nameProblems, type Project, type RustTarget } from "@nyaaai/testkit/project";
import { pascal, quote, snake } from "@nyaaai/testkit/rustNames";
import { rustfmtBin } from "./child";
import { formProblems, irOf } from "./contract/ir";
import { emitRust } from "./contract/rust";
import { manifestProblems, removeStale, writeList } from "./manifest";
import { loadProject, PROJECT_ROOT } from "./project";
import { promptFiles } from "./prompts";

////////////////////////////////
//  Interfaces & Types

interface Fresh {
	files: Map<string, string>;
	problems: string[];
}

////////////////////////////////
//  Constants

const SCHEMA = "schema";
const GENERATED = "//! Generated from the case groups by scripts/contract.ts. Do not edit.";
/** Formatter output limit. */
const GENERATED_MAX_BYTES = 1 << 28;

////////////////////////////////
//  Functions & Helpers

/** Local or NyaaAI Biome. */
function biomeBin(): string {
	const held = [PROJECT_ROOT, path.join(PROJECT_ROOT, "ts")]
		.map((folder) => path.join(folder, "node_modules", ".bin", "biome"))
		.find((file) => existsSync(file));
	if (!held) throw new Error("no Biome to format generated TypeScript with");
	return held;
}

function biomeFormat(text: string): string {
	const ran = spawnSync(biomeBin(), ["format", "--stdin-file-path=generated.ts"], {
		cwd: PROJECT_ROOT,
		input: text,
		encoding: "utf8",
		maxBuffer: GENERATED_MAX_BYTES,
	});
	if (ran.status !== 0) throw new Error(`biome refused generated code:\n${ran.error?.message ?? ran.stderr}`);
	return ran.stdout;
}

function rustfmt(text: string): string {
	const ran = spawnSync(rustfmtBin(), ["--edition", "2024"], {
		cwd: PROJECT_ROOT,
		input: text,
		encoding: "utf8",
		maxBuffer: GENERATED_MAX_BYTES,
	});
	if (ran.status !== 0) throw new Error(`rustfmt refused generated code:\n${ran.error?.message ?? ran.stderr}`);
	return ran.stdout;
}

const formatted = (file: string, text: string): string =>
	file.endsWith(".rs") ? rustfmt(text) : file.endsWith(".ts") ? biomeFormat(text) : text;

/** Rust cases beside handlers. */
const casesTarget = (project: Project): RustTarget => ({
	crate: project.cases.crate,
	module: `${project.cases.module}::shapes`,
	file: `${project.cases.dir}/shapes.rs`,
	bare: true,
});

/** Snake-case Rust routes. */
function caseDispatch(groups: CaseGroup[]): string {
	const arms = [...kindsOf(groups)].map(([name, held]) => {
		const [group, key] = name.split(".") as [string, string];
		const shapes = `shapes::${pascal(name)}Input, shapes::${pascal(name)}Result`;
		const handler = `super::${snake(group)}::${snake(key)}`;
		return held.context
			? `${quote(name)} => run_with::<${shapes}>(input, root, ${handler}),`
			: `${quote(name)} => run::<${shapes}>(input, ${handler}),`;
	});
	const contextual = [...kindsOf(groups).values()].some((held) => held.context);
	return `${GENERATED}

use std::path::Path;

use nyaaai_contract::json::Value;
use nyaaai_testkit::cases::{run${contextual ? ", run_with" : ""}};

use super::shapes;

/// Runs one case kind. \`root\` is the project root.
pub fn dispatch(kind: &str, input: &Value, root: &Path) -> Option<Result<Value, String>> {
${contextual ? "" : "let _ = root;"}
Some(match kind {
${arms.join("\n")}
_ => return None,
})
}
`;
}

/** One match over every set with a generated `normalize`. */
function normalizerRegistry(project: Project): string {
	const held = new Map<string, RustTarget>();
	for (const [set, target] of [...Object.entries(project.sets), ...Object.entries(project.external)]) {
		if (target && !target.bare) held.set(set, target);
	}
	const arms = [...held].map(([set, target]) => {
		const root = target.crate === project.cases.crate ? "crate" : target.crate;
		return `${quote(set)} => ${root}::${target.module}::normalize(shape, value),`;
	});
	return `${GENERATED}

use nyaaai_contract::json::Value;
use nyaaai_contract::shape::Refused;

/// Normalizes a value as a shape of a set. None for an unknown set or shape.
pub fn normalize(set: &str, shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
${arms.length === 0 ? "let _ = (set, shape, value);\nNone" : `match set {\n${arms.join("\n")}\n_ => None,\n}`}
}
`;
}

const caseModules = (groups: CaseGroup[]): string =>
	`${GENERATED}

pub mod dispatch;
pub mod normalizers;
pub mod shapes;

${groups.map((group) => `pub mod ${snake(group.name)};`).join("\n")}
`;

/** Names that become one Rust name. */
function collisions(project: Project): string[] {
	const problems: string[] = [];
	const clash = (what: string, names: string[]): void => {
		const seen = new Map<string, string>();
		for (const name of names) {
			const held = seen.get(snake(name));
			if (held !== undefined) problems.push(`${what} ${held} and ${name} are both ${snake(name)} in Rust`);
			else seen.set(snake(name), name);
		}
	};
	clash(
		"groups",
		project.groups.map((group) => group.name),
	);
	for (const group of project.groups) clash(`kinds of ${group.name},`, Object.keys(group.kinds));
	return problems;
}

/** A set listed twice. */
const doubled = (project: Project): string[] =>
	Object.keys(project.external)
		.filter((set) => Object.hasOwn(project.sets, set))
		.map((set) => `set ${set} is under both sets and external`);

export function freshFiles(project: Project): Fresh {
	const { docs, problems } = exportSchemas();
	problems.push(...nameProblems(project), ...collisions(project), ...doubled(project));
	const files = new Map<string, string>();
	const outputs = new Map<string, SetSchema>();
	const targets = new Map<string, RustTarget>([
		...Object.entries(project.sets),
		...Object.entries(project.external).flatMap(
			([set, held]): Array<[string, RustTarget]> => (held ? [[set, held]] : []),
		),
		[CASES_SET, casesTarget(project)],
	]);
	const external = new Set(Object.keys(project.external));
	for (const [set, forms] of docs) {
		outputs.set(set, forms.output);
		if (external.has(set)) continue;
		for (const form of SCHEMA_FORMS) {
			files.set(`${SCHEMA}/${set}.${form}.json`, `${JSON.stringify(forms[form], null, "\t")}\n`);
		}
		problems.push(...formProblems(set, forms.input, forms.output));
		if (!targets.has(set) && !project.schemaOnly.includes(set)) problems.push(`set ${set} has no Rust target`);
	}
	for (const set of targets.keys()) if (!docs.has(set)) problems.push(`target ${set} has no set`);
	if (problems.length > 0) return { files, problems };
	try {
		const sets = irOf(outputs);
		for (const [set, ir] of sets) {
			const target = targets.get(set);
			if (target && !external.has(set)) files.set(target.file, rustfmt(emitRust(ir, sets, targets)));
		}
	} catch (error) {
		return { files, problems: [error instanceof Error ? error.message : String(error)] };
	}
	const prompts = promptFiles(PROJECT_ROOT, project.prompts);
	if (prompts.problems.length > 0) return { files, problems: prompts.problems };
	const more = new Map<string, string>([
		...prompts.typescript,
		...prompts.rust,
		[`${project.cases.dir}/dispatch.rs`, caseDispatch(project.groups)],
		[`${project.cases.dir}/normalizers.rs`, normalizerRegistry(project)],
		[`${project.cases.dir}/mod.rs`, caseModules(project.groups)],
		...project.generated.flatMap((make) => [...make()]),
	]);
	for (const [file, text] of more) {
		if (files.has(file)) problems.push(`${file} is generated twice`);
		files.set(file, formatted(file, text));
	}
	return { files, problems };
}

export function drift(root: string, fresh: Map<string, string>): string[] {
	const problems: string[] = [];
	for (const [file, text] of fresh) {
		const at = path.join(root, file);
		if (!existsSync(at)) problems.push(`${file} is not written`);
		else if (readFileSync(at, "utf8") !== text) problems.push(`${file} differs from a fresh run`);
	}
	return [...problems, ...manifestProblems(root, "contract", fresh.keys())];
}

function refresh(root: string, fresh: Map<string, string>): void {
	removeStale(root, "contract", fresh.keys());
	for (const [file, text] of fresh) {
		mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
		writeFileSync(path.join(root, file), text);
	}
	writeList(root, "contract", fresh.keys());
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const { files, problems } = freshFiles(await loadProject());
	if (problems.length === 0 && process.argv.includes("--refresh")) {
		refresh(PROJECT_ROOT, files);
		console.log(`contract: ${files.size} written`);
	}
	const found = problems.length > 0 ? problems : drift(PROJECT_ROOT, files);
	for (const problem of found) console.error(problem);
	console.log(found.length === 0 ? `contract: ${files.size} files current` : "contract: FAILED");
	process.exit(found.length === 0 ? 0 : 1);
}
