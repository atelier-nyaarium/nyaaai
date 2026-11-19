import { providerMeter } from "@nyaaai/agentic/usage/meter";
import { z } from "@nyaaai/contract/shape";
import { MeteredCall } from "@nyaaai/contract/shapes/usage";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { scriptedUsageSink } from "@nyaaai/testkit/scriptedPorts";
import { meterCases } from "../caseSources/meter";

////////////////////////////////
//  Shapes

/** What the health was told of a reply's billing. */
const MeterBilled = z.object({
	entry: z.string(),
	source: z.string(),
	allowance: z.number().exactOptional(),
	cost: z.number().exactOptional(),
});

////////////////////////////////
//  Group

export const meter = caseGroup("meter", {
	shapes: { MeterBilled },
	kinds: {
		/** Calls in order into a meter over a scripted usage sink, a health that records and a stats hook. */
		called: kind(
			z.object({
				calls: z.array(
					z.object({
						call: MeteredCall,
						/** The id the sink answers this call; null is none. */
						id: z.string().nullable(),
					}),
				),
			}),
			z.object({
				calls: z.array(
					z.object({
						/** What the usage sink was given. */
						recorded: MeteredCall,
						/** What the health's `billed` was given, or null when it was not called. */
						billed: MeterBilled.nullable(),
						/** What the stats hook was given, or null when it was not called. */
						stats: z.object({ id: z.string(), generationId: z.string(), call: MeteredCall }).nullable(),
					}),
				),
			}),
			(input) => {
				const sink = scriptedUsageSink();
				let billed: z.infer<typeof MeterBilled> | null = null;
				let stats: { id: string; generationId: string; call: z.infer<typeof MeteredCall> } | null = null;
				const meter = providerMeter({
					usage: sink,
					health: {
						billed: (entry, reply) => {
							billed = { entry, ...reply };
						},
					},
					stats: (id, generationId, call) => {
						stats = { id, generationId, call: structuredClone(call) };
					},
				});
				const calls = [];
				for (const each of input.calls) {
					sink.setIds(each.id === null ? [] : [each.id]);
					billed = null;
					stats = null;
					meter.called(each.call);
					const recorded = sink.recorded().at(-1);
					if (recorded === undefined) throw new Error("The sink was given nothing.");
					calls.push({ recorded, billed, stats });
				}
				return { calls };
			},
		),
	},
	sources: casesFor("meter", meterCases),
});
