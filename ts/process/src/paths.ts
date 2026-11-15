// Paths as the OS reads them.

import { realpath, stat } from "node:fs/promises";
import path from "node:path";

////////////////////////////////
//  Functions & Helpers

/**
 * Resolves missing tails, rejecting `..` there. Follows links before `..`; `realpath` alone does not guarantee this.
 */
export async function resolvedPath(target: string): Promise<string> {
	if (target === "") throw new Error("an empty path names nothing");
	const whole = path.isAbsolute(target) ? target : `${process.cwd()}${path.sep}${target}`;
	const { root } = path.parse(whole);
	const parts = whole
		.slice(root.length)
		.split(path.sep)
		.filter((part) => part !== "" && part !== ".");
	// Trailing slash requires directory.
	const folderOnly = whole.endsWith(path.sep);
	let held = await realpath(root);
	for (const [at, part] of parts.entries()) {
		if (part === "..") {
			if (!(await stat(held)).isDirectory()) throw new Error(`${held} is not a folder`);
			held = path.dirname(held);
			continue;
		}
		try {
			held = await realpath(path.join(held, part));
		} catch (error) {
			const rest = parts.slice(at);
			if ((error as { code?: string }).code !== "ENOENT" || rest.includes("..")) throw error;
			return path.join(held, ...rest);
		}
	}
	if (folderOnly && !(await stat(held)).isDirectory()) throw new Error(`${held} is not a folder`);
	return held;
}

/** Checks resolved path containment. */
export async function isInside(child: string, parent: string): Promise<boolean> {
	const relative = path.relative(await resolvedPath(parent), await resolvedPath(child));
	return !(relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative));
}
