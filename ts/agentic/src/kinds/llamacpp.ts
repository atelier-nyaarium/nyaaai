// llama.cpp: the model list and each model's props, read by hand.

import { encodePathSegment } from "@nyaaai/contract/text";
import { joinAll } from "./join";
import { effortsOf, type LocalModel, type LocalServer, type LocalTransport } from "./local/catalog";
import { localKind } from "./local/kind";
import { dataList, fieldsOf, objectField, positiveField, textField, textsField } from "./readers";

////////////////////////////////
//  Interfaces & Types

/** A model of the list. */
export interface ListedModel {
	id: string;
	/** The trained length: the meta's, else the router's context length. */
	trained: number | null;
}

export interface Props {
	context?: number;
	efforts: readonly string[];
}

////////////////////////////////
//  Constants

const LIST = "/v1/models";

////////////////////////////////
//  Functions & Helpers

/** Models with an id. */
export function listed(raw: unknown): ListedModel[] | undefined {
	const items = dataList(raw);
	if (items === undefined) return undefined;
	return items.flatMap((item) => {
		const fields = fieldsOf(item);
		const id = fields === undefined ? null : textField(fields, "id");
		if (fields === undefined || id === null || id.length === 0) return [];
		const meta = objectField(fields, "meta");
		// Router lists, loaded or not.
		const trained =
			(meta === null ? null : positiveField(meta, "n_ctx_train")) ?? positiveField(fields, "context_length");
		return [{ id, trained }];
	});
}

/** What a props body says; none unless it is an object. */
export function propsOfBody(raw: unknown): Props | undefined {
	const fields = fieldsOf(raw);
	if (fields === undefined) return undefined;
	const settings = objectField(fields, "default_generation_settings");
	const caps = objectField(fields, "chat_template_caps");
	const context = settings === null ? null : positiveField(settings, "n_ctx");
	const efforts = caps === null ? null : textsField(caps, "reasoning_effort_levels");
	return { efforts: efforts ?? [], ...(context === null ? {} : { context }) };
}

/** Undefined while a router's model sleeps. */
async function propsOf(
	transport: LocalTransport,
	id: string,
	autoload: boolean,
	signal: AbortSignal,
): Promise<Props | undefined> {
	const path = `/props?model=${encodePathSegment(id)}&autoload=${autoload}` as const;
	return propsOfBody(await transport.getJson(path, signal, "none").catch(() => undefined));
}

/** Every template may call tools. A sleeping model carries known facts. */
export function modelOf(
	id: string,
	trained: number | undefined,
	props: Props | undefined,
	known: LocalModel | undefined,
): LocalModel {
	const efforts = props === undefined ? (known?.efforts ?? []) : effortsOf(props.efforts);
	const context = props === undefined ? known?.context : props.context;
	return {
		tag: id,
		tools: true,
		thinking: efforts.some((effort) => effort !== "none"),
		efforts,
		...(context === undefined ? {} : { context }),
		...(trained === undefined ? {} : { trained }),
		...(props === undefined ? { probed: false as const } : {}),
	};
}

export function server(transport: LocalTransport): LocalServer {
	return {
		models: async (known, signal) => {
			const models = listed(await transport.getJson(LIST, signal, "none"));
			if (models === undefined) throw new Error("The model list did not parse.");
			return joinAll(
				models.map(
					(model) => async () =>
						modelOf(
							model.id,
							model.trained ?? undefined,
							await propsOf(transport, model.id, false, signal),
							known.find((each) => each.tag === model.id),
						),
				),
			);
		},
		loaded: async (tag, signal) => {
			const context = (await propsOf(transport, tag, false, signal))?.context;
			return new Map(context === undefined ? [] : [[tag, context]]);
		},
		warm: async (tag, signal) => {
			await propsOf(transport, tag, true, signal);
		},
	};
}

////////////////////////////////
//  Kind

export const LLAMACPP = localKind({
	kind: "llamacpp",
	label: "llama.cpp",
	url: "http://127.0.0.1:8080",
	replay: "reasoning_content",
	reach: async (transport, signal) => listed(await transport.getJson(LIST, signal, "none"))?.length,
	server,
});
