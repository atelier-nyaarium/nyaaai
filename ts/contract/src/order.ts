// UTF-16 code unit order.

////////////////////////////////
//  Functions & Helpers

/** Compares by UTF-16 code unit. */
export function compareText(left: string, right: string): number {
	if (left < right) return -1;
	return left > right ? 1 : 0;
}

export const sortedTexts = (texts: readonly string[]): string[] => [...texts].sort(compareText);
