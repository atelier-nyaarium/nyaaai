// Rust shape emitter.

import type { Hook } from "@nyaaai/contract/shape";
import type { Field, Node, SetIr, Shape } from "./ir";

////////////////////////////////
//  Interfaces & Types

export interface RustTarget {
	/** Rust crate path. */
	crate: string;
	/** Module path in crate. */
	module: string;
	file: string;
	/** Skips named normalization. */
	bare?: true;
}

type Of<Kind extends Node["kind"]> = Extract<Node, { kind: Kind }>;

interface Held {
	expr: string;
	isRef: boolean;
}

////////////////////////////////
//  Constants

const KEYWORDS = new Set(
	`as async await break const continue crate dyn else enum extern false fn for gen if impl in let loop match mod move
	mut pub ref return self Self static struct super trait true try type unsafe use where while abstract become box do
	final macro override priv typeof unsized virtual yield`.split(/\s+/),
);

const LOCALS = new Set(["object", "value", "at", "held", "hooks", "shape"]);

/** Standalone type kinds. */
const ITEMS = new Set<Node["kind"]>(["object", "enum", "union", "tagged"]);

const DERIVES = "#[derive(Debug, Clone, PartialEq)]";

////////////////////////////////
//  Functions & Helpers

function words(text: string): string[] {
	return text
		.replace(/([a-z0-9])([A-Z])/g, "$1 $2")
		.split(/[^A-Za-z0-9]+/)
		.filter((word) => word.length > 0);
}

function identifier(text: string, made: string): string {
	if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(made)) throw new Error(`${JSON.stringify(text)} names no Rust identifier`);
	return KEYWORDS.has(made) ? `${made}_` : made;
}

export const snake = (text: string): string =>
	identifier(
		text,
		words(text)
			.map((word) => word.toLowerCase())
			.join("_"),
	);

export const pascal = (text: string): string =>
	identifier(
		text,
		words(text)
			.map((word) => `${word.charAt(0).toUpperCase()}${word.slice(1)}`)
			.join(""),
	);

/** ASCII-only Rust literal. */
export function quote(text: string): string {
	let made = '"';
	for (const char of text) {
		const point = char.codePointAt(0) as number;
		if (char === '"' || char === "\\") made += `\\${char}`;
		else if (point >= 0x20 && point < 0x7f) made += char;
		else made += `\\u{${point.toString(16)}}`;
	}
	return `${made}"`;
}

function raw(text: string): string {
	const runs = text.match(/#+/g) ?? [];
	const fence = "#".repeat(1 + Math.max(0, ...runs.map((run) => run.length)));
	return `r${fence}"${text}"${fence}`;
}

const float = (value: number): string =>
	Number.isInteger(value) && Math.abs(value) < 1e21 ? `${value}.0` : String(value);

const some = (value: number | undefined): string => (value === undefined ? "None" : `Some(${value})`);

const unsigned = (node: Of<"number">): boolean =>
	node.integer && ((node.minimum ?? -1) >= 0 || (node.exclusiveMinimum ?? -1) >= 0);

/** Required literals hold no data. */
const constant = (node: Node): node is Of<"literal"> | Of<"null"> =>
	(node.kind === "literal" || node.kind === "null") && !node.hooks;

/** Calls decoders as closures. */
function decoder(expr: string): string {
	const plain = /^([\w:]+)\(value, at\)$/.exec(expr);
	return plain?.[1] ?? `|value, at| ${expr}`;
}

class Emitter {
	private readonly taken = new Set<string>();
	private readonly names = new Map<Node, string>();
	private readonly pending: Shape[] = [];
	private readonly boxed = new Set<Node>();
	private usesHooks = false;

	constructor(
		private readonly ir: SetIr,
		private readonly sets: Map<string, SetIr>,
		private readonly targets: Map<string, RustTarget>,
	) {
		for (const { name, node } of ir.shapes) {
			if (pascal(name) !== name) throw new Error(`${ir.set}.${name}: a shape name is PascalCase`);
			this.taken.add(name);
			if (ITEMS.has(node.kind)) this.names.set(node, name);
		}
		this.findCycles();
	}

	emit(): string {
		const items: string[] = [];
		for (const shape of this.ir.shapes) {
			items.push(this.item(shape));
			for (let held = this.pending.shift(); held; held = this.pending.shift()) items.push(this.item(held));
		}
		const own = this.target(this.ir.set);
		if (!own.bare) items.push(this.normalize());
		const body = items.join("\n\n");
		const root = own.crate === "nyaaai_contract" ? "crate" : "nyaaai_contract";
		const json = ["Object", "Record", "Value"].filter((name) => new RegExp(`\\b${name}\\b`).test(body));
		const stem = own.module.split("::").at(-1);
		return [
			`//! Generated from schema/${this.ir.set}.output.json by scripts/contract.ts. Do not edit.`,
			"",
			"#![allow(clippy::enum_variant_names, clippy::large_enum_variant, clippy::ptr_arg)]",
			"",
			`use ${root}::json::{${json.join(", ")}};`,
			`use ${root}::shape::{self, Path, Refused${/\bShape\b/.test(body) ? ", Shape" : ""}};`,
			...(this.usesHooks ? ["", `use super::${stem}_hooks as hooks;`] : []),
			"",
			body,
			"",
		].join("\n");
	}

	private target(set: string): RustTarget {
		const held = this.targets.get(set);
		if (!held) throw new Error(`set ${set} has no Rust target`);
		return held;
	}

	private shape(node: Of<"ref">): Shape {
		const held = this.sets.get(node.set)?.shapes.find((shape) => shape.name === node.name);
		if (!held) throw new Error(`${node.set}.${node.name} names no shape`);
		return held;
	}

	private path(set: string, name: string): string {
		if (set === this.ir.set) return name;
		const target = this.target(set);
		const root = target.crate === this.target(this.ir.set).crate ? "crate" : target.crate;
		return `${root}::${target.module}::${name}`;
	}

	/** Box recursive by-value references. */
	private findCycles(): void {
		const edges = new Map<string, Array<Of<"ref">>>();
		const collect = (node: Node, found: Array<Of<"ref">>): void => {
			if (node.kind === "ref" && node.set === this.ir.set) found.push(node);
			else if (node.kind === "object") for (const field of node.fields) collect(field.node, found);
			else if (node.kind === "nullable") collect(node.inner, found);
			else if (node.kind === "union") for (const branch of node.branches) collect(branch, found);
			else if (node.kind === "tagged") for (const branch of node.branches) collect(branch.node, found);
		};
		for (const { name, node } of this.ir.shapes) {
			const found: Array<Of<"ref">> = [];
			collect(node, found);
			edges.set(name, found);
		}
		const reaches = (from: string, to: string, seen: Set<string>): boolean => {
			if (from === to) return true;
			if (seen.has(from)) return false;
			seen.add(from);
			return (edges.get(from) ?? []).some((ref) => reaches(ref.name, to, seen));
		};
		for (const [name, refs] of edges) {
			for (const ref of refs) if (reaches(ref.name, name, new Set())) this.boxed.add(ref);
		}
	}

	private nameOf(node: Node, hint: string): string {
		const held = this.names.get(node);
		if (held) return held;
		if (this.taken.has(hint)) throw new Error(`${this.ir.set}: two types are named ${hint}`);
		this.taken.add(hint);
		this.names.set(node, hint);
		this.pending.push({ name: hint, node });
		return hint;
	}

	private hook(hook: Hook): string {
		this.usesHooks = true;
		return `hooks::${snake(hook.name)}`;
	}

	private typeOf(node: Node, hint: string): string {
		switch (node.kind) {
			case "string":
				return "String";
			case "number":
				return node.integer ? (unsigned(node) ? "u64" : "i64") : "f64";
			case "boolean":
				return "bool";
			case "null":
			case "literal":
				return "()";
			case "unknown":
				return "Value";
			case "array":
				return `Vec<${this.typeOf(node.items, `${hint}Item`)}>`;
			case "record":
				return `Record<${this.typeOf(node.values, `${hint}Value`)}>`;
			case "nullable":
				return `Option<${this.typeOf(node.inner, hint)}>`;
			case "ref": {
				const path = this.path(node.set, node.name);
				return this.boxed.has(node) ? `Box<${path}>` : path;
			}
			default:
				return this.nameOf(node, hint);
		}
	}

	private decodeOf(node: Node, hint: string): string {
		if (ITEMS.has(node.kind)) return `${this.nameOf(node, hint)}::decode_at(value, at)`;
		return this.hooked(node.hooks, this.plainDecode(node, hint));
	}

	private hooked(hooks: Hook[] | undefined, plain: string): string {
		let expr = plain;
		for (const hook of hooks ?? []) {
			if (hook.kind === "refine") {
				expr = `${expr}.and_then(|held| shape::holds(${this.hook(hook)}(&held), held, at))`;
			}
		}
		for (const hook of hooks ?? []) {
			if (hook.kind === "preprocess") expr = `{ let value = &${this.hook(hook)}(value); ${expr} }`;
		}
		return expr;
	}

	private plainDecode(node: Node, hint: string): string {
		switch (node.kind) {
			case "string":
				return node.minLength === undefined && node.maxLength === undefined
					? "shape::text(value, at)"
					: `shape::text_within(value, at, ${some(node.minLength)}, ${some(node.maxLength)})`;
			case "number":
				return this.numberDecode(node);
			case "boolean":
				return "shape::flag(value, at)";
			case "null":
				return "shape::null(value, at)";
			case "unknown":
				return "shape::any(value, at)";
			case "literal":
				return this.literalDecode(node);
			case "array": {
				const items = decoder(this.decodeOf(node.items, `${hint}Item`));
				return node.minItems === undefined && node.maxItems === undefined
					? `shape::list(value, at, ${items})`
					: `shape::list_within(value, at, ${some(node.minItems)}, ${some(node.maxItems)}, ${items})`;
			}
			case "record":
				return `shape::record(value, at, ${decoder(this.decodeOf(node.values, `${hint}Value`))})`;
			case "nullable":
				return `shape::nullable(value, at, ${decoder(this.decodeOf(node.inner, hint))})`;
			case "ref": {
				const target = this.shape(node);
				const call = ITEMS.has(target.node.kind)
					? `${this.path(node.set, node.name)}::decode_at(value, at)`
					: `${this.path(node.set, `decode_${snake(node.name)}`)}(value, at)`;
				return this.boxed.has(node) ? `${call}.map(Box::new)` : call;
			}
			default:
				throw new Error(`${hint}: ${node.kind} is an item`);
		}
	}

	private numberDecode(node: Of<"number">): string {
		const natural = unsigned(node);
		const bound = (value: number) => {
			if (!node.integer) return float(value);
			if (!Number.isInteger(value)) throw new Error(`an integer bound of ${value}`);
			return String(value);
		};
		const checks: Array<[string, number | undefined]> = [
			["at_least", natural && node.minimum === 0 ? undefined : node.minimum],
			["above", node.exclusiveMinimum],
			["at_most", node.maximum],
			["below", node.exclusiveMaximum],
		];
		let expr = `shape::${node.integer ? (natural ? "natural" : "integer") : "number"}(value, at)`;
		for (const [check, value] of checks) {
			if (value !== undefined) expr += `.and_then(|held| shape::${check}(held, ${bound(value)}, at))`;
		}
		return expr;
	}

	private literalDecode(node: Of<"literal">): string {
		if (typeof node.value === "string") return `shape::is_text(value, at, ${quote(node.value)})`;
		if (typeof node.value === "number") return `shape::is_number(value, at, ${float(node.value)})`;
		return `shape::is_flag(value, at, ${node.value})`;
	}

	private encodeOf(node: Node, hint: string, held: Held): string {
		const copy = held.isRef ? `*${held.expr}` : held.expr;
		const borrow = held.isRef ? held.expr : `&${held.expr}`;
		const inner = (child: Node, childHint: string) => {
			const made = this.encodeOf(child, childHint, { expr: "held", isRef: true });
			const plain = /^([\w:]+)\(held\)$/.exec(made);
			return plain?.[1] ?? `|${/\bheld\b/.test(made) ? "held" : "_"}| ${made}`;
		};
		switch (node.kind) {
			case "string":
				return `Value::from(${held.expr}.as_str())`;
			case "number":
				return node.integer ? `Value::Number(${copy} as f64)` : `Value::Number(${copy})`;
			case "boolean":
				return `Value::Bool(${copy})`;
			case "null":
				return "Value::Null";
			case "literal":
				return this.literalValue(node);
			case "unknown":
				return `${held.expr}.clone()`;
			case "array":
				return node.items.kind === "unknown"
					? `Value::Array(${held.expr}.clone())`
					: `Value::Array(${held.expr}.iter().map(${inner(node.items, `${hint}Item`)}).collect())`;
			case "record":
				return `shape::record_value(${borrow}, ${inner(node.values, `${hint}Value`)})`;
			case "nullable":
				return `${held.expr}.as_ref().map_or(Value::Null, ${inner(node.inner, hint)})`;
			case "ref":
				return ITEMS.has(this.shape(node).node.kind)
					? `${held.expr}.encode()`
					: `${this.path(node.set, `encode_${snake(node.name)}`)}(${borrow})`;
			default:
				return `${held.expr}.encode()`;
		}
	}

	private literalValue(node: Of<"literal">): string {
		if (typeof node.value === "string") return `Value::from(${quote(node.value)})`;
		if (typeof node.value === "number") return `Value::Number(${float(node.value)})`;
		return `Value::Bool(${node.value})`;
	}

	private item({ name, node }: Shape): string {
		switch (node.kind) {
			case "object":
				return this.struct(name, node);
			case "enum":
				return this.choice(name, node);
			case "union":
				return this.union(name, node);
			case "tagged":
				return this.tagged(name, node);
			default:
				return this.alias(name, node);
		}
	}

	/** Hooks wrap plain decode. */
	private implShape(name: string, hooks: Hook[] | undefined, body: string, encode: string): string {
		const impl = (decode: string) =>
			[
				`impl Shape for ${name} {`,
				"fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {",
				decode,
				"}",
				"",
				"fn encode(&self) -> Value {",
				encode,
				"}",
				"}",
			].join("\n");
		if (!hooks?.length) return impl(body);
		return [
			`impl ${name} {`,
			"fn decode_plain(value: &Value, at: &mut Path) -> Result<Self, Refused> {",
			body,
			"}",
			"}",
			"",
			impl(this.hooked(hooks, "Self::decode_plain(value, at)")),
		].join("\n");
	}

	private struct(name: string, node: Of<"object">): string {
		const seen = new Set<string>();
		const fields = node.fields.map((field) => {
			const rust = snake(field.name);
			const data = !(field.presence === "required" && constant(field.node));
			if (data && seen.has(rust)) throw new Error(`${name}: two fields are named ${rust}`);
			if (data) seen.add(rust);
			return {
				...field,
				rust,
				data,
				local: LOCALS.has(rust) ? `${rust}_field` : rust,
				hint: `${name}${pascal(field.name)}`,
			};
		});
		const type = (field: Field & { hint: string }) => {
			const held = this.typeOf(field.node, field.hint);
			return field.presence === "optional" ? `Option<${held}>` : held;
		};
		const decode = fields.map((field) => {
			const key = quote(field.name);
			const inner = decoder(this.decodeOf(field.node, field.hint));
			const call =
				field.presence === "default"
					? `shape::defaulted(object, ${key}, at, ${raw(JSON.stringify(field.default))}, ${inner})?`
					: `shape::${field.presence}(object, ${key}, at, ${inner})?`;
			return field.data ? `let ${field.local} = ${call};` : `${call};`;
		});
		const built = fields
			.filter((field) => field.data)
			.map((field) => (field.local === field.rust ? field.rust : `${field.rust}: ${field.local}`));
		const encode = fields.map((field) => {
			const key = quote(field.name);
			if (field.presence === "optional") {
				const made = this.encodeOf(field.node, field.hint, { expr: "held", isRef: true });
				return /\bheld\b/.test(made)
					? `if let Some(held) = &self.${field.rust} { object.insert(${key}, ${made}); }`
					: `if self.${field.rust}.is_some() { object.insert(${key}, ${made}); }`;
			}
			const held = { expr: `self.${field.rust}`, isRef: false };
			return `object.insert(${key}, ${this.encodeOf(field.node, field.hint, held)});`;
		});
		return [
			DERIVES,
			`pub struct ${name} {`,
			...fields.filter((field) => field.data).map((field) => `pub ${field.rust}: ${type(field)},`),
			"}",
			"",
			this.implShape(
				name,
				node.hooks,
				[
					fields.length === 0 ? "shape::object(value, at)?;" : "let object = shape::object(value, at)?;",
					...decode,
					`Ok(Self { ${built.join(", ")} })`,
				].join("\n"),
				fields.length === 0
					? "Value::Object(Object::new())"
					: ["let mut object = Object::new();", ...encode, "Value::Object(object)"].join("\n"),
			),
		].join("\n");
	}

	private choice(name: string, node: Of<"enum">): string {
		const variants = node.values.map((value) => ({ value, rust: pascal(value) }));
		if (new Set(variants.map((variant) => variant.rust)).size !== variants.length) {
			throw new Error(`${name}: two values share a variant name`);
		}
		return [
			"#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]",
			`pub enum ${name} {`,
			...variants.map((variant) => `${variant.rust},`),
			"}",
			"",
			`impl ${name} {`,
			`pub const ALL: [Self; ${variants.length}] = [${variants.map((variant) => `Self::${variant.rust}`).join(", ")}];`,
			"",
			"pub fn as_str(self) -> &'static str {",
			"match self {",
			...variants.map((variant) => `Self::${variant.rust} => ${quote(variant.value)},`),
			"}",
			"}",
			"",
			"pub fn parse(text: &str) -> Option<Self> {",
			"Self::ALL.into_iter().find(|held| held.as_str() == text)",
			"}",
			"}",
			"",
			this.implShape(
				name,
				node.hooks,
				'value.as_str().and_then(Self::parse).ok_or_else(|| shape::refused(at, "not one of the values"))',
				"Value::from(self.as_str())",
			),
		].join("\n");
	}

	/** Variant follows branch kind. */
	private variant(node: Node): string {
		switch (node.kind) {
			case "string":
				return "Text";
			case "number":
				return node.integer ? "Integer" : "Number";
			case "boolean":
				return "Flag";
			case "null":
				return "Null";
			case "unknown":
				return "Any";
			case "literal":
				return pascal(String(node.value));
			case "enum":
				return "Choice";
			case "array":
				return "List";
			case "record":
				return "Record";
			case "object":
				return "Object";
			case "nullable":
				return this.variant(node.inner);
			case "ref":
				return node.name;
			default:
				return "Either";
		}
	}

	private union(name: string, node: Of<"union">): string {
		const drafts = node.branches.map((branch) => this.variant(branch));
		const branches = node.branches.map((branch, at) => {
			const draft = drafts[at] as string;
			const twins = drafts.filter((held) => held === draft).length;
			const rust =
				twins > 1 ? `${draft}${drafts.slice(0, at + 1).filter((held) => held === draft).length}` : draft;
			return { node: branch, rust, hint: `${name}${rust}`, unit: constant(branch) };
		});
		const decode = branches.map((branch) => {
			const made = this.decodeOf(branch.node, branch.hint);
			return branch.unit
				? `if ${made}.is_ok() { return Ok(Self::${branch.rust}); }`
				: `if let Ok(held) = ${made} { return Ok(Self::${branch.rust}(held)); }`;
		});
		const encode = branches.map((branch) => {
			const made = this.encodeOf(branch.node, branch.hint, { expr: "held", isRef: true });
			return branch.unit ? `Self::${branch.rust} => ${made},` : `Self::${branch.rust}(held) => ${made},`;
		});
		return [
			DERIVES,
			`pub enum ${name} {`,
			...branches.map((branch) =>
				branch.unit ? `${branch.rust},` : `${branch.rust}(${this.typeOf(branch.node, branch.hint)}),`,
			),
			"}",
			"",
			this.implShape(
				name,
				node.hooks,
				[...decode, 'Err(shape::refused(at, "no branch matches"))'].join("\n"),
				["match self {", ...encode, "}"].join("\n"),
			),
		].join("\n");
	}

	private tagged(name: string, node: Of<"tagged">): string {
		const branches = node.branches.map((branch) => {
			const rust = pascal(branch.value);
			return { ...branch, rust, hint: `${name}${rust}` };
		});
		if (new Set(branches.map((branch) => branch.rust)).size !== branches.length) {
			throw new Error(`${name}: two tags share a variant name`);
		}
		const tag = quote(node.tag);
		return [
			DERIVES,
			`pub enum ${name} {`,
			...branches.map((branch) => `${branch.rust}(${this.typeOf(branch.node, branch.hint)}),`),
			"}",
			"",
			this.implShape(
				name,
				node.hooks,
				[
					"let object = shape::object(value, at)?;",
					`match shape::tag(object, ${tag}, at)? {`,
					...branches.map(
						(branch) =>
							`${quote(branch.value)} => ${this.decodeOf(branch.node, branch.hint)}.map(Self::${branch.rust}),`,
					),
					`_ => Err(shape::no_branch(at, ${tag})),`,
					"}",
				].join("\n"),
				[
					"match self {",
					...branches.map(
						(branch) =>
							`Self::${branch.rust}(held) => ${this.encodeOf(branch.node, branch.hint, { expr: "held", isRef: true })},`,
					),
					"}",
				].join("\n"),
			),
		].join("\n");
	}

	private alias(name: string, node: Node): string {
		const encode = this.encodeOf(node, name, { expr: "held", isRef: true });
		return [
			`pub type ${name} = ${this.typeOf(node, name)};`,
			"",
			`pub fn decode_${snake(name)}(value: &Value, at: &mut Path) -> Result<${name}, Refused> {`,
			this.decodeOf(node, name),
			"}",
			"",
			`pub fn encode_${snake(name)}(${/\bheld\b/.test(encode) ? "held" : "_held"}: &${name}) -> Value {`,
			encode,
			"}",
		].join("\n");
	}

	private normalize(): string {
		const arms = this.ir.shapes.map(({ name, node }) =>
			ITEMS.has(node.kind)
				? `${quote(name)} => ${name}::decode_at(value, at).map(|held| held.encode()),`
				: `${quote(name)} => decode_${snake(name)}(value, at).map(|held| encode_${snake(name)}(&held)),`,
		);
		return [
			"/// Decodes then encodes a named shape.",
			"pub fn normalize(shape: &str, value: &Value) -> Option<Result<Value, Refused>> {",
			"let at = &mut Path::new();",
			"Some(match shape {",
			...arms,
			"_ => return None,",
			"})",
			"}",
		].join("\n");
	}
}

/** rustfmt formats the output. */
export function emitRust(ir: SetIr, sets: Map<string, SetIr>, targets: Map<string, RustTarget>): string {
	return new Emitter(ir, sets, targets).emit();
}
