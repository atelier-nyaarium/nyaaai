// Contract generator test shapes.

import { preprocessed, refined, shapeSet } from "@nyaaai/contract/shape";
import { z } from "zod";

////////////////////////////////
//  Functions & Helpers

const isRecord = (raw: unknown): raw is Record<string, unknown> =>
	typeof raw === "object" && raw !== null && !Array.isArray(raw);

function renameLegacy(raw: unknown): unknown {
	if (!isRecord(raw)) return raw;
	const held = { ...raw };
	if (!("id" in held) && "name" in held) held.id = held.name;
	if (typeof held.tags === "string") held.tags = [held.tags];
	return held;
}

const trimmed = (raw: unknown): unknown => (typeof raw === "string" ? raw.trim() : raw);

////////////////////////////////
//  Shapes

export const Color = z.enum(["red", "green", "light-blue"]);
export type Color = z.infer<typeof Color>;

export const Name = z.string().min(1).max(3);
export type Name = z.infer<typeof Name>;

export const Names = z.array(Name).max(2);
export type Names = z.infer<typeof Names>;

export const Leaf = z.object({
	kind: z.literal("leaf"),
	value: z.int(),
	color: Color.exactOptional(),
});
export type Leaf = z.infer<typeof Leaf>;

export interface Branch {
	kind: "branch";
	kids: Tree[];
}
export type Tree = Leaf | Branch;

export const Branch: z.ZodType<Branch> = z.object({
	kind: z.literal("branch"),
	get kids() {
		return z.array(Tree);
	},
});

export const Tree: z.ZodType<Tree> = z.discriminatedUnion("kind", [Leaf, Branch as never]);

export interface Chain {
	value: number;
	next: Chain | null;
}

/** Recursive by-value shape. */
export const Chain: z.ZodType<Chain> = z.object({
	value: z.number(),
	get next() {
		return Chain.nullable();
	},
});

export const Presence = z.object({
	optional: z.string().exactOptional(),
	nullable: z.string().nullable(),
	both: z.string().nullable().exactOptional(),
	defaulted: z.int().default(3),
	defaultedNull: z.string().nullable().default(null),
	list: z.array(z.string()).default(["a"]),
	any: z.unknown().exactOptional(),
	must: z.unknown(),
});
export type Presence = z.infer<typeof Presence>;

export const Overlap = z.union([
	z.object({ a: z.string() }),
	z.object({ a: z.string(), b: z.number() }),
	Leaf,
	z.literal("auto"),
	z.string(),
	z.int(),
	z.number(),
	z.null(),
	z.array(z.boolean()),
]);
export type Overlap = z.infer<typeof Overlap>;

export const Open = z.object({
	counts: z.record(z.string(), z.int().min(0)),
	nested: z.record(z.string(), z.record(z.string(), Leaf)),
});
export type Open = z.infer<typeof Open>;

export const Bounded = z.object({
	name: Name,
	ratio: z.number().gt(0).lte(1),
	few: z.array(z.boolean()).min(1).max(2),
	level: z.int().min(-2).max(2),
	count: z.int().min(0),
});
export type Bounded = z.infer<typeof Bounded>;

export const Literals = z.object({
	yes: z.literal(true),
	five: z.literal(5),
	nothing: z.null(),
	word: z.literal("word"),
	maybe: z.literal("x").exactOptional(),
});
export type Literals = z.infer<typeof Literals>;

export const Outer = z.object({
	inner: z.object({
		deep: z.object({ x: z.int() }).nullable(),
	}),
	list: z.array(z.object({ color: Color, names: Names })),
	type: z.enum(["struct", "enum"]),
});
export type Outer = z.infer<typeof Outer>;

export const Event = z.discriminatedUnion("type", [
	z.object({ type: z.literal("text"), text: z.string() }),
	z.object({ type: z.literal("tool-call"), id: z.string(), input: z.unknown() }),
	z.object({ type: z.literal("tree"), tree: Tree }),
]);
export type Event = z.infer<typeof Event>;

export const Legacy = preprocessed(
	"renameLegacy",
	renameLegacy,
	z.object({ id: z.string(), tags: z.array(z.string()) }),
);
export type Legacy = z.infer<typeof Legacy>;

export const Span = refined("ordered", z.object({ from: z.int(), to: z.int() }), (span) => span.from <= span.to);
export type Span = z.infer<typeof Span>;

export const Holder = z.object({
	legacy: Legacy,
	span: Span.nullable(),
	label: preprocessed("trimmed", trimmed, z.string().min(1)),
	spans: z.array(Span),
});
export type Holder = z.infer<typeof Holder>;

export const tortureSet = shapeSet("torture", {
	Color,
	Name,
	Names,
	Leaf,
	Branch,
	Tree,
	Chain,
	Presence,
	Overlap,
	Open,
	Bounded,
	Literals,
	Outer,
	Event,
	Legacy,
	Span,
	Holder,
});
