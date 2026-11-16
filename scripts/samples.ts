// Schema sampler parity.
// Run: bun scripts/samples.ts

import { exportSchemas, type Json, type SetSchema } from "@nyaaai/contract/schemaExport";
import { decodeText, shapeSets } from "@nyaaai/contract/shape";
import { fullValue, sampleTexts } from "@nyaaai/testkit/samples";
import { runCommands, rustAdapter } from "./adapterHost";
import { CASE_PREFIX, runCase } from "./caseAdapter";
import { loadProject } from "./project";

////////////////////////////////
//  Interfaces & Types

interface Difference {
	set: string;
	shape: string;
	text: string;
	typescript: string;
	rust: string;
}

////////////////////////////////
//  Constants

const SHOWN = 20;
const TEXT_CUT = 200;
/** Texts in one request. */
const PART = 2000;

////////////////////////////////
//  Functions & Helpers

const cut = (text: string): string => (text.length > TEXT_CUT ? `${text.slice(0, TEXT_CUT)}...` : text);

const resultsOf = (held: unknown): unknown[] | undefined => {
	const results = (held as { results?: unknown } | null)?.results;
	return Array.isArray(results) ? results : undefined;
};

////////////////////////////////
//  Main

if (import.meta.main) {
	const project = await loadProject();
	const kind = project.roundTrip;
	if (kind === undefined) {
		console.log("samples: nothing to sample, the project names no round-trip kind");
		process.exit(0);
	}
	const started = Date.now();
	const failures: string[] = [];
	const { docs } = exportSchemas();
	const inputs = new Map<string, SetSchema>([...docs].map(([set, forms]) => [set, forms.input]));
	const seeds = new Map<string, string[]>();
	for (const group of project.groups) {
		for (const source of group.sources.filter((held) => held.kind === kind)) {
			const { set, shape, texts } = source.input as { set: string; shape: string; texts: string[] };
			seeds.set(`${set}.${shape}`, [...(seeds.get(`${set}.${shape}`) ?? []), ...texts]);
		}
	}
	const jobs: Array<{ run: string; set: string; shape: string; texts: string[] }> = [];
	let shapes = 0;
	for (const set of Object.keys(project.sets)) {
		const input = inputs.get(set);
		if (!input) {
			failures.push(`${set}: the set has no input schema document`);
			continue;
		}
		const zod = shapeSets().find((held) => held.name === set)?.shapes ?? {};
		for (const shape of Object.keys(input.$defs)) {
			const name = `${set}.${shape}`;
			const schema = zod[shape];
			const decodes = (text: string): boolean => schema !== undefined && decodeText(schema, text).ok;
			const built = fullValue(inputs, set, shape);
			let base: Json | undefined;
			if (built === undefined || !decodes(JSON.stringify(built))) {
				const seed = (seeds.get(name) ?? []).find(decodes);
				if (seed === undefined) {
					failures.push(`${name}: no value decodes to mutate; add a round-trip case that decodes`);
					continue;
				}
				base = JSON.parse(seed) as Json;
			}
			const texts = sampleTexts(inputs, set, shape, base);
			shapes += 1;
			for (let at = 0; at === 0 || at < texts.length; at += PART) {
				jobs.push({ run: `${name}/${at / PART}`, set, shape, texts: texts.slice(at, at + PART) });
			}
		}
	}
	const rust = await runCommands(
		rustAdapter(project),
		jobs.map(({ run, ...input }) => ({ run, command: `${CASE_PREFIX}${kind}`, input })),
	);
	const differences: Difference[] = [];
	let values = 0;
	for (const { run, ...job } of jobs) {
		const theirs = rust.get(run);
		const mine = await runCase(project, kind, job).then(
			(held) => ({ held }),
			(error: unknown) => ({ error: error instanceof Error ? error.message : String(error) }),
		);
		const rows = "held" in mine ? resultsOf(mine.held) : undefined;
		const others = theirs && "result" in theirs ? resultsOf(theirs.result) : undefined;
		if (rows?.length !== job.texts.length || others?.length !== job.texts.length) {
			const show = (side: unknown): string => cut(JSON.stringify(side) ?? "none");
			differences.push({
				set: job.set,
				shape: job.shape,
				text: `(part ${run}, ${job.texts.length} texts sent)`,
				typescript: show("held" in mine ? mine.held : mine),
				rust: show(theirs),
			});
			continue;
		}
		values += job.texts.length;
		job.texts.forEach((text, at) => {
			const one = JSON.stringify(rows[at]);
			const two = JSON.stringify(others[at]);
			if (one !== two) {
				differences.push({ set: job.set, shape: job.shape, text: cut(text), typescript: one, rust: two });
			}
		});
	}
	for (const failure of failures) console.error(`samples: ${failure}`);
	for (const found of differences.slice(0, SHOWN)) {
		console.error(`${found.set}.${found.shape}: ${JSON.stringify(found.text)}`);
		console.error(`  TypeScript ${found.typescript}`);
		console.error(`  Rust       ${found.rust}`);
	}
	if (differences.length > SHOWN) console.error(`... ${differences.length - SHOWN} more differences`);
	console.log(`samples: ran in ${Date.now() - started} ms`);
	const ok = differences.length === 0 && failures.length === 0;
	console.log(ok ? `samples: ${values} values over ${shapes} shapes agree in both languages` : "samples: FAILED");
	process.exit(ok ? 0 : 1);
}
