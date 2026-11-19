// LM Studio: the model list, read by hand.

import { effortsOf, itemsOf, type LocalModel, type LocalServer, type LocalTransport } from "./local/catalog";
import { localKind } from "./local/kind";
import { fieldOf, fieldsOf, objectField, positiveField, textField, textsField } from "./readers";

////////////////////////////////
//  Interfaces & Types

/** A chat model of the list. */
export interface ListedModel {
	key: string;
	/** The widest context the model trained for. */
	trained: number | null;
	/** The first loaded instance's context. */
	loaded: number | null;
	/** The reasoning options, when listed. */
	options: string[] | null;
}

////////////////////////////////
//  Constants

const LIST = "/api/v1/models";

////////////////////////////////
//  Functions & Helpers

/** The first instance's context; none when any instance does not read. */
function loadedOf(fields: Readonly<Record<string, unknown>>): number | null {
	const list = fieldOf(fields, "loaded_instances");
	if (!Array.isArray(list)) return null;
	const configs = list.map((instance) => {
		const held = fieldsOf(instance);
		return held === undefined ? null : objectField(held, "config");
	});
	const first = configs[0];
	if (configs.includes(null) || first === undefined || first === null) return null;
	return positiveField(first, "context_length");
}

function listedOf(item: unknown): ListedModel | undefined {
	const fields = fieldsOf(item);
	if (fields === undefined) return undefined;
	const key = textField(fields, "key");
	if (key === null || key.length === 0 || textField(fields, "type") !== "llm") return undefined;
	const capabilities = objectField(fields, "capabilities");
	const reasoning = capabilities === null ? null : objectField(capabilities, "reasoning");
	return {
		key,
		trained: positiveField(fields, "max_context_length"),
		loaded: loadedOf(fields),
		options: reasoning === null ? null : textsField(reasoning, "allowed_options"),
	};
}

export function chatModels(raw: unknown): ListedModel[] | undefined {
	const items = itemsOf(raw, "models");
	if (items === undefined) return undefined;
	return items.flatMap((item) => {
		const model = listedOf(item);
		return model === undefined ? [] : [model];
	});
}

/** Every model may call tools. */
export function modelOf(model: ListedModel): LocalModel {
	const options = model.options ?? [];
	const thinks = options.some((option) => option !== "off");
	const values = options.map((option) => (option === "off" ? false : option === "on" ? true : option));
	return {
		tag: model.key,
		tools: true,
		thinking: thinks,
		efforts: thinks ? effortsOf(values) : [],
		...(model.loaded === null ? {} : { loaded: model.loaded }),
		...(model.trained === null ? {} : { trained: model.trained }),
	};
}

/** Loaded context by key. */
export function loadedContexts(models: readonly ListedModel[]): Map<string, number> {
	return new Map(models.flatMap((model) => (model.loaded === null ? [] : [[model.key, model.loaded] as const])));
}

export function server(transport: LocalTransport): LocalServer {
	return {
		live: true,
		models: async (_known, signal) => {
			const models = chatModels(await transport.getJson(LIST, signal, "none"));
			if (models === undefined) throw new Error("The model list did not parse.");
			return models.map(modelOf);
		},
		loaded: async (_tag, signal) => loadedContexts(chatModels(await transport.getJson(LIST, signal, "none")) ?? []),
	};
}

////////////////////////////////
//  Kind

export const LMSTUDIO = localKind({
	kind: "lmstudio",
	label: "LM Studio",
	url: "http://127.0.0.1:1234",
	reach: async (transport, signal) => chatModels(await transport.getJson(LIST, signal, "none"))?.length,
	server,
});
