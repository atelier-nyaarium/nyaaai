// A later entry's cold catalog and the resolver's answer. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { caseOf } from "./catalogKit";

////////////////////////////////
//  Fixtures

const LUNA = "openai/gpt-6-luna";

const KEY = { kind: "key", key: "fake-key-one" };

const FIRST = { id: "a", kind: "openrouter", label: "First" };
const SECOND = { id: "b", kind: "openrouter", label: "Second" };

const routes = (model: string) => ({ kind: "routes", model, use: "chat" });
const after = (ms: number, model: string, warmed: string[] = []) => ({
	kind: "after",
	ms,
	model,
	use: "chat",
	warmed,
});
const warm = (entry: string) => ({ kind: "warm", entry });

const run = (name: string, input: Record<string, unknown>): CaseSource =>
	caseOf("resolver.routes")(name, {
		entries: [FIRST, SECOND],
		answers: { a: KEY, b: KEY },
		failing: [],
		rows: [LUNA],
		kinds: ["openrouter"],
		listings: {},
		zdrOnly: true,
		...input,
	});

////////////////////////////////
//  Cases

export const resolverColdCases: CaseSource[] = [
	run("cold-a-later-entrys-catalog-holds-the-answer-two-seconds-and-the-next-call-has-both", {
		cold: ["b"],
		steps: [after(2000, LUNA), warm("b"), routes(LUNA)],
	}),
	run("cold-the-first-entrys-catalog-is-waited-for-as-before", {
		cold: ["a"],
		steps: [after(2000, LUNA, ["a"])],
	}),
];
