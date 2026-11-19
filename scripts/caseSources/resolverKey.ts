// A route's key, what a key answer reaches, and what identifies a route. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { caseOf } from "./catalogKit";

////////////////////////////////
//  Fixtures

const KEY = { kind: "key", key: "key-0123456789abcdef" };
const OTHER_KEY = { kind: "key", key: "key-fedcba9876543210" };

const LAB = { id: "lab", kind: "ollama", label: "Lab", url: "http://lab.local:11434" };
const MOVED = { ...LAB, url: "http://lab.local:9999" };
const FIRST = { id: "a", kind: "openrouter", label: "First" };
const SECOND = { id: "b", kind: "openrouter", label: "Second" };
const GO = { id: "go", kind: "opencode", plan: "go", label: "Go" };

const ANSWERS = [KEY, { kind: "none" }, { kind: "refused" }, { kind: "unready" }];

////////////////////////////////
//  Functions & Helpers

const key = caseOf("resolver.keySource");
const reach = caseOf("resolver.reach");
const routeKey = caseOf("resolver.routeKey");

const get = { kind: "get" };
const list = (...entries: unknown[]) => ({ kind: "list", entries });
const answer = (id: string, held: unknown) => ({ kind: "answer", id, answer: held });

////////////////////////////////
//  Cases

export const resolverKeyCases: CaseSource[] = [
	// A key stays with its URL.
	key("key-the-entry-as-listed-answers-its-key", { entry: LAB, list: [LAB], answers: { lab: KEY }, steps: [get] }),
	key("key-a-moved-entry-answers-none", { entry: LAB, list: [MOVED], answers: { lab: KEY }, steps: [get] }),
	key("key-an-entry-that-moves-and-comes-back", {
		entry: LAB,
		list: [LAB],
		answers: { lab: KEY },
		steps: [get, list(MOVED), get, list(LAB), get],
	}),
	key("key-a-removed-url-entry-answers-none", {
		entry: LAB,
		list: [LAB],
		answers: { lab: KEY },
		steps: [get, list(), get],
	}),
	key("key-a-removed-key-entry-has-no-url-to-differ", {
		entry: FIRST,
		list: [FIRST],
		answers: { a: KEY },
		steps: [get, list(), get],
	}),
	key("key-another-entry-with-the-id-of-a-different-url", {
		entry: LAB,
		list: [{ ...LAB, kind: "lmstudio" }],
		answers: { lab: KEY },
		steps: [get],
	}),
	key("key-a-url-gained-by-the-entry", {
		entry: FIRST,
		list: [{ ...FIRST, url: "http://lab.local:1" }],
		answers: { a: KEY },
		steps: [get],
	}),
	key("key-the-url-text-must-match-exactly", {
		entry: LAB,
		list: [{ ...LAB, url: "http://lab.local:11434/" }],
		answers: { lab: KEY },
		steps: [get],
	}),
	key("key-a-plan-change-keeps-the-key-for-the-same-url", {
		entry: GO,
		list: [{ ...GO, plan: "payg" }],
		answers: { go: KEY },
		steps: [get],
	}),
	key("key-other-entries-are-not-read", {
		entry: FIRST,
		list: [SECOND, FIRST],
		answers: { a: KEY, b: OTHER_KEY },
		steps: [get],
	}),

	// What the host's answer is.
	...ANSWERS.map((held) =>
		key(`key-answer-${held.kind}`, { entry: FIRST, list: [FIRST], answers: { a: held }, steps: [get] }),
	),
	key("key-an-entry-with-no-answer-has-none", { entry: FIRST, list: [FIRST], answers: {}, steps: [get] }),
	key("key-the-answer-is-read-at-each-request", {
		entry: FIRST,
		list: [FIRST],
		answers: { a: KEY },
		steps: [get, answer("a", OTHER_KEY), get, answer("a", { kind: "none" }), get, answer("a", KEY), get],
	}),
	key("key-the-key-text-is-given-as-it-is", {
		entry: FIRST,
		list: [FIRST],
		answers: { a: { kind: "key", key: "  key-with-blanks  " } },
		steps: [get],
	}),
	key("key-an-empty-key-is-a-key", {
		entry: FIRST,
		list: [FIRST],
		answers: { a: { kind: "key", key: "" } },
		steps: [get],
	}),

	// The reach.
	reach("reach-every-answer-for-a-key-kind-and-a-url-kind", {
		rows: ["key", "url"].flatMap((field) => ANSWERS.map((held) => ({ field, answer: held }))),
	}),

	// What identifies a route.
	routeKey("route-key-of-every-field", {
		entries: [FIRST, GO, LAB, { ...GO, plan: "payg" }, { ...LAB, label: "Other" }, { ...FIRST, label: "Renamed" }],
	}),
	routeKey("route-key-no-two-fields-run-together", {
		entries: [
			{ id: "a", kind: "opencode", plan: "go", label: "x" },
			{ id: "a", kind: "ollama", label: "x", url: "go" },
			{ id: "a", kind: "ollama", label: "x", url: "go" },
			{ id: "ago", kind: "openrouter", label: "x" },
		],
	}),
];
