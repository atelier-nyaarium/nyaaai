// Prompt generator tests.

import { afterAll, beforeAll, describe, expect, it } from "bun:test";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { clearedNote, compactedSummary } from "@nyaaai/operator/prompts";
import { ROOT } from "../child";
import { promptFiles } from "../prompts";

////////////////////////////////
//  Functions & Helpers

let temp: string;

/** Temporary prompt project. */
function world(name: string, prompts: Record<string, string>): string {
	const root = path.join(temp, name);
	mkdirSync(path.join(root, "ts", "agentic"), { recursive: true });
	writeFileSync(path.join(root, "ts", "agentic", "package.json"), "{}");
	mkdirSync(path.join(root, "prompts", "agentic"), { recursive: true });
	for (const [file, text] of Object.entries(prompts)) writeFileSync(path.join(root, "prompts", file), text);
	return root;
}

const filled = (file: string, slots: Record<string, string>): string =>
	readFileSync(path.join(ROOT, file), "utf8")
		.replace(/\n$/, "")
		.replace(/\{\{(\w+)\}\}/g, (_, slot: string) => slots[slot] as string);

////////////////////////////////
//  Tests

beforeAll(() => {
	temp = mkdtempSync(path.join(tmpdir(), "nyaaai-prompts-"));
});

afterAll(() => {
	rmSync(temp, { recursive: true, force: true });
});

describe("the prompt generator", () => {
	it("renders the repository's prompts as their files read", () => {
		// biome-ignore lint/suspicious/noTemplateCurlyInString: a slot value may hold one.
		const slots = { summary: "what $& `stood` ${x}", sourcePath: "/a \\ b.jsonl", rows: "7" };
		expect(compactedSummary(slots)).toBe(filled("prompts/operator/compacted-summary.md", slots));
		expect(clearedNote(slots)).toBe(filled("prompts/operator/cleared-note.md", slots));
	});

	it("carries every character of a prompt into the generated module", async () => {
		// biome-ignore lint/suspicious/noTemplateCurlyInString: a prompt may hold one.
		const text = 'Tick ` and ${not a slot} and \\ and "quotes"\n{{first}} then {{second}} and {{first}} again\ttab';
		const root = world("tricky", { "agentic/odd-one.md": `${text}\n`, "agentic/plain.md": "no slots `here`\n" });
		const { typescript, rust, problems } = promptFiles(root);
		expect(problems).toEqual([]);
		const file = path.join(root, "generated.ts");
		writeFileSync(file, typescript.get("ts/agentic/src/prompts.ts") as string);
		const made = (await import(file)) as { oddOne: (slots: object) => string; plain: string };
		expect(made.oddOne({ first: "{{second}}", second: "2" })).toBe(
			text.replaceAll("{{first}}", "\u{0}").replaceAll("{{second}}", "2").replaceAll("\u{0}", "{{second}}"),
		);
		expect(made.plain).toBe("no slots `here`");
		expect(rust.get("rust/agentic/src/prompts.rs")).toContain(
			"pub fn odd_one(first: &str, second: &str) -> String",
		);
	});

	it("names a prompt it cannot generate", () => {
		const root = world("bad", {
			"agentic/Bad_Name.md": "x\n",
			"agentic/half-slot.md": "a {{open b\n",
			"agentic/spaced.md": "a {{ name }} b\n",
			"agentic/notes.txt": "x\n",
			"agentic/fine.md": "x\n",
		});
		mkdirSync(path.join(root, "prompts", "nowhere"));
		const { problems } = promptFiles(root);
		expect(problems.map((problem) => problem.slice(0, problem.indexOf(":"))).sort()).toEqual([
			"prompts/agentic/Bad_Name.md",
			"prompts/agentic/half-slot.md",
			"prompts/agentic/notes.txt",
			"prompts/agentic/spaced.md",
			"prompts/nowhere",
		]);
	});
});
