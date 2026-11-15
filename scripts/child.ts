// Git and process helpers.

import { execFileSync, spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { homedir } from "node:os";
import path from "node:path";

////////////////////////////////
//  Constants

export const ROOT = path.join(import.meta.dirname, "..");

////////////////////////////////
//  Functions & Helpers

const GIT_OVERRIDES = [
	"GIT_DIR",
	"GIT_WORK_TREE",
	"GIT_INDEX_FILE",
	"GIT_COMMON_DIR",
	"GIT_OBJECT_DIRECTORY",
	"GIT_ALTERNATE_OBJECT_DIRECTORIES",
	"GIT_PREFIX",
];

function ownRepository(env: NodeJS.ProcessEnv): NodeJS.ProcessEnv {
	return Object.fromEntries(Object.entries(env).filter(([name]) => !GIT_OVERRIDES.includes(name)));
}

export function git(cwd: string, args: string[], env: Record<string, string> = {}): string {
	try {
		return execFileSync("git", args, {
			cwd,
			encoding: "utf8",
			env: { ...ownRepository(process.env), ...env },
			stdio: ["ignore", "pipe", "pipe"],
		}).trim();
	} catch (error) {
		const stderr = (error as { stderr?: string }).stderr?.trim();
		throw new Error(
			`git ${args.join(" ")} in ${cwd}: ${stderr || (error instanceof Error ? error.message : error)}`,
		);
	}
}

export function gitOrNull(cwd: string, args: string[]): string | null {
	try {
		return git(cwd, args);
	} catch {
		return null;
	}
}

/** PATH precedes CARGO_HOME. */
function rustBin(name: string): string {
	const onPath = spawnSync(name, ["--version"], { stdio: "ignore" }).status === 0;
	if (onPath) return name;
	const home = process.env.CARGO_HOME ?? path.join(homedir(), ".cargo");
	const held = path.join(home, "bin", name);
	if (!existsSync(held)) throw new Error(`${name} is neither on PATH nor under CARGO_HOME`);
	return held;
}

export const cargoBin = (): string => rustBin("cargo");

export const rustfmtBin = (): string => rustBin("rustfmt");

/** Let children handle SIGINT. */
export function outliveInterrupts(): void {
	process.on("SIGINT", () => {});
}
