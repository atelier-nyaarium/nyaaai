// The three standard uses of a model.

import type { ModelUse } from "@nyaaai/contract/shapes/model";

////////////////////////////////
//  Constants

/** Needs nothing of a model; leaves out hidden families. */
export const CHAT: ModelUse = { name: "chat", requires: [], hides: true };

/** Needs the schema ability. */
export const COMPACTION: ModelUse = { name: "compaction", requires: ["schema"], hides: false };

export const ANALYSIS: ModelUse = { name: "analysis", requires: ["schema"], hides: false };

/** In the order chat, compaction, analysis. */
export const STANDARD_USES: readonly ModelUse[] = [CHAT, COMPACTION, ANALYSIS];
