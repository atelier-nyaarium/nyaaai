// Snapshot working and untracked files.
// Keep checkout files and staging unchanged.
// Keep HEAD unchanged.
//
// Run: bun scripts/snapshot.ts

import { copyFileSync, existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { git, gitOrNull, outliveInterrupts, ROOT } from "./child";

////////////////////////////////
//  Interfaces & Types

export interface Snapshot {
	ref: string;
	sha: string;
	subject: string;
}

////////////////////////////////
//  Constants

/** Keeps the newest snapshot fetchable. */
export const SNAPSHOT_REF = "refs/snapshots/dev";
export const SNAPSHOT_PREFIX = "Dev snapshot of ";

////////////////////////////////
//  Functions & Helpers

export function snapshot(root: string): Snapshot {
	if (gitOrNull(root, ["rev-parse", "--verify", "--quiet", "HEAD"]) === null) {
		throw new Error(`${root} has no commit to snapshot from`);
	}
	const scratch = mkdtempSync(path.join(tmpdir(), "nyaaai-snapshot-"));
	const indexFile = path.join(scratch, "index");
	const env = { GIT_INDEX_FILE: indexFile };
	try {
		// Keep the real index untouched.
		const real = path.resolve(root, git(root, ["rev-parse", "--git-path", "index"]));
		if (existsSync(real)) copyFileSync(real, indexFile);
		git(root, ["add", "--all"], env);
		const tree = git(root, ["write-tree"], env);
		const head = git(root, ["rev-parse", "HEAD"]);
		const branch = gitOrNull(root, ["symbolic-ref", "--short", "HEAD"]) ?? "a detached HEAD";
		const subject = `${SNAPSHOT_PREFIX}${head.slice(0, 7)} on ${branch}`;
		const sha = git(root, ["commit-tree", tree, "-p", head, "-m", subject]);
		git(root, ["update-ref", SNAPSHOT_REF, sha]);
		return { ref: SNAPSHOT_REF, sha, subject };
	} finally {
		rmSync(scratch, { recursive: true, force: true });
	}
}

////////////////////////////////
//  Main

if (import.meta.main) {
	outliveInterrupts();
	try {
		console.log(JSON.stringify(snapshot(process.argv[2] ?? ROOT)));
	} catch (error) {
		console.error(error instanceof Error ? error.message : error);
		process.exit(1);
	}
}
