// Generated from prompts/operator by scripts/contract.ts. Do not edit.

/** prompts/operator/cleared-note.md */
export const clearedNote = (slots: { rows: string; sourcePath: string }): string =>
	`[The owner cleared ${slots.rows} earlier rows of this conversation without summarizing them. They remain in the transcript at: ${slots.sourcePath}]`;

/** prompts/operator/compacted-summary.md */
export const compactedSummary = (slots: { summary: string; sourcePath: string }): string =>
	`This session is being continued from an earlier part of this conversation, which the owner compacted above a message they chose. The summary below covers everything before that message.

${slots.summary}

If you need specific details from before the compaction, read the full transcript at: ${slots.sourcePath}

The messages after this summary are the conversation from that message on, kept verbatim.`;
