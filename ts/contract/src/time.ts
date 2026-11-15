// Millisecond and ISO times.

import { numberOfText } from "./number";

////////////////////////////////
//  Interfaces & Types

/** Time in milliseconds. */
export interface Clock {
	nowMs: () => number;
}

export interface Scheduler extends Clock {
	/** Abort rejects with reason. */
	sleep: (ms: number, signal?: AbortSignal) => Promise<void>;
}

////////////////////////////////
//  Constants

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
const MONTH = `(${MONTHS.join("|")})`;
const SHORT_DAY = "(?:Mon|Tue|Wed|Thu|Fri|Sat|Sun)";
const LONG_DAY = "(?:Monday|Tuesday|Wednesday|Thursday|Friday|Saturday|Sunday)";
const CLOCK = "(\\d{2}):(\\d{2}):(\\d{2})";
/** IMF-fixdate format. */
const IMF_DATE = new RegExp(`^${SHORT_DAY}, (\\d{2}) ${MONTH} (\\d{4}) ${CLOCK} GMT$`);
/** RFC 850 format. */
const RFC850_DATE = new RegExp(`^${LONG_DAY}, (\\d{2})-${MONTH}-(\\d{2}) ${CLOCK} GMT$`);
/** Asctime format. */
const ASCTIME_DATE = new RegExp(`^${SHORT_DAY} ${MONTH} ( \\d|\\d{2}) ${CLOCK} (\\d{4})$`);
/** Below this: seconds. */
const SECONDS_BELOW = 1e11;
const ISO = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.(\d{1,9}))?(?:Z|([+-])(\d{2}):(\d{2}))$/;
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

/** Drops submillisecond precision. */
export function parseIsoTime(text: string): number {
	const parts = ISO.exec(text);
	if (!parts) return Number.NaN;
	const [year, month, day, hour, minute, second] = parts.slice(1, 7).map(Number) as [
		number,
		number,
		number,
		number,
		number,
		number,
	];
	const offsetHour = Number(parts[9] ?? 0);
	const offsetMinute = Number(parts[10] ?? 0);
	if (month < 1 || month > 12 || day < 1 || day > daysIn(year, month)) return Number.NaN;
	if (hour > 23 || minute > 59 || second > 59 || offsetHour > 23 || offsetMinute > 59) return Number.NaN;
	const millis = Number((parts[7] ?? "").padEnd(3, "0").slice(0, 3));
	const offset = (parts[8] === "-" ? -1 : 1) * (offsetHour * 60 + offsetMinute) * 60_000;
	return daysFromCivil(year, month, day) * DAY_MS + ((hour * 60 + minute) * 60 + second) * 1000 + millis - offset;
}

/** ECMAScript Date limit. */
const LAST_MS = 8_640_000_000_000_000;

/** Formats valid Date milliseconds. */
export function isoTime(ms: number): string | undefined {
	return Number.isInteger(ms) && Math.abs(ms) <= LAST_MS ? new Date(ms).toISOString() : undefined;
}

function civilMs(year: number, month: number, day: number, hour: number, minute: number, second: number) {
	if (month < 1 || month > 12 || day < 1 || day > daysIn(year, month)) return undefined;
	if (hour > 23 || minute > 59 || second > 59) return undefined;
	return daysFromCivil(year, month, day) * DAY_MS + ((hour * 60 + minute) * 60 + second) * 1000;
}

/** Exact-case HTTP dates; RFC 850 uses a 50-year pivot. */
export function parseHttpDate(text: string, nowMs: number): number | undefined {
	const month = (name: string | undefined) => MONTHS.indexOf(name as string) + 1;
	const imf = IMF_DATE.exec(text);
	if (imf) {
		const [day, name, year, hour, minute, second] = imf.slice(1);
		return civilMs(Number(year), month(name), Number(day), Number(hour), Number(minute), Number(second));
	}
	const old = RFC850_DATE.exec(text);
	if (old) {
		const [day, name, short, hour, minute, second] = old.slice(1);
		const nowYear = Number((dayKey(nowMs) ?? "1970").slice(0, 4));
		const near = Math.floor(nowYear / 100) * 100 + Number(short);
		const year = near - nowYear > 50 ? near - 100 : nowYear - near >= 50 ? near + 100 : near;
		return civilMs(year, month(name), Number(day), Number(hour), Number(minute), Number(second));
	}
	const asc = ASCTIME_DATE.exec(text);
	if (asc) {
		const [name, day, hour, minute, second, year] = asc.slice(1);
		return civilMs(Number(year), month(name), Number(day), Number(hour), Number(minute), Number(second));
	}
	return undefined;
}

function parseStamp(text: string, nowMs: number): number | undefined {
	const iso = parseIsoTime(text);
	return Number.isNaN(iso) ? parseHttpDate(text, nowMs) : iso;
}

const wholeMs = (ms: number): number | undefined =>
	Number.isFinite(ms) && Math.abs(ms) <= Number.MAX_SAFE_INTEGER ? Math.round(ms) + 0 : undefined;

/** Nonnegative wait in milliseconds. */
export function parseRetryAfter(header: string | null | undefined, nowMs: number): number | undefined {
	const text = (header ?? "").trim();
	if (text.length === 0) return undefined;
	const seconds = numberOfText(text);
	const wait = seconds === undefined ? parseStamp(text, nowMs) : wholeMs(seconds * 1000);
	if (wait === undefined) return undefined;
	return wholeMs(Math.max(0, seconds === undefined ? wait - nowMs : wait));
}

/** Seconds, milliseconds, or dates. */
export function resetStampMs(value: unknown, nowMs: number): number | undefined {
	if (typeof value === "number") return wholeMs(Math.abs(value) < SECONDS_BELOW ? value * 1000 : value);
	return typeof value === "string" ? parseStamp(value.trim(), nowMs) : undefined;
}

/** UTC-offset day; years 0000-9999. */
export function dayKey(ms: number, offsetMinutes = 0): string | undefined {
	const text = isoTime(ms + offsetMinutes * 60_000);
	return text?.length === 24 ? text.slice(0, 10) : undefined;
}

export const systemClock: Clock = { nowMs: () => Date.now() };

export const systemScheduler: Scheduler = {
	...systemClock,
	sleep: (ms, signal) =>
		new Promise((resolve, reject) => {
			if (signal?.aborted) return reject(signal.reason);
			const stop = () => {
				clearTimeout(timer);
				reject(signal?.reason);
			};
			const timer = setTimeout(() => {
				signal?.removeEventListener("abort", stop);
				resolve();
			}, ms);
			signal?.addEventListener("abort", stop, { once: true });
		}),
};

/** Advances on each read. */
export function steppedClock(startMs: number, stepMs = 0): Clock {
	let reads = 0;
	return { nowMs: () => startMs + stepMs * reads++ };
}
