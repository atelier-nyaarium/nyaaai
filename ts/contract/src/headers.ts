// Header lists.

import { asciiLower } from "./text";

////////////////////////////////
//  Functions & Helpers

/** The first value whose name matches, ASCII case aside. */
export function headerOf(headers: ReadonlyArray<readonly [string, string]>, name: string): string | undefined {
	const wanted = asciiLower(name);
	return headers.find(([held]) => asciiLower(held) === wanted)?.[1];
}
