import { describe, expect, it } from "bun:test";
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { CLI_VERSION, compactedSession } from "@nyaaai/testkit/claudeSession";
import { cutSession, verifyCut } from "../sessionCut";
import { builtFile, fileCutPorts, linesOf, sessionLines } from "../sessionFile";

/** Synthetic session folder. */
function scratch() {
	const folder = mkdtempSync(path.join(tmpdir(), "nyaaai-file-ports-"));
	const sessions = path.join(folder, "sessions");
	mkdirSync(sessions);
	const source = path.join(sessions, "source.jsonl");
	const bytes = Buffer.from(`${compactedSession().session.lines.join("\n")}\n`);
	writeFileSync(source, bytes);
	return { folder, sessions, source, bytes };
}

describe("session file rows", () => {
	it("streams the rows a held file splits into, across chunk edges", async () => {
		const folder = mkdtempSync(path.join(tmpdir(), "nyaaai-session-file-"));
		try {
			// Multibyte chars cross chunks.
			const row = (at: number) => `${at} \u{1F600}\u{e9}\u{2028}${"x".repeat(at % 97)}\r`;
			const text = Array.from({ length: 40_000 }, (_, at) => row(at)).join("\n");
			const bytes = Buffer.from(`${text}\nunterminated tail`);
			const file = path.join(folder, "session.jsonl");
			writeFileSync(file, bytes);

			const streamed: string[] = [];
			for await (const line of sessionLines(file)) streamed.push(line);
			expect(bytes.length).toBeGreaterThan(2 * (1 << 20));
			expect(streamed).toEqual([...linesOf(bytes)]);
			expect(streamed).toHaveLength(40_000);
			expect(streamed[7]).toBe(row(7));
		} finally {
			rmSync(folder, { recursive: true, force: true });
		}
	});

	it("builds a sound file in a folder it makes, and leaves the source alone", async () => {
		const { folder, sessions, source, bytes } = scratch();
		try {
			const out = path.join(folder, "out", "nested");
			const ports = fileCutPorts({ source, out, cliVersion: CLI_VERSION, keepOutOf: sessions });
			const done = await cutSession(ports, { mode: "trim" });
			if (!done.ok) throw new Error(done.detail);
			const built = builtFile(out, done.sessionId);
			expect(readdirSync(out)).toEqual([path.basename(built)]);
			expect(await verifyCut(sessionLines(built), done.sessionId, false)).toEqual([]);
			expect(readFileSync(source)).toEqual(bytes);
			expect(readdirSync(sessions)).toEqual(["source.jsonl"]);
		} finally {
			rmSync(folder, { recursive: true, force: true });
		}
	});

	it("refuses to build inside the folder it is kept out of", async () => {
		const { folder, sessions, source, bytes } = scratch();
		try {
			for (const [out, keepOutOf] of [
				[sessions, sessions],
				[path.join(sessions, "not", "made"), sessions],
				[path.join(folder, "out"), path.parse(folder).root],
			] as const) {
				const ports = fileCutPorts({ source, out, cliVersion: CLI_VERSION, keepOutOf });
				await expect(cutSession(ports, { mode: "trim" })).rejects.toThrow();
			}
			expect(readdirSync(folder)).toEqual(["sessions"]);
			expect(readdirSync(sessions)).toEqual(["source.jsonl"]);
			expect(readFileSync(source)).toEqual(bytes);
		} finally {
			rmSync(folder, { recursive: true, force: true });
		}
	});

	it("leaves no file behind when the rows fail halfway", async () => {
		const { folder, source } = scratch();
		try {
			const out = path.join(folder, "out");
			const ports = fileCutPorts({ source, out, cliVersion: CLI_VERSION });
			async function* torn(): AsyncGenerator<string> {
				yield "{}";
				throw new Error("the source went away");
			}
			await expect(ports.create("half", torn())).rejects.toThrow();
			expect(readdirSync(out)).toEqual([]);

			async function* rows(): AsyncGenerator<string> {
				yield "{}";
			}
			for (const id of ["../outside", "a/b", "..", "", "a\\b"]) {
				expect(() => ports.discard(id)).toThrow();
				await expect(ports.create(id, rows())).rejects.toThrow();
			}
			await ports.create("taken", rows());
			await expect(ports.create("taken", torn())).rejects.toThrow();
			expect(readFileSync(path.join(out, "taken.jsonl"), "utf8")).toBe("{}\n");
		} finally {
			rmSync(folder, { recursive: true, force: true });
		}
	});
});
