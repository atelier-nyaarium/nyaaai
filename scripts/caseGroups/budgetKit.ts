// Shapes the budget cases share with the transport cases.

import type { BudgetChanges as Changes } from "@nyaaai/contract/fetch";
import { z } from "@nyaaai/contract/shape";

////////////////////////////////
//  Shapes

/** One preset's changed fields; any number, so a refusal can be asked. */
export const BudgetChange = z.object({
	totalMs: z.number().exactOptional(),
	idleMs: z.number().exactOptional(),
	maxBytes: z.number().exactOptional(),
});

/** A host's changes by preset name; an unknown name is refused. */
export const BudgetChanges = z.record(z.string(), BudgetChange);

////////////////////////////////
//  Functions & Helpers

/** The table function judges the names and the values. */
export const changesOf = (held: z.infer<typeof BudgetChanges> | undefined): Changes => (held ?? {}) as Changes;
