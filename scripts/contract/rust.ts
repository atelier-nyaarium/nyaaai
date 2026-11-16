// Rust shape emitter.

import type { Hook } from "@nyaaai/contract/shape";
import type { RustTarget } from "@nyaaai/testkit/project";
import { pascal, quote, snake } from "@nyaaai/testkit/rustNames";
import type { Field, Node, SetIr, Shape } from "./ir";

////////////////////////////////
//  Interfaces & Types

type Of<Kind extends Node["kind"]> = Extract<Node, { kind: Kind }>;

interface Held {
	expr: string;
	isRef: boolean;
}

/** A decoder expression, and the function it is when it is only a call. */
interface Dec {
	text: string;
	/** Set when `text` is `callee(value, at)`. */
	callee?: string;
}

/** An encoder expression. */
interface Enc {
	text: string;
	/** Reads the value it was handed. */
	reads: boolean;
	/** Set when `text` is `callee(held)`. */
	callee?: string | undefined;
}

/** Imported names the body needs. */
type Needs = Set<"Object" | "Record" | "Value" | "Shape">;

type Named = Extract<Hook, { name: string }>;

type Match = Extract<Hook, { kind: "match" }>;

interface PatternStatic {
	name: string;
	source: string;
	flags: string;
}

////////////////////////////////
//  Constants

const LOCALS = new Set(["object", "value", "at", "held", "hooks", "shape"]);

/** Standalone type kinds. */
const ITEMS = new Set<Node["kind"]>(["object", "enum", "union", "tagged"]);

const DERIVES = "#[derive(Debug, Clone, PartialEq)]";

////////////////////////////////
//  Functions & Helpers

function raw(text: string): string {
	let longest = 0;
	let run = 0;
	for (const char of text) {
		run = char === "#" ? run + 1 : 0;
		longest = Math.max(longest, run);
	}
	const fence = "#".repeat(1 + longest);
	return `r${fence}"${text}"${fence}`;
}

const float = (value: number): string =>
	Number.isInteger(value) && Math.abs(value) < 1e21 ? `${value}.0` : String(value);

const some = (value: number | undefined): string => (value === undefined ? "None" : `Some(${value})`);

const unsigned = (node: Of<"number">): boolean =>
	node.integer && ((node.minimum ?? -1) >= 0 || (node.exclusiveMinimum ?? -1) >= 0);

/** Literals need no payload. */
const constant = (node: Node): node is Of<"literal"> | Of<"null"> =>
	(node.kind === "literal" || node.kind === "null") && !node.hooks;

/** Calls decoders as closures. */
const decoder = (expr: Dec): string => expr.callee ?? `|value, at| ${expr.text}`;

const plainCall = (callee: string): Dec => ({ text: `${callee}(value, at)`, callee });

class Emitter {
	private readonly taken = new Set<string>();
	private readonly names = new Map<Node, string>();
	private readonly pending: Shape[] = [];
	private readonly boxed = new Set<Node>();
	private readonly hookUses = new Map<string, { name: string; use: string }>();
	private readonly patterns = new Map<string, PatternStatic>();
	private readonly needs: Needs = new Set();

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
		this.refuseAliasCycles();
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
		const json = (["Object", "Record", "Value"] as const).filter((name) => this.needs.has(name));
		const stem = own.module.split("::").at(-1);
		const statics = [...this.patterns.values()].map(
			({ name, source, flags }) =>
				`static ${name}: LazyLock<Option<Pattern>> = LazyLock::new(|| Pattern::compile(${quote(source)}, ${quote(flags)}).ok());`,
		);
		return [
			`//! Generated from schema/${this.ir.set}.output.json by scripts/contract.ts. Do not edit.`,
			"",
			"#![allow(clippy::enum_variant_names, clippy::large_enum_variant, clippy::ptr_arg)]",
			"",
			...(statics.length > 0 ? ["use std::sync::LazyLock;", ""] : []),
			`use ${root}::json::{${json.join(", ")}};`,
			...(statics.length > 0 ? [`use ${root}::pattern::Pattern;`] : []),
			`use ${root}::shape::{self, Path, Refused${this.needs.has("Shape") ? ", Shape" : ""}};`,
			...(this.hookUses.size > 0 ? ["", `use super::${stem}_hooks as hooks;`] : []),
			"",
			...(statics.length > 0 ? [...statics, ""] : []),
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

	/** Box cross-set value cycles. */
	private findCycles(): void {
		const key = (set: string, name: string) => `${set}.${name}`;
		const edges = new Map<string, Array<Of<"ref">>>();
		const collect = (node: Node, found: Array<Of<"ref">>): void => {
			if (node.kind === "ref") found.push(node);
			else if (node.kind === "object") for (const field of node.fields) collect(field.node, found);
			else if (node.kind === "nullable") collect(node.inner, found);
			else if (node.kind === "tuple") for (const item of node.items) collect(item, found);
			else if (node.kind === "union") for (const branch of node.branches) collect(branch, found);
			else if (node.kind === "tagged") for (const branch of node.branches) collect(branch.node, found);
		};
		for (const { set, shapes } of this.sets.values()) {
			for (const { name, node } of shapes) {
				const found: Array<Of<"ref">> = [];
				collect(node, found);
				edges.set(key(set, name), found);
			}
		}
		const reaches = (from: string, to: string, seen: Set<string>): boolean => {
			if (from === to) return true;
			if (seen.has(from)) return false;
			seen.add(from);
			return (edges.get(from) ?? []).some((ref) => reaches(key(ref.set, ref.name), to, seen));
		};
		const crate = (set: string) => this.targets.get(set)?.crate;
		for (const { name } of this.ir.shapes) {
			const own = key(this.ir.set, name);
			for (const ref of edges.get(own) ?? []) {
				if (!reaches(key(ref.set, ref.name), own, new Set())) continue;
				if (crate(ref.set) !== crate(this.ir.set)) {
					throw new Error(`${own}: a cycle through ${ref.set}.${ref.name} crosses crates`);
				}
				this.boxed.add(ref);
			}
		}
	}

	/** A type alias cannot name itself, through other aliases either. */
	private refuseAliasCycles(): void {
		const key = (set: string, name: string) => `${set}.${name}`;
		const aliases = new Map<string, Node>();
		for (const { set, shapes } of this.sets.values()) {
			for (const { name, node } of shapes) if (!ITEMS.has(node.kind)) aliases.set(key(set, name), node);
		}
		// An object or a union ends the chain: it is a type of its own.
		const named = (node: Node): string[] => {
			if (node.kind === "ref") return [key(node.set, node.name)];
			if (node.kind === "array") return named(node.items);
			if (node.kind === "record") return named(node.values);
			if (node.kind === "nullable") return named(node.inner);
			return node.kind === "tuple" ? node.items.flatMap(named) : [];
		};
		const reaches = (from: string, to: string, seen: Set<string>): boolean => {
			const node = aliases.get(from);
			if (!node || seen.has(from)) return false;
			seen.add(from);
			return named(node).some((next) => next === to || reaches(next, to, seen));
		};
		for (const { name, node } of this.ir.shapes) {
			const own = key(this.ir.set, name);
			if (!ITEMS.has(node.kind) && reaches(own, own, new Set())) {
				throw new Error(`${own}: a recursive shape is an object or a union; wrap this one in an object`);
			}
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

	/** One Rust function per hook name, serving one use. */
	private hook(hook: Named, use: string): string {
		const rust = snake(hook.name);
		const held = this.hookUses.get(rust);
		if (held && held.name !== hook.name) {
			throw new Error(`${this.ir.set}: hooks ${held.name} and ${hook.name} are both ${rust} in Rust`);
		}
		if (held && held.use !== use) {
			throw new Error(`${this.ir.set}: hook ${hook.name} is both ${held.use} and ${use}`);
		}
		this.hookUses.set(rust, { name: hook.name, use });
		return `hooks::${rust}`;
	}

	private pattern(hook: Match): string {
		const key = `${hook.flags}/${hook.source}`;
		const held = this.patterns.get(key) ?? {
			name: `PATTERN_${this.patterns.size + 1}`,
			source: hook.source,
			flags: hook.flags,
		};
		this.patterns.set(key, held);
		return held.name;
	}

	private typeOf(node: Node, hint: string, needs: Needs = this.needs): string {
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
				needs.add("Value");
				return "Value";
			case "array":
				return `Vec<${this.typeOf(node.items, `${hint}Item`, needs)}>`;
			case "tuple":
				return `(${node.items.map((item, at) => `${this.typeOf(item, `${hint}Item${at}`, needs)},`).join(" ")})`;
			case "record":
				needs.add("Record");
				return `Record<${this.typeOf(node.values, `${hint}Value`, needs)}>`;
			case "nullable":
				return `Option<${this.typeOf(node.inner, hint, needs)}>`;
			case "ref": {
				const path = this.path(node.set, node.name);
				return this.boxed.has(node) ? `Box<${path}>` : path;
			}
			default:
				return this.nameOf(node, hint);
		}
	}

	private decodeOf(node: Node, hint: string): Dec {
		if (ITEMS.has(node.kind)) return this.itemDecode(this.nameOf(node, hint));
		return this.hooked(node.hooks, this.plainDecode(node, hint), () => this.typeOf(node, hint, new Set()));
	}

	/** Calls the trait method. */
	private itemDecode(name: string): Dec {
		this.needs.add("Shape");
		return plainCall(`${name}::decode_at`);
	}

	/** Check input type. */
	private hooked(hooks: Hook[] | undefined, plain: Dec, type: () => string): Dec {
		let expr = plain.text;
		const message = (hook: { message?: string }) =>
			hook.message === undefined ? "None" : `Some(${quote(hook.message)})`;
		for (const hook of hooks ?? []) {
			if (hook.kind === "refine") {
				const check = this.hook(hook, `a check of ${type()}`);
				expr = `${expr}.and_then(|held| shape::holds(${check}(&held), held, at, ${message(hook)}))`;
			} else if (hook.kind === "match") {
				const pattern = `${this.pattern(hook)}.as_ref()`;
				expr = `${expr}.and_then(|held| shape::matches(${pattern}, held, at, ${message(hook)}))`;
			}
		}
		for (const hook of hooks ?? []) {
			if (hook.kind === "preprocess") {
				expr = `{ let value = &${this.hook(hook, "a preprocess")}(value); ${expr} }`;
			}
		}
		return expr === plain.text ? plain : { text: expr };
	}

	private plainDecode(node: Node, hint: string): Dec {
		switch (node.kind) {
			case "string":
				return node.minLength === undefined && node.maxLength === undefined
					? plainCall("shape::text")
					: { text: `shape::text_within(value, at, ${some(node.minLength)}, ${some(node.maxLength)})` };
			case "number":
				return this.numberDecode(node);
			case "boolean":
				return plainCall("shape::flag");
			case "null":
				return plainCall("shape::null");
			case "unknown":
				return plainCall("shape::any");
			case "literal":
				return { text: this.literalDecode(node) };
			case "array": {
				const items = decoder(this.decodeOf(node.items, `${hint}Item`));
				return {
					text:
						node.minItems === undefined && node.maxItems === undefined
							? `shape::list(value, at, ${items})`
							: `shape::list_within(value, at, ${some(node.minItems)}, ${some(node.maxItems)}, ${items})`,
				};
			}
			case "tuple": {
				const places = node.items.map((item, at) => {
					const place = decoder(this.decodeOf(item, `${hint}Item${at}`));
					return `shape::place(items, ${at}, at, ${place})?,`;
				});
				return {
					text: `shape::tuple(value, at, ${node.items.length}).and_then(|items| Ok((${places.join(" ")})))`,
				};
			}
			case "record":
				return { text: `shape::record(value, at, ${decoder(this.decodeOf(node.values, `${hint}Value`))})` };
			case "nullable":
				return { text: `shape::nullable(value, at, ${decoder(this.decodeOf(node.inner, hint))})` };
			case "ref": {
				const target = this.shape(node);
				const call = ITEMS.has(target.node.kind)
					? this.itemDecode(this.path(node.set, node.name))
					: plainCall(this.path(node.set, `decode_${snake(node.name)}`));
				return this.boxed.has(node) ? { text: `${call.text}.map(Box::new)` } : call;
			}
			default:
				throw new Error(`${hint}: ${node.kind} is an item`);
		}
	}

	private numberDecode(node: Of<"number">): Dec {
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
		const first = plainCall(`shape::${node.integer ? (natural ? "natural" : "integer") : "number"}`);
		let expr = first.text;
		for (const [check, value] of checks) {
			if (value !== undefined) expr += `.and_then(|held| shape::${check}(held, ${bound(value)}, at))`;
		}
		return expr === first.text ? first : { text: expr };
	}

	private literalDecode(node: Of<"literal">): string {
		if (typeof node.value === "string") return `shape::is_text(value, at, ${quote(node.value)})`;
		if (typeof node.value === "number") return `shape::is_number(value, at, ${float(node.value)})`;
		return `shape::is_flag(value, at, ${node.value})`;
	}

	private encodeOf(node: Node, hint: string, held: Held): Enc {
		const copy = held.isRef ? `*${held.expr}` : held.expr;
		const borrow = held.isRef ? held.expr : `&${held.expr}`;
		const inner = (child: Node, childHint: string) => {
			const made = this.encodeOf(child, childHint, { expr: "held", isRef: true });
			return made.callee ?? `|${made.reads ? "held" : "_"}| ${made.text}`;
		};
		const value = (text: string, reads = true): Enc => {
			this.needs.add("Value");
			return { text, reads };
		};
		switch (node.kind) {
			case "string":
				return value(`Value::from(${held.expr}.as_str())`);
			case "number":
				return value(node.integer ? `Value::Number(${copy} as f64)` : `Value::Number(${copy})`);
			case "boolean":
				return value(`Value::Bool(${copy})`);
			case "null":
				return value("Value::Null", false);
			case "literal":
				return this.literalValue(node);
			case "unknown":
				return { text: `${held.expr}.clone()`, reads: true };
			case "array":
				return value(
					node.items.kind === "unknown"
						? `Value::Array(${held.expr}.clone())`
						: `Value::Array(${held.expr}.iter().map(${inner(node.items, `${hint}Item`)}).collect())`,
				);
			case "tuple": {
				const places = node.items.map((item, at) =>
					this.encodeOf(item, `${hint}Item${at}`, { expr: `${held.expr}.${at}`, isRef: false }),
				);
				return value(
					`Value::Array(vec![${places.map((place) => place.text).join(", ")}])`,
					places.some((place) => place.reads),
				);
			}
			case "record":
				return { text: `shape::record_value(${borrow}, ${inner(node.values, `${hint}Value`)})`, reads: true };
			case "nullable":
				return value(`${held.expr}.as_ref().map_or(Value::Null, ${inner(node.inner, hint)})`);
			case "ref": {
				if (ITEMS.has(this.shape(node).node.kind)) return this.encoded(held);
				const callee = this.path(node.set, `encode_${snake(node.name)}`);
				return { text: `${callee}(${borrow})`, reads: true, callee: held.isRef ? callee : undefined };
			}
			default:
				return this.encoded(held);
		}
	}

	/** Calls the trait method. */
	private encoded(held: Held): Enc {
		this.needs.add("Shape");
		return { text: `${held.expr}.encode()`, reads: true };
	}

	private literalValue(node: Of<"literal">): Enc {
		this.needs.add("Value");
		if (typeof node.value === "string") return { text: `Value::from(${quote(node.value)})`, reads: false };
		if (typeof node.value === "number") return { text: `Value::Number(${float(node.value)})`, reads: false };
		return { text: `Value::Bool(${node.value})`, reads: false };
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
		this.needs.add("Shape");
		this.needs.add("Value");
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
			impl(this.hooked(hooks, plainCall("Self::decode_plain"), () => name).text),
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
				return made.reads
					? `if let Some(held) = &self.${field.rust} { object.insert(${key}, ${made.text}); }`
					: `if self.${field.rust}.is_some() { object.insert(${key}, ${made.text}); }`;
			}
			const held = { expr: `self.${field.rust}`, isRef: false };
			return `object.insert(${key}, ${this.encodeOf(field.node, field.hint, held).text});`;
		});
		this.needs.add("Object");
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
			case "tuple":
				return "Tuple";
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
			const made = this.decodeOf(branch.node, branch.hint).text;
			return branch.unit
				? `if ${made}.is_ok() { return Ok(Self::${branch.rust}); }`
				: `if let Ok(held) = ${made} { return Ok(Self::${branch.rust}(held)); }`;
		});
		const encode = branches.map((branch) => {
			const made = this.encodeOf(branch.node, branch.hint, { expr: "held", isRef: true }).text;
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
							`${quote(branch.value)} => ${this.decodeOf(branch.node, branch.hint).text}.map(Self::${branch.rust}),`,
					),
					`_ => Err(shape::no_branch(at, ${tag})),`,
					"}",
				].join("\n"),
				[
					"match self {",
					...branches.map(
						(branch) =>
							`Self::${branch.rust}(held) => ${this.encodeOf(branch.node, branch.hint, { expr: "held", isRef: true }).text},`,
					),
					"}",
				].join("\n"),
			),
		].join("\n");
	}

	private alias(name: string, node: Node): string {
		const encode = this.encodeOf(node, name, { expr: "held", isRef: true });
		this.needs.add("Value");
		return [
			`pub type ${name} = ${this.typeOf(node, name)};`,
			"",
			`pub fn decode_${snake(name)}(value: &Value, at: &mut Path) -> Result<${name}, Refused> {`,
			this.decodeOf(node, name).text,
			"}",
			"",
			`pub fn encode_${snake(name)}(${encode.reads ? "held" : "_held"}: &${name}) -> Value {`,
			encode.text,
			"}",
		].join("\n");
	}

	private normalize(): string {
		this.needs.add("Value");
		if (this.ir.shapes.some(({ node }) => ITEMS.has(node.kind))) this.needs.add("Shape");
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
