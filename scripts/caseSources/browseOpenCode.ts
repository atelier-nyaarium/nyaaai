// OpenCode browser and tester cases, on a scripted network. Every value is made up.

import type { CaseSource } from "../caseAdapter";
import { failing, HOUR, type Json, NOW, reply } from "./catalogKit";
import { LIST_GO, listed } from "./catalogOpenCode";

////////////////////////////////
//  Functions & Helpers

const browser = (name: string, input: Record<string, unknown>): CaseSource => ({
	kind: "browse.opencodeBrowser",
	name,
	input: { startMs: NOW, plan: "go", ...input },
});

const list = (extra: Record<string, unknown> = {}) => ({ kind: "list", ...extra });
const detail = (id: string) => ({ kind: "detail", id });
const listedStep = (id: string) => ({ kind: "listed", id });
const view = (row: string, use?: string) => ({ kind: "view", row, ...(use === undefined ? {} : { use }) });

const data = (...items: unknown[]): Json => ({ data: items }) as Json;

const GO = reply(LIST_GO);

////////////////////////////////
//  Cases

export const browseOpenCodeCases: CaseSource[] = [
	browser("opencode-list-with-zdr-and-family", { exchanges: [GO], steps: [list()] }),
	browser("opencode-list-payg", { plan: "payg", exchanges: [GO], steps: [list()] }),
	browser("opencode-list-reads-once-in-twelve-hours", {
		exchanges: [GO, GO],
		steps: [list(), list(), { kind: "advance", ms: 12 * HOUR - 1 }, list(), { kind: "advance", ms: 1 }, list()],
	}),
	browser("opencode-list-failing-answers-nothing-and-fresh-retries", {
		exchanges: [failing(), GO],
		steps: [list(), list(), list({ fresh: true })],
	}),
	browser("opencode-list-leaves-out-unbound-suffixes", {
		exchanges: [
			reply(
				data(
					listed("beam-free"),
					listed("glm-latest"),
					listed("glm-5.3-flash"),
					listed("free-glm"),
					listed("glm-free-2"),
				),
			),
		],
		steps: [list()],
	}),
	browser("opencode-list-media-types-fold-to-their-kind", {
		exchanges: [
			reply(
				data({
					id: "glm-5.3-flash",
					name: "GLM Flash",
					description: "Fast.",
					modalities: {
						input: ["text", "image/png", "image/jpeg", "application/pdf", "audio", "image"],
						output: ["text", "audio/wav"],
					},
					limit: { input: 50, context: null },
					capabilities: {
						tools: { supported: true },
						reasoning: { supported: true },
						structured_output: true,
					},
					api: { messages: {} },
				}),
			),
		],
		steps: [list()],
	}),
	browser("opencode-list-context-prefers-context-then-input", {
		exchanges: [
			reply(
				data(
					listed("glm-a", { limit: { context: 10, input: 5 } }),
					listed("glm-b", { limit: { input: 5 } }),
					listed("glm-c", { limit: {} }),
					listed("glm-d", { limit: null }),
				),
			),
		],
		steps: [list()],
	}),
	browser("opencode-list-tools-none-and-nozdr", {
		exchanges: [
			reply(
				data(
					listed("glm-no-api", { api: {} }),
					listed("glm-no-tools", { capabilities: { tools: { supported: false } } }),
					listed("glm-ok"),
					listed("grok-4.7"),
					{ id: "glm-bare" },
				),
			),
		],
		steps: [list()],
	}),
	browser("opencode-list-a-model-is-curated-by-its-opencode-id", {
		exchanges: [reply(data(listed("gpt-6-luna"), listed("gpt-6.1-sol"), listed("openai/gpt-6-luna")))],
		steps: [list()],
	}),
	browser("opencode-detail-carries-retention-and-api-facts", {
		exchanges: [GO],
		steps: [detail("deepseek-v4.1-flash"), detail("grok-4.7"), detail("gpt-6-luna"), detail("minimax-m3")],
	}),
	browser("opencode-detail-of-an-unlisted-id-is-nothing", {
		exchanges: [GO],
		steps: [detail("gone"), detail("deepseek-v4.1-flash")],
	}),
	browser("opencode-detail-without-an-api", {
		exchanges: [reply(data(listed("glm-5.3-flash", { api: null, limit: null })))],
		steps: [detail("glm-5.3-flash")],
	}),
	browser("opencode-detail-without-a-list-is-nothing", {
		exchanges: [failing()],
		steps: [detail("deepseek-v4.1-flash")],
	}),
	browser("opencode-detail-payg-retention", {
		plan: "payg",
		exchanges: [GO],
		steps: [detail("deepseek-v4.1-flash"), detail("gpt-6-luna"), detail("gemini-3.8-flash")],
	}),
	browser("opencode-tester-listed-and-views", {
		exchanges: [GO],
		steps: [
			listedStep("glm-5.3-flash"),
			listedStep("deepseek-v4.1-flash"),
			listedStep("minimax-m3"),
			listedStep("gone"),
			view("opencode:glm-5.3-flash"),
			view("opencode:grok-4.7"),
			view("opencode:gone"),
			view("opencode:minimax-m3", "analysis"),
		],
	}),
	browser("opencode-tester-listed-name-falls-back-to-id", {
		exchanges: [
			reply(
				data(
					{ id: "glm-a" },
					{ id: "glm-b", name: "B", capabilities: { reasoning: { effort: [] } } },
					{ id: "glm-c", capabilities: { reasoning: { effort: ["low"] } } },
				),
			),
		],
		steps: [listedStep("glm-a"), listedStep("glm-b"), listedStep("glm-c")],
	}),
	browser("opencode-tester-view-ignores-the-zdr-flag-of-the-pick", {
		exchanges: [GO],
		steps: [view("openai/gpt-6-luna"), view("deepseek/deepseek-v4.1-flash")],
	}),
	browser("opencode-tester-without-a-list-is-unknown", {
		exchanges: [failing()],
		steps: [listedStep("glm-5.3-flash"), view("opencode:glm-5.3-flash")],
	}),
	browser("opencode-tester-a-row-of-another-kind-is-unknown", {
		exchanges: [GO],
		steps: [view("mistralai/mistral-large-2512"), view("openrouter:lab/model-x")],
	}),
	browser("opencode-tester-caller-aborted-before", {
		exchanges: [GO],
		steps: [{ kind: "listed", id: "glm-5.3-flash", abortedBefore: true }, listedStep("glm-5.3-flash")],
	}),
];
