// Prompt module generator.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { Pattern } from "@nyaaai/contract/pattern";
import { quote, snake } from "@nyaaai/testkit/rustNames";

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

const STEM = Pattern.compile("^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$");

////////////////////////////////
//  Functions & Helpers

const isLetter = (char: string | undefined): boolean =>
	char !== undefined && ((char >= "a" && char <= "z") || (char >= "A" && char <= "Z"));
const isNameChar = (char: string | undefined): boolean =>
	isLetter(char) || (char !== undefined && char >= "0" && char <= "9");

/** Alternating text and `{{name}}` slot names. */
function piecesOf(text: string): string[] {
	const pieces: string[] = [];
	let from = 0;
	let at = 0;
	while (at < text.length) {
		if (text.startsWith("{{", at) && isLetter(text[at + 2])) {
			let end = at + 3;
			while (isNameChar(text[end])) end += 1;
			if (text.startsWith("}}", end)) {
				pieces.push(text.slice(from, at), text.slice(at + 2, end));
				from = end + 2;
				at = from;
				continue;
			}
		}
		at += 1;
	}
	pieces.push(text.slice(from));
	return pieces;
}

/** Drops a dash before a lowercase letter or digit and capitalizes it. */
function camel(stem: string): string {
	let made = "";
	for (let at = 0; at < stem.length; at += 1) {
		const next = stem[at + 1];
		const lower = next !== undefined && ((next >= "a" && next <= "z") || (next >= "0" && next <= "9"));
		if (stem[at] === "-" && lower) {
			made += next.toUpperCase();
			at += 1;
		} else {
			made += stem[at];
		}
	}
	return made;
}

/** One trailing newline stays outside the prompt. */
function promptOf(file: string, text: string, problems: string[]): Prompt | undefined {
	const stem = path.basename(file, ".md");
	if (!STEM.test(stem)) {
		problems.push(`${file}: a prompt file is named in kebab-case`);
		return undefined;
	}
	const pieces = piecesOf(text.endsWith("\n") ? text.slice(0, -1) : text);
	const texts = pieces.filter((_, at) => at % 2 === 0);
	if (texts.some((held) => held.includes("{{") || held.includes("}}"))) {
		problems.push(`${file}: a slot is {{name}}, letters and digits`);
		return undefined;
	}
	const slots = [...new Set(pieces.filter((_, at) => at % 2 === 1))];
	return { name: camel(stem), file, pieces, slots };
}

/** Escapes backslash, backtick and `${`. */
function templateText(text: string): string {
	let made = "";
	for (let at = 0; at < text.length; at += 1) {
		const char = text[at] as string;
		made += char === "\\" || char === "`" || (char === "$" && text[at + 1] === "{") ? `\\${char}` : char;
	}
	return made;
}

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

/** Builds prompt modules from `folder`. Absent: none. */
export function promptFiles(root: string, folder: string | undefined): PromptFiles {
	const made: PromptFiles = { typescript: new Map(), rust: new Map(), problems: [] };
	if (folder === undefined) return made;
	const at = path.join(root, folder);
	if (!existsSync(at)) {
		made.problems.push(`${folder}: no such prompt folder`);
		return made;
	}
	for (const entry of readdirSync(at, { withFileTypes: true }).sort((left, right) =>
		left.name < right.name ? -1 : 1,
	)) {
		const held = `${folder}/${entry.name}`;
		if (!entry.isDirectory() || !existsSync(path.join(root, "ts", entry.name, "package.json"))) {
			made.problems.push(`${held}: prompts sit in a folder named for their package`);
			continue;
		}
		const prompts = readdirSync(path.join(root, held))
			.sort()
			.flatMap((name) => {
				const file = `${held}/${name}`;
				if (!name.endsWith(".md")) {
					made.problems.push(`${file}: a prompt is a .md file`);
					return [];
				}
				return promptOf(file, readFileSync(path.join(root, file), "utf8"), made.problems) ?? [];
			});
		made.typescript.set(`ts/${entry.name}/src/prompts.ts`, typescriptOf(held, prompts));
		made.rust.set(`rust/${entry.name}/src/prompts.rs`, rustOf(held, prompts));
	}
	return made;
}
