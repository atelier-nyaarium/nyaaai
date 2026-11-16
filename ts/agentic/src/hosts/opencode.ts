// OpenCode's host, failure and usage rules.

import { headerOf } from "@nyaaai/contract/headers";
import type { Failure, FailureCategory, HeaderList, NormalizedUsage } from "@nyaaai/contract/shapes/step";
import { asciiLower } from "@nyaaai/contract/text";
import { type Clock, resetStampMs } from "@nyaaai/contract/time";
import { messageOf, type WireError } from "../failure";
import { type Host, isJson, numberOf } from "../native";

////////////////////////////////
//  Constants

export const SESSION_HEADER = "x-opencode-session";

/** Go and workspace limits. */
const LIMITS: ReadonlySet<string> = new Set(["GoUsageLimitError", "BudgetLimitError"]);

/** Documented `billing_source` values. */
const SOURCES: ReadonlySet<string> = new Set(["free", "balance", "go", "go-plus", "byok"]);

////////////////////////////////
//  Functions & Helpers

function untilReset(at: string | number | undefined, nowMs: number): number | undefined {
	const ms = resetStampMs(at, nowMs);
	return ms === undefined ? undefined : Math.max(0, ms - nowMs);
}

function amountOf(value: unknown): number | undefined {
	const amount = numberOf(value);
	return amount !== undefined && amount >= 0 ? amount + 0 : undefined;
}

function overloaded(error: WireError): boolean {
	return (error.message ?? "").includes("[overloaded]") || error.type === "overloaded_error";
}

function categoryOf(code: number | undefined, error: WireError, streamed: boolean): FailureCategory {
	const message = error.message ?? "";
	// Overload marker takes precedence.
	if ((streamed || code === 429) && overloaded(error)) return "overloaded";
	if (LIMITS.has(error.type ?? "")) return "allowance";
	if (code === undefined) return "stream";
	if (code === 400) return asciiLower(message).includes("model is unavailable") ? "notFound" : "badRequest";
	if (code === 401) return "keyRejected";
	if (code === 402) return "outOfCredit";
	if (code === 403) return "refused";
	if (code === 404) return "notFound";
	if (code === 429) return "rateLimited";
	if (code === 408 || code >= 500) return "unavailable";
	return "badRequest";
}

/** Rate limits prefer Retry-After. */
export function opencodeFailure(
	clock: Clock,
	status: number | "stream",
	error: WireError,
	headers?: HeaderList,
): Failure {
	const code = typeof status === "number" ? status : typeof error.code === "number" ? error.code : undefined;
	const category = categoryOf(code, error, status === "stream");
	const waits =
		category === "rateLimited" && (headers === undefined || headerOf(headers, "retry-after") === undefined);
	const retryAfterMs = waits ? untilReset(error.resetsAt, clock.nowMs()) : undefined;
	const resetsAt = category === "allowance" ? resetStampMs(error.resetsAt, clock.nowMs()) : undefined;
	return {
		category,
		status,
		message: messageOf(error.message) ?? "OpenCode reported an error.",
		...(retryAfterMs === undefined ? {} : { retryAfterMs }),
		...(resetsAt === undefined ? {} : { resetsAt }),
	};
}

/** OpenCode usage fields. */
export function opencodeUsage(own: NormalizedUsage, raw: Record<string, unknown>): NormalizedUsage {
	const block = isJson(raw["opencode"]) ? raw["opencode"] : {};
	const tokens = isJson(block["tokens"]) ? block["tokens"] : {};
	const cost = amountOf(raw["cost"]);
	const source = block["billing_source"];
	const listCost = amountOf(block["list_cost"]);
	const allowance = amountOf(block["allowance"]);
	const cached = own.cached ?? amountOf(tokens["cache_read"]);
	const writes = [amountOf(tokens["cache_write_5m"]), amountOf(tokens["cache_write_1h"])].filter(
		(each): each is number => each !== undefined,
	);
	const cacheWrite =
		own.cacheWrite ?? (writes.length === 0 ? undefined : writes.reduce((sum, each) => sum + each, 0));
	const reasoning = own.reasoning ?? amountOf(tokens["reasoning"]);
	return {
		...own,
		...(cached === undefined ? {} : { cached }),
		...(cacheWrite === undefined ? {} : { cacheWrite }),
		...(reasoning === undefined ? {} : { reasoning }),
		...(cost === undefined ? {} : { cost }),
		...(typeof source === "string" && SOURCES.has(source) ? { billingSource: source } : {}),
		...(listCost === undefined ? {} : { listCost }),
		...(allowance === undefined ? {} : { allowance }),
	};
}

/** The plan's routes, for the native families. */
export function opencodeHost(prefix: string, clock: Clock): Host {
	return {
		prefix,
		headers: (request) => ({ [SESSION_HEADER]: request.session }),
		usage: opencodeUsage,
		failure: (status, error, headers) => opencodeFailure(clock, status, error, headers),
	};
}
