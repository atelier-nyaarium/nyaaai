// Contract generator tests.

import { afterAll, beforeAll, describe, expect, it } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import type { Json, SetSchema } from "@nyaaai/contract/schemaExport";
import { shapeSets } from "@nyaaai/contract/shape";
import { CASE_KINDS } from "@nyaaai/testkit/caseKinds";
import { contractCases } from "../caseSources/contract";
import { SOURCES } from "../cases";
import { drift } from "../contract";
import { formProblems, irOf } from "../contract/ir";
import { emitRust, type RustTarget } from "../contract/rust";

////////////////////////////////
//  Functions & Helpers

let temp: string;

const doc = ($defs: Record<string, Json>): SetSchema => ({
	$schema: "https://json-schema.org/draft/2020-12/schema",
	$id: "t.output.json",
	"x-contract-version": "0.0.0",
	$defs,
});

const object = (properties: Record<string, Json>, extra: Record<string, Json> = {}): Json => ({
	type: "object",
	properties,
	required: Object.keys(properties),
	additionalProperties: false,
	...extra,
});

const TARGETS = new Map<string, RustTarget>([["t", { crate: "nyaaai_contract", module: "shapes::t", file: "t.rs" }]]);

function rustOf($defs: Record<string, Json>): string {
	const sets = irOf(new Map([["t", doc($defs)]]));
	return emitRust(sets.get("t") as never, sets, TARGETS);
}

const failure = (run: () => unknown): string => {
	try {
		run();
	} catch (error) {
		return error instanceof Error ? error.message : String(error);
	}
	return "";
};

////////////////////////////////
//  Tests

beforeAll(() => {
	temp = mkdtempSync(path.join(tmpdir(), "nyaaai-contract-"));
});

afterAll(() => {
	rmSync(temp, { recursive: true, force: true });
});

describe("the contract generator", () => {
	it("names a generated file that is missing, stale or orphaned", () => {
		mkdirSync(path.join(temp, "schema"));
		writeFileSync(path.join(temp, "schema", "a.output.json"), "old");
		writeFileSync(path.join(temp, "schema", "gone.output.json"), "{}");
		const fresh = new Map([
			["schema/a.output.json", "new"],
			["rust/a.rs", "new"],
		]);
		expect(drift(temp, fresh).map((problem) => problem.split(" ")[0])).toEqual([
			"schema/a.output.json",
			"rust/a.rs",
			"schema/gone.output.json",
		]);
		writeFileSync(path.join(temp, "schema", "a.output.json"), "new");
		rmSync(path.join(temp, "schema", "gone.output.json"));
		mkdirSync(path.join(temp, "rust"));
		writeFileSync(path.join(temp, "rust", "a.rs"), "new");
		expect(drift(temp, fresh)).toEqual([]);
	});

	it("holds a round-trip case for every named shape", () => {
		const cased = new Set(
			contractCases.map(({ input }) => {
				const { set, shape } = input as { set: string; shape: string };
				return `${set}.${shape}`;
			}),
		);
		// Runner-owned shapes.
		const named = shapeSets()
			.filter((set) => set.name !== "cases" && set.name !== "caseFile")
			.flatMap((set) => Object.keys(set.shapes).map((shape) => `${set.name}.${shape}`));
		expect(named.filter((name) => !cased.has(name))).toEqual([]);
	});

	it("holds a case for every function kind", () => {
		const cased = new Set(SOURCES.map((source) => source.kind));
		expect(Object.keys(CASE_KINDS).filter((kind) => !cased.has(kind))).toEqual([]);
	});

	it("refuses a schema outside the dialect", () => {
		const outside: Array<Record<string, Json>> = [
			{ A: { type: "string", pattern: "^a" } },
			{ A: { type: "string", format: "uri" } },
			{ A: { type: "array", prefixItems: [{ type: "string" }] } },
			{ A: { type: "object", properties: {}, required: [] } },
			{ A: { type: "object", propertyNames: { enum: ["a"] }, additionalProperties: {} } },
			{ A: object({ a: { type: "string", default: "x" } }, { required: [] }) },
			{ A: { oneOf: [{ $ref: "#/$defs/B" }] }, B: object({ kind: { type: "string", const: "b" } }) },
			{ A: { oneOf: [{ $ref: "#/$defs/B" }], "x-tag": "kind" }, B: object({ kind: { type: "string" } }) },
			{ A: { $ref: "#/$defs/Missing" } },
		];
		for (const $defs of outside) {
			expect(failure(() => rustOf($defs))).not.toBe("");
		}
	});

	it("lets the two forms differ by defaults alone", () => {
		const output = doc({ A: object({ a: { type: "string", default: "x" }, b: { type: "number" } }) });
		const input = doc({
			A: {
				type: "object",
				properties: { a: { type: "string", default: "x" }, b: { type: "number" } },
				required: ["b"],
			},
		});
		expect(formProblems("t", input, output)).toEqual([]);
		const piped = doc({
			A: { type: "object", properties: { a: { type: "number" }, b: { type: "number" } }, required: ["b"] },
		});
		expect(formProblems("t", piped, output)).toHaveLength(1);
	});

	it("refuses two names that become one in Rust", () => {
		const clashes: Array<Record<string, Json>> = [
			{ A: object({ maxSize: { type: "string" }, max_size: { type: "string" } }) },
			{ A: { type: "string", enum: ["no-rows", "no_rows"] } },
			{ A: object({ inner: object({}) }), AInner: object({}) },
			{ lower: { type: "string" } },
		];
		for (const $defs of clashes) {
			expect(failure(() => rustOf($defs))).not.toBe("");
		}
	});

	it("boxes a reference that would hold itself by value", () => {
		const rust = rustOf({
			Chain: object({ next: { anyOf: [{ $ref: "#/$defs/Chain" }, { type: "null" }] } }),
			Tree: object({ kids: { type: "array", items: { $ref: "#/$defs/Tree" } } }),
		});
		expect(rust).toContain("pub next: Option<Box<Chain>>");
		expect(rust).toContain("pub kids: Vec<Tree>");
	});
});
