// Case folder tree.

import { lstatSync, mkdirSync, symlinkSync, writeFileSync } from "node:fs";
import path from "node:path";
import { base64Decode } from "@nyaaai/contract/base64";
import { shapeSet, z } from "@nyaaai/contract/shape";
import { TMP_MARK } from "./caseContext";

////////////////////////////////
//  Shapes

/** Entry fields select file, link, or folder. */
export const TreeEntry = z.object({
	/** Relative tree path. */
	path: z.string(),
	text: z.string().exactOptional(),
	base64: z.string().exactOptional(),
	/** `$TMP` names planted folder. */
	link: z.string().exactOptional(),
});
export type TreeEntry = z.infer<typeof TreeEntry>;

export const treeSet = shapeSet("tree", { TreeEntry });

////////////////////////////////
//  Functions & Helpers

/** Detects path/link escapes. */
function leaves(folder: string, entry: string): boolean {
	const parts = entry.split("/").filter((part) => part !== "" && part !== ".");
	if (entry.startsWith("/") || parts.includes("..")) return true;
	return parts.some((_, at) =>
		lstatSync(path.join(folder, ...parts.slice(0, at + 1)), { throwIfNoEntry: false })?.isSymbolicLink(),
	);
}

/** Parents precede children. */
export function plant(folder: string, entries: TreeEntry[]): void {
	for (const entry of entries) {
		if (leaves(folder, entry.path)) throw new Error(`tree entry ${entry.path} leaves its folder`);
		const at = path.join(folder, entry.path);
		const file = entry.text !== undefined || entry.base64 !== undefined;
		mkdirSync(file || entry.link !== undefined ? path.dirname(at) : at, { recursive: true });
		if (entry.link !== undefined) symlinkSync(entry.link.replaceAll(TMP_MARK, folder), at);
		else if (entry.base64 !== undefined) {
			const bytes = base64Decode(entry.base64);
			if (bytes === undefined) throw new Error("not base64");
			writeFileSync(at, bytes);
		} else if (entry.text !== undefined) writeFileSync(at, entry.text);
	}
}
