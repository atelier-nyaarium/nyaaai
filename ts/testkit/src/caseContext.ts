// Case handler context.

import { mkdtempSync, realpathSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { countedIds } from "@nyaaai/contract/id";
import { steppedClock } from "@nyaaai/contract/time";
import type { CaseContext } from "./caseGroup";

////////////////////////////////
//  Constants

/** Rust shares clock values. */
export const CASE_NOW_MS = 1_794_535_200_000;
export const CASE_STEP_MS = 1000;

export const CASE_ID_PREFIX = "case-";

export const TMP_MARK = "$TMP";
export const ROOT_MARK = "$ROOT";

////////////////////////////////
//  Functions & Helpers

function replaced(value: unknown, change: (text: string) => string): unknown {
	if (typeof value === "string") return change(value);
	if (Array.isArray(value)) return value.map((item) => replaced(item, change));
	if (value === null || typeof value !== "object") return value;
	return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, replaced(item, change)]));
}

/** Cleans temp and marks paths. */
export function openContext(root: string): { context: CaseContext; close: (result: unknown) => unknown } {
	let tmp: string | undefined;
	const context: CaseContext = {
		tmp: () => {
			tmp ??= realpathSync(mkdtempSync(path.join(tmpdir(), "nyaaai-case-")));
			return tmp;
		},
		root,
		clock: steppedClock(CASE_NOW_MS, CASE_STEP_MS),
		ids: countedIds(CASE_ID_PREFIX),
	};
	const close = (result: unknown): unknown => {
		const made = tmp;
		if (made !== undefined) rmSync(made, { recursive: true, force: true });
		return replaced(result, (text) => {
			const marked = made === undefined ? text : text.replaceAll(made, TMP_MARK);
			return root === "" ? marked : marked.replaceAll(root, ROOT_MARK);
		});
	};
	return { context, close };
}
