import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Functions & Helpers

const messages = (name: string, ...values: unknown[]): CaseSource => ({
	kind: "failure.message",
	name,
	input: { values },
});

const failures = (name: string, ...thrown: object[]): CaseSource => ({
	kind: "failure.of",
	name,
	input: { thrown },
});

const failed = (category: string, message: string, extra?: object) => ({
	make: "failed",
	category,
	message,
	...(extra === undefined ? {} : { extra }),
});

const provider = (failure: object) => ({ make: "provider", failure });

export const failureCases: CaseSource[] = [
	messages("text", "x", "A message.", " padded ", "line\nbreak"),
	messages(
		"blank",
		"",
		" ",
		"  \t\n\r",
		"\u{a0}",
		"\u{2003}",
		"\u{2028}",
		"\u{2029}",
		"\u{feff}",
		"\u{3000}",
		"\u{1680}",
		"\u{202f}\u{205f}",
		"\u{b}\u{c}",
	),
	messages(
		"not-whitespace",
		"\u{200b}",
		"\u{200c}\u{200d}",
		"\u{85}",
		"\u{180e}",
		"\u{1680}x",
		"\u{2060}",
		"\u{feff}x",
	),
	messages("not-text", 5, 0, null, true, false, [], ["a"], {}, { message: "x" }),
	failures(
		"failed-defaults",
		failed("badRequest", "Bad."),
		failed("keyMissing", "No key."),
		failed("stream", "Cut."),
		failed("idle", ""),
	),
	failures(
		"failed-extra",
		failed("stream", "Bad chunk.", { status: "gemini" }),
		failed("rateLimited", "Slow.", { status: 429, retryAfterMs: 7000 }),
		failed("allowance", "Used up.", { resetsAt: 1794535200000 }),
		failed("refused", "No.", { reasons: ["a", "b"] }),
		failed("stream", "Cut.", { billed: { usage: { input: 3, output: 1 }, host: "h", generationId: "g" } }),
		failed("stream", "Cut.", { billed: {} }),
		failed("overloaded", "Busy.", {}),
	),
	failures(
		"provider-keeps-its-failure",
		provider({ category: "rateLimited", status: 429, retryAfterMs: 3000, message: "Slow down." }),
		provider({ category: "outOfCredit", status: 402, message: "" }),
		provider({ category: "network", status: "network", message: "Down.", reasons: [] }),
	),
	failures(
		"not-decoded",
		{ make: "error", message: "plain" },
		{ make: "value", value: "text" },
		{ make: "value", value: null },
		{ make: "value", value: 5 },
		{ make: "value", value: { failure: { category: "idle", status: "idle", message: "x" } } },
		{ make: "value", value: { category: "idle", status: "idle", message: "x" } },
		{ make: "value", value: ["failure"] },
	),
	failures(
		"mixed",
		{ make: "error", message: "plain" },
		failed("truncated", "Cut."),
		{ make: "value", value: 1 },
		provider({ category: "unavailable", status: 503, message: "Down." }),
	),
	failures("none"),
];
