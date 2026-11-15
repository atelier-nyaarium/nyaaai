//! Generated from the case kinds by scripts/contract.ts. Do not edit.

use nyaaai_contract::json::Value;
use nyaaai_contract::shape::Shape;

use super::case_handlers as handlers;
use super::case_shapes;

/// Runs one case kind.
pub fn dispatch(kind: &str, input: &Value) -> Option<Result<Value, String>> {
    Some(match kind {
        "json.roundTrip" => {
            run::<case_shapes::JsonRoundTripInput, case_shapes::JsonRoundTripResult>(
                input,
                handlers::json_round_trip,
            )
        }
        "shape.compatibleVersions" => run::<
            case_shapes::ShapeCompatibleVersionsInput,
            case_shapes::ShapeCompatibleVersionsResult,
        >(input, handlers::shape_compatible_versions),
        "base64.decode" => run::<case_shapes::Base64DecodeInput, case_shapes::Base64DecodeResult>(
            input,
            handlers::base64_decode,
        ),
        "json.canonical" => {
            run::<case_shapes::JsonCanonicalInput, case_shapes::JsonCanonicalResult>(
                input,
                handlers::json_canonical,
            )
        }
        "js.string" => run::<case_shapes::JsStringInput, case_shapes::JsStringResult>(
            input,
            handlers::js_string,
        ),
        "order.sortedTexts" => run::<
            case_shapes::OrderSortedTextsInput,
            case_shapes::OrderSortedTextsResult,
        >(input, handlers::order_sorted_texts),
        "contract.roundTrip" => run::<
            case_shapes::ContractRoundTripInput,
            case_shapes::ContractRoundTripResult,
        >(input, handlers::contract_round_trip),
        "time.parseIso" => run::<case_shapes::TimeParseIsoInput, case_shapes::TimeParseIsoResult>(
            input,
            handlers::time_parse_iso,
        ),
        "time.iso" => {
            run::<case_shapes::TimeIsoInput, case_shapes::TimeIsoResult>(input, handlers::time_iso)
        }
        "time.httpDate" => run::<case_shapes::TimeHttpDateInput, case_shapes::TimeHttpDateResult>(
            input,
            handlers::time_http_date,
        ),
        "time.retryAfter" => run::<
            case_shapes::TimeRetryAfterInput,
            case_shapes::TimeRetryAfterResult,
        >(input, handlers::time_retry_after),
        "time.resetStamp" => run::<
            case_shapes::TimeResetStampInput,
            case_shapes::TimeResetStampResult,
        >(input, handlers::time_reset_stamp),
        "time.dayKey" => run::<case_shapes::TimeDayKeyInput, case_shapes::TimeDayKeyResult>(
            input,
            handlers::time_day_key,
        ),
        "number.text" => run::<case_shapes::NumberTextInput, case_shapes::NumberTextResult>(
            input,
            handlers::number_text,
        ),
        "number.round" => run::<case_shapes::NumberRoundInput, case_shapes::NumberRoundResult>(
            input,
            handlers::number_round,
        ),
        "number.fixed" => run::<case_shapes::NumberFixedInput, case_shapes::NumberFixedResult>(
            input,
            handlers::number_fixed,
        ),
        "number.ofText" => run::<case_shapes::NumberOfTextInput, case_shapes::NumberOfTextResult>(
            input,
            handlers::number_of_text,
        ),
        "number.integerOfText" => run::<
            case_shapes::NumberIntegerOfTextInput,
            case_shapes::NumberIntegerOfTextResult,
        >(input, handlers::number_integer_of_text),
        "text.cuts" => run::<case_shapes::TextCutsInput, case_shapes::TextCutsResult>(
            input,
            handlers::text_cuts,
        ),
        "text.slices" => run::<case_shapes::TextSlicesInput, case_shapes::TextSlicesResult>(
            input,
            handlers::text_slices,
        ),
        "text.find" => run::<case_shapes::TextFindInput, case_shapes::TextFindResult>(
            input,
            handlers::text_find,
        ),
        "sessionCut.rehomed" => run::<
            case_shapes::SessionCutRehomedInput,
            case_shapes::SessionCutRehomedResult,
        >(input, handlers::session_cut_rehomed),
        "sessionCut.index" => run::<
            case_shapes::SessionCutIndexInput,
            case_shapes::SessionCutIndexResult,
        >(input, handlers::session_cut_index),
        "sessionCut.plan" => run::<
            case_shapes::SessionCutPlanInput,
            case_shapes::SessionCutPlanResult,
        >(input, handlers::session_cut_plan),
        "sessionCut.cut" => {
            run::<case_shapes::SessionCutCutInput, case_shapes::SessionCutCutResult>(
                input,
                handlers::session_cut_cut,
            )
        }
        "sessionCut.verify" => run::<
            case_shapes::SessionCutVerifyInput,
            case_shapes::SessionCutVerifyResult,
        >(input, handlers::session_cut_verify),
        "sessionCut.pickTurn" => run::<
            case_shapes::SessionCutPickTurnInput,
            case_shapes::SessionCutPickTurnResult,
        >(input, handlers::session_cut_pick_turn),
        "sessionFile.linesOf" => run::<
            case_shapes::SessionFileLinesOfInput,
            case_shapes::SessionFileLinesOfResult,
        >(input, handlers::session_file_lines_of),
        "paths.isInside" => {
            run::<case_shapes::PathsIsInsideInput, case_shapes::PathsIsInsideResult>(
                input,
                handlers::paths_is_inside,
            )
        }
        "claude.versionIn" => run::<
            case_shapes::ClaudeVersionInInput,
            case_shapes::ClaudeVersionInResult,
        >(input, handlers::claude_version_in),
        "pattern.problems" => run::<
            case_shapes::PatternProblemsInput,
            case_shapes::PatternProblemsResult,
        >(input, handlers::pattern_problems),
        "pattern.matches" => run::<
            case_shapes::PatternMatchesInput,
            case_shapes::PatternMatchesResult,
        >(input, handlers::pattern_matches),
        "pattern.find" => run::<case_shapes::PatternFindInput, case_shapes::PatternFindResult>(
            input,
            handlers::pattern_find,
        ),
        "pattern.edits" => run::<case_shapes::PatternEditsInput, case_shapes::PatternEditsResult>(
            input,
            handlers::pattern_edits,
        ),
        "strictSchema.normalize" => run::<
            case_shapes::StrictSchemaNormalizeInput,
            case_shapes::StrictSchemaNormalizeResult,
        >(input, handlers::strict_schema_normalize),
        "strictSchema.problems" => run::<
            case_shapes::StrictSchemaProblemsInput,
            case_shapes::StrictSchemaProblemsResult,
        >(input, handlers::strict_schema_problems),
        "strictSchema.validate" => run::<
            case_shapes::StrictSchemaValidateInput,
            case_shapes::StrictSchemaValidateResult,
        >(input, handlers::strict_schema_validate),
        "strictSchema.validateList" => run::<
            case_shapes::StrictSchemaValidateListInput,
            case_shapes::StrictSchemaValidateListResult,
        >(input, handlers::strict_schema_validate_list),
        "testkit.compactedSession" => run::<
            case_shapes::TestkitCompactedSessionInput,
            case_shapes::TestkitCompactedSessionResult,
        >(input, handlers::testkit_compacted_session),
        "testkit.replay" => {
            run::<case_shapes::TestkitReplayInput, case_shapes::TestkitReplayResult>(
                input,
                handlers::testkit_replay,
            )
        }
        "testkit.wheel" => run::<case_shapes::TestkitWheelInput, case_shapes::TestkitWheelResult>(
            input,
            handlers::testkit_wheel,
        ),
        "testkit.random" => {
            run::<case_shapes::TestkitRandomInput, case_shapes::TestkitRandomResult>(
                input,
                handlers::testkit_random,
            )
        }
        _ => return None,
    })
}

fn run<Input: Shape, Output: Shape>(
    input: &Value,
    handler: fn(Input) -> Result<Output, String>,
) -> Result<Value, String> {
    let input = Input::decode(input).map_err(|refused| format!("the input: {refused}"))?;
    handler(input).map(|result| result.encode())
}
