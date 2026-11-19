// The curated model list as data for Rust.

import { MODELS } from "@nyaaai/agentic/models/curated";
import { stringifyJson } from "@nyaaai/contract/json";
import { rustModule, rustText } from "./rustConstants";

////////////////////////////////
//  Functions & Helpers

/** One JSON text: an array of `ModelRow`, which Rust reads through the shape. */
export const curatedTableRust = (): string =>
	rustModule("The curated model list.", [
		rustText("CURATED_JSON", stringifyJson(MODELS), "A JSON array of `ModelRow`, in picker order."),
	]);
