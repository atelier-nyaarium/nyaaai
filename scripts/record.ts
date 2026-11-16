// Records a real provider exchange as a trace. Paid: run by hand.
// Run: bun scripts/record.ts openrouter [tool-loop|title|no-zdr]   (the key on standard input)

import { mkdirSync, writeFileSync } from "node:fs";
import path from "node:path";
import { OPENROUTER_CHAT } from "@nyaaai/agentic/dialects/openrouter";
import { failureOf } from "@nyaaai/agentic/failure";
import { platformFetch } from "@nyaaai/agentic/fetch";
import { replyCalls, replyText, replyThoughts, StepAssembler } from "@nyaaai/agentic/step";
import { staticKey, Transport } from "@nyaaai/agentic/transport";
import type { Fetch } from "@nyaaai/contract/fetch";
import { randomUuid } from "@nyaaai/contract/id";
import { parseJson } from "@nyaaai/contract/json";
import type { CanonicalRequest, StepResult, ToolSpec, Turn } from "@nyaaai/contract/shapes/step";
import { type Scheduler, systemScheduler } from "@nyaaai/contract/time";
import { recordingFetch } from "@nyaaai/testkit/recorder";
import { ROOT } from "./child";
import { replaceTraceIds } from "./traceIds";

////////////////////////////////
//  Interfaces & Types

export interface Scenario {
	name: string;
	run: (transport: Transport, session: string) => Promise<void>;
}

////////////////////////////////
//  Constants

export const BASE_URL = "https://openrouter.ai/api/v1";
const MODEL = "openai/gpt-6-luna";
/** No zero-retention endpoint. */
const NO_ZDR_MODEL = "anthropic/claude-fable-5.1";
const STEP_MS = 120_000;
const LOOKUP_TOOL: ToolSpec = {
	name: "lookup",
	description: "Look up the value stored under a key.",
	parameters: {
		type: "object",
		properties: { key: { type: "string" } },
		required: ["key"],
		additionalProperties: false,
	},
};

////////////////////////////////
//  Functions & Helpers

function parsedOrText(text: string): unknown {
	return parseJson(text) ?? text;
}

export function openRouterTransport(fetch: Fetch, key: string, scheduler: Scheduler): Transport {
	return new Transport({
		fetch,
		scheduler,
		key: staticKey(key),
		log: () => {},
		baseUrl: BASE_URL,
		label: "OpenRouter",
		userAgent: "nyaaai-record/0",
		errors: OPENROUTER_CHAT,
		auth: "bearer",
	});
}

function requestOf(session: string, model: string, history: Turn[], extra: Partial<CanonicalRequest> = {}) {
	return {
		model,
		system: [],
		history,
		cache: { kind: "none" },
		session,
		zdrOnly: true,
		...extra,
	} satisfies CanonicalRequest;
}

async function step(transport: Transport, request: CanonicalRequest): Promise<StepResult> {
	const assembler = new StepAssembler();
	for await (const event of transport.stream(OPENROUTER_CHAT, request, AbortSignal.timeout(STEP_MS)))
		assembler.add(event);
	return assembler.result();
}

async function toolLoop(transport: Transport, session: string): Promise<void> {
	const extra = { tools: [LOOKUP_TOOL], effort: "low" } satisfies Partial<CanonicalRequest>;
	const history: Turn[] = [
		{
			role: "user",
			text: `Use the lookup tool for the keys "alpha" and "beta", in parallel if you can. Then reply with both values.`,
		},
	];
	const first = await step(transport, requestOf(session, MODEL, history, extra));
	const calls = replyCalls(first);
	if (calls.length === 0) throw new Error("The first step made no tool call.");
	const text = replyText(first);
	history.push({
		role: "step",
		producer: { model: MODEL, ...(first.serving.host === undefined ? {} : { host: first.serving.host }) },
		parts: [
			...replyThoughts(first),
			...(text.length > 0 ? [{ kind: "text" as const, text }] : []),
			...calls.flatMap((call) => {
				const key = String((parsedOrText(call.argumentsText) as { key?: unknown } | null)?.key);
				return [
					{
						kind: "call" as const,
						index: call.index,
						wireId: call.wireId,
						name: call.name,
						arguments: call.argumentsText,
					},
					{ kind: "result" as const, index: call.index, output: `value-of-${key}`, isError: false },
				];
			}),
		],
	});
	await step(transport, requestOf(session, MODEL, history, extra));
}

async function title(transport: Transport, session: string): Promise<void> {
	const result = await transport.complete(
		OPENROUTER_CHAT,
		requestOf(
			session,
			MODEL,
			[
				{
					role: "user",
					text: "Name this conversation in a few words: Rename the tally helper and update its callers.",
				},
			],
			{
				effort: "low",
				schema: {
					name: "title",
					json: {
						type: "object",
						properties: { title: { type: "string" } },
						required: ["title"],
						additionalProperties: false,
					},
				},
			},
		),
		AbortSignal.timeout(STEP_MS),
	);
	if (typeof (parsedOrText(replyText(result)) as { title?: unknown } | null)?.title !== "string")
		throw new Error("The title reply did not parse.");
}

async function noZdr(transport: Transport, session: string): Promise<void> {
	try {
		await step(transport, requestOf(session, NO_ZDR_MODEL, [{ role: "user", text: "Say hi." }]));
	} catch (error) {
		const failure = failureOf(error);
		if (failure === undefined) throw error;
		if (failure.status === 404) return;
		throw new Error(`Expected HTTP 404, got ${failure.status}.`);
	}
	throw new Error("Expected HTTP 404, got a stream.");
}

export const SCENARIOS: Scenario[] = [
	{ name: "tool-loop", run: toolLoop },
	{ name: "title", run: title },
	{ name: "no-zdr", run: noZdr },
];

/** The trace file text. */
export async function recordScenario(
	scenario: Scenario,
	inner: Fetch,
	key: string,
	session: string,
	scheduler: Scheduler = systemScheduler,
): Promise<string> {
	const recorder = recordingFetch(inner, [key]);
	await scenario.run(openRouterTransport(recorder.fetch, key, scheduler), session);
	return `${JSON.stringify(replaceTraceIds(recorder.trace()), null, "\t")}\n`;
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const [provider, only, ...rest] = process.argv.slice(2);
	const chosen = only === undefined ? SCENARIOS : SCENARIOS.filter((scenario) => scenario.name === only);
	if (provider !== "openrouter" || rest.length > 0 || chosen.length === 0) {
		console.error(
			`usage: bun scripts/record.ts openrouter [${SCENARIOS.map((scenario) => scenario.name).join("|")}]`,
		);
		process.exit(1);
	}
	const key = (await Bun.stdin.text()).trim();
	if (key.length === 0) {
		console.error("No OpenRouter key on standard input.");
		process.exit(1);
	}
	const folder = path.join(ROOT, "cases", "traces");
	let failed = 0;
	for (const scenario of chosen) {
		try {
			const text = await recordScenario(scenario, platformFetch(), key, randomUuid());
			mkdirSync(folder, { recursive: true });
			const file = path.join(folder, `openrouter.${scenario.name}.json`);
			writeFileSync(file, text);
			console.log(`wrote ${path.relative(ROOT, file)}`);
		} catch (error) {
			failed += 1;
			const message = error instanceof Error ? error.message : "the scenario failed";
			console.error(`openrouter ${scenario.name}: ${message.split(key).join("[key]")}`);
		}
	}
	process.exit(failed === 0 ? 0 : 1);
}
