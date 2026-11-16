// The model registry port.

import type { ModelRow, StoredRegistry, TestRecord } from "./shapes/model";

////////////////////////////////
//  Interfaces & Types

/** Rows and the stored registry. */
export interface ModelRegistry {
	rows: () => readonly ModelRow[];
	/** Records, added keys and hidden keys. */
	stored: () => StoredRegistry;
	/** Writes one record; null when written, else why not. */
	record: (entry: string, id: string, record: TestRecord) => Promise<string | null>;
	/** Told when rows or the stored registry changed; the result unsubscribes. */
	onChange: (listener: () => void) => () => void;
}
