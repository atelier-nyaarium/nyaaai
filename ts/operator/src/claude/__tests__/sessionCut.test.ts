import { describe, expect, it } from "bun:test";
import { CLI_VERSION, compactedSession, SESSION_ID, SyntheticSession } from "@nyaaai/testkit/claudeSession";
import {
	type CutPorts,
	type CutRequest,
	cutSession,
	indexSession,
	pickTurn,
	planCut,
	rehomed,
	verifyCut,
} from "../sessionCut";

async function* each(lines: string[]): AsyncGenerator<string> {
	yield* lines;
}

/** In-memory file ports. */
function portsOver(source: string[], summarize?: CutPorts["summarize"]) {
	const built = new Map<string, string[]>();
	let minted = 0;
	const ports: CutPorts = {
		read: () => each(source),
		sourcePath: "/sessions/old.jsonl",
		create: async (id, lines) => {
			const rows: string[] = [];
			for await (const line of lines) rows.push(line);
			built.set(id, rows);
		},
		readBuilt: (id) => each(built.get(id) ?? []),
		discard: async (id) => void built.delete(id),
		newId: () => `new-${++minted}`,
		now: () => new Date("2026-11-13T02:00:00.000Z"),
		cliVersion: async () => CLI_VERSION,
		...(summarize ? { summarize } : {}),
	};
	return { ports, built };
}

const cut = async (source: string[], request: CutRequest, summarize?: CutPorts["summarize"]) => {
	const { ports, built } = portsOver(source, summarize);
	const result = await cutSession(ports, request);
	return {
		result,
		rows: result.ok ? (built.get(result.sessionId) ?? []).map((line) => JSON.parse(line)) : [],
		built,
	};
};

const reason = (plan: ReturnType<typeof planCut>) => (plan.ok ? "planned" : plan.reason);

describe("claude session cut", () => {
	it("clears above a turn: the kept rows stay byte for byte, behind a boundary that preserves them in order", async () => {
		const { session, second } = compactedSession();
		const { result, rows, built } = await cut(session.lines, { mode: "clear", targetUuid: second });
		if (!result.ok) throw new Error(result.detail);

		const raw = built.get(result.sessionId) ?? [];
		const keptSource = session.lines.slice(session.lines.findIndex((line) => line.includes(`"uuid":"${second}"`)));
		const body = raw.slice(4, 4 + keptSource.length);
		expect(body.map((line) => rehomed(line, SESSION_ID))).toEqual(keptSource);
		// Top-level IDs only.
		expect(body.join("\n")).toContain(`"input":{"session_id":"${SESSION_ID}"}`);
		expect(body.join("\n")).toContain(`ran in ${SESSION_ID}`);

		const boundary = rows.at(-2);
		const summary = rows.at(-1);
		const chain = body.map((line) => JSON.parse(line).uuid).filter(Boolean);
		expect(boundary.compactMetadata.preservedMessages.uuids).toEqual(chain);
		expect(boundary.compactMetadata.preservedSegment).toEqual({
			headUuid: second,
			anchorUuid: summary.uuid,
			tailUuid: chain.at(-1),
		});
		expect(boundary.parentUuid).toBeNull();
		expect(summary.parentUuid).toBe(boundary.uuid);
		expect(summary.message.content).toContain("what stood");
		expect(summary.message.content).toContain("/sessions/old.jsonl");
		expect(rows.every((row) => row.sessionId === undefined || row.sessionId === result.sessionId)).toBe(true);
	});

	it("keeps the CLI's own sum: what stood, plus what this cut dropped", async () => {
		const { session, second } = compactedSession();
		const { rows } = await cut(session.lines, { mode: "clear", targetUuid: second });
		const meta = rows.at(-2).compactMetadata;
		expect(meta.cumulativeDroppedTokens).toBe(1000 + meta.preTokens - meta.postTokens);
		expect(meta.preTokens).toBeGreaterThanOrEqual(meta.postTokens);
	});

	it("trims to the live segment and its preserved rows, copying the CLI's own boundary", async () => {
		const { session, keptA } = compactedSession();
		const { result, rows, built } = await cut(session.lines, { mode: "trim" });
		if (!result.ok) throw new Error(result.detail);

		const texts = rows.map((row) => row.message?.content).filter((content) => typeof content === "string");
		expect(texts).not.toContain("ancient");
		expect(texts.slice(0, 3)).toEqual(["kept before the compaction", "what stood", "first turn"]);
		const boundaries = rows.filter((row) => row.subtype === "compact_boundary");
		expect(boundaries).toHaveLength(1);
		expect(boundaries[0].compactMetadata.preservedSegment.headUuid).toBe(keptA);
		const raw = built.get(result.sessionId) ?? [];
		const source = session.lines.find((line) => line.includes('"subtype":"compact_boundary"')) ?? "";
		expect(raw.map((line) => rehomed(line, SESSION_ID))).toContain(source);
	});

	it("leaves out the request log and a pointer whose row was cut", async () => {
		const { session, third } = compactedSession();
		const { rows } = await cut(session.lines, { mode: "clear", targetUuid: third });
		expect(rows.some((row) => row.type === "api-request-blob")).toBe(false);
		const held = new Set(rows.map((row) => row.uuid));
		expect(rows.filter((row) => row.type === "last-prompt").every((row) => held.has(row.leafUuid))).toBe(true);
		expect(rows.filter((row) => row.type === "last-prompt")).toHaveLength(1);
	});

	it("cuts at a message typed mid-turn once every call above it is answered, and nowhere inside a call", async () => {
		const { session, midTurn } = compactedSession();
		const index = await indexSession(each(session.lines));
		const answer = index.tail.find((row) => row.type === "user" && !row.delivered)?.uuid;
		if (!answer) throw new Error("the fixture holds no tool answer");
		const refusal = (request: CutRequest) => reason(planCut(index, request, CLI_VERSION));

		expect(refusal({ mode: "clear", targetUuid: midTurn })).toBe("planned");
		expect(refusal({ mode: "clear", targetUuid: answer })).toBe("target_not_turn_start");
		expect(refusal({ mode: "clear", targetUuid: "nowhere" })).toBe("target_compacted");
		expect(refusal({ mode: "compact", targetUuid: midTurn })).toBe("planned");

		const open = new SyntheticSession();
		open.user("one");
		open.calls("pending");
		const typed = open.user("typed before the answer");
		open.answers("pending");
		const later = await indexSession(each(open.lines));
		expect(reason(planCut(later, { mode: "clear", targetUuid: typed }, CLI_VERSION))).toBe("target_mid_tool");
	});

	it("refuses a CLI or a file shape it has not been proven on, and rows it cannot read", async () => {
		const { session, second } = compactedSession();
		const index = await indexSession(each(session.lines));
		expect(reason(planCut(index, { mode: "clear", targetUuid: second }, "2.2.0"))).toBe("cli_version");

		const reshaped = async (from: string, to: string) => {
			const odd = session.lines.map((line) =>
				line.includes('"subtype":"compact_boundary"') ? line.replace(from, to) : line,
			);
			expect(odd).not.toEqual(session.lines);
			return reason(planCut(await indexSession(each(odd)), { mode: "trim" }, CLI_VERSION));
		};
		expect(await reshaped('"level":"info"', '"level":"info","surprise":1')).toBe("boundary_shape");
		expect(await reshaped('"postTokens":400', '"postTokens":"400"')).toBe("boundary_shape");
		expect(await reshaped('"uuids":[', '"uuids":[7,')).toBe("boundary_shape");
		expect(await reshaped('"uuids":["u3","u4"]', '"uuids":"u3"')).toBe("boundary_shape");

		const torn = [...session.lines, '{"type":"user","uuid":"torn'];
		expect(reason(planCut(await indexSession(each(torn)), { mode: "trim" }, CLI_VERSION))).toBe("unreadable_rows");

		const fresh = new SyntheticSession();
		fresh.user("only turn");
		expect(reason(planCut(await indexSession(each(fresh.lines)), { mode: "trim" }, CLI_VERSION))).toBe(
			"nothing_to_trim",
		);
	});

	it("counts every compaction row, and keeps only the live one", async () => {
		const { session } = compactedSession();
		const again = session.user("more");
		session.compacts("what stood later", [again]);
		session.user("after both");
		const index = await indexSession(each(session.lines));
		expect(index.boundaries).toBe(2);
		expect(index.standing).toBe("what stood later");
		expect(index.boundary?.preserved).toEqual([again]);
	});

	it("compacts through a summarizer handed what stood and the rows above the cut", async () => {
		const { session, third } = compactedSession();
		const seen: { standing: string | null; rows: number }[] = [];
		const { result, rows } = await cut(session.lines, { mode: "compact", targetUuid: third }, async (input) => {
			seen.push({ standing: input.standing, rows: input.rows.length });
			return "folded";
		});
		expect(result.ok).toBe(true);
		expect(seen).toEqual([{ standing: "what stood", rows: 11 }]);
		expect(rows.at(-1).message.content).toContain("\n\nfolded\n\n");
		expect(rows.at(-1).message.content).toContain("/sessions/old.jsonl");

		const none = await cut(session.lines, { mode: "compact", targetUuid: third });
		expect(none.result).toMatchObject({ ok: false, reason: "summary_needed" });
	});

	it("compacts with a summary it is handed, ending the kept rows before the request that asked for it", async () => {
		const { session, second } = compactedSession();
		const kept = session.lines.length;
		session.user('Owner asked to compact. Call the tool with cutId "cut-00aa".');
		session.calls("t9");
		const source = [...session.lines, '{"type":"assistant","uuid":"torn'];
		const request: CutRequest = {
			mode: "compact",
			targetUuid: second,
			endBefore: "cut-00aa",
			summaryText: "handed",
		};
		const { result, rows } = await cut(source, request);
		if (!result.ok) throw new Error(result.detail);

		expect(rows.at(-1).message.content).toContain("handed");
		expect(JSON.stringify(rows)).not.toContain("cut-00aa");
		expect(result.plan.keptLines.at(-1)).toBe(kept);

		const late = [
			...session.lines,
			JSON.stringify({
				type: "attachment",
				uuid: "late-1",
				parentUuid: "u-last",
				attachment: { type: "queued_command", prompt: "one more thing" },
				sessionId: SESSION_ID,
			}),
		];
		expect((await cut(late, request)).result).toMatchObject({ ok: false, reason: "late_message" });

		const unasked = await cut(source, { ...request, endBefore: "cut-ffff" });
		expect(unasked.result).toMatchObject({ ok: false, reason: "end_missing" });
		// Open calls block cuts.
		const { endBefore: _end, ...whole } = request;
		expect((await cut(session.lines, whole)).result).toMatchObject({ ok: false, reason: "tail_mid_tool" });
	});

	it("finds a chat row's turn by its text, nearest in time, and otherwise the turn at or before it", async () => {
		const bodyOf = (text: string) => text.replace(/^\[[^\]]*\]\n/, "");
		const session = new SyntheticSession();
		session.header();
		const first = session.user("go");
		session.says("went");
		const tagged = session.user("[from the phone]\nship   it");
		session.calls("t1");
		session.user("typed while it worked");
		session.answers("t1");
		const taken = session.absorbs("[from the phone]\nkeep going");
		session.says("shipped");
		const again = session.user("go");
		session.says("went again");
		const index = await indexSession(each(session.lines));
		const at = (second: number) => Date.parse(`2026-11-13T00:00:${String(second).padStart(2, "0")}.000Z`);

		expect(pickTurn(index, { at: at(1), text: "go" }, bodyOf)).toEqual({ uuid: first, exact: true });
		expect(pickTurn(index, { at: at(9), text: "go" }, bodyOf)).toEqual({ uuid: again, exact: true });
		expect(pickTurn(index, { at: at(3), text: "ship it" }, bodyOf)).toEqual({ uuid: tagged, exact: true });
		// Answered mid-turn input: cuttable.
		expect(pickTurn(index, { at: at(7), text: "keep going" }, bodyOf)).toEqual({ uuid: taken, exact: true });
		// Incomplete turns aren't cuttable.
		expect(pickTurn(index, { at: at(4), text: "typed while it worked" })).toEqual({ uuid: tagged, exact: false });
		expect(pickTurn(index, { at: at(8) })).toEqual({ uuid: taken, exact: false });
		expect(pickTurn(index, { at: at(0) - 60_000 })).toBeNull();
	});

	it("reads a built file back for what would stop it resuming", async () => {
		const { session, second } = compactedSession();
		const { result, built } = await cut(session.lines, { mode: "clear", targetUuid: second });
		if (!result.ok) throw new Error(result.detail);
		const sound = built.get(result.sessionId) ?? [];
		expect(await verifyCut(each(sound), result.sessionId, true)).toEqual([]);

		const rewritten = (change: (row: { compactMetadata: { preservedMessages: { uuids: unknown } } }) => void) =>
			sound.map((line) => {
				const row = JSON.parse(line);
				if (row.subtype !== "compact_boundary") return line;
				change(row);
				return JSON.stringify(row);
			});
		const reordered = rewritten((row) => (row.compactMetadata.preservedMessages.uuids as string[]).reverse());
		expect(await verifyCut(each(reordered), result.sessionId, true)).toContain(
			"preserved rows are not in conversation order",
		);
		const unlisted = rewritten((row) => {
			row.compactMetadata.preservedMessages.uuids = "u12";
		});
		expect(await verifyCut(each(unlisted), result.sessionId, true)).toContain("the preserved rows are not a list");
		expect(await verifyCut(each(sound.slice(0, -1)), result.sessionId, true)).toContain(
			"no summary follows the compaction row",
		);
		expect(await verifyCut(each(sound), "another-id", true)).toContain("line 1 names another session");
	});
});
