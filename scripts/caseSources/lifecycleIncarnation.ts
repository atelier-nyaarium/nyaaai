// Requests that meet a change of the entry they were made for. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import {
	A,
	advance,
	B,
	BALANCE,
	BOX,
	down,
	failing,
	GO,
	KEY,
	LOCAL_ROW,
	later,
	list,
	overloaded,
	plan,
	run,
	says,
	send,
} from "./lifecycleKit";

////////////////////////////////
//  Fixtures

const ALONE = { entries: [A], keys: { a: KEY } };

/** The same entry under another plan: another identity. */
const EDITED = { ...A, plan: "payg" };

const SLOW = { headMs: 100 };

////////////////////////////////
//  Cases

export const incarnationCases: CaseSource[] = [
	// A route held across a change of identity.
	run("a-held-route-of-a-url-entry-sends-nothing-after-its-url-was-edited", {
		entries: [BOX],
		keys: {},
		rows: [LOCAL_ROW],
		exchanges: [down, says("Local")],
		steps: [
			plan("held", LOCAL_ROW),
			list({ ...BOX, url: "http://box.local:11435" }),
			send("late", { model: LOCAL_ROW, plan: "held" }),
		],
	}),
	run("a-held-route-of-a-removed-entry-is-skipped-as-unavailable", {
		exchanges: [says("B")],
		steps: [plan("held"), list(B), send("late", { plan: "held" })],
	}),
	run("a-held-route-of-the-only-entry-removed-and-added-again-ends-unavailable-and-a-new-plan-serves", {
		...ALONE,
		exchanges: [says("A")],
		steps: [plan("held"), list(), list(A), send("late", { plan: "held" }), send("next")],
	}),

	// A request in flight when its entry changes.
	run("a-529-for-a-request-admitted-before-an-identity-change-leaves-the-new-gate-open", {
		...ALONE,
		exchanges: [{ ...overloaded, ...SLOW }, says("A")],
		steps: [later(50, list(EDITED)), send("first"), send("next")],
	}),
	run("a-401-for-a-request-in-flight-while-its-entry-is-removed-and-added-again-leaves-the-new-entry-alone", {
		...ALONE,
		exchanges: [{ ...failing(401), ...SLOW }, says("A"), says("A")],
		steps: [later(30, list()), later(60, list(A)), send("first"), send("next"), send("after")],
	}),
	run("a-reply-for-a-request-in-flight-while-its-entry-is-removed-and-added-again-bills-nothing", {
		...ALONE,
		exchanges: [says("A", undefined, SLOW), says("A")],
		steps: [later(30, list()), later(60, list(A)), send("first"), send("next")],
	}),

	run("a-held-route-after-the-registry-is-disposed-is-skipped-as-unavailable", {
		exchanges: [says("B")],
		steps: [plan("held"), { kind: "dispose" }, send("late", { plan: "held" })],
	}),

	// A bill with no ticket.
	run("a-reply-with-no-gate-for-a-request-in-flight-while-its-entry-is-removed-and-added-again-bills-nothing", {
		entries: [GO],
		keys: { go: KEY },
		gate: false,
		exchanges: [says("Go", BALANCE, SLOW)],
		steps: [later(30, list()), later(60, list(GO)), send("first")],
	}),

	// A waiter at a paused gate.
	run("a-waiter-at-a-paused-gate-ends-unavailable-when-its-entry-changes-identity", {
		...ALONE,
		exchanges: [overloaded, says("A")],
		steps: [send("first"), send("second"), list(EDITED), send("third")],
	}),
	run("a-waiter-admitted-as-a-probe-sends-nothing-when-its-entry-changes-identity-at-that-instant", {
		...ALONE,
		exchanges: [overloaded, says("A")],
		steps: [send("first"), send("second"), later(60_000, list(EDITED)), advance(60_000), send("third")],
	}),
];
