import { failed, failureOf, messageOf, ProviderError } from "@nyaaai/agentic/failure";
import { z } from "@nyaaai/contract/shape";
import { Failure, FailureCategory, NormalizedUsage } from "@nyaaai/contract/shapes/step";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { failureCases } from "../caseSources/failure";

/** Decoded API error. */
export const WireError = z.object({
	code: z.union([z.number(), z.string()]).exactOptional(),
	type: z.string().exactOptional(),
	message: z.string().exactOptional(),
	metadata: z.record(z.string(), z.unknown()).exactOptional(),
	resetsAt: z.union([z.number(), z.string()]).exactOptional(),
});

/** What `failed` takes beside the category and message. */
const FailureExtra = z.object({
	status: z.union([z.number(), z.string()]).exactOptional(),
	retryAfterMs: z.int().exactOptional(),
	resetsAt: z.int().exactOptional(),
	reasons: z.array(z.string()).exactOptional(),
	billed: z
		.object({
			usage: NormalizedUsage.exactOptional(),
			host: z.string().exactOptional(),
			generationId: z.string().exactOptional(),
		})
		.exactOptional(),
});

/** A thrown value, by how it is made. */
const Thrown = z.discriminatedUnion("make", [
	z.object({
		make: z.literal("failed"),
		category: FailureCategory,
		message: z.string(),
		extra: FailureExtra.exactOptional(),
	}),
	z.object({ make: z.literal("provider"), failure: Failure }),
	z.object({ make: z.literal("error"), message: z.string() }),
	z.object({ make: z.literal("value"), value: z.unknown() }),
]);

function thrownOf(held: z.output<typeof Thrown>): unknown {
	switch (held.make) {
		case "failed":
			return failed(held.category, held.message, held.extra);
		case "provider":
			return new ProviderError(held.failure);
		case "error":
			return new Error(held.message);
		case "value":
			return held.value;
	}
}

export const failure = caseGroup("failure", {
	shapes: { WireError, FailureExtra },
	kinds: {
		/** Null when blank or not text. */
		message: kind(
			z.object({ values: z.array(z.unknown()) }),
			z.object({ messages: z.array(z.string().nullable()) }),
			(input) => ({ messages: input.values.map((value) => messageOf(value) ?? null) }),
		),
		/** The failure a thrown value carries; null for one no codec decoded. */
		of: kind(
			z.object({ thrown: z.array(Thrown) }),
			z.object({ failures: z.array(Failure.nullable()) }),
			(input) => ({ failures: input.thrown.map((held) => failureOf(thrownOf(held)) ?? null) }),
		),
	},
	sources: casesFor("failure", failureCases),
});
