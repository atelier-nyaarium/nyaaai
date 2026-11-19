import { appOf, callRecordOf, dayOf, linesOf, lineText, statsLineOf } from "@nyaaai/agentic/usage/record";
import { z } from "@nyaaai/contract/shape";
import { CallRecord, StatsLine, UsageLine } from "@nyaaai/contract/shapes/usage";
import { steppedClock } from "@nyaaai/contract/time";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { usageCases } from "../caseSources/usage";

const Offsets = z.array(z.object({ fromMs: z.int(), minutes: z.int() }));

/** Values a text cannot carry, put where a path says before the call is used. */
const NonFinite = z.array(z.object({ path: z.string(), is: z.enum(["notANumber", "plusInfinity", "minusInfinity"]) }));

const FLOATS = {
	notANumber: Number.NaN,
	plusInfinity: Number.POSITIVE_INFINITY,
	minusInfinity: Number.NEGATIVE_INFINITY,
};

/** A copy of `value` with each path set to its non-finite number; a dot goes one level in. */
function withNonFinite(value: unknown, set: z.output<typeof NonFinite>): unknown {
	const held = structuredClone(value) as Record<string, unknown>;
	for (const { path, is } of set) {
		const keys = path.split(".");
		let at: Record<string, unknown> = held;
		for (const key of keys.slice(0, -1)) at = at[key] as Record<string, unknown>;
		at[keys[keys.length - 1] as string] = FLOATS[is];
	}
	return held;
}

const pathText = (path: ReadonlyArray<string | number>): string => path.join(".");

export const usage = caseGroup("usage", {
	kinds: {
		/**
		 * `call` is a metered call as the meter hands it, which may hold nulls and, through `nonFinite`, numbers a
		 * text cannot carry. `refusedAt` is the path of the first field the shape refuses, joined by dots; the
		 * record, its line and the day are then null. `day` is the day `nowMs` falls on under `offsets`.
		 */
		record: kind(
			z.object({
				call: z.unknown(),
				nonFinite: NonFinite,
				id: z.string(),
				home: z.string(),
				nowMs: z.int(),
				offsets: Offsets,
			}),
			z.object({
				record: CallRecord.nullable(),
				refusedAt: z.string().nullable(),
				line: z.string().nullable(),
				day: z.string().nullable(),
			}),
			(input) => {
				const built = callRecordOf(
					withNonFinite(input.call, input.nonFinite) as Parameters<typeof callRecordOf>[0],
					input.id,
					input.home,
				);
				const day = dayOf(input.nowMs, steppedClock(0, 0, input.offsets)) ?? null;
				if (!built.ok) return { record: null, refusedAt: pathText(built.path), line: null, day };
				return { record: built.value, refusedAt: null, line: lineText({ call: built.value }), day };
			},
		),
		/** The same for a generation's stats. `stats` may hold nulls. */
		stats: kind(
			z.object({ stats: z.unknown(), nonFinite: NonFinite, id: z.string() }),
			z.object({
				line: StatsLine.nullable(),
				refusedAt: z.string().nullable(),
				text: z.string().nullable(),
			}),
			(input) => {
				const built = statsLineOf(
					input.id,
					withNonFinite(input.stats, input.nonFinite) as Parameters<typeof statsLineOf>[1],
				);
				if (!built.ok) return { line: null, refusedAt: pathText(built.path), text: null };
				return { line: built.value, refusedAt: null, text: lineText(built.value) };
			},
		),
		/** A day file's text read: what fits, and how many lines with text were dropped. */
		lines: kind(
			z.object({ text: z.string() }),
			z.object({ lines: z.array(UsageLine), dropped: z.int() }),
			(input) => linesOf(input.text),
		),
		/** A folder shown with home as `~`. */
		app: kind(
			z.object({ rows: z.array(z.object({ folder: z.string(), home: z.string() })) }),
			z.object({ apps: z.array(z.string()) }),
			(input) => ({ apps: input.rows.map((row) => appOf(row.folder, row.home)) }),
		),
		/** The local day of each instant under the zone `offsets` script; null outside years 0000 to 9999. */
		day: kind(
			z.object({ offsets: Offsets, instants: z.array(z.int()) }),
			z.object({ days: z.array(z.string().nullable()) }),
			(input) => {
				const clock = steppedClock(0, 0, input.offsets);
				return { days: input.instants.map((at) => dayOf(at, clock) ?? null) };
			},
		),
	},
	sources: casesFor("usage", usageCases),
});
