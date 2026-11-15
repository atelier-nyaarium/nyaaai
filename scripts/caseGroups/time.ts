import { z } from "@nyaaai/contract/shape";
import { dayKey, isoTime, parseHttpDate, parseIsoTime, parseRetryAfter, resetStampMs } from "@nyaaai/contract/time";
import { caseGroup, casesFor, kind } from "@nyaaai/testkit/caseGroup";
import { timeCases } from "../caseSources/time";

const times = () => z.object({ results: z.array(z.int().nullable()) });

export const time = caseGroup("time", {
	kinds: {
		parseIso: kind(z.object({ texts: z.array(z.string()) }), times(), (input) => ({
			results: input.texts.map((text) => {
				const at = parseIsoTime(text);
				return Number.isNaN(at) ? null : at;
			}),
		})),
		iso: kind(
			z.object({ times: z.array(z.int()) }),
			z.object({ texts: z.array(z.string().nullable()) }),
			(input) => ({ texts: input.times.map((ms) => isoTime(ms) ?? null) }),
		),
		httpDate: kind(z.object({ texts: z.array(z.string()), nowMs: z.int() }), times(), (input) => ({
			results: input.texts.map((text) => parseHttpDate(text, input.nowMs) ?? null),
		})),
		retryAfter: kind(z.object({ headers: z.array(z.string().nullable()), nowMs: z.int() }), times(), (input) => ({
			results: input.headers.map((header) => parseRetryAfter(header, input.nowMs) ?? null),
		})),
		resetStamp: kind(z.object({ values: z.array(z.unknown()), nowMs: z.int() }), times(), (input) => ({
			results: input.values.map((value) => resetStampMs(value, input.nowMs) ?? null),
		})),
		dayKey: kind(
			z.object({ times: z.array(z.int()), offsets: z.array(z.int()) }),
			z.object({ rows: z.array(z.array(z.string().nullable())) }),
			(input) => ({ rows: input.times.map((ms) => input.offsets.map((offset) => dayKey(ms, offset) ?? null)) }),
		),
	},
	sources: casesFor("time", timeCases),
});
