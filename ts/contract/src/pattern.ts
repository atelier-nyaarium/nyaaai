export interface PatternMatch {
	start: number;
	end: number;
	text: string;
	captures: Array<string | null>;
}

export class PatternError extends Error {
	constructor(message: string) {
		super(message);
		this.name = "PatternError";
	}
}

type Range = [number, number];
type Ranges = Range[];

type Expr =
	| { kind: "empty" }
	| { kind: "literal"; value: number }
	| { kind: "class"; ranges: Ranges }
	| { kind: "dot" }
	| { kind: "anchor"; value: "start" | "end" }
	| { kind: "boundary"; negated: boolean }
	| { kind: "sequence"; items: Expr[] }
	| { kind: "alternative"; items: Expr[] }
	| { kind: "group"; item: Expr; capture: boolean }
	| { kind: "repeat"; item: Expr; min: number; max: number | undefined; lazy: boolean };

type ClassAtom = { kind: "literal"; value: number } | { kind: "ranges"; value: Ranges } | { kind: "hyphen" };

const WORD: Ranges = [
	[0x30, 0x39],
	[0x41, 0x5a],
	[0x5f, 0x5f],
	[0x61, 0x7a],
];
const DIGIT: Ranges = [[0x30, 0x39]];
const SPACE: Ranges = [
	[0x09, 0x0d],
	[0x20, 0x20],
	[0xa0, 0xa0],
	[0x1680, 0x1680],
	[0x2000, 0x200a],
	[0x2028, 0x2029],
	[0x202f, 0x202f],
	[0x205f, 0x205f],
	[0x3000, 0x3000],
	[0xfeff, 0xfeff],
];
const SCALARS: Ranges = [
	[0, 0xd7ff],
	[0xe000, 0x10ffff],
];
const HEX = /^[0-9a-fA-F]+$/;

const MAX_DEPTH = 32;

/** Expanded atoms and ranges. */
const MAX_COST = 10_000;

interface Traits {
	/** Matches empty text. */
	nullable: boolean;
	captures: boolean;
	/** Captures always participate. */
	certain: boolean;
	cost: number;
}

const ATOM: Traits = { nullable: false, captures: false, certain: true, cost: 1 };
const ASSERTION: Traits = { ...ATOM, nullable: true };

function normalizeRanges(ranges: Ranges): Ranges {
	const sorted = ranges
		.flatMap(([start, end]) => {
			const parts: Ranges = [];
			if (start <= 0xd7ff) parts.push([start, Math.min(end, 0xd7ff)]);
			if (end >= 0xe000) parts.push([Math.max(start, 0xe000), end]);
			return parts;
		})
		.filter(([start, end]) => start <= end)
		.sort((left, right) => left[0] - right[0]);
	const held: Ranges = [];
	for (const [start, end] of sorted) {
		const last = held.at(-1);
		if (last && start <= last[1] + 1) last[1] = Math.max(last[1], end);
		else held.push([start, end]);
	}
	return held;
}

function complementRanges(ranges: Ranges): Ranges {
	const present = normalizeRanges(ranges);
	const result: Ranges = [];
	for (const [floor, ceiling] of SCALARS) {
		let next = floor;
		for (const [start, end] of present) {
			if (end < floor || start > ceiling) continue;
			if (start > next) result.push([next, Math.min(start - 1, ceiling)]);
			next = Math.max(next, end + 1);
			if (next > ceiling) break;
		}
		if (next <= ceiling) result.push([next, ceiling]);
	}
	return result;
}

function foldAscii(ranges: Ranges): Ranges {
	const result = [...ranges];
	for (const [start, end] of ranges) {
		const upperStart = Math.max(start, 0x41);
		const upperEnd = Math.min(end, 0x5a);
		if (upperStart <= upperEnd) result.push([upperStart + 0x20, upperEnd + 0x20]);
		const lowerStart = Math.max(start, 0x61);
		const lowerEnd = Math.min(end, 0x7a);
		if (lowerStart <= lowerEnd) result.push([lowerStart - 0x20, lowerEnd - 0x20]);
	}
	return normalizeRanges(result);
}

function rangesOf(value: "d" | "w" | "s" | "D" | "W" | "S"): Ranges {
	const positive = value.toLowerCase() === "d" ? DIGIT : value.toLowerCase() === "w" ? WORD : SPACE;
	return value === value.toLowerCase() ? positive.map(([start, end]) => [start, end]) : complementRanges(positive);
}

function scalar(source: string, at: number): [number, number] {
	const value = source.codePointAt(at);
	if (value === undefined) throw new PatternError("expected a character");
	if (value >= 0xd800 && value <= 0xdfff) throw new PatternError("surrogate code points are refused");
	return [value, at + (value > 0xffff ? 2 : 1)];
}

class Parser {
	private at = 0;
	private depth = 0;

	constructor(
		private readonly source: string,
		private readonly insensitive: boolean,
	) {}

	parse(): Expr {
		const value = this.expression();
		if (this.at !== this.source.length) throw new PatternError("unbalanced group");
		return value;
	}

	private peek(): string | undefined {
		return this.source[this.at];
	}

	private expression(): Expr {
		const branches = [this.sequence()];
		while (this.peek() === "|") {
			this.at += 1;
			branches.push(this.sequence());
		}
		return branches.length === 1 ? (branches[0] as Expr) : { kind: "alternative", items: branches };
	}

	private sequence(): Expr {
		const items: Expr[] = [];
		while (this.at < this.source.length && this.peek() !== ")" && this.peek() !== "|") {
			if (this.isQuantifierStart()) throw new PatternError("quantifier has nothing to repeat");
			let item = this.atom();
			if (this.isQuantifierStart()) item = this.repeat(item);
			items.push(item);
		}
		return items.length === 0
			? { kind: "empty" }
			: items.length === 1
				? (items[0] as Expr)
				: { kind: "sequence", items };
	}

	private isQuantifierStart(): boolean {
		return this.peek() === "*" || this.peek() === "+" || this.peek() === "?" || this.peek() === "{";
	}

	private atom(): Expr {
		const held = this.peek();
		if (held === undefined) throw new PatternError("expected an expression");
		if (held === "(") return this.group();
		if (held === "[") return this.characterClass();
		if (held === ".") {
			this.at += 1;
			return { kind: "dot" };
		}
		if (held === "^") {
			this.at += 1;
			return { kind: "anchor", value: "start" };
		}
		if (held === "$") {
			this.at += 1;
			return { kind: "anchor", value: "end" };
		}
		if (held === "\\") {
			const escaped = this.escape(false);
			if (escaped.kind === "boundary") return { kind: "boundary", negated: escaped.negated };
			if (escaped.kind === "literal") return { kind: "literal", value: escaped.value };
			return { kind: "class", ranges: escaped.ranges };
		}
		if (held === ")") throw new PatternError("unbalanced group");
		if (held === "]" || held === "{") throw new PatternError("unexpected delimiter");
		const [value, after] = scalar(this.source, this.at);
		this.at = after;
		return { kind: "literal", value };
	}

	private group(): Expr {
		this.at += 1;
		this.depth += 1;
		if (this.depth > MAX_DEPTH) throw new PatternError("groups nest too deep");
		let capture = true;
		if (this.peek() === "?") {
			this.at += 1;
			if (this.peek() === ":") {
				capture = false;
				this.at += 1;
			} else {
				if (this.peek() === "=" || this.peek() === "!")
					throw new PatternError("lookaround is outside the dialect");
				if (this.peek() === "<") {
					this.at += 1;
					if (this.peek() === "=" || this.peek() === "!")
						throw new PatternError("lookaround is outside the dialect");
					throw new PatternError("named groups are outside the dialect");
				}
				throw new PatternError("unsupported group syntax");
			}
		}
		const item = this.expression();
		if (this.peek() !== ")") throw new PatternError("unbalanced group");
		this.at += 1;
		this.depth -= 1;
		return { kind: "group", item, capture };
	}

	private repeat(item: Expr): Expr {
		const token = this.peek();
		let min: number;
		let max: number | undefined;
		if (token === "*") {
			min = 0;
			max = undefined;
			this.at += 1;
		} else if (token === "+") {
			min = 1;
			max = undefined;
			this.at += 1;
		} else if (token === "?") {
			min = 0;
			max = 1;
			this.at += 1;
		} else {
			const start = this.at;
			this.at += 1;
			const body = /^\{([0-9]+)(?:,([0-9]*))?\}/.exec(this.source.slice(start));
			if (!body) throw new PatternError("invalid quantifier");
			this.at = start + body[0].length;
			min = Number(body[1]);
			const hasComma = body[2] !== undefined;
			max = hasComma ? (body[2] === "" ? undefined : Number(body[2])) : min;
			if (
				!Number.isSafeInteger(min) ||
				min > 1000 ||
				(max !== undefined && (!Number.isSafeInteger(max) || max > 1000))
			)
				throw new PatternError("quantifier bound exceeds 1000");
			if (max !== undefined && min > max) throw new PatternError("inverted quantifier bounds");
		}
		const lazy = this.peek() === "?";
		if (lazy) this.at += 1;
		return { kind: "repeat", item, min, max, lazy };
	}

	private characterClass(): Expr {
		this.at += 1;
		const negated = this.peek() === "^";
		if (negated) this.at += 1;
		if (this.peek() === "]") throw new PatternError("empty class");
		const atoms: ClassAtom[] = [];
		while (this.at < this.source.length && this.peek() !== "]") {
			if (this.peek() === "\\") {
				const escaped = this.escape(true);
				if (escaped.kind === "boundary") throw new PatternError("word boundary is outside a class");
				atoms.push(
					escaped.kind === "literal"
						? { kind: "literal", value: escaped.value }
						: { kind: "ranges", value: escaped.ranges },
				);
			} else if (this.peek() === "-") {
				this.at += 1;
				atoms.push({ kind: "hyphen" });
			} else {
				const [value, after] = scalar(this.source, this.at);
				this.at = after;
				atoms.push({ kind: "literal", value });
			}
		}
		if (this.peek() !== "]") throw new PatternError("unbalanced class");
		this.at += 1;
		if (atoms.length === 0) throw new PatternError("empty class");
		const ranges: Ranges = [];
		for (let index = 0; index < atoms.length; ) {
			const first = atoms[index] as ClassAtom;
			const hyphen = atoms[index + 1];
			const last = atoms[index + 2];
			if (hyphen?.kind === "hyphen" && last !== undefined) {
				if (first.kind !== "literal" || last.kind !== "literal")
					throw new PatternError("range endpoints must be characters");
				if (first.value > last.value) throw new PatternError("inverted range");
				ranges.push([first.value, last.value]);
				index += 3;
			} else if (first.kind === "hyphen") {
				ranges.push([0x2d, 0x2d]);
				index += 1;
			} else {
				ranges.push(...(first.kind === "literal" ? [[first.value, first.value] as Range] : first.value));
				index += 1;
			}
		}
		let held = normalizeRanges(ranges);
		if (this.insensitive) held = foldAscii(held);
		if (negated) held = complementRanges(held);
		return { kind: "class", ranges: held };
	}

	private escape(
		inClass: boolean,
	):
		| { kind: "literal"; value: number }
		| { kind: "ranges"; ranges: Ranges }
		| { kind: "boundary"; negated: boolean } {
		this.at += 1;
		const code = this.peek();
		if (code === undefined) throw new PatternError("trailing escape");
		this.at += 1;
		const simple: Record<string, number> = { n: 0x0a, r: 0x0d, t: 0x09, f: 0x0c, v: 0x0b, "0": 0 };
		if (simple[code] !== undefined) {
			if (code === "0" && /[0-9]/.test(this.peek() ?? ""))
				throw new PatternError("decimal escapes are outside the dialect");
			return { kind: "literal", value: simple[code] as number };
		}
		if ("dwsDWS".includes(code))
			return { kind: "ranges", ranges: rangesOf(code as "d" | "w" | "s" | "D" | "W" | "S") };
		if (code === "b" || code === "B") {
			if (inClass) throw new PatternError("unsupported class escape");
			return { kind: "boundary", negated: code === "B" };
		}
		if (code === "x") return { kind: "literal", value: this.hexEscape(2) };
		if (code === "u") {
			if (this.peek() === "{") {
				this.at += 1;
				const start = this.at;
				while (/[0-9a-fA-F]/.test(this.peek() ?? "")) this.at += 1;
				const digits = this.source.slice(start, this.at);
				if (digits.length < 1 || digits.length > 6 || this.peek() !== "}")
					throw new PatternError("invalid Unicode escape");
				this.at += 1;
				const value = Number.parseInt(digits, 16);
				if (value > 0x10ffff || (value >= 0xd800 && value <= 0xdfff))
					throw new PatternError("invalid Unicode scalar");
				return { kind: "literal", value };
			}
			return { kind: "literal", value: this.hexEscape(4) };
		}
		if (code === "p" || code === "P") throw new PatternError("Unicode properties are outside the dialect");
		if (code === "c") throw new PatternError("control escapes are outside the dialect");
		if (code === "k") throw new PatternError("backreferences are outside the dialect");
		if (/[1-9]/.test(code)) throw new PatternError("backreferences are outside the dialect");
		if ("\\/.*+?()[]{}|^$-".includes(code)) return { kind: "literal", value: code.codePointAt(0) as number };
		throw new PatternError("unsupported escape");
	}

	private hexEscape(count: number): number {
		const digits = this.source.slice(this.at, this.at + count);
		if (digits.length !== count || !HEX.test(digits)) throw new PatternError("invalid hexadecimal escape");
		this.at += count;
		const value = Number.parseInt(digits, 16);
		if (value >= 0xd800 && value <= 0xdfff) throw new PatternError("surrogate code points are refused");
		return value;
	}
}

const capped = (cost: number): number => Math.min(cost, MAX_COST + 1);

/** Rejects cross-engine differences. */
function traitsOf(expr: Expr): Traits {
	switch (expr.kind) {
		case "empty":
		case "anchor":
		case "boundary":
			return ASSERTION;
		case "literal":
		case "dot":
			return ATOM;
		case "class":
			return { ...ATOM, cost: Math.max(1, expr.ranges.length) };
		case "sequence": {
			const items = expr.items.map(traitsOf);
			return {
				nullable: items.every((item) => item.nullable),
				captures: items.some((item) => item.captures),
				certain: items.every((item) => item.certain),
				cost: capped(items.reduce((sum, item) => sum + item.cost, 0)),
			};
		}
		case "alternative": {
			const items = expr.items.map(traitsOf);
			const captures = items.some((item) => item.captures);
			return {
				nullable: items.some((item) => item.nullable),
				captures,
				// Alternatives can skip captures.
				certain: !captures,
				cost: capped(items.reduce((sum, item) => sum + item.cost, 0)),
			};
		}
		case "group": {
			const item = traitsOf(expr.item);
			return { ...item, captures: item.captures || expr.capture };
		}
		case "repeat": {
			const item = traitsOf(expr.item);
			if (item.nullable) throw new PatternError("a repeated item must take a character");
			if ((expr.max === undefined || expr.max > 1) && !item.certain)
				throw new PatternError("a capture in a repeat must take part in every round");
			return {
				nullable: expr.min === 0,
				captures: item.captures,
				certain: expr.min === 0 ? !item.captures : item.certain,
				cost: capped(item.cost * Math.max(1, expr.max ?? expr.min + 1)),
			};
		}
	}
}

function codePointText(value: number): string {
	return (value >= 0x30 && value <= 0x39) ||
		(value >= 0x41 && value <= 0x5a) ||
		value === 0x5f ||
		(value >= 0x61 && value <= 0x7a)
		? String.fromCodePoint(value)
		: `\\u{${value.toString(16)}}`;
}

function emitRanges(ranges: Ranges): string {
	return `[${ranges
		.map(([start, end]) => `${codePointText(start)}${start === end ? "" : `-${codePointText(end)}`}`)
		.join("")}]`;
}

function emit(expr: Expr, flags: Set<string>): string {
	switch (expr.kind) {
		case "empty":
			return "(?:)";
		case "literal": {
			if (
				flags.has("i") &&
				((expr.value >= 0x41 && expr.value <= 0x5a) || (expr.value >= 0x61 && expr.value <= 0x7a))
			) {
				const folded = expr.value >= 0x41 && expr.value <= 0x5a ? expr.value + 0x20 : expr.value - 0x20;
				return emitRanges(
					normalizeRanges([
						[expr.value, expr.value],
						[folded, folded],
					]),
				);
			}
			return codePointText(expr.value);
		}
		case "class":
			return emitRanges(expr.ranges);
		case "dot":
			return flags.has("s") ? "[\\s\\S]" : "[^\\n]";
		case "anchor":
			if (!flags.has("m")) return expr.value === "start" ? "^" : "(?![\\s\\S])";
			return expr.value === "start" ? "(?<![^\\n])" : "(?![^\\n])";
		case "boundary":
			return expr.negated ? "\\B" : "\\b";
		case "sequence":
			return expr.items.map((item) => emit(item, flags)).join("");
		case "alternative":
			return `(?:${expr.items.map((item) => emit(item, flags)).join("|")})`;
		case "group":
			return `${expr.capture ? "(" : "(?:"}${emit(expr.item, flags)})`;
		case "repeat": {
			const quantifier =
				expr.max === undefined
					? expr.min === 0
						? "*"
						: expr.min === 1
							? "+"
							: `{${expr.min},}`
					: expr.min === 0 && expr.max === 1
						? "?"
						: expr.min === expr.max
							? `{${expr.min}}`
							: `{${expr.min},${expr.max}}`;
			return `(?:${emit(expr.item, flags)})${quantifier}${expr.lazy ? "?" : ""}`;
		}
	}
}

function flagsOf(flags: string): Set<string> {
	const seen = new Set<string>();
	for (const flag of flags) {
		if (!"ims".includes(flag) || seen.has(flag)) throw new PatternError("unsupported or repeated flag");
		seen.add(flag);
	}
	return seen;
}

function compiled(source: string, flagText: string): { regex: RegExp; flags: Set<string> } {
	const flags = flagsOf(flagText);
	const tree = new Parser(source, flags.has("i")).parse();
	if (traitsOf(tree).cost > MAX_COST) throw new PatternError("the pattern is too large");
	try {
		return { regex: new RegExp(emit(tree, flags), "gu"), flags };
	} catch (error) {
		throw new PatternError(error instanceof Error ? error.message : "pattern refused");
	}
}

function offset(text: string, value: number): number {
	const finite = Number.isNaN(value) ? 0 : Math.trunc(value);
	const held = Math.min(text.length, Math.max(0, finite));
	return held > 0 &&
		held < text.length &&
		text.charCodeAt(held - 1) >= 0xd800 &&
		text.charCodeAt(held - 1) <= 0xdbff &&
		text.charCodeAt(held) >= 0xdc00 &&
		text.charCodeAt(held) <= 0xdfff
		? held + 1
		: held;
}

function nextOffset(text: string, at: number): number {
	if (at >= text.length) return text.length + 1;
	return at + ((text.codePointAt(at) ?? 0) > 0xffff ? 2 : 1);
}

function substitute(match: PatternMatch, template: string): string {
	let result = "";
	for (let at = 0; at < template.length; ) {
		if (template[at] !== "$" || at + 1 >= template.length) {
			result += template[at];
			at += 1;
			continue;
		}
		const next = template[at + 1] as string;
		if (next === "$") {
			result += "$";
			at += 2;
		} else if (next === "&") {
			result += match.text;
			at += 2;
		} else if (/[1-9]/.test(next)) {
			result += match.captures[Number(next) - 1] ?? "";
			at += 2;
		} else {
			result += "$";
			at += 1;
		}
	}
	return result;
}

export class Pattern {
	private constructor(
		readonly source: string,
		readonly flags: string,
		private readonly regex: RegExp,
	) {}

	static compile(source: string, flags = ""): Pattern {
		const held = compiled(source, flags);
		return new Pattern(source, flags, held.regex);
	}

	test(text: string): boolean {
		return this.find(text) !== undefined;
	}

	find(text: string, from = 0): PatternMatch | undefined {
		this.regex.lastIndex = offset(text, from);
		const found = this.regex.exec(text);
		if (!found) return undefined;
		const start = found.index;
		const end = start + found[0].length;
		return {
			start,
			end,
			text: found[0],
			captures: found.slice(1).map((capture) => capture ?? null),
		};
	}

	findAll(text: string): PatternMatch[] {
		const found: PatternMatch[] = [];
		let from = 0;
		while (from <= text.length) {
			const match = this.find(text, from);
			if (!match) break;
			found.push(match);
			if (match.end > match.start) from = match.end;
			else {
				from = nextOffset(text, match.end);
				if (from > text.length) break;
			}
		}
		return found;
	}

	replaceAll(text: string, template: string): string {
		return this.replaceWith(text, (match) => substitute(match, template));
	}

	replaceWith(text: string, each: (match: PatternMatch) => string): string {
		let result = "";
		let tail = 0;
		for (const match of this.findAll(text)) {
			result += text.slice(tail, match.start) + each(match);
			tail = match.end;
		}
		return result + text.slice(tail);
	}

	/** Unmatched captures become empty. */
	split(text: string): string[] {
		if (text.length === 0) return this.find(text) ? [] : [""];
		const pieces: string[] = [];
		let piece = 0;
		let from = 0;
		while (from < text.length) {
			const match = this.find(text, from);
			if (!match || match.start >= text.length) break;
			// Skip piece-start empty matches.
			if (match.end === piece) {
				from = nextOffset(text, from);
				continue;
			}
			pieces.push(text.slice(piece, match.start), ...match.captures.map((capture) => capture ?? ""));
			piece = match.end;
			from = match.end;
		}
		pieces.push(text.slice(piece));
		return pieces;
	}
}

export function patternProblem(source: string, flags = ""): string | undefined {
	try {
		compiled(source, flags);
		return undefined;
	} catch (error) {
		return error instanceof Error ? error.message : "pattern refused";
	}
}
