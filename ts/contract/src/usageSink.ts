// The usage sink port.

import type { GenerationStats, MeteredCall, UnfilledRecord } from "./shapes/usage";

////////////////////////////////
//  Interfaces & Types

/** Where metered calls and their later stats are written. */
export interface UsageSink {
	/** The record's id; undefined when not kept. */
	record: (call: MeteredCall) => string | undefined;
	/** Joins stats to a record by its id. */
	fill: (id: string, stats: GenerationStats) => void;
	/** Whether the record already holds stats; `at` is when the call began. */
	filled: (id: string, at: number) => Promise<boolean>;
	/** Records since `since` that name a generation with no stats yet. */
	unfilled: (since: number) => Promise<UnfilledRecord[]>;
}
