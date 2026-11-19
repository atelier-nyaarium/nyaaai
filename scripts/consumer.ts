// Consumer commands.
// Run from consumer root: bun <submodule>/scripts/consumer.ts <command>

import { spawnSync } from "node:child_process";
import { existsSync, lstatSync, mkdirSync, readFileSync, realpathSync, rmSync, symlinkSync } from "node:fs";
import path from "node:path";
import { git, gitOrNull } from "./child";
import { SNAPSHOT_PREFIX, type Snapshot } from "./snapshot";

////////////////////////////////
//  Interfaces & Types

interface Manifest {
	dependencies?: Record<string, string>;
	devDependencies?: Record<string, string>;
	nyaaai?: { packages?: unknown };
	workspaces?: string[] | { packages?: string[] };
}

////////////////////////////////
//  Constants

const SCOPE = "@nyaaai/";

/** Packages a parity.ts imports. */
const REQUIRED = ["contract", "testkit", "adapter"];

const CLIPPY = "rust/clippy.toml";
const TOOLCHAIN = "rust-toolchain.toml";

////////////////////////////////
//  Functions & Helpers

function readManifest(file: string): Manifest {
	return JSON.parse(readFileSync(file, "utf8")) as Manifest;
}

function rootManifest(root: string): Manifest {
	const file = path.join(root, "package.json");
	return existsSync(file) ? readManifest(file) : {};
}

const workspacePatterns = (manifest: Manifest): string[] =>
	Array.isArray(manifest.workspaces) ? manifest.workspaces : (manifest.workspaces?.packages ?? []);

/** Checked-out package names. */
function allPackages(submodule: string): string[] {
	const file = path.join(submodule, "ts", "package.json");
	const names = existsSync(file) ? workspacePatterns(readManifest(file)) : [];
	if (names.some((name) => /[*?[\]{}]/.test(name))) throw new Error(`${file} lists a workspace pattern, not a name`);
	return names;
}

export function usedPackages(root: string, submodule: string): string[] {
	const all = allPackages(submodule);
	const listed = rootManifest(root).nyaaai?.packages;
	if (listed === undefined || all.length === 0) return all;
	if (!Array.isArray(listed) || listed.some((name) => typeof name !== "string")) {
		throw new Error('"nyaaai.packages" in package.json must be a list of names');
	}
	const names = listed as string[];
	const unknown = names.filter((name) => !all.includes(name));
	if (unknown.length > 0) throw new Error(`unknown nyaaai package: ${unknown.join(", ")}`);
	return names;
}

function packageDir(submodule: string, name: string): string {
	return path.join(submodule, "ts", name);
}

function linkPath(root: string, name: string): string {
	return path.join(root, "node_modules", "@nyaaai", name);
}

export function link(root: string, submodule: string): string[] {
	const lines: string[] = [];
	for (const name of usedPackages(root, submodule)) {
		const source = packageDir(submodule, name);
		const manifest = path.join(source, "package.json");
		if (!existsSync(manifest)) continue;
		const target = linkPath(root, name);
		mkdirSync(path.dirname(target), { recursive: true });
		// lstat detects broken links.
		const held = lstatSync(target, { throwIfNoEntry: false });
		if (held?.isSymbolicLink()) rmSync(target);
		else if (held) throw new Error(`${target} is not a link; remove it first`);
		symlinkSync(source, target, "dir");
		const version = (readManifest(manifest) as { version?: string }).version ?? "?";
		lines.push(`nyaaai: @nyaaai/${name}@${version} -> ${path.relative(root, source)}`);
	}
	return lines;
}

function submoduleRelative(root: string, submodule: string): string {
	return path.relative(root, submodule).split(path.sep).join("/");
}

/** Resolves local against the consumer root. */
export function dev(root: string, submodule: string, local: string | undefined): string[] {
	if (!local) throw new Error("NYAAAI_DIR is not set: point it at a local NyaaAI checkout");
	const localDir = path.resolve(root, local);
	const run = spawnSync(process.execPath, ["scripts/snapshot.ts"], { cwd: localDir, encoding: "utf8" });
	if (run.status !== 0) {
		throw new Error(`snapshot in ${localDir} failed: ${(run.stderr || run.stdout || "").trim()}`);
	}
	const lastLine = run.stdout.trim().split("\n").pop() ?? "";
	let taken: Snapshot;
	try {
		taken = JSON.parse(lastLine) as Snapshot;
	} catch {
		throw new Error(`snapshot in ${localDir} printed no JSON line`);
	}
	git(submodule, ["fetch", localDir, taken.ref]);
	git(submodule, ["checkout", "--detach", taken.sha]);
	return [`nyaaai: ${taken.sha.slice(0, 7)} ${taken.subject}`, ...link(root, submodule)];
}

export function devReset(root: string, submodule: string): string[] {
	git(root, ["submodule", "update", "--init", "--", submoduleRelative(root, submodule)]);
	return link(root, submodule);
}

export function bump(submodule: string): string[] {
	const before = git(submodule, ["rev-parse", "HEAD"]);
	git(submodule, ["fetch", "origin", "main"]);
	git(submodule, ["checkout", "--detach", "origin/main"]);
	const after = git(submodule, ["rev-parse", "HEAD"]);
	return [`nyaaai: ${before.slice(0, 7)} -> ${after.slice(0, 7)}`];
}

/** The commit the consumer's HEAD or index records for the submodule path, from `ls-tree` or `ls-files` text. */
function recordedCommit(listing: string | null, shaAt: number): string | undefined {
	const [fields] = (listing ?? "").split("\t");
	return fields === undefined || fields === "" ? undefined : fields.split(" ")[shaAt];
}

/** Failures when the submodule is not at the commit the consumer records, in its HEAD and its index. */
function recordedFailures(root: string, submodule: string): string[] {
	const at = submoduleRelative(root, submodule);
	const head = git(submodule, ["rev-parse", "HEAD"]);
	const held = recordedCommit(gitOrNull(root, ["ls-tree", "HEAD", "--", at]), 2);
	const staged = recordedCommit(gitOrNull(root, ["ls-files", "--stage", "--", at]), 1);
	if (held === undefined) return [`the consumer's HEAD records no commit for ${at}`];
	const failures: string[] = [];
	if (held !== head)
		failures.push(
			`submodule HEAD ${head.slice(0, 7)} is not the commit the consumer's HEAD records (${held.slice(0, 7)})`,
		);
	if (staged !== head)
		failures.push(
			`submodule HEAD ${head.slice(0, 7)} is not the commit the consumer's index records (${staged?.slice(0, 7) ?? "none"})`,
		);
	return failures;
}

/** Manifests of the consumer's workspace packages, the root's aside. */
function workspaceManifests(root: string, submodule: string): string[] {
	const skipped = submoduleRelative(root, submodule);
	const found = new Set<string>();
	for (const pattern of workspacePatterns(rootManifest(root))) {
		if (pattern.startsWith("!")) continue;
		for (const file of new Bun.Glob(`${pattern}/package.json`).scanSync({ cwd: root })) {
			if (file.split("/").includes("node_modules") || file.startsWith(`${skipped}/`)) continue;
			found.add(file);
		}
	}
	return [...found].sort();
}

function tomlOf(file: string): Record<string, unknown> {
	return Bun.TOML.parse(readFileSync(file, "utf8")) as Record<string, unknown>;
}

const clippyPaths = (file: string): string[] => {
	const listed = tomlOf(file)["disallowed-methods"];
	if (!Array.isArray(listed)) return [];
	return listed.flatMap((entry: unknown) => {
		const held = typeof entry === "string" ? entry : (entry as { path?: unknown } | null)?.path;
		return typeof held === "string" ? [held] : [];
	});
};

/** The toolchain pin and Clippy bans a consumer's Rust shares. */
function rustFailures(root: string, submodule: string): string[] {
	if (!existsSync(path.join(root, "rust", "Cargo.toml"))) return [];
	const failures: string[] = [];
	const ours = path.join(submodule, TOOLCHAIN);
	const channel = (file: string) => (tomlOf(file)["toolchain"] as { channel?: unknown } | undefined)?.channel;
	const wanted = existsSync(ours) ? channel(ours) : undefined;
	const held = path.join(root, TOOLCHAIN);
	if (!existsSync(held)) failures.push(`${TOOLCHAIN} is missing; NyaaAI pins channel ${wanted}`);
	else if (channel(held) !== wanted) {
		failures.push(`${TOOLCHAIN} names channel ${channel(held)}; NyaaAI pins channel ${wanted}`);
	}
	const bans = clippyPaths(path.join(submodule, CLIPPY));
	const copy = path.join(root, CLIPPY);
	if (!existsSync(copy)) failures.push(`${CLIPPY} is missing; NyaaAI lists ${bans.length} disallowed-methods paths`);
	else {
		const have = clippyPaths(copy);
		for (const ban of bans) {
			if (!have.includes(ban)) failures.push(`${CLIPPY} lacks disallowed-methods path ${ban}; NyaaAI lists it`);
		}
	}
	return failures;
}

export function assertConsumer(root: string, submodule: string, packageMode: boolean): string[] {
	const failures: string[] = [];
	const notes: string[] = [];
	if (!existsSync(path.join(packageDir(submodule, "contract"), "package.json"))) {
		throw new Error(`submodule is not initialized: ${path.join(submodule, "ts/contract/package.json")} is missing`);
	}

	const used = usedPackages(root, submodule);
	if (existsSync(path.join(root, "parity.ts"))) {
		for (const name of REQUIRED) {
			if (!used.includes(name)) failures.push(`parity.ts needs @nyaaai/${name}, which is not used`);
		}
	}
	const folders = [
		submodule,
		path.join(submodule, "ts"),
		...allPackages(submodule).map((n) => packageDir(submodule, n)),
	];
	for (const folder of folders) {
		if (existsSync(path.join(folder, "node_modules"))) {
			failures.push(`node_modules inside the submodule: ${path.join(folder, "node_modules")}`);
		}
	}

	for (const name of used) {
		const target = linkPath(root, name);
		try {
			if (realpathSync(target) !== realpathSync(packageDir(submodule, name))) {
				failures.push(`link @nyaaai/${name} does not point at the submodule`);
			}
		} catch {
			failures.push(`link @nyaaai/${name} is missing or broken`);
		}
	}

	const manifest = rootManifest(root);
	const held = { ...manifest.devDependencies, ...manifest.dependencies };
	for (const name of used) {
		const file = path.join(packageDir(submodule, name), "package.json");
		if (!existsSync(file)) continue;
		for (const [dep, version] of Object.entries(readManifest(file).dependencies ?? {})) {
			if (dep.startsWith(SCOPE)) {
				if (!used.includes(dep.slice(SCOPE.length)))
					failures.push(`@nyaaai/${name} needs ${dep}, which is not used`);
				continue;
			}
			const pinned = held[dep];
			if (pinned !== version) {
				failures.push(
					`dependency ${dep}@${version} of @nyaaai/${name} is ${pinned === undefined ? "not held" : `held as ${pinned}`} by the consumer`,
				);
			}
		}
	}

	const pins = new Map<string, string>();
	for (const name of used) {
		const file = path.join(packageDir(submodule, name), "package.json");
		if (!existsSync(file)) continue;
		for (const [dep, version] of Object.entries(readManifest(file).dependencies ?? {})) {
			if (!dep.startsWith(SCOPE)) pins.set(dep, version);
		}
	}
	for (const file of workspaceManifests(root, submodule)) {
		const own = readManifest(path.join(root, file));
		for (const [dep, version] of Object.entries({ ...own.devDependencies, ...own.dependencies })) {
			const pinned = pins.get(dep);
			if (pinned !== undefined && pinned !== version) {
				failures.push(`${file} pins ${dep} ${version}; NyaaAI pins ${dep} ${pinned}`);
			}
		}
	}
	failures.push(...rustFailures(root, submodule));

	const subject = git(submodule, ["log", "-1", "--format=%s"]);
	const isSnapshot = subject.startsWith(SNAPSHOT_PREFIX);
	if (!packageMode) {
		if (isSnapshot)
			notes.push(`nyaaai: building against ${subject} (${git(submodule, ["rev-parse", "--short", "HEAD"])})`);
	} else {
		if (isSnapshot) failures.push(`submodule is on a dev snapshot: ${subject}`);
		if (gitOrNull(submodule, ["merge-base", "--is-ancestor", "HEAD", "refs/remotes/origin/main"]) === null) {
			failures.push("submodule HEAD is not on origin/main as last fetched");
		}
		if (git(submodule, ["status", "--porcelain"]) !== "") failures.push("submodule working tree is dirty");
		failures.push(...recordedFailures(root, submodule));
	}

	if (failures.length > 0) throw new Error(`nyaaai assert failed: ${failures.join("; ")}`);
	return notes;
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const root = process.cwd();
	const submodule = path.join(import.meta.dirname, "..");
	const [command, ...flags] = process.argv.slice(2);
	try {
		let lines: string[];
		if (command === "link") lines = link(root, submodule);
		else if (command === "dev") {
			lines = flags.includes("--reset")
				? devReset(root, submodule)
				: dev(root, submodule, process.env["NYAAAI_DIR"]);
		} else if (command === "bump") lines = bump(submodule);
		else if (command === "assert") lines = assertConsumer(root, submodule, flags.includes("--package"));
		else throw new Error("usage: consumer.ts link | dev [--reset] | bump | assert [--package]");
		for (const line of lines) console.log(line);
	} catch (error) {
		console.error(error instanceof Error ? error.message : error);
		process.exit(1);
	}
}
