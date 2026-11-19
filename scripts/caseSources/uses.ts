// The standard uses as case inputs: a source names one, a case holds its value.

import { STANDARD_USES } from "@nyaaai/agentic/models/uses";
import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Functions & Helpers

/** A value with each `use` that names a standard use replaced by that use. */
function valued(held: unknown): unknown {
	if (Array.isArray(held)) return held.map(valued);
	if (held === null || typeof held !== "object") return held;
	return Object.fromEntries(
		Object.entries(held).map(([key, each]) => {
			const standard = key === "use" ? STANDARD_USES.find((use) => use.name === each) : undefined;
			return [key, standard ?? valued(each)];
		}),
	);
}

/** The sources, a use named by its standard name read as its value. */
export const withUses = (sources: CaseSource[]): CaseSource[] =>
	sources.map((source) => ({ ...source, input: valued(source.input) }));
