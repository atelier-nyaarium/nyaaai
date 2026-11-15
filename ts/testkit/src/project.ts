// Parity project setup.

import type { Command } from "@nyaaai/adapter/runtime";
import { Pattern } from "@nyaaai/contract/pattern";
import { shapeSet } from "@nyaaai/contract/shape";
import "@nyaaai/contract/shapes/frames";
import "./caseFile";
import "./tree";
import type { RunCase } from "./caseFile";
import { type CaseGroup, caseShapes } from "./caseGroup";

////////////////////////////////
//  Interfaces & Types

export interface RustTarget {
	crate: string;
	module: string;
	file: string;
	/** Skips named normalization. */
	bare?: true;
}

export type RunSource = Omit<RunCase, "format" | "expected">;

export interface Project {
	/** Sets register when imported. */
	sets: Record<string, RustTarget>;
	/** External sets. Null means no Rust. */
	external: Record<string, RustTarget | null>;
	/** Schemas only, no Rust. */
	schemaOnly: string[];
	groups: CaseGroup[];
	runs: RunSource[];
	/** Generated case targets. */
	cases: { crate: string; module: string; dir: string };
	/** Extra TypeScript commands. */
	commands: ReadonlyMap<string, Command>;
	rustAdapter: { build: string[]; run: string[] };
	/** Generated files to format. */
	generated: Array<() => Map<string, string>>;
	/** Command that must pass before a refresh. Absent: none. */
	tests?: string[];
	/** Prompt folder under the root. Absent: no prompts. */
	prompts?: string;
	/** Round-trip case kind the sampler runs. Absent: none. */
	roundTrip?: string;
}

/** `external` defaults to the unlisted NyaaAI sets. */
export type ProjectParts = Omit<Project, "external"> & { external?: Project["external"] };

////////////////////////////////
//  Constants

/** NyaaAI sets for consumers. */
export const NYAAAI_SETS = {
	frames: { crate: "nyaaai_contract", module: "shapes::frames", file: "rust/contract/src/shapes/frames.rs" },
	tree: { crate: "nyaaai_testkit", module: "tree_shapes", file: "rust/testkit/src/tree_shapes.rs" },
	caseFile: null,
} satisfies Record<string, RustTarget | null>;

export const CASES_SET = "cases";

/** Group names and kind keys. */
const IDENTIFIER = Pattern.compile("^[A-Za-z][A-Za-z0-9]*$");

/** Case and run names become file names. */
const CASE_NAME = Pattern.compile("^[A-Za-z0-9]+(?:[._-][A-Za-z0-9]+)*$");

////////////////////////////////
//  Functions & Helpers

/** Registers case shapes. */
export function project(parts: ProjectParts): Project {
	shapeSet(CASES_SET, caseShapes(parts.groups));
	const listed = new Set([...Object.keys(parts.sets), ...parts.schemaOnly]);
	const defaults = Object.entries(NYAAAI_SETS).filter(([set]) => !listed.has(set));
	return { ...parts, external: { ...Object.fromEntries(defaults), ...parts.external } };
}

/** Names that cannot be file names. */
export function nameProblems(parts: Pick<Project, "groups" | "runs">): string[] {
	const problems: string[] = [];
	const check = (what: string, name: string, form: Pattern = CASE_NAME): void => {
		if (!form.test(name)) problems.push(`${what} ${JSON.stringify(name)} is not a plain name`);
	};
	for (const group of parts.groups) {
		check("group", group.name, IDENTIFIER);
		for (const key of Object.keys(group.kinds)) check(`kind of ${group.name}`, key, IDENTIFIER);
		for (const source of group.sources) check(`case of ${source.kind}`, source.name);
	}
	for (const run of parts.runs) {
		check("run group", run.kind.split(".")[0] as string, IDENTIFIER);
		check(`run of ${run.kind}`, run.name);
	}
	return problems;
}
