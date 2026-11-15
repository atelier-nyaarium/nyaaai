//! Strict schema cases.

use nyaaai_contract::json::{Value, parse_json};
use nyaaai_contract::shape::PathPart;
use nyaaai_contract::shapes::frames;
use nyaaai_contract::strict_schema::{self, Issue};
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

fn strict_schema_issue(issue: Issue) -> StrictSchemaIssue {
    StrictSchemaIssue {
        path: issue
            .path
            .into_iter()
            .map(|part| match part {
                PathPart::Key(key) => frames::PathPart::Text(key),
                PathPart::Index(index) => frames::PathPart::Integer(index as u64),
            })
            .collect(),
        code: StrictSchemaIssueCode::parse(issue.code.as_str()).expect("shared issue code"),
        message: issue.message,
    }
}

fn issues_of(schema: &Value, text: &str) -> Outcome<Vec<StrictSchemaIssue>> {
    let value = parse_json(text).ok_or("a value text is not JSON")?;
    Ok(strict_schema::validate(schema, &value)
        .into_iter()
        .map(strict_schema_issue)
        .collect())
}

pub fn normalize(input: StrictSchemaNormalizeInput) -> Outcome<StrictSchemaNormalizeResult> {
    Ok(StrictSchemaNormalizeResult {
        schemas: input
            .schemas
            .iter()
            .map(strict_schema::normalize_schema)
            .collect(),
    })
}

pub fn problems(input: StrictSchemaProblemsInput) -> Outcome<StrictSchemaProblemsResult> {
    Ok(StrictSchemaProblemsResult {
        problems: input
            .schemas
            .iter()
            .map(|schema| {
                strict_schema::schema_problems(&strict_schema::normalize_schema(schema))
                    .into_iter()
                    .map(strict_schema_issue)
                    .collect()
            })
            .collect(),
    })
}

pub fn validate(input: StrictSchemaValidateInput) -> Outcome<StrictSchemaValidateResult> {
    let issues = input
        .texts
        .iter()
        .map(|text| issues_of(&input.schema, text))
        .collect::<Outcome<_>>()?;
    Ok(StrictSchemaValidateResult { issues })
}

pub fn validate_list(
    input: StrictSchemaValidateListInput,
) -> Outcome<StrictSchemaValidateListResult> {
    let issues = input
        .counts
        .iter()
        .map(|count| {
            let items = vec![input.item.as_str(); *count as usize].join(",");
            issues_of(&input.schema, &format!("[{items}]"))
        })
        .collect::<Outcome<_>>()?;
    Ok(StrictSchemaValidateListResult { issues })
}
