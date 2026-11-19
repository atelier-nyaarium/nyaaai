// A day file's lines: building, writing, reading, and the day a line belongs to.

import { stringifyJson } from "@nyaaai/contract/json";
import { type Decoded, decode, decodeText } from "@nyaaai/contract/shape";
import {
	CallRecord,
	GenerationStats,
	type MeteredCall,
	type StatsLine,
	UsageLine,
} from "@nyaaai/contract/shapes/usage";
import { asciiLower, trimText } from "@nyaaai/contract/text";
import { type Clock, dayKey } from "@nyaaai/contract/time";

////////////////////////////////
//  Interfaces & Types

/** What a day file's text held. */
export interface LinesRead {
	lines: UsageLine[];
	/** Lines with text that did not parse or fit. */
	dropped: number;
}

////////////////////////////////
//  Functions & Helpers

/** Wire fields are cast, not checked, so a null or NaN may arrive. */
function present<T extends Record<string, unknown>>(fields: T): Partial<T> {
	return Object.fromEntries(
		Object.entries(fields).filter(
			([, value]) => value !== undefined && value !== null && !(typeof value === "number" && Number.isNaN(value)),
		),
	) as Partial<T>;
}

function withoutSeparators(text: string): string {
	let end = text.length;
	while (end > 0 && (text[end - 1] === "/" || text[end - 1] === "\\")) end -= 1;
	return text.slice(0, end);
}

/** The local calendar day of `atMs` in the clock's zone; none outside years 0000 to 9999. */
export function dayOf(atMs: number, clock: Pick<Clock, "utcOffsetMinutes">): string | undefined {
	return dayKey(atMs, clock.utcOffsetMinutes(atMs));
}

interface Segment {
	/** The separator before it; none for the first. */
	sep: string;
	seg: string;
}

function segmentsOf(text: string): Segment[] {
	const out: Segment[] = [];
	let sep = "";
	let start = 0;
	for (let at = 0; at <= text.length; at += 1) {
		const unit = text[at];
		if (unit !== undefined && unit !== "/" && unit !== "\\") continue;
		out.push({ sep, seg: text.slice(start, at) });
		sep = unit ?? "";
		start = at + 1;
	}
	return out;
}

/** A letter and a colon first. */
function hasDrive(text: string): boolean {
	const code = text.charCodeAt(0) | 0x20;
	return code >= 0x61 && code <= 0x7a && text[1] === ":";
}

/** `.` and `..` segments resolved, never past the root or the drive. */
function resolved(text: string): string {
	const parts = segmentsOf(text);
	if (!parts.some((part) => part.seg === "." || part.seg === "..")) return text;
	const first = parts[0]?.seg ?? "";
	const kept = first === "" || (first.length === 2 && hasDrive(first)) ? 1 : 0;
	const out: Segment[] = [];
	for (const part of parts) {
		if (part.seg === ".") continue;
		if (part.seg === "..") {
			if (out.length > kept) out.pop();
			continue;
		}
		out.push(out.length === 0 ? { sep: "", seg: part.seg } : part);
	}
	return out.map((part) => `${part.sep}${part.seg}`).join("");
}

const plainPath = (text: string): string => withoutSeparators(resolved(withoutSeparators(text)));

/** A folder path with home as `~` and a closing separator; none for a blank folder. */
export function appOf(folder: string, home: string): string | undefined {
	if (trimText(folder) === "") return undefined;
	const separator = folder.includes("\\") && !folder.includes("/") ? "\\" : "/";
	const trimmed = plainPath(folder);
	const homeTrimmed = plainPath(home);
	const fold = hasDrive(trimmed) && hasDrive(homeTrimmed) ? asciiLower : (text: string) => text;
	const held = fold(trimmed);
	const heldHome = fold(homeTrimmed);
	const shown =
		homeTrimmed !== "" && (held === heldHome || held.startsWith(`${heldHome}${separator}`))
			? `~${trimmed.slice(homeTrimmed.length)}`
			: trimmed;
	return `${shown}${separator}`;
}

/** A call as its day file keeps it; a refusal when the shape does not fit. */
export function callRecordOf(call: MeteredCall, id: string, home: string): Decoded<CallRecord> {
	const { usage, cost, tag } = call;
	const priced = Number.isFinite(cost?.usd);
	return decode(
		CallRecord,
		present({
			id,
			at: call.at,
			feature: tag?.feature,
			lane: tag?.lane,
			model: call.model,
			entry: call.entry,
			label: call.label,
			app: typeof tag?.folder === "string" ? appOf(tag.folder, home) : undefined,
			ms: call.ms,
			first: call.firstTokenMs,
			in: usage?.input,
			out: usage?.output,
			cached: usage?.cached,
			cacheWrite: usage?.cacheWrite,
			reasoning: usage?.reasoning,
			cost: cost === undefined || cost === null ? undefined : priced ? cost.usd : 0,
			accountCost: usage?.cost,
			upstreamCost: usage?.upstreamCost,
			byok: usage?.byok,
			billingSource: usage?.billingSource,
			estimated: cost === undefined || cost === null ? undefined : cost.estimated || !priced ? true : undefined,
			unpriced: cost?.unpriced,
			provider: call.provider,
			gen: call.generationId,
			finish: call.finishReason,
			failure: call.failure,
			cancelled: call.cancelled,
		}),
	);
}

/** A generation's stats as its day file keeps them; a refusal when the shape does not fit. */
export function statsLineOf(id: string, stats: GenerationStats): Decoded<StatsLine> {
	const of = decode(GenerationStats, present({ ...stats }));
	return of.ok ? { ok: true, value: { stats: { id, of: of.value } } } : of;
}

/** The line as written to a day file, with its line feed. */
export function lineText(line: UsageLine): string {
	return `${stringifyJson(line)}\n`;
}

/** A day file's lines in order; text that does not parse or fit is dropped, blank lines are not counted. */
export function linesOf(text: string): LinesRead {
	const lines: UsageLine[] = [];
	let dropped = 0;
	for (const raw of text.split("\n")) {
		if (trimText(raw) === "") continue;
		const read = decodeText(UsageLine, raw);
		if (read.ok) lines.push(read.value);
		else dropped += 1;
	}
	return { lines, dropped };
}
