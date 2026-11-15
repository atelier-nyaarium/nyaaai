import { countedIds } from "@nyaaai/contract/id";
import { z } from "@nyaaai/contract/shape";
import { CutDone, CutPlan, CutRefused, CutRequest, PickedTurn, SessionIndex } from "@nyaaai/contract/shapes/sessionCut";
import { steppedClock } from "@nyaaai/contract/time";
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
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { sessionCutCases } from "../caseSources/sessionCut";

////////////////////////////////
//  Shapes

export const SessionSource = z.object({
	lines: z.array(z.string()).exactOptional(),
	base64: z.string().exactOptional(),
	file: z.string().exactOptional(),
});
export type SessionSource = z.infer<typeof SessionSource>;

/** Turn body cut mode. */
export const BodyOf = z.enum(["whole", "afterFirstLine"]);
export type BodyOf = z.infer<typeof BodyOf>;

export const SessionCutCutInput = z.object({
	source: SessionSource,
	request: CutRequest,
	cliVersion: z.string(),
	sourcePath: z.string(),
	/** Ids append counts. */
	idPrefix: z.string(),
	nowMs: z.int(),
	/** Absent disables summarization. */
	summary: z.string().exactOptional(),
});
export type SessionCutCutInput = z.infer<typeof SessionCutCutInput>;

////////////////////////////////
//  Functions & Helpers

export function rowsOf(source: SessionSource): string[] {
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

async function cut(input: SessionCutCutInput) {
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

////////////////////////////////
//  Group

export const sessionCut = caseGroup("sessionCut", {
	shapes: { SessionSource, BodyOf },
	kinds: {
		rehomed: kind(
			z.object({ rows: z.array(z.string()), sessionId: z.string() }),
			z.object({ rows: z.array(z.string()) }),
			(input) => ({ rows: input.rows.map((row) => rehomed(row, input.sessionId)) }),
		),
		index: kind(z.object({ source: SessionSource }), z.object({ index: SessionIndex }), async (input) => ({
			index: await indexSession(streamOf(input.source)),
		})),
		plan: kind(
			z.object({ source: SessionSource, requests: z.array(CutRequest), cliVersion: z.string() }),
			z.object({ plans: z.array(z.union([CutPlan, CutRefused])) }),
			async (input) => {
				const index = await indexSession(streamOf(input.source));
				return { plans: input.requests.map((request) => planCut(index, request, input.cliVersion)) };
			},
		),
		cut: kind(
			SessionCutCutInput,
			z.object({
				error: z.string().exactOptional(),
				result: z.union([CutDone, CutRefused]).exactOptional(),
				built: z.array(z.string()).nullable().exactOptional(),
				/** Summarizer input. */
				summarized: z
					.object({ standing: z.string().nullable(), rows: z.array(z.string()) })
					.nullable()
					.exactOptional(),
			}),
			cut,
		),
		verify: kind(
			z.object({ source: SessionSource, sessionId: z.string(), paired: z.boolean() }),
			z.object({ problems: z.array(z.string()) }),
			async (input) => ({
				problems: await verifyCut(each(rowsOf(input.source)), input.sessionId, input.paired),
			}),
		),
		pickTurn: kind(
			z.object({
				source: SessionSource,
				picks: z.array(z.object({ at: z.int(), text: z.string().exactOptional(), bodyOf: BodyOf })),
			}),
			z.object({ picked: z.array(PickedTurn.nullable()) }),
			async (input) => {
				const index = await indexSession(each(rowsOf(input.source)));
				return { picked: input.picks.map(({ bodyOf, ...pick }) => pickTurn(index, pick, BODIES[bodyOf])) };
			},
		),
	},
	sources: casesFor("sessionCut", sessionCutCases),
});
