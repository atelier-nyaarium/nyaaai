// Claude Code version.

import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { Pattern } from "@nyaaai/contract/pattern";
import { trimText } from "@nyaaai/contract/text";

////////////////////////////////
//  Constants

const VERSION_WAIT_MS = 15_000;

////////////////////////////////
//  Functions & Helpers

const VERSION_PATTERN = Pattern.compile("\\d+\\.\\d+\\.\\d+");

export const versionIn = (text: string): string | undefined => VERSION_PATTERN.find(text)?.text;

export async function installedVersion(): Promise<string> {
	const { stdout } = await promisify(execFile)("claude", ["--version"], { timeout: VERSION_WAIT_MS });
	return versionIn(stdout) ?? trimText(stdout);
}
