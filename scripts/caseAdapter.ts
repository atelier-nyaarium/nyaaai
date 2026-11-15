// TypeScript case adapter.

import { parseJson, stringifyJson } from "../ts/contract/src/json";
import { isoTime, parseIsoTime } from "../ts/contract/src/time";
import { versionIn } from "../ts/operator/src/claude/cliVersion";
import {
	type CutPorts,
	type CutRequest,
	cutSession,
	indexSession,
	pickTurn,
	planCut,
	rehomed,
	type SessionIndex,
	type TurnPick,
	verifyCut,
} from "../ts/operator/src/claude/sessionCut";
import { linesOf, sessionLines } from "../ts/operator/src/claude/sessionFile";
import { compactedSession } from "../ts/testkit/src/claudeSession";

////////////////////////////////
//  Interfaces & Types

export interface CaseSource {
	/** Dotted group and operation. */
	kind: string;
	name: string;
	input: unknown;
}

/** LF rows, base64 bytes, or a local file. */
export type SessionBytes = { lines: string[] } | { base64: string } | { file: string };

/** How to extract the message body. */
export type BodyOf = "whole" | "afterFirstLine";

export interface CutCaseInput {
	source: SessionBytes;
	request: CutRequest;
	cliVersion: string;
	sourcePath: string;
	/** Prefix for minted ids. */
	idPrefix: string;
	nowMs: number;
	/** Absent means no summary. */
	summary?: string;
}

type Handler = (input: never) => unknown | Promise<unknown>;

////////////////////////////////
//  Functions & Helpers

function jsonRoundTrip(input: { texts: string[] }) {
	return {
		results: input.texts.map((text) => {
			const value = parseJson(text);
			return value === undefined ? { ok: false } : { ok: true, text: stringifyJson(value) };
		}),
	};
}

function timeParseIso(input: { texts: string[] }) {
	return {
		results: input.texts.map((text) => {
			const at = parseIsoTime(text);
			return Number.isNaN(at) ? null : at;
		}),
	};
}

const timeIso = (input: { times: number[] }) => ({ texts: input.times.map((ms) => isoTime(ms) ?? null) });

function rowsOf(source: SessionBytes): string[] {
	if ("file" in source) throw new Error("a file source is read as a stream only");
	const bytes =
		"base64" in source ? Buffer.from(source.base64, "base64") : Buffer.from(`${source.lines.join("\n")}\n`);
	return [...linesOf(bytes)];
}

async function* each(rows: string[]): AsyncGenerator<string> {
	yield* rows;
}

/** Files stream, rows load once. */
const streamOf = (source: SessionBytes) => ("file" in source ? sessionLines(source.file) : each(rowsOf(source)));

const indexJson = (index: SessionIndex) => ({ ...index, header: [...index.header] });

const BODIES: Record<BodyOf, (text: string) => string> = {
	whole: (text) => text,
	afterFirstLine: (text) => text.slice(text.indexOf("\n") + 1),
};

const cutRehomed = (input: { rows: string[]; sessionId: string }) => ({
	rows: input.rows.map((row) => rehomed(row, input.sessionId)),
});

async function cutIndex(input: { source: SessionBytes }) {
	return { index: indexJson(await indexSession(streamOf(input.source))) };
}

async function cutPlan(input: { source: SessionBytes; requests: CutRequest[]; cliVersion: string }) {
	const index = await indexSession(streamOf(input.source));
	return { plans: input.requests.map((request) => planCut(index, request, input.cliVersion)) };
}

async function cutCut(input: CutCaseInput) {
	const rows = rowsOf(input.source);
	const built = new Map<string, string[]>();
	let minted = 0;
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
		newId: () => `${input.idPrefix}${++minted}`,
		now: () => new Date(input.nowMs),
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

const fileLinesOf = (input: { base64: string }) => ({ rows: [...linesOf(Buffer.from(input.base64, "base64"))] });

async function cutVerify(input: { source: SessionBytes; sessionId: string; paired: boolean }) {
	return { problems: await verifyCut(each(rowsOf(input.source)), input.sessionId, input.paired) };
}

async function cutPickTurn(input: { source: SessionBytes; picks: Array<TurnPick & { bodyOf: BodyOf }> }) {
	const index = await indexSession(each(rowsOf(input.source)));
	return { picked: input.picks.map(({ bodyOf, ...pick }) => pickTurn(index, pick, BODIES[bodyOf])) };
}

const HANDLERS: Record<string, Handler> = {
	"json.roundTrip": jsonRoundTrip,
	"time.parseIso": timeParseIso,
	"time.iso": timeIso,
	"sessionCut.rehomed": cutRehomed,
	"sessionCut.index": cutIndex,
	"sessionCut.plan": cutPlan,
	"sessionCut.cut": cutCut,
	"sessionCut.verify": cutVerify,
	"sessionCut.pickTurn": cutPickTurn,
	"sessionFile.linesOf": fileLinesOf,
	"claude.versionIn": (input: { texts: string[] }) => ({ found: input.texts.map((text) => versionIn(text) ?? null) }),
	"testkit.compactedSession": () => ({ lines: compactedSession().session.lines }),
};

/** JSON result has no undefined. */
export async function runCase(kind: string, input: unknown): Promise<unknown> {
	const handler = HANDLERS[kind];
	if (!handler) throw new Error(`unknown kind ${kind}`);
	// Match case-file JSON input.
	const decoded = parseJson(JSON.stringify(input));
	return JSON.parse(JSON.stringify(await handler(decoded as never)));
}
