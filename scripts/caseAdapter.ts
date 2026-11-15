// TypeScript case adapter.

import { parseJson } from "@nyaaai/contract/json";
import { decode } from "@nyaaai/contract/shape";
import { openContext } from "@nyaaai/testkit/caseContext";
import { type CaseKind, kindsOf } from "@nyaaai/testkit/caseGroup";
import type { Project } from "@nyaaai/testkit/project";
import { PROJECT_ROOT } from "./project";

export type { CaseSource } from "@nyaaai/testkit/caseGroup";

////////////////////////////////
//  Constants

export const CASE_PREFIX = "case:";

////////////////////////////////
//  Functions & Helpers

/** Context surrounds the handler. */
export async function runKind(held: CaseKind, input: unknown): Promise<unknown> {
	if (!held.context) return (held.run as (input: unknown) => unknown)(input);
	const { context, close } = openContext(PROJECT_ROOT);
	try {
		return close(await held.run(input, context));
	} catch (error) {
		close(undefined);
		throw error;
	}
}

/** Decode both case shapes. */
export async function runCase(project: Project, kind: string, input: unknown): Promise<unknown> {
	const held = kindsOf(project.groups).get(kind);
	if (!held) throw new Error(`unknown kind ${kind}`);
	const given = decode(held.input, parseJson(JSON.stringify(input)));
	if (!given.ok) throw new Error(`${kind}: the input at ${JSON.stringify(given.path)}: ${given.message}`);
	const result = decode(held.result, JSON.parse(JSON.stringify(await runKind(held, given.value))));
	if (!result.ok) throw new Error(`${kind}: the result at ${JSON.stringify(result.path)}: ${result.message}`);
	return result.value;
}
