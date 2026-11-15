// Gate tests.

import { afterAll, beforeAll, describe, expect, it } from "bun:test";
import {
	chmodSync,
	existsSync,
	mkdirSync,
	mkdtempSync,
	readdirSync,
	readFileSync,
	rmSync,
	symlinkSync,
	writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { caseGroup } from "@nyaaai/testkit/caseGroup";
import { nameProblems } from "@nyaaai/testkit/project";
import { rethrown } from "@nyaaai/testkit/rejection";
import { bannedSymbols } from "../bannedSymbols";
import { drift, treeHash } from "../cases";
import { git } from "../child";
import { drift as schemaDrift } from "../contract";
import { type CrateSources, checkLock, type LockedCrate } from "../crates";
import { recorded, removeStale, writeList } from "../manifest";

////////////////////////////////
//  Constants

const DAY_MS = 86_400_000;
const NOW = Date.parse("2026-11-15T00:00:00Z");
const REGISTRY = 'source = "registry+https://github.com/rust-lang/crates.io-index"';
const NAMED: Record<string, string[]> = { "RUSTSEC-2026-0002": ["young"], "RUSTSEC-2026-0003": ["old"] };
const EXCUSED = { name: "young", version: "2.0.0", advisory: "RUSTSEC-2026-0002" };
const LOCK = `
version = 4

[[package]]
name = "old"
version = "1.0.0"
${REGISTRY}

[[package]]
name = "young"
version = "2.0.0"
${REGISTRY}

[[package]]
name = "local"
version = "0.0.0"
`;

////////////////////////////////
//  Functions & Helpers

let temp: string;

function world(name: string, advisories: Record<string, string[]> = {}) {
	const folder = path.join(temp, name);
	mkdirSync(folder);
	const lock = path.join(folder, "Cargo.lock");
	writeFileSync(lock, LOCK);
	const asked: string[] = [];
	const sources: CrateSources = {
		publishedAt: async (crate: LockedCrate) => {
			asked.push(crate.name);
			return new Date(NOW - (crate.name === "young" ? 2 : 30) * DAY_MS).toISOString();
		},
		advisories: async (crates) => crates.map((crate) => advisories[crate.name] ?? []),
		advisoryCrates: async (id) => NAMED[id] ?? [],
		now: () => NOW,
	};
	const exempt = (value: unknown) => writeFileSync(path.join(folder, "crate-exemptions.json"), JSON.stringify(value));
	return { folder, lock, sources, asked, exempt };
}

////////////////////////////////
//  Tests

beforeAll(() => {
	temp = mkdtempSync(path.join(tmpdir(), "nyaaai-gate-"));
});

afterAll(() => {
	rmSync(temp, { recursive: true, force: true });
});

describe("crates", () => {
	it("fails a young registry crate until it is exempt, and asks for each date once", async () => {
		const { folder, lock, sources, asked, exempt } = world("ages");
		const problems = await checkLock(lock, sources);
		expect(problems).toHaveLength(1);
		expect(problems[0]).toContain("young@2.0.0");
		expect(asked).toEqual(["old", "young"]);
		expect(Object.keys(JSON.parse(readFileSync(path.join(folder, "crate-ages.json"), "utf8")))).toEqual([
			"old@1.0.0",
			"young@2.0.0",
		]);

		for (const entry of [
			{ name: "young", version: "2.0.0", advisory: " " },
			{ name: "young", version: "2.0.0", advisory: "reviewed" },
			{ ...EXCUSED, version: "2.0.1" },
			{ ...EXCUSED, name: "old" },
			{ ...EXCUSED, advisory: "RUSTSEC-0000-0000" },
			{ ...EXCUSED, advisory: "RUSTSEC-2026-0003" },
		]) {
			exempt({ young: [entry], advisories: [] });
			expect(await checkLock(lock, sources)).toHaveLength(1);
		}
		exempt({ young: [EXCUSED], advisories: [] });
		expect(await checkLock(lock, sources)).toEqual([]);
		expect(asked).toEqual(["old", "young"]);
	});

	it("fails a crate an advisory names until the advisory is accepted", async () => {
		const { lock, sources, exempt } = world("advisories", { old: ["RUSTSEC-2026-0001"] });
		exempt({ young: [EXCUSED], advisories: [] });
		const problems = await checkLock(lock, sources);
		expect(problems).toHaveLength(1);
		expect(problems[0]).toContain("RUSTSEC-2026-0001");

		exempt({
			young: [EXCUSED],
			advisories: [{ id: "RUSTSEC-2026-0001", reason: "not reachable" }],
		});
		expect(await checkLock(lock, sources)).toEqual([]);
	});

	it("fails a package from a git source or another registry", async () => {
		const { lock, sources, exempt } = world("foreign");
		exempt({ young: [EXCUSED], advisories: [] });
		const foreign = [
			'source = "git+https://example.com/forked.git?rev=abc#abc"',
			'source = "registry+https://example.com/index"',
			'source = "sparse+https://example.com/index/"',
		];
		writeFileSync(
			lock,
			LOCK +
				foreign
					.map((source, at) => `\n[[package]]\nname = "foreign-${at}"\nversion = "1.0.0"\n${source}\n`)
					.join(""),
		);
		const problems = await checkLock(lock, sources);
		expect(problems).toHaveLength(3);
		expect(problems.every((problem) => problem.includes("foreign-"))).toBe(true);

		// Ignore non-package tables.
		writeFileSync(
			lock,
			`${LOCK}\n[[patch.unused]]\nname = "unused"\nversion = "1.0.0"\nsource = "git+https://example.com/x.git"\n`,
		);
		expect(await checkLock(lock, sources)).toEqual([]);
		writeFileSync(lock, LOCK.replace('version = "1.0.0"', "version = '1.0.0'"));
		expect(await checkLock(lock, sources)).toEqual([]);
		for (const broken of [
			LOCK.replace('version = "1.0.0"', "version = 1"),
			"[[package",
			`${LOCK}\n[[package]]\nname = "x"\n`,
		]) {
			writeFileSync(lock, broken);
			expect(await rethrown(checkLock(lock, sources))).toThrow();
		}
	});

	it("writes no age file for a lock with no registry crate", async () => {
		const { folder, lock, sources } = world("local-only");
		writeFileSync(lock, 'version = 4\n\n[[package]]\nname = "local"\nversion = "0.0.0"\n');
		expect(await checkLock(lock, sources)).toEqual([]);
		expect(existsSync(path.join(folder, "crate-ages.json"))).toBe(false);
	});
});

describe("banned symbols", () => {
	it("names the file and line of each, in tracked and untracked files, and skips ignored ones", () => {
		const folder = path.join(temp, "symbols");
		mkdirSync(folder);
		git(folder, ["init", "-b", "main"]);
		writeFileSync(path.join(folder, ".gitignore"), "ignored.txt\n");
		writeFileSync(path.join(folder, "clean.txt"), "plain - text\n");
		writeFileSync(path.join(folder, "dash.txt"), "one\ntwo \u{2014} three\n");
		writeFileSync(path.join(folder, "quote.txt"), "\u{201c}quoted\u{201d}\n\n\u{200b}\n");
		writeFileSync(path.join(folder, "ignored.txt"), "\u{2014}\n");
		git(folder, ["add", "dash.txt"]);
		expect(bannedSymbols(folder).sort()).toEqual(["dash.txt:2", "quote.txt:1", "quote.txt:3"]);

		const each = ["\u{2014}", "\u{2018}", "\u{2019}", "\u{201c}", "\u{201d}", "\u{200b}", "\u{200c}", "\u{200d}"];
		writeFileSync(path.join(folder, "each.txt"), each.map((symbol) => `a${symbol}b\n`).join(""));
		expect(bannedSymbols(folder).filter((hit) => hit.startsWith("each.txt"))).toHaveLength(each.length);
	});
});

describe("generated files", () => {
	it("compares against the manifest, and a refresh removes only what it generated", () => {
		const folder = path.join(temp, "manifest");
		mkdirSync(path.join(folder, "cases", "group"), { recursive: true });
		const put = (file: string, text: string) => writeFileSync(path.join(folder, file), text);
		put("cases/group/current.json", "same\n");
		put("cases/group/stale.json", "old\n");
		put("cases/group/mine.json", "mine\n");
		const fresh = new Map([
			["cases/group/current.json", "same\n"],
			["cases/group/stale.json", "new\n"],
			["cases/group/unwritten.json", "new\n"],
		]);
		// No manifest yet: stale, and a hand-written file is no drift.
		expect(drift(folder, fresh)).toEqual([
			"cases/group/stale.json differs from a fresh run",
			"cases/group/unwritten.json is not written",
			"parity.generated.json is stale",
		]);

		writeList(folder, "cases", [...fresh.keys(), "cases/group/gone.json"]);
		put("cases/group/gone.json", "left\n");
		expect(drift(folder, fresh)).toContain("cases/group/gone.json is generated by nothing");
		removeStale(folder, "cases", fresh.keys());
		expect(existsSync(path.join(folder, "cases/group/gone.json"))).toBe(false);
		expect(existsSync(path.join(folder, "cases/group/mine.json"))).toBe(true);

		put("cases/group/stale.json", "new\n");
		put("cases/group/unwritten.json", "new\n");
		writeList(folder, "cases", fresh.keys());
		expect(drift(folder, fresh)).toEqual([]);

		// Each script rewrites only its own list.
		writeList(folder, "contract", ["schema/a.output.json"]);
		expect(recorded(folder, "cases")).toEqual([...fresh.keys()].sort());
		expect(drift(folder, fresh)).toEqual([]);
		const contract = new Map([["schema/a.output.json", "x"]]);
		mkdirSync(path.join(folder, "schema"));
		put("schema/a.output.json", "x");
		put("schema/other.json", "mine\n");
		expect(schemaDrift(folder, contract)).toEqual([]);
		expect(readFileSync(path.join(folder, "parity.generated.json"), "utf8")).toEndWith("}\n");
	});
});

describe("manifest safety", () => {
	it("refuses paths that leave the project or name more than a generated file", () => {
		const outer = path.join(temp, "safety");
		const folder = path.join(outer, "root");
		mkdirSync(path.join(folder, "schema"), { recursive: true });
		mkdirSync(path.join(outer, "other"));
		writeFileSync(path.join(outer, "other", "victim"), "keep\n");
		symlinkSync(path.join(outer, "other"), path.join(folder, "link"));
		const listing = (value: object) =>
			writeFileSync(path.join(folder, "parity.generated.json"), JSON.stringify(value));
		const refused = (value: object, message: string) => {
			listing(value);
			expect(() => recorded(folder, "contract")).toThrow(message);
		};

		listing({ contract: ["link/victim"] });
		expect(() => removeStale(folder, "contract", [])).toThrow("link");
		expect(existsSync(path.join(outer, "other", "victim"))).toBe(true);

		refused({ contract: ["parity.generated.json"] }, "itself");
		refused({ contract: ["a/x"], cases: ["a/x"] }, "twice");
		for (const entry of [".", "schema/", "a/../b", "./a", "../x"]) refused({ contract: [entry] }, entry);

		refused({ contract: ["a\\b"] }, "backslash");

		// A folder entry is no file, and a name with two leading dots is inside.
		listing({ contract: ["..x", "schema"] });
		expect(recorded(folder, "contract")).toEqual(["..x", "schema"]);
		removeStale(folder, "contract", []);
		expect(existsSync(path.join(folder, "schema"))).toBe(true);

		writeList(folder, "cases", ["a"]);
		expect(readdirSync(folder).filter((name) => name.endsWith(".tmp"))).toEqual([]);
	});
});

describe("project tree check", () => {
	it("sees edits to dirty files and files in untracked folders, not ignored ones", () => {
		const folder = path.join(temp, "tree-hash");
		mkdirSync(folder);
		expect(treeHash(folder)).toBeNull();
		git(folder, ["init", "-b", "main"]);
		writeFileSync(path.join(folder, ".gitignore"), "ignored.txt\n");
		writeFileSync(path.join(folder, "tracked.txt"), "one\n");
		git(folder, ["add", "--all"]);
		writeFileSync(path.join(folder, "tracked.txt"), "two\n");
		mkdirSync(path.join(folder, "loose"));
		writeFileSync(path.join(folder, "loose", "a.txt"), "a\n");
		const before = treeHash(folder);
		writeFileSync(path.join(folder, "ignored.txt"), "x\n");
		expect(treeHash(folder)).toBe(before);
		chmodSync(path.join(folder, "tracked.txt"), 0o755);
		expect(treeHash(folder)).not.toBe(before);
		chmodSync(path.join(folder, "tracked.txt"), 0o644);
		writeFileSync(path.join(folder, "tracked.txt"), "three\n");
		const edited = treeHash(folder);
		expect(edited).not.toBe(before);
		writeFileSync(path.join(folder, "loose", "b.txt"), "b\n");
		expect(treeHash(folder)).not.toBe(edited);
	});
});

describe("case names", () => {
	it("refuses a name that is no plain file name", () => {
		const group = (name: string, key: string, source: string) =>
			caseGroup(name, {
				kinds: { [key]: { context: false } as never },
				sources: [{ kind: `${name}.${key}`, name: source, input: {} }],
			});
		expect(nameProblems({ groups: [group("ok", "kind", "fine.name-1")], runs: [] })).toEqual([]);
		const bad = nameProblems({ groups: [group("../g", "a/b", "x/../../etc")], runs: [] });
		expect(bad).toHaveLength(3);
		expect(nameProblems({ groups: [group("a.b", "k", "n"), group("g", "k", "n.")], runs: [] })).toHaveLength(2);
	});
});
