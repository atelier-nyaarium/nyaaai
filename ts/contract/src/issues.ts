import type { PathPart } from "./shape";

export const ISSUE_MESSAGES = {
	type: "expected {types}",
	enum: "not in enum",
	const: "not equal to const",
	minimum: "below {bound}",
	exclusive_minimum: "not above {bound}",
	maximum: "above {bound}",
	exclusive_maximum: "not below {bound}",
	min_length: "shorter than {bound}",
	max_length: "longer than {bound}",
	min_items: "fewer than {bound}",
	max_items: "more than {bound}",
	required: "missing {name}",
	additional: "additional {name}",
	property_name: "invalid property name",
	no_branch: "no branch matches",
	many_branches: "multiple branches match",
	unknown_keyword: "unsupported keyword {name}",
	bad_keyword: "invalid keyword {name}",
	bad_reference: "invalid reference",
	reference_loop: "reference loop",
	limit: "past a limit",
} as const;

export type IssueCode = keyof typeof ISSUE_MESSAGES;

export interface Issue {
	path: PathPart[];
	code: IssueCode;
	message: string;
}
