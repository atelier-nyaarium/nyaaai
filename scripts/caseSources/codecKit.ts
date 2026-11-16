// Case builders shared by the codec sources.

import type { CaseSource } from "../caseAdapter";
import { type Json, request } from "./codecFixtures";

////////////////////////////////
//  Functions & Helpers

export const encodes = (
	name: string,
	codec: Json,
	over: Json = {},
	mode: "stream" | "complete" = "stream",
): CaseSource => ({
	kind: "codec.encode",
	name,
	input: { codec, request: request(over), mode },
});

export const streams = (name: string, codec: Json, ...items: Json[]): CaseSource => ({
	kind: "codec.stream",
	name,
	input: { codec, items },
});

export const completes = (name: string, codec: Json, body: unknown): CaseSource => ({
	kind: "codec.complete",
	name,
	input: { codec, body },
});

export const fails = (
	name: string,
	codec: Json,
	status: number,
	body: unknown,
	headers: string[][] = [],
): CaseSource => ({
	kind: "codec.failure",
	name,
	input: { codec, status, body, headers },
});
