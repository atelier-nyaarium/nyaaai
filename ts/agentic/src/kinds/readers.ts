// Readers of loose catalog JSON.

////////////////////////////////
//  Interfaces & Types

export type Fields = Readonly<Record<string, unknown>>;

////////////////////////////////
//  Functions & Helpers

/** An object that is no list. */
export function fieldsOf(value: unknown): Fields | undefined {
	return typeof value === "object" && value !== null && !Array.isArray(value) ? (value as Fields) : undefined;
}

/** Own fields only. */
export function fieldOf(fields: Fields, key: string): unknown {
	return Object.hasOwn(fields, key) ? fields[key] : undefined;
}

export function textField(fields: Fields, key: string): string | null {
	const value = fieldOf(fields, key);
	return typeof value === "string" ? value : null;
}

/** Finite numbers only. */
export function numberField(fields: Fields, key: string): number | null {
	const value = fieldOf(fields, key);
	return typeof value === "number" && Number.isFinite(value) ? value : null;
}

/** A length: a finite number above zero. */
export function positiveField(fields: Fields, key: string): number | null {
	const value = numberField(fields, key);
	return value !== null && value > 0 ? value : null;
}

export function booleanField(fields: Fields, key: string): boolean | null {
	const value = fieldOf(fields, key);
	return typeof value === "boolean" ? value : null;
}

/** A list of texts; one other item makes it missing. */
export function textsField(fields: Fields, key: string): string[] | null {
	const value = fieldOf(fields, key);
	if (!Array.isArray(value)) return null;
	return value.every((item) => typeof item === "string") ? (value as string[]) : null;
}

/** A nested object, or null. */
export function objectField(fields: Fields, key: string): Fields | null {
	return fieldsOf(fieldOf(fields, key)) ?? null;
}

/** The items of `{ data: [...] }`. */
export function dataList(raw: unknown): readonly unknown[] | undefined {
	const fields = fieldsOf(raw);
	const list = fields === undefined ? undefined : fieldOf(fields, "data");
	return Array.isArray(list) ? list : undefined;
}
