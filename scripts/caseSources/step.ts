import type { CaseSource } from "../caseAdapter";

////////////////////////////////
//  Functions & Helpers

type Segment = { kind: "summary" | "content"; text: string };

const content = (text: string): Segment => ({ kind: "content", text });
const summary = (text: string): Segment => ({ kind: "summary", text });

const text = (held: string, phase?: string) => ({
	kind: "text",
	text: held,
	...(phase === undefined ? {} : { phase }),
});

const part = (type: string, segments?: Segment[], extra: Record<string, unknown> = {}) => ({
	kind: "thought",
	type,
	...(segments === undefined ? {} : { segments }),
	...extra,
});

const thought = (held: ReturnType<typeof part>, block?: string, split?: boolean) => ({
	kind: "thought",
	part: held,
	...(block === undefined ? {} : { block }),
	...(split === undefined ? {} : { split }),
});

const call = (index: number, wireId: string, name: string, argumentsText: string) => ({
	kind: "call",
	index,
	wireId,
	name,
	argumentsText,
});

const serving = (held: { host?: string; generationId?: string }) => ({ kind: "serving", serving: held });
const usage = (input: number, output: number, more: Record<string, unknown> = {}) => ({
	kind: "usage",
	usage: { input, output, ...more },
});
const finish = (reason: string) => ({ kind: "finish", reason });

const assembles = (name: string, ...events: object[]): CaseSource => ({
	kind: "step.assemble",
	name,
	input: { events },
});

const replies = (name: string, parts: object[], finishReason = "stop"): CaseSource => ({
	kind: "step.replies",
	name,
	input: { result: { parts, finish: finishReason, serving: {} } },
});

const STOP = finish("stop");

export const stepCases: CaseSource[] = [
	// Text.
	assembles("text-joins", text("He"), text("llo"), STOP),
	assembles("text-phases", text("a"), text("b", "final"), text("c", "final"), text("d"), text("e", "commentary")),
	assembles("text-phase-then-none", text("a", "final"), text("b"), text("c", "final")),
	assembles("text-empty-is-ignored", text(""), text("a"), text("")),
	assembles("text-empty-only", text("")),
	assembles("text-after-call-is-new", text("a"), call(0, "c0", "read", "{}"), text("b"), text("c")),
	assembles("text-after-thought-is-new", thought(part("t", [content("x")])), text("a"), text("b")),

	// Nothing, or no output.
	assembles("no-events"),
	assembles("no-finish", text("a"), usage(1, 2)),
	assembles("usage-only", usage(1, 2)),
	assembles("serving-only", serving({ host: "h1", generationId: "g1" })),
	assembles("finish-only", STOP),
	assembles("finish-empty-first-wins", finish(""), STOP),
	assembles("finish-first-wins", STOP, finish("length"), finish("error")),
	assembles("finish-names", text("a"), finish("content_filter")),

	// Serving and usage.
	assembles(
		"serving-first-wins",
		serving({ host: "h1" }),
		serving({ host: "h2", generationId: "g1" }),
		serving({ generationId: "g2" }),
	),
	assembles(
		"serving-empty-strings",
		serving({ host: "" }),
		serving({ host: "h2", generationId: "" }),
		serving({ generationId: "g1" }),
	),
	assembles("serving-empty-object", serving({}), text("a")),
	assembles(
		"usage-last-wins",
		usage(3, 1),
		usage(10, 5, { cached: 2, cost: 0.5, byok: true, billingSource: "credit" }),
		text("a"),
		STOP,
	),
	assembles("usage-before-text", usage(1, 1), text("a")),
	assembles("billed-after-serving-and-usage", serving({ host: "h1" }), usage(4, 2), serving({ generationId: "g9" })),

	// Thoughts without a block.
	assembles(
		"thought-loose-joins",
		thought(part("reasoning.text", [content("Div")])),
		thought(part("reasoning.text", [content("ide.")])),
	),
	assembles(
		"thought-loose-types-differ",
		thought(part("reasoning.text", [content("a")])),
		thought(part("reasoning.summary", [content("b")])),
		thought(part("reasoning.summary", [content("c")])),
	),
	assembles(
		"thought-loose-seam-kinds-differ",
		thought(part("t", [summary("s1")])),
		thought(part("t", [content("c1")])),
		thought(part("t", [content("c2"), summary("s2")])),
		thought(part("t", [summary("s3")])),
	),
	assembles(
		"thought-loose-several-segments",
		thought(part("t", [summary("a"), content("b")])),
		thought(part("t", [content("c"), content("d")])),
	),
	assembles(
		"thought-loose-after-text",
		thought(part("t", [content("a")])),
		text("x"),
		thought(part("t", [content("b")])),
	),
	assembles(
		"thought-loose-after-call",
		thought(part("t", [content("a")])),
		call(0, "c0", "read", "{}"),
		thought(part("t", [content("b")])),
	),
	assembles(
		"thought-opaque-seals",
		thought(part("t", [content("a")], { opaque: "sig-1" })),
		thought(part("t", [content("b")])),
		thought(part("t", [content("c")])),
	),
	assembles(
		"thought-opaque-delta-is-new",
		thought(part("t", [content("a")])),
		thought(part("t", [content("b")], { opaque: "sig-1" })),
	),
	assembles(
		"thought-fields-replace",
		thought(part("t", [content("a")], { id: "i1", format: "f1", index: 0, anchor: "a1" })),
		thought(part("t", [content("b")], { id: "i2", index: 3 })),
		thought(part("t", undefined, { opaque: "sig-2", anchor: "a2" })),
	),
	assembles(
		"thought-fields-stay-when-absent",
		thought(part("t", [content("a")], { id: "i1", format: "f1", index: 0, anchor: "a1" })),
		thought(part("t", [content("b")])),
	),
	assembles(
		"thought-type-stays",
		thought(part("t1", [content("a")])),
		thought(part("t1", [content("b")], { format: "x" })),
	),
	assembles("thought-no-segments", thought(part("t")), thought(part("t")), thought(part("t", [content("a")]))),
	assembles("thought-block-keeps-first-part-as-is", thought(part("t", [content("a")], { index: 2 }), "b0")),

	// Thoughts with a block.
	assembles(
		"thought-blocks-interleave",
		thought(part("reasoning", [content("A1")]), "reasoning#0"),
		thought(part("reasoning", [content("B1")]), "reasoning#1"),
		thought(part("reasoning", [content("A2")]), "reasoning#0"),
		thought(part("reasoning", [content("B2")]), "reasoning#1"),
	),
	assembles(
		"thought-blocks-after-other-parts",
		thought(part("t", [content("A1")]), "b0"),
		text("x"),
		thought(part("t", [content("A2")]), "b0"),
		call(0, "c0", "read", "{}"),
		thought(part("t", [content("A3")]), "b0"),
	),
	assembles(
		"thought-block-adopts-loose-tail",
		thought(part("t", [content("a")])),
		thought(part("t", [content("b")]), "b0"),
		thought(part("t", [content("c")]), "b0"),
		thought(part("t", [content("d")])),
	),
	assembles(
		"thought-block-adopts-once",
		thought(part("t", [content("a")])),
		thought(part("t", [content("b")]), "b0"),
		thought(part("t", [content("c")]), "b1"),
	),
	assembles(
		"thought-block-does-not-adopt-other-type",
		thought(part("t1", [content("a")])),
		thought(part("t2", [content("b")]), "b0"),
	),
	assembles(
		"thought-block-does-not-adopt-sealed",
		thought(part("t", [content("a")], { opaque: "sig" })),
		thought(part("t", [content("b")]), "b0"),
	),
	assembles(
		"thought-block-does-not-adopt-after-text",
		thought(part("t", [content("a")])),
		text("x"),
		thought(part("t", [content("b")]), "b0"),
	),
	assembles(
		"thought-block-sealed-still-joins-its-block",
		thought(part("t", [content("a")]), "b0"),
		thought(part("t", [content("b")], { opaque: "sig" }), "b0"),
		thought(part("t", [content("c")]), "b0"),
	),
	assembles(
		"thought-blocks-do-not-continue-loose",
		thought(part("t", [content("a")]), "b0"),
		thought(part("t", [content("b")])),
	),
	assembles(
		"thought-split-starts-a-segment",
		thought(part("t", [summary("one")]), "i1"),
		thought(part("t", [summary("two")]), "i1", true),
		thought(part("t", [summary("three")]), "i1"),
		thought(part("t", [content("four")]), "i1", true),
	),
	assembles(
		"thought-split-false-joins",
		thought(part("t", [summary("one")]), "i1"),
		thought(part("t", [summary("two")]), "i1", false),
	),
	assembles(
		"thought-split-without-block",
		thought(part("t", [content("a")])),
		thought(part("t", [content("b")]), undefined, true),
	),
	assembles(
		"thought-split-on-first-delta",
		thought(part("t", [content("a"), content("b")]), "i1", true),
		thought(part("t", [content("c")]), "i1"),
	),

	// Calls.
	assembles(
		"calls-by-index",
		call(1, "", "write", '{"pa'),
		call(0, "", "read", ""),
		call(1, "call_b", "ignored", 'th":"a"}'),
		call(2, "call_c", "broken", "{not json"),
		call(0, "call_a", "", ""),
		finish("tool_calls"),
	),
	assembles("call-pieces-append", call(0, "c0", "read", '{"a'), call(0, "", "", '":1,'), call(0, "c0", "", '"b":2}')),
	assembles("call-first-name-wins", call(0, "", "", "x"), call(0, "c0", "read", "y"), call(0, "c1x", "other", "z")),
	assembles("call-same-id-repeats", call(0, "c0", "read", "a"), call(0, "c0", "read", "b")),
	assembles("call-late-id", call(0, "", "read", "{}"), call(0, "c9", "", "")),
	assembles("call-empty-id-filled", call(0, "", "read", "{}"), call(2, "", "write", '{"a":1}')),
	assembles(
		"call-empty-id-filled-with-generation",
		serving({ generationId: "gen-7" }),
		call(0, "", "read", "{}"),
		call(3, "", "write", "{}"),
	),
	assembles("call-empty-id-filled-with-empty-generation", serving({ generationId: "" }), call(0, "", "read", "{}")),
	assembles(
		"call-blank-arguments",
		call(0, "a", "r", " \t\n"),
		call(1, "b", "r", ""),
		call(2, "c", "r", " {} "),
		call(3, "d", "r", "\u{a0}"),
	),
	assembles(
		"call-arguments-kept",
		call(0, "a", "r", "{not json"),
		call(1, "b", "r", "[1]"),
		call(2, "c", "r", " 0 "),
	),
	assembles("call-index-large", call(4294967296, "a", "r", "{}"), call(0, "b", "r", "{}")),
	assembles("call-index-order-is-arrival", call(5, "a", "r", "{}"), call(1, "b", "r", "{}"), call(3, "c", "r", "{}")),
	assembles("call-index-fraction", call(0, "a", "r", "{}"), call(0.5, "b", "r", "{}"), text("never")),
	assembles("call-index-negative", text("a"), call(-1, "a", "r", "{}")),
	assembles("call-index-negative-fraction", call(-0.5, "a", "r", "{}")),
	assembles("call-index-fraction-of-one", call(1.5, "a", "r", "{}")),
	assembles("call-index-shared", call(0, "call_a", "read", "{"), call(0, "call_b", "", "}"), text("never")),
	assembles(
		"call-index-shared-after-empty-id",
		call(0, "", "read", "{"),
		call(0, "call_b", "", "}"),
		call(0, "call_c", "", ""),
	),

	// Order and flags.
	assembles(
		"order-mixed",
		thought(part("t", [content("a")])),
		text("x"),
		call(0, "c0", "read", "{}"),
		text("y"),
		call(1, "c1", "read", "{}"),
		call(0, "", "", "!"),
		finish("tool_calls"),
	),
	assembles("seen-by-text", text("a")),
	assembles("seen-by-thought", thought(part("t"))),
	assembles("seen-by-call", call(0, "", "", "")),
	assembles("seen-not-by-metadata", serving({ host: "h" }), usage(1, 1), STOP),
	assembles(
		"added-flags",
		text("a"),
		text(""),
		thought(part("t")),
		call(0, "a", "r", "{}"),
		serving({ host: "h" }),
		usage(1, 1),
		STOP,
	),
	assembles(
		"failure-keeps-state",
		text("a"),
		serving({ host: "h" }),
		usage(1, 1),
		call(0.5, "a", "r", "{}"),
		text("b"),
	),

	// Readers of a result.
	replies("replies-empty", []),
	replies("replies-text-joins-everything", [
		{ kind: "text", text: "a" },
		{ kind: "thought", type: "t" },
		{ kind: "text", text: "b", phase: "final" },
		{ kind: "call", index: 0, wireId: "c0", name: "r", argumentsText: "{}" },
		{ kind: "text", text: "c" },
	]),
	replies("replies-calls-by-index", [
		{ kind: "call", index: 2, wireId: "c2", name: "r", argumentsText: "{}" },
		{ kind: "text", text: "x" },
		{ kind: "call", index: 0, wireId: "c0", name: "r", argumentsText: "{}" },
		{ kind: "call", index: 1, wireId: "c1", name: "r", argumentsText: "{}" },
	]),
	replies("replies-calls-ties-keep-order", [
		{ kind: "call", index: 1, wireId: "first", name: "r", argumentsText: "{}" },
		{ kind: "call", index: 0, wireId: "zero", name: "r", argumentsText: "{}" },
		{ kind: "call", index: 1, wireId: "second", name: "r", argumentsText: "{}" },
		{ kind: "call", index: 1, wireId: "third", name: "r", argumentsText: "{}" },
	]),
	replies("replies-thoughts-in-order", [
		{ kind: "thought", type: "a", segments: [{ kind: "content", text: "1" }] },
		{ kind: "text", text: "x" },
		{ kind: "thought", type: "b", opaque: "sig" },
	]),
];
