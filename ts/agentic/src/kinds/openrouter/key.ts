// OpenRouter key check.

import type { Span } from "@nyaaai/contract/fetch";
import { failureOf } from "../../failure";
import type { KeyAuth, Transport } from "../../transport";
import { type Fields, fieldOf, fieldsOf } from "../readers";

////////////////////////////////
//  Interfaces & Types

/** USD; absent limit means unlimited. */
export type KeyCheck =
	| { ok: true; usage?: number; limit?: number; reset?: string }
	| { ok: false; reason: "rejected" | "unreachable" };

////////////////////////////////
//  Functions & Helpers

/** A finite number. */
function usd(fields: Fields, key: string): number | undefined {
	const value = fieldOf(fields, key);
	return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

/** GET /key validates keys and reports limits. */
export async function checkKey(
	transport: Pick<Transport, "getJson">,
	auth: Exclude<KeyAuth, "none">,
	span: Span,
): Promise<KeyCheck> {
	let body: unknown;
	try {
		body = await transport.getJson("/key", span, auth);
	} catch (error) {
		if (span.signal.aborted) throw error;
		const category = failureOf(error)?.category;
		const rejected = category === "keyRejected" || category === "refused" || category === "keyMissing";
		return { ok: false, reason: rejected ? "rejected" : "unreachable" };
	}
	const fields = fieldsOf(body);
	const data = (fields === undefined ? undefined : fieldsOf(fieldOf(fields, "data"))) ?? {};
	const usage = usd(data, "usage");
	const limit = usd(data, "limit");
	const reset = fieldOf(data, "limit_reset");
	return {
		ok: true,
		...(usage === undefined ? {} : { usage }),
		...(limit === undefined ? {} : { limit }),
		...(typeof reset === "string" ? { reset } : {}),
	};
}
