import { describe, expect, it } from "bun:test";
import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { Pattern } from "@nyaaai/contract/pattern";
import { ROOT } from "../child";
import {
	descendants,
	isBindingElement,
	isCallExpression,
	isElementAccessExpression,
	isExportDeclaration,
	isIdentifier,
	isImportDeclaration,
	isImportEqualsDeclaration,
	isImportTypeNode,
	isLiteralTypeNode,
	isNoSubstitutionTemplateLiteral,
	isObjectBindingPattern,
	isPropertyAccessExpression,
	isRegularExpressionLiteral,
	isStringLiteral,
	isVariableDeclaration,
	type Node,
	parseSources,
	type SourceFile,
	SyntaxKind,
} from "../syntax";

/** Text of a string literal. */
function literalText(node: Node | undefined): string | undefined {
	return node && (isStringLiteral(node) || isNoSubstitutionTemplateLiteral(node)) ? node.text : undefined;
}

/** `object.key` or `object["key"]`. */
function memberOf(node: Node): { object: Node; key: string; at: number } | undefined {
	if (isPropertyAccessExpression(node)) return { object: node.expression, key: node.name.text, at: node.name.end };
	if (!isElementAccessExpression(node)) return undefined;
	const key = literalText(node.argumentExpression);
	return key === undefined ? undefined : { object: node.expression, key, at: node.argumentExpression.end };
}

/** An identifier that names a value, not a member, a key or a type. */
function isReference(node: Node): boolean {
	const parent = node.parent as Node & { name?: Node; propertyName?: Node };
	if (parent.name === node || parent.propertyName === node) return false;
	return parent.kind !== SyntaxKind.TypeReference && parent.kind !== SyntaxKind.QualifiedName;
}

function isMemberObject(node: Node): boolean {
	const member = memberOf(node.parent);
	return member !== undefined && member.object === node;
}

/** Regex literals and every use of `RegExp`. */
function regexUses(file: SourceFile): string[] {
	const found: string[] = [];
	for (const node of descendants(file)) {
		if (isRegularExpressionLiteral(node)) found.push("a regex literal");
		else if (isIdentifier(node) && node.text === "RegExp" && isReference(node)) found.push("RegExp");
	}
	return found;
}

/** Literal arguments of `Pattern.compile`. */
function patternCalls(file: SourceFile): Array<[string, string]> {
	const found: Array<[string, string]> = [];
	for (const node of descendants(file)) {
		if (!isCallExpression(node)) continue;
		const member = memberOf(node.expression);
		if (member?.key !== "compile" || !isIdentifier(member.object) || member.object.text !== "Pattern") continue;
		const [source, flags, ...rest] = node.arguments.map((argument) => literalText(argument));
		if (source !== undefined && rest.length === 0 && (flags !== undefined || node.arguments.length === 1)) {
			found.push([source, flags ?? ""]);
		}
	}
	return found;
}

/** Members with cross-language differences. */
const PLATFORM_MEMBERS = new Map([
	[
		"Math",
		new Map([
			["round", "roundHalfUp"],
			["random", "a random source"],
		]),
	],
	["Buffer", new Map([["byteLength", "utf8Length"]])],
	[
		"Number",
		new Map([
			["parseInt", "integerOfText"],
			["parseFloat", "numberOfText"],
		]),
	],
]);

const PLATFORM_FUNCTIONS = new Map([
	["parseInt", "integerOfText"],
	["parseFloat", "numberOfText"],
]);

const PLATFORM_METHODS = new Map([
	["trim", "trimText"],
	["trimStart", "trimText"],
	["trimEnd", "trimText"],
	["toLowerCase", "asciiLower"],
	["toUpperCase", "asciiUpper"],
	["toLocaleLowerCase", "asciiLower"],
	["toLocaleUpperCase", "asciiUpper"],
	["localeCompare", "compareText"],
	["normalize", "nfc"],
	["toFixed", "fixedText"],
	["toPrecision", "numberText"],
	["toExponential", "numberText"],
]);

/** Primitive replacement mapping, in source order. */
function platformCalls(file: SourceFile): string[] {
	const found: Array<[at: number, text: string]> = [];
	for (const node of descendants(file)) {
		if (isIdentifier(node) && isReference(node)) {
			const name = node.text;
			if (name === "JSON") found.push([node.end, "JSON: stringifyJson or parseJson"]);
			if (name === "globalThis") found.push([node.end, "globalThis: the primitives"]);
			if (name === "Date") found.push([node.end, "Date: a clock"]);
			if ((name === "String" || name === "Number") && !isMemberObject(node)) {
				found.push([node.end, `${name}: numberText or numberOfText`]);
			}
			const use = PLATFORM_FUNCTIONS.get(name);
			if (use) found.push([node.end, `${name}: ${use}`]);
			if ((name === "Math" || name === "Buffer") && !isMemberObject(node) && !isDestructured(node)) {
				found.push([node.end, `${name}: use its members directly`]);
			}
		}
		const member = memberOf(node);
		const method = member && PLATFORM_METHODS.get(member.key);
		if (member && method) found.push([member.at, `${member.key}: ${method}`]);
		if (member && isIdentifier(member.object)) {
			const use = PLATFORM_MEMBERS.get(member.object.text)?.get(member.key);
			if (use) found.push([member.at, `${member.object.text}.${member.key}: ${use}`]);
		}
		if (isBindingElement(node) && isObjectBindingPattern(node.parent)) {
			const source = node.parent.parent;
			const object = isVariableDeclaration(source) ? source.initializer : undefined;
			const keyNode = node.propertyName ?? node.name;
			const key = keyNode && (isIdentifier(keyNode) ? keyNode.text : literalText(keyNode));
			const use = object && isIdentifier(object) ? PLATFORM_MEMBERS.get(object.text)?.get(key ?? "") : undefined;
			if (use && object && isIdentifier(object)) found.push([node.end, `${object.text}.${key}: ${use}`]);
		}
		if (!isCallExpression(node)) continue;
		const callee = memberOf(node.expression);
		if (!callee) continue;
		if (callee.key === "sort" && node.arguments.length === 0) found.push([callee.at, "sort: sortedTexts"]);
	}
	return found.sort((left, right) => left[0] - right[0]).map(([, text]) => text);
}

/** `const { round } = Math`: the destructuring names the member. */
function isDestructured(node: Node): boolean {
	return isVariableDeclaration(node.parent) && isObjectBindingPattern(node.parent.name);
}

function testFiles(): string[] {
	const packages = readdirSync(path.join(ROOT, "ts"), { withFileTypes: true })
		.filter((entry) => entry.isDirectory() && entry.name !== "node_modules")
		.map((entry) => path.join(ROOT, "ts", entry.name, "src"));
	return [...packages, path.join(ROOT, "scripts")]
		.flatMap(filesUnder)
		.filter((file) => file.includes(`${path.sep}__tests__${path.sep}`) && file.endsWith(".ts"));
}

/** Consumer packages and scripts. */
const CONSUMER_PACKAGES = ["contract", "testkit", "adapter"];
const CONSUMER_SCRIPTS = [
	"adapterHost.ts",
	"bannedSymbols.ts",
	"caseAdapter.ts",
	"cases.ts",
	"child.ts",
	"consumer.ts",
	"crates.ts",
	"contract.ts",
	"contract/ir.ts",
	"contract/rust.ts",
	"manifest.ts",
	"project.ts",
	"prompts.ts",
	"runScript.ts",
	"samples.ts",
	"serve.ts",
	"snapshot.ts",
];

/** Module specifiers a file names, however it names them. */
function specifiersOf(file: SourceFile): string[] {
	const found: Array<string | undefined> = [];
	for (const node of descendants(file)) {
		if (isImportDeclaration(node) || isExportDeclaration(node)) found.push(literalText(node.moduleSpecifier));
		else if (isImportEqualsDeclaration(node) && node.moduleReference.kind === SyntaxKind.ExternalModuleReference) {
			found.push(literalText((node.moduleReference as unknown as { expression: Node }).expression));
		} else if (isImportTypeNode(node) && isLiteralTypeNode(node.argument))
			found.push(literalText(node.argument.literal));
		else if (isCallExpression(node)) {
			const callee = node.expression;
			const loads =
				callee.kind === SyntaxKind.ImportKeyword || (isIdentifier(callee) && callee.text === "require");
			if (loads) found.push(literalText(node.arguments[0]));
		}
	}
	return found.filter((specifier) => specifier !== undefined);
}

/** `import(x)` whose argument is not a string literal. */
function dynamicImports(file: SourceFile): number {
	return descendants(file).filter(
		(node) =>
			isCallExpression(node) &&
			node.expression.kind === SyntaxKind.ImportKeyword &&
			literalText(node.arguments[0]) === undefined,
	).length;
}

function foreignImports(file: string, source: SourceFile): string[] {
	return specifiersOf(source).filter((specifier) => {
		if (specifier.startsWith("@nyaaai/")) return !CONSUMER_PACKAGES.includes(specifier.split("/")[1] ?? "");
		if (!specifier.startsWith(".")) return false;
		const target = path.posix.join(path.posix.dirname(file), `${specifier}.ts`);
		return !CONSUMER_SCRIPTS.includes(target);
	});
}

function filesUnder(folder: string): string[] {
	return readdirSync(folder, { withFileTypes: true }).flatMap((entry) => {
		const held = path.join(folder, entry.name);
		if (entry.name === "node_modules" || entry.name === "target" || entry.name === ".tsbuild") return [];
		return entry.isDirectory() ? filesUnder(held) : [held];
	});
}

function engineTypeScriptFiles(): string[] {
	return ["process", "agentic", "operator", "adapter", "cli"].flatMap((name) => sourcesOf(name));
}

/** Non-test TypeScript of one package. */
function sourcesOf(name: string): string[] {
	return filesUnder(path.join(ROOT, "ts", name, "src")).filter(
		(file) => /\.tsx?$/.test(file) && !file.includes(`${path.sep}__tests__${path.sep}`),
	);
}

function allPackageNames(): string[] {
	return readdirSync(path.join(ROOT, "ts"), { withFileTypes: true })
		.filter((entry) => entry.isDirectory() && entry.name !== "node_modules")
		.map((entry) => entry.name);
}

async function parseFiles(files: string[]): Promise<Map<string, SourceFile>> {
	return parseSources(Object.fromEntries(files.map((file) => [file, readFileSync(file, "utf8")])));
}

/** Parses probe sources, in order. */
async function parseProbes(sources: string[]): Promise<SourceFile[]> {
	const parsed = await parseSources(Object.fromEntries(sources.map((text, at) => [`probe${at}.ts`, text])));
	return [...parsed.values()];
}

interface CargoManifest {
	package?: { name?: string };
	dependencies?: Record<string, unknown>;
	"dev-dependencies"?: Record<string, unknown>;
	"build-dependencies"?: Record<string, unknown>;
	workspace?: { members?: string[] };
}

function cargoOf(text: string): CargoManifest {
	return Bun.TOML.parse(text) as CargoManifest;
}

function crateManifest(name: string): CargoManifest {
	return cargoOf(readFileSync(path.join(ROOT, "rust", name, "Cargo.toml"), "utf8"));
}

function dependencyNames(manifest: CargoManifest): string[] {
	return [manifest.dependencies, manifest["dev-dependencies"], manifest["build-dependencies"]].flatMap((table) =>
		Object.keys(table ?? {}),
	);
}

function operatorNames(): string[] {
	const packageName = (
		JSON.parse(readFileSync(path.join(ROOT, "ts", "operator", "package.json"), "utf8")) as { name: string }
	).name;
	const crate = crateManifest("operator").package?.name;
	if (!crate) throw new Error("the operator crate names itself nowhere");
	return [packageName, crate, crate.replaceAll("-", "_"), "../operator/"];
}

function namesOperator(files: Array<[string, string]>): string[] {
	const names = operatorNames();
	return files.filter(([, text]) => names.some((name) => text.includes(name))).map(([file]) => file);
}

describe("residue", () => {
	it("engine TypeScript uses frozen patterns", async () => {
		const probes = await parseProbes([
			"const hit = /a+/g;",
			"const hit = new RegExp('a+');",
			"const hit = RegExp(pattern);",
			"const value = left / right / 2; const note = '/a+/'; // /a+/",
			// biome-ignore lint/suspicious/noTemplateCurlyInString: source under test
			"const a = `x ${text.replace(/a+/, '')} y`;",
			"Pattern.compile('a+'); Pattern.compile('a', 'i'); Pattern.compile(source); Pattern.compile('a', flags);",
		]);
		expect(probes.map(regexUses)).toEqual([
			["a regex literal"],
			["RegExp"],
			["RegExp"],
			[],
			["a regex literal"],
			[],
		]);
		expect(patternCalls(probes[5] as SourceFile)).toEqual([
			["a+", ""],
			["a", "i"],
		]);

		const problems: string[] = [];
		for (const [file, source] of await parseFiles(engineTypeScriptFiles())) {
			if (regexUses(source).length > 0) problems.push(`${file} has a regex literal or uses RegExp`);
			for (const [text, flags] of patternCalls(source)) {
				try {
					Pattern.compile(text, flags);
				} catch {
					problems.push(`${file} compiles a pattern outside the dialect`);
				}
			}
		}
		expect(problems).toEqual([]);
	});

	it("dynamic imports name a literal", async () => {
		const probes = await parseProbes(["await import(target);", "await import('./a'); await import(`./b`);"]);
		expect(probes.map(dynamicImports)).toEqual([1, 0]);

		const files = [
			...engineTypeScriptFiles(),
			...CONSUMER_SCRIPTS.map((script) => path.join(ROOT, "scripts", script)),
		];
		const parsed = await parseFiles(files);
		// The loader imports the project file by its path, once.
		const loader = path.join(ROOT, "scripts", "project.ts");
		const counts = new Map([...parsed].map(([file, source]) => [file, dynamicImports(source)]));
		expect(counts.get(loader)).toBe(1);
		expect([...counts].filter(([file, count]) => file !== loader && count > 0).map(([file]) => file)).toEqual([]);
	});

	it("TypeScript sources hold no regex", async () => {
		const files = allPackageNames().flatMap(sourcesOf);
		expect(files.length).toBeGreaterThan(0);
		const problems = [...(await parseFiles(files))].flatMap(([file, source]) =>
			regexUses(source).map((use) => `${file} holds ${use}`),
		);
		expect(problems).toEqual([]);
	});

	it("engine TypeScript reaches the platform through primitives", async () => {
		const probes: Array<[string, string[]]> = [
			["const text = JSON.stringify(value);", ["JSON: stringifyJson or parseJson"]],
			[
				"const { parse } = JSON; const spell = JSON['stringify'];",
				["JSON: stringifyJson or parseJson", "JSON: stringifyJson or parseJson"],
			],
			["return Math.round(bytes / 4) + Math.min(a, b);", ["Math.round: roundHalfUp"]],
			[
				"return Math['round'](bytes) + Math['random']();",
				["Math.round: roundHalfUp", "Math.random: a random source"],
			],
			[
				"const { round, min } = Math; const alias = Math;",
				["Math.round: roundHalfUp", "Math: use its members directly"],
			],
			[
				"return Buffer.byteLength(text) + parseInt(text, 10);",
				["Buffer.byteLength: utf8Length", "parseInt: integerOfText"],
			],
			["const note = String(count) + held.toString();", ["String: numberText or numberOfText"]],
			["if (Number.isFinite(value)) return Number(text);", ["Number: numberText or numberOfText"]],
			["return counts.map(String).join(String.fromCodePoint(44));", ["String: numberText or numberOfText"]],
			["const at = new Date().getTime() + Date.now();", ["Date: a clock", "Date: a clock"]],
			["names.sort(); lines.sort((a, b) => a - b);", ["sort: sortedTexts"]],
			[
				"return text?.trim().toLowerCase() + text['toUpperCase']();",
				["trim: trimText", "toLowerCase: asciiLower", "toUpperCase: asciiUpper"],
			],
			[
				"const held = shape.String(1); // String(x)\nconst trim = 'x.trim()'; let when: Date; const { Date: when2 } = shape;",
				[],
			],
		];
		const parsed = await parseProbes(probes.map(([source]) => source));
		for (const [at, [, expected]] of probes.entries())
			expect(platformCalls(parsed[at] as SourceFile)).toEqual(expected);

		const problems = [...(await parseFiles(engineTypeScriptFiles()))].flatMap(([file, source]) =>
			platformCalls(source).map((call) => `${file} calls ${call}`),
		);
		expect(problems).toEqual([]);
	});

	it("the matcher runs on no platform regex", async () => {
		const matcher = path.join(ROOT, "ts", "contract", "src", "pattern.ts");
		const parsed = await parseFiles([matcher]);
		expect(regexUses(parsed.get(matcher) as SourceFile)).toEqual([]);
		expect(dependencyNames(crateManifest("contract"))).not.toContain("regex");
	});

	it("tests read a rejection through the testkit", async () => {
		const probes = await parseProbes([
			"await expect(run()).rejects.toThrow();",
			"await expect(run())['rejects'].toThrow(); const rejects = 1;",
			"const note = 'x.rejects'; const rejects = 1; // .rejects",
		]);
		const rejections = (file: SourceFile) =>
			descendants(file).filter((node) => memberOf(node)?.key === "rejects").length;
		expect(probes.map(rejections)).toEqual([1, 1, 0]);

		const files = testFiles();
		const parsed = await parseFiles(files);
		expect([...parsed].filter(([, source]) => rejections(source) > 0).map(([file]) => file)).toEqual([]);
	});

	it("engine Rust never names regex", () => {
		const names = ["process", "agentic", "operator", "adapter", "cli"];
		expect(names.filter((name) => dependencyNames(crateManifest(name)).includes("regex"))).toEqual([]);
		expect(dependencyNames(cargoOf('[dependencies]\nregex = "1"'))).toContain("regex");
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

	it("a consumer runs the scripts with three packages", async () => {
		const scripts = CONSUMER_SCRIPTS.map((script) => path.join(ROOT, "scripts", script));
		const parsed = await parseFiles(scripts);
		for (const [at, script] of CONSUMER_SCRIPTS.entries()) {
			const source = parsed.get(scripts[at] as string) as SourceFile;
			expect([script, foreignImports(script, source)]).toEqual([script, []]);
		}
		const [probe] = await parseProbes([
			[
				'import { x } from "@nyaaai/cli/commands";',
				'import "./caseGroups/json";',
				'import y from "zod";',
				'export { z } from "@nyaaai/operator";',
				'const lazy = await import("../ts/other");',
				'import ok from "@nyaaai/contract/json";',
				'import self from "./consumer";',
			].join("\n"),
		]);
		expect(foreignImports("cases.ts", probe as SourceFile)).toEqual([
			"@nyaaai/cli/commands",
			"./caseGroups/json",
			"@nyaaai/operator",
			"../ts/other",
		]);

		for (const name of CONSUMER_PACKAGES) {
			const manifest = JSON.parse(readFileSync(path.join(ROOT, "ts", name, "package.json"), "utf8")) as Record<
				string,
				Record<string, string> | undefined
			>;
			const needs = [
				...[manifest["dependencies"], manifest["devDependencies"], manifest["peerDependencies"]].flatMap(
					(table) =>
						Object.entries(table ?? {})
							.filter(([, version]) => version.startsWith("workspace"))
							.map(([dependency]) => dependency.replace("@nyaaai/", "")),
				),
				...dependencyNames(crateManifest(name)).map((dependency) => dependency.replace("nyaaai-", "")),
			];
			expect([name, needs.filter((other) => !CONSUMER_PACKAGES.includes(other))]).toEqual([name, []]);
		}
	});

	it("both workspaces hold the same packages", () => {
		const packages = (
			JSON.parse(readFileSync(path.join(ROOT, "ts", "package.json"), "utf8")) as { workspaces: string[] }
		).workspaces;
		const crates = cargoOf(readFileSync(path.join(ROOT, "rust", "Cargo.toml"), "utf8")).workspace?.members ?? [];
		expect(packages.filter((name) => name !== "ui").sort()).toEqual([...crates].sort());
	});
});
