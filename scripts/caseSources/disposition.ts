// Failure disposition cases.

import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Helpers

const CATEGORIES = [
	"keyMissing",
	"keyRejected",
	"outOfCredit",
	"allowance",
	"refused",
	"notFound",
	"badRequest",
	"replayRejected",
	"rateLimited",
	"overloaded",
	"unavailable",
	"network",
	"stream",
	"empty",
	"truncated",
	"idle",
];

/** No view first, then each status a view holds. */
const VIEWS: Array<string | null> = [null, "ok", "retired", "noZdrHost", "noTools", "unknown"];

const failure = (category: string, extra: Record<string, unknown> = {}) => ({
	category,
	status: category,
	message: "m",
	...extra,
});

const against = (name: string, failures: unknown[]): CaseSource => ({
	kind: "disposition.of",
	name,
	input: { rows: failures.flatMap((held) => VIEWS.map((status) => ({ failure: held, status }))) },
});

////////////////////////////////
//  Cases

export const dispositionCases: CaseSource[] = [
	...CATEGORIES.map((category) => against(`of-${category}`, [failure(category)])),
	against("of-out-of-credit-with-retry-after", [
		failure("outOfCredit", { retryAfterMs: 1 }),
		failure("outOfCredit", { retryAfterMs: 0 }),
		failure("outOfCredit", { retryAfterMs: 60000, status: 402 }),
	]),
	against("of-out-of-credit-with-reset", [failure("outOfCredit", { resetsAt: 1790000000000 })]),
	against("of-allowance-with-retry-after", [failure("allowance", { retryAfterMs: 5000 })]),
	against("of-not-found-with-status", [
		failure("notFound", { status: 404 }),
		failure("notFound", { status: "stream" }),
	]),
	against("of-status-and-reasons-do-not-matter", [
		failure("rateLimited", { status: 429, retryAfterMs: 1000, reasons: ["x"] }),
		failure("badRequest", { status: 400, reasons: ["x"] }),
		failure("refused", { status: 403, billed: { host: "h" } }),
	]),
	{
		kind: "disposition.skipReason",
		name: "every-category",
		input: { failures: CATEGORIES.map((category) => failure(category)) },
	},
	{
		kind: "disposition.skipReason",
		name: "out-of-credit-either-way",
		input: {
			failures: [
				failure("outOfCredit"),
				failure("outOfCredit", { retryAfterMs: 1000 }),
				failure("allowance", { retryAfterMs: 1000 }),
				failure("notFound", { status: 404 }),
			],
		},
	},
];
