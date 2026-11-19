// OpenAI-compatible servers: the model list, the network a host sits on, the request root.

import { Pattern } from "@nyaaai/contract/pattern";
import type { EntryFields } from "@nyaaai/contract/shapes/provider";
import { parseUrl } from "@nyaaai/contract/url";
import type { LocalModel, LocalServer, LocalTransport } from "./local/catalog";
import { HOME, localKind, type Terms } from "./local/kind";
import { dataList, fieldsOf, numberField, textField } from "./readers";

////////////////////////////////
//  Interfaces & Types

/** A model of the list; the widest of the three context fields the servers use. */
export interface ListedModel {
	id: string;
	context: number | null;
}

////////////////////////////////
//  Constants

const LIST = "/models";

/** Loopback, private ranges, LAN names; a host as the URL parser wrote it. */
const PRIVATE_HOST = Pattern.compile(
	"^(localhost|(127|10)\\.[\\d.]+|192\\.168\\.[\\d.]+|172\\.(1[6-9]|2\\d|3[01])\\.[\\d.]+|169\\.254\\.[\\d.]+|100\\.(6[4-9]|[7-9]\\d|1[01]\\d|12[0-7])\\.[\\d.]+|\\[(::1|f[cd][\\da-f:]*|fe[89ab][\\da-f:]*)\\]|[^.[]+|.+\\.(localhost|local|lan|internal|home\\.arpa))$",
	"i",
);

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
			numberField(fields, "context_length") ??
			numberField(fields, "max_context_length") ??
			numberField(fields, "max_model_len");
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

export function server(transport: LocalTransport): LocalServer {
	return {
		models: async (_known, signal) => {
			const models = listed(await transport.getJson(LIST, signal, "stored"));
			if (models === undefined) throw new Error("The model list did not parse.");
			return models.map(modelOf);
		},
	};
}

/** Off this network: no URL, an unreadable one, or a host that is not private. */
export function remote(entry: EntryFields): boolean {
	if (entry.url === undefined) return true;
	const parts = parseUrl(entry.url);
	return parts === undefined || !PRIVATE_HOST.test(parts.host);
}

/** A remote host may keep data and bill. */
export function termsOf(entry: EntryFields): Terms {
	return remote(entry) ? { kept: true, price: "unknown" } : HOME;
}

/** A bare origin means `/v1`; a URL that does not read throws. */
export function baseOf(url: string): string {
	const parts = parseUrl(url);
	if (parts === undefined) throw new Error("Invalid URL.");
	return parts.path === "/" ? `${url}/v1` : url;
}

////////////////////////////////
//  Kind

export const COMPATIBLE = localKind({
	kind: "compatible",
	label: "OpenAI-compatible",
	optionalKey: true,
	base: baseOf,
	terms: termsOf,
	chatPath: "/chat/completions",
	reach: async (transport, signal) => listed(await transport.getJson(LIST, signal, "stored"))?.length,
	server,
});
