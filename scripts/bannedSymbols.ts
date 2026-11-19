// Check banned symbols.
// Run from the project root: bun scripts/bannedSymbols.ts

import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { git } from "./child";
import { PROJECT_ROOT } from "./project";

////////////////////////////////
//  Constants

/** Code points that never appear in a file or an escaped case. */
export const BANNED = new Set([0x2014, 0x2018, 0x2019, 0x201c, 0x201d, 0x200b, 0x200c, 0x200d, 0xfeff]);

////////////////////////////////
//  Functions & Helpers

const SKIPPED = new Set(["node_modules", "target"]);

/** Inside a skipped folder or a nested checkout. */
function skipped(root: string, file: string, nested: Map<string, boolean>): boolean {
	const parts = file.split("/").slice(0, -1);
	for (let at = 0; at < parts.length; at += 1) {
		if (SKIPPED.has(parts[at] as string)) return true;
		const folder = parts.slice(0, at + 1).join("/");
		if (!nested.has(folder)) nested.set(folder, existsSync(path.join(root, folder, ".git")));
		if (nested.get(folder)) return true;
	}
	return false;
}

const ATTRIBUTE_BATCH = 200;

/** Tracked files that git marks `linguist-generated`, set or `true`. A failing git skips nothing. */
function generated(root: string): Set<string> {
	const marked = new Set<string>();
	try {
		const tracked = git(root, ["ls-files", "--cached", "-z"])
			.split("\0")
			.filter((file) => file.length > 0);
		for (let at = 0; at < tracked.length; at += ATTRIBUTE_BATCH) {
			const fields = git(root, [
				"check-attr",
				"-z",
				"linguist-generated",
				"--",
				...tracked.slice(at, at + ATTRIBUTE_BATCH),
			]).split("\0");
			for (let field = 0; field + 2 < fields.length; field += 3) {
				if (fields[field + 2] === "set" || fields[field + 2] === "true") marked.add(fields[field] as string);
			}
		}
	} catch {
		return new Set();
	}
	return marked;
}

/** Includes untracked, excludes ignored and tracked files marked generated. */
export function bannedSymbols(root: string): string[] {
	const files = git(root, ["ls-files", "--cached", "--others", "--exclude-standard", "-z"]).split("\0");
	const built = generated(root);
	const hits: string[] = [];
	const nested = new Map<string, boolean>();
	for (const file of files) {
		if (file.length === 0 || built.has(file) || skipped(root, file, nested)) continue;
		let text: string;
		try {
			text = readFileSync(path.join(root, file), "utf8");
		} catch {
			continue;
		}
		text.split("\n").forEach((line, at) => {
			if ([...line].some((char) => BANNED.has(char.codePointAt(0) as number))) hits.push(`${file}:${at + 1}`);
		});
	}
	return hits;
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const hits = bannedSymbols(PROJECT_ROOT);
	for (const hit of hits) console.error(`banned symbol: ${hit}`);
	console.log(hits.length === 0 ? "symbols: none banned" : `symbols: ${hits.length} banned`);
	process.exit(hits.length === 0 ? 0 : 1);
}
