// Provider shape round-trip cases.

import type { CaseSource } from "../caseAdapter";

type Text = string | object | number | boolean | null;

const provider = (shape: string, texts: Text[]): CaseSource => ({
	kind: "contract.roundTrip",
	name: `provider-${shape}`,
	input: {
		set: "provider",
		shape,
		texts: texts.map((text) => (typeof text === "string" ? text : JSON.stringify(text))),
	},
});

const quoted = (...values: string[]): string[] => values.map((value) => JSON.stringify(value));

const entry = { id: "e1", kind: "openrouter", label: "Main" };
const fullEntry = {
	id: "e2",
	kind: "opencode",
	label: "Plan",
	plan: "go",
	url: "http://lab.local:11434",
	extraUsage: true,
};
const line = { kind: "ollama", label: "Ollama", field: "url" };
const view = { id: "e1", kind: "openrouter", label: "Main", status: { kind: "none" } };
const skip = { entry: "e1", label: "Main", reason: "credit" };

export const providerShapeCases: CaseSource[] = [
	provider("ProviderKindName", [
		...quoted("openrouter", "opencode", "ollama", "lmstudio", "llamacpp", "compatible", "other", ""),
		1,
		null,
	]),
	provider("ProviderPlan", [...quoted("go", "payg", "free"), null, 1]),
	provider("ProviderEntry", [
		entry,
		fullEntry,
		{ ...entry, extra: { deep: 1 } },
		{ ...entry, extraUsage: false },
		{ kind: "openrouter", label: "x" },
		{ ...entry, kind: "other" },
		{ ...entry, plan: "free" },
		{ ...entry, extraUsage: "yes" },
		{ ...entry, url: 1 },
		[],
	]),
	provider("EntryFields", [
		{ kind: "llamacpp", label: "Lab" },
		{ kind: "opencode", label: "Plan", plan: "payg", url: "http://lab.local:8080", extraUsage: false },
		entry,
		{ kind: "llamacpp" },
		{ kind: "x", label: "Lab" },
	]),
	provider("KeyState", [
		{ kind: "none" },
		{ kind: "checking" },
		{ kind: "ok" },
		{ kind: "ok", usage: 1.5, limit: 10, models: 3 },
		{ kind: "rejected" },
		{ kind: "unreachable" },
		{ kind: "invalid" },
		{ kind: "none", extra: 1 },
		{ kind: "ok", usage: "1" },
		{ kind: "ok", models: null },
		{ kind: "other" },
		{},
	]),
	provider("KindLine", [
		line,
		{ ...line, kind: "openrouter", field: "key" },
		{ ...line, plan: "go", url: "http://lab.local:11434", optionalKey: true },
		{ ...line, extra: 1 },
		{ ...line, optionalKey: false },
		{ ...line, field: "path" },
		{ kind: "ollama", field: "url" },
	]),
	provider("ProviderView", [
		view,
		{
			...view,
			plan: "go",
			url: "http://lab.local:11434",
			status: { kind: "ok", usage: 2, limit: 5, models: 4 },
			facts: "Billed 1.00",
			balance: true,
			extraUsage: false,
		},
		{ ...view, extra: [1] },
		{ ...view, status: undefined },
		{ ...view, status: { kind: "other" } },
		{ ...view, balance: false },
		{ ...view, facts: 1 },
		{ id: "e1" },
	]),
	provider("KeyReach", [...quoted("ok", "keyMissing", "keyRejected", "unreachable", "checking"), null]),
	provider("Limits", [
		{},
		{ limit: 5, reset: "2026-11-01T00:00:00Z" },
		{ limit: 5, extra: 1 },
		{ limit: "5" },
		{ reset: 1 },
		[],
	]),
	provider("Billed", [
		{ billing: "credit" },
		{ billing: "credit", allowance: 0.5, cost: 0.25 },
		{ billing: "credit", extra: 1 },
		{ allowance: 1 },
		{ billing: "credit", cost: "1" },
	]),
	provider("SkipReason", [
		...quoted(
			"keyMissing",
			"keyRejected",
			"credit",
			"allowance",
			"model",
			"overloaded",
			"unavailable",
			"network",
			"paused",
			"other",
		),
		1,
	]),
	provider("Skip", [skip, { ...skip, extra: 1 }, { ...skip, reason: "other" }, { entry: "e1", label: "Main" }, []]),
	provider("Passed", [
		{ ...skip, before: 0 },
		{ ...skip, before: 2, extra: 1 },
		skip,
		{ ...skip, before: "1" },
		{ ...skip, reason: "other", before: 1 },
	]),
	provider("NoticeKind", [
		...quoted(
			"keyMissing",
			"keyRejected",
			"spendLimit",
			"refused",
			"noZdrHost",
			"modelRetired",
			"modelUnavailable",
			"unreachable",
			"badRequest",
			"retriesExhausted",
			"providerOverloaded",
			"streamFailed",
			"outputCut",
			"noModel",
			"historyDropped",
			"other",
		),
		null,
	]),
	provider("Disposition", [
		{ act: "retry" },
		{ act: "retry", extra: 1 },
		{ act: "fallThrough", notice: "keyMissing" },
		{ act: "stop", notice: "badRequest" },
		{ act: "stop", notice: "other" },
		{ act: "stop" },
		{ act: "wait", notice: "keyMissing" },
		{ notice: "keyMissing" },
		{},
	]),
];

export { quoted };
