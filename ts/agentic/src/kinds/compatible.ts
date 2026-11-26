// OpenAI-compatible model list, network and base URL.

import { parseUrl } from "@nyaaai/contract/url";
import { firstDistinct, MAX_LISTED } from "./join";
import type { LocalModel, LocalServer, LocalTransport } from "./local/catalog";
import { localKind } from "./local/kind";

export { remote, termsOf } from "./local/kind";

import { dataList, fieldsOf, positiveField, textField } from "./readers";

////////////////////////////////
//  Interfaces & Types

/** A listed model; context is the widest of the servers' fields. */
export interface ListedModel {
	id: string;
	context: number | null;
}

////////////////////////////////
//  Constants

const LIST = "/models";

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
		const context =
			positiveField(fields, "context_length") ??
			positiveField(fields, "max_context_length") ??
			positiveField(fields, "max_model_len");
		return [{ id, context }];
	});
}

/** Capabilities unknown; the test decides. */
export function modelOf(model: ListedModel): LocalModel {
	return {
		tag: model.id,
		tools: true,
		thinking: false,
		efforts: [],
		...(model.context === null ? {} : { context: model.context }),
	};
}

export function server(transport: LocalTransport, maxListed = MAX_LISTED): LocalServer {
	return {
		models: async (_known, span) => {
			const models = listed(await transport.getJson(LIST, span, "stored"));
			if (models === undefined) throw new Error("The model list did not parse.");
			return firstDistinct(models, (model) => model.id, maxListed).map(modelOf);
		},
	};
}

/** The text without its trailing slashes. */
function withoutSlashes(text: string): string {
	let end = text.length;
	while (end > 0 && text[end - 1] === "/") end -= 1;
	return text.slice(0, end);
}

/** A bare origin means `/v1`; an unreadable URL, or one with a query, fragment or credentials, throws. */
export function baseOf(url: string): string {
	const parts = parseUrl(url);
	const plain = parts !== undefined && parts.query === "" && parts.fragment === "";
	if (parts === undefined || !plain || parts.username !== "" || parts.password !== "")
		throw new Error("Invalid URL.");
	return parts.path === "/" ? `${parts.origin}/v1` : withoutSlashes(`${parts.origin}${parts.path}`);
}

////////////////////////////////
//  Kind

export const COMPATIBLE = localKind({
	kind: "compatible",
	label: "OpenAI-compatible",
	optionalKey: true,
	base: baseOf,
	chatPath: "/chat/completions",
	reach: async (transport, span) => listed(await transport.getJson(LIST, span, "stored"))?.length,
	server,
});
