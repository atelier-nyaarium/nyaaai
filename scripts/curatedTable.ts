// The curated model list as data for Rust.

import { MODELS } from "@nyaaai/agentic/models/curated";
import { parseJson, stringifyJson } from "@nyaaai/contract/json";
import { decode } from "@nyaaai/contract/shape";
import { ModelRow } from "@nyaaai/contract/shapes/model";
import { rustModule, rustText } from "./rustConstants";

////////////////////////////////
//  Functions & Helpers

/** Each row survives the shape and the text; else the message names the row. */
function checked(): string {
	const rows = MODELS.map((row) => {
		const text = stringifyJson(row);
		const read = decode(ModelRow, parseJson(text));
		if (!read.ok)
			throw new Error(
				`The curated row ${row.key} does not fit ModelRow at ${JSON.stringify(read.path)}: ${read.message}`,
			);
		if (stringifyJson(read.value) !== text)
			throw new Error(`The curated row ${row.key} changes when read as a ModelRow.`);
		return text;
	});
	return `[${rows.join(",")}]`;
}

/** One JSON text: an array of `ModelRow`, which Rust reads through the shape. */
export const curatedTableRust = (): string =>
	rustModule("The curated model list.", [
		rustText("CURATED_JSON", checked(), "A JSON array of `ModelRow`, in picker order."),
	]);
