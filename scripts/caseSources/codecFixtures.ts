// Builders and conformance fixtures for the codec cases. Values are made up.

////////////////////////////////
//  Interfaces & Types

export type Json = Record<string, unknown>;

/** What a codec is run against: its name, a model it knows and one it does not. */
export interface Fixture {
	name: string;
	codec: Json;
	/** Reads the step's thoughts. */
	model: string;
	/** Reads none of them. */
	foreign: string;
	stream: Json[];
	/** The unstreamed reply. */
	reply: Json;
	/** What both of them assemble to. */
	result: Json;
	/** No end marker. */
	cut: Json[];
	/** An in-stream failure after output. */
	failing: Json[];
	/** HTTP status, body, and the category it decodes to. */
	failures: ReadonlyArray<readonly [number, unknown, string]>;
}

////////////////////////////////
//  Constants

export const LUNA = "openai/gpt-6-luna";
export const OPUS = "anthropic/claude-opus-5.5";
export const DEEPSEEK = "deepseek-v4.1-flash";

/** A scripted instant: 2027-01-15T08:00:00Z. */
export const NOW = 1_800_000_000_000;

export const DONE: Json = { kind: "done" };

export const CHAT_API = { family: "chat-completions", structured: false, choice: [], strict: false };

export const DEEPSEEK_API = {
	family: "chat-completions",
	structured: true,
	choice: ["auto", "none", "required", "tool"],
	strict: false,
	reasoning: "reasoning_content",
};

export const OPENROUTER: Json = { family: "chat-completions", dialect: "openrouter" };

export const OPENCODE_BARE: Json = { family: "chat-completions", dialect: "opencode" };

const READ_ARGS = '{"path":"a"}';
const CALL = { kind: "call", index: 0, wireId: "call_a", name: "read", argumentsText: READ_ARGS };
const OVERLOADED = "[overloaded] the server is overloaded; retry shortly";

////////////////////////////////
//  Functions & Helpers

export function opencode(over: { efforts?: string[]; api?: Json; nowMs?: number } = {}): Json {
	return {
		family: "chat-completions",
		dialect: "opencode",
		facts: { api: { ...DEEPSEEK_API, ...over.api }, efforts: over.efforts ?? ["high"] },
		...(over.nowMs === undefined ? {} : { nowMs: over.nowMs }),
	};
}

export function local(
	spec: Json = {},
	efforts: string[] = ["none", "low", "medium", "high"],
	api: Json = CHAT_API,
): Json {
	const { free, ...rest } = spec;
	return {
		family: "chat-completions",
		dialect: "local",
		facts: { api, efforts, ...(free === true ? { free } : {}) },
		local: { host: "Lab", label: "Ollama", ...rest },
	};
}

/** One data item holding JSON, or text as it came. */
export function data(chunk: Json | string): Json {
	return { kind: "data", data: typeof chunk === "string" ? chunk : JSON.stringify(chunk) };
}

export const comment = (text: string): Json => ({ kind: "comment", text });

export function text(content: unknown): Json {
	return { id: "gen-1", provider: "Azure", choices: [{ index: 0, delta: { role: "assistant", content } }] };
}

export function call(index: number, part: { id?: string; name?: string; args?: string }): Json {
	return {
		id: "gen-1",
		provider: "Azure",
		choices: [
			{
				index: 0,
				delta: {
					tool_calls: [{ index, id: part.id, function: { name: part.name, arguments: part.args ?? "" } }],
				},
			},
		],
	};
}

export function finish(reason: unknown, usage?: unknown): Json {
	return { id: "gen-1", choices: [{ index: 0, delta: { content: "" }, finish_reason: reason }], usage };
}

export const reasoned = (detail: Json): Json => ({
	id: "gen-1",
	provider: "Azure",
	choices: [{ delta: { reasoning_details: [detail] } }],
});

export const reasoningContent = (value: string): Json => ({
	id: "gen-1",
	choices: [{ delta: { reasoning_content: value } }],
});

export const reasoning = (value: string): Json => ({ id: "gen-1", choices: [{ delta: { reasoning: value } }] });

export function request(over: Json = {}): Json {
	return {
		model: LUNA,
		system: ["Rules."],
		history: [{ role: "user", text: "Hi" }],
		cache: { kind: "implicit" },
		session: "root-1",
		zdrOnly: true,
		...over,
	};
}

/** A step turn whose calls each carry a result. */
export function stepOf(result: Json, model: string): Json {
	const parts = (result["parts"] as Json[]).flatMap((part): Json[] =>
		part["kind"] === "call"
			? [
					{
						kind: "call",
						index: part["index"],
						wireId: part["wireId"],
						name: part["name"],
						arguments: part["argumentsText"],
					},
					{ kind: "result", index: part["index"], output: `ran ${part["name"]}`, isError: false },
				]
			: [part],
	);
	return { role: "step", producer: { model }, parts };
}

////////////////////////////////
//  Fixtures

const OPENROUTER_USAGE = { prompt_tokens: 4, completion_tokens: 2, cost: 0.1 };

const DETAILS = [
	{ type: "reasoning.text", index: 0, text: "Thinking.", signature: "sig_1" },
	{ type: "reasoning.encrypted", index: 1, data: "enc_2" },
];

const choice = (message: Json) => ({ message, finish_reason: "tool_calls" });

const wireCall = { id: "call_a", type: "function", function: { name: "read", arguments: READ_ARGS } };

export const OPENROUTER_FIXTURE: Fixture = {
	name: "openrouter",
	codec: OPENROUTER,
	model: LUNA,
	foreign: OPUS,
	stream: [
		data(reasoned({ type: "reasoning.text", index: 0, text: "Think" })),
		data(reasoned({ type: "reasoning.text", index: 0, text: "ing.", signature: "sig_1" })),
		data(reasoned({ type: "reasoning.encrypted", index: 1, data: "enc_2" })),
		data(text("Reading.")),
		data(call(0, { id: "call_a", name: "read", args: '{"path":' })),
		data(call(0, { args: '"a"}' })),
		data(finish("tool_calls", OPENROUTER_USAGE)),
		DONE,
	],
	reply: {
		id: "gen-1",
		provider: "Azure",
		choices: [choice({ content: "Reading.", reasoning_details: DETAILS, tool_calls: [wireCall] })],
		usage: OPENROUTER_USAGE,
	},
	result: {
		parts: [
			{
				kind: "thought",
				type: "reasoning.text",
				segments: [{ kind: "content", text: "Thinking." }],
				opaque: "sig_1",
				index: 0,
			},
			{ kind: "thought", type: "reasoning.encrypted", opaque: "enc_2", index: 1 },
			{ kind: "text", text: "Reading." },
			CALL,
		],
		finish: "tool_calls",
		usage: { input: 4, output: 2, cost: 0.1 },
		serving: { host: "Azure", generationId: "gen-1" },
	},
	cut: [data(text("Hi"))],
	failing: [
		data(text("Hi")),
		data({
			id: "gen-1",
			provider: "Azure",
			error: { code: 429, message: "Rate limit exceeded", metadata: { error_type: "rate_limit_exceeded" } },
			usage: OPENROUTER_USAGE,
		}),
		DONE,
	],
	failures: [
		[401, { error: { message: "No auth" } }, "keyRejected"],
		[402, { error: { message: "Insufficient credits" } }, "outOfCredit"],
		[400, { error: { message: "Reasoning must go to the provider that created it." } }, "replayRejected"],
	],
};

const OPENCODE_USAGE = {
	prompt_tokens: 100,
	completion_tokens: 10,
	prompt_tokens_details: { cached_tokens: 60 },
	completion_tokens_details: { reasoning_tokens: 4 },
	cost: 0.002,
	opencode: { billing_source: "go", tokens: { cache_write_5m: 5, cache_write_1h: 2 } },
};

export const OPENCODE_FIXTURE: Fixture = {
	name: "opencode",
	codec: opencode(),
	model: DEEPSEEK,
	foreign: "gpt-6-luna",
	stream: [
		data(reasoningContent("Think")),
		data(reasoningContent("ing.")),
		data(text("Reading.")),
		data(call(0, { id: "call_a", name: "read", args: '{"path":' })),
		data(call(0, { args: '"a"}' })),
		data(finish("tool_calls", OPENCODE_USAGE)),
		DONE,
	],
	reply: {
		id: "gen-1",
		choices: [choice({ content: "Reading.", reasoning_content: "Thinking.", tool_calls: [wireCall] })],
		usage: OPENCODE_USAGE,
	},
	result: {
		parts: [
			{ kind: "thought", type: "reasoning_content", segments: [{ kind: "content", text: "Thinking." }] },
			{ kind: "text", text: "Reading." },
			CALL,
		],
		finish: "tool_calls",
		usage: {
			input: 100,
			output: 10,
			cached: 60,
			cacheWrite: 7,
			reasoning: 4,
			cost: 0.002,
			billingSource: "go",
		},
		serving: { generationId: "gen-1" },
	},
	cut: [data(text("Hi"))],
	failing: [
		data(text("Hi")),
		data({ id: "gen-1", error: { type: "server_error", message: OVERLOADED }, usage: OPENROUTER_USAGE }),
		DONE,
	],
	failures: [
		[401, { type: "error", error: { type: "AuthError", message: "Invalid API key." } }, "keyRejected"],
		[429, { type: "error", error: { type: "GoUsageLimitError", message: "Usage limit reached." } }, "allowance"],
		[400, { error: { message: "Model is unavailable." } }, "notFound"],
	],
};

const LOCAL_USAGE = { prompt_tokens: 100, completion_tokens: 10 };

export const LOCAL_FIXTURE: Fixture = {
	name: "local",
	codec: local({ replay: "reasoning", free: true }),
	model: "local-a:27b",
	foreign: "local-b:9b",
	stream: [
		data(reasoning("Think")),
		data(reasoning("ing.")),
		data(text("Reading.")),
		data(call(0, { id: "call_a", name: "read", args: '{"path":' })),
		data(call(0, { args: '"a"}' })),
		data(finish("tool_calls", LOCAL_USAGE)),
		DONE,
	],
	reply: {
		id: "gen-1",
		choices: [choice({ content: "Reading.", reasoning: "Thinking.", tool_calls: [wireCall] })],
		usage: LOCAL_USAGE,
	},
	result: {
		parts: [
			{ kind: "thought", type: "reasoning", segments: [{ kind: "content", text: "Thinking." }] },
			{ kind: "text", text: "Reading." },
			CALL,
		],
		finish: "tool_calls",
		usage: { input: 100, output: 10, cost: 0 },
		serving: { host: "Lab", generationId: "gen-1" },
	},
	cut: [data(text("Hi"))],
	failing: [
		data(text("Hi")),
		data({ id: "gen-1", error: { type: "api_error", message: "the model failed to generate a response" } }),
		DONE,
	],
	failures: [
		[
			404,
			{ error: { type: "not_found_error", message: 'model "qwen" not found, try pulling it first' } },
			"notFound",
		],
		[400, { error: { type: "invalid_request_error", message: '"qwen" does not support thinking' } }, "badRequest"],
		[503, { error: "server busy, please try again.  maximum pending requests exceeded" }, "unavailable"],
	],
};

export const FIXTURES: Fixture[] = [OPENROUTER_FIXTURE, OPENCODE_FIXTURE, LOCAL_FIXTURE];
