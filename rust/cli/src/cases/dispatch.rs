//! Generated from the case groups by scripts/contract.ts. Do not edit.

use std::path::Path;

use nyaaai_contract::json::Value;
use nyaaai_testkit::cases::{run, run_with};

use super::shapes;

/// Runs one case kind. `root` is the project root.
pub fn dispatch(kind: &str, input: &Value, root: &Path) -> Option<Result<Value, String>> {
    Some(match kind {
        "json.roundTrip" => run::<shapes::JsonRoundTripInput, shapes::JsonRoundTripResult>(
            input,
            super::json::round_trip,
        ),
        "json.canonical" => run::<shapes::JsonCanonicalInput, shapes::JsonCanonicalResult>(
            input,
            super::json::canonical,
        ),
        "js.string" => {
            run::<shapes::JsStringInput, shapes::JsStringResult>(input, super::js::string)
        }
        "order.sortedTexts" => {
            run::<shapes::OrderSortedTextsInput, shapes::OrderSortedTextsResult>(
                input,
                super::order::sorted_texts,
            )
        }
        "base64.decode" => run::<shapes::Base64DecodeInput, shapes::Base64DecodeResult>(
            input,
            super::base64::decode,
        ),
        "headers.of" => {
            run::<shapes::HeadersOfInput, shapes::HeadersOfResult>(input, super::headers::of)
        }
        "headers.response" => run::<shapes::HeadersResponseInput, shapes::HeadersResponseResult>(
            input,
            super::headers::response,
        ),
        "shape.compatibleVersions" => run::<
            shapes::ShapeCompatibleVersionsInput,
            shapes::ShapeCompatibleVersionsResult,
        >(input, super::shape::compatible_versions),
        "contract.roundTrip" => run::<
            shapes::ContractRoundTripInput,
            shapes::ContractRoundTripResult,
        >(input, super::contract::round_trip),
        "time.parseIso" => run::<shapes::TimeParseIsoInput, shapes::TimeParseIsoResult>(
            input,
            super::time::parse_iso,
        ),
        "time.iso" => run::<shapes::TimeIsoInput, shapes::TimeIsoResult>(input, super::time::iso),
        "time.httpDate" => run::<shapes::TimeHttpDateInput, shapes::TimeHttpDateResult>(
            input,
            super::time::http_date,
        ),
        "time.retryAfter" => run::<shapes::TimeRetryAfterInput, shapes::TimeRetryAfterResult>(
            input,
            super::time::retry_after,
        ),
        "time.resetStamp" => run::<shapes::TimeResetStampInput, shapes::TimeResetStampResult>(
            input,
            super::time::reset_stamp,
        ),
        "time.dayKey" => {
            run::<shapes::TimeDayKeyInput, shapes::TimeDayKeyResult>(input, super::time::day_key)
        }
        "number.text" => {
            run::<shapes::NumberTextInput, shapes::NumberTextResult>(input, super::number::text)
        }
        "number.round" => {
            run::<shapes::NumberRoundInput, shapes::NumberRoundResult>(input, super::number::round)
        }
        "number.fixed" => {
            run::<shapes::NumberFixedInput, shapes::NumberFixedResult>(input, super::number::fixed)
        }
        "number.ofText" => run::<shapes::NumberOfTextInput, shapes::NumberOfTextResult>(
            input,
            super::number::of_text,
        ),
        "number.integerOfText" => run::<
            shapes::NumberIntegerOfTextInput,
            shapes::NumberIntegerOfTextResult,
        >(input, super::number::integer_of_text),
        "text.cuts" => {
            run::<shapes::TextCutsInput, shapes::TextCutsResult>(input, super::text::cuts)
        }
        "text.slices" => {
            run::<shapes::TextSlicesInput, shapes::TextSlicesResult>(input, super::text::slices)
        }
        "text.spaces" => {
            run::<shapes::TextSpacesInput, shapes::TextSpacesResult>(input, super::text::spaces)
        }
        "text.find" => {
            run::<shapes::TextFindInput, shapes::TextFindResult>(input, super::text::find)
        }
        "text.utf8Chunks" => run::<shapes::TextUtf8ChunksInput, shapes::TextUtf8ChunksResult>(
            input,
            super::text::utf8_chunks,
        ),
        "text.hasControl" => run::<shapes::TextHasControlInput, shapes::TextHasControlResult>(
            input,
            super::text::has_control,
        ),
        "text.pathSegment" => run::<shapes::TextPathSegmentInput, shapes::TextPathSegmentResult>(
            input,
            super::text::path_segment,
        ),
        "text.nfc" => run::<shapes::TextNfcInput, shapes::TextNfcResult>(input, super::text::nfc),
        "url.origin" => {
            run::<shapes::UrlOriginInput, shapes::UrlOriginResult>(input, super::url::origin)
        }
        "url.parts" => {
            run::<shapes::UrlPartsInput, shapes::UrlPartsResult>(input, super::url::parts)
        }
        "pattern.problems" => run::<shapes::PatternProblemsInput, shapes::PatternProblemsResult>(
            input,
            super::pattern::problems,
        ),
        "pattern.matches" => run::<shapes::PatternMatchesInput, shapes::PatternMatchesResult>(
            input,
            super::pattern::matches,
        ),
        "pattern.find" => {
            run::<shapes::PatternFindInput, shapes::PatternFindResult>(input, super::pattern::find)
        }
        "pattern.edits" => run::<shapes::PatternEditsInput, shapes::PatternEditsResult>(
            input,
            super::pattern::edits,
        ),
        "strictSchema.normalize" => run::<
            shapes::StrictSchemaNormalizeInput,
            shapes::StrictSchemaNormalizeResult,
        >(input, super::strict_schema::normalize),
        "strictSchema.problems" => run::<
            shapes::StrictSchemaProblemsInput,
            shapes::StrictSchemaProblemsResult,
        >(input, super::strict_schema::problems),
        "strictSchema.validate" => run::<
            shapes::StrictSchemaValidateInput,
            shapes::StrictSchemaValidateResult,
        >(input, super::strict_schema::validate),
        "strictSchema.validateList" => run::<
            shapes::StrictSchemaValidateListInput,
            shapes::StrictSchemaValidateListResult,
        >(input, super::strict_schema::validate_list),
        "sessionCut.rehomed" => run::<
            shapes::SessionCutRehomedInput,
            shapes::SessionCutRehomedResult,
        >(input, super::session_cut::rehomed),
        "sessionCut.index" => run::<shapes::SessionCutIndexInput, shapes::SessionCutIndexResult>(
            input,
            super::session_cut::index,
        ),
        "sessionCut.plan" => run::<shapes::SessionCutPlanInput, shapes::SessionCutPlanResult>(
            input,
            super::session_cut::plan,
        ),
        "sessionCut.cut" => run::<shapes::SessionCutCutInput, shapes::SessionCutCutResult>(
            input,
            super::session_cut::cut,
        ),
        "sessionCut.verify" => {
            run::<shapes::SessionCutVerifyInput, shapes::SessionCutVerifyResult>(
                input,
                super::session_cut::verify,
            )
        }
        "sessionCut.pickTurn" => run::<
            shapes::SessionCutPickTurnInput,
            shapes::SessionCutPickTurnResult,
        >(input, super::session_cut::pick_turn),
        "sessionFile.linesOf" => run::<
            shapes::SessionFileLinesOfInput,
            shapes::SessionFileLinesOfResult,
        >(input, super::session_file::lines_of),
        "paths.isInside" => run_with::<shapes::PathsIsInsideInput, shapes::PathsIsInsideResult>(
            input,
            root,
            super::paths::is_inside,
        ),
        "claude.versionIn" => run::<shapes::ClaudeVersionInInput, shapes::ClaudeVersionInResult>(
            input,
            super::claude::version_in,
        ),
        "testkit.entryKeys" => {
            run::<shapes::TestkitEntryKeysInput, shapes::TestkitEntryKeysResult>(
                input,
                super::testkit::entry_keys,
            )
        }
        "testkit.providerList" => run::<
            shapes::TestkitProviderListInput,
            shapes::TestkitProviderListResult,
        >(input, super::testkit::provider_list),
        "testkit.modelRegistry" => run::<
            shapes::TestkitModelRegistryInput,
            shapes::TestkitModelRegistryResult,
        >(input, super::testkit::model_registry),
        "testkit.deadline" => run::<shapes::TestkitDeadlineInput, shapes::TestkitDeadlineResult>(
            input,
            super::testkit::deadline,
        ),
        "testkit.compactedSession" => run::<
            shapes::TestkitCompactedSessionInput,
            shapes::TestkitCompactedSessionResult,
        >(input, super::testkit::compacted_session),
        "testkit.wheel" => run::<shapes::TestkitWheelInput, shapes::TestkitWheelResult>(
            input,
            super::testkit::wheel,
        ),
        "testkit.scheduler" => {
            run::<shapes::TestkitSchedulerInput, shapes::TestkitSchedulerResult>(
                input,
                super::testkit::scheduler,
            )
        }
        "testkit.random" => run::<shapes::TestkitRandomInput, shapes::TestkitRandomResult>(
            input,
            super::testkit::random,
        ),
        "testkit.replay" => run_with::<shapes::TestkitReplayInput, shapes::TestkitReplayResult>(
            input,
            root,
            super::testkit::replay,
        ),
        "testkit.context" => run_with::<shapes::TestkitContextInput, shapes::TestkitContextResult>(
            input,
            root,
            super::testkit::context,
        ),
        "noticeText.texts" => run::<shapes::NoticeTextTextsInput, shapes::NoticeTextTextsResult>(
            input,
            super::notice_text::texts,
        ),
        "effort.ladder" => run::<shapes::EffortLadderInput, shapes::EffortLadderResult>(
            input,
            super::effort::ladder,
        ),
        "effort.resolve" => run::<shapes::EffortResolveInput, shapes::EffortResolveResult>(
            input,
            super::effort::resolve,
        ),
        "sse.items" => {
            run::<shapes::SseItemsInput, shapes::SseItemsResult>(input, super::sse::items)
        }
        "sse.splits" => {
            run::<shapes::SseSplitsInput, shapes::SseSplitsResult>(input, super::sse::splits)
        }
        "step.assemble" => run::<shapes::StepAssembleInput, shapes::StepAssembleResult>(
            input,
            super::step::assemble,
        ),
        "step.replies" => {
            run::<shapes::StepRepliesInput, shapes::StepRepliesResult>(input, super::step::replies)
        }
        "failure.message" => run::<shapes::FailureMessageInput, shapes::FailureMessageResult>(
            input,
            super::failure::message,
        ),
        "failure.of" => {
            run::<shapes::FailureOfInput, shapes::FailureOfResult>(input, super::failure::of)
        }
        "native.readers" => run::<shapes::NativeReadersInput, shapes::NativeReadersResult>(
            input,
            super::native::readers,
        ),
        "native.schemaPrompt" => run::<
            shapes::NativeSchemaPromptInput,
            shapes::NativeSchemaPromptResult,
        >(input, super::native::schema_prompt),
        "native.systemTexts" => run::<
            shapes::NativeSystemTextsInput,
            shapes::NativeSystemTextsResult,
        >(input, super::native::system_texts),
        "native.arguments" => run::<shapes::NativeArgumentsInput, shapes::NativeArgumentsResult>(
            input,
            super::native::arguments,
        ),
        "native.currentTurn" => run::<
            shapes::NativeCurrentTurnInput,
            shapes::NativeCurrentTurnResult,
        >(input, super::native::current_turn),
        "native.chunk" => {
            run::<shapes::NativeChunkInput, shapes::NativeChunkResult>(input, super::native::chunk)
        }
        "native.wireError" => run::<shapes::NativeWireErrorInput, shapes::NativeWireErrorResult>(
            input,
            super::native::wire_error,
        ),
        "native.streamFailure" => run::<
            shapes::NativeStreamFailureInput,
            shapes::NativeStreamFailureResult,
        >(input, super::native::stream_failure),
        "native.assembled" => run::<shapes::NativeAssembledInput, shapes::NativeAssembledResult>(
            input,
            super::native::assembled,
        ),
        "native.serving" => run::<shapes::NativeServingInput, shapes::NativeServingResult>(
            input,
            super::native::serving,
        ),
        "schemaProjection.project" => run::<
            shapes::SchemaProjectionProjectInput,
            shapes::SchemaProjectionProjectResult,
        >(input, super::schema_projection::project),
        "codec.encode" => {
            run::<shapes::CodecEncodeInput, shapes::CodecEncodeResult>(input, super::codec::encode)
        }
        "codec.stream" => {
            run::<shapes::CodecStreamInput, shapes::CodecStreamResult>(input, super::codec::stream)
        }
        "codec.complete" => run::<shapes::CodecCompleteInput, shapes::CodecCompleteResult>(
            input,
            super::codec::complete,
        ),
        "codec.failure" => run::<shapes::CodecFailureInput, shapes::CodecFailureResult>(
            input,
            super::codec::failure,
        ),
        "transport.exchange" => run_with::<
            shapes::TransportExchangeInput,
            shapes::TransportExchangeResult,
        >(input, root, super::transport::exchange),
        "transport.key" => run::<shapes::TransportKeyInput, shapes::TransportKeyResult>(
            input,
            super::transport::key,
        ),
        "secret.spellings" => run::<shapes::SecretSpellingsInput, shapes::SecretSpellingsResult>(
            input,
            super::secret::spellings,
        ),
        "secret.scrub" => {
            run::<shapes::SecretScrubInput, shapes::SecretScrubResult>(input, super::secret::scrub)
        }
        "entry.parse" => {
            run::<shapes::EntryParseInput, shapes::EntryParseResult>(input, super::entry::parse)
        }
        "entry.parseList" => run::<shapes::EntryParseListInput, shapes::EntryParseListResult>(
            input,
            super::entry::parse_list,
        ),
        "entry.sameOrigin" => run::<shapes::EntrySameOriginInput, shapes::EntrySameOriginResult>(
            input,
            super::entry::same_origin,
        ),
        "entry.id" => run::<shapes::EntryIdInput, shapes::EntryIdResult>(input, super::entry::id),
        "entry.secretKey" => run::<shapes::EntrySecretKeyInput, shapes::EntrySecretKeyResult>(
            input,
            super::entry::secret_key,
        ),
        "rowKey.build" => {
            run::<shapes::RowKeyBuildInput, shapes::RowKeyBuildResult>(input, super::row_key::build)
        }
        "rowKey.parse" => {
            run::<shapes::RowKeyParseInput, shapes::RowKeyParseResult>(input, super::row_key::parse)
        }
        "rowKey.shown" => {
            run::<shapes::RowKeyShownInput, shapes::RowKeyShownResult>(input, super::row_key::shown)
        }
        "rowKey.servedId" => run::<shapes::RowKeyServedIdInput, shapes::RowKeyServedIdResult>(
            input,
            super::row_key::served_id,
        ),
        "curated.rows" => {
            run::<shapes::CuratedRowsInput, shapes::CuratedRowsResult>(input, super::curated::rows)
        }
        "registry.testRecord" => run::<
            shapes::RegistryTestRecordInput,
            shapes::RegistryTestRecordResult,
        >(input, super::registry::test_record),
        "registry.parse" => run::<shapes::RegistryParseInput, shapes::RegistryParseResult>(
            input,
            super::registry::parse,
        ),
        "registry.scoped" => run::<shapes::RegistryScopedInput, shapes::RegistryScopedResult>(
            input,
            super::registry::scoped,
        ),
        "registry.rows" => run::<shapes::RegistryRowsInput, shapes::RegistryRowsResult>(
            input,
            super::registry::rows,
        ),
        "registry.offered" => run::<shapes::RegistryOfferedInput, shapes::RegistryOfferedResult>(
            input,
            super::registry::offered,
        ),
        "registry.list" => run::<shapes::RegistryListInput, shapes::RegistryListResult>(
            input,
            super::registry::list,
        ),
        "registry.select" => run::<shapes::RegistrySelectInput, shapes::RegistrySelectResult>(
            input,
            super::registry::select,
        ),
        "registry.passed" => run::<shapes::RegistryPassedInput, shapes::RegistryPassedResult>(
            input,
            super::registry::passed,
        ),
        "registry.ids" => {
            run::<shapes::RegistryIdsInput, shapes::RegistryIdsResult>(input, super::registry::ids)
        }
        "view.unknown" => {
            run::<shapes::ViewUnknownInput, shapes::ViewUnknownResult>(input, super::view::unknown)
        }
        "view.tiers" => {
            run::<shapes::ViewTiersInput, shapes::ViewTiersResult>(input, super::view::tiers)
        }
        "view.codecFacts" => run::<shapes::ViewCodecFactsInput, shapes::ViewCodecFactsResult>(
            input,
            super::view::codec_facts,
        ),
        "view.outputLimit" => run::<shapes::ViewOutputLimitInput, shapes::ViewOutputLimitResult>(
            input,
            super::view::output_limit,
        ),
        "view.runEffort" => run::<shapes::ViewRunEffortInput, shapes::ViewRunEffortResult>(
            input,
            super::view::run_effort,
        ),
        "retention.of" => {
            run::<shapes::RetentionOfInput, shapes::RetentionOfResult>(input, super::retention::of)
        }
        "retention.isZdr" => run::<shapes::RetentionIsZdrInput, shapes::RetentionIsZdrResult>(
            input,
            super::retention::is_zdr,
        ),
        "disposition.of" => run::<shapes::DispositionOfInput, shapes::DispositionOfResult>(
            input,
            super::disposition::of,
        ),
        "disposition.skipReason" => run::<
            shapes::DispositionSkipReasonInput,
            shapes::DispositionSkipReasonResult,
        >(input, super::disposition::skip_reason),
        _ => return None,
    })
}
