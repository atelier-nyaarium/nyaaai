//! Text cases.

use nyaaai_contract::base64::base64_decode;
use nyaaai_contract::text;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn cuts(input: TextCutsInput) -> Outcome<TextCutsResult> {
    let points = &input.points;
    let each = |held: &dyn Fn(usize) -> String| -> Vec<String> {
        points.iter().map(|at| held(*at as usize)).collect()
    };
    let offsets = |held: &dyn Fn(usize) -> usize| -> Vec<i64> {
        points.iter().map(|at| held(*at as usize) as i64).collect()
    };
    let rows = input
        .texts
        .iter()
        .map(|held| TextCutsResultRowsItem {
            length: text::text_length(held) as i64,
            utf8: text::utf8_length(held) as i64,
            before: offsets(&|at| text::cut_before(held, at)),
            after: offsets(&|at| text::cut_after(held, at)),
            heads: each(&|max| text::head_of(held, max).to_owned()),
            tails: each(&|max| text::tail_of(held, max).to_owned()),
            bytes: each(&|max| text::cut_utf8(held, max).to_owned()),
            lower: text::ascii_lower(held),
            upper: text::ascii_upper(held),
        })
        .collect();
    Ok(TextCutsResult { rows })
}

pub fn slices(input: TextSlicesInput) -> Outcome<TextSlicesResult> {
    let slices = input
        .spans
        .iter()
        .map(|span| {
            text::slice_text(&input.text, span.start as usize, span.end as usize).to_owned()
        })
        .collect();
    Ok(TextSlicesResult { slices })
}

pub fn spaces(input: TextSpacesInput) -> Outcome<TextSpacesResult> {
    let each =
        |change: &dyn Fn(&str) -> String| input.texts.iter().map(|held| change(held)).collect();
    Ok(TextSpacesResult {
        trimmed: each(&|held| text::trim_text(held).to_owned()),
        collapsed: each(&|held| text::collapse_space(held)),
    })
}

pub fn find(input: TextFindInput) -> Outcome<TextFindResult> {
    let table = |find: &dyn Fn(&str, usize) -> Option<usize>| -> Vec<Vec<i64>> {
        input
            .needles
            .iter()
            .map(|needle| {
                input
                    .froms
                    .iter()
                    .map(|from| find(needle, *from as usize).map_or(-1, |at| at as i64))
                    .collect()
            })
            .collect()
    };
    Ok(TextFindResult {
        first: table(&|needle, from| text::index_of_text(&input.text, needle, from)),
        last: table(&|needle, from| text::last_index_of_text(&input.text, needle, from)),
    })
}

pub fn nfc(input: TextNfcInput) -> Outcome<TextNfcResult> {
    let texts = input.texts.iter().map(|held| text::nfc(held)).collect();
    Ok(TextNfcResult { texts })
}

pub fn utf8_chunks(input: TextUtf8ChunksInput) -> Outcome<TextUtf8ChunksResult> {
    let mut runs = Vec::new();
    for held in &input.runs {
        let mut decoder = text::Utf8Decoder::new();
        let mut pass = |chunks: &[String]| -> Outcome<(Vec<String>, String)> {
            let mut pushed = Vec::new();
            for chunk in chunks {
                let bytes = base64_decode(chunk).ok_or("a chunk is not base64")?;
                pushed.push(decoder.push(&bytes));
            }
            Ok((pushed, decoder.end()))
        };
        let (pushed, ended) = pass(&held.chunks)?;
        let second = match &held.again {
            Some(chunks) => {
                let (pushed, ended) = pass(chunks)?;
                Some(TextUtf8ChunksResultRunsItemSecond { pushed, ended })
            }
            None => None,
        };
        runs.push(TextUtf8ChunksResultRunsItem {
            pushed,
            ended,
            second,
        });
    }
    Ok(TextUtf8ChunksResult { runs })
}

pub fn path_segment(input: TextPathSegmentInput) -> Outcome<TextPathSegmentResult> {
    let texts = input
        .texts
        .iter()
        .map(|held| text::encode_path_segment(held))
        .collect();
    Ok(TextPathSegmentResult { texts })
}

pub fn has_control(input: TextHasControlInput) -> Outcome<TextHasControlResult> {
    Ok(TextHasControlResult {
        results: input
            .texts
            .iter()
            .map(|held| text::has_control(held))
            .collect(),
    })
}
