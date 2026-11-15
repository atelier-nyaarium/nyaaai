// Case adapter.

import { mkdirSync, mkdtempSync, realpathSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { base64Decode, base64Encode } from "@nyaaai/contract/base64";
import { countedIds } from "@nyaaai/contract/id";
import { canonicalJson, jsString, parseJson, stringifyJson } from "@nyaaai/contract/json";
import { fixedText, integerOfText, numberOfText, numberText, roundHalfUp } from "@nyaaai/contract/number";
import { compareText, sortedTexts } from "@nyaaai/contract/order";
import { Pattern, patternProblem } from "@nyaaai/contract/pattern";
import { sequenceRandom } from "@nyaaai/contract/random";
import type { Json } from "@nyaaai/contract/schemaExport";
import { compatibleVersions, decode, decodeText, type Input, type Output, shapeSets } from "@nyaaai/contract/shape";
import { type Issue, normalizeSchema, schemaProblems, validate } from "@nyaaai/contract/strictSchema";
import {
	asciiLower,
	asciiUpper,
	cutAfter,
	cutBefore,
	cutUtf8,
	headOf,
	indexOfText,
	lastIndexOfText,
	sliceText,
	tailOf,
	textLength,
	utf8Length,
} from "@nyaaai/contract/text";
import {
	dayKey,
	isoTime,
	parseHttpDate,
	parseIsoTime,
	parseRetryAfter,
	resetStampMs,
	steppedClock,
} from "@nyaaai/contract/time";
import { versionIn } from "@nyaaai/operator/claude/cliVersion";
import {
	type CutPorts,
	cutSession,
	indexSession,
	pickTurn,
	planCut,
	rehomed,
	verifyCut,
} from "@nyaaai/operator/claude/sessionCut";
import { linesOf, sessionLines } from "@nyaaai/operator/claude/sessionFile";
import { isInside } from "@nyaaai/process/paths";
import { type BodyOf, CASE_KINDS, type CaseKind, type SessionSource } from "@nyaaai/testkit/caseKinds";
import { compactedSession } from "@nyaaai/testkit/claudeSession";
import { TimerWheel } from "@nyaaai/testkit/scheduler";
import "@nyaaai/testkit/tortureShapes";
import { readTrace, replayTransport } from "@nyaaai/testkit/trace";
import { ROOT } from "./child";

////////////////////////////////
//  Interfaces & Types

export interface CaseSource {
	/** Dotted case kind. */
	kind: string;
	name: string;
	input: unknown;
}

type Awaitable<T> = T | Promise<T>;

/** `a.b` maps to `case:a.b`. */
export const CASE_PREFIX = "case:";

type Handlers = {
	[Kind in CaseKind]: (
		input: Output<(typeof CASE_KINDS)[Kind]["input"]>,
	) => Awaitable<Input<(typeof CASE_KINDS)[Kind]["result"]>>;
};

type InputOf<Kind extends CaseKind> = Output<(typeof CASE_KINDS)[Kind]["input"]>;

////////////////////////////////
//  Functions & Helpers

function jsonRoundTrip(input: InputOf<"json.roundTrip">) {
	return {
		results: input.texts.map((text) => {
			const value = parseJson(text);
			return value === undefined ? { ok: false } : { ok: true, text: stringifyJson(value) };
		}),
	};
}

function timeParseIso(input: InputOf<"time.parseIso">) {
	return {
		results: input.texts.map((text) => {
			const at = parseIsoTime(text);
			return Number.isNaN(at) ? null : at;
		}),
	};
}

function jsonCanonical(input: InputOf<"json.canonical">) {
	return {
		texts: input.texts.map((text) => {
			const value = parseJson(text);
			return value === undefined ? null : canonicalJson(value);
		}),
	};
}

function contractRoundTrip(input: InputOf<"contract.roundTrip">) {
	const schema = shapeSets().find((set) => set.name === input.set)?.shapes[input.shape];
	if (!schema) throw new Error(`no shape ${input.set}.${input.shape}`);
	return {
		results: input.texts.map((text) => {
			const held = decodeText(schema, text);
			return held.ok ? { ok: true, text: stringifyJson(held.value) } : { ok: false, path: held.path };
		}),
	};
}

const timeIso = (input: InputOf<"time.iso">) => ({ texts: input.times.map((ms) => isoTime(ms) ?? null) });

function rowsOf(source: SessionSource): string[] {
	if (source.base64 !== undefined) return [...linesOf(Buffer.from(source.base64, "base64"))];
	if (source.lines === undefined) throw new Error("a file source is read as a stream only");
	return [...linesOf(Buffer.from(`${source.lines.join("\n")}\n`))];
}

async function* each(rows: string[]): AsyncGenerator<string> {
	yield* rows;
}

/** Files stream; rows preload. */
const streamOf = (source: SessionSource) =>
	source.file !== undefined ? sessionLines(source.file) : each(rowsOf(source));

const BODIES: Record<BodyOf, (text: string) => string> = {
	whole: (text) => text,
	afterFirstLine: (text) => text.slice(text.indexOf("\n") + 1),
};

const cutRehomed = (input: InputOf<"sessionCut.rehomed">) => ({
	rows: input.rows.map((row) => rehomed(row, input.sessionId)),
});

async function cutIndex(input: InputOf<"sessionCut.index">) {
	return { index: await indexSession(streamOf(input.source)) };
}

async function cutPlan(input: InputOf<"sessionCut.plan">) {
	const index = await indexSession(streamOf(input.source));
	return { plans: input.requests.map((request) => planCut(index, request, input.cliVersion)) };
}

async function cutCut(input: InputOf<"sessionCut.cut">) {
	const rows = rowsOf(input.source);
	const built = new Map<string, string[]>();
	let summarized: { standing: string | null; rows: string[] } | null = null;
	const ports: CutPorts = {
		read: () => each(rows),
		sourcePath: input.sourcePath,
		create: async (id, lines) => {
			const held: string[] = [];
			for await (const line of lines) held.push(line);
			built.set(id, held);
		},
		readBuilt: (id) => each(built.get(id) ?? []),
		discard: async (id) => void built.delete(id),
		ids: countedIds(input.idPrefix),
		clock: steppedClock(input.nowMs),
		cliVersion: async () => input.cliVersion,
		...(input.summary !== undefined
			? {
					summarize: async (given) => {
						summarized = given;
						return input.summary as string;
					},
				}
			: {}),
	};
	try {
		const result = await cutSession(ports, input.request);
		return { result, built: result.ok ? (built.get(result.sessionId) ?? null) : null, summarized };
	} catch (error) {
		return { error: error instanceof Error ? error.message : String(error) };
	}
}

const fileLinesOf = (input: InputOf<"sessionFile.linesOf">) => ({
	rows: [...linesOf(Buffer.from(input.base64, "base64"))],
});

async function cutVerify(input: InputOf<"sessionCut.verify">) {
	return { problems: await verifyCut(each(rowsOf(input.source)), input.sessionId, input.paired) };
}

async function cutPickTurn(input: InputOf<"sessionCut.pickTurn">) {
	const index = await indexSession(each(rowsOf(input.source)));
	return { picked: input.picks.map(({ bodyOf, ...pick }) => pickTurn(index, pick, BODIES[bodyOf])) };
}

async function pathsIsInside(input: InputOf<"paths.isInside">) {
	const root = realpathSync(mkdtempSync(path.join(tmpdir(), "nyaaai-paths-")));
	const started = process.cwd();
	try {
		for (const entry of input.tree) {
			const at = `${root}/${entry.path}`;
			if (entry.link !== undefined) symlinkSync(entry.link.replaceAll("$ROOT", root), at);
			else if (entry.file) writeFileSync(at, "");
			else mkdirSync(at, { recursive: true });
		}
		if (input.relative) process.chdir(root);
		const place = (text: string) => (input.relative || text.startsWith("/") ? text : `${root}/${text}`);
		const results: Array<boolean | "error"> = [];
		for (const check of input.checks) {
			results.push(await isInside(place(check.child), place(check.parent)).catch(() => "error" as const));
		}
		return { results };
	} finally {
		process.chdir(started);
		rmSync(root, { recursive: true, force: true });
	}
}

async function testkitReplay(input: InputOf<"testkit.replay">) {
	const trace = await readTrace(path.join(ROOT, input.file));
	const transport = replayTransport(trace);
	const exchanges = [];
	for (const exchange of trace.exchanges) {
		const response = await transport.fetch("http://replay.invalid/", { body: stringifyJson(exchange.request) });
		const chunks: Uint8Array[] = [];
		for await (const chunk of response.body as unknown as AsyncIterable<Uint8Array>) chunks.push(chunk);
		const headers = [...response.headers].sort(([left], [right]) => compareText(left, right));
		exchanges.push({
			status: response.status,
			headers: Object.fromEntries(headers),
			chunks: chunks.map((chunk) => chunk.length),
			body: Buffer.concat(chunks).toString("utf8"),
		});
	}
	return { requests: transport.requests, exchanges };
}

function issuesOf(schema: unknown, text: string): Issue[] {
	const value = parseJson(text);
	if (value === undefined) throw new Error("a value text is not JSON");
	return validate(schema as Json, value as Json);
}

export const HANDLERS: Handlers = {
	"json.roundTrip": jsonRoundTrip,
	"shape.compatibleVersions": (input) => ({
		compatible: input.pairs.map((pair) => compatibleVersions(pair.left, pair.right)),
	}),
	"base64.decode": (input) => ({
		texts: input.texts.map((text) => {
			const bytes = base64Decode(text);
			return bytes === undefined ? null : base64Encode(bytes);
		}),
	}),
	"json.canonical": jsonCanonical,
	"js.string": (input) => ({
		strings: input.texts.map((text) => {
			const value = parseJson(text);
			if (value === undefined) throw new Error("a value text is not JSON");
			return jsString(value);
		}),
	}),
	"order.sortedTexts": (input) => ({ texts: sortedTexts(input.texts) }),
	"contract.roundTrip": contractRoundTrip,
	"time.parseIso": timeParseIso,
	"time.iso": timeIso,
	"time.httpDate": (input) => ({
		results: input.texts.map((text) => parseHttpDate(text, input.nowMs) ?? null),
	}),
	"time.retryAfter": (input) => ({
		results: input.headers.map((header) => parseRetryAfter(header, input.nowMs) ?? null),
	}),
	"time.resetStamp": (input) => ({
		results: input.values.map((value) => resetStampMs(value, input.nowMs) ?? null),
	}),
	"time.dayKey": (input) => ({
		rows: input.times.map((ms) => input.offsets.map((offset) => dayKey(ms, offset) ?? null)),
	}),
	"number.text": (input) => ({ texts: input.values.map(numberText) }),
	// Preserve negative zero.
	"number.round": (input) => ({
		texts: input.values.map((value) => (Object.is(roundHalfUp(value), -0) ? "-0" : numberText(roundHalfUp(value)))),
	}),
	"number.fixed": (input) => ({
		rows: input.values.map((value) => input.digits.map((digits) => fixedText(value, digits))),
	}),
	"number.ofText": (input) => ({ values: input.texts.map((text) => numberOfText(text) ?? null) }),
	"number.integerOfText": (input) => ({ values: input.texts.map((text) => integerOfText(text) ?? null) }),
	"text.cuts": (input) => ({
		rows: input.texts.map((text) => ({
			length: textLength(text),
			utf8: utf8Length(text),
			before: input.points.map((at) => cutBefore(text, at)),
			after: input.points.map((at) => cutAfter(text, at)),
			heads: input.points.map((max) => headOf(text, max)),
			tails: input.points.map((max) => tailOf(text, max)),
			bytes: input.points.map((max) => cutUtf8(text, max)),
			lower: asciiLower(text),
			upper: asciiUpper(text),
		})),
	}),
	"text.slices": (input) => ({ slices: input.spans.map((span) => sliceText(input.text, span.start, span.end)) }),
	"text.find": (input) => ({
		first: input.needles.map((needle) => input.froms.map((from) => indexOfText(input.text, needle, from))),
		last: input.needles.map((needle) => input.froms.map((from) => lastIndexOfText(input.text, needle, from))),
	}),
	"sessionCut.rehomed": cutRehomed,
	"sessionCut.index": cutIndex,
	"sessionCut.plan": cutPlan,
	"sessionCut.cut": cutCut,
	"sessionCut.verify": cutVerify,
	"sessionCut.pickTurn": cutPickTurn,
	"sessionFile.linesOf": fileLinesOf,
	"paths.isInside": pathsIsInside,
	"claude.versionIn": (input) => ({ found: input.texts.map((text) => versionIn(text) ?? null) }),
	"pattern.problems": (input) => ({
		refused: input.patterns.map(({ source, flags }) => patternProblem(source, flags) !== undefined),
	}),
	"pattern.matches": (input) => {
		const pattern = Pattern.compile(input.source, input.flags);
		return { results: input.texts.map((text) => ({ test: pattern.test(text), all: pattern.findAll(text) })) };
	},
	"pattern.find": (input) => {
		const pattern = Pattern.compile(input.source, input.flags);
		return { found: input.froms.map((from) => pattern.find(input.text, from) ?? null) };
	},
	"pattern.edits": (input) => {
		const pattern = Pattern.compile(input.source, input.flags);
		return {
			results: input.texts.map((text) => ({
				replaced: pattern.replaceAll(text, input.template),
				pieces: pattern.split(text),
			})),
		};
	},
	"strictSchema.normalize": (input) => ({ schemas: input.schemas.map((schema) => normalizeSchema(schema as Json)) }),
	"strictSchema.problems": (input) => ({
		problems: input.schemas.map((schema) => schemaProblems(normalizeSchema(schema as Json))),
	}),
	"strictSchema.validate": (input) => ({ issues: input.texts.map((text) => issuesOf(input.schema, text)) }),
	"strictSchema.validateList": (input) => ({
		issues: input.counts.map((count) =>
			issuesOf(input.schema, `[${Array.from({ length: count }, () => input.item).join(",")}]`),
		),
	}),
	"testkit.compactedSession": () => ({ lines: compactedSession().session.lines }),
	"testkit.replay": testkitReplay,
	"testkit.wheel": (input) => {
		const wheel = new TimerWheel(input.startMs);
		return {
			steps: input.steps.map((step) => {
				const fired = step.op === "advance" ? wheel.advance(step.ms ?? 0) : [];
				const cancelled = step.op === "cancel" && wheel.cancel(step.tag ?? "");
				if (step.op === "after") wheel.after(step.ms ?? 0, step.tag ?? "");
				return { nowMs: wheel.nowMs(), waiting: wheel.waiting(), fired, cancelled };
			}),
		};
	},
	"testkit.random": (input) => {
		const random = sequenceRandom(input.values);
		return { drawn: Array.from({ length: input.draws }, () => random.next()) };
	},
};

const isKind = (kind: string): kind is CaseKind => Object.hasOwn(CASE_KINDS, kind);

export async function runCase(kind: string, input: unknown): Promise<unknown> {
	if (!isKind(kind)) throw new Error(`unknown kind ${kind}`);
	const shapes = CASE_KINDS[kind];
	// Match serialized case inputs.
	const given = decode(shapes.input, parseJson(JSON.stringify(input)));
	if (!given.ok) throw new Error(`${kind}: the input at ${JSON.stringify(given.path)}: ${given.message}`);
	const handler = HANDLERS[kind] as (input: unknown) => unknown;
	const result = decode(shapes.result, JSON.parse(JSON.stringify(await handler(given.value))));
	if (!result.ok) throw new Error(`${kind}: the result at ${JSON.stringify(result.path)}: ${result.message}`);
	return result.value;
}
