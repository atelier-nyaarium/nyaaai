// Case kind shapes.

import { ISSUE_MESSAGES, type IssueCode } from "@nyaaai/contract/issues";
import { shapeSet } from "@nyaaai/contract/shape";
import { PathPart } from "@nyaaai/contract/shapes/frames";
import { CutDone, CutPlan, CutRefused, CutRequest, PickedTurn, SessionIndex } from "@nyaaai/contract/shapes/sessionCut";
import { z } from "zod";

////////////////////////////////
//  Shapes

export const SessionSource = z.object({
	lines: z.array(z.string()).exactOptional(),
	base64: z.string().exactOptional(),
	file: z.string().exactOptional(),
});
export type SessionSource = z.infer<typeof SessionSource>;

/** Turn-start body extraction mode. */
export const BodyOf = z.enum(["whole", "afterFirstLine"]);
export type BodyOf = z.infer<typeof BodyOf>;

/** `$ROOT` marks root. */
export const TreeEntry = z.object({
	path: z.string(),
	file: z.literal(true).exactOptional(),
	link: z.string().exactOptional(),
});
export type TreeEntry = z.infer<typeof TreeEntry>;

export const JsonRoundTripInput = z.object({ texts: z.array(z.string()) });
export const JsonRoundTripResult = z.object({
	results: z.array(z.object({ ok: z.boolean(), text: z.string().exactOptional() })),
});

export const ShapeCompatibleVersionsInput = z.object({
	pairs: z.array(z.object({ left: z.string(), right: z.string() })),
});
export const ShapeCompatibleVersionsResult = z.object({ compatible: z.array(z.boolean()) });

export const Base64DecodeInput = z.object({ texts: z.array(z.string()) });
/** Null marks decode refusal. */
export const Base64DecodeResult = z.object({ texts: z.array(z.string().nullable()) });

export const JsonCanonicalInput = z.object({ texts: z.array(z.string()) });
export const JsonCanonicalResult = z.object({ texts: z.array(z.string().nullable()) });
/** JSON-encoded values. */
export const JsStringInput = z.object({ texts: z.array(z.string()) });
export const JsStringResult = z.object({ strings: z.array(z.string()) });

export const OrderSortedTextsInput = z.object({ texts: z.array(z.string()) });
export const OrderSortedTextsResult = z.object({ texts: z.array(z.string()) });

export const ContractRoundTripInput = z.object({ set: z.string(), shape: z.string(), texts: z.array(z.string()) });
export const ContractRoundTripResult = z.object({
	results: z.array(
		z.object({ ok: z.boolean(), text: z.string().exactOptional(), path: z.array(PathPart).exactOptional() }),
	),
});

export const TimeParseIsoInput = z.object({ texts: z.array(z.string()) });
export const TimeParseIsoResult = z.object({ results: z.array(z.int().nullable()) });

export const TimeIsoInput = z.object({ times: z.array(z.int()) });
export const TimeIsoResult = z.object({ texts: z.array(z.string().nullable()) });

export const TimeHttpDateInput = z.object({ texts: z.array(z.string()), nowMs: z.int() });
export const TimeHttpDateResult = z.object({ results: z.array(z.int().nullable()) });

export const TimeRetryAfterInput = z.object({ headers: z.array(z.string().nullable()), nowMs: z.int() });
export const TimeRetryAfterResult = z.object({ results: z.array(z.int().nullable()) });

export const TimeResetStampInput = z.object({ values: z.array(z.unknown()), nowMs: z.int() });
export const TimeResetStampResult = z.object({ results: z.array(z.int().nullable()) });

export const TimeDayKeyInput = z.object({ times: z.array(z.int()), offsets: z.array(z.int()) });
export const TimeDayKeyResult = z.object({ rows: z.array(z.array(z.string().nullable())) });

export const NumberTextInput = z.object({ values: z.array(z.number()) });
export const NumberTextResult = z.object({ texts: z.array(z.string()) });

export const NumberRoundInput = z.object({ values: z.array(z.number()) });
export const NumberRoundResult = z.object({ texts: z.array(z.string()) });

export const NumberFixedInput = z.object({ values: z.array(z.number()), digits: z.array(z.int().min(0)) });
export const NumberFixedResult = z.object({ rows: z.array(z.array(z.string())) });

export const NumberOfTextInput = z.object({ texts: z.array(z.string()) });
export const NumberOfTextResult = z.object({ values: z.array(z.number().nullable()) });

export const NumberIntegerOfTextInput = z.object({ texts: z.array(z.string()) });
export const NumberIntegerOfTextResult = z.object({ values: z.array(z.int().nullable()) });

export const TextCutsInput = z.object({ texts: z.array(z.string()), points: z.array(z.int().min(0)) });
export const TextCutsResult = z.object({
	rows: z.array(
		z.object({
			length: z.int(),
			utf8: z.int(),
			before: z.array(z.int()),
			after: z.array(z.int()),
			heads: z.array(z.string()),
			tails: z.array(z.string()),
			bytes: z.array(z.string()),
			lower: z.string(),
			upper: z.string(),
		}),
	),
});

export const TextSlicesInput = z.object({
	text: z.string(),
	spans: z.array(z.object({ start: z.int().min(0), end: z.int().min(0) })),
});
export const TextSlicesResult = z.object({ slices: z.array(z.string()) });

export const TextFindInput = z.object({
	text: z.string(),
	needles: z.array(z.string()),
	froms: z.array(z.int().min(0)),
});
/** `-1` marks no match. */
export const TextFindResult = z.object({ first: z.array(z.array(z.int())), last: z.array(z.array(z.int())) });

export const SessionCutRehomedInput = z.object({ rows: z.array(z.string()), sessionId: z.string() });
export const SessionCutRehomedResult = z.object({ rows: z.array(z.string()) });

export const SessionCutIndexInput = z.object({ source: SessionSource });
export const SessionCutIndexResult = z.object({ index: SessionIndex });

export const SessionCutPlanInput = z.object({
	source: SessionSource,
	requests: z.array(CutRequest),
	cliVersion: z.string(),
});
export const SessionCutPlanResult = z.object({ plans: z.array(z.union([CutPlan, CutRefused])) });

export const SessionCutCutInput = z.object({
	source: SessionSource,
	request: CutRequest,
	cliVersion: z.string(),
	sourcePath: z.string(),
	/** IDs append counts. */
	idPrefix: z.string(),
	nowMs: z.int(),
	/** Absent disables summarization. */
	summary: z.string().exactOptional(),
});
export type SessionCutCutInput = z.infer<typeof SessionCutCutInput>;
export const SessionCutCutResult = z.object({
	error: z.string().exactOptional(),
	result: z.union([CutDone, CutRefused]).exactOptional(),
	built: z.array(z.string()).nullable().exactOptional(),
	/** Summarizer arguments. */
	summarized: z
		.object({ standing: z.string().nullable(), rows: z.array(z.string()) })
		.nullable()
		.exactOptional(),
});

export const SessionCutVerifyInput = z.object({ source: SessionSource, sessionId: z.string(), paired: z.boolean() });
export const SessionCutVerifyResult = z.object({ problems: z.array(z.string()) });

export const SessionCutPickTurnInput = z.object({
	source: SessionSource,
	picks: z.array(z.object({ at: z.int(), text: z.string().exactOptional(), bodyOf: BodyOf })),
});
export const SessionCutPickTurnResult = z.object({ picked: z.array(PickedTurn.nullable()) });

export const SessionFileLinesOfInput = z.object({ base64: z.string() });
export const SessionFileLinesOfResult = z.object({ rows: z.array(z.string()) });

export const PathsIsInsideInput = z.object({
	tree: z.array(TreeEntry),
	checks: z.array(z.object({ child: z.string(), parent: z.string() })),
	/** Relative paths use root. */
	relative: z.literal(true).exactOptional(),
});
export const PathsIsInsideResult = z.object({ results: z.array(z.union([z.boolean(), z.literal("error")])) });

export const ClaudeVersionInInput = z.object({ texts: z.array(z.string()) });
export const ClaudeVersionInResult = z.object({ found: z.array(z.string().nullable()) });

export const PatternCaseMatch = z.object({
	start: z.int().min(0),
	end: z.int().min(0),
	text: z.string(),
	captures: z.array(z.string().nullable()),
});
export const PatternProblemsInput = z.object({
	patterns: z.array(z.object({ source: z.string(), flags: z.string() })),
});
export const PatternProblemsResult = z.object({ refused: z.array(z.boolean()) });
export const PatternMatchesInput = z.object({ source: z.string(), flags: z.string(), texts: z.array(z.string()) });
export const PatternMatchesResult = z.object({
	results: z.array(z.object({ test: z.boolean(), all: z.array(PatternCaseMatch) })),
});
export const PatternFindInput = z.object({
	source: z.string(),
	flags: z.string(),
	text: z.string(),
	froms: z.array(z.int().min(0)),
});
export const PatternFindResult = z.object({ found: z.array(PatternCaseMatch.nullable()) });
export const PatternEditsInput = z.object({
	source: z.string(),
	flags: z.string(),
	template: z.string(),
	texts: z.array(z.string()),
});
export const PatternEditsResult = z.object({
	results: z.array(z.object({ replaced: z.string(), pieces: z.array(z.string()) })),
});
export const StrictSchemaIssue = z.object({
	path: z.array(PathPart),
	code: z.enum(Object.keys(ISSUE_MESSAGES) as [IssueCode, ...IssueCode[]]),
	message: z.string(),
});
export const StrictSchemaNormalizeInput = z.object({ schemas: z.array(z.unknown()) });
export const StrictSchemaNormalizeResult = z.object({ schemas: z.array(z.unknown()) });
export const StrictSchemaProblemsInput = z.object({ schemas: z.array(z.unknown()) });
export const StrictSchemaProblemsResult = z.object({ problems: z.array(z.array(StrictSchemaIssue)) });
/** Model output JSON texts. */
export const StrictSchemaValidateInput = z.object({ schema: z.unknown(), texts: z.array(z.string()) });
export const StrictSchemaValidateResult = z.object({ issues: z.array(z.array(StrictSchemaIssue)) });
/** Repeated item JSON text. */
export const StrictSchemaValidateListInput = z.object({
	schema: z.unknown(),
	item: z.string(),
	counts: z.array(z.int().min(0)),
});
export const StrictSchemaValidateListResult = z.object({ issues: z.array(z.array(StrictSchemaIssue)) });

export const TestkitCompactedSessionInput = z.object({});
export const TestkitCompactedSessionResult = z.object({ lines: z.array(z.string()) });

export const TestkitWheelInput = z.object({
	startMs: z.int(),
	steps: z.array(
		z.object({
			op: z.enum(["after", "cancel", "advance"]),
			ms: z.int().exactOptional(),
			tag: z.string().exactOptional(),
		}),
	),
});
export const TestkitWheelResult = z.object({
	steps: z.array(
		z.object({
			nowMs: z.int(),
			waiting: z.array(z.string()),
			fired: z.array(z.object({ tag: z.string(), atMs: z.int() })),
			cancelled: z.boolean(),
		}),
	),
});

export const TestkitRandomInput = z.object({ values: z.array(z.number()), draws: z.int().min(0) });
export const TestkitRandomResult = z.object({ drawn: z.array(z.number()) });

export const TestkitReplayInput = z.object({
	/** Path is repository-relative. */
	file: z.string(),
});
export const TestkitReplayResult = z.object({
	requests: z.array(z.unknown()),
	exchanges: z.array(
		z.object({
			status: z.int(),
			headers: z.record(z.string(), z.string()),
			/** Byte length per chunk. */
			chunks: z.array(z.int()),
			body: z.string(),
		}),
	),
});

/** `together` starts requests first. */
export const TestkitConverseInput = z.object({ requests: z.int().min(0), together: z.boolean() });
export const TestkitConverseResult = z.object({
	ids: z.array(z.string()),
	nowMs: z.int(),
	/** Unreported steering inputs. */
	steers: z.array(z.string()),
});

////////////////////////////////
//  Constants

/** Rust uses snake_case handlers. */
export const CASE_KINDS = {
	"json.roundTrip": { input: JsonRoundTripInput, result: JsonRoundTripResult },
	"shape.compatibleVersions": { input: ShapeCompatibleVersionsInput, result: ShapeCompatibleVersionsResult },
	"base64.decode": { input: Base64DecodeInput, result: Base64DecodeResult },
	"json.canonical": { input: JsonCanonicalInput, result: JsonCanonicalResult },
	"js.string": { input: JsStringInput, result: JsStringResult },
	"order.sortedTexts": { input: OrderSortedTextsInput, result: OrderSortedTextsResult },
	"contract.roundTrip": { input: ContractRoundTripInput, result: ContractRoundTripResult },
	"time.parseIso": { input: TimeParseIsoInput, result: TimeParseIsoResult },
	"time.iso": { input: TimeIsoInput, result: TimeIsoResult },
	"time.httpDate": { input: TimeHttpDateInput, result: TimeHttpDateResult },
	"time.retryAfter": { input: TimeRetryAfterInput, result: TimeRetryAfterResult },
	"time.resetStamp": { input: TimeResetStampInput, result: TimeResetStampResult },
	"time.dayKey": { input: TimeDayKeyInput, result: TimeDayKeyResult },
	"number.text": { input: NumberTextInput, result: NumberTextResult },
	"number.round": { input: NumberRoundInput, result: NumberRoundResult },
	"number.fixed": { input: NumberFixedInput, result: NumberFixedResult },
	"number.ofText": { input: NumberOfTextInput, result: NumberOfTextResult },
	"number.integerOfText": { input: NumberIntegerOfTextInput, result: NumberIntegerOfTextResult },
	"text.cuts": { input: TextCutsInput, result: TextCutsResult },
	"text.slices": { input: TextSlicesInput, result: TextSlicesResult },
	"text.find": { input: TextFindInput, result: TextFindResult },
	"sessionCut.rehomed": { input: SessionCutRehomedInput, result: SessionCutRehomedResult },
	"sessionCut.index": { input: SessionCutIndexInput, result: SessionCutIndexResult },
	"sessionCut.plan": { input: SessionCutPlanInput, result: SessionCutPlanResult },
	"sessionCut.cut": { input: SessionCutCutInput, result: SessionCutCutResult },
	"sessionCut.verify": { input: SessionCutVerifyInput, result: SessionCutVerifyResult },
	"sessionCut.pickTurn": { input: SessionCutPickTurnInput, result: SessionCutPickTurnResult },
	"sessionFile.linesOf": { input: SessionFileLinesOfInput, result: SessionFileLinesOfResult },
	"paths.isInside": { input: PathsIsInsideInput, result: PathsIsInsideResult },
	"claude.versionIn": { input: ClaudeVersionInInput, result: ClaudeVersionInResult },
	"pattern.problems": { input: PatternProblemsInput, result: PatternProblemsResult },
	"pattern.matches": { input: PatternMatchesInput, result: PatternMatchesResult },
	"pattern.find": { input: PatternFindInput, result: PatternFindResult },
	"pattern.edits": { input: PatternEditsInput, result: PatternEditsResult },
	"strictSchema.normalize": { input: StrictSchemaNormalizeInput, result: StrictSchemaNormalizeResult },
	"strictSchema.problems": { input: StrictSchemaProblemsInput, result: StrictSchemaProblemsResult },
	"strictSchema.validate": { input: StrictSchemaValidateInput, result: StrictSchemaValidateResult },
	"strictSchema.validateList": { input: StrictSchemaValidateListInput, result: StrictSchemaValidateListResult },
	"testkit.compactedSession": { input: TestkitCompactedSessionInput, result: TestkitCompactedSessionResult },
	"testkit.replay": { input: TestkitReplayInput, result: TestkitReplayResult },
	"testkit.wheel": { input: TestkitWheelInput, result: TestkitWheelResult },
	"testkit.random": { input: TestkitRandomInput, result: TestkitRandomResult },
} as const;

export type CaseKind = keyof typeof CASE_KINDS;

export const casesSet = shapeSet("cases", {
	SessionSource,
	BodyOf,
	TreeEntry,
	JsonRoundTripInput,
	JsonRoundTripResult,
	ShapeCompatibleVersionsInput,
	ShapeCompatibleVersionsResult,
	Base64DecodeInput,
	Base64DecodeResult,
	JsStringInput,
	JsStringResult,
	JsonCanonicalInput,
	JsonCanonicalResult,
	OrderSortedTextsInput,
	OrderSortedTextsResult,
	ContractRoundTripInput,
	ContractRoundTripResult,
	TimeParseIsoInput,
	TimeParseIsoResult,
	TimeIsoInput,
	TimeIsoResult,
	TimeHttpDateInput,
	TimeHttpDateResult,
	TimeRetryAfterInput,
	TimeRetryAfterResult,
	TimeResetStampInput,
	TimeResetStampResult,
	TimeDayKeyInput,
	TimeDayKeyResult,
	NumberTextInput,
	NumberTextResult,
	NumberRoundInput,
	NumberRoundResult,
	NumberFixedInput,
	NumberFixedResult,
	NumberOfTextInput,
	NumberOfTextResult,
	NumberIntegerOfTextInput,
	NumberIntegerOfTextResult,
	TextCutsInput,
	TextCutsResult,
	TextSlicesInput,
	TextSlicesResult,
	TextFindInput,
	TextFindResult,
	SessionCutRehomedInput,
	SessionCutRehomedResult,
	SessionCutIndexInput,
	SessionCutIndexResult,
	SessionCutPlanInput,
	SessionCutPlanResult,
	SessionCutCutInput,
	SessionCutCutResult,
	SessionCutVerifyInput,
	SessionCutVerifyResult,
	SessionCutPickTurnInput,
	SessionCutPickTurnResult,
	SessionFileLinesOfInput,
	SessionFileLinesOfResult,
	PathsIsInsideInput,
	PathsIsInsideResult,
	ClaudeVersionInInput,
	ClaudeVersionInResult,
	PatternCaseMatch,
	PatternProblemsInput,
	PatternProblemsResult,
	PatternMatchesInput,
	PatternMatchesResult,
	PatternFindInput,
	PatternFindResult,
	PatternEditsInput,
	PatternEditsResult,
	StrictSchemaIssue,
	StrictSchemaNormalizeInput,
	StrictSchemaNormalizeResult,
	StrictSchemaProblemsInput,
	StrictSchemaProblemsResult,
	StrictSchemaValidateInput,
	StrictSchemaValidateResult,
	StrictSchemaValidateListInput,
	StrictSchemaValidateListResult,
	TestkitCompactedSessionInput,
	TestkitCompactedSessionResult,
	TestkitReplayInput,
	TestkitReplayResult,
	TestkitWheelInput,
	TestkitWheelResult,
	TestkitRandomInput,
	TestkitRandomResult,
	TestkitConverseInput,
	TestkitConverseResult,
});
