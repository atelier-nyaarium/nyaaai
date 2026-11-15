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
	workspaces?: string[];
}

////////////////////////////////
//  Constants

const SCOPE = "@nyaaai/";

////////////////////////////////
//  Functions & Helpers

function readManifest(file: string): Manifest {
	return JSON.parse(readFileSync(file, "utf8")) as Manifest;
}

function rootManifest(root: string): Manifest {
	const file = path.join(root, "package.json");
	return existsSync(file) ? readManifest(file) : {};
}

/** Checked-out package names. */
function allPackages(submodule: string): string[] {
	const file = path.join(submodule, "ts", "package.json");
	const names = existsSync(file) ? (readManifest(file).workspaces ?? []) : [];
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

export function assertConsumer(root: string, submodule: string, packageMode: boolean): string[] {
	const failures: string[] = [];
	const notes: string[] = [];
	if (!existsSync(path.join(packageDir(submodule, "contract"), "package.json"))) {
		throw new Error(`submodule is not initialized: ${path.join(submodule, "ts/contract/package.json")} is missing`);
	}

	const used = usedPackages(root, submodule);
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
				: dev(root, submodule, process.env.NYAAAI_DIR);
		} else if (command === "bump") lines = bump(submodule);
		else if (command === "assert") lines = assertConsumer(root, submodule, flags.includes("--package"));
		else throw new Error("usage: consumer.ts link | dev [--reset] | bump | assert [--package]");
		for (const line of lines) console.log(line);
	} catch (error) {
		console.error(error instanceof Error ? error.message : error);
		process.exit(1);
	}
}
