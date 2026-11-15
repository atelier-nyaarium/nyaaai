// Millisecond and ISO times.

////////////////////////////////
//  Constants

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

/** Submillisecond fraction digits drop. */
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

/** JavaScript Date limit. */
const LAST_MS = 8_640_000_000_000_000;

/** Valid Date milliseconds only. */
export function isoTime(ms: number): string | undefined {
	return Number.isInteger(ms) && Math.abs(ms) <= LAST_MS ? new Date(ms).toISOString() : undefined;
}
