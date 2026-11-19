// Consumer script tests.

import { afterAll, beforeAll, describe, expect, it } from "bun:test";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { git, ROOT } from "../child";
import { assertConsumer, bump, dev, devReset, link } from "../consumer";
import { SNAPSHOT_PREFIX } from "../snapshot";

////////////////////////////////
//  Constants

const LOCAL_FILE = { protocol: "protocol.file.allow=always" };
const TIMEOUT = 120000;
const IMPORT = 'import { rehomed } from "@nyaaai/operator/claude/sessionCut"; console.log(typeof rehomed);';

////////////////////////////////
//  Functions & Helpers

interface World {
	origin: string;
	consumer: string;
	submodule: string;
	local: string;
}

let temp: string;
let source: string;
const made: string[] = [];

function commitAll(cwd: string, message: string): void {
	git(cwd, ["add", "--all"]);
	git(cwd, ["commit", "-m", message]);
}

function identify(cwd: string): void {
	git(cwd, ["config", "user.name", "Test"]);
	git(cwd, ["config", "user.email", "test@example.com"]);
}

function writeJson(file: string, value: unknown): void {
	writeFileSync(file, `${JSON.stringify(value, null, "\t")}\n`);
}

/** Git state is untouched. */
function makeSource(): string {
	const dir = path.join(temp, "source");
	const files = git(ROOT, ["ls-files", "--cached", "--others", "--exclude-standard"]).split("\n");
	for (const file of files) {
		const from = path.join(ROOT, file);
		if (!file || !existsSync(from)) continue;
		cpSync(from, path.join(dir, file), { recursive: true });
	}
	for (const name of ["contract", "testkit"]) {
		const manifest = path.join(dir, "ts", name, "package.json");
		const parsed = JSON.parse(readFileSync(manifest, "utf8")) as { dependencies?: Record<string, string> };
		const own = Object.entries(parsed.dependencies ?? {}).filter(([dep]) => dep.startsWith("@nyaaai/"));
		writeJson(manifest, { ...parsed, dependencies: { ...Object.fromEntries(own), "left-pad": "1.3.0" } });
	}
	git(dir, ["init", "-b", "main"]);
	identify(dir);
	commitAll(dir, "Source");
	return dir;
}

function makeWorld(consumerManifest: Record<string, unknown> = {}): World {
	const dir = mkdtempSync(path.join(temp, "world-"));
	made.push(dir);
	const origin = path.join(dir, "origin.git");
	git(dir, ["clone", "--bare", source, origin]);
	const consumer = path.join(dir, "consumer");
	mkdirSync(consumer);
	git(consumer, ["init", "-b", "main"]);
	identify(consumer);
	writeJson(path.join(consumer, "package.json"), {
		name: "consumer",
		private: true,
		dependencies: { "left-pad": "1.3.0" },
		...consumerManifest,
	});
	git(consumer, ["-c", LOCAL_FILE.protocol, "submodule", "add", origin, "nyaaai"]);
	commitAll(consumer, "Consumer");
	const local = path.join(dir, "local");
	git(dir, ["clone", origin, local]);
	identify(local);
	return { origin, consumer, submodule: path.join(consumer, "nyaaai"), local };
}

function importType(root: string): string {
	const run = Bun.spawnSync([process.execPath, "-e", IMPORT], { cwd: root });
	return run.stdout.toString().trim();
}

function cli(root: string, args: string[], env: Record<string, string> = {}): number {
	return Bun.spawnSync([process.execPath, path.join("nyaaai", "scripts", "consumer.ts"), ...args], {
		cwd: root,
		env: { ...process.env, ...env },
	}).exitCode;
}

function failure(run: () => unknown): string {
	try {
		run();
	} catch (error) {
		return error instanceof Error ? error.message : String(error);
	}
	return "";
}

function treeState(dir: string): string {
	return [
		git(dir, ["rev-parse", "HEAD"]),
		git(dir, ["diff", "--cached"]),
		git(dir, ["status", "--porcelain"]),
		readFileSync(path.join(dir, "README.md"), "utf8"),
	].join("\n--\n");
}

////////////////////////////////
//  Tests

beforeAll(() => {
	temp = mkdtempSync(path.join(tmpdir(), "nyaaai-consumer-"));
	source = makeSource();
}, TIMEOUT);

afterAll(() => {
	rmSync(temp, { recursive: true, force: true });
}, TIMEOUT);

describe("consumer", () => {
	it(
		"links packages so imports resolve",
		() => {
			const all = makeWorld();
			link(all.consumer, all.submodule);
			expect(importType(all.consumer)).toBe("function");

			const some = makeWorld({ nyaaai: { packages: ["contract"] } });
			link(some.consumer, some.submodule);
			expect(existsSync(path.join(some.consumer, "node_modules/@nyaaai/contract"))).toBe(true);
			expect(existsSync(path.join(some.consumer, "node_modules/@nyaaai/operator"))).toBe(false);

			expect(
				failure(() => link(makeWorld({ nyaaai: { packages: ["nope"] } }).consumer, all.submodule)),
			).toContain("nope");

			rmSync(path.join(some.consumer, "node_modules/@nyaaai/contract"));
			mkdirSync(path.join(some.consumer, "node_modules/@nyaaai/contract"));
			writeFileSync(path.join(some.consumer, "node_modules/@nyaaai/contract/keep.txt"), "mine\n");
			expect(failure(() => link(some.consumer, some.submodule))).toContain("not a link");
			expect(existsSync(path.join(some.consumer, "node_modules/@nyaaai/contract/keep.txt"))).toBe(true);

			const empty = mkdtempSync(path.join(temp, "empty-"));
			mkdirSync(path.join(empty, "nyaaai"));
			expect(link(empty, path.join(empty, "nyaaai"))).toEqual([]);
			expect(existsSync(path.join(empty, "node_modules/@nyaaai"))).toBe(false);
		},
		TIMEOUT,
	);

	it(
		"dev snapshots the local checkout and resets",
		() => {
			const world = makeWorld();
			writeFileSync(path.join(world.local, "README.md"), "changed\n");
			writeFileSync(path.join(world.local, "untracked.txt"), "new\n");
			const before = treeState(world.local);

			expect(failure(() => dev(world.consumer, world.submodule, undefined))).toContain("NYAAAI_DIR");
			const recorded = git(world.submodule, ["rev-parse", "HEAD"]);
			dev(world.consumer, world.submodule, world.local);

			expect(git(world.submodule, ["log", "-1", "--format=%s"]).startsWith(SNAPSHOT_PREFIX)).toBe(true);
			expect(readFileSync(path.join(world.submodule, "README.md"), "utf8")).toBe("changed\n");
			expect(readFileSync(path.join(world.submodule, "untracked.txt"), "utf8")).toBe("new\n");
			expect(treeState(world.local)).toBe(before);
			expect(readFileSync(path.join(world.local, "untracked.txt"), "utf8")).toBe("new\n");

			expect(assertConsumer(world.consumer, world.submodule, false).join("\n")).toContain("building against");
			expect(failure(() => assertConsumer(world.consumer, world.submodule, true))).toContain("snapshot");

			devReset(world.consumer, world.submodule);
			expect(git(world.submodule, ["rev-parse", "HEAD"])).toBe(recorded);
			expect(assertConsumer(world.consumer, world.submodule, true)).toEqual([]);
		},
		TIMEOUT,
	);

	it(
		"bump moves the submodule to origin main",
		() => {
			const world = makeWorld();
			writeFileSync(path.join(world.local, "README.md"), "bumped\n");
			commitAll(world.local, "Bump");
			git(world.local, ["push", "origin", "main"]);
			const pushed = git(world.local, ["rev-parse", "HEAD"]);

			bump(world.submodule);
			expect(git(world.submodule, ["rev-parse", "HEAD"])).toBe(pushed);
			link(world.consumer, world.submodule);
			expect(assertConsumer(world.consumer, world.submodule, true)).toEqual([]);
		},
		TIMEOUT,
	);

	it(
		"assert names each failure",
		() => {
			const world = makeWorld();
			link(world.consumer, world.submodule);
			expect(assertConsumer(world.consumer, world.submodule, true)).toEqual([]);

			mkdirSync(path.join(world.submodule, "ts/node_modules"));
			expect(failure(() => assertConsumer(world.consumer, world.submodule, false))).toContain("node_modules");
			rmSync(path.join(world.submodule, "ts/node_modules"), { recursive: true });

			rmSync(path.join(world.consumer, "node_modules/@nyaaai/operator"));
			expect(failure(() => assertConsumer(world.consumer, world.submodule, false))).toContain("operator");
			link(world.consumer, world.submodule);

			const manifest = path.join(world.consumer, "package.json");
			const base = JSON.parse(readFileSync(manifest, "utf8")) as Record<string, unknown>;
			for (const dependencies of [{}, { "left-pad": "^1.3.0" }, { "left-pad": "1.2.0" }]) {
				writeJson(manifest, { ...base, dependencies });
				expect(failure(() => assertConsumer(world.consumer, world.submodule, false))).toContain("left-pad");
			}
			writeJson(manifest, { ...base, dependencies: {}, devDependencies: { "left-pad": "1.3.0" } });
			expect(assertConsumer(world.consumer, world.submodule, false)).toEqual([]);

			const alone = makeWorld({ nyaaai: { packages: ["operator"] } });
			link(alone.consumer, alone.submodule);
			expect(failure(() => assertConsumer(alone.consumer, alone.submodule, false))).toContain("@nyaaai/contract");

			writeFileSync(path.join(alone.consumer, "parity.ts"), "export default {};\n");
			writeJson(path.join(alone.consumer, "package.json"), {
				...JSON.parse(readFileSync(path.join(alone.consumer, "package.json"), "utf8")),
				nyaaai: { packages: ["contract", "operator"] },
			});
			link(alone.consumer, alone.submodule);
			expect(failure(() => assertConsumer(alone.consumer, alone.submodule, false))).toContain("@nyaaai/testkit");

			identify(world.submodule);
			writeFileSync(path.join(world.submodule, "README.md"), "never pushed\n");
			commitAll(world.submodule, "Unpushed");
			expect(failure(() => assertConsumer(world.consumer, world.submodule, true))).toContain("origin/main");
			expect(assertConsumer(world.consumer, world.submodule, false)).toEqual([]);
			git(world.submodule, ["checkout", "--detach", "origin/main"]);

			writeFileSync(path.join(world.submodule, "stray.txt"), "x\n");
			expect(failure(() => assertConsumer(world.consumer, world.submodule, true))).toContain("dirty");
			expect(assertConsumer(world.consumer, world.submodule, false)).toEqual([]);
		},
		TIMEOUT,
	);

	it(
		"assert compares workspace pins, the toolchain pin and the Clippy bans",
		() => {
			const world = makeWorld({ workspaces: ["ts/*"] });
			link(world.consumer, world.submodule);
			mkdirSync(path.join(world.consumer, "ts", "app"), { recursive: true });
			writeJson(path.join(world.consumer, "ts/app/package.json"), { dependencies: { "left-pad": "1.2.0" } });
			expect(failure(() => assertConsumer(world.consumer, world.submodule, false))).toContain(
				"ts/app/package.json",
			);
			writeJson(path.join(world.consumer, "ts/app/package.json"), { dependencies: { "left-pad": "1.3.0" } });
			expect(assertConsumer(world.consumer, world.submodule, false)).toEqual([]);

			mkdirSync(path.join(world.consumer, "rust"));
			writeFileSync(path.join(world.consumer, "rust/Cargo.toml"), "[workspace]\n");
			const refused = () => failure(() => assertConsumer(world.consumer, world.submodule, false));
			expect(refused()).toContain("rust-toolchain.toml is missing");
			expect(refused()).toContain("rust/clippy.toml is missing");

			writeFileSync(path.join(world.consumer, "rust-toolchain.toml"), '[toolchain]\nchannel = "1.0.0"\n');
			expect(refused()).toContain("names channel 1.0.0");
			cpSync(path.join(world.submodule, "rust-toolchain.toml"), path.join(world.consumer, "rust-toolchain.toml"));
			writeFileSync(path.join(world.consumer, "rust/clippy.toml"), "disallowed-methods = []\n");
			expect(refused()).toContain("lacks disallowed-methods path str::trim");
			cpSync(path.join(world.submodule, "rust/clippy.toml"), path.join(world.consumer, "rust/clippy.toml"));
			expect(assertConsumer(world.consumer, world.submodule, false)).toEqual([]);
		},
		TIMEOUT,
	);

	it(
		"runs from a clean clone's root",
		() => {
			const world = makeWorld();
			const clone = path.join(path.dirname(world.consumer), "clone");
			git(path.dirname(world.consumer), [
				"-c",
				LOCAL_FILE.protocol,
				"clone",
				"--recurse-submodules",
				world.consumer,
				clone,
			]);
			expect(cli(clone, ["link"])).toBe(0);
			expect(importType(clone)).toBe("function");
			expect(cli(clone, ["assert", "--package"])).toBe(0);
			expect(cli(clone, ["nonsense"])).toBe(1);

			const recorded = git(path.join(clone, "nyaaai"), ["rev-parse", "HEAD"]);
			expect(cli(clone, ["dev"])).toBe(1);
			expect(cli(clone, ["dev"], { NYAAAI_DIR: path.relative(clone, world.local) })).toBe(0);
			expect(git(path.join(clone, "nyaaai"), ["rev-parse", "HEAD"])).not.toBe(recorded);
			expect(cli(clone, ["dev", "--reset"])).toBe(0);
			expect(git(path.join(clone, "nyaaai"), ["rev-parse", "HEAD"])).toBe(recorded);
		},
		TIMEOUT,
	);
});
