//! Number cases.

use nyaaai_contract::number;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn text(input: NumberTextInput) -> Outcome<NumberTextResult> {
    let texts = input
        .values
        .iter()
        .map(|value| number::number_text(*value))
        .collect();
    Ok(NumberTextResult { texts })
}

/// Preserves negative zero.
pub fn round(input: NumberRoundInput) -> Outcome<NumberRoundResult> {
    let texts = input
        .values
        .iter()
        .map(|value| {
            let rounded = number::round_half_up(*value);
            if rounded == 0.0 && rounded.is_sign_negative() {
                "-0".to_owned()
            } else {
                number::number_text(rounded)
            }
        })
        .collect();
    Ok(NumberRoundResult { texts })
}

pub fn fixed(input: NumberFixedInput) -> Outcome<NumberFixedResult> {
    let rows = input
        .values
        .iter()
        .map(|value| {
            input
                .digits
                .iter()
                .map(|digits| number::fixed_text(*value, *digits as usize))
                .collect()
        })
        .collect();
    Ok(NumberFixedResult { rows })
}

pub fn of_text(input: NumberOfTextInput) -> Outcome<NumberOfTextResult> {
    let values = input
        .texts
        .iter()
        .map(|text| number::number_of_text(text))
        .collect();
    Ok(NumberOfTextResult { values })
}

pub fn integer_of_text(input: NumberIntegerOfTextInput) -> Outcome<NumberIntegerOfTextResult> {
    let values = input
        .texts
        .iter()
        .map(|text| number::integer_of_text(text))
        .collect();
    Ok(NumberIntegerOfTextResult { values })
}
