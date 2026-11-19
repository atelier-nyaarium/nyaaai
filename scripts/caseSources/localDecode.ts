// Local decoder cases. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { caseOf, cut, type Json, setIn, wrongFor } from "./catalogKit";

////////////////////////////////
//  Interfaces & Types

type Holds = "text" | "number" | "object" | "list" | "boolean";

/** A field that falls back, the path in its item and the kind of value it holds. */
interface Site {
	path: string[];
	holds: Holds;
	/** A list item that makes the whole list fall back. */
	badItem?: Json;
}

////////////////////////////////
//  Functions & Helpers

const efforts = caseOf("local.efforts");
const label = caseOf("local.label");
const view = caseOf("local.view");
const tags = caseOf("local.ollamaTags");
const show = caseOf("local.ollamaShow");
const numCtx = caseOf("local.ollamaNumCtx");
const running = caseOf("local.ollamaRunning");
const lmModels = caseOf("local.lmstudioModels");
const lmLoaded = caseOf("local.lmstudioLoaded");
const llamaModels = caseOf("local.llamacppModels");
const llamaProps = caseOf("local.llamacppProps");
const llamaModel = caseOf("local.llamacppModel");
const compatModels = caseOf("local.compatibleModels");
const compatTerms = caseOf("local.compatibleTerms");

/** One case per way a site can be wrong: omitted, null, a wrong type, and for a list a wrong item. */
function siteCases(
	make: (name: string, input: Json) => CaseSource,
	base: Json,
	sites: Site[],
	prefix: string,
	wrap: (item: Json) => Json,
): CaseSource[] {
	return sites.flatMap((site) => {
		const name = `${prefix}-${site.path.join(".")}`;
		const cases = [
			make(`${name}-omitted`, wrap(cut(base, site.path))),
			make(`${name}-null`, wrap(setIn(base, site.path, null))),
			make(`${name}-wrong-type`, wrap(setIn(base, site.path, wrongFor(site.holds)))),
		];
		if (site.badItem !== undefined)
			cases.push(make(`${name}-wrong-item`, wrap(setIn(base, site.path, site.badItem))));
		return cases;
	});
}

/** A body whose item list is under `key`. */
const listOf =
	(key: string) =>
	(...items: Json[]): Json => ({ [key]: items });

////////////////////////////////
//  Fixtures

const A = "local-a:27b";
const B = "local-b:9b";

const SHOW: Json = {
	parameters: "num_ctx                        32768\ntemperature                    0.6",
	capabilities: ["completion", "tools", "thinking"],
	thinking: { values: [false, true], default: true },
	model_info: { "general.architecture": "arch-a", "arch-a.context_length": 262_144 },
};

const SHOW_SITES: Site[] = [
	{ path: ["capabilities"], holds: "list", badItem: ["completion", 3] },
	{ path: ["thinking"], holds: "object" },
	{ path: ["thinking", "values"], holds: "list", badItem: [false, 1] },
	{ path: ["parameters"], holds: "text" },
	{ path: ["model_info"], holds: "object" },
	{ path: ["model_info", "general.architecture"], holds: "text" },
	{ path: ["model_info", "arch-a.context_length"], holds: "number" },
];

const PS_ITEM: Json = { name: A, model: A, context_length: 65_536, size_vram: 1 };

const LM_ITEM: Json = {
	key: "local-a",
	type: "llm",
	max_context_length: 262_144,
	loaded_instances: [{ id: "local-a", config: { context_length: 8192 } }],
	capabilities: {
		vision: false,
		trained_for_tool_use: true,
		reasoning: { allowed_options: ["off", "on"], default: "on" },
	},
};

const LM_SITES: Site[] = [
	{ path: ["max_context_length"], holds: "number" },
	{ path: ["loaded_instances"], holds: "list", badItem: [{ config: { context_length: 1 } }, "x"] },
	{ path: ["loaded_instances", "0", "config"], holds: "object" },
	{ path: ["loaded_instances", "0", "config", "context_length"], holds: "number" },
	{ path: ["capabilities"], holds: "object" },
	{ path: ["capabilities", "reasoning"], holds: "object" },
	{ path: ["capabilities", "reasoning", "allowed_options"], holds: "list", badItem: ["on", 1] },
];

const LLAMA_ITEM: Json = { id: "local-a.gguf", meta: { n_ctx_train: 131_072 }, context_length: 32_768 };

const LLAMA_SITES: Site[] = [
	{ path: ["meta"], holds: "object" },
	{ path: ["meta", "n_ctx_train"], holds: "number" },
	{ path: ["context_length"], holds: "number" },
];

const PROPS: Json = {
	default_generation_settings: { n_ctx: 65_536 },
	chat_template_caps: { reasoning_effort_levels: ["low", "high"] },
};

const PROPS_SITES: Site[] = [
	{ path: ["default_generation_settings"], holds: "object" },
	{ path: ["default_generation_settings", "n_ctx"], holds: "number" },
	{ path: ["chat_template_caps"], holds: "object" },
	{ path: ["chat_template_caps", "reasoning_effort_levels"], holds: "list", badItem: ["low", 1] },
];

const COMPAT_ITEM: Json = { id: "local-a", context_length: 32_768, max_context_length: 16_384, max_model_len: 8192 };

const COMPAT_SITES: Site[] = [
	{ path: ["context_length"], holds: "number" },
	{ path: ["max_context_length"], holds: "number" },
	{ path: ["max_model_len"], holds: "number" },
];

/** The strings an item's required text field can hold wrongly, as items of a list. */
const BAD_ITEMS: Json[] = ["x", 3, null, ["a"], true];

/** A body that is no object holding a list. */
const NOT_OBJECTS: Array<[string, Json]> = [
	["null", null],
	["text", "x"],
	["number", 3],
	["list", []],
	["list-of-items", [{ name: A }]],
];

////////////////////////////////
//  Cases

export const localDecodeCases: CaseSource[] = [
	// Efforts.
	efforts("efforts-none", { values: [] }),
	efforts("efforts-false-is-none", { values: [false] }),
	efforts("efforts-true-is-low-medium-high", { values: [true] }),
	efforts("efforts-false-and-true", { values: [false, true] }),
	efforts("efforts-named", { values: ["low", "high"] }),
	efforts("efforts-named-in-ladder-order", { values: ["high", "low", "minimal"] }),
	efforts("efforts-the-top-two-are-never-on", { values: [true, "xhigh", "max"] }),
	efforts("efforts-unknown-names-drop", { values: ["bogus", "low"] }),
	efforts("efforts-only-unknown-names", { values: ["bogus"] }),
	efforts("efforts-case-matters", { values: ["Low", "HIGH"] }),
	efforts("efforts-repeats-count-once", { values: ["low", "low", true] }),

	// Labels.
	label("label-plain-tag", { tags: [A] }),
	label("label-path-keeps-the-last-part", { tags: ["org/repo/model.gguf", "a/b"] }),
	label("label-backslashes-split-too", { tags: ["dir\\sub\\file.gguf", "dir/sub\\file"] }),
	label("label-trailing-and-doubled-separators", { tags: ["a/b/", "a//b//", "/x", "x/"] }),
	label("label-blank-parts-are-skipped", { tags: ["a/ /", "  /x", "a/\t/\n"] }),
	label("label-the-part-is-kept-as-it-is", { tags: [" a / b ", "x/ y "] }),
	label("label-all-blank-keeps-the-tag", { tags: ["/", "//", " / ", "", " ", "\\"] }),
	label("label-cut-at-200-units", { tags: ["x".repeat(250), `dir/${"y".repeat(210)}`] }),
	label("label-exactly-200-is-whole", { tags: ["z".repeat(200), "z".repeat(201)] }),
	label("label-cut-never-splits-a-pair", { tags: [`${"x".repeat(199)}\u{1f600}tail`] }),
	label("label-a-long-whole-tag-when-every-part-is-blank", { tags: [`${" ".repeat(150)}/${" ".repeat(150)}`] }),

	// Views.
	view("view-ok-plain", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [] },
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),
	view("view-thinking-names-the-reasoning-field", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: true, efforts: ["low", "high"] },
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),
	view("view-no-tools", {
		row: `local:lab/${B}`,
		model: { tag: B, tools: false, thinking: false, efforts: [], context: 8192 },
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),
	view("view-no-tools-beats-the-retention-gate", {
		row: `local:lab/${B}`,
		model: { tag: B, tools: false, thinking: true, efforts: ["low"] },
		terms: { kept: true, price: "unknown" },
		zdrOnly: true,
	}),
	view("view-kept-under-zero-retention", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: true, efforts: ["low"], context: 8192 },
		terms: { kept: true, price: "unknown" },
		zdrOnly: true,
	}),
	view("view-kept-without-zero-retention-marks-the-data", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [], context: 8192 },
		terms: { kept: true, price: "unknown" },
		zdrOnly: false,
	}),
	view("view-free-home-needs-no-gate", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [], context: 8192 },
		terms: { kept: false, price: "free" },
		zdrOnly: false,
	}),
	view("view-context-wins-over-loaded", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [], context: 8192 },
		loaded: 4096,
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),
	view("view-loaded-fills-an-unset-context", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [] },
		loaded: 4096,
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),
	view("view-trained-length-caps-the-context", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [], context: 262_144, trained: 131_072 },
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),
	view("view-trained-length-caps-the-loaded-context", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [], trained: 4096 },
		loaded: 65_536,
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),
	view("view-trained-longer-than-the-context-changes-nothing", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [], context: 4096, trained: 131_072 },
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),
	view("view-trained-alone-gives-no-limit", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [], trained: 131_072 },
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),
	view("view-unprobed-model-still-serves", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [], probed: false },
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),

	// Ollama tags.
	tags("tags-two-local-models", { body: listOf("models")({ name: A, model: A }, { name: B, model: B }) }),
	tags("tags-extra-fields-are-ignored", {
		body: { models: [{ name: A, model: A, size: 3, details: { family: "arch-a" } }], extra: 1 },
	}),
	tags("tags-none", { body: { models: [] } }),
	tags("tags-order-and-repeats-are-kept", {
		body: listOf("models")({ name: B }, { name: A }, { name: B }),
	}),
	tags("tags-remote-model-is-dropped", {
		body: listOf("models")({ name: A }, { name: "far:1b-cloud", remote_host: "https://remote.example:443" }),
	}),
	tags("tags-remote-host-empty-text-still-drops", {
		body: listOf("models")({ name: A, remote_host: "" }),
	}),
	tags("tags-remote-host-omitted-is-local", { body: listOf("models")({ name: A }) }),
	tags("tags-remote-host-null-is-local", { body: listOf("models")({ name: A, remote_host: null }) }),
	tags("tags-remote-host-wrong-type-is-local", {
		body: listOf("models")(
			{ name: A, remote_host: 7 },
			{ name: B, remote_host: ["x"] },
			{ name: "c", remote_host: {} },
		),
	}),
	tags("tags-name-omitted-drops", { body: listOf("models")({ model: A }, { name: B }) }),
	tags("tags-name-null-drops", { body: listOf("models")({ name: null }, { name: B }) }),
	tags("tags-name-wrong-type-drops", { body: listOf("models")({ name: 7 }, { name: B }) }),
	tags("tags-name-empty-drops", { body: listOf("models")({ name: "" }, { name: B }) }),
	...BAD_ITEMS.map((item, at) =>
		tags(`tags-item-${at}-not-an-object-drops`, { body: listOf("models")(item, { name: B }) }),
	),
	tags("tags-models-omitted", { body: { data: [{ name: A }] } }),
	tags("tags-models-null", { body: { models: null } }),
	tags("tags-models-wrong-type-text", { body: { models: "x" } }),
	tags("tags-models-wrong-type-object", { body: { models: { name: A } } }),
	tags("tags-models-wrong-type-number", { body: { models: 3 } }),
	...NOT_OBJECTS.map(([name, body]) => tags(`tags-body-${name}`, { body })),
	tags("tags-a-foreign-server-answers-data", { body: { object: "list", data: [{ id: A }] } }),

	// Ollama show.
	show("show-full", { tag: A, body: SHOW }),
	show("show-the-tag-is-the-callers", { tag: B, body: SHOW }),
	show("show-completion-only", { tag: A, body: { capabilities: ["completion"] } }),
	show("show-no-completion-is-not-a-chat-model", { tag: A, body: { capabilities: ["embedding"] } }),
	show("show-empty-capabilities", { tag: A, body: { capabilities: [] } }),
	show("show-tools-without-completion", { tag: A, body: { capabilities: ["tools", "thinking"] } }),
	...siteCases(
		(name, input) => show(name, { tag: A, body: input }),
		SHOW,
		SHOW_SITES,
		"show",
		(item) => item,
	),
	show("show-thinking-values-only-false-disables-thinking", {
		tag: A,
		body: { ...(SHOW as object), thinking: { values: [false] } },
	}),
	show("show-thinking-values-false-and-a-name", {
		tag: A,
		body: { ...(SHOW as object), thinking: { values: [false, "high"] } },
	}),
	show("show-thinking-values-levels", {
		tag: A,
		body: { ...(SHOW as object), thinking: { values: ["low", "medium", "high"], default: "medium" } },
	}),
	show("show-thinking-values-true-only", {
		tag: A,
		body: { ...(SHOW as object), thinking: { values: [true] } },
	}),
	show("show-thinking-values-unknown-names-think-with-no-effort", {
		tag: A,
		body: { ...(SHOW as object), thinking: { values: ["bogus"] } },
	}),
	show("show-thinking-values-empty-does-not-think", {
		tag: A,
		body: { ...(SHOW as object), thinking: { values: [] } },
	}),
	show("show-thinking-listed-without-the-capability-does-not-think", {
		tag: A,
		body: { ...(SHOW as object), capabilities: ["completion", "tools"] },
	}),
	show("show-thinking-capability-without-a-list-thinks-with-no-effort", {
		tag: A,
		body: { capabilities: ["completion", "thinking"] },
	}),
	show("show-no-tools", { tag: A, body: { capabilities: ["completion", "thinking"], thinking: { values: [true] } } }),
	show("show-context-from-the-baked-parameters", {
		tag: A,
		body: { capabilities: ["completion"], parameters: "temperature 0.7\nnum_ctx 4096\nstop <x>" },
	}),
	show("show-context-zero-is-none", { tag: A, body: { capabilities: ["completion"], parameters: "num_ctx 0" } }),
	show("show-context-line-that-does-not-read-is-none", {
		tag: A,
		body: { capabilities: ["completion"], parameters: "num_ctx big" },
	}),
	show("show-trained-needs-a-positive-number", {
		tag: A,
		body: {
			capabilities: ["completion"],
			model_info: { "general.architecture": "arch-a", "arch-a.context_length": 0 },
		},
	}),
	show("show-trained-negative-is-none", {
		tag: A,
		body: {
			capabilities: ["completion"],
			model_info: { "general.architecture": "arch-a", "arch-a.context_length": -4 },
		},
	}),
	show("show-trained-of-another-architecture-is-none", {
		tag: A,
		body: {
			capabilities: ["completion"],
			model_info: { "general.architecture": "arch-b", "arch-a.context_length": 4096 },
		},
	}),
	show("show-trained-key-without-architecture-is-none", {
		tag: A,
		body: { capabilities: ["completion"], model_info: { "arch-a.context_length": 4096 } },
	}),
	show("show-trained-text-is-none", {
		tag: A,
		body: {
			capabilities: ["completion"],
			model_info: { "general.architecture": "arch-a", "arch-a.context_length": "4096" },
		},
	}),
	show("show-model-info-values-of-any-type-are-fine", {
		tag: A,
		body: {
			capabilities: ["completion"],
			model_info: {
				"general.architecture": "arch-a",
				"arch-a.context_length": 4096,
				other: [1, { x: 2 }],
				more: null,
			},
		},
	}),
	...NOT_OBJECTS.map(([name, body]) => show(`show-body-${name}`, { tag: A, body })),
	show("show-extra-fields-are-ignored", {
		tag: A,
		body: { capabilities: ["completion"], details: { family: "x" }, license: "y" },
	}),

	// The num_ctx line.
	numCtx("num-ctx-a-parameters-block", {
		texts: ["num_ctx                        262144\ntemperature                    0.6"],
	}),
	numCtx("num-ctx-the-first-line-that-reads-wins", {
		texts: ["num_ctx 4096\nnum_ctx 8192", "temperature 0.7\nnum_ctx 4096", "num_ctx 0\nnum_ctx 8192"],
	}),
	numCtx("num-ctx-end-of-the-text-ends-the-line", {
		texts: ["num_ctx 4096", "num_ctx 4096 ", "num_ctx\t4096\t", "num_ctx 4096 ", "num_ctx 4096 \n"],
	}),
	numCtx("num-ctx-any-line-end-ends-the-line", {
		texts: ["num_ctx 4096\nx", "num_ctx 4096\rx", "num_ctx 4096\r\nx", "num_ctx 4096 x", "num_ctx 4096 x"],
	}),
	numCtx("num-ctx-blanks-then-a-line-end", {
		texts: ["num_ctx 4096  \nx", "num_ctx 4096  　\nx", "num_ctx 4096 \n\n x"],
	}),
	numCtx("num-ctx-the-number-may-follow-on-a-later-line", {
		texts: ["num_ctx\n4096", "num_ctx  \n 4096", "num_ctx\n\n4096\n", "num_ctx\r\n4096"],
	}),
	numCtx("num-ctx-more-after-the-number-on-the-line-is-no-match", {
		texts: ["num_ctx 4096 x", "num_ctx 4096x", "num_ctx 40.5", "num_ctx 1e5", "num_ctx 4096,"],
	}),
	numCtx("num-ctx-a-later-line-reads-after-a-line-that-does-not", {
		texts: ["num_ctx 4096 x\nnum_ctx 8192", "num_ctx x\nnum_ctx 8192"],
	}),
	numCtx("num-ctx-needs-blanks-before-the-number", {
		texts: ["num_ctx4096", "num_ctx", "num_ctx ", "num_ctx\n", "num_ctx:4096"],
	}),
	numCtx("num-ctx-starts-a-line", {
		texts: ["  num_ctx 4096", "x num_ctx 4096", "xnum_ctx 4096", "\tnum_ctx 4096"],
	}),
	numCtx("num-ctx-a-line-start-follows-any-line-end", {
		texts: ["x\nnum_ctx 5", "x\rnum_ctx 5", "x\r\nnum_ctx 5", "x num_ctx 5", "x num_ctx 5", "\nnum_ctx 5"],
	}),
	numCtx("num-ctx-name-is-exact", {
		texts: ["NUM_CTX 4096", "Num_Ctx 4096", "num_ctx_x 4096", "num-ctx 4096", "numctx 4096"],
	}),
	numCtx("num-ctx-zero-and-signs", {
		texts: ["num_ctx 0", "num_ctx 00", "num_ctx -5", "num_ctx +5", "num_ctx 007", "num_ctx 0\nnum_ctx 4096"],
	}),
	numCtx("num-ctx-big-numbers", {
		texts: ["num_ctx 99999999999999999999", "num_ctx 9007199254740993", "num_ctx 1000000"],
	}),
	numCtx("num-ctx-a-number-no-double-holds-is-none", { texts: [`num_ctx ${"9".repeat(400)}`] }),
	numCtx("num-ctx-empty-text", { texts: ["", "\n", "\n\n"] }),
	numCtx("num-ctx-other-digits-are-not-digits", {
		texts: ["num_ctx ٤٢", "num_ctx ４２", "num_ctx 4٢"],
	}),

	// Ollama's running list.
	running("running-contexts-by-name", {
		body: listOf("models")(PS_ITEM, { name: B, model: B, context_length: 8192 }),
	}),
	running("running-a-model-without-a-context-is-left-out", {
		body: listOf("models")({ name: A }, { name: B, context_length: 4096 }),
	}),
	running("running-context-null-is-left-out", { body: listOf("models")({ name: A, context_length: null }) }),
	running("running-context-wrong-type-is-left-out", { body: listOf("models")({ name: A, context_length: "4096" }) }),
	running("running-name-omitted-is-left-out", { body: listOf("models")({ context_length: 4096 }) }),
	running("running-name-wrong-type-is-left-out", { body: listOf("models")({ name: 7, context_length: 4096 }) }),
	running("running-an-empty-name-is-kept", { body: listOf("models")({ name: "", context_length: 4096 }) }),
	running("running-a-repeated-name-takes-the-later-context", {
		body: listOf("models")(
			{ name: A, context_length: 1 },
			{ name: B, context_length: 2 },
			{ name: A, context_length: 3 },
		),
	}),
	...BAD_ITEMS.map((item, at) =>
		running(`running-item-${at}-not-an-object-is-left-out`, { body: listOf("models")(item, PS_ITEM) }),
	),
	running("running-models-omitted-is-empty", { body: { data: [PS_ITEM] } }),
	running("running-models-null-is-empty", { body: { models: null } }),
	running("running-models-wrong-type-is-empty", { body: { models: "x" } }),
	...NOT_OBJECTS.map(([name, body]) => running(`running-body-${name}-is-empty`, { body })),
	running("running-nothing-runs", { body: { models: [] } }),

	// LM Studio's list.
	lmModels("lmstudio-full", { body: listOf("models")(LM_ITEM) }),
	lmModels("lmstudio-minimal", { body: listOf("models")({ key: "local-b", type: "llm" }) }),
	lmModels("lmstudio-chat-models-only", {
		body: listOf("models")(
			LM_ITEM,
			{ key: "embed-a", type: "embedding", loaded_instances: [] },
			{ key: "local-b", type: "llm", max_context_length: 4096, loaded_instances: [] },
		),
	}),
	...siteCases(
		(name, input) => lmModels(name, { body: listOf("models")(input) }),
		LM_ITEM,
		LM_SITES,
		"lmstudio",
		(item) => item,
	),
	lmModels("lmstudio-reasoning-off-only-does-not-think", {
		body: listOf("models")(setIn(LM_ITEM, ["capabilities", "reasoning", "allowed_options"], ["off"])),
	}),
	lmModels("lmstudio-reasoning-on-only-offers-low-medium-high", {
		body: listOf("models")(setIn(LM_ITEM, ["capabilities", "reasoning", "allowed_options"], ["on"])),
	}),
	lmModels("lmstudio-reasoning-named-levels", {
		body: listOf("models")(
			setIn(LM_ITEM, ["capabilities", "reasoning", "allowed_options"], ["off", "low", "high"]),
		),
	}),
	lmModels("lmstudio-reasoning-unknown-option-thinks-with-none-only", {
		body: listOf("models")(setIn(LM_ITEM, ["capabilities", "reasoning", "allowed_options"], ["off", "bogus"])),
	}),
	lmModels("lmstudio-reasoning-no-options-does-not-think", {
		body: listOf("models")(setIn(LM_ITEM, ["capabilities", "reasoning", "allowed_options"], [])),
	}),
	lmModels("lmstudio-two-instances-the-first-gives-the-context", {
		body: listOf("models")(
			setIn(
				LM_ITEM,
				["loaded_instances"],
				[{ config: { context_length: 4096 } }, { config: { context_length: 8192 } }],
			),
		),
	}),
	lmModels("lmstudio-first-instance-without-a-context-gives-none", {
		body: listOf("models")(
			setIn(LM_ITEM, ["loaded_instances"], [{ config: {} }, { config: { context_length: 8192 } }]),
		),
	}),
	lmModels("lmstudio-no-instances", { body: listOf("models")(setIn(LM_ITEM, ["loaded_instances"], [])) }),
	lmModels("lmstudio-key-omitted-drops", {
		body: listOf("models")(cut(LM_ITEM, ["key"]), { key: "local-b", type: "llm" }),
	}),
	lmModels("lmstudio-key-null-drops", {
		body: listOf("models")(setIn(LM_ITEM, ["key"], null), { key: "local-b", type: "llm" }),
	}),
	lmModels("lmstudio-key-wrong-type-drops", {
		body: listOf("models")(setIn(LM_ITEM, ["key"], 7), { key: "local-b", type: "llm" }),
	}),
	lmModels("lmstudio-key-empty-drops", {
		body: listOf("models")(setIn(LM_ITEM, ["key"], ""), { key: "local-b", type: "llm" }),
	}),
	lmModels("lmstudio-type-omitted-drops", {
		body: listOf("models")(cut(LM_ITEM, ["type"]), { key: "local-b", type: "llm" }),
	}),
	lmModels("lmstudio-type-null-drops", {
		body: listOf("models")(setIn(LM_ITEM, ["type"], null), { key: "local-b", type: "llm" }),
	}),
	lmModels("lmstudio-type-wrong-type-drops", {
		body: listOf("models")(setIn(LM_ITEM, ["type"], 7), { key: "local-b", type: "llm" }),
	}),
	lmModels("lmstudio-type-other-text-drops", {
		body: listOf("models")(setIn(LM_ITEM, ["type"], "vlm"), setIn(LM_ITEM, ["type"], "LLM"), {
			key: "local-b",
			type: "llm",
		}),
	}),
	...BAD_ITEMS.map((item, at) =>
		lmModels(`lmstudio-item-${at}-not-an-object-drops`, {
			body: listOf("models")(item, { key: "local-b", type: "llm" }),
		}),
	),
	lmModels("lmstudio-models-omitted", { body: { data: [LM_ITEM] } }),
	lmModels("lmstudio-models-null", { body: { models: null } }),
	lmModels("lmstudio-models-wrong-type", { body: { models: "x" } }),
	...NOT_OBJECTS.map(([name, body]) => lmModels(`lmstudio-body-${name}`, { body })),
	lmModels("lmstudio-order-and-repeats-are-kept", {
		body: listOf("models")({ key: "b", type: "llm" }, { key: "a", type: "llm" }, { key: "b", type: "llm" }),
	}),
	lmModels("lmstudio-extra-fields-are-ignored", {
		body: listOf("models")({ ...(LM_ITEM as object), publisher: "lab", format: "gguf" }),
	}),

	// LM Studio's loaded contexts.
	lmLoaded("lmstudio-loaded-by-key", {
		body: listOf("models")(
			LM_ITEM,
			{ key: "local-b", type: "llm", loaded_instances: [{ config: { context_length: 4096 } }] },
			{ key: "local-c", type: "llm", loaded_instances: [] },
		),
	}),
	lmLoaded("lmstudio-loaded-embeddings-are-left-out", {
		body: listOf("models")(
			{ key: "embed-a", type: "embedding", loaded_instances: [{ config: { context_length: 512 } }] },
			LM_ITEM,
		),
	}),
	lmLoaded("lmstudio-loaded-a-bad-instance-list-gives-none", {
		body: listOf("models")(setIn(LM_ITEM, ["loaded_instances"], ["x"])),
	}),
	lmLoaded("lmstudio-loaded-a-repeated-key-takes-the-later", {
		body: listOf("models")(LM_ITEM, setIn(LM_ITEM, ["loaded_instances"], [{ config: { context_length: 2048 } }])),
	}),
	lmLoaded("lmstudio-loaded-unreadable-list-is-empty", { body: { data: [] } }),
	lmLoaded("lmstudio-loaded-body-null-is-empty", { body: null }),

	// llama.cpp's list.
	llamaModels("llamacpp-full", { body: { object: "list", data: [LLAMA_ITEM] } }),
	llamaModels("llamacpp-minimal", { body: { data: [{ id: "local-b.gguf" }] } }),
	...siteCases(
		(name, input) => llamaModels(name, { body: listOf("data")(input) }),
		LLAMA_ITEM,
		LLAMA_SITES,
		"llamacpp",
		(item) => item,
	),
	llamaModels("llamacpp-meta-and-router-length-both-missing", {
		body: listOf("data")({ id: "local-a.gguf", meta: {} }),
	}),
	llamaModels("llamacpp-the-meta-length-wins-over-the-router-length", {
		body: listOf("data")({ id: "local-a.gguf", meta: { n_ctx_train: 4096 }, context_length: 8192 }),
	}),
	llamaModels("llamacpp-the-router-length-fills-a-missing-meta-length", {
		body: listOf("data")({ id: "local-a.gguf", meta: null, context_length: 8192 }),
	}),
	llamaModels("llamacpp-id-omitted-drops", { body: listOf("data")({}, { id: "local-b.gguf" }) }),
	llamaModels("llamacpp-id-null-drops", { body: listOf("data")({ id: null }, { id: "local-b.gguf" }) }),
	llamaModels("llamacpp-id-wrong-type-drops", { body: listOf("data")({ id: 7 }, { id: "local-b.gguf" }) }),
	llamaModels("llamacpp-id-empty-drops", { body: listOf("data")({ id: "" }, { id: "local-b.gguf" }) }),
	...BAD_ITEMS.map((item, at) =>
		llamaModels(`llamacpp-item-${at}-not-an-object-drops`, { body: listOf("data")(item, { id: "local-b.gguf" }) }),
	),
	llamaModels("llamacpp-data-omitted", { body: { models: [{ id: "local-a.gguf" }] } }),
	llamaModels("llamacpp-data-null", { body: { data: null } }),
	llamaModels("llamacpp-data-wrong-type", { body: { data: "x" } }),
	...NOT_OBJECTS.map(([name, body]) => llamaModels(`llamacpp-body-${name}`, { body })),
	llamaModels("llamacpp-order-and-repeats-are-kept", { body: listOf("data")({ id: "b" }, { id: "a" }, { id: "b" }) }),

	// llama.cpp's props.
	llamaProps("props-full", { body: PROPS }),
	llamaProps("props-empty-object", { body: {} }),
	...siteCases(
		(name, input) => llamaProps(name, { body: input }),
		PROPS,
		PROPS_SITES,
		"props",
		(item) => item,
	),
	...NOT_OBJECTS.map(([name, body]) => llamaProps(`props-body-${name}`, { body })),
	llamaProps("props-extra-fields-are-ignored", { body: { ...(PROPS as object), total_slots: 1 } }),
	llamaProps("props-levels-unknown-names-are-kept-as-text", {
		body: { chat_template_caps: { reasoning_effort_levels: ["low", "bogus"] } },
	}),

	// llama.cpp's model from a list item and its props.
	llamaModel("model-with-props", {
		id: "local-a.gguf",
		trained: 131_072,
		props: { context: 65_536, efforts: ["low", "high", "bogus"] },
	}),
	llamaModel("model-props-without-efforts-do-not-think", {
		id: "local-a.gguf",
		props: { context: 4096, efforts: [] },
	}),
	llamaModel("model-props-without-a-context", { id: "local-a.gguf", props: { efforts: ["low"] } }),
	llamaModel("model-props-with-a-context-replace-the-known-one", {
		id: "local-a.gguf",
		props: { context: 4096, efforts: ["low"] },
		known: { tag: "local-a.gguf", tools: true, thinking: true, efforts: ["high"], context: 8192 },
	}),
	llamaModel("model-props-without-a-context-drop-the-known-one", {
		id: "local-a.gguf",
		props: { efforts: [] },
		known: { tag: "local-a.gguf", tools: true, thinking: true, efforts: ["high"], context: 8192 },
	}),
	llamaModel("model-asleep-carries-the-known-facts", {
		id: "local-a.gguf",
		trained: 131_072,
		known: { tag: "local-a.gguf", tools: true, thinking: true, efforts: ["low", "high"], context: 8192 },
	}),
	llamaModel("model-asleep-and-unknown", { id: "local-b.gguf" }),
	llamaModel("model-asleep-the-known-context-may-be-none", {
		id: "local-a.gguf",
		known: { tag: "local-a.gguf", tools: true, thinking: false, efforts: [] },
	}),
	llamaModel("model-none-only-effort-does-not-think", {
		id: "local-a.gguf",
		props: { efforts: ["none"] },
	}),
	llamaModel("model-asleep-known-facts-win-over-nothing-but-the-trained-length-is-new", {
		id: "local-a.gguf",
		trained: 1024,
		known: { tag: "local-a.gguf", tools: true, thinking: true, efforts: ["low"], context: 8192, trained: 4096 },
	}),

	// The OpenAI-compatible list.
	compatModels("compatible-full", { body: { object: "list", data: [COMPAT_ITEM] } }),
	compatModels("compatible-minimal", { body: listOf("data")({ id: "local-b" }) }),
	...siteCases(
		(name, input) => compatModels(name, { body: listOf("data")(input) }),
		COMPAT_ITEM,
		COMPAT_SITES,
		"compatible",
		(item) => item,
	),
	compatModels("compatible-the-first-length-that-reads-wins", {
		body: listOf("data")(
			{ id: "a", context_length: 1, max_context_length: 2, max_model_len: 3 },
			{ id: "b", context_length: null, max_context_length: 2, max_model_len: 3 },
			{ id: "c", context_length: "1", max_context_length: null, max_model_len: 3 },
			{ id: "d", max_model_len: 3 },
			{ id: "e" },
		),
	}),
	compatModels("compatible-a-zero-length-is-missing", {
		body: listOf("data")({ id: "a", context_length: 0, max_model_len: 3 }),
	}),
	compatModels("compatible-id-omitted-drops", { body: listOf("data")({ context_length: 4 }, { id: "b" }) }),
	compatModels("compatible-id-null-drops", { body: listOf("data")({ id: null }, { id: "b" }) }),
	compatModels("compatible-id-wrong-type-drops", { body: listOf("data")({ id: 7 }, { id: "b" }) }),
	compatModels("compatible-id-empty-drops", { body: listOf("data")({ id: "" }, { id: "b" }) }),
	...BAD_ITEMS.map((item, at) =>
		compatModels(`compatible-item-${at}-not-an-object-drops`, { body: listOf("data")(item, { id: "b" }) }),
	),
	compatModels("compatible-data-omitted", { body: { models: [{ id: "a" }] } }),
	compatModels("compatible-data-null", { body: { data: null } }),
	compatModels("compatible-data-wrong-type", { body: { data: "x" } }),
	...NOT_OBJECTS.map(([name, body]) => compatModels(`compatible-body-${name}`, { body })),
	compatModels("compatible-order-and-repeats-are-kept", {
		body: listOf("data")({ id: "b" }, { id: "a" }, { id: "b" }),
	}),
	compatModels("compatible-every-model-may-call-tools-and-none-thinks", { body: listOf("data")({ id: "a" }) }),

	// Which hosts are this network.
	compatTerms("hosts-loopback-and-names-of-this-network", {
		urls: [
			"http://127.0.0.1:8000",
			"http://localhost:8000",
			"http://sub.localhost",
			"http://[::1]:8000",
			"http://labhost:8000",
			"http://lab.local:8000",
			"http://lab.lan",
			"http://lab.internal",
			"http://lab.home.arpa",
			"http://a.b.local",
		],
	}),
	compatTerms("hosts-private-ranges", {
		urls: [
			"http://10.1.2.3:8000",
			"http://10.0.0.1",
			"http://172.16.0.5",
			"http://172.20.0.5",
			"http://172.31.255.1",
			"http://192.168.7.9",
			"http://169.254.1.1",
			"http://100.64.0.1",
			"http://100.100.5.5",
			"http://100.127.255.255",
		],
	}),
	compatTerms("hosts-just-outside-the-private-ranges", {
		urls: [
			"http://172.15.0.5",
			"http://172.32.0.5",
			"http://192.169.1.5",
			"http://169.253.1.1",
			"http://100.63.255.255",
			"http://100.128.0.1",
			"http://11.0.0.1",
			"http://126.0.0.1",
			"http://8.8.8.8",
		],
	}),
	compatTerms("hosts-ipv6", {
		urls: [
			"http://[fd00::1]",
			"http://[fc00::1]",
			"http://[fe80::1]",
			"http://[fe9a::1]",
			"http://[feab::1]",
			"http://[febf::1]",
			"http://[fec0::1]",
			"http://[2001:db8::1]:8000",
			"http://[::2]",
			"http://[::ffff:127.0.0.1]",
			"http://[::]",
		],
	}),
	compatTerms("hosts-look-alikes-are-remote", {
		urls: [
			"http://10.example.com",
			"http://127.evil.example",
			"http://127.0.0.1.evil.example",
			"http://192.168.7.9.evil.example",
			"http://api.example.com",
			"http://localhost.example.com",
			"http://example.local.example.com",
			"https://host.lab.example/api/v2",
			"http://labhost.",
			"http://lab.local.",
		],
	}),
	compatTerms("hosts-the-parser-writes-the-host", {
		urls: [
			"http://LAB.LOCAL",
			"HTTP://LABHOST:8000",
			"http://[FD00::1]",
			"http://0x7f.1",
			"http://2130706433",
			"http://10.1",
			"http://012.0.0.1",
		],
	}),
	compatTerms("hosts-no-url-is-remote", { urls: [null] }),
	compatTerms("hosts-an-url-that-does-not-read-is-remote", {
		urls: ["", "not a url", "labhost:8000", "ftp://lab.local", "file:///x", "http://"],
	}),
	compatTerms("hosts-a-host-the-parser-refuses-is-remote", {
		urls: ["http://büro.local", "http://bo%78.local", "http://テスト.local"],
	}),
	compatTerms("hosts-path-port-and-credentials-do-not-change-the-host", {
		urls: [
			"http://lab.local:9/a/b",
			"https://user:pw@labhost/x",
			"http://host.example:80/x",
			"http://lab.local?x=1#y",
		],
	}),

	// A length is a number above zero.
	lmModels("lmstudio-a-zero-or-negative-trained-length-is-missing", {
		body: listOf("models")(
			setIn(LM_ITEM, ["max_context_length"], 0),
			setIn(setIn(LM_ITEM, ["key"], "local-b"), ["max_context_length"], -1),
		),
	}),
	lmModels("lmstudio-a-zero-or-negative-loaded-length-is-missing", {
		body: listOf("models")(
			setIn(LM_ITEM, ["loaded_instances"], [{ config: { context_length: 0 } }]),
			setIn(setIn(LM_ITEM, ["key"], "local-b"), ["loaded_instances"], [{ config: { context_length: -8 } }]),
		),
	}),
	lmLoaded("lmstudio-loaded-a-zero-length-is-left-out", {
		body: listOf("models")(setIn(LM_ITEM, ["loaded_instances"], [{ config: { context_length: 0 } }])),
	}),
	compatModels("compatible-a-negative-length-lets-the-next-field-supply-it", {
		body: listOf("data")({ id: "a", context_length: -1, max_context_length: 0, max_model_len: 3 }),
	}),
	compatModels("compatible-every-length-missing-with-zeros", {
		body: listOf("data")({ id: "a", context_length: 0, max_context_length: -2, max_model_len: 0 }),
	}),
	llamaModels("llamacpp-a-zero-trained-length-lets-the-router-length-supply-it", {
		body: listOf("data")({ id: "a", meta: { n_ctx_train: 0 }, context_length: 4096 }),
	}),
	llamaModels("llamacpp-zero-lengths-are-missing", {
		body: listOf("data")({ id: "a", meta: { n_ctx_train: -1 }, context_length: 0 }),
	}),
	llamaProps("props-a-zero-or-negative-slot-length-is-missing", {
		body: { default_generation_settings: { n_ctx: 0 }, chat_template_caps: {} },
	}),
	llamaProps("props-a-negative-slot-length-is-missing", {
		body: { default_generation_settings: { n_ctx: -4 } },
	}),
	running("running-a-zero-or-negative-context-is-left-out", {
		body: listOf("models")(
			{ name: A, context_length: 0 },
			{ name: B, context_length: -1 },
			{ name: "c", context_length: 8 },
		),
	}),
	view("view-a-zero-context-lets-the-loaded-one-supply-it", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [], context: 0 },
		loaded: 4096,
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),
	view("view-a-zero-trained-length-caps-nothing", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [], context: 8192, trained: 0 },
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),
	view("view-no-positive-length-gives-no-limit", {
		row: `local:lab/${A}`,
		model: { tag: A, tools: true, thinking: false, efforts: [], context: -1, trained: 4096 },
		loaded: 0,
		terms: { kept: false, price: "free" },
		zdrOnly: true,
	}),

	// The network of a host: unique-local is fc00::/7, link-local fe80::/10, a trailing dot is the same host.
	compatTerms("hosts-ipv6-judged-on-the-first-piece-as-a-number", {
		urls: [
			"http://[fc::1]",
			"http://[fd::1]",
			"http://[fda::1]",
			"http://[fe8::1]",
			"http://[fc00::1]",
			"http://[fdff:ffff::1]",
			"http://[fe80::1]",
			"http://[febf::1]",
			"http://[fec0::1]",
			"http://[fbff::1]",
			"http://[fc00::]",
			"http://[1fc0::1]",
			"http://[fe8:0:0:0:0:0:0:1]",
		],
	}),
	compatTerms("hosts-unspecified-addresses-are-this-machine", {
		urls: [
			"http://0.0.0.0",
			"http://0.0.0.0:8000",
			"http://[::]",
			"http://[::]:8000",
			"http://0",
			"http://0.0.0.1",
		],
	}),
	compatTerms("hosts-a-trailing-dot-is-the-same-host", {
		urls: [
			"http://localhost./",
			"http://lab.local./",
			"http://labhost./",
			"http://lab.lan.",
			"http://example.com./",
			"http://api.example.com.",
			"http://127.0.0.1./",
			"http://8.8.8.8./",
		],
	}),
	compatTerms("hosts-one-label-and-local-names-stay-private", {
		urls: ["http://ai/", "http://labhost", "http://evil.com.local/", "http://a.b.internal", "http://example.com"],
	}),
];
