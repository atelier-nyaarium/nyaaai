// Scripted run cases.

import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { MAX_FRAME_BYTES } from "@nyaaai/cli/adapter";
import { compareText } from "@nyaaai/contract/order";
import type { Played, RunCase, RunStep } from "@nyaaai/testkit/caseFile";
import { type AdapterCommand, AdapterHost } from "./adapterHost";

////////////////////////////////
//  Interfaces & Types

export type Step = RunStep;

export type RunSource = Omit<RunCase, "format" | "expected">;

////////////////////////////////
//  Constants

const TMP = "$TMP";
const CONNECTION = new Set(["welcome", "reject", "fault"]);

////////////////////////////////
//  Functions & Helpers

const rowsOf = (text: string): string[] => (text.endsWith("\n") ? text.slice(0, -1) : text).split("\n");

export async function play(adapter: AdapterCommand, source: RunSource): Promise<Played> {
	const temp = realpathSync(mkdtempSync(path.join(tmpdir(), "nyaaai-run-")));
	const host = new AdapterHost(adapter, (line) => line.replaceAll(temp, TMP));
	try {
		for (const [name, rows] of Object.entries(source.files ?? {})) {
			mkdirSync(path.dirname(path.join(temp, name)), { recursive: true });
			writeFileSync(path.join(temp, name), `${rows.join("\n")}\n`);
		}
		let exit: number | undefined;
		for (const step of source.script) {
			if ("send" in step) host.sendText(JSON.stringify(step.send).replaceAll(TMP, temp));
			else if ("sendText" in step) host.sendText(step.sendText);
			else if ("sendBase64" in step) host.sendBytes(Buffer.from(step.sendBase64, "base64"));
			else if ("sendTail" in step) host.sendTail(step.sendTail);
			else if ("sendOverCap" in step) host.sendText(`"${"a".repeat(MAX_FRAME_BYTES + step.sendOverCap - 2)}"`);
			else if ("await" in step) await host.awaitFrame(...[step.await].flat());
			else exit = await host.close();
		}
		exit ??= await host.close();
		const played: Played = { exit, connection: [], runs: {}, files: {} };
		for (const frame of host.frames) {
			if (CONNECTION.has(frame.type as string)) {
				played.connection.push(frame);
			} else {
				const run = frame.run as string;
				played.runs[run] = [...(played.runs[run] ?? []), frame];
			}
		}
		played.runs = Object.fromEntries(
			Object.entries(played.runs).sort(([left], [right]) => compareText(left, right)),
		);
		for (const name of source.read ?? []) {
			played.files[name] = rowsOf(readFileSync(path.join(temp, name), "utf8").replaceAll(temp, TMP));
		}
		return played;
	} catch (error) {
		host.kill();
		throw new Error(
			`${source.kind} ${source.name} on ${adapter.name}: ${error instanceof Error ? error.message : error}`,
		);
	} finally {
		rmSync(temp, { recursive: true, force: true });
	}
}
