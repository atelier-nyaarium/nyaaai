// Builds a codec from its name in a case.

import type { Codec } from "@nyaaai/agentic/codec";
import { localChat } from "@nyaaai/agentic/dialects/local";
import { opencodeChat } from "@nyaaai/agentic/dialects/opencode";
import { OPENROUTER_CHAT } from "@nyaaai/agentic/dialects/openrouter";
import { z } from "@nyaaai/contract/shape";
import { ApiFamily, CodecFacts } from "@nyaaai/contract/shapes/step";
import { steppedClock } from "@nyaaai/contract/time";

////////////////////////////////
//  Interfaces & Types

/** A URL server's dialect and where it serves from. */
export const LocalName = z.object({
	host: z.string(),
	label: z.string(),
	replay: z.enum(["reasoning", "reasoning_content"]).exactOptional(),
	/** Starts with a slash. */
	chatPath: z.string().exactOptional(),
	optionalKey: z.boolean().exactOptional(),
});

/** A codec by data: `<family>/<dialect>` picks the builder, the rest feeds it. */
export const CodecName = z.object({
	family: ApiFamily,
	dialect: z.string(),
	facts: CodecFacts.exactOptional(),
	local: LocalName.exactOptional(),
	/** The scripted instant a dialect measures reset stamps from. Defaults to zero. */
	nowMs: z.int().exactOptional(),
});
export type CodecName = z.infer<typeof CodecName>;

type Builder = (name: CodecName) => Codec;

////////////////////////////////
//  Functions & Helpers

function slashed(path: string): `/${string}` {
	if (!path.startsWith("/")) throw new Error(`a chat path starts with a slash: ${path}`);
	return `/${path.slice(1)}` as const;
}

function localOf(name: CodecName): Codec {
	const { local } = name;
	if (local === undefined) throw new Error("a local codec needs its `local` part");
	return localChat(name.facts, local.host, {
		label: local.label,
		...(local.replay === undefined ? {} : { replay: local.replay }),
		...(local.chatPath === undefined ? {} : { chatPath: slashed(local.chatPath) }),
		...(local.optionalKey === true ? { optionalKey: true as const } : {}),
	});
}

////////////////////////////////
//  Builders

/** A later family registers one row here, and the same arm in `rust/cli/src/cases/codec.rs`. */
const BUILDERS = new Map<string, Builder>([
	["chat-completions/openrouter", () => OPENROUTER_CHAT],
	["chat-completions/opencode", (name) => opencodeChat(name.facts, steppedClock(name.nowMs ?? 0))],
	["chat-completions/local", localOf],
]);

export function codecOf(name: CodecName): Codec {
	const build = BUILDERS.get(`${name.family}/${name.dialect}`);
	if (build === undefined) throw new Error(`no codec named ${name.family}/${name.dialect}`);
	return build(name);
}
