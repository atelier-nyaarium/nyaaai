// Holds the curated list against the live catalog. Run by hand; exits 1 on a retired or non-zero-retention model.

import { platformFetch } from "@nyaaai/agentic/fetch";
import {
	joinCatalog,
	parseEndpoints,
	parseModels,
	sentParams,
	usableHosts,
} from "@nyaaai/agentic/kinds/openrouter/catalog";
import { OPENROUTER, OPENROUTER_ENTRY } from "@nyaaai/agentic/kinds/openrouter/kind";
import { MODELS } from "@nyaaai/agentic/models/curated";
import { CHAT } from "@nyaaai/agentic/models/uses";
import { fixedText, numberOfText, numberText } from "@nyaaai/contract/number";
import type { ModelRow, ModelView } from "@nyaaai/contract/shapes/model";
import { deadline, systemScheduler } from "@nyaaai/contract/time";

////////////////////////////////
//  Interfaces & Types

/** What the check found. */
export interface Report {
	/** One line per curated row, then one per row with newer models. */
	lines: string[];
	/** A curated model is retired or has no zero-retention host. */
	failed: boolean;
}

////////////////////////////////
//  Constants

const FETCH_MS = 30_000;

////////////////////////////////
//  Functions & Helpers

/** `*` stands for any run of characters; every other character stands for itself. */
export function globMatch(glob: string, text: string): boolean {
	const parts = glob.split("*");
	const [first = "", ...rest] = parts;
	if (rest.length === 0) return text === first;
	const last = rest[rest.length - 1] ?? "";
	if (!text.startsWith(first) || text.length < first.length + last.length || !text.endsWith(last)) return false;
	const end = text.length - last.length;
	let at = first.length;
	for (const part of rest.slice(0, -1)) {
		const found = text.indexOf(part, at);
		if (found < 0 || found + part.length > end) return false;
		at = found + part.length;
	}
	return at <= end;
}

function dollars(value: number): string {
	const text = fixedText(value, 4);
	const rounded = numberOfText(text);
	return `$${rounded === undefined ? text : numberText(rounded)}`;
}

function range([low, high]: readonly [number, number]): string {
	return low === high ? dollars(low) : `${dollars(low)}-${dollars(high)}`;
}

function describe(view: ModelView): string {
	const verified = view.row.verifiedPrice;
	const parts = [view.row.key, view.status, `${view.hosts.length} hosts`];
	const { price } = view;
	if (typeof price === "object") {
		const moved = price.prompt[0] > verified.prompt || price.completion[0] > verified.completion ? " (up)" : "";
		parts.push(
			`${range(price.prompt)} in, ${range(price.completion)} out per M${moved}; verified ${dollars(verified.prompt)} in, ${dollars(verified.completion)} out`,
		);
	}
	if (view.retiredOn !== undefined) parts.push(`removal ${view.retiredOn}`);
	return parts.join("  ");
}

/** What the catalog's two bodies say of the rows; none when they do not read. */
export function reportOf(models: unknown, zdr: unknown, rows: readonly ModelRow[], nowMs: number): Report | undefined {
	const entries = parseModels(models);
	const endpoints = parseEndpoints(zdr);
	if (entries === undefined || endpoints === undefined) return undefined;
	const views = joinCatalog(rows, models, zdr, nowMs);
	const lines = views.map(describe);
	const known = new Set(rows.flatMap((row) => row.bindings["openrouter"]?.id ?? []));
	for (const row of rows) {
		const id = row.bindings["openrouter"]?.id;
		if (id === undefined) continue;
		const created = entries.get(id)?.created ?? undefined;
		if (created === undefined) {
			lines.push(`${row.key}: its date is unknown, newer models are not listed`);
			continue;
		}
		const newer = [...entries.values()]
			.filter(
				(entry) =>
					!known.has(entry.id) &&
					globMatch(row.family, entry.id) &&
					(entry.created ?? 0) > created &&
					usableHosts(endpoints.get(entry.id) ?? [], sentParams(row, CHAT)).length > 0,
			)
			.map((entry) => entry.id);
		if (newer.length > 0) lines.push(`${row.key}: newer with a zero-retention host: ${newer.join(", ")}`);
	}
	return { lines, failed: views.some((view) => view.status === "retired" || view.status === "noZdrHost") };
}

////////////////////////////////
//  Main

if (import.meta.main) {
	const transport = OPENROUTER.transport(
		OPENROUTER_ENTRY,
		{ get: async () => undefined },
		{ fetch: platformFetch(), scheduler: systemScheduler, log: () => {}, userAgent: "nyaaai-models-check/1" },
	);
	const limit = deadline(systemScheduler, FETCH_MS);
	const [models, zdr] = await Promise.all([
		transport.getJson("/models", limit.signal, "none"),
		transport.getJson("/endpoints/zdr", limit.signal, "none"),
	]);
	limit.clear();
	const report = reportOf(models, zdr, MODELS, systemScheduler.nowMs());
	if (report === undefined) {
		console.error("The catalog did not parse.");
		process.exit(1);
	}
	for (const line of report.lines) console.log(line);
	process.exit(report.failed ? 1 : 0);
}
