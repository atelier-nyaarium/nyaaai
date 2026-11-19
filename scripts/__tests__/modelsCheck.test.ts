// The curated list held against a made-up catalog; no network.

import { describe, expect, it } from "bun:test";
import type { ModelRow } from "@nyaaai/contract/shapes/model";
import { globMatch, reportOf } from "../modelsCheck";

////////////////////////////////
//  Fixtures

const NOW = 1_790_337_600_000;

const row = (key: string, id: string, family: string, over: Partial<ModelRow> = {}): ModelRow => ({
	key,
	bindings: { openrouter: { id } },
	family,
	label: key,
	reasoning: false,
	schema: false,
	cache: "implicit",
	verified: "2026-01-01",
	verifiedPrice: { prompt: 1, completion: 2 },
	identity: { kind: "curated", key },
	...over,
});

const ROWS = [
	row("lab/glm-5", "lab/glm-5", "lab/glm-*"),
	row("lab/gone", "lab/gone", "lab/gone-*"),
	row("lab/bare", "lab/bare", "lab/bare"),
	row("other/x", "other/x", "other/*-x"),
];

const MODELS = {
	data: [
		{ id: "lab/glm-5", created: 100, pricing: { prompt: "0.000001", completion: "0.000002" } },
		{ id: "lab/glm-6", created: 200, pricing: { prompt: "0.000001", completion: "0.000002" } },
		{ id: "lab/glm-7", created: 300 },
		{ id: "lab/glm-4", created: 50 },
		{ id: "lab/bare", created: 10 },
		{ id: "other/x", created: 10 },
		{ id: "other/new-x", created: 20 },
	],
};

const ZDR = {
	data: [
		{
			model_id: "lab/glm-5",
			tag: "a",
			supported_parameters: ["tools"],
			pricing: { prompt: "0.000001", completion: "0.000002" },
		},
		{ model_id: "lab/glm-6", tag: "a", supported_parameters: ["tools"] },
		{ model_id: "lab/glm-4", tag: "a", supported_parameters: ["tools"] },
		{ model_id: "lab/bare", tag: "a", supported_parameters: ["response_format"] },
		{ model_id: "other/new-x", tag: "b", supported_parameters: ["tools"] },
	],
};

////////////////////////////////
//  Tests

describe("reportOf", () => {
	it("describes every row and finds newer models with a zero-retention host", () => {
		const report = reportOf(MODELS, ZDR, ROWS, NOW);
		expect(report?.lines.map((line) => line.split("  ")[0])).toEqual([
			"lab/glm-5",
			"lab/gone",
			"lab/bare",
			"other/x",
			"lab/glm-5: newer with a zero-retention host: lab/glm-6",
			"lab/gone: its date is unknown, newer models are not listed",
			"other/x: newer with a zero-retention host: other/new-x",
		]);
		expect(report?.lines.map((line) => line.split("  ")[1])).toEqual([
			"ok",
			"retired",
			"noZdrHost",
			"noZdrHost",
			undefined,
			undefined,
			undefined,
		]);
	});

	it("lists no newer model for a row whose date is unknown, and says so", () => {
		const models = {
			data: [{ id: "lab/glm-5" }, { id: "lab/glm-4", created: 50 }, { id: "lab/glm-6", created: 200 }],
		};
		const zdr = {
			data: ["lab/glm-5", "lab/glm-4", "lab/glm-6"].map((model_id) => ({
				model_id,
				tag: "a",
				supported_parameters: ["tools"],
			})),
		};
		const report = reportOf(models, zdr, [ROWS[0] as ModelRow], NOW);
		expect(report?.lines.map((line) => line.split("  ")[0])).toEqual([
			"lab/glm-5",
			"lab/glm-5: its date is unknown, newer models are not listed",
		]);
	});

	it("lists no newer model for a row the catalog lacks, and says so", () => {
		const report = reportOf(MODELS, ZDR, [ROWS[1] as ModelRow], NOW);
		expect(report?.lines.map((line) => line.split("  ")[0])).toEqual([
			"lab/gone",
			"lab/gone: its date is unknown, newer models are not listed",
		]);
		expect(report?.failed).toBe(true);
	});

	it("fails when a curated model is retired or has no host, and passes when none is", () => {
		expect(reportOf(MODELS, ZDR, ROWS, NOW)?.failed).toBe(true);
		expect(reportOf(MODELS, ZDR, [ROWS[0] as ModelRow], NOW)?.failed).toBe(false);
		expect(reportOf(MODELS, ZDR, [ROWS[1] as ModelRow], NOW)?.failed).toBe(true);
		expect(reportOf(MODELS, ZDR, [ROWS[2] as ModelRow], NOW)?.failed).toBe(true);
	});

	it("counts a model with no host or an older one as not newer, and a known one never", () => {
		const report = reportOf(MODELS, ZDR, [ROWS[0] as ModelRow, row("lab/glm-6", "lab/glm-6", "lab/glm-*")], NOW);
		expect(report?.lines.filter((line) => line.includes("newer"))).toEqual([]);
	});

	it("reads no report from a body that is no list", () => {
		expect(reportOf({}, ZDR, ROWS, NOW)).toBeUndefined();
		expect(reportOf(MODELS, null, ROWS, NOW)).toBeUndefined();
		expect(reportOf("x", "y", ROWS, NOW)).toBeUndefined();
	});

	it("prints prices as the catalog gives them, with a removal day", () => {
		const report = reportOf(
			{
				data: [
					{
						id: "lab/glm-5",
						expiration_date: "2026-12-01",
						pricing: { prompt: "0.0000005", completion: "0.0000025" },
					},
				],
			},
			{
				data: [
					{
						model_id: "lab/glm-5",
						tag: "a",
						supported_parameters: ["tools"],
						pricing: { prompt: "0.0000005", completion: "0.0000025" },
					},
				],
			},
			[ROWS[0] as ModelRow],
			NOW,
		);
		expect(report?.lines).toEqual([
			"lab/glm-5  ok  1 hosts  $0.5 in, $2.5 out per M (up); verified $1 in, $2 out  removal 2026-12-01",
			"lab/glm-5: its date is unknown, newer models are not listed",
		]);
	});
});

describe("globMatch", () => {
	const regex = (glob: string) => new RegExp(`^${glob.replace(/[.+?^${}()|[\]\\]/g, "\\$&").replaceAll("*", ".*")}$`);
	const GLOBS = [
		"a",
		"*",
		"a*",
		"*a",
		"a*b",
		"*a*",
		"a*b*c",
		"**",
		"a**b",
		"lab/glm-*",
		"x.y*",
		"a+b*",
		"(a)*[b]",
		"",
	];
	const TEXTS = [
		"",
		"a",
		"b",
		"ab",
		"aab",
		"abc",
		"abbc",
		"acb",
		"a/b",
		"ba",
		"lab/glm-5",
		"lab/glm-",
		"xzy",
		"x.y",
		"x.yz",
		"a+b",
		"a+bc",
		"(a)[b]",
		"(a)x[b]",
	];

	it("matches as the escaped expression it replaces", () => {
		for (const glob of GLOBS)
			for (const text of TEXTS)
				expect([glob, text, globMatch(glob, text)]).toEqual([glob, text, regex(glob).test(text)]);
	});
});
