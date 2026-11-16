// Joining streamed step events.

import type {
	Failure,
	FinishReason,
	NormalizedUsage,
	ReplyPart,
	Serving,
	StepEvent,
	StepResult,
	ThoughtDelta,
	ThoughtPart,
} from "@nyaaai/contract/shapes/step";
import { trimText } from "@nyaaai/contract/text";
import { failed } from "./failure";

////////////////////////////////
//  Interfaces & Types

export type ReplyCall = Extract<ReplyPart, { kind: "call" }>;

export type Billed = NonNullable<Failure["billed"]>;

////////////////////////////////
//  Functions & Helpers

export function replyText(result: StepResult): string {
	return result.parts.flatMap((part) => (part.kind === "text" ? [part.text] : [])).join("");
}

/** By index. */
export function replyCalls(result: StepResult): ReplyCall[] {
	return result.parts
		.filter((part): part is ReplyCall => part.kind === "call")
		.sort((left, right) => left.index - right.index);
}

export function replyThoughts(result: StepResult): ThoughtPart[] {
	return result.parts.filter((part): part is ThoughtPart => part.kind === "thought");
}

/** A part sharing nothing with the original. */
function copiedPart<T extends ReplyPart>(part: T): T {
	if (part.kind !== "thought" || part.segments === undefined) return { ...part };
	return { ...part, segments: part.segments.map((segment) => ({ ...segment })) };
}

/** Joins matching seam segments unless split. */
function joinedSegments<T extends { kind: string; text: string }>(
	left: readonly T[],
	right: readonly T[],
	split: boolean,
): T[] {
	const last = left.at(-1);
	const first = right[0];
	if (split || last === undefined || first === undefined || last.kind !== first.kind)
		return [...left, ...right.map((segment) => ({ ...segment }))];
	return [
		...left.slice(0, -1),
		{ ...last, text: last.text + first.text },
		...right.slice(1).map((segment) => ({ ...segment })),
	];
}

/** Text appends; fields replace. */
function mergedThought(into: ThoughtPart, part: ThoughtPart, split: boolean): ThoughtPart {
	const segments = joinedSegments(into.segments ?? [], part.segments ?? [], split);
	return {
		...into,
		...(segments.length === 0 ? {} : { segments }),
		...(part.opaque === undefined ? {} : { opaque: part.opaque }),
		...(part.id === undefined ? {} : { id: part.id }),
		...(part.format === undefined ? {} : { format: part.format }),
		...(part.index === undefined ? {} : { index: part.index }),
		...(part.anchor === undefined ? {} : { anchor: part.anchor }),
	};
}

////////////////////////////////
//  Classes

/** Calls join by index; thoughts by block. */
export class StepAssembler {
	private readonly parts: ReplyPart[] = [];
	private readonly calls = new Map<number, number>();
	private readonly blocks = new Map<string, number>();
	/** Last blockless position. */
	private loose: number | undefined;
	private readonly serving: Serving = {};
	private finish: FinishReason | undefined;
	private usage: NormalizedUsage | undefined;
	private seen = false;
	private started = false;

	get outputSeen(): boolean {
		return this.seen;
	}

	/** Failed-attempt billing, if reported. */
	get billed(): Billed | undefined {
		if (!this.started) return undefined;
		return {
			...(this.usage === undefined ? {} : { usage: this.usage }),
			...(this.serving.host === undefined ? {} : { host: this.serving.host }),
			...(this.serving.generationId === undefined ? {} : { generationId: this.serving.generationId }),
		};
	}

	/** Current assembled parts. */
	get snapshot(): ReplyPart[] {
		return this.parts.map(copiedPart);
	}

	/** Whether parts changed. */
	add(event: StepEvent): boolean {
		this.started = true;
		switch (event.kind) {
			case "text":
				if (event.text.length === 0) return false;
				this.addText(event.text, event.phase);
				return true;
			case "thought":
				this.addThought(event);
				return true;
			case "call":
				this.addCall(event);
				return true;
			case "serving":
				if (this.serving.host === undefined && event.serving.host !== undefined)
					this.serving.host = event.serving.host;
				if (this.serving.generationId === undefined && event.serving.generationId !== undefined)
					this.serving.generationId = event.serving.generationId;
				return false;
			case "usage":
				this.usage = event.usage;
				return false;
			case "finish":
				// Usage repeats finish.
				this.finish ??= event.reason;
				return false;
		}
	}

	/** Missing call ids and empty arguments are filled. */
	result(): StepResult {
		const parts = this.parts.map((part): ReplyPart => {
			if (part.kind !== "call") return copiedPart(part);
			return {
				...part,
				wireId: part.wireId.length > 0 ? part.wireId : `${this.serving.generationId ?? "call"}_${part.index}`,
				argumentsText: trimText(part.argumentsText).length === 0 ? "{}" : part.argumentsText,
			};
		});
		return {
			parts,
			finish: this.finish ?? "",
			...(this.usage === undefined ? {} : { usage: this.usage }),
			serving: { ...this.serving },
		};
	}

	/** Joins the last text of the same phase. */
	private addText(text: string, phase: string | undefined): void {
		this.seen = true;
		const last = this.parts.at(-1);
		if (last?.kind === "text" && last.phase === phase)
			this.parts[this.parts.length - 1] = { ...last, text: last.text + text };
		else this.parts.push(phase === undefined ? { kind: "text", text } : { kind: "text", text, phase });
	}

	/** Blocks join by ID; loose deltas merge. */
	private addThought({ part, block, split }: ThoughtDelta): void {
		this.seen = true;
		const at = block === undefined ? this.continued(part) : this.keyed(block, part);
		const known = at === undefined ? undefined : this.parts[at];
		if (at !== undefined && known?.kind === "thought") {
			this.parts[at] = mergedThought(known, part, split === true);
			return;
		}
		if (block === undefined) this.loose = this.parts.length;
		else this.blocks.set(block, this.parts.length);
		this.parts.push(copiedPart(part));
	}

	/** Unsealed loose tail of this type. */
	private looseTail(type: string): number | undefined {
		const at = this.parts.length - 1;
		const last = this.parts[at];
		return this.loose === at && last?.kind === "thought" && last.type === type && last.opaque === undefined
			? at
			: undefined;
	}

	private continued(part: ThoughtPart): number | undefined {
		return part.opaque === undefined ? this.looseTail(part.type) : undefined;
	}

	/** A block's first delta adopts the loose tail. */
	private keyed(block: string, part: ThoughtPart): number | undefined {
		const known = this.blocks.get(block);
		if (known !== undefined) return known;
		const at = this.looseTail(part.type);
		if (at === undefined) return undefined;
		this.blocks.set(block, at);
		this.loose = undefined;
		return at;
	}

	private addCall(event: Extract<StepEvent, { kind: "call" }>): void {
		if (!Number.isInteger(event.index) || event.index < 0)
			throw failed("stream", "A tool call had no valid index.");
		this.seen = true;
		const at = this.calls.get(event.index);
		const known = at === undefined ? undefined : this.parts[at];
		if (at === undefined || known?.kind !== "call") {
			this.calls.set(event.index, this.parts.length);
			this.parts.push({
				kind: "call",
				index: event.index,
				wireId: event.wireId,
				name: event.name,
				argumentsText: event.argumentsText,
			});
			return;
		}
		if (known.wireId.length > 0 && event.wireId.length > 0 && known.wireId !== event.wireId)
			throw failed("stream", "Two tool calls shared an index.");
		this.parts[at] = {
			...known,
			wireId: known.wireId.length > 0 ? known.wireId : event.wireId,
			name: known.name.length > 0 ? known.name : event.name,
			argumentsText: known.argumentsText + event.argumentsText,
		};
	}
}
