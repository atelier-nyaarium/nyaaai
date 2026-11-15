// Synthetic cases contain no real transcripts.

import type { CutRequest, TurnPick } from "@nyaaai/operator/claude/sessionCut";
import { CLI_VERSION, compactedSession, SESSION_ID, SyntheticSession } from "@nyaaai/testkit/claudeSession";
import type { BodyOf, CaseSource, CutCaseInput, SessionBytes } from "../caseAdapter";

////////////////////////////////
//  Constants

const NOW_MS = 1_794_535_200_000;
const START_MS = 1_794_528_000_000;
const TORN = '{"type":"assistant","uuid":"torn';
const BOUNDARY = '"subtype":"compact_boundary"';

////////////////////////////////
//  Functions & Helpers

const rows = (lines: string[]): SessionBytes => ({ lines: [...lines] });
const raw = (bytes: Uint8Array | string): SessionBytes => ({ base64: Buffer.from(bytes).toString("base64") });
const json = (row: unknown) => JSON.stringify(row);

/** First accepted row UUID. */
function uuidWhere(lines: string[], accepts: (row: Record<string, unknown>) => boolean): string {
	for (const line of lines) {
		const row = JSON.parse(line) as Record<string, unknown>;
		if (accepts(row)) return String(row.uuid);
	}
	throw new Error("no such row");
}

const answersCall = (row: Record<string, unknown>) => json(row).includes('"tool_result"');

const reshaped = (lines: string[], from: string, to: string) => {
	const next = lines.map((line) => (line.includes(BOUNDARY) ? line.replace(from, to) : line));
	if (json(next) === json(lines)) throw new Error(`nothing to reshape: ${from}`);
	return next;
};

const cutCase = (
	name: string,
	source: SessionBytes,
	request: CutRequest,
	more: Partial<CutCaseInput> = {},
): CaseSource => ({
	kind: "sessionCut.cut",
	name,
	input: {
		source,
		request,
		cliVersion: CLI_VERSION,
		sourcePath: "/sessions/old.jsonl",
		idPrefix: "new-",
		nowMs: NOW_MS,
		...more,
	} satisfies CutCaseInput,
});

const planCase = (
	name: string,
	source: SessionBytes,
	requests: CutRequest[],
	cliVersion = CLI_VERSION,
): CaseSource => ({
	kind: "sessionCut.plan",
	name,
	input: { source, requests, cliVersion },
});

const indexCase = (name: string, source: SessionBytes): CaseSource => ({
	kind: "sessionCut.index",
	name,
	input: { source },
});

const verifyCase = (name: string, source: SessionBytes, sessionId: string, paired: boolean): CaseSource => ({
	kind: "sessionCut.verify",
	name,
	input: { source, sessionId, paired },
});

/** Self-compaction, then a tool call. */
function asked() {
	const { session, second } = compactedSession();
	session.user('Owner asked to compact. Call the tool with cutId "cut-00aa".');
	session.calls("t9");
	return { session, second };
}

/** Uncompacted session with a second-turn call. */
function fresh() {
	const session = new SyntheticSession();
	session.header();
	const first = session.user("first");
	session.says("answered");
	const second = session.user("second");
	session.calls("f1");
	session.answers("f1");
	const end = session.says("done");
	return { session, first, second, end };
}

/** Clear output fixture. */
function builtClear(): string[] {
	const { session, second } = compactedSession();
	const kept = session.lines.slice(session.lines.findIndex((line) => line.includes(`"uuid":"${second}"`)));
	const chain = kept.map((line) => (JSON.parse(line) as { uuid?: string }).uuid).filter((id) => id !== undefined);
	const envelope = {
		userType: "external",
		entrypoint: "cli",
		cwd: "/work",
		sessionId: "built-1",
		version: CLI_VERSION,
		gitBranch: "main",
		slug: "a-slug",
	};
	return [
		...kept
			.filter((line) => !line.includes("api-request"))
			.map((line) => line.replaceAll(`"sessionId":"${SESSION_ID}"`, '"sessionId":"built-1"')),
		json({
			parentUuid: null,
			logicalParentUuid: chain.at(-1),
			isSidechain: false,
			type: "system",
			subtype: "compact_boundary",
			content: "Conversation compacted",
			level: "info",
			compactMetadata: {
				trigger: "manual",
				preTokens: 900,
				postTokens: 500,
				cumulativeDroppedTokens: 1400,
				durationMs: 1,
				preservedSegment: { headUuid: chain[0], anchorUuid: "anchor", tailUuid: chain.at(-1) },
				preservedMessages: { anchorUuid: "anchor", uuids: chain, allUuids: chain },
			},
			uuid: "edge",
			timestamp: "2026-11-13T02:00:00.000Z",
			...envelope,
		}),
		json({
			parentUuid: "edge",
			isSidechain: false,
			type: "user",
			message: { role: "user", content: "what stood" },
			isCompactSummary: true,
			uuid: "anchor",
			timestamp: "2026-11-13T02:00:00.000Z",
			...envelope,
		}),
	];
}

function withBoundary(lines: string[], change: (row: Record<string, unknown>) => void): string[] {
	return lines.map((line) => {
		if (!line.includes(BOUNDARY)) return line;
		const row = JSON.parse(line) as Record<string, unknown>;
		change(row);
		return json(row);
	});
}

const metadataOf = (row: Record<string, unknown>) =>
	row.compactMetadata as { preservedMessages: Record<string, unknown>; preservedSegment: Record<string, unknown> };

////////////////////////////////
//  Cases

function cutCases(): CaseSource[] {
	const base = compactedSession();
	const lines = base.session.lines;
	const ask = asked();
	const never = fresh();

	const crlf = lines.map((line) => `${line}\r`);

	const oddEnvelope = compactedSession();
	oddEnvelope.session.raw(
		`{"parentUuid":"u18","type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"odd"}]},"uuid":"odd-1","timestamp":"2026-11-13T00:00:30.000Z","userType":1e21,"cwd":{"b":1,"2":2,"1":3,"b":4},"sessionId":"${SESSION_ID}","version":"${CLI_VERSION}","gitBranch":null,"slug":["a",1.50,-0,{"10":1,"9":2}],"turnPosition":{"z":1,"0":-0,"y":1.0e3}}`,
	);

	const spelled = reshaped(
		reshaped(lines, '"cumulativeDroppedTokens":1000', '"cumulativeDroppedTokens":1.0e3'),
		'"postTokens":400',
		'"postTokens":4.005E2',
	);

	const marks = new SyntheticSession();
	marks.header();
	marks.user("before \u{1F600}\u{2028} the cut");
	marks.says("line\u{2028}separator \u{1F9EA} and \u{2029} paragraph");
	const marked = marks.user("keep \u{1F600} from here \u{2028}on");
	marks.says("done \u{1F600}");

	const duplicated = fresh();
	duplicated.session.raw(
		`{"parentUuid":"${never.end}","type":"user","uuid":"dup-a","uuid":"dup-b","message":{"role":"user","content":"twice"},"sessionId":"first","sessionId":"${SESSION_ID}","version":"${CLI_VERSION}","timestamp":"2026-11-13T00:00:40.000Z"}`,
	);
	duplicated.session.raw(
		`{"parentUuid":"dup-b","type":"assistant","uuid":"dup-c","message":{"role":"assistant","content":[{"type":"text","text":"ok"}]},"session_id":"${SESSION_ID}","sessionId":"${SESSION_ID}","version":"${CLI_VERSION}","nested":{"sessionId":"${SESSION_ID}"}}`,
	);

	const escaped = fresh();
	escaped.session.raw(
		`{"parentUuid":"${never.end}","type":"user","uuid":"esc-a","message":{"role":"user","content":"escaped"},"sessionId":"${SESSION_ID}","session\\u0049d":"${SESSION_ID}"}`,
	);

	const lone = compactedSession();
	const loneLines = lone.session.lines.map((line) =>
		line.includes('"isCompactSummary":true')
			? line.replace("what stood", "stood \\ud83d alone \\ud83d\\ude00")
			: line,
	);

	const orphan = fresh();
	orphan.session.answers("ghost");
	orphan.session.says("after the ghost");

	const twice = compactedSession();
	const more = twice.session.user("more");
	twice.session.says("more answered");
	twice.session.compacts("what stood later", [more]);
	const afterBoth = twice.session.user("after both");
	twice.session.says("done again");
	const last = twice.session.user("last");
	twice.session.says("end");

	const broken = new TextEncoder().encode(
		`${never.session.lines.join("\n")}\n{"parentUuid":"${never.end}","type":"user","uuid":"bad-1","message":{"role":"user","content":"bad `,
	);
	const invalid = new Uint8Array([
		...broken,
		0xff,
		0xe2,
		0x82,
		0x20,
		0xc0,
		0x80,
		0xed,
		0xa0,
		0x80,
		...new TextEncoder().encode(
			` bytes"},"sessionId":"${SESSION_ID}","version":"${CLI_VERSION}"}\n{"parentUuid":"bad-1","type":"assistant","uuid":"bad-2","message":{"role":"assistant","content":[{"type":"text","text":"fine"}]},"sessionId":"${SESSION_ID}"}\n`,
		),
	]);

	return [
		cutCase("clear-above-a-turn", rows(lines), { mode: "clear", targetUuid: base.second }),
		cutCase("clear-at-the-last-turn", rows(lines), { mode: "clear", targetUuid: base.third }),
		cutCase("clear-mid-turn", rows(lines), { mode: "clear", targetUuid: base.midTurn }),
		cutCase("clear-through-a-row", rows(lines), { mode: "clear", targetUuid: base.second, throughUuid: "u13" }),
		cutCase("trim", rows(lines), { mode: "trim" }),
		cutCase(
			"compact-through-a-summarizer",
			rows(lines),
			{ mode: "compact", targetUuid: base.third },
			{ summary: "folded" },
		),
		cutCase(
			"compact-summary-is-trimmed",
			rows(lines),
			{ mode: "compact", targetUuid: base.third },
			{ summary: "\n\t folded \u{a0}\u{2028}\n" },
		),
		cutCase("compact-with-no-summarizer", rows(lines), { mode: "compact", targetUuid: base.third }),
		cutCase("compact-blank-summary", rows(lines), {
			mode: "compact",
			targetUuid: base.third,
			summaryText: " \n\u{feff}",
		}),
		cutCase(
			"compact-blank-from-the-summarizer",
			rows(lines),
			{ mode: "compact", targetUuid: base.third },
			{ summary: "\t" },
		),
		cutCase(
			"compact-handed-summary",
			rows([...ask.session.lines, TORN]),
			{ mode: "compact", targetUuid: ask.second, endBefore: "cut-00aa", summaryText: "handed" },
			{ summary: "never asked" },
		),
		cutCase(
			"compact-late-message",
			rows([
				...ask.session.lines,
				json({
					type: "attachment",
					uuid: "late-1",
					parentUuid: "u-last",
					attachment: { type: "queued_command", prompt: "one more thing" },
					sessionId: SESSION_ID,
				}),
			]),
			{ mode: "compact", targetUuid: ask.second, endBefore: "cut-00aa", summaryText: "handed" },
		),
		cutCase("compact-request-missing", rows(ask.session.lines), {
			mode: "compact",
			targetUuid: ask.second,
			endBefore: "cut-ffff",
			summaryText: "handed",
		}),
		cutCase("compact-ends-inside-a-call", rows(ask.session.lines), {
			mode: "compact",
			targetUuid: ask.second,
			summaryText: "handed",
		}),
		cutCase("compact-live-tail", rows(ask.session.lines), {
			mode: "compact",
			targetUuid: ask.second,
			summaryText: "handed",
			liveTail: true,
		}),
		cutCase("never-compacted-clear", rows(never.session.lines), { mode: "clear", targetUuid: never.second }),
		cutCase(
			"never-compacted-compact",
			rows(never.session.lines),
			{ mode: "compact", targetUuid: never.second },
			{ summary: "the first turn" },
		),
		cutCase("never-compacted-trim", rows(never.session.lines), { mode: "trim" }),
		cutCase("crlf-rows", rows(crlf), { mode: "clear", targetUuid: base.second }),
		cutCase("crlf-rows-trim", rows(crlf), { mode: "trim" }),
		cutCase("odd-envelope", rows(oddEnvelope.session.lines), { mode: "clear", targetUuid: oddEnvelope.third }),
		cutCase("numbers-spelled-oddly", rows(spelled), { mode: "clear", targetUuid: base.second }),
		cutCase("numbers-spelled-oddly-trim", rows(spelled), { mode: "trim" }),
		cutCase("separators-and-astral", rows(marks.lines), { mode: "clear", targetUuid: marked }),
		cutCase("duplicate-keys", rows(duplicated.session.lines), { mode: "clear", targetUuid: "dup-b" }),
		cutCase("escaped-session-key", rows(escaped.session.lines), { mode: "clear", targetUuid: "esc-a" }),
		cutCase("lone-surrogate-in-the-summary", rows(loneLines), { mode: "clear", targetUuid: lone.second }),
		cutCase("answer-without-its-call", rows(orphan.session.lines), { mode: "clear", targetUuid: orphan.second }),
		cutCase("twice-compacted-clear", rows(twice.session.lines), { mode: "clear", targetUuid: last }),
		cutCase(
			"twice-compacted-compact",
			rows(twice.session.lines),
			{ mode: "compact", targetUuid: last },
			{ summary: "both folded" },
		),
		cutCase("twice-compacted-trim", rows(twice.session.lines), { mode: "trim" }),
		cutCase("twice-compacted-first-turn", rows(twice.session.lines), { mode: "clear", targetUuid: afterBoth }),
		cutCase("invalid-utf8", raw(invalid), { mode: "clear", targetUuid: "bad-1" }),
		cutCase(
			"new-id-needs-escaping",
			rows(lines),
			{ mode: "clear", targetUuid: base.second },
			{ idPrefix: 'id "\\\u{1F600}\n' },
		),
		cutCase("far-future", rows(lines), { mode: "clear", targetUuid: base.second }, { nowMs: 253_402_300_800_000 }),
		cutCase(
			"source-path-in-the-summary",
			rows(lines),
			{ mode: "clear", targetUuid: base.second },
			{ sourcePath: '/home/x/a \u{1F600} "b"\\c.jsonl' },
		),
	];
}

function planCases(): CaseSource[] {
	const base = compactedSession();
	const lines = base.session.lines;
	const answer = uuidWhere(lines, answersCall);
	const never = fresh();

	const open = new SyntheticSession();
	open.user("one");
	open.calls("pending");
	const typed = open.user("typed before the answer");
	open.answers("pending");

	const unresolved = new SyntheticSession();
	unresolved.user("a");
	const held = unresolved.says("b");
	unresolved.compacts("s", [held, "missing-1"]);
	unresolved.user("c");

	const tornInside = [...lines];
	tornInside.splice(lines.findIndex((line) => line.includes(`"uuid":"${base.third}"`)) + 1, 0, TORN);
	const tornAbove = [...lines];
	tornAbove.splice(
		lines.findIndex((line) => line.includes(`"uuid":"${base.second}"`)),
		0,
		TORN,
	);

	const repeated = new SyntheticSession();
	repeated.raw('{"type":"user","parentUuid":"outside","uuid":"a","message":{"content":"kept"}}');
	repeated.compacts("s", ["a", "a"]);
	repeated.user("after");

	return [
		planCase("preserved-id-repeated", rows(reshaped(repeated.lines, '"headUuid":"a"', '"headUuid":"elsewhere"')), [
			{ mode: "trim" },
		]),
		planCase("compacted", rows(lines), [
			{ mode: "clear", targetUuid: base.midTurn },
			{ mode: "clear", targetUuid: answer },
			{ mode: "clear", targetUuid: "nowhere" },
			{ mode: "compact", targetUuid: base.midTurn },
			{ mode: "clear" },
			{ mode: "clear", targetUuid: base.second, throughUuid: base.third },
			{ mode: "clear", targetUuid: base.third, throughUuid: base.second },
			{ mode: "clear", targetUuid: base.second, throughUuid: "nowhere" },
			{ mode: "clear", targetUuid: base.second, throughUuid: base.second },
			{ mode: "clear", targetUuid: base.second, endBefore: "" },
			{ mode: "clear", targetUuid: base.second, endBefore: "third", throughUuid: "nowhere" },
			{ mode: "clear", targetUuid: base.second, throughUuid: "u9" },
			{ mode: "clear", targetUuid: base.second, throughUuid: "u9", liveTail: true },
			{ mode: "trim", targetUuid: base.second, endBefore: "x", summaryText: "y" },
			{ mode: "clear", targetUuid: "u6" },
			{ mode: "clear", targetUuid: "s5" },
			{ mode: "clear", targetUuid: "b5" },
		]),
		planCase(
			"installed-cli-unproven",
			rows(lines),
			[{ mode: "clear", targetUuid: base.second }, { mode: "trim" }],
			"2.2.0",
		),
		planCase(
			"file-cli-unproven",
			rows(lines.map((line) => line.replace(`"version":"${CLI_VERSION}"`, '"version":"2.2.0"'))),
			[{ mode: "clear", targetUuid: base.second }, { mode: "trim" }],
		),
		planCase("boundary-extra-key", rows(reshaped(lines, '"level":"info"', '"level":"info","surprise":1')), [
			{ mode: "trim" },
			{ mode: "clear", targetUuid: base.second },
		]),
		planCase("boundary-automatic", rows(reshaped(lines, '"level":"info"', '"level":"info","isMeta":true')), [
			{ mode: "trim" },
		]),
		planCase(
			"boundary-read-versions",
			rows(reshaped(lines, '"durationMs":9', '"durationMs":9,"preCompactArtifactReadVersions":{}')),
			[{ mode: "trim" }],
		),
		planCase("boundary-missing-key", rows(reshaped(lines, '"durationMs":9,', "")), [{ mode: "trim" }]),
		planCase("boundary-text-count", rows(reshaped(lines, '"postTokens":400', '"postTokens":"400"')), [
			{ mode: "trim" },
		]),
		planCase(
			"boundary-null-count",
			rows(reshaped(lines, '"cumulativeDroppedTokens":1000', '"cumulativeDroppedTokens":null')),
			[{ mode: "trim" }],
		),
		planCase("boundary-endless-count", rows(reshaped(lines, '"postTokens":400', '"postTokens":1e999')), [
			{ mode: "trim" },
		]),
		planCase("boundary-number-in-list", rows(reshaped(lines, '"uuids":[', '"uuids":[7,')), [{ mode: "trim" }]),
		planCase("boundary-list-is-text", rows(reshaped(lines, '"uuids":["u3","u4"]', '"uuids":"u3"')), [
			{ mode: "trim" },
		]),
		planCase("boundary-list-absent", rows(reshaped(lines, '"uuids":["u3","u4"],', "")), [{ mode: "trim" }]),
		planCase(
			"boundary-metadata-is-text",
			rows(
				reshaped(
					lines,
					/"compactMetadata":\{.*?\}\},/.exec(lines.find((line) => line.includes(BOUNDARY)) ?? "")?.[0] ?? "",
					'"compactMetadata":"gone",',
				),
			),
			[{ mode: "trim" }],
		),
		planCase("torn-tail", rows([...lines, TORN]), [{ mode: "trim" }, { mode: "clear", targetUuid: base.third }]),
		planCase("torn-inside-the-kept-rows", rows(tornInside), [
			{ mode: "clear", targetUuid: base.third },
			{ mode: "clear", targetUuid: base.second, throughUuid: "u13" },
			{ mode: "trim" },
		]),
		planCase("torn-above-the-kept-rows", rows(tornAbove), [
			{ mode: "clear", targetUuid: base.second },
			{ mode: "trim" },
		]),
		planCase("never-compacted", rows(never.session.lines), [
			{ mode: "trim" },
			{ mode: "clear", targetUuid: "nowhere" },
			{ mode: "clear", targetUuid: never.first },
			{ mode: "clear", targetUuid: never.second },
			{ mode: "compact", targetUuid: never.second },
			{ mode: "clear", targetUuid: never.end },
		]),
		planCase("empty-file", raw(""), [{ mode: "trim" }, { mode: "clear", targetUuid: "u1" }]),
		planCase("blank-lines-only", raw("\n\n\n"), [{ mode: "trim" }]),
		planCase("typed-with-a-call-open", rows(open.lines), [{ mode: "clear", targetUuid: typed }]),
		planCase("preserved-rows-missing", rows(unresolved.lines), [
			{ mode: "trim" },
			{ mode: "clear", targetUuid: "u5" },
		]),
	];
}

function indexCases(): CaseSource[] {
	const base = compactedSession();
	const lines = base.session.lines;
	const never = fresh();
	const text = (type: string, value: unknown) =>
		json({ type, uuid: `m-${type}-${json(value).length}`, message: { role: type, content: value } });

	const shapes = [
		"",
		"[1]",
		'"text"',
		"5",
		"null",
		"true",
		'{"uuid":"no-type"}',
		'{"type":5,"uuid":"n1"}',
		'{"type":["a",null,{"b":1},[2,[3]],true,1e21,-0],"uuid":"n2"}',
		'{"type":{"a":1},"uuid":"n3"}',
		'{"type":null,"uuid":"n4"}',
		'{"type":true,"uuid":"n5"}',
		'{"type":1e999,"uuid":"n6"}',
		'{"type":"user","uuid":""}',
		'{"type":"user","uuid":7}',
		'{"type":"mode","uuid":"h1","version":"0.0.1"}',
		'{"type":"mode"}',
		'{"type":"atis-latch","version":7}',
		text("user", "plain"),
		text("user", []),
		text("user", [
			{ type: "text", text: "a" },
			{ type: "text", text: 7 },
			{ type: "image" },
			"loose",
			null,
			[1],
			5,
		]),
		text("user", [[{ type: "tool_result" }], { type: "text", text: "array part first" }]),
		text("user", [
			{ type: "tool_result", tool_use_id: "c1" },
			{ type: "text", text: "with a result" },
		]),
		text("user", [{ type: "tool_result", tool_use_id: 9 }]),
		text("user", { type: "text", text: "an object" }),
		text("user", 12),
		text("user", null),
		'{"type":"user","uuid":"m-none"}',
		'{"type":"user","uuid":"m-text-message","message":"just text"}',
		'{"type":"user","uuid":"m-list-message","message":[{"content":"x"}]}',
		'{"type":"user","uuid":"m-side","isSidechain":true,"message":{"content":"sidechain"}}',
		'{"type":"user","uuid":"m-sum","isCompactSummary":true,"message":{"content":"summary with no boundary"}}',
		'{"type":"user","uuid":"m-side-text","isSidechain":"true","message":{"content":"sidechain as text"}}',
		text("assistant", [
			{ type: "tool_use", id: "c1" },
			{ type: "tool_use", id: "c2" },
			{ type: "tool_use" },
			{ type: "tool_use", id: 3 },
		]),
		text("assistant", "assistant text"),
		text("user", [
			{ type: "tool_result", tool_use_id: "c2" },
			{ type: "tool_result", tool_use_id: "c2" },
			{ type: "tool_result", tool_use_id: "never" },
		]),
		'{"type":"assistant","uuid":"m-user-part","message":{"content":[{"type":"tool_result","tool_use_id":"c1"}]}}',
		'{"type":"attachment","uuid":"a1","attachment":{"type":"queued_command","prompt":"taken in"}}',
		'{"type":"attachment","uuid":"a2","attachment":{"type":"queued_command","prompt":7}}',
		'{"type":"attachment","uuid":"a3","attachment":{"type":"file","prompt":"not a command"}}',
		'{"type":"attachment","uuid":"a4","attachment":"queued_command"}',
		'{"type":"attachment","uuid":"a5","attachment":["queued_command"]}',
		'{"type":"attachment","uuid":"a6"}',
		'{"type":"last-prompt","leafUuid":"a1","messageId":"m-none","snapshotMessageId":7,"timestamp":"2026-11-13T00:00:00.000Z"}',
		'{"type":"file-history-snapshot","snapshotMessageId":"a1","messageId":null,"timestamp":12}',
		'{"type":"api-request","uuid":"r1","message":{"content":[{"type":"tool_use","id":"logged"}]}}',
		'{"type":"user","uuid":"m-escaped","message":{"content":"lone \\ud83d and pair \\ud83d\\ude00"},"version":"\\ud83d"}',
		`{"type":"user","uuid":"m-deep","message":{"content":${"[".repeat(1022)}${"]".repeat(1022)}}}`,
		`{"type":"user","uuid":"m-too-deep","message":{"content":${"[".repeat(1023)}${"]".repeat(1023)}}}`,
		'{"type":"user","uuid":"m-last","message":{"content":"the last row"},"timestamp":"not a time"}   ',
	];

	const cycles = new SyntheticSession();
	cycles.raw('{"type":"user","uuid":"loop-a","parentUuid":"loop-b","message":{"content":"a"}}');
	cycles.raw('{"type":"user","uuid":"loop-b","parentUuid":"loop-a","message":{"content":"b"}}');
	cycles.compacts("looped", ["loop-a", "loop-b"]);
	cycles.raw('{"type":"user","uuid":"self","parentUuid":"self","message":{"content":"c"}}');
	cycles.compacts("self", ["self"]);
	cycles.raw('{"type":"user","uuid":"k1","parentUuid":7,"message":{"content":"d"}}');
	cycles.raw('{"type":"user","uuid":"k2","parentUuid":"k1","message":{"content":"e"}}');
	cycles.raw('{"type":"user","uuid":"k3","parentUuid":"k2","message":{"content":"f"}}');
	cycles.compacts("head not reached", ["k2", "k3"]);

	const bare = [
		...never.session.lines,
		'{"type":"system","subtype":"compact_boundary"}',
		'{"type":"user","uuid":"after-bare","message":{"content":"after a bare boundary"}}',
		'{"type":"system","subtype":"compact_boundary","uuid":"","compactMetadata":{"preservedSegment":"x","preservedMessages":[1]}}',
		'{"type":"system","subtype":"compact_boundary","uuid":"b-null","compactMetadata":null,"isCompactSummary":true,"message":{"content":"a boundary that is its own summary"}}',
		'{"type":"user","uuid":"second-summary","isCompactSummary":true,"message":{"content":"not the first"}}',
		'{"type":"system","subtype":"compact_boundary ","uuid":"not-a-boundary"}',
	];

	const twice = compactedSession();
	const more = twice.session.user("more");
	twice.session.compacts("what stood later", [more]);
	twice.session.user("after both");

	const bom = new Uint8Array([0xef, 0xbb, 0xbf, ...new TextEncoder().encode(`${never.session.lines.join("\n")}\n`)]);
	const unterminated = `${never.session.lines.join("\n")}\n{"type":"user","uuid":"tail","message":{"content":"no line end"}}`;

	return [
		indexCase("compacted", rows(lines)),
		indexCase("never-compacted", rows(never.session.lines)),
		indexCase("twice-compacted", rows(twice.session.lines)),
		indexCase("row-shapes", rows(shapes)),
		indexCase("preserved-walks", rows(cycles.lines)),
		indexCase("bare-boundaries", rows(bare)),
		indexCase("crlf-rows", rows(lines.map((line) => `${line}\r`))),
		indexCase("empty-file", raw("")),
		indexCase("blank-lines", raw("\n\r\n \n\n")),
		indexCase("byte-order-mark", raw(bom)),
		indexCase("unterminated-tail", raw(unterminated)),
		indexCase("only-an-unterminated-row", raw('{"type":"user","uuid":"tail"}')),
	];
}

function rehomedCases(): CaseSource[] {
	const rehome = (name: string, sessionId: string, texts: string[]): CaseSource => ({
		kind: "sessionCut.rehomed",
		name,
		input: { rows: texts, sessionId },
	});
	return [
		rehome("own-ids-only", "new-id", [
			'{"sessionId":"old","type":"user"}',
			'{"type":"user","session_id":"old","sessionId":"old"}',
			'{"a":"\u{1F600}\u{1F9EA} astral before","sessionId":"old","b":"\u{2028}"}',
			'{"input":{"sessionId":"nested"},"sessionId":"old"}',
			'{"list":[{"sessionId":"nested"}],"sessionId":"old"}',
			'[{"sessionId":"in a list"}]',
			'{"session\\u0049d":"escaped key","sessionId":"old"}',
			'{"sessionId":"a\\"b\\\\","after":"x","session_id":"c\\\\\\"d"}',
			'{"sessionId":123,"session_id":null,"x":{"sessionId":"old"}}',
			'{"sessionId": "spaced","session_id" :"spaced"}',
			'{"sessionId":"one","sessionId":"two"}',
			'{"text":"{\\"sessionId\\":\\"in text\\"}","sessionId":"old"}',
			'{"text":"braces } ] { [ in text","sessionId":"old"}',
			'{"text":"ends with a backslash \\\\","sessionId":"old"}',
			'{"sessionId":"","type":"x"}',
			'{"sessionId":"old"}\r',
			'{"sessionId":"old",\t"session_id":"old"}',
			'"sessionId":"old"',
			'{"sessionId":["old"],"session_id":{"sessionId":"old"}}',
			'{"sessionid":"old","SessionId":"old","sessionId ":"old"}',
		]),
		rehome("damaged-rows", "new-id", [
			"",
			"{",
			'{"sessionId":"old',
			'{"sessionId":"old\\',
			'{"sessionId":',
			'{"sessionId"',
			'{"sessionId',
			'{"sessionId":"old"',
			'}{"sessionId":"old"}',
			']]{"sessionId":"old"}',
			'{{"sessionId":"old"}}',
			'{"a":"unclosed,"sessionId":"old"}',
			'{"a":"x"}{"sessionId":"second object"}',
			'{"a\\":"x","sessionId":"old"}',
		]),
		rehome("id-needs-escaping", 'q"uote\\ \u{1F600}\n\u{2028}\t\u{1}', ['{"sessionId":"old","session_id":"old"}']),
		rehome("empty-id", "", ['{"sessionId":"old"}']),
	];
}

function verifyCases(): CaseSource[] {
	const sound = builtClear();
	const reordered = withBoundary(sound, (row) => (metadataOf(row).preservedMessages.uuids as string[]).reverse());
	const tolerant = [
		...sound.slice(0, -2),
		'{"type":"user","uuid":"ghost-answer","message":{"content":[{"type":"tool_result","tool_use_id":"never-called"}]},"sessionId":"built-1"}',
		...sound.slice(-2),
	];
	return [
		verifyCase("sound", rows(sound), "built-1", true),
		verifyCase("another-session", rows(sound), "another-id", true),
		verifyCase("reordered", rows(reordered), "built-1", true),
		verifyCase("no-summary", rows(sound.slice(0, -1)), "built-1", true),
		verifyCase("no-boundary", rows(sound.slice(0, -2)), "built-1", true),
		verifyCase("two-boundaries", rows([...sound, ...sound.slice(-2)]), "built-1", true),
		verifyCase(
			"summary-not-next",
			rows([...sound.slice(0, -1), '{"type":"mode","sessionId":"built-1"}', sound.at(-1) as string]),
			"built-1",
			true,
		),
		verifyCase(
			"list-is-text",
			rows(
				withBoundary(sound, (row) => {
					metadataOf(row).preservedMessages.uuids = "u12";
				}),
			),
			"built-1",
			true,
		),
		verifyCase(
			"list-holds-other-values",
			rows(
				withBoundary(sound, (row) => {
					metadataOf(row).preservedMessages.uuids = [7, null, { a: 1 }, "u12"];
				}),
			),
			"built-1",
			true,
		),
		verifyCase(
			"list-absent",
			rows(
				withBoundary(sound, (row) => {
					delete metadataOf(row).preservedMessages.uuids;
				}),
			),
			"built-1",
			true,
		),
		verifyCase(
			"ends-differ",
			rows(
				withBoundary(sound, (row) => {
					metadataOf(row).preservedSegment.headUuid = "u13";
				}),
			),
			"built-1",
			true,
		),
		verifyCase(
			"anchors-differ",
			rows(
				withBoundary(sound, (row) => {
					metadataOf(row).preservedMessages.anchorUuid = "elsewhere";
				}),
			),
			"built-1",
			true,
		),
		verifyCase(
			"boundary-has-a-parent",
			rows(
				withBoundary(sound, (row) => {
					row.parentUuid = "u20";
				}),
			),
			"built-1",
			true,
		),
		verifyCase(
			"boundary-parent-absent",
			rows(
				withBoundary(sound, (row) => {
					delete row.parentUuid;
				}),
			),
			"built-1",
			true,
		),
		verifyCase(
			"metadata-absent",
			rows(
				withBoundary(sound, (row) => {
					delete row.compactMetadata;
				}),
			),
			"built-1",
			true,
		),
		verifyCase("answer-without-its-call-paired", rows(tolerant), "built-1", true),
		verifyCase("answer-without-its-call-unpaired", rows(tolerant), "built-1", false),
		verifyCase(
			"rows-a-resume-cannot-take",
			rows([
				'{"type":"api-request","sessionId":"built-1"}',
				"",
				"not json",
				'{"type":"last-prompt","leafUuid":"missing","messageId":"u12","sessionId":"built-1","session_id":"other"}',
				'{"type":"user","sessionId":7,"session_id":"other","uuid":"u12"}',
				...sound,
			]),
			"built-1",
			true,
		),
		verifyCase("empty-file", raw(""), "built-1", true),
		verifyCase(
			"summary-with-no-parent-after-a-boundary-with-no-id",
			rows([
				'{"type":"system","subtype":"compact_boundary","parentUuid":null}',
				'{"type":"user","isCompactSummary":true}',
			]),
			"built-1",
			false,
		),
	];
}

function pickCases(): CaseSource[] {
	const session = new SyntheticSession();
	session.header();
	const picks: Array<TurnPick & { bodyOf: BodyOf }> = [];
	const at = (second: number) => START_MS + second * 1000;
	session.user("go");
	session.says("went");
	session.user("[from the phone]\nship   it");
	session.calls("t1");
	session.user("typed while it worked");
	session.answers("t1");
	session.absorbs("[from the phone]\nkeep going");
	session.says("shipped");
	session.user("go");
	session.says("went again");
	session.raw('{"type":"user","uuid":"odd-time","timestamp":"Nov 13 2026","message":{"content":"timeless"}}');
	session.raw('{"type":"user","uuid":"no-time","message":{"content":"timeless"}}');
	session.raw(
		'{"type":"user","uuid":"offset-time","timestamp":"2026-11-13T01:00:20+01:00","message":{"content":"\u{a0}spaced\u{2028}\u{3000}out\u{feff}\t"}}',
	);
	session.raw('{"type":"user","uuid":"tie-a","timestamp":"2026-11-13T00:00:30.000Z","message":{"content":"tie"}}');
	session.raw('{"type":"user","uuid":"tie-b","timestamp":"2026-11-13T00:00:32.000Z","message":{"content":"tie"}}');

	picks.push(
		{ at: at(1), text: "go", bodyOf: "afterFirstLine" },
		{ at: at(9), text: "go", bodyOf: "whole" },
		{ at: at(3), text: "ship it", bodyOf: "afterFirstLine" },
		{ at: at(3), text: "ship it", bodyOf: "whole" },
		{ at: at(7), text: "keep going", bodyOf: "afterFirstLine" },
		{ at: at(4), text: "typed while it worked", bodyOf: "whole" },
		{ at: at(8), bodyOf: "whole" },
		{ at: at(0) - 60_000, bodyOf: "whole" },
		{ at: at(0), bodyOf: "whole" },
		{ at: at(0), text: "missing text", bodyOf: "whole" },
		{ at: at(1) - 2000, text: "missing text", bodyOf: "whole" },
		{ at: at(1) - 2001, text: "missing text", bodyOf: "whole" },
		{ at: at(5), text: "  ", bodyOf: "whole" },
		{ at: at(5), text: "", bodyOf: "whole" },
		{ at: at(100), text: "timeless", bodyOf: "whole" },
		{ at: at(100), text: "spaced out", bodyOf: "whole" },
		{ at: at(31), text: "tie", bodyOf: "whole" },
		{ at: at(100), bodyOf: "whole" },
	);
	return [{ kind: "sessionCut.pickTurn", name: "turns", input: { source: rows(session.lines), picks } }];
}

export const sessionCutCases: CaseSource[] = [
	...cutCases(),
	...planCases(),
	...indexCases(),
	...rehomedCases(),
	...verifyCases(),
	...pickCases(),
];
