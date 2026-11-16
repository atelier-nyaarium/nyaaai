// The provider list port.

import type { ProviderEntry } from "./shapes/provider";

////////////////////////////////
//  Interfaces & Types

/** The entries in priority order. */
export interface ProviderList {
	entries: () => readonly ProviderEntry[];
	/** Told when the list or an entry changed; the result unsubscribes. */
	onChange: (listener: () => void) => () => void;
}
