// Claude Code version lookup.

import { execFile } from "node:child_process";
import { promisify } from "node:util";

////////////////////////////////
//  Constants

const VERSION_WAIT_MS = 15_000;

////////////////////////////////
//  Functions & Helpers

/** First dotted version string. */
export const versionIn = (text: string): string | undefined => /\d+\.\d+\.\d+/.exec(text)?.[0];

export async function installedVersion(): Promise<string> {
	const { stdout } = await promisify(execFile)("claude", ["--version"], { timeout: VERSION_WAIT_MS });
	return versionIn(stdout) ?? stdout.trim();
}
