import { describe, expect, it } from "bun:test";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { Pattern } from "@nyaaai/contract/pattern";
import { ROOT } from "../child";

interface Token {
	kind: "word" | "string" | "template" | "regex" | "punct";
	value: string;
	/** Marks control-head closers. */
	head?: true;
}

const HEAD_WORDS = new Set(["if", "while", "for", "with"]);

const PREFIX_WORDS = new Set([
	"await",
	"case",
	"default",
	"delete",
	"do",
	"else",
	"in",
	"instanceof",
	"new",
	"of",
	"return",
	"throw",
	"typeof",
	"void",
	"yield",
]);

function quoted(source: string, start: number): { token: Token; end: number } {
	const quote = source[start] as string;
	let at = start + 1;
	let value = "";
	while (at < source.length && source[at] !== quote) {
		if (source[at] !== "\\") {
			value += source[at];
			at += 1;
			continue;
		}
		at += 1;
		const code = source[at];
		if (code === undefined) break;
		at += 1;
		const simple: Record<string, string> = { n: "\n", r: "\r", t: "\t", b: "\b", f: "\f", v: "\v", "0": "\0" };
		if (simple[code] !== undefined) value += simple[code];
		else if (code === "\n") continue;
		else if (code === "\r") {
			if (source[at] === "\n") at += 1;
		} else if (code === "x") {
			const digits = source.slice(at, at + 2);
			if (/^[0-9a-fA-F]{2}$/.test(digits)) {
				value += String.fromCharCode(Number.parseInt(digits, 16));
				at += 2;
			} else value += "x";
		} else if (code === "u") {
			if (source[at] === "{") {
				const end = source.indexOf("}", at + 1);
				const digits = end < 0 ? "" : source.slice(at + 1, end);
				if (/^[0-9a-fA-F]{1,6}$/.test(digits) && Number.parseInt(digits, 16) <= 0x10ffff) {
					value += String.fromCodePoint(Number.parseInt(digits, 16));
					at = end + 1;
				} else value += "u";
			} else {
				const digits = source.slice(at, at + 4);
				if (/^[0-9a-fA-F]{4}$/.test(digits)) {
					value += String.fromCharCode(Number.parseInt(digits, 16));
					at += 4;
				} else value += "u";
			}
		} else value += code;
	}
	return { token: { kind: "string", value }, end: Math.min(source.length, at + 1) };
}

function regexEnd(source: string, start: number): number {
	let at = start + 1;
	let inClass = false;
	let escaped = false;
	while (at < source.length) {
		const code = source[at] as string;
		if (escaped) escaped = false;
		else if (code === "\\") escaped = true;
		else if (code === "[") inClass = true;
		else if (code === "]") inClass = false;
		else if (code === "/" && !inClass) {
			at += 1;
			while (/[A-Za-z]/.test(source[at] ?? "")) at += 1;
			return at;
		}
		at += 1;
	}
	return start + 1;
}

function endsExpression(token: Token | undefined): boolean {
	if (!token || token.head) return false;
	if (token.kind === "string" || token.kind === "template" || token.kind === "regex") return true;
	if (token.kind === "word") return !PREFIX_WORDS.has(token.value);
	return [")", "]", "}", "++", "--"].includes(token.value);
}

function tokensOf(source: string): Token[] {
	const tokens: Token[] = [];
	scan(source, 0, tokens, false);
	return tokens;
}

/** Stops at template-hole end. */
function scan(source: string, from: number, tokens: Token[], inTemplate: boolean): number {
	let at = from;
	let braces = 0;
	const heads: boolean[] = [];
	while (at < source.length) {
		const code = source[at] as string;
		if (/\s/.test(code)) {
			at += 1;
			continue;
		}
		if (code === "/" && source[at + 1] === "/") {
			at += 2;
			while (at < source.length && source[at] !== "\n") at += 1;
			continue;
		}
		if (code === "/" && source[at + 1] === "*") {
			const end = source.indexOf("*/", at + 2);
			at = end < 0 ? source.length : end + 2;
			continue;
		}
		if (code === "'" || code === '"') {
			const held = quoted(source, at);
			tokens.push(held.token);
			at = held.end;
			continue;
		}
		if (code === "`") {
			at += 1;
			while (at < source.length) {
				if (source[at] === "\\") at += 2;
				else if (source[at] === "`") {
					at += 1;
					break;
				} else if (source[at] === "$" && source[at + 1] === "{") at = scan(source, at + 2, tokens, true);
				else at += 1;
			}
			tokens.push({ kind: "template", value: "" });
			continue;
		}
		if (code === "(") heads.push(HEAD_WORDS.has(tokens.at(-1)?.value ?? ""));
		if (code === ")" && heads.pop()) {
			tokens.push({ kind: "punct", value: code, head: true });
			at += 1;
			continue;
		}
		if (code === "{") braces += 1;
		if (code === "}") {
			if (inTemplate && braces === 0) return at + 1;
			braces -= 1;
		}
		if (code === "/") {
			if (!endsExpression(tokens.at(-1))) {
				const end = regexEnd(source, at);
				if (end > at + 1) {
					tokens.push({ kind: "regex", value: source.slice(at, end) });
					at = end;
					continue;
				}
			}
			const value = source[at + 1] === "=" ? "/=" : "/";
			tokens.push({ kind: "punct", value });
			at += value.length;
			continue;
		}
		if (/[A-Za-z_$]/.test(code)) {
			const start = at++;
			while (/[A-Za-z0-9_$]/.test(source[at] ?? "")) at += 1;
			tokens.push({ kind: "word", value: source.slice(start, at) });
			continue;
		}
		if (/[0-9]/.test(code)) {
			const start = at++;
			while (/[A-Za-z0-9_.]/.test(source[at] ?? "")) at += 1;
			tokens.push({ kind: "word", value: source.slice(start, at) });
			continue;
		}
		const paired = source.slice(at, at + 2);
		const value = [
			"++",
			"--",
			"?.",
			"=>",
			"==",
			"!=",
			"<=",
			">=",
			"&&",
			"||",
			"??",
			"+=",
			"-=",
			"*=",
			"/=",
		].includes(paired)
			? paired
			: code;
		tokens.push({ kind: "punct", value });
		at += value.length;
	}
	return at;
}

function hasRegexLiteral(source: string): boolean {
	return tokensOf(source).some((token) => token.kind === "regex");
}

function patternCalls(source: string): Array<[string, string]> {
	const tokens = tokensOf(source);
	const found: Array<[string, string]> = [];
	for (let at = 0; at + 4 < tokens.length; at += 1) {
		if (
			tokens[at]?.kind !== "word" ||
			tokens[at]?.value !== "Pattern" ||
			tokens[at + 1]?.value !== "." ||
			tokens[at + 2]?.value !== "compile" ||
			tokens[at + 3]?.value !== "(" ||
			tokens[at + 4]?.kind !== "string"
		)
			continue;
		const sourceText = tokens[at + 4]?.value as string;
		if (tokens[at + 5]?.value === ")") found.push([sourceText, ""]);
		else if (tokens[at + 5]?.value === "," && tokens[at + 6]?.kind === "string" && tokens[at + 7]?.value === ")")
			found.push([sourceText, tokens[at + 6]?.value as string]);
	}
	return found;
}

function hasRegExpCall(source: string): boolean {
	const tokens = tokensOf(source);
	return tokens.some(
		(token, at) => token.kind === "word" && token.value === "RegExp" && tokens[at + 1]?.value === "(",
	);
}

function filesUnder(folder: string): string[] {
	return readdirSync(folder, { withFileTypes: true }).flatMap((entry) => {
		const held = path.join(folder, entry.name);
		if (entry.name === "node_modules" || entry.name === "target" || entry.name === ".tsbuild") return [];
		return entry.isDirectory() ? filesUnder(held) : [held];
	});
}

function engineTypeScriptFiles(): string[] {
	const folders = ["process", "agentic", "operator", "cli"].map((name) => path.join(ROOT, "ts", name, "src"));
	const visit = (folder: string): string[] =>
		readdirSync(folder, { withFileTypes: true }).flatMap((entry) => {
			if (entry.name === "__tests__" || entry.name === "node_modules" || entry.name === "target") return [];
			const held = path.join(folder, entry.name);
			if (entry.isDirectory()) return visit(held);
			return entry.isFile() && entry.name.endsWith(".ts") ? [held] : [];
		});
	return folders.flatMap(visit);
}

function rustEngineFiles(): string[] {
	return ["process", "agentic", "operator", "cli"]
		.flatMap((name) => filesUnder(path.join(ROOT, "rust", name, "src")))
		.filter((file) => file.endsWith(".rs"));
}

function operatorNames(): string[] {
	const packageName = (
		JSON.parse(readFileSync(path.join(ROOT, "ts", "operator", "package.json"), "utf8")) as { name: string }
	).name;
	const crate = /^name = "([^"]+)"$/m.exec(
		readFileSync(path.join(ROOT, "rust", "operator", "Cargo.toml"), "utf8"),
	)?.[1];
	if (!crate) throw new Error("the operator crate names itself nowhere");
	return [packageName, crate, crate.replaceAll("-", "_"), "../operator/"];
}

function namesOperator(files: Array<[string, string]>): string[] {
	const names = operatorNames();
	return files.filter(([, text]) => names.some((name) => text.includes(name))).map(([file]) => file);
}

describe("residue", () => {
	it("engine TypeScript uses frozen patterns", () => {
		const problems: string[] = [];
		for (const file of engineTypeScriptFiles()) {
			const text = readFileSync(file, "utf8");
			const tokens = tokensOf(text);
			if (tokens.some((token) => token.kind === "regex")) problems.push(`${file} has a regex literal`);
			if (hasRegExpCall(text)) problems.push(`${file} calls RegExp`);
			for (const [source, flags] of patternCalls(text)) {
				try {
					Pattern.compile(source, flags);
				} catch {
					problems.push(`${file} compiles a pattern outside the dialect`);
				}
			}
		}
		expect(problems).toEqual([]);
	});

	it("the regex scanner skips non-code slashes", () => {
		// `@{` marks template holes.
		const hole = (text: string) => text.replaceAll("@{", "$".concat("{"));
		const probes: Array<[string, boolean]> = [
			["const hit = /a+/;", true],
			["const hit = /[a/]+/;", true],
			["return /a+/;", true],
			["const value = left / right;", false],
			["const value = left /= right;", false],
			["const value = '/a+/';", false],
			['const value = "/a+/";', false],
			["const value = ` /a+/ `;", false],
			["// /a+/\nconst value = left / right;", false],
			["/* /a+/ */ const value = left / right;", false],
			["export default /a+/;", true],
			["if (x) /a+/.test(y);", true],
			["while (next(x)) /a+/.exec(y);", true],
			["const value = (left) / right / 2;", false],
			[hole('const value = `a @{text.replace(/a+/, "")} b`;'), true],
			[hole("const value = `@{left / right} and @{up / down}`;"), false],
			[hole("const value = `a @{`b @{c} /x/`} /y/`;"), false],
			[hole("const value = `@{{ a: 1 }.a / 2}` + left / right;"), false],
		];
		for (const [source, expected] of probes) expect(hasRegexLiteral(source)).toBe(expected);
	});

	it("engine Rust never names regex", () => {
		const files = rustEngineFiles();
		const offenders = files.filter((file) => {
			const text = readFileSync(file, "utf8");
			return text.includes("regex::") || /\buse\s+regex\b/.test(text);
		});
		expect(offenders).toEqual([]);
	});

	it("agentic never names operator", () => {
		const folders = [path.join(ROOT, "ts", "agentic"), path.join(ROOT, "rust", "agentic")];
		const files = folders.flatMap(filesUnder).map((file): [string, string] => [file, readFileSync(file, "utf8")]);
		expect(files.length).toBeGreaterThan(0);
		expect(namesOperator(files)).toEqual([]);

		const [packageName, crate, crateInCode] = operatorNames();
		const probes: Array<[string, string]> = [
			["a.ts", `import { x } from "${packageName}/claude/sessionCut";`],
			["b.ts", 'import { x } from "../../operator/src/index";'],
			["Cargo.toml", `${crate}.workspace = true`],
			["c.rs", `use ${crateInCode}::claude;`],
			["d.ts", 'import { x } from "@nyaaai/contract/json";'],
		];
		expect(namesOperator(probes)).toEqual(["a.ts", "b.ts", "Cargo.toml", "c.rs"]);
	});

	it("both workspaces hold the same packages", () => {
		const packages = (
			JSON.parse(readFileSync(path.join(ROOT, "ts", "package.json"), "utf8")) as { workspaces: string[] }
		).workspaces;
		const members = /^members = \[([^\]]*)\]/m.exec(
			readFileSync(path.join(ROOT, "rust", "Cargo.toml"), "utf8"),
		)?.[1];
		const crates = (members ?? "").split(",").map((name) => name.trim().replaceAll('"', ""));
		expect(packages.filter((name) => name !== "ui").sort()).toEqual(crates.sort());
	});
});
