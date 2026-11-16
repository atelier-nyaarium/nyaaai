import type { CaseSource } from "../caseAdapter";

const LADDER = ["none", "minimal", "low", "medium", "high", "xhigh", "max"];

/** Every rung, then names off the ladder. */
const SWEEP = [...LADDER, "turbo", ""];

const resolves = (name: string, offered: string[], wanted: string[] = SWEEP): CaseSource => ({
	kind: "effort.resolve",
	name,
	input: { offered, wanted },
});

export const effortCases: CaseSource[] = [
	{ kind: "effort.ladder", name: "ladder", input: {} },
	resolves("listed-gaps", ["low", "high", "xhigh"]),
	resolves("whole-ladder", LADDER),
	resolves("nothing-offered", []),
	resolves("one-rung", ["medium"]),
	resolves("only-max", ["max"]),
	resolves("max-and-low", ["max", "low"]),
	resolves("unordered", ["xhigh", "low", "high"]),
	resolves("duplicates", ["low", "low", "high", "high"]),
	resolves("off-ladder-offered", ["fast", "low", "high"], [...SWEEP, "fast"]),
	resolves("only-off-ladder", ["fast", "slow"], [...SWEEP, "fast"]),
	resolves("case-matters", ["Low", "HIGH", "low"], ["low", "Low", "high", "HIGH"]),
	resolves("no-wanted", ["low", "high"], []),
];
