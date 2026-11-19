import { checkKey } from "@nyaaai/agentic/kinds/openrouter/key";
import type { KeyAuth } from "@nyaaai/agentic/transport";
import { jsString } from "@nyaaai/contract/json";
import { z } from "@nyaaai/contract/shape";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { kindKeyCases } from "../caseSources/kindKey";
import { abortOptions, BASE_URL, callerSignal, NetExchange, networkOf } from "./catalogKit";

////////////////////////////////
//  Group

export const kindGroup = caseGroup("kind", {
	shapes: { NetExchange },
	kinds: {
		/** `checkKey` over a scripted network; each check reads one reply. */
		openrouterKey: kind(
			z.object({
				startMs: z.int(),
				exchanges: z.array(NetExchange),
				/** The stored key; null is none. */
				stored: z.string().nullable(),
				checks: z.array(
					z.discriminatedUnion("auth", [
						z.object({ auth: z.literal("stored"), ...abortOptions }),
						z.object({ auth: z.literal("key"), key: z.string(), ...abortOptions }),
					]),
				),
			}),
			z.object({
				checks: z.array(
					z.object({
						/** Null when the check threw. */
						ok: z.boolean().nullable(),
						reason: z.enum(["rejected", "unreachable"]).nullable(),
						usage: z.number().nullable(),
						limit: z.number().nullable(),
						reset: z.string().nullable(),
						thrown: z.string().nullable(),
					}),
				),
				/** Paths asked with the authorization sent, in order. */
				requests: z.array(z.object({ path: z.string(), authorization: z.string().nullable() })),
				waiting: z.array(z.string()),
			}),
			async (input) => {
				const net = networkOf(input.startMs, input.exchanges, input.stored ?? undefined);
				const checks: Array<{
					ok: boolean | null;
					reason: "rejected" | "unreachable" | null;
					usage: number | null;
					limit: number | null;
					reset: string | null;
					thrown: string | null;
				}> = [];
				let tags = 0;
				for (const held of input.checks) {
					tags += 1;
					const signal = callerSignal(net.wheel, `caller-${tags}`, held);
					const auth: KeyAuth = held.auth === "stored" ? "stored" : { key: held.key };
					try {
						const result = await checkKey(net.transport, auth, signal);
						checks.push(
							result.ok
								? {
										ok: true,
										reason: null,
										usage: result.usage ?? null,
										limit: result.limit ?? null,
										reset: result.reset ?? null,
										thrown: null,
									}
								: {
										ok: false,
										reason: result.reason,
										usage: null,
										limit: null,
										reset: null,
										thrown: null,
									},
						);
					} catch (error) {
						checks.push({
							ok: null,
							reason: null,
							usage: null,
							limit: null,
							reset: null,
							thrown: error instanceof Error ? error.message : jsString(error),
						});
					}
					net.wheel.cancel(`caller-${tags}`);
				}
				return {
					checks,
					requests: net.played.requests.map((sent) => ({
						path: sent.url.slice(BASE_URL.length),
						authorization: sent.headers.find(([name]) => name === "Authorization")?.[1] ?? null,
					})),
					waiting: net.wheel.waiting(),
				};
			},
		),
	},
	sources: casesFor("kind", kindKeyCases),
});
