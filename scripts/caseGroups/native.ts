import { failureOf } from "@nyaaai/agentic/failure";
import {
	argumentsOf,
	assembled,
	currentTurnFrom,
	type Host,
	isJson,
	numberOf,
	parsedChunk,
	schemaPrompt,
	servingEvents,
	streamFailure,
	stringOf,
	systemTexts,
	wireError,
} from "@nyaaai/agentic/native";
import { parseJson } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import {
	type CanonicalRequest,
	Failure,
	NormalizedUsage,
	Serving,
	StepEvent,
	StepResult,
	Turn,
} from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { nativeCases } from "../caseSources/native";
import { WireError } from "./failure";

////////////////////////////////
//  Functions & Helpers

/** JSON text read as the codecs read provider JSON. */
function jsonOf(text: string): unknown {
	const value = parseJson(text);
	if (value === undefined) throw new Error("a value text is not JSON");
	return value;
}

const record = () => z.record(z.string(), z.unknown());

/** A host that only names where its failure came from. */
const HOST: Host = {
	prefix: "/v1",
	headers: () => ({}),
	usage: (own) => own,
	failure: (status, error) => ({ category: "stream", status, message: error.message ?? "" }),
};

function requestOf(system: string[], summary: string | null): CanonicalRequest {
	return {
		model: "m-1",
		system,
		...(summary === null ? {} : { summary }),
		history: [],
		cache: { kind: "none" },
		session: "s1",
		zdrOnly: false,
	};
}

export const native = caseGroup("native", {
	kinds: {
		/** The readers of loose provider JSON, over JSON texts. */
		readers: kind(
			z.object({ texts: z.array(z.string()) }),
			z.object({
				rows: z.array(z.object({ isJson: z.boolean(), string: z.string(), number: z.number().nullable() })),
			}),
			(input) => ({
				rows: input.texts.map((text) => {
					const value = jsonOf(text);
					return { isJson: isJson(value), string: stringOf(value), number: numberOf(value) ?? null };
				}),
			}),
		),
		/** Key order of the object decides the text. */
		schemaPrompt: kind(
			z.object({ schemas: z.array(record()) }),
			z.object({ texts: z.array(z.string()) }),
			(input) => ({ texts: input.schemas.map(schemaPrompt) }),
		),
		systemTexts: kind(
			z.object({
				rows: z.array(
					z.object({
						system: z.array(z.string()),
						summary: z.string().nullable(),
						prompted: z.string().nullable(),
					}),
				),
			}),
			z.object({ texts: z.array(z.array(z.string())) }),
			(input) => ({
				texts: input.rows.map((row) =>
					systemTexts(requestOf(row.system, row.summary), row.prompted ?? undefined),
				),
			}),
		),
		/** Malformed text and non-objects become an empty object. */
		arguments: kind(z.object({ texts: z.array(z.string()) }), z.object({ values: z.array(record()) }), (input) => ({
			values: input.texts.map(argumentsOf),
		})),
		/** The index after each history's latest user turn. */
		currentTurn: kind(
			z.object({ histories: z.array(z.array(Turn)) }),
			z.object({ indexes: z.array(z.int()) }),
			(input) => ({ indexes: input.histories.map(currentTurnFrom) }),
		),
		chunk: kind(
			z.object({ texts: z.array(z.string()) }),
			z.object({ rows: z.array(z.object({ chunk: record().nullable(), failure: Failure.nullable() })) }),
			(input) => ({
				rows: input.texts.map((text) => {
					try {
						return { chunk: parsedChunk(text), failure: null };
					} catch (error) {
						const failure = failureOf(error);
						if (failure === undefined) throw error;
						return { chunk: null, failure };
					}
				}),
			}),
		),
		/** Bodies are JSON texts. */
		wireError: kind(
			z.object({ rows: z.array(z.object({ status: z.int(), body: z.string() })) }),
			z.object({ errors: z.array(WireError) }),
			(input) => ({ errors: input.rows.map((row) => wireError(row.status, jsonOf(row.body))) }),
		),
		/** The host answers a stream failure with its error's message; billing rides along. */
		streamFailure: kind(
			z.object({ error: WireError, usage: NormalizedUsage.nullable(), serving: Serving }),
			z.object({ failure: Failure }),
			(input) => ({ failure: streamFailure(HOST, input.error, input.usage ?? undefined, input.serving).failure }),
		),
		/** Whole-reply events; a step with no finish is a failure. */
		assembled: kind(
			z.object({ events: z.array(StepEvent) }),
			z.object({ result: StepResult.nullable(), failure: Failure.nullable() }),
			(input) => {
				try {
					return { result: assembled(input.events), failure: null };
				} catch (error) {
					const failure = failureOf(error);
					if (failure === undefined) throw error;
					return { result: null, failure };
				}
			},
		),
		/** Only a non-empty text id names a serving. */
		serving: kind(
			z.object({ ids: z.array(z.unknown()) }),
			z.object({ events: z.array(z.array(StepEvent)) }),
			(input) => ({ events: input.ids.map(servingEvents) }),
		),
	},
	sources: casesFor("native", nativeCases),
});
