// Contract generator.
// Run: bun scripts/contract.ts [--refresh]

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import path from "node:path";
import { ISSUE_MESSAGES } from "@nyaaai/contract/issues";
import { exportSchemas, SCHEMA_FORMS, type SetSchema } from "@nyaaai/contract/schemaExport";
import { CONTRACT_VERSION } from "@nyaaai/contract/shape";
import "@nyaaai/contract/shapes/frames";
import "@nyaaai/contract/shapes/sessionCut";
import { CASE_KINDS } from "@nyaaai/testkit/caseKinds";
import "@nyaaai/testkit/caseFile";
import "@nyaaai/testkit/tortureShapes";
import "@nyaaai/testkit/trace";
import { ROOT, rustfmtBin } from "./child";
import { formProblems, irOf } from "./contract/ir";
import { emitRust, pascal, quote, type RustTarget, snake } from "./contract/rust";
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
const VERSION_FILE = "rust/contract/src/version.rs";
const DISPATCH_FILE = "rust/cli/src/case_dispatch.rs";

/** Imports register Rust targets. */
export const TARGETS = new Map<string, RustTarget>([
	["frames", { crate: "nyaaai_contract", module: "shapes::frames", file: "rust/contract/src/shapes/frames.rs" }],
	[
		"sessionCut",
		{ crate: "nyaaai_contract", module: "shapes::session_cut", file: "rust/contract/src/shapes/session_cut.rs" },
	],
	["torture", { crate: "nyaaai_testkit", module: "torture_shapes", file: "rust/testkit/src/torture_shapes.rs" }],
	["trace", { crate: "nyaaai_testkit", module: "trace_shapes", file: "rust/testkit/src/trace_shapes.rs" }],
	["cases", { crate: "nyaaai_cases", module: "case_shapes", file: "rust/cli/src/case_shapes.rs", bare: true }],
]);

/** Schema-only sets. */
const SCHEMA_ONLY = new Set(["caseFile"]);
function issueRust(): string {
	const entries = Object.entries(ISSUE_MESSAGES);
	const variants = entries.map(([code]) => pascal(code)).join(",\n    ");
	const names = entries.map(([code]) => `Self::${pascal(code)} => ${quote(code)},`).join("\n            ");
	const templates = entries
		.map(([code, template]) => `Self::${pascal(code)} => ${quote(template)},`)
		.join("\n            ");
	return [
		"//! Generated.",
		"",
		"#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
		"pub enum IssueCode {",
		`    ${variants}`,
		"}",
		"",
		"impl IssueCode {",
		"    pub fn as_str(&self) -> &'static str {",
		"        match self {",
		`            ${names}`,
		"        }",
		"    }",
		"",
		"    pub fn template(&self) -> &'static str {",
		"        match self {",
		`            ${templates}`,
		"        }",
		"    }",
		"}",
		"",
	].join("\n");
}

////////////////////////////////
//  Functions & Helpers

function biomeFormat(text: string): string {
	const ran = spawnSync(
		path.join(ROOT, "ts", "node_modules", ".bin", "biome"),
		["format", "--stdin-file-path=generated.ts"],
		{
			cwd: ROOT,
			input: text,
			encoding: "utf8",
		},
	);
	if (ran.status !== 0) throw new Error(`biome refused generated code:\n${ran.stderr}`);
	return ran.stdout;
}

function rustfmt(text: string): string {
	const ran = spawnSync(rustfmtBin(), ["--edition", "2024"], { cwd: ROOT, input: text, encoding: "utf8" });
	if (ran.status !== 0) throw new Error(`rustfmt refused generated code:\n${ran.stderr}`);
	return ran.stdout;
}

function caseDispatch(): string {
	const arms = Object.keys(CASE_KINDS).map((kind) => {
		const name = pascal(kind);
		return `${quote(kind)} => run::<case_shapes::${name}Input, case_shapes::${name}Result>(input, handlers::${snake(kind)}),`;
	});
	return [
		"//! Generated from the case kinds by scripts/contract.ts. Do not edit.",
		"",
		"use nyaaai_contract::json::Value;",
		"use nyaaai_contract::shape::Shape;",
		"",
		"use super::case_handlers as handlers;",
		"use super::case_shapes;",
		"",
		"/// Runs one case kind.",
		"pub fn dispatch(kind: &str, input: &Value) -> Option<Result<Value, String>> {",
		"Some(match kind {",
		...arms,
		"_ => return None,",
		"})",
		"}",
		"",
		"fn run<Input: Shape, Output: Shape>(input: &Value, handler: fn(Input) -> Result<Output, String>) -> Result<Value, String> {",
		'let input = Input::decode(input).map_err(|refused| format!("the input: {refused}"))?;',
		"handler(input).map(|result| result.encode())",
		"}",
		"",
	].join("\n");
}

export function freshFiles(): Fresh {
	const { docs, problems } = exportSchemas();
	const files = new Map<string, string>();
	const outputs = new Map<string, SetSchema>();
	for (const [set, forms] of docs) {
		for (const form of SCHEMA_FORMS) {
			files.set(`${SCHEMA}/${set}.${form}.json`, `${JSON.stringify(forms[form], null, "\t")}\n`);
		}
		problems.push(...formProblems(set, forms.input, forms.output));
		if (!TARGETS.has(set) && !SCHEMA_ONLY.has(set)) problems.push(`set ${set} has no Rust target`);
		outputs.set(set, forms.output);
	}
	for (const set of TARGETS.keys()) if (!docs.has(set)) problems.push(`target ${set} has no set`);
	if (problems.length > 0) return { files, problems };
	try {
		const sets = irOf(outputs);
		for (const [set, ir] of sets) {
			const target = TARGETS.get(set);
			if (target) files.set(target.file, rustfmt(emitRust(ir, sets, TARGETS)));
		}
	} catch (error) {
		return { files, problems: [error instanceof Error ? error.message : String(error)] };
	}
	const prompts = promptFiles(ROOT);
	if (prompts.problems.length > 0) return { files, problems: prompts.problems };
	for (const [file, text] of prompts.typescript) files.set(file, biomeFormat(text));
	for (const [file, text] of prompts.rust) files.set(file, rustfmt(text));
	files.set(DISPATCH_FILE, rustfmt(caseDispatch()));
	files.set("rust/contract/src/issues.rs", rustfmt(issueRust()));
	files.set(
		VERSION_FILE,
		rustfmt(
			`//! Generated by scripts/contract.ts. Do not edit.\n\npub const CONTRACT_VERSION: &str = ${JSON.stringify(CONTRACT_VERSION)};\n`,
		),
	);
	return { files, problems };
}

function schemaFiles(root: string): string[] {
	const at = path.join(root, SCHEMA);
	if (!existsSync(at)) return [];
	return readdirSync(at).map((name) => `${SCHEMA}/${name}`);
}

export function drift(root: string, fresh: Map<string, string>): string[] {
	const problems: string[] = [];
	for (const [file, text] of fresh) {
		const at = path.join(root, file);
		if (!existsSync(at)) problems.push(`${file} is not written`);
		else if (readFileSync(at, "utf8") !== text) problems.push(`${file} differs from a fresh run`);
	}
	for (const file of schemaFiles(root)) if (!fresh.has(file)) problems.push(`${file} has no set`);
	return problems;
}

function refresh(root: string, fresh: Map<string, string>): void {
	for (const file of schemaFiles(root)) if (!fresh.has(file)) rmSync(path.join(root, file));
	for (const [file, text] of fresh) {
		mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
		writeFileSync(path.join(root, file), text);
	}
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const { files, problems } = freshFiles();
	if (problems.length === 0 && process.argv.includes("--refresh")) {
		refresh(ROOT, files);
		console.log(`contract: ${files.size} written`);
	}
	const found = problems.length > 0 ? problems : drift(ROOT, files);
	for (const problem of found) console.error(problem);
	console.log(found.length === 0 ? `contract: ${files.size} files current` : "contract: FAILED");
	process.exit(found.length === 0 ? 0 : 1);
}
