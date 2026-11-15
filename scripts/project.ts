// Project loader for scripts.

import { existsSync } from "node:fs";
import path from "node:path";
import type { Project } from "@nyaaai/testkit/project";

////////////////////////////////
//  Constants

/** Scripts use consumer cwd. */
export const PROJECT_ROOT = process.cwd();

const FILE = "parity.ts";

////////////////////////////////
//  Functions & Helpers

let loaded: Promise<Project> | undefined;

export function loadProject(): Promise<Project> {
	loaded ??= (async () => {
		const file = path.join(PROJECT_ROOT, FILE);
		if (!existsSync(file)) throw new Error(`${PROJECT_ROOT} holds no ${FILE}; run from the project root`);
		const held = (await import(file)) as { default?: Project };
		if (!held.default) throw new Error(`${file} has no default export`);
		return held.default;
	})();
	return loaded;
}
