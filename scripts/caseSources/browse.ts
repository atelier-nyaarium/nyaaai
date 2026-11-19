// OpenRouter browse cases, over the decoders and over the scripted network. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { cut, failing, HOUR, type Json, NOW, refused, reply, setIn, wrongFor } from "./catalogKit";

////////////////////////////////
//  Fixtures

const BEFORE = 1_777_593_600_000;
const AFTER = 1_782_864_000_000;
const REMOVAL_DAY = 1_780_272_000_000;

const ZDR_BODY: Json = {
	data: [
		{
			model_id: "lab/glm",
			tag: "decart",
			supported_parameters: ["tools", "reasoning"],
			pricing: { prompt: "0.00000049", completion: "0.00000154" },
		},
		{
			model_id: "lab/glm",
			tag: "relace",
			supported_parameters: ["tools"],
			pricing: { prompt: "0.0000002", completion: "0.000004" },
		},
		{
			model_id: "x/plain",
			tag: "host",
			supported_parameters: ["response_format"],
			pricing: { prompt: "0.000001", completion: "0.000002" },
		},
		{
			model_id: "x/thinker",
			tag: "host",
			supported_parameters: ["tools"],
			pricing: { prompt: "0.000001", completion: "0.000002" },
		},
		{
			model_id: "x/retiring",
			tag: "host",
			supported_parameters: ["tools"],
			pricing: { prompt: "0.000001", completion: "0.000002" },
		},
	],
};

const RAW: Json[] = [
	{
		id: "lab/glm",
		name: "Lab: GLM",
		description: "A reasoning model.",
		context_length: 1_000_000,
		architecture: { input_modalities: ["text"], output_modalities: ["text"] },
		supported_parameters: ["tools", "reasoning", "structured_outputs"],
		reasoning: { supported_efforts: ["low", "high"] },
	},
	{ id: "lab/glm:batch", name: "Lab: GLM (batch)" },
	{ id: "x/plain", supported_parameters: ["tools"] },
	{ id: "x/thinker", reasoning: { supported_efforts: ["high"] } },
	{ id: "x/retiring", expiration_date: "2026-06-01" },
	{ id: 7 },
	{ id: "../escape" },
];

const ITEM: Json = {
	id: "lab/model-a",
	name: "Model A",
	description: "A model for cases.",
	context_length: 128_000,
	architecture: { input_modalities: ["text", "image"], output_modalities: ["text"] },
	supported_parameters: ["tools", "reasoning"],
	expiration_date: "2027-01-15",
	reasoning: { supported_efforts: ["low", "high"] },
};

const ITEM_ZDR: Json = {
	data: [
		{
			model_id: "lab/model-a",
			tag: "host-a",
			supported_parameters: ["tools", "reasoning"],
			pricing: { prompt: "0.000001", completion: "0.000002" },
		},
	],
};

type Site = { path: string[]; holds: "text" | "number" | "object" | "list" };

const ENTRY_SITES: Site[] = [
	{ path: ["name"], holds: "text" },
	{ path: ["description"], holds: "text" },
	{ path: ["context_length"], holds: "number" },
	{ path: ["architecture"], holds: "object" },
	{ path: ["architecture", "input_modalities"], holds: "list" },
	{ path: ["architecture", "output_modalities"], holds: "list" },
	{ path: ["supported_parameters"], holds: "list" },
	{ path: ["expiration_date"], holds: "text" },
	{ path: ["reasoning"], holds: "object" },
	{ path: ["reasoning", "supported_efforts"], holds: "list" },
];

const ENDPOINT: Json = {
	provider_name: "Host A",
	name: "Host A name",
	tag: "host-a",
	context_length: 131_000,
	max_completion_tokens: 118_000,
	quantization: "fp8",
	uptime_last_30m: 99.5,
	pricing: { prompt: "0.00000049", completion: "0.00000154", input_cache_read: "0.00000009" },
};

const PROVIDER_SITES: Site[] = [
	{ path: ["provider_name"], holds: "text" },
	{ path: ["name"], holds: "text" },
	{ path: ["tag"], holds: "text" },
	{ path: ["context_length"], holds: "number" },
	{ path: ["max_completion_tokens"], holds: "number" },
	{ path: ["quantization"], holds: "text" },
	{ path: ["uptime_last_30m"], holds: "number" },
	{ path: ["pricing"], holds: "object" },
];

////////////////////////////////
//  Functions & Helpers

const models = (name: string, raw: Json[], zdr: unknown, nowMs = BEFORE): CaseSource => ({
	kind: "browse.openrouterModels",
	name,
	input: { raw, zdr, nowMs },
});

const providers = (name: string, raw: unknown, zdr: unknown = null): CaseSource => ({
	kind: "browse.openrouterProviders",
	name,
	input: { raw, zdr },
});

const detail = (endpoints: unknown[]): Json => ({ data: { endpoints } }) as Json;

function sites(
	make: (name: string, item: Json) => CaseSource,
	item: Json,
	list: Site[],
	prefix: string,
	wrongItem: (path: string[]) => Json,
): CaseSource[] {
	const cases: CaseSource[] = [];
	for (const site of list) {
		const name = `${prefix}-${site.path.join(".")}`;
		cases.push(make(`${name}-omitted`, cut(item, site.path)));
		cases.push(make(`${name}-null`, setIn(item, site.path, null)));
		cases.push(make(`${name}-wrong-type`, setIn(item, site.path, wrongFor(site.holds))));
		if (site.holds === "list") cases.push(make(`${name}-wrong-item`, setIn(item, site.path, wrongItem(site.path))));
	}
	return cases;
}

/** A request whose history holds a step of the same model with a thought to resend. */
const REQUEST: Json = {
	model: "openai/gpt-6-luna",
	system: ["Rules."],
	history: [{ role: "user", text: "Hi" }],
	cache: { kind: "none" },
	session: "s-1",
	zdrOnly: true,
};

const step = (model: string): Json => ({
	role: "step",
	producer: { model },
	parts: [
		{
			kind: "thought",
			type: "reasoning.text",
			segments: [{ kind: "content", text: "Thinking." }],
			opaque: "sig",
			index: 0,
		},
		{ kind: "text", text: "Done." },
	],
});

const STEPPED: Json = {
	...(REQUEST as object),
	history: [{ role: "user", text: "Hi" }, step("openai/gpt-6-luna"), { role: "user", text: "More" }],
} as Json;

const STEPPED_OTHER: Json = {
	...(REQUEST as object),
	history: [{ role: "user", text: "Hi" }, step("lab/other"), { role: "user", text: "More" }],
} as Json;

const encode = (mode: string, request: Json = REQUEST): Json => ({ kind: "encode", request, mode });

const pins = (name: string, calls: Json[]): CaseSource => ({ kind: "browse.openrouterPins", name, input: { calls } });

const browser = (name: string, input: Record<string, unknown>): CaseSource => ({
	kind: "browse.openrouterBrowser",
	name,
	input: { startMs: NOW, ...input },
});

const hop = (name: string, input: Record<string, unknown>): CaseSource => ({
	kind: "browse.openrouterHop",
	name,
	input: { startMs: NOW, ...input },
});

const MODELS_BODY = reply({ data: RAW });
const ZDR = reply(ZDR_BODY);
const PAIR = [MODELS_BODY, ZDR];
const DETAIL_BODY = reply(
	detail([
		{
			provider_name: "Decart",
			tag: "decart",
			context_length: 131_000,
			pricing: { prompt: "0.00000049", completion: "0.00000154" },
		},
		{ provider_name: "Novita", tag: "novita/fp8", pricing: { prompt: "0.00000065" } },
	]),
);

const list = (extra: Record<string, unknown> = {}) => ({ kind: "list", ...extra });
const advance = (ms: number) => ({ kind: "advance", ms });

////////////////////////////////
//  Cases

export const browseCases: CaseSource[] = [
	// The listing.
	models("models-listed-and-batch-twins-folded", RAW, ZDR_BODY),
	models("models-after-a-removal-day", RAW, ZDR_BODY, AFTER),
	models("models-at-the-removal-day-itself", RAW, ZDR_BODY, REMOVAL_DAY),
	models("models-one-millisecond-before-the-removal-day", RAW, ZDR_BODY, REMOVAL_DAY - 1),
	...sites(
		(name, item) => models(name, [item], ITEM_ZDR),
		ITEM,
		ENTRY_SITES,
		"models",
		(path) => (path[path.length - 1] === "supported_efforts" ? ["low", 3] : ["a", 3]),
	),
	models("models-full-item", [ITEM], ITEM_ZDR),
	models("models-minimal-item", [{ id: "lab/bare" }], ITEM_ZDR),
	models("models-curated-flag", [{ id: "openai/gpt-6-luna" }, { id: "lab/model-a" }], { data: [] }),
	models("models-id-null", [setIn(ITEM, ["id"], null)], ITEM_ZDR),
	models("models-item-not-an-object", ["lab/model-a", null, [ITEM], ITEM], ITEM_ZDR),
	models(
		"models-ids-the-host-accepts",
		[
			"lab/model",
			"lab/model:free",
			"lab/a/b",
			"a.b/c-d",
			"~lab/model",
			"lab",
			"lab/",
			"/model",
			"lab//x",
			"-lab/model",
			"lab/mod el",
			"lab/m\u{f8}del",
			"",
			"lab/model:batch",
			"lab/model:Batch",
			"lab/x:batch:batch",
		].map((id) => ({ id })),
		{ data: [] },
	),
	models("models-no-zero-retention-host-reads-unknown-price", [ITEM], { data: [] }),
	models("models-price-needs-both-ranges", [ITEM], {
		data: [
			{
				model_id: "lab/model-a",
				tag: "a",
				supported_parameters: ["tools", "reasoning"],
				pricing: { prompt: "0.000001" },
			},
		],
	}),
	models("models-price-ranges-over-hosts", [ITEM], {
		data: [
			{ model_id: "lab/model-a", tag: "a", pricing: { prompt: "0.000003", completion: "0.000004" } },
			{ model_id: "lab/model-a", tag: "b", pricing: { prompt: "0.000001", completion: "0.000009" } },
			{ model_id: "lab/model-a", tag: "c", pricing: { prompt: "oops" } },
		],
	}),
	models("models-zdr-body-unreadable-reads-no-hosts", [ITEM], { error: "down" }),
	models("models-reasoning-needs-a-host-with-reasoning", [setIn(ITEM, ["expiration_date"], null)], {
		data: [{ model_id: "lab/model-a", tag: "a", supported_parameters: ["tools"] }],
	}),
	models(
		"models-empty-efforts-need-no-reasoning",
		[setIn(setIn(ITEM, ["expiration_date"], null), ["reasoning"], { supported_efforts: [] })],
		{
			data: [{ model_id: "lab/model-a", tag: "a", supported_parameters: ["tools"] }],
		},
	),
	models("models-empty-listing", [], ZDR_BODY),

	// The providers of one model.
	providers(
		"providers-marked-by-tag-and-priced-per-million",
		detail([ENDPOINT, { provider_name: "Novita", tag: "novita/fp8", pricing: { prompt: "0.00000065" } }]),
		{
			data: [{ model_id: "lab/model-a", tag: "host-a" }],
		},
	),
	providers("providers-body-has-no-endpoints-list", { data: [] }),
	...sites(
		(name, item) => providers(name, detail([item]), { data: [{ model_id: "lab/model-a", tag: "host-a" }] }),
		ENDPOINT,
		PROVIDER_SITES,
		"providers",
		() => ["a", 3],
	),
	providers(
		"providers-pricing-prompt-text-spelling",
		detail([setIn(ENDPOINT, ["pricing"], { prompt: 1e-6, completion: " 0.5 ", input_cache_read: "-1" })]),
	),
	providers(
		"providers-pricing-any-json-passes-as-no-price",
		detail([setIn(ENDPOINT, ["pricing"], { prompt: null, completion: {}, input_cache_read: [1] })]),
	),
	providers("providers-empty-item-is-a-provider-without-a-name", detail([{}])),
	providers("providers-item-not-an-object", detail(["x", null, [ENDPOINT], 3, ENDPOINT])),
	providers(
		"providers-name-falls-back-to-name-then-tag",
		detail([{ name: "Only name", tag: "t" }, { tag: "only-tag" }, { provider_name: "P", name: "N", tag: "t" }]),
	),
	providers("providers-zero-retention-only-by-exact-tag", detail([{ tag: "a" }, { tag: "A" }, { tag: "a/b" }, {}]), {
		data: [
			{ model_id: "m", tag: "a" },
			{ model_id: "m", tag: null },
			{ model_id: "m" },
			{ model_id: "m", tag: "" },
		],
	}),
	providers("providers-zero-retention-tags-of-an-unreadable-body", detail([{ tag: "a" }]), { error: "down" }),
	providers("providers-body-null", null),
	providers("providers-body-text", "x"),
	providers("providers-body-list", [ENDPOINT]),
	providers("providers-body-without-data", { endpoints: [ENDPOINT] }),
	providers("providers-data-null", { data: null }),
	providers("providers-data-list", { data: [ENDPOINT] }),
	providers("providers-endpoints-null", { data: { endpoints: null } }),
	providers("providers-endpoints-object", { data: { endpoints: ENDPOINT } }),
	providers("providers-endpoints-text", { data: { endpoints: "x" } }),

	// The browser on a scripted network.
	browser("browser-list-reads-both-and-lists", {
		exchanges: PAIR,
		steps: [list(), list()],
	}),
	browser("browser-list-after-the-cache-expires-reads-again", {
		exchanges: [...PAIR, MODELS_BODY, ZDR],
		steps: [list(), advance(13 * HOUR), list()],
	}),
	browser("browser-list-after-a-removal-day-passes", {
		exchanges: [
			reply({ data: [{ id: "x/retiring", expiration_date: "2026-09-25T13:00:00Z" }] }),
			reply({ data: [{ model_id: "x/retiring", tag: "host", supported_parameters: ["tools"] }] }),
		],
		steps: [list(), advance(HOUR - 1), list(), advance(1), list()],
	}),
	browser("browser-list-failing-answers-nothing-and-fresh-retries", {
		exchanges: [failing(), failing(), ...PAIR],
		steps: [list(), list(), list({ fresh: true }), list()],
	}),
	browser("browser-detail-marks-zero-retention-hosts", {
		exchanges: [...PAIR, DETAIL_BODY],
		steps: [{ kind: "detail", id: "lab/glm" }],
	}),
	...["lab/glm?x=1", "lab/glm#frag", "lab/../glm", "lab/glm one"].map((id, at) =>
		browser(`browser-detail-an-id-goes-into-the-path-segment-by-segment-${at}`, {
			exchanges: [...PAIR, DETAIL_BODY],
			steps: [{ kind: "detail", id }],
		}),
	),
	browser("browser-detail-with-a-listing-that-failed-marks-no-host", {
		exchanges: [failing(), failing(), DETAIL_BODY],
		steps: [{ kind: "detail", id: "lab/glm" }],
	}),
	browser("browser-detail-a-body-that-is-no-detail-answers-nothing", {
		exchanges: [...PAIR, reply({ data: [] })],
		steps: [{ kind: "detail", id: "lab/glm" }],
	}),
	browser("browser-detail-a-failed-read-throws", {
		exchanges: [...PAIR, failing(404)],
		steps: [{ kind: "detail", id: "lab/glm" }],
	}),
	browser("browser-detail-has-no-deadline-of-its-own", {
		exchanges: [...PAIR, { ...DETAIL_BODY, headMs: 60_000 }],
		steps: [{ kind: "detail", id: "lab/glm" }],
	}),
	browser("browser-detail-the-callers-abort-ends-it", {
		exchanges: [...PAIR, { ...DETAIL_BODY, headMs: 10 }],
		steps: [{ kind: "detail", id: "lab/glm", abortAfterMs: 5 }],
	}),
	browser("browser-detail-aborted-before-still-starts-the-listing", {
		exchanges: [...PAIR, DETAIL_BODY],
		steps: [
			{ kind: "detail", id: "lab/glm", abortedBefore: true },
			{ kind: "detail", id: "lab/glm" },
		],
	}),
	browser("browser-detail-asks-the-path-of-the-model-id", {
		exchanges: [...PAIR, DETAIL_BODY, DETAIL_BODY],
		steps: [
			{ kind: "detail", id: "lab/glm" },
			{ kind: "detail", id: "x/plain" },
		],
	}),
	browser("browser-tester-listed-and-view", {
		exchanges: PAIR,
		steps: [
			{ kind: "listed", id: "lab/glm" },
			{ kind: "listed", id: "x/plain" },
			{ kind: "listed", id: "lab/none" },
		],
	}),
	browser("browser-tester-view-joins-a-row-on-zero-retention-hosts", {
		exchanges: [
			reply({
				data: [
					{
						id: "lab/made-up",
						supported_parameters: ["tools", "reasoning"],
						reasoning: { supported_efforts: ["low"] },
					},
					{ id: "lab/other" },
				],
			}),
			reply({
				data: [
					{
						model_id: "lab/made-up",
						tag: "h",
						supported_parameters: ["tools", "reasoning"],
						pricing: { prompt: "0.000001", completion: "0.000002" },
					},
				],
			}),
		],
		steps: [
			{ kind: "view", row: "openrouter:lab/made-up" },
			{ kind: "view", row: "openrouter:lab/other", use: "analysis" },
			{ kind: "view", row: "openrouter:lab/gone" },
		],
	}),
	browser("browser-tester-without-a-listing-answers-nothing-and-unknown", {
		exchanges: [failing(), failing()],
		steps: [
			{ kind: "listed", id: "lab/glm" },
			{ kind: "view", row: "openrouter:lab/glm" },
		],
	}),
	browser("browser-refused-connections", {
		exchanges: [refused(), refused()],
		steps: [list()],
	}),

	// The pins on the Chat codec.
	pins("pins-alternate-the-hosts-and-note-resent-reasoning", [
		{ kind: "pin", hosts: ["a", "b"] },
		encode("stream"),
		encode("stream"),
		{ kind: "passedBack" },
		encode("stream", STEPPED),
		{ kind: "passedBack" },
		encode("stream", STEPPED),
		encode("stream", STEPPED),
		{ kind: "passedBack" },
	]),
	pins("pins-the-first-request-is-not-a-resend", [
		{ kind: "pin", hosts: ["a"] },
		encode("stream", STEPPED),
		{ kind: "passedBack" },
		encode("stream", STEPPED),
		{ kind: "passedBack" },
	]),
	pins("pins-no-hosts-leaves-the-request-alone", [
		encode("stream"),
		{ kind: "pin", hosts: [] },
		encode("stream", STEPPED),
		encode("stream", STEPPED),
		{ kind: "passedBack" },
	]),
	pins("pins-a-complete-request-is-not-pinned-nor-noted", [
		{ kind: "pin", hosts: ["a", "b"] },
		encode("complete", STEPPED),
		encode("complete", STEPPED),
		{ kind: "passedBack" },
		encode("stream"),
		encode("stream"),
	]),
	pins("pins-pinning-again-starts-over", [
		{ kind: "pin", hosts: ["a", "b", "c"] },
		encode("stream"),
		encode("stream"),
		{ kind: "pin", hosts: ["x", "y"] },
		encode("stream"),
		encode("stream"),
		encode("stream"),
		{ kind: "pin", hosts: [] },
		encode("stream"),
	]),
	pins("pins-a-reply-from-another-model-is-not-resent", [
		{ kind: "pin", hosts: ["a"] },
		encode("stream", STEPPED_OTHER),
		encode("stream", STEPPED_OTHER),
		{ kind: "passedBack" },
	]),

	// The hosts a test pins.
	hop("hop-two-hosts-in-both-lists", {
		exchanges: [reply(detail([{ tag: "a/fp8" }, { tag: "b" }, { tag: "c" }, { tag: "a/fp4" }]))],
		id: "lab/glm",
		viewHosts: ["a", "b/us", "c"],
	}),
	hop("hop-one-host-in-both-lists-pins-none", {
		exchanges: [reply(detail([{ tag: "a" }, { tag: "x" }]))],
		id: "lab/glm",
		viewHosts: ["a", "b"],
	}),
	hop("hop-no-host-in-the-view", {
		exchanges: [reply(detail([{ tag: "a" }, { tag: "b" }]))],
		id: "lab/glm",
		viewHosts: [],
	}),
	hop("hop-order-is-the-serving-order", {
		exchanges: [reply(detail([{ tag: "c" }, { tag: "a" }, { tag: "b" }]))],
		id: "lab/glm",
		viewHosts: ["a", "b", "c"],
	}),
	hop("hop-hosts-without-a-tag-are-left-out", {
		exchanges: [reply(detail([{}, { tag: "" }, { tag: "a" }, { provider_name: "A" }]))],
		id: "lab/glm",
		viewHosts: ["", "a"],
	}),
	hop("hop-a-host-named-by-its-provider-meets-the-tag-in-any-case", {
		exchanges: [reply(detail([{ tag: "together/fp8" }, { tag: "novita" }, { tag: "other" }]))],
		id: "lab/glm",
		viewHosts: ["Together", "NOVITA ", "other-host"],
	}),
	hop("hop-an-endpoint-with-no-tag-meets-no-host", {
		exchanges: [reply(detail([{ provider_name: "Together" }, { tag: "" }, { tag: "a" }, { tag: "b" }]))],
		id: "lab/glm",
		viewHosts: ["", "Together", "a"],
	}),
	...["lab/glm?x=1", "lab/glm#frag", "lab/../glm", "../glm", "lab/./glm", "lab/glm one", "lab//glm", "lab/gl%6D"].map(
		(id, at) =>
			hop(`hop-an-id-goes-into-the-path-segment-by-segment-${at}`, {
				exchanges: [reply(detail([{ tag: "a" }, { tag: "b" }]))],
				id,
				viewHosts: ["a", "b"],
			}),
	),
	hop("hop-a-body-that-is-no-detail-pins-none", {
		exchanges: [reply({ data: [] })],
		id: "lab/glm",
		viewHosts: ["a", "b"],
	}),
	hop("hop-a-failed-read-throws", {
		exchanges: [failing(500)],
		id: "lab/glm",
		viewHosts: ["a", "b"],
	}),
	hop("hop-a-slow-read-ends-at-the-deadline", {
		exchanges: [{ ...reply(detail([{ tag: "a" }, { tag: "b" }])), headMs: 30_001 }],
		id: "lab/glm",
		viewHosts: ["a", "b"],
	}),
	hop("hop-a-read-just-inside-the-deadline-lands", {
		exchanges: [{ ...reply(detail([{ tag: "a" }, { tag: "b" }])), headMs: 29_999 }],
		id: "lab/glm",
		viewHosts: ["a", "b"],
	}),
	hop("hop-the-callers-abort-ends-it", {
		exchanges: [{ ...reply(detail([{ tag: "a" }, { tag: "b" }])), headMs: 100 }],
		id: "lab/glm",
		viewHosts: ["a", "b"],
		abortAfterMs: 50,
	}),
	hop("hop-aborted-before-asks-nothing", {
		exchanges: [reply(detail([{ tag: "a" }, { tag: "b" }]))],
		id: "lab/glm",
		viewHosts: ["a", "b"],
		abortedBefore: true,
	}),
];
