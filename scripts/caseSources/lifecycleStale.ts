// Requests that meet a change while they fly or wait. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import {
	advance,
	BALANCE,
	failing,
	GO,
	KEY,
	key,
	later,
	NEW_KEY,
	overloaded,
	poolLimit,
	run,
	says,
	send,
} from "./lifecycleKit";

////////////////////////////////
//  Fixtures

const SLOW = { headMs: 100 };

const GO_ALONE = { entries: [GO], keys: { go: KEY } };

const replaced = (id: string) => later(50, { kind: "key", id, answer: NEW_KEY });

////////////////////////////////
//  Cases

export const staleCases: CaseSource[] = [
	run("a-401-after-the-key-was-replaced-leaves-the-new-check-alone", {
		entries: [{ id: "a", kind: "openrouter", label: "Alpha" }],
		keys: { a: KEY },
		exchanges: [{ ...failing(401), ...SLOW }, says("A")],
		steps: [replaced("a"), send("first"), send("next")],
	}),
	run("an-allowance-refusal-after-the-key-was-replaced-opens-no-window", {
		...GO_ALONE,
		exchanges: [{ ...poolLimit(), ...SLOW }, says("Go"), says("Go")],
		steps: [replaced("go"), send("first"), send("next")],
	}),
	run("a-billed-reply-after-the-key-was-replaced-records-no-bill", {
		...GO_ALONE,
		exchanges: [says("Go", BALANCE, SLOW), says("Go")],
		steps: [replaced("go"), send("first"), send("next")],
	}),
	run("a-401-before-the-key-was-replaced-still-rejects-it", {
		entries: [{ id: "a", kind: "openrouter", label: "Alpha" }],
		keys: { a: KEY },
		exchanges: [{ ...failing(401), ...SLOW }],
		steps: [send("first"), key("a", NEW_KEY)],
	}),
	run("a-retry-on-an-earlier-route-skips-a-pause-another-request-started", {
		exchanges: [failing(503), overloaded, says("B"), says("B")],
		steps: [send("second"), send("first"), advance(1000)],
	}),
];
