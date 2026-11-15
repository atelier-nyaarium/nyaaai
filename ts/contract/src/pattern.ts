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
	/** Indices start at 1. */
	| { kind: "group"; item: Expr; capture: number | undefined }
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
const MAX_DEPTH = 32;
const MAX_CAPTURES = 99;

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

function isDigit(value: string | undefined): boolean {
	return value !== undefined && value >= "0" && value <= "9";
}

function isHex(value: string | undefined): boolean {
	return (
		(value !== undefined && value >= "0" && value <= "9") ||
		(value !== undefined && value >= "a" && value <= "f") ||
		(value !== undefined && value >= "A" && value <= "F")
	);
}

class Parser {
	private at = 0;
	private depth = 0;
	captures = 0;

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
		if (held === "]" || held === "{" || held === "}") throw new PatternError("unexpected delimiter");
		const [value, after] = scalar(this.source, this.at);
		this.at = after;
		return { kind: "literal", value };
	}

	private group(): Expr {
		this.at += 1;
		this.depth += 1;
		if (this.depth > MAX_DEPTH) throw new PatternError("groups nest too deep");
		let capture: number | undefined;
		if (this.peek() === "?") {
			this.at += 1;
			if (this.peek() === ":") {
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
		} else {
			this.captures += 1;
			capture = this.captures;
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
			this.at += 1;
			const minStart = this.at;
			while (isDigit(this.peek())) this.at += 1;
			if (this.at === minStart) throw new PatternError("invalid quantifier");
			min = Number(this.source.slice(minStart, this.at));
			let hasComma = false;
			let maxText = "";
			if (this.peek() === ",") {
				hasComma = true;
				this.at += 1;
				const maxStart = this.at;
				while (isDigit(this.peek())) this.at += 1;
				maxText = this.source.slice(maxStart, this.at);
			}
			if (this.peek() !== "}") throw new PatternError("invalid quantifier");
			this.at += 1;
			max = hasComma ? (maxText === "" ? undefined : Number(maxText)) : min;
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
			if (code === "0" && isDigit(this.peek())) throw new PatternError("decimal escapes are outside the dialect");
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
				while (isHex(this.peek())) this.at += 1;
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
		if (code >= "1" && code <= "9") throw new PatternError("backreferences are outside the dialect");
		if ("\\/.*+?()[]{}|^$".includes(code) || (inClass && code === "-"))
			return { kind: "literal", value: code.codePointAt(0) as number };
		throw new PatternError("unsupported escape");
	}

	private hexEscape(count: number): number {
		const digits = this.source.slice(this.at, this.at + count);
		if (digits.length !== count || ![...digits].every(isHex)) throw new PatternError("invalid hexadecimal escape");
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
			return { ...item, captures: item.captures || expr.capture !== undefined };
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

function flagsOf(flags: string): Set<string> {
	const seen = new Set<string>();
	for (const flag of flags) {
		if (!"ims".includes(flag) || seen.has(flag)) throw new PatternError("unsupported or repeated flag");
		seen.add(flag);
	}
	return seen;
}

type AssertKind = "Start" | "End" | "LineStart" | "LineEnd" | "Boundary" | "NotBoundary";

type Instruction =
	| { kind: "Char"; ranges: Ranges }
	| { kind: "Split"; first: number; second: number }
	| { kind: "Jump"; target: number }
	| { kind: "Save"; slot: number }
	| { kind: "Assert"; assertion: AssertKind }
	| { kind: "Match" };

interface Thread {
	pc: number;
	captures: Int32Array;
	startIndex: number;
}

interface TextInput {
	text: string;
	values: Uint32Array;
	starts: Uint32Array;
	length: number;
}

interface VmMatch {
	startIndex: number;
	endIndex: number;
	captures: Int32Array;
}

function compileExpression(expr: Expr, program: Instruction[], flags: Set<string>): void {
	switch (expr.kind) {
		case "empty":
			return;
		case "literal": {
			let ranges: Ranges = [[expr.value, expr.value]];
			if (flags.has("i")) ranges = foldAscii(ranges);
			program.push({ kind: "Char", ranges });
			return;
		}
		case "class":
			program.push({ kind: "Char", ranges: expr.ranges });
			return;
		case "dot":
			program.push({ kind: "Char", ranges: flags.has("s") ? SCALARS : complementRanges([[0x0a, 0x0a]]) });
			return;
		case "anchor":
			program.push({
				kind: "Assert",
				assertion:
					expr.value === "start"
						? flags.has("m")
							? "LineStart"
							: "Start"
						: flags.has("m")
							? "LineEnd"
							: "End",
			});
			return;
		case "boundary":
			program.push({ kind: "Assert", assertion: expr.negated ? "NotBoundary" : "Boundary" });
			return;
		case "sequence":
			for (const item of expr.items) compileExpression(item, program, flags);
			return;
		case "alternative": {
			const jumps: number[] = [];
			for (let index = 0; index < expr.items.length - 1; index += 1) {
				const split = program.length;
				program.push({ kind: "Split", first: split + 1, second: -1 });
				compileExpression(expr.items[index] as Expr, program, flags);
				jumps.push(program.length);
				program.push({ kind: "Jump", target: -1 });
				(program[split] as Extract<Instruction, { kind: "Split" }>).second = program.length;
			}
			compileExpression(expr.items[expr.items.length - 1] as Expr, program, flags);
			for (const jump of jumps) (program[jump] as Extract<Instruction, { kind: "Jump" }>).target = program.length;
			return;
		}
		case "group": {
			const { capture } = expr;
			if (capture !== undefined) program.push({ kind: "Save", slot: capture * 2 });
			compileExpression(expr.item, program, flags);
			if (capture !== undefined) program.push({ kind: "Save", slot: capture * 2 + 1 });
			return;
		}
		case "repeat": {
			for (let index = 0; index < expr.min; index += 1) compileExpression(expr.item, program, flags);
			if (expr.max === undefined) {
				const split = program.length;
				program.push({ kind: "Split", first: -1, second: -1 });
				const body = program.length;
				compileExpression(expr.item, program, flags);
				program.push({ kind: "Jump", target: split });
				const out = program.length;
				const held = program[split] as Extract<Instruction, { kind: "Split" }>;
				held.first = expr.lazy ? out : body;
				held.second = expr.lazy ? body : out;
			} else {
				for (let index = expr.min; index < expr.max; index += 1) {
					const split = program.length;
					program.push({ kind: "Split", first: -1, second: -1 });
					const body = program.length;
					compileExpression(expr.item, program, flags);
					const out = program.length;
					const held = program[split] as Extract<Instruction, { kind: "Split" }>;
					held.first = expr.lazy ? out : body;
					held.second = expr.lazy ? body : out;
				}
			}
			return;
		}
	}
}

function compileProgram(tree: Expr, flags: Set<string>): Instruction[] {
	const program: Instruction[] = [{ kind: "Save", slot: 0 }];
	compileExpression(tree, program, flags);
	program.push({ kind: "Save", slot: 1 }, { kind: "Match" });
	return program;
}

function textInput(text: string): TextInput {
	let count = 0;
	for (let at = 0; at < text.length; ) {
		const value = text.codePointAt(at) as number;
		at += value > 0xffff ? 2 : 1;
		count += 1;
	}
	const values = new Uint32Array(count);
	const starts = new Uint32Array(count);
	let index = 0;
	for (let at = 0; at < text.length; ) {
		const value = text.codePointAt(at) as number;
		values[index] = value >= 0xd800 && value <= 0xdfff ? 0xfffd : value;
		starts[index] = at;
		at += value > 0xffff ? 2 : 1;
		index += 1;
	}
	return { text, values, starts, length: text.length };
}

function offsetIndex(input: TextInput, offset: number): number {
	let low = 0;
	let high = input.values.length;
	while (low < high) {
		const middle = (low + high) >>> 1;
		if ((input.starts[middle] as number) < offset) low = middle + 1;
		else high = middle;
	}
	return low;
}

function offsetAt(input: TextInput, index: number): number {
	return index >= input.values.length ? input.length : (input.starts[index] as number);
}

function isWord(value: number | undefined): boolean {
	return (
		value !== undefined &&
		((value >= 0x30 && value <= 0x39) ||
			(value >= 0x41 && value <= 0x5a) ||
			value === 0x5f ||
			(value >= 0x61 && value <= 0x7a))
	);
}

function holdsAssertion(assertion: AssertKind, input: TextInput, index: number): boolean {
	const left = index > 0 ? input.values[index - 1] : undefined;
	const right = index < input.values.length ? input.values[index] : undefined;
	switch (assertion) {
		case "Start":
			return index === 0;
		case "End":
			return index === input.values.length;
		case "LineStart":
			return index === 0 || left === 0x0a;
		case "LineEnd":
			return index === input.values.length || right === 0x0a;
		case "Boundary":
			return isWord(left) !== isWord(right);
		case "NotBoundary":
			return isWord(left) === isWord(right);
	}
}

function addThread(
	program: Instruction[],
	threads: Thread[],
	seen: Float64Array,
	step: number,
	pc: number,
	captures: Int32Array,
	startIndex: number,
	input: TextInput,
	index: number,
): void {
	const stack: Thread[] = [{ pc, captures, startIndex }];
	while (stack.length > 0) {
		const held = stack.pop() as Thread;
		if (seen[held.pc] === step) continue;
		seen[held.pc] = step;
		const instruction = program[held.pc] as Instruction;
		switch (instruction.kind) {
			case "Jump":
				stack.push({ pc: instruction.target, captures: held.captures, startIndex: held.startIndex });
				break;
			case "Split":
				stack.push({ pc: instruction.second, captures: held.captures, startIndex: held.startIndex });
				stack.push({ pc: instruction.first, captures: held.captures, startIndex: held.startIndex });
				break;
			case "Save": {
				const saved = held.captures.slice();
				saved[instruction.slot] = offsetAt(input, index);
				stack.push({
					pc: held.pc + 1,
					captures: saved,
					startIndex: held.startIndex,
				});
				break;
			}
			case "Assert":
				if (holdsAssertion(instruction.assertion, input, index))
					stack.push({ pc: held.pc + 1, captures: held.captures, startIndex: held.startIndex });
				break;
			case "Char":
			case "Match":
				threads.push(held);
				break;
		}
	}
}

function hasScalar(ranges: Ranges, value: number): boolean {
	for (const [start, end] of ranges) if (value >= start && value <= end) return true;
	return false;
}

function runProgram(
	program: Instruction[],
	input: TextInput,
	startIndex: number,
	captureCount: number,
): VmMatch | undefined {
	let current: Thread[] = [];
	let currentSeen = new Float64Array(program.length);
	let nextSeen = new Float64Array(program.length);
	let currentStep = 1;
	let nextStep = 2;
	const emptyCaptures = new Int32Array((captureCount + 1) * 2);
	emptyCaptures.fill(-1);
	let candidate: VmMatch | undefined;
	for (let index = startIndex; index <= input.values.length; index += 1) {
		if (candidate === undefined)
			addThread(program, current, currentSeen, currentStep, 0, emptyCaptures, index, input, index);
		const next: Thread[] = [];
		for (const thread of current) {
			const instruction = program[thread.pc] as Instruction;
			if (instruction.kind === "Match") {
				candidate = { startIndex: thread.startIndex, endIndex: index, captures: thread.captures };
				break;
			}
			if (
				instruction.kind === "Char" &&
				index < input.values.length &&
				hasScalar(instruction.ranges, input.values[index] as number)
			)
				addThread(
					program,
					next,
					nextSeen,
					nextStep,
					thread.pc + 1,
					thread.captures,
					thread.startIndex,
					input,
					index + 1,
				);
		}
		if (candidate !== undefined && next.length === 0) break;
		current = next;
		const previousSeen = currentSeen;
		currentSeen = nextSeen;
		nextSeen = previousSeen;
		currentStep = nextStep;
		nextStep += 1;
	}
	return candidate;
}

function compiled(source: string, flagText: string): { program: Instruction[]; captures: number } {
	const flags = flagsOf(flagText);
	const parser = new Parser(source, flags.has("i"));
	const tree = parser.parse();
	if (traitsOf(tree).cost > MAX_COST) throw new PatternError("the pattern is too large");
	if (parser.captures > MAX_CAPTURES) throw new PatternError("too many capture groups");
	return { program: compileProgram(tree, flags), captures: parser.captures };
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
		} else if (next >= "1" && next <= "9") {
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
		private readonly program: Instruction[],
		private readonly captures: number,
	) {}

	static compile(source: string, flags = ""): Pattern {
		const held = compiled(source, flags);
		return new Pattern(source, flags, held.program, held.captures);
	}

	test(text: string): boolean {
		return this.find(text) !== undefined;
	}

	find(text: string, from = 0): PatternMatch | undefined {
		const input = textInput(text);
		const index = offsetIndex(input, offset(text, from));
		const found = runProgram(this.program, input, index, this.captures);
		return found === undefined ? undefined : this.materialize(input, found);
	}

	findAll(text: string): PatternMatch[] {
		return this.all(textInput(text));
	}

	replaceAll(text: string, template: string): string {
		return this.replaceWith(text, (match) => substitute(match, template));
	}

	replaceWith(text: string, each: (match: PatternMatch) => string): string {
		let result = "";
		let tail = 0;
		for (const match of this.all(textInput(text))) {
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
		for (const match of this.all(textInput(text))) {
			if (match.start >= text.length) break;
			if (match.end === piece) continue;
			pieces.push(text.slice(piece, match.start), ...match.captures.map((capture) => capture ?? ""));
			piece = match.end;
		}
		pieces.push(text.slice(piece));
		return pieces;
	}

	private all(input: TextInput): PatternMatch[] {
		const found: PatternMatch[] = [];
		let from = 0;
		while (from <= input.values.length) {
			const held = runProgram(this.program, input, from, this.captures);
			if (held === undefined) break;
			found.push(this.materialize(input, held));
			from = held.endIndex > held.startIndex ? held.endIndex : held.endIndex + 1;
		}
		return found;
	}

	private materialize(input: TextInput, found: VmMatch): PatternMatch {
		const start = found.captures[0] as number;
		const end = found.captures[1] as number;
		const captures: Array<string | null> = [];
		for (let group = 1; group <= this.captures; group += 1) {
			const captureStart = found.captures[group * 2] as number;
			const captureEnd = found.captures[group * 2 + 1] as number;
			captures.push(captureStart < 0 || captureEnd < 0 ? null : input.text.slice(captureStart, captureEnd));
		}
		return { start, end, text: input.text.slice(start, end), captures };
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
