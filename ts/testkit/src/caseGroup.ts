// Case kind groups.

import type { Ids } from "@nyaaai/contract/id";
import type { AnyShape, Input, Output } from "@nyaaai/contract/shape";
import type { Clock } from "@nyaaai/contract/time";

////////////////////////////////
//  Interfaces & Types

type Awaitable<T> = T | Promise<T>;

export interface CaseContext {
	/** Fresh case folder. */
	tmp(): string;
	/** Read-only project root. */
	root: string;
	clock: Clock;
	ids: Ids;
}

export interface CaseKind<In extends AnyShape = AnyShape, Out extends AnyShape = AnyShape> {
	input: In;
	result: Out;
	/** Context only when declared. */
	run: (input: Output<In>, context: CaseContext) => Awaitable<Input<Out>>;
	/** Enables context and path marking. */
	context: boolean;
}

export interface CaseSource {
	/** `<group>.<kind>`. */
	kind: string;
	name: string;
	input: unknown;
}

export interface CaseGroup {
	name: string;
	/** Shared named shapes. */
	shapes: Record<string, AnyShape>;
	kinds: Record<string, CaseKind>;
	sources: CaseSource[];
}

////////////////////////////////
//  Functions & Helpers

export function kind<In extends AnyShape, Out extends AnyShape>(
	input: In,
	result: Out,
	run: (input: Output<In>) => Awaitable<Input<Out>>,
): CaseKind;
export function kind<In extends AnyShape, Out extends AnyShape>(
	input: In,
	result: Out,
	run: CaseKind<In, Out>["run"],
	options: { context: true },
): CaseKind;
export function kind<In extends AnyShape, Out extends AnyShape>(
	input: In,
	result: Out,
	run: CaseKind<In, Out>["run"],
	options: { context?: true } = {},
): CaseKind {
	return { input, result, run: run as CaseKind["run"], context: options.context ?? false };
}

export function caseGroup(
	name: string,
	parts: { shapes?: Record<string, AnyShape>; kinds: Record<string, CaseKind>; sources?: CaseSource[] },
): CaseGroup {
	const sources = parts.sources ?? [];
	for (const source of sources) {
		const key = source.kind.slice(name.length + 1);
		if (source.kind !== `${name}.${key}` || !Object.hasOwn(parts.kinds, key))
			throw new Error(`group ${name} holds a case of kind ${source.kind}`);
	}
	return { name, shapes: parts.shapes ?? {}, kinds: parts.kinds, sources };
}

export const casesFor = (group: string, ...lists: CaseSource[][]): CaseSource[] =>
	lists.flat().filter((source) => source.kind.startsWith(`${group}.`));

const pascal = (name: string): string => name.slice(0, 1).toUpperCase() + name.slice(1);

/** Keys use `<group>.<kind>`. */
export function kindsOf(groups: CaseGroup[]): Map<string, CaseKind> {
	const kinds = new Map<string, CaseKind>();
	for (const group of groups) {
		for (const [key, held] of Object.entries(group.kinds)) {
			const name = `${group.name}.${key}`;
			if (kinds.has(name)) throw new Error(`two kinds are named ${name}`);
			kinds.set(name, held);
		}
	}
	return kinds;
}

export function caseShapes(groups: CaseGroup[]): Record<string, AnyShape> {
	const shapes: Record<string, AnyShape> = {};
	const add = (name: string, shape: AnyShape): void => {
		if (Object.hasOwn(shapes, name) && shapes[name] !== shape) throw new Error(`two case shapes are named ${name}`);
		shapes[name] = shape;
	};
	for (const group of groups) {
		for (const [name, shape] of Object.entries(group.shapes)) add(name, shape);
		for (const [key, held] of Object.entries(group.kinds)) {
			add(`${pascal(group.name)}${pascal(key)}Input`, held.input);
			add(`${pascal(group.name)}${pascal(key)}Result`, held.result);
		}
	}
	return shapes;
}
