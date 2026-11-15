// Check banned symbols.
// Run: bun scripts/bannedSymbols.ts

import { readFileSync } from "node:fs";
import path from "node:path";
import { git, ROOT } from "./child";

////////////////////////////////
//  Constants

const BANNED = /[\u{2014}\u{2018}\u{2019}\u{201c}\u{201d}\u{200b}\u{200c}\u{200d}]/u;

////////////////////////////////
//  Functions & Helpers

/** Includes untracked, excludes ignored. */
export function bannedSymbols(root: string): string[] {
	const files = git(root, ["ls-files", "--cached", "--others", "--exclude-standard", "-z"]).split("\0");
	const hits: string[] = [];
	for (const file of files) {
		if (file.length === 0) continue;
		let text: string;
		try {
			text = readFileSync(path.join(root, file), "utf8");
		} catch {
			continue;
		}
		text.split("\n").forEach((line, at) => {
			if (BANNED.test(line)) hits.push(`${file}:${at + 1}`);
		});
	}
	return hits;
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const hits = bannedSymbols(ROOT);
	for (const hit of hits) console.error(`banned symbol: ${hit}`);
	console.log(hits.length === 0 ? "symbols: none banned" : `symbols: ${hits.length} banned`);
	process.exit(hits.length === 0 ? 0 : 1);
}
