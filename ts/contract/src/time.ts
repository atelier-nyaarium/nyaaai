// Time primitives.

import { numberOfText } from "./number";
import { trimText } from "./text";

////////////////////////////////
//  Interfaces & Types

/** Time in milliseconds. */
export interface Clock {
	nowMs: () => number;
}

/** Delays are whole milliseconds, at most 2^53-1. */
export interface Scheduler extends Clock {
	/** Abort rejects with reason. */
	sleep: (ms: number, signal?: AbortSignal) => Promise<void>;
	/** Runs once after `ms`; the result cancels it. */
	schedule: (ms: number, run: () => void) => () => void;
}

////////////////////////////////
//  Constants

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const SHORT_DAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
const LONG_DAYS = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
/** Below this: seconds. */
const SECONDS_BELOW = 1e11;
const DAY_MS = 86_400_000;

////////////////////////////////
//  Functions & Helpers

const leap = (year: number) => (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0;

const daysIn = (year: number, month: number) =>
	month === 2 ? (leap(year) ? 29 : 28) : [4, 6, 9, 11].includes(month) ? 30 : 31;

/** Gregorian days since epoch. */
function daysFromCivil(year: number, month: number, day: number): number {
	const shifted = month <= 2 ? year - 1 : year;
	const era = Math.floor(shifted / 400);
	const yearOfEra = shifted - era * 400;
	const dayOfYear = Math.floor((153 * (month + (month > 2 ? -3 : 9)) + 2) / 5) + day - 1;
	const dayOfEra = yearOfEra * 365 + Math.floor(yearOfEra / 4) - Math.floor(yearOfEra / 100) + dayOfYear;
	return era * 146_097 + dayOfEra - 719_468;
}

interface Cursor {
	text: string;
	at: number;
}

/** Exactly `count` ASCII digits. */
function digits(cursor: Cursor, count: number): number | undefined {
	let number = 0;
	for (let place = cursor.at; place < cursor.at + count; place++) {
		const code = cursor.text.charCodeAt(place);
		if (!(code >= 0x30 && code <= 0x39)) return undefined;
		number = number * 10 + (code - 0x30);
	}
	cursor.at += count;
	return number;
}

const isDigitAt = (text: string, at: number) => text.charCodeAt(at) >= 0x30 && text.charCodeAt(at) <= 0x39;

function literal(cursor: Cursor, wanted: string): boolean {
	if (!cursor.text.startsWith(wanted, cursor.at)) return false;
	cursor.at += wanted.length;
	return true;
}

/** One of `names`, case-sensitive. */
function name(cursor: Cursor, names: string[]): string | undefined {
	const found = names.find((held) => cursor.text.startsWith(held, cursor.at));
	if (found !== undefined) cursor.at += found.length;
	return found;
}

/** `hh:mm:ss`. */
function clock(cursor: Cursor): [number, number, number] | undefined {
	const hour = digits(cursor, 2);
	if (hour === undefined || !literal(cursor, ":")) return undefined;
	const minute = digits(cursor, 2);
	if (minute === undefined || !literal(cursor, ":")) return undefined;
	const second = digits(cursor, 2);
	return second === undefined ? undefined : [hour, minute, second];
}

/** Drops submillisecond precision. */
export function parseIsoTime(text: string): number {
	const cursor: Cursor = { text, at: 0 };
	const year = digits(cursor, 4);
	if (year === undefined || !literal(cursor, "-")) return Number.NaN;
	const month = digits(cursor, 2);
	if (month === undefined || !literal(cursor, "-")) return Number.NaN;
	const day = digits(cursor, 2);
	if (day === undefined || !literal(cursor, "T")) return Number.NaN;
	const time = clock(cursor);
	if (time === undefined) return Number.NaN;
	const [hour, minute, second] = time;

	let fraction = "";
	if (literal(cursor, ".")) {
		const from = cursor.at;
		while (isDigitAt(text, cursor.at)) cursor.at += 1;
		fraction = text.slice(from, cursor.at);
		if (fraction.length < 1 || fraction.length > 9) return Number.NaN;
	}

	let offsetHour = 0;
	let offsetMinute = 0;
	let sign = 1;
	if (!literal(cursor, "Z")) {
		if (literal(cursor, "-")) sign = -1;
		else if (!literal(cursor, "+")) return Number.NaN;
		const heldHour = digits(cursor, 2);
		if (heldHour === undefined || !literal(cursor, ":")) return Number.NaN;
		const heldMinute = digits(cursor, 2);
		if (heldMinute === undefined) return Number.NaN;
		offsetHour = heldHour;
		offsetMinute = heldMinute;
		if (offsetHour > 23 || offsetMinute > 59) return Number.NaN;
	}
	if (cursor.at !== text.length) return Number.NaN;
	if (month < 1 || month > 12 || day < 1 || day > daysIn(year, month)) return Number.NaN;
	if (hour > 23 || minute > 59 || second > 59) return Number.NaN;
	const millis = Number(fraction.padEnd(3, "0").slice(0, 3));
	const offset = sign * (offsetHour * 60 + offsetMinute) * 60_000;
	return daysFromCivil(year, month, day) * DAY_MS + ((hour * 60 + minute) * 60 + second) * 1000 + millis - offset;
}

/** ECMAScript Date limit. */
const LAST_MS = 8_640_000_000_000_000;

/** Valid Date milliseconds only. */
export function isoTime(ms: number): string | undefined {
	return Number.isInteger(ms) && Math.abs(ms) <= LAST_MS ? new Date(ms).toISOString() : undefined;
}

function civilMs(year: number, month: number, day: number, time: [number, number, number]) {
	const [hour, minute, second] = time;
	if (month < 1 || month > 12 || day < 1 || day > daysIn(year, month)) return undefined;
	if (hour > 23 || minute > 59 || second > 59) return undefined;
	return daysFromCivil(year, month, day) * DAY_MS + ((hour * 60 + minute) * 60 + second) * 1000;
}

const monthOf = (held: string) => MONTHS.indexOf(held) + 1;

function imfDate(text: string): number | undefined {
	const cursor: Cursor = { text, at: 0 };
	if (name(cursor, SHORT_DAYS) === undefined || !literal(cursor, ", ")) return undefined;
	const day = digits(cursor, 2);
	if (day === undefined || !literal(cursor, " ")) return undefined;
	const month = name(cursor, MONTHS);
	if (month === undefined || !literal(cursor, " ")) return undefined;
	const year = digits(cursor, 4);
	if (year === undefined || !literal(cursor, " ")) return undefined;
	const time = clock(cursor);
	if (time === undefined || !literal(cursor, " GMT") || cursor.at !== text.length) return undefined;
	return civilMs(year, monthOf(month), day, time);
}

function rfc850Date(text: string, nowMs: number): number | undefined {
	const cursor: Cursor = { text, at: 0 };
	if (name(cursor, LONG_DAYS) === undefined || !literal(cursor, ", ")) return undefined;
	const day = digits(cursor, 2);
	if (day === undefined || !literal(cursor, "-")) return undefined;
	const month = name(cursor, MONTHS);
	if (month === undefined || !literal(cursor, "-")) return undefined;
	const short = digits(cursor, 2);
	if (short === undefined || !literal(cursor, " ")) return undefined;
	const time = clock(cursor);
	if (time === undefined || !literal(cursor, " GMT") || cursor.at !== text.length) return undefined;
	const nowYear = Number((dayKey(nowMs) ?? "1970").slice(0, 4));
	const near = Math.floor(nowYear / 100) * 100 + short;
	const year = near - nowYear > 50 ? near - 100 : nowYear - near >= 50 ? near + 100 : near;
	return civilMs(year, monthOf(month), day, time);
}

function asctimeDate(text: string): number | undefined {
	const cursor: Cursor = { text, at: 0 };
	if (name(cursor, SHORT_DAYS) === undefined || !literal(cursor, " ")) return undefined;
	const month = name(cursor, MONTHS);
	if (month === undefined || !literal(cursor, " ")) return undefined;
	const day = literal(cursor, " ") ? digits(cursor, 1) : digits(cursor, 2);
	if (day === undefined || !literal(cursor, " ")) return undefined;
	const time = clock(cursor);
	if (time === undefined || !literal(cursor, " ")) return undefined;
	const year = digits(cursor, 4);
	if (year === undefined || cursor.at !== text.length) return undefined;
	return civilMs(year, monthOf(month), day, time);
}

/** RFC 850 uses a 50-year pivot. */
export function parseHttpDate(text: string, nowMs: number): number | undefined {
	return imfDate(text) ?? rfc850Date(text, nowMs) ?? asctimeDate(text);
}

function parseStamp(text: string, nowMs: number): number | undefined {
	const iso = parseIsoTime(text);
	return Number.isNaN(iso) ? parseHttpDate(text, nowMs) : iso;
}

const wholeMs = (ms: number): number | undefined =>
	Number.isFinite(ms) && Math.abs(ms) <= Number.MAX_SAFE_INTEGER ? Math.round(ms) + 0 : undefined;

/** Nonnegative wait in milliseconds. */
export function parseRetryAfter(header: string | null | undefined, nowMs: number): number | undefined {
	const text = trimText(header ?? "");
	if (text.length === 0) return undefined;
	const seconds = numberOfText(text);
	const wait = seconds === undefined ? parseStamp(text, nowMs) : wholeMs(seconds * 1000);
	if (wait === undefined) return undefined;
	return wholeMs(Math.max(0, seconds === undefined ? wait - nowMs : wait));
}

/** Seconds, milliseconds, or dates. */
export function resetStampMs(value: unknown, nowMs: number): number | undefined {
	if (typeof value === "number") return wholeMs(Math.abs(value) < SECONDS_BELOW ? value * 1000 : value);
	return typeof value === "string" ? parseStamp(trimText(value), nowMs) : undefined;
}

/** UTC-offset day; years 0000-9999. */
export function dayKey(ms: number, offsetMinutes = 0): string | undefined {
	const text = isoTime(ms + offsetMinutes * 60_000);
	return text?.length === 24 ? text.slice(0, 10) : undefined;
}

export const systemClock: Clock = { nowMs: () => Date.now() };

/** Longest single platform timer. */
const MAX_TIMER_MS = 2 ** 31 - 1;

/** Chains timers past the platform's limit; the result cancels. */
function afterMs(ms: number, run: () => void): () => void {
	let left = Number.isFinite(ms) && ms > 0 ? ms : 0;
	let timer: ReturnType<typeof setTimeout> | undefined;
	const step = () => {
		const wait = Math.min(left, MAX_TIMER_MS);
		left -= wait;
		timer = setTimeout(left > 0 ? step : run, wait);
	};
	step();
	return () => clearTimeout(timer);
}

export const systemScheduler: Scheduler = {
	...systemClock,
	sleep: (ms, signal) =>
		new Promise((resolve, reject) => {
			if (signal?.aborted) return reject(signal.reason);
			const stop = () => {
				cancel();
				reject(signal?.reason);
			};
			const cancel = afterMs(ms, () => {
				signal?.removeEventListener("abort", stop);
				resolve();
			});
			signal?.addEventListener("abort", stop, { once: true });
		}),
	schedule: afterMs,
};

/** Advances on each read. */
export function steppedClock(startMs: number, stepMs = 0): Clock {
	let reads = 0;
	return { nowMs: () => startMs + stepMs * reads++ };
}
