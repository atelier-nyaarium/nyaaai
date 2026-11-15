// Session file ports.

import { createReadStream } from "node:fs";
import { mkdir, open, realpath, rm } from "node:fs/promises";
import path from "node:path";
import { randomUuid } from "@nyaaai/contract/id";
import type { CutPorts, Summarizer } from "./sessionCut";

////////////////////////////////
//  Interfaces & Types

export interface FileCutOptions {
	source: string;
	/** Built session output directory. */
	out: string;
	cliVersion: string;
	/** Output stays outside. */
	keepOutOf?: string;
	summarize?: Summarizer;
}

////////////////////////////////
//  Constants

const READ_CHUNK_BYTES = 1 << 20;
const WRITE_CHUNK_CHARS = 1 << 20;
const LF = 0x0a;

////////////////////////////////
//  Functions & Helpers

/** Bad-byte replacement; BOM retained. */
const decoder = () => new TextDecoder("utf-8", { ignoreBOM: true });

/** LF-delimited; drops unterminated tail. */
export function* linesOf(bytes: Uint8Array): Generator<string> {
	const text = decoder();
	let from = 0;
	for (let at = bytes.indexOf(LF, from); at >= 0; at = bytes.indexOf(LF, from)) {
		yield text.decode(bytes.subarray(from, at));
		from = at + 1;
	}
}

/** LF-delimited; drops unterminated tail. */
export async function* sessionLines(file: string): AsyncGenerator<string> {
	const text = decoder();
	let rest: Uint8Array[] = [];
	const chunks: AsyncIterable<Uint8Array> = createReadStream(file, { highWaterMark: READ_CHUNK_BYTES });
	for await (const chunk of chunks) {
		let from = 0;
		for (let at = chunk.indexOf(LF, from); at >= 0; at = chunk.indexOf(LF, from)) {
			const piece = chunk.subarray(from, at);
			yield text.decode(rest.length > 0 ? Buffer.concat([...rest, piece]) : piece);
			rest = [];
			from = at + 1;
		}
		if (from < chunk.length) rest.push(chunk.subarray(from));
	}
}

/** Resolves missing path tails. */
async function resolved(target: string): Promise<string> {
	// Unnormalized: a link is followed before its `..`.
	let held = path.isAbsolute(target) ? target : `${process.cwd()}${path.sep}${target}`;
	const tail: string[] = [];
	for (;;) {
		try {
			return path.join(await realpath(held), ...tail.reverse());
		} catch (error) {
			const last = path.basename(held);
			const missing = (error as { code?: string }).code === "ENOENT";
			if (!missing || last === ".." || last === "." || path.dirname(held) === held) throw error;
			tail.push(last);
			held = path.dirname(held);
		}
	}
}

function inside(child: string, parent: string): boolean {
	const relative = path.relative(parent, child);
	return !(relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative));
}

export function builtFile(out: string, sessionId: string): string {
	if (sessionId.length === 0 || /[/\\]|^\.\.?$/.test(sessionId)) throw new Error("the session id is not a file name");
	return path.join(out, `${sessionId}.jsonl`);
}

/** File-backed cut ports. */
export function fileCutPorts(options: FileCutOptions): CutPorts {
	return {
		read: () => sessionLines(options.source),
		sourcePath: options.source,
		create: async (sessionId, lines) => {
			const kept = options.keepOutOf;
			if (kept !== undefined && inside(await resolved(options.out), await resolved(kept))) {
				throw new Error(`${options.out} is inside ${kept}`);
			}
			await mkdir(options.out, { recursive: true });
			const target = builtFile(options.out, sessionId);
			const file = await open(target, "wx");
			try {
				let held = "";
				for await (const line of lines) {
					held += `${line}\n`;
					if (held.length < WRITE_CHUNK_CHARS) continue;
					await file.write(held);
					held = "";
				}
				await file.write(held);
				await file.close();
			} catch (error) {
				// Remove partial output.
				await file.close().catch(() => {});
				await rm(target, { force: true });
				throw error;
			}
		},
		readBuilt: (sessionId) => sessionLines(builtFile(options.out, sessionId)),
		discard: (sessionId) => rm(builtFile(options.out, sessionId)),
		newId: randomUuid,
		now: () => new Date(),
		cliVersion: async () => options.cliVersion,
		...(options.summarize ? { summarize: options.summarize } : {}),
	};
}
