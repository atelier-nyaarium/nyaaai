// Gate part tests.

import { afterAll, beforeAll, describe, expect, it } from "bun:test";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { bannedSymbols } from "../bannedSymbols";
import { drift } from "../cases";
import { git } from "../child";
import { type CrateSources, checkLock, type LockedCrate } from "../crates";

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
		await expect(checkLock(lock, sources)).rejects.toThrow();
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

describe("case files", () => {
	it("names a file that is unwritten, stale or without a source", () => {
		const folder = path.join(temp, "drift");
		mkdirSync(path.join(folder, "cases", "group"), { recursive: true });
		writeFileSync(path.join(folder, "cases", "group", "current.json"), "same\n");
		writeFileSync(path.join(folder, "cases", "group", "stale.json"), "old\n");
		writeFileSync(path.join(folder, "cases", "group", "orphan.json"), "left\n");
		const fresh = new Map([
			["cases/group/current.json", "same\n"],
			["cases/group/stale.json", "new\n"],
			["cases/group/unwritten.json", "new\n"],
		]);
		const named = drift(folder, fresh).map((problem) => problem.split(" ")[0]);
		expect(named.sort()).toEqual([
			"cases/group/orphan.json",
			"cases/group/stale.json",
			"cases/group/unwritten.json",
		]);
	});
});
