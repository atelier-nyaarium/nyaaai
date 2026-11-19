// The usage meter: what a call's record, bill and stats go to. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { caseOf } from "./catalogKit";

////////////////////////////////
//  Fixtures

const AT = 1_794_528_001_000;

const BASE = {
	tag: { feature: "chat" },
	model: "vendor/model-a",
	at: AT,
	ms: 120,
	cost: { usd: 0.001, estimated: false },
};

const REPLY = { input: 100, output: 10, cost: 0.001 };

const called = (name: string, calls: Array<{ call: object; id: string | null }>): CaseSource =>
	caseOf("meter.called")(name, { calls });

////////////////////////////////
//  Cases

export const meterCases: CaseSource[] = [
	called("a-call-with-an-id-an-entry-and-a-billing-source-reaches-all-three", [
		{
			call: {
				...BASE,
				entry: "go",
				label: "Go",
				usage: { ...REPLY, billingSource: "balance", allowance: 0.5 },
				generationId: "gen-1",
				provider: "Azure",
				finishReason: "stop",
			},
			id: "u1",
		},
	]),
	called("a-call-with-no-id-from-the-sink-is-not-queued-for-stats", [
		{
			call: { ...BASE, entry: "go", usage: { ...REPLY, billingSource: "go" }, generationId: "gen-1" },
			id: null,
		},
	]),
	called("a-call-with-no-generation-id-is-recorded-and-not-queued-for-stats", [
		{ call: { ...BASE, entry: "go", usage: { ...REPLY, billingSource: "go" } }, id: "u1" },
	]),
	called("a-call-with-no-entry-bills-nobody", [
		{ call: { ...BASE, usage: { ...REPLY, billingSource: "balance" }, generationId: "gen-1" }, id: "u1" },
	]),
	called("a-call-with-no-billing-source-bills-nobody", [
		{ call: { ...BASE, entry: "a", usage: REPLY, generationId: "gen-1" }, id: "u1" },
	]),
	called("a-call-with-no-usage-bills-nobody", [
		{ call: { ...BASE, entry: "a", failure: "503", generationId: "gen-1" }, id: "u1" },
	]),
	called("a-billing-source-alone-bills-with-no-allowance-or-cost", [
		{ call: { ...BASE, entry: "go", usage: { input: 1, output: 1, billingSource: "go" } }, id: "u1" },
	]),
	called("an-allowance-and-a-cost-reach-the-bill", [
		{
			call: { ...BASE, entry: "go", usage: { ...REPLY, billingSource: "balance", allowance: 0.25 } },
			id: null,
		},
	]),
	called("a-cancelled-call-and-a-timed-out-call-are-recorded-as-given", [
		{ call: { ...BASE, entry: "a", cancelled: true }, id: "u1" },
		{ call: { ...BASE, entry: "a", failure: "timedOut" }, id: "u2" },
	]),
	called("each-call-takes-the-next-id-and-queues-its-own-stats", [
		{ call: { ...BASE, entry: "a", generationId: "gen-1" }, id: "u1" },
		{ call: { ...BASE, entry: "b", generationId: "gen-2", usage: { ...REPLY, billingSource: "go" } }, id: "u2" },
		{ call: { ...BASE, entry: "b", generationId: "gen-3" }, id: null },
	]),
	called("a-call-keeps-every-optional-field-it-was-given", [
		{
			call: {
				...BASE,
				tag: { feature: "analysis", lane: "title", folder: "/work/app" },
				entry: "a",
				label: "Alpha",
				firstTokenMs: 40,
				usage: {
					input: 100,
					output: 10,
					cached: 20,
					cacheWrite: 5,
					reasoning: 3,
					cost: 0.5,
					upstreamCost: 0.4,
					byok: true,
					billingSource: "balance",
					listCost: 0.6,
					allowance: 0.7,
				},
				provider: "Azure",
				generationId: "gen-1",
				finishReason: "stop",
			},
			id: "u1",
		},
	]),
];
