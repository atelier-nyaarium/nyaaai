// Ollama: tags, show and running models, read by hand.

import { numberOfText } from "@nyaaai/contract/number";
import { isSpace } from "@nyaaai/contract/text";
import { firstDistinct, joinLimited, MAX_LISTED } from "./join";
import { effortsOf, itemsOf, type LocalModel, type LocalServer, type LocalTransport } from "./local/catalog";
import { localKind } from "./local/kind";
import { fieldOf, fieldsOf, objectField, positiveField, textField, textsField } from "./readers";

////////////////////////////////
//  Constants

const NUM_CTX = "num_ctx";

////////////////////////////////
//  Functions & Helpers

const isLineEnd = (unit: number) => unit === 0x0a || unit === 0x0d || unit === 0x2028 || unit === 0x2029;

const isDigit = (unit: number) => unit >= 0x30 && unit <= 0x39;

/** The first `num_ctx` number; none unless above zero. */
export function numCtxOf(text: string): number | undefined {
	let from = 0;
	while (from <= text.length) {
		const digits = text.startsWith(NUM_CTX, from) ? digitsAfter(text, from + NUM_CTX.length) : undefined;
		if (digits !== undefined) {
			let first = 0;
			while (first < digits.length - 1 && digits[first] === "0") first += 1;
			const found = numberOfText(digits.slice(first));
			return found !== undefined && found > 0 ? found : undefined;
		}
		let next = from;
		while (next < text.length && !isLineEnd(text.charCodeAt(next))) next += 1;
		if (next >= text.length) return undefined;
		from = next + 1;
	}
	return undefined;
}

/** The digits after blanks, when they end the line. */
function digitsAfter(text: string, start: number): string | undefined {
	let at = start;
	while (at < text.length && isSpace(text.charCodeAt(at))) at += 1;
	if (at === start) return undefined;
	const first = at;
	while (at < text.length && isDigit(text.charCodeAt(at))) at += 1;
	if (at === first) return undefined;
	const digits = text.slice(first, at);
	let ends = at === text.length;
	while (at < text.length && isSpace(text.charCodeAt(at))) {
		if (isLineEnd(text.charCodeAt(at))) ends = true;
		at += 1;
	}
	return ends || at === text.length ? digits : undefined;
}

/** Local tags only. */
export function localTags(raw: unknown): string[] | undefined {
	const items = itemsOf(raw, "models");
	if (items === undefined) return undefined;
	return items.flatMap((item) => {
		const fields = fieldsOf(item);
		const name = fields === undefined ? null : textField(fields, "name");
		if (fields === undefined || name === null || name.length === 0) return [];
		return textField(fields, "remote_host") === null ? [name] : [];
	});
}

/** The thinking values, when the field holds a list of them. */
function thinkingValues(fields: Readonly<Record<string, unknown>>): (boolean | string)[] | null {
	const thinking = objectField(fields, "thinking");
	const list = thinking === null ? undefined : fieldOf(thinking, "values");
	if (!Array.isArray(list)) return null;
	return list.every((value) => typeof value === "boolean" || typeof value === "string")
		? (list as (boolean | string)[])
		: null;
}

/** Completion models only. */
export function modelOf(tag: string, raw: unknown): LocalModel | undefined {
	const fields = fieldsOf(raw);
	if (fields === undefined) return undefined;
	const can = textsField(fields, "capabilities") ?? [];
	if (!can.includes("completion")) return undefined;
	const listed = thinkingValues(fields);
	const values = listed ?? [];
	// False-only disables thinking.
	const thinks = can.includes("thinking") && (listed === null || values.some((value) => value !== false));
	const parameters = textField(fields, "parameters");
	const context = numCtxOf(parameters ?? "");
	const info = objectField(fields, "model_info");
	const arch = info === null ? undefined : fieldOf(info, "general.architecture");
	const trained = typeof arch === "string" && info !== null ? fieldOf(info, `${arch}.context_length`) : undefined;
	return {
		tag,
		tools: can.includes("tools"),
		thinking: thinks,
		efforts: thinks ? effortsOf(values) : [],
		...(context === undefined ? {} : { context }),
		...(typeof trained === "number" && trained > 0 ? { trained } : {}),
	};
}

/** Loaded context by name, from `/api/ps`. */
export function runningOf(raw: unknown): Map<string, number> {
	const contexts = new Map<string, number>();
	for (const item of itemsOf(raw, "models") ?? []) {
		const fields = fieldsOf(item);
		const name = fields === undefined ? null : textField(fields, "name");
		if (fields === undefined || name === null) continue;
		const context = positiveField(fields, "context_length");
		if (context !== null) contexts.set(name, context);
	}
	return contexts;
}

export function server(transport: LocalTransport, maxListed = MAX_LISTED): LocalServer {
	return {
		models: async (known, signal) => {
			const listing = localTags(await transport.getJson("/api/tags", signal, "none"));
			if (listing === undefined) throw new Error("The model list did not parse.");
			const tags = firstDistinct(listing, (tag) => tag, maxListed);
			const last = new Map(known.map((model) => [model.tag, model]));
			const shown = await joinLimited(
				tags.map(
					(tag) => () =>
						transport.postJson("/api/show", { model: tag }, signal, "none").then(
							(raw) => modelOf(tag, raw),
							// Failed show retains prior.
							() => last.get(tag),
						),
				),
			);
			return shown.filter((model): model is LocalModel => model !== undefined);
		},
		loaded: async (_tag, signal) => runningOf(await transport.getJson("/api/ps", signal, "none")),
	};
}

////////////////////////////////
//  Kind

export const OLLAMA = localKind({
	kind: "ollama",
	label: "Ollama",
	url: "http://127.0.0.1:11434",
	replay: "reasoning",
	reach: async (transport, signal) => localTags(await transport.getJson("/api/tags", signal, "none"))?.length,
	server,
});
