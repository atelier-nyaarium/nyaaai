// Header lists.

import { sortedTexts } from "./order";
import type { HeaderList } from "./shapes/step";
import { asciiLower } from "./text";

////////////////////////////////
//  Functions & Helpers

/** The first value whose name matches, ASCII case aside. */
export function headerOf(headers: ReadonlyArray<readonly [string, string]>, name: string): string | undefined {
	const wanted = asciiLower(name);
	return headers.find(([held]) => asciiLower(held) === wanted)?.[1];
}

/** Lowercase names, sorted; repeats joined with `, `, `set-cookie` kept apart. */
export function responseHeaders(raw: ReadonlyArray<readonly [string, string]>): HeaderList {
	const named = new Map<string, string[]>();
	for (const [name, value] of raw) {
		const key = asciiLower(name);
		const held = named.get(key);
		if (held) held.push(value);
		else named.set(key, [value]);
	}
	return sortedTexts([...named.keys()]).flatMap((key): HeaderList => {
		const values = named.get(key) ?? [];
		return key === "set-cookie" ? values.map((value) => [key, value]) : [[key, values.join(", ")]];
	});
}
