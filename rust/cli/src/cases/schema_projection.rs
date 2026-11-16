//! Schema projection cases.

use nyaaai_agentic::schema_projection::{
    clean_schema, nullable_optionals, strict_schema, without_nulls,
};
use nyaaai_contract::json::Value;
use nyaaai_testkit::cases::Outcome;

use super::shapes::*;

pub fn project(input: SchemaProjectionProjectInput) -> Outcome<SchemaProjectionProjectResult> {
    let schema = &input.schema;
    let output = match input.projection {
        SchemaProjectionProjectInputProjection::Clean => Ok(Value::Object(clean_schema(schema))),
        SchemaProjectionProjectInputProjection::Nullable => {
            Ok(Value::Object(nullable_optionals(schema)))
        }
        SchemaProjectionProjectInputProjection::WithoutNulls => {
            Ok(without_nulls(schema, &input.value))
        }
        SchemaProjectionProjectInputProjection::Strict => strict_schema(schema).map(Value::Object),
    };
    Ok(match output {
        Ok(output) => SchemaProjectionProjectResult {
            output,
            refusal: None,
        },
        Err(refusal) => SchemaProjectionProjectResult {
            output: Value::Null,
            refusal: Some(refusal),
        },
    })
}
