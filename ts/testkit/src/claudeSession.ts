// Synthetic session rows.

////////////////////////////////
//  Constants

export const SESSION_ID = "11111111-1111-4111-8111-111111111111";
export const CLI_VERSION = "2.1.323";
/** Fixture timestamp. */
const START_MS = 1_794_528_000_000;

////////////////////////////////
//  Functions & Helpers

export class SyntheticSession {
	readonly lines: string[] = [];
	private last: string | null = null;
	private count = 0;

	/** Generates a chained UUID. */
	chain(row: Record<string, unknown>, id?: string): string {
		const uuid = id ?? `u${++this.count}`;
		this.lines.push(
			JSON.stringify({
				parentUuid: this.last,
				isSidechain: false,
				...row,
				uuid,
				timestamp: new Date(START_MS + this.count * 1000).toISOString(),
				userType: "external",
				entrypoint: "cli",
				cwd: "/work",
				sessionId: SESSION_ID,
				version: CLI_VERSION,
				gitBranch: "main",
				slug: "a-slug",
			}),
		);
		this.last = uuid;
		return uuid;
	}

	raw(line: string): void {
		this.lines.push(line);
	}

	header(): void {
		for (const type of ["mode", "permission-mode", "session-owner", "atis-latch"])
			this.lines.push(JSON.stringify({ type, sessionId: SESSION_ID }));
	}

	user(text: string): string {
		return this.chain({ type: "user", message: { role: "user", content: text } });
	}

	says(text: string): string {
		return this.chain({ type: "assistant", message: { role: "assistant", content: [{ type: "text", text }] } });
	}

	calls(id: string): string {
		return this.chain({
			type: "assistant",
			message: {
				role: "assistant",
				content: [{ type: "tool_use", id, name: "Bash", input: { session_id: SESSION_ID } }],
			},
		});
	}

	/** In-turn queued message. */
	absorbs(prompt: string): string {
		return this.chain({ type: "attachment", attachment: { type: "queued_command", prompt } });
	}

	answers(id: string): string {
		return this.chain({
			type: "user",
			message: {
				role: "user",
				content: [{ type: "tool_result", tool_use_id: id, content: `ran in ${SESSION_ID}` }],
			},
		});
	}

	requestLog(id: string): void {
		this.lines.push(
			JSON.stringify({
				type: "api-request-blob",
				sessionId: SESSION_ID,
				hash: "h",
				message: { role: "assistant", content: [{ type: "tool_use", id, name: "Bash", input: {} }] },
			}),
		);
	}

	lastPrompt(leaf: string): void {
		this.lines.push(JSON.stringify({ type: "last-prompt", leafUuid: leaf, sessionId: SESSION_ID }));
	}

	/** CLI-style compaction. */
	compacts(summary: string, preserved: string[], dropped = 1000): void {
		const boundary = `b${++this.count}`;
		const anchor = `s${this.count}`;
		this.lines.push(
			JSON.stringify({
				parentUuid: null,
				logicalParentUuid: this.last,
				isSidechain: false,
				type: "system",
				subtype: "compact_boundary",
				content: "Conversation compacted",
				level: "info",
				compactMetadata: {
					trigger: "manual",
					preTokens: 5000,
					postTokens: 400,
					cumulativeDroppedTokens: dropped,
					durationMs: 9,
					preservedSegment: { headUuid: preserved[0], anchorUuid: anchor, tailUuid: preserved.at(-1) },
					preservedMessages: { anchorUuid: anchor, uuids: preserved, allUuids: [...preserved, "gone"] },
				},
				uuid: boundary,
				timestamp: "2026-11-13T00:01:00.000Z",
				userType: "external",
				entrypoint: "cli",
				cwd: "/work",
				sessionId: SESSION_ID,
				version: CLI_VERSION,
				gitBranch: "main",
				slug: "a-slug",
			}),
		);
		this.last = boundary;
		this.chain(
			{
				type: "user",
				message: { role: "user", content: summary },
				isCompactSummary: true,
				isVisibleInTranscriptOnly: true,
			},
			anchor,
		);
	}
}

/** Compacted session fixture. */
export function compactedSession() {
	const session = new SyntheticSession();
	session.header();
	session.user("ancient");
	session.says("ancient answer");
	const keptA = session.user("kept before the compaction");
	const keptB = session.says("kept answer");
	session.compacts("what stood", [keptA, keptB]);
	session.user("first turn");
	session.calls("t1");
	session.requestLog("ghost");
	session.answers("t1");
	session.says("done");
	const second = session.user("second turn");
	session.lastPrompt(second);
	session.calls("t2");
	session.answers("t2");
	const midTurn = session.user("typed while it worked");
	session.says("both done");
	const third = session.user("third turn");
	session.header();
	const end = session.says("finished");
	session.lastPrompt(end);
	return { session, second, third, midTurn, keptA };
}
