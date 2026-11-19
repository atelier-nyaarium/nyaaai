// OpenRouter join cases, and the number and date readings. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { HOUR, type Json, NOW } from "./catalogKit";

////////////////////////////////
//  Fixtures

const LUNA = "openai/gpt-6-luna";
const OPUS = "anthropic/claude-opus-5.5";
const SOL = "openai/gpt-6.1-sol";
const LARGE = "mistralai/mistral-large-2512";
const FLASH = "google/gemini-3.8-flash";
const KIMI = "moonshotai/kimi-k3";

const ROWS = [LUNA, OPUS, SOL, LARGE, FLASH, KIMI];

const PARAMS = ["tools", "reasoning", "response_format", "structured_outputs"];

const MODELS_BODY: Json = {
	data: [
		{
			id: LUNA,
			context_length: 1_000_000,
			expiration_date: null,
			pricing: {
				prompt: "0.00000012",
				completion: "0.0000006",
				input_cache_read: "0.000000012",
				overrides: [
					{
						min_prompt_tokens: 250_000,
						prompt: "0.00000024",
						completion: "0.0000009",
						input_cache_read: "0.000000024",
					},
				],
			},
			reasoning: { supported_efforts: ["max", "xhigh", "high", "medium", "low", "none"] },
		},
		{
			id: OPUS,
			expiration_date: "2026-10-01",
			pricing: { prompt: "0.000005", completion: "0.000025" },
		},
		{ id: SOL, expiration_date: "2026-09-01" },
		{ id: LARGE, pricing: { prompt: "0.0000004", completion: "0.0000012" } },
		{ id: FLASH, reasoning: { supported_efforts: ["high", "low"] } },
		{ unexpected: true },
	],
};

const ZDR_BODY: Json = {
	data: [
		{
			model_id: LUNA,
			tag: "alpha",
			context_length: 1_000_000,
			max_prompt_tokens: 900_000,
			supported_parameters: PARAMS,
			pricing: { prompt: "0.00000012", completion: "0.0000006" },
		},
		{
			model_id: LUNA,
			tag: "alpha/eu",
			context_length: 1_000_000,
			max_prompt_tokens: null,
			supported_parameters: PARAMS,
			pricing: { prompt: "0.00000013", completion: "0.00000066" },
		},
		{
			model_id: LUNA,
			tag: "cheap-no-reasoning",
			supported_parameters: ["tools"],
			pricing: { prompt: "0.00000001", completion: "0.00000001" },
		},
		{
			model_id: OPUS,
			tag: "beta",
			context_length: 900_000,
			supported_parameters: PARAMS,
			pricing: { prompt: "0.0000055", completion: "0.00003" },
		},
		{
			model_id: LARGE,
			tag: "gamma",
			context_length: 260_000,
			supported_parameters: ["tools"],
			pricing: { prompt: "0.0000004", completion: "0.0000012" },
		},
		{ model_id: FLASH, tag: "delta", supported_parameters: ["reasoning"] },
	],
};

////////////////////////////////
//  Functions & Helpers

const pass = (use: string | object, zdrOnly: boolean) => ({ use, zdrOnly });

const join = (
	name: string,
	rows: string[],
	models: unknown,
	zdr: unknown,
	passes: Array<{ use: string | object; zdrOnly: boolean }> = [pass("chat", true)],
	nowMs = NOW,
): CaseSource => ({ kind: "catalog.openrouterJoin", name, input: { rows, models, zdr, nowMs, passes } });

const listing = (...items: Json[]): Json => ({ data: items });

/** A model of the one row, with this much of the listing. */
const one = (extra: Record<string, unknown>): Json => ({ id: LUNA, ...extra }) as Json;

/** A host of the one row. */
const host = (extra: Record<string, unknown>): Json =>
	({ model_id: LUNA, supported_parameters: ["tools", "reasoning"], ...extra }) as Json;

const PRICES = { prompt: "0.000001", completion: "0.000002" };

const perMillionCase = (name: string, ...values: unknown[]): CaseSource => ({
	kind: "catalog.perMillion",
	name,
	input: { values },
});

const expiryCase = (name: string, ...texts: string[]): CaseSource => ({
	kind: "catalog.expiry",
	name,
	input: { texts },
});

type Host = { params: string[] | null };

const standingCase = (
	name: string,
	rows: Array<{ expiration: string | null; hosts: Host[]; needs: Array<string | string[]>; nowMs: number }>,
): CaseSource => ({ kind: "catalog.standing", name, input: { rows } });

////////////////////////////////
//  Cases

export const catalogJoinCases: CaseSource[] = [
	join("join-live-catalog-and-usable-hosts", ROWS, MODELS_BODY, ZDR_BODY),
	join("join-live-catalog-listing-hosts-added", ROWS, MODELS_BODY, ZDR_BODY, [pass("chat", false)]),
	join(
		"join-by-use",
		[LUNA, LARGE],
		MODELS_BODY,
		{
			data: [
				{
					model_id: LUNA,
					tag: "schema",
					supported_parameters: ["reasoning", ...PARAMS.slice(2)],
					pricing: PRICES,
				},
				{ model_id: LUNA, tag: "tools", supported_parameters: ["tools", "reasoning"], pricing: PRICES },
				{
					model_id: LUNA,
					tag: "agentic",
					supported_parameters: ["tools", "reasoning", ...PARAMS.slice(2)],
					pricing: PRICES,
				},
				{ model_id: LARGE, tag: "tools", supported_parameters: ["tools"], pricing: PRICES },
			],
		},
		[pass("chat", true), pass("compaction", true), pass("analysis", true)],
	),
	join(
		"join-zdr-off-adds-the-listing-host",
		[FLASH],
		listing({
			id: FLASH,
			context_length: 1_000_000,
			pricing: { prompt: "0.0000003", completion: "0.0000025" },
			supported_parameters: ["tools", "reasoning"],
		}),
		ZDR_BODY,
		[pass("chat", true), pass("chat", false)],
	),
	join(
		"join-zdr-off-keeps-zdr-hosts-and-the-listing-host",
		[LUNA],
		listing(
			one({
				context_length: 500_000,
				pricing: { prompt: "0.0000002", completion: "0.000001" },
				supported_parameters: PARAMS,
			}),
		),
		listing(host({ tag: "alpha", supported_parameters: PARAMS, pricing: PRICES })),
		[pass("chat", false)],
	),
	join(
		"join-zdr-off-marks-data-kept-when-a-host-is-not-zdr-and-zdr-on-does-not",
		[LUNA],
		listing(
			one({
				context_length: 500_000,
				pricing: { prompt: "0.0000002", completion: "0.000001" },
				supported_parameters: PARAMS,
			}),
		),
		listing(host({ tag: "alpha", supported_parameters: PARAMS, pricing: PRICES })),
		[pass("chat", true), pass("chat", false)],
	),
	join(
		"join-zdr-off-leaves-data-kept-unset-when-every-usable-host-is-zdr",
		[LUNA],
		listing(one({ context_length: 500_000, supported_parameters: [] })),
		listing(host({ tag: "alpha", supported_parameters: PARAMS, pricing: PRICES })),
		[pass("chat", true), pass("chat", false)],
	),
	join(
		"join-drift-in-optional-fields-reads-as-absent",
		[LUNA],
		listing(
			one({
				context_length: "big",
				pricing: { prompt: "0.0000001", completion: "0.0000005", overrides: [{ prompt: "x" }] },
				reasoning: { supported_efforts: "high" },
			}),
		),
		ZDR_BODY,
	),
	join(
		"join-cache-write-1h-priced",
		[LUNA],
		listing(one({ pricing: { ...PRICES, input_cache_write_1h: "0.0000002" } })),
		ZDR_BODY,
	),
	join(
		"join-cache-write-1h-priced-at-zero",
		[LUNA],
		listing(one({ pricing: { ...PRICES, input_cache_write_1h: "0" } })),
		ZDR_BODY,
	),
	join("join-cache-write-1h-absent", [LUNA], listing(one({ pricing: PRICES })), ZDR_BODY),
	join(
		"join-cache-write-1h-not-a-price",
		[LUNA],
		listing(one({ pricing: { ...PRICES, input_cache_write_1h: "free" } })),
		ZDR_BODY,
	),
	join(
		"join-cache-write-tiers",
		[LUNA],
		listing(
			one({
				pricing: {
					...PRICES,
					input_cache_write: "0.000000125",
					input_cache_write_1h: "0.0000002",
					overrides: [{ min_prompt_tokens: 1000, prompt: "0.0000015" }],
				},
			}),
		),
		ZDR_BODY,
	),
	join("join-models-unreadable-leaves-every-row-unknown", ROWS, { error: "down" }, ZDR_BODY),
	join("join-hosts-unreadable-leaves-every-row-unknown", ROWS, MODELS_BODY, { error: "down" }),
	join("join-both-unreadable", ROWS, null, "x", [pass("chat", true), pass("chat", false)]),
	join("join-models-empty-list-retires-every-row", ROWS, listing(), ZDR_BODY),
	join("join-row-not-listed-is-retired", [KIMI, LUNA], MODELS_BODY, ZDR_BODY),
	join(
		"join-removal-day-boundary",
		[LUNA, OPUS, SOL],
		listing(
			{ id: LUNA, expiration_date: "2026-09-25" },
			{ id: OPUS, expiration_date: "2026-09-26" },
			{ id: SOL, expiration_date: "2026-09-25T12:00:00Z" },
		),
		{
			data: ["a", "b", "c"].map((_tag, at) => ({
				model_id: [LUNA, OPUS, SOL][at],
				tag: "h",
				supported_parameters: PARAMS,
			})),
		},
		[pass("chat", true)],
	),
	join("join-removal-just-before-now", [LUNA], listing({ id: LUNA, expiration_date: "2026-09-25T11:59:59Z" }), {
		data: [host({ tag: "h" })],
	}),
	join(
		"join-removal-with-offset",
		[LUNA, OPUS],
		listing(
			{ id: LUNA, expiration_date: "2026-09-25T13:30:00+02:00" },
			{ id: OPUS, expiration_date: "2026-09-25T13:30:00-02:00" },
		),
		{ data: [host({ tag: "h" }), host({ model_id: OPUS, tag: "h" })] },
	),
	join(
		"join-removal-not-a-date-never-retires",
		[LUNA, OPUS, SOL],
		listing(
			{ id: LUNA, expiration_date: "soon" },
			{ id: OPUS, expiration_date: "2026-13-40" },
			{ id: SOL, expiration_date: "" },
		),
		{ data: [host({ tag: "h" }), host({ model_id: OPUS, tag: "h" }), host({ model_id: SOL, tag: "h" })] },
	),
	join(
		"join-removal-spellings-the-primitive-refuses-never-retire",
		[LUNA, OPUS, SOL, LARGE],
		listing(
			{ id: LUNA, expiration_date: "2026-09" },
			{ id: OPUS, expiration_date: "2026" },
			{ id: SOL, expiration_date: "2026-09-25 00:00:00" },
			{ id: LARGE, expiration_date: "Sep 25 2026" },
		),
		{
			data: [
				host({ tag: "h" }),
				host({ model_id: OPUS, tag: "h" }),
				host({ model_id: SOL, tag: "h" }),
				host({ model_id: LARGE, tag: "h" }),
			],
		},
	),
	join("join-host-names", [LUNA], listing(one({})), {
		data: [
			host({ tag: "tagged", provider_name: "Named" }),
			host({ provider_name: "Named only" }),
			host({ tag: "", provider_name: "Empty tag" }),
			host({ tag: "" }),
			host({}),
		],
	}),
	join("join-prompt-limit-first-known-of-prompt-context-listing", [LUNA], listing(one({ context_length: 700 })), {
		data: [
			host({ tag: "a", max_prompt_tokens: 500, context_length: 600 }),
			host({ tag: "b", context_length: 400 }),
			host({ tag: "c" }),
		],
	}),
	join("join-prompt-limit-from-the-listing-alone", [LUNA], listing(one({ context_length: 700 })), {
		data: [host({ tag: "a" })],
	}),
	join("join-no-limit-anywhere", [LUNA], listing(one({})), { data: [host({ tag: "a" })] }),
	join("join-price-ranges-over-hosts", [LUNA], listing(one({ pricing: PRICES })), {
		data: [
			host({ tag: "a", pricing: { prompt: "0.000003", completion: "0.000004" } }),
			host({ tag: "b", pricing: { prompt: "0.000001", completion: "0.000009" } }),
			host({ tag: "c", pricing: { prompt: "oops" } }),
			host({ tag: "d" }),
		],
	}),
	join("join-price-needs-both-ranges", [LUNA], listing(one({ pricing: PRICES })), {
		data: [host({ tag: "a", pricing: { prompt: "0.000001" } })],
	}),
	join(
		"join-tiers-sorted-and-inherit",
		[LUNA],
		listing(
			one({
				pricing: {
					prompt: "0.000001",
					completion: "0.000002",
					input_cache_read: "0.0000001",
					input_cache_write: "0.00000125",
					overrides: [
						{ min_prompt_tokens: 300_000, prompt: "0.000003" },
						{ min_prompt_tokens: 100_000, completion: "0.000004", input_cache_read: "0.0000002" },
						{
							min_prompt_tokens: 200_000,
							input_cache_write: "0.0000025",
							input_cache_write_1h: "0.000005",
						},
					],
				},
			}),
		),
		{ data: [host({ tag: "a", pricing: PRICES })] },
	),
	join(
		"join-tiers-none-when-the-listing-has-no-base-price",
		[LUNA],
		listing(one({ pricing: { prompt: "0.000001" } })),
		{ data: [host({ tag: "a", pricing: PRICES })] },
	),
	join(
		"join-efforts-in-the-order-of-the-scale",
		[LUNA, LARGE],
		listing(one({ reasoning: { supported_efforts: ["max", "bogus", "low", "xhigh", "low"] } }), {
			id: LARGE,
			reasoning: { supported_efforts: ["low", "high"] },
		}),
		{ data: [host({ tag: "a" }), host({ model_id: LARGE, tag: "a" })] },
	),
	join(
		"join-data-kept-when-no-host-is-zero-retention",
		[LUNA, OPUS],
		listing(one({ supported_parameters: PARAMS }), { id: OPUS, supported_parameters: ["tools", "reasoning"] }),
		{ data: [host({ tag: "a", supported_parameters: ["tools"] })] },
		[pass("chat", false)],
	),
	join(
		"join-made-up-hosted-row",
		["openrouter:lab/model-x", LUNA],
		listing({ id: "lab/model-x", supported_parameters: ["tools", "reasoning"] }),
		{
			data: [
				{ model_id: "lab/model-x", tag: "h", supported_parameters: ["tools", "reasoning"], pricing: PRICES },
			],
		},
		[pass("chat", true), pass("analysis", true)],
	),
	join(
		"join-later-model-of-an-id-replaces-the-earlier",
		[LUNA],
		listing(one({ expiration_date: "2026-09-01" }), one({ name: "Again" })),
		{ data: [host({ tag: "h" })] },
	),
	join("join-no-rows", [], MODELS_BODY, ZDR_BODY),

	// Listed prices.
	perMillionCase("per-million-numbers", 0, 0.000001, 0.0000005, 1.5e-7, 2e-7, 1, 25, 1e-13, 4.9e-13, 5e-13),
	perMillionCase(
		"per-million-json-number-texts",
		"0",
		"0.000001",
		"1e-6",
		"1E-6",
		"0.0000005",
		"1.5e-7",
		"25",
		"1e2",
		"0.5e-6",
	),
	perMillionCase(
		"per-million-rounds-to-six-decimals",
		"0.00000012345",
		"0.00000099999",
		"0.0000004999999",
		"0.0000005",
		"0.00000051",
	),
	perMillionCase("per-million-negative-is-no-price", -1, "-1", -0.000001, "-0.000001", -1e-30),
	perMillionCase("per-million-negative-zero-is-zero", "-0", "-0.0", "-0e5"),
	perMillionCase("per-million-not-a-number-or-text", null, true, false, [], [1], {}, { price: 1 }),
	perMillionCase(
		"per-million-text-spellings-the-primitive-refuses",
		"",
		" ",
		"\t\n",
		" 0.5 ",
		"0.5 ",
		" 0.5",
		"+1",
		"+0.5",
		".5",
		"5.",
		"-.5",
		"00.5",
		"007",
		"0x10",
		"0b11",
		"0o7",
		"1_0",
		"1e",
		"1e+",
		"e5",
		"1,5",
	),
	perMillionCase("per-million-not-finite", "Infinity", "-Infinity", "NaN", "inf", "1e999", "1e309"),
	perMillionCase("per-million-text-other", "free", "abc", "1.2.3", "1 2", "0.5USD", "\u{661}\u{662}"),

	// Removal dates.
	expiryCase("expiry-days", "2026-10-01", "2026-01-01", "2028-02-29", "0001-01-01", "9999-12-31", "1970-01-01"),
	expiryCase(
		"expiry-times",
		"2026-10-01T00:00:00Z",
		"2026-10-01T00:00:00.000Z",
		"2026-10-01T12:30:45Z",
		"2026-10-01T12:30:45.123456Z",
		"2026-10-01T12:30:45+02:00",
		"2026-10-01T12:30:45-08:00",
	),
	expiryCase(
		"expiry-not-days",
		"2026-02-29",
		"2026-02-30",
		"2026-13-01",
		"2026-00-10",
		"2026-10-00",
		"2026-10-32",
		"abcdefghij",
	),
	expiryCase(
		"expiry-times-out-of-range",
		"2026-10-01T24:00:00Z",
		"2026-10-01T23:60:00Z",
		"2026-10-01T23:59:60Z",
		"2026-10-01T12:00:00+24:00",
	),
	expiryCase(
		"expiry-spellings-the-primitive-refuses",
		"2026",
		"2026-10",
		"2026-10-01T12:00",
		"2026-10-01 12:00:00",
		"2026-10-01t12:00:00z",
		"2026-10-01T12:00:00+0100",
		"2026-10-01T12:00:00+01",
		"Oct 1 2026",
		"October 1, 2026",
		"Thu, 01 Oct 2026 00:00:00 GMT",
		"2026/10/01",
		"10/01/2026",
		"20261001",
		"+002026-10-01",
		" 2026-10-01",
		"2026-10-01 ",
		"2026-10-01\n",
	),
	expiryCase("expiry-nothing", "", "soon", "never", "0", "1790000000000", "2026-10-01Tabc"),

	// Standing: a removal, then the hosts that take every parameter.
	standingCase("standing-ok-and-no-host", [
		{
			expiration: null,
			hosts: [{ params: ["tools", "reasoning"] }, { params: ["tools"] }, { params: null }],
			needs: ["tools", "reasoning"],
			nowMs: NOW,
		},
		{ expiration: null, hosts: [], needs: ["tools"], nowMs: NOW },
		{ expiration: null, hosts: [{ params: ["tools"] }], needs: [], nowMs: NOW },
		{ expiration: null, hosts: [{ params: null }], needs: ["tools"], nowMs: NOW },
		{ expiration: null, hosts: [{ params: null }], needs: [], nowMs: NOW },
	]),
	standingCase("standing-retired-by-removal", [
		{ expiration: "2026-09-25", hosts: [{ params: ["tools"] }], needs: ["tools"], nowMs: NOW },
		{ expiration: "2026-09-26", hosts: [{ params: ["tools"] }], needs: ["tools"], nowMs: NOW },
		{ expiration: "2026-09-25T12:00:00Z", hosts: [{ params: ["tools"] }], needs: ["tools"], nowMs: NOW },
		{ expiration: "2026-09-25T12:00:00Z", hosts: [{ params: ["tools"] }], needs: ["tools"], nowMs: NOW - 1 },
		{ expiration: "2026-09-25T12:00:00Z", hosts: [{ params: ["tools"] }], needs: ["tools"], nowMs: NOW + HOUR },
		{ expiration: "never", hosts: [{ params: ["tools"] }], needs: ["tools"], nowMs: NOW },
		{ expiration: "", hosts: [{ params: ["tools"] }], needs: ["tools"], nowMs: NOW },
	]),
	standingCase("standing-any-of-a-list-suffices", [
		{
			expiration: null,
			hosts: [{ params: ["response_format"] }, { params: ["structured_outputs"] }, { params: ["tools"] }],
			needs: [["response_format", "structured_outputs"]],
			nowMs: NOW,
		},
		{
			expiration: null,
			hosts: [{ params: ["response_format", "tools"] }, { params: ["tools"] }],
			needs: ["tools", ["response_format", "structured_outputs"]],
			nowMs: NOW,
		},
		{ expiration: null, hosts: [{ params: ["tools"] }], needs: [[]], nowMs: NOW },
	]),
];

////////////////////////////////
//  Uses of a consumer's own

const SUMMARY = { name: "summary", requires: ["schema"], hides: true };
const TRIAGE = { name: "triage", requires: [], hides: false };

const byUse = catalogJoinCases.find((each) => each.name === "join-by-use");

if (byUse !== undefined) {
	const input = byUse.input as Record<string, unknown>;
	catalogJoinCases.push(
		{
			...byUse,
			name: "join-custom-uses-by-what-they-require",
			input: {
				...input,
				passes: [
					pass(SUMMARY, true),
					pass(TRIAGE, true),
					pass({ name: "chat", requires: ["schema"], hides: true }, true),
					pass({ name: "analysis", requires: [], hides: false }, true),
				],
			},
		},
		{
			...byUse,
			name: "join-custom-use-needs-the-structured-parameters-of-a-host",
			input: { ...input, passes: [pass(SUMMARY, false), pass(TRIAGE, false)] },
		},
	);
}
