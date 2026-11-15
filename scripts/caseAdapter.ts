// TypeScript case adapter.

import { mkdirSync, mkdtempSync, realpathSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import { isoTime, parseIsoTime } from "@nyaaai/contract/time";
import { versionIn } from "@nyaaai/operator/claude/cliVersion";
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
} from "@nyaaai/operator/claude/sessionCut";
import { linesOf, sessionLines } from "@nyaaai/operator/claude/sessionFile";
import { isInside } from "@nyaaai/process/paths";
import { compactedSession } from "@nyaaai/testkit/claudeSession";

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

/** Folder, file, or link. */
export interface TreeEntry {
	path: string;
	file?: true;
	/** `$ROOT` marks root. */
	link?: string;
}

async function pathsIsInside(input: {
	tree: TreeEntry[];
	checks: Array<{ child: string; parent: string }>;
	/** Relative paths use root. */
	relative?: true;
}) {
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
	"paths.isInside": pathsIsInside,
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
