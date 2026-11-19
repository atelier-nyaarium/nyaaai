// Model row identity and its key codec.

import { Pattern } from "@nyaaai/contract/pattern";
import type { RowIdentity } from "@nyaaai/contract/shapes/model";
import { hasControl, trimText } from "@nyaaai/contract/text";
import { isEntryId, PROVIDER_KINDS } from "../provider/entry";
import { MODELS } from "./curated";

////////////////////////////////
//  Constants

const LOCAL = "local";

const CURATED: ReadonlySet<string> = new Set(MODELS.map((row) => row.key));

/** A provider's own id or tag; `~` starts an alias. */
const SERVED_ID = Pattern.compile("^~?\\w[\\w.:+-]*(?:\\/\\w[\\w.:+-]*)*$");

const ID_CHARS = 200;

const TAG_CHARS = 1024;

////////////////////////////////
//  Functions & Helpers

function servedId(value: string): boolean {
	return value.length <= ID_CHARS && SERVED_ID.test(value);
}

/** Servers name these: paths, spaces. */
function localTag(value: string): boolean {
	return trimText(value).length > 0 && value.length <= TAG_CHARS && !hasControl(value);
}

/** Settings and pick form. */
export function rowKey(identity: RowIdentity): string {
	switch (identity.kind) {
		case "curated":
			return identity.key;
		case "hosted":
			return `${identity.provider}:${identity.id}`;
		case "local":
			return `${LOCAL}:${identity.entry}/${identity.tag}`;
	}
}

/** Scheme before any slash; none means a curated key. */
export function parseRowKey(value: unknown): RowIdentity | undefined {
	if (typeof value !== "string") return undefined;
	const colon = value.indexOf(":");
	const slash = value.indexOf("/");
	if (colon < 0 || (slash >= 0 && slash < colon))
		return CURATED.has(value) ? { kind: "curated", key: value } : undefined;
	const scheme = value.slice(0, colon);
	const rest = value.slice(colon + 1);
	if (scheme === LOCAL) {
		const at = rest.indexOf("/");
		const entry = rest.slice(0, at);
		const tag = rest.slice(at + 1);
		return at > 0 && isEntryId(entry) && localTag(tag) ? { kind: "local", entry, tag } : undefined;
	}
	const provider = PROVIDER_KINDS.find((each) => each === scheme);
	return provider !== undefined && servedId(rest) ? { kind: "hosted", provider, id: rest } : undefined;
}

/** Browse id; slashless hosted ids show their kind. */
export function shownId(identity: RowIdentity): string {
	switch (identity.kind) {
		case "curated":
			return identity.key;
		case "hosted":
			return identity.id.includes("/") ? identity.id : `${identity.provider}/${identity.id}`;
		case "local":
			return `${identity.entry}/${identity.tag}`;
	}
}

/** A stored key's Browse id; unreadable keys as stored. */
export function shownKey(key: string): string {
	const identity = parseRowKey(key);
	return identity === undefined ? key : shownId(identity);
}

/** The serving kind's model id. */
export function servedIdOf(identity: Exclude<RowIdentity, { kind: "curated" }>): string {
	return identity.kind === "hosted" ? identity.id : identity.tag;
}
