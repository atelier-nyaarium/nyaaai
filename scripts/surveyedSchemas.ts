import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import type { Json } from "@nyaaai/contract/schemaExport";
import { type Issue, normalizeSchema, schemaProblems } from "@nyaaai/contract/strictSchema";
import { runCommands, rustAdapter } from "./adapterHost";

interface ProblemResult {
	problems: Issue[][];
}

function filesUnder(folder: string): string[] {
	return readdirSync(folder, { withFileTypes: true })
		.flatMap((entry) => {
			const file = path.join(folder, entry.name);
			return entry.isDirectory()
				? filesUnder(file)
				: entry.isFile() && entry.name.endsWith(".json")
					? [file]
					: [];
		})
		.sort();
}

if (import.meta.main) {
	const folder = process.argv[2];
	if (folder === undefined) throw new Error("usage: bun scripts/surveyedSchemas.ts <folder>");
	const files = filesUnder(folder);
	const schemas = files.map((file) => JSON.parse(readFileSync(file, "utf8")) as Json);
	const checked = schemas.map((schema) => schemaProblems(normalizeSchema(schema)));
	const rust = await runCommands(
		rustAdapter(),
		schemas.map((schema, index) => ({
			run: String(index),
			command: "case:strictSchema.problems",
			input: { schemas: [schema] },
		})),
	);
	let acceptedRust = 0;
	let differing = 0;
	for (const [index, file] of files.entries()) {
		const result = rust.get(String(index));
		const other =
			result !== undefined && "result" in result ? (result.result as ProblemResult).problems[0] : undefined;
		if (other?.length === 0) acceptedRust += 1;
		const same = other !== undefined && JSON.stringify(checked[index]) === JSON.stringify(other);
		if (!same) differing += 1;
		if (checked[index]?.length !== 0 || other?.length !== 0) {
			const codes = (problems: Issue[] | undefined) => problems?.map(({ code }) => code).join(",") ?? "failed";
			console.log(`${file}: TypeScript=${codes(checked[index])}; Rust=${codes(other)}`);
		}
	}
	const acceptedTypeScript = checked.filter((problems) => problems.length === 0).length;
	console.log(
		`surveyed schemas: files=${files.length} accepted-ts=${acceptedTypeScript} accepted-rust=${acceptedRust} differing=${differing}`,
	);
	process.exit(differing === 0 && acceptedTypeScript === files.length && acceptedRust === files.length ? 0 : 1);
}
