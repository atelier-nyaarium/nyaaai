// Prompt module generator.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { quote, snake } from "./contract/rust";

////////////////////////////////
//  Interfaces & Types

interface Prompt {
	/** camelCase filename stem. */
	name: string;
	file: string;
	/** Alternating text and slots. */
	pieces: string[];
	/** Slots in first-use order. */
	slots: string[];
}

export interface PromptFiles {
	/** Unformatted source. */
	typescript: Map<string, string>;
	rust: Map<string, string>;
	problems: string[];
}

////////////////////////////////
//  Constants

const FOLDER = "prompts";
/** Prompt slot syntax. */
const SLOT = /\{\{([A-Za-z][A-Za-z0-9]*)\}\}/;
const STEM = /^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$/;

////////////////////////////////
//  Functions & Helpers

const camel = (stem: string): string => stem.replace(/-([a-z0-9])/g, (_, letter: string) => letter.toUpperCase());

/** One trailing newline stays outside the prompt. */
function promptOf(file: string, text: string, problems: string[]): Prompt | undefined {
	const stem = path.basename(file, ".md");
	if (!STEM.test(stem)) {
		problems.push(`${file}: a prompt file is named in kebab-case`);
		return undefined;
	}
	const pieces = (text.endsWith("\n") ? text.slice(0, -1) : text).split(SLOT);
	const texts = pieces.filter((_, at) => at % 2 === 0);
	if (texts.some((held) => held.includes("{{") || held.includes("}}"))) {
		problems.push(`${file}: a slot is {{name}}, letters and digits`);
		return undefined;
	}
	const slots = [...new Set(pieces.filter((_, at) => at % 2 === 1))];
	return { name: camel(stem), file, pieces, slots };
}

const templateText = (text: string): string => text.replace(/[\\`]|\$\{/g, (held) => `\\${held}`);

function typescriptOf(folder: string, prompts: Prompt[]): string {
	const items = prompts.map((prompt) => {
		const body = prompt.pieces
			.map((piece, at) => (at % 2 === 0 ? templateText(piece) : `\${slots.${piece}}`))
			.join("");
		const made =
			prompt.slots.length === 0
				? `export const ${prompt.name} = ${JSON.stringify(prompt.pieces[0])};`
				: `export const ${prompt.name} = (slots: { ${prompt.slots.map((slot) => `${slot}: string`).join("; ")} }): string =>\n\t\`${body}\`;`;
		return `/** ${prompt.file} */\n${made}`;
	});
	return [`// Generated from ${folder} by scripts/contract.ts. Do not edit.`, ...items, ""].join("\n\n");
}

function rustOf(folder: string, prompts: Prompt[]): string {
	const items = prompts.map((prompt) => {
		const name = snake(prompt.name);
		if (prompt.slots.length === 0) {
			return `/// ${prompt.file}\npub const ${name.toUpperCase()}: &str = ${quote(prompt.pieces[0] as string)};`;
		}
		const pieces = prompt.pieces.map((piece, at) => (at % 2 === 0 ? quote(piece) : snake(piece)));
		return [
			`/// ${prompt.file}`,
			`pub fn ${name}(${prompt.slots.map((slot) => `${snake(slot)}: &str`).join(", ")}) -> String {`,
			`[${pieces.join(", ")}].concat()`,
			"}",
		].join("\n");
	});
	return [`//! Generated from ${folder} by scripts/contract.ts. Do not edit.`, ...items, ""].join("\n\n");
}

/** Builds prompt modules. */
export function promptFiles(root: string): PromptFiles {
	const made: PromptFiles = { typescript: new Map(), rust: new Map(), problems: [] };
	const at = path.join(root, FOLDER);
	if (!existsSync(at)) return made;
	for (const entry of readdirSync(at, { withFileTypes: true }).sort((left, right) =>
		left.name < right.name ? -1 : 1,
	)) {
		const folder = `${FOLDER}/${entry.name}`;
		if (!entry.isDirectory() || !existsSync(path.join(root, "ts", entry.name, "package.json"))) {
			made.problems.push(`${folder}: prompts sit in a folder named for their package`);
			continue;
		}
		const prompts = readdirSync(path.join(root, folder))
			.sort()
			.flatMap((name) => {
				const file = `${folder}/${name}`;
				if (!name.endsWith(".md")) {
					made.problems.push(`${file}: a prompt is a .md file`);
					return [];
				}
				return promptOf(file, readFileSync(path.join(root, file), "utf8"), made.problems) ?? [];
			});
		made.typescript.set(`ts/${entry.name}/src/prompts.ts`, typescriptOf(folder, prompts));
		made.rust.set(`rust/${entry.name}/src/prompts.rs`, rustOf(folder, prompts));
	}
	return made;
}
