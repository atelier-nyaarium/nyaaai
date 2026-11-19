// Model request metering: where a call's record, bill and stats go.

import type { Ticket } from "@nyaaai/contract/shapes/health";
import type { CallTag, MeteredCall } from "@nyaaai/contract/shapes/usage";
import type { UsageSink } from "@nyaaai/contract/usageSink";
import type { Billing, ProviderHealth } from "../health/health";

////////////////////////////////
//  Interfaces & Types

/** A feature id; unlabeled ones show by id. */
export type Feature = string;

export interface Meter {
	/** The ticket the request held, if the gate gave one. */
	called(call: MeteredCall, ticket?: Ticket): void;
}

export interface MeterDeps {
	/** Where the call is written; the id it answers names the record. */
	usage: Pick<UsageSink, "record">;
	health: Pick<ProviderHealth, "billed">;
	/** Queues delayed stats for a record that names a generation. */
	stats(id: string, generationId: string, call: MeteredCall): void;
}

////////////////////////////////
//  Constants

export const KNOWN_FEATURES = ["chat", "analysis", "wand", "tester"] as const;

////////////////////////////////
//  Functions & Helpers

export function onBehalfOf(tag: CallTag, lane: "title" | "summary"): CallTag {
	return { feature: "analysis", lane, ...(tag.folder === undefined ? {} : { folder: tag.folder }) };
}

/** Records calls and billing. */
export function providerMeter(d: MeterDeps): Meter {
	return {
		called: (call, ticket) => {
			const id = d.usage.record(call);
			if (id !== undefined && call.generationId !== undefined) d.stats(id, call.generationId, call);
			const reply = call.usage;
			const source = reply?.billingSource;
			if (call.entry !== undefined && source !== undefined) {
				const billing: Billing = { source };
				if (reply?.allowance !== undefined) billing.allowance = reply.allowance;
				if (reply?.cost !== undefined) billing.cost = reply.cost;
				d.health.billed(call.entry, billing, ticket);
			}
		},
	};
}
