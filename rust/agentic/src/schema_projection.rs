//! Provider JSON schema projections.

use nyaaai_contract::json::{Object, Value};

/// JSON Schema nodes.
pub type Schema = Object;

fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(held) => *held,
        Value::Number(number) => *number != 0.0 && !number.is_nan(),
        Value::String(text) => !text.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

fn without_key(object: &Object, key: &str) -> Object {
    let mut out = Object::new();
    for (held, field) in object.iter() {
        if held != key {
            out.insert(held, field.clone());
        }
    }
    out
}

fn remove_format(node: &Value) -> Value {
    match node {
        Value::Array(items) => Value::Array(items.iter().map(remove_format).collect()),
        Value::Object(object) => {
            let mut out = Object::new();
            for (key, field) in object.iter() {
                let uri = matches!(field.as_str(), Some("uri" | "uri-reference"));
                if key != "format" || !uri {
                    out.insert(key, remove_format(field));
                }
            }
            Value::Object(out)
        }
        other => other.clone(),
    }
}

/// Removes unsupported formats and closes the root object.
pub fn clean_schema(schema: &Schema) -> Schema {
    let mut out = schema.clone();
    if schema.get("$schema").is_some_and(truthy) {
        out = without_key(&out, "$schema");
    }
    if out.get("type").and_then(Value::as_str) == Some("object") {
        out.insert("additionalProperties", Value::Bool(false));
        if let Value::Object(cleaned) = remove_format(&Value::Object(out.clone())) {
            out = cleaned;
        }
    }
    out
}

fn required_of(schema: &Schema) -> Vec<&str> {
    match schema.get("required") {
        Some(Value::Array(items)) => items.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    }
}

fn branches_of(schema: &Schema) -> Vec<&Schema> {
    let branches = match schema.get("anyOf") {
        None | Some(Value::Null) => schema.get("oneOf"),
        held => held,
    };
    match branches {
        Some(Value::Array(items)) => items.iter().filter_map(Value::as_object).collect(),
        _ => Vec::new(),
    }
}

fn accepts_null(schema: &Schema) -> bool {
    let type_ = schema.get("type");
    match type_ {
        Some(Value::String(text)) if text == "null" => return true,
        Some(Value::Array(types)) if types.iter().any(|held| held.as_str() == Some("null")) => {
            return true;
        }
        _ => {}
    }
    let branches = branches_of(schema);
    if !branches.is_empty() {
        return branches.into_iter().any(accepts_null);
    }
    if let Some(Value::Array(values)) = schema.get("enum") {
        return values.iter().any(Value::is_null);
    }
    if let Some(held) = schema.get("const") {
        return held.is_null();
    }
    type_.is_none()
}

fn or_null(schema: &Schema) -> Schema {
    if let Some(Value::String(type_)) = schema.get("type")
        && schema.get("enum").is_none()
        && schema.get("const").is_none()
    {
        let mut out = schema.clone();
        out.insert(
            "type",
            Value::Array(vec![Value::from(type_.as_str()), Value::from("null")]),
        );
        return out;
    }
    let mut null = Object::new();
    null.insert("type", Value::from("null"));
    let mut out = Object::new();
    out.insert(
        "anyOf",
        Value::Array(vec![
            Value::Object(without_key(schema, "description")),
            Value::Object(null),
        ]),
    );
    if let Some(description) = schema.get("description") {
        out.insert("description", description.clone());
    }
    out
}

/// Models fill all fields.
pub fn nullable_optionals(schema: &Schema) -> Schema {
    let mut out = schema.clone();
    if let Some(items) = schema.get("items").and_then(Value::as_object) {
        out.insert("items", Value::Object(nullable_optionals(items)));
    }
    for key in ["anyOf", "oneOf"] {
        if let Some(Value::Array(branches)) = schema.get(key) {
            let mapped = branches
                .iter()
                .map(|branch| match branch {
                    Value::Object(object) => Value::Object(nullable_optionals(object)),
                    other => other.clone(),
                })
                .collect();
            out.insert(key, Value::Array(mapped));
        }
    }
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return out;
    };
    let required = required_of(schema);
    let mut fields = Object::new();
    for (key, field) in properties.iter() {
        let Value::Object(field) = field else {
            fields.insert(key, field.clone());
            continue;
        };
        let inner = nullable_optionals(field);
        let held = if required.contains(&key) || accepts_null(&inner) {
            inner
        } else {
            or_null(&inner)
        };
        fields.insert(key, Value::Object(held));
    }
    out.insert("properties", Value::Object(fields));
    out.insert(
        "required",
        Value::Array(properties.keys().map(Value::from).collect()),
    );
    out
}

/// Drops each null `nullable_optionals` allowed in a tool call; a union node passes as sent.
pub fn without_nulls(schema: &Schema, value: &Value) -> Value {
    match value {
        Value::Array(items) => match schema.get("items").and_then(Value::as_object) {
            Some(each) => {
                Value::Array(items.iter().map(|item| without_nulls(each, item)).collect())
            }
            None => value.clone(),
        },
        Value::Object(object) => {
            let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
                return value.clone();
            };
            let required = required_of(schema);
            let mut out = Object::new();
            for (key, field) in object.iter() {
                let Some(Value::Object(inner)) = properties.get(key) else {
                    out.insert(key, field.clone());
                    continue;
                };
                if field.is_null() && !required.contains(&key) && !accepts_null(inner) {
                    continue;
                }
                out.insert(key, without_nulls(inner, field));
            }
            Value::Object(out)
        }
        other => other.clone(),
    }
}

/// Closes objects, uses `anyOf`; refuses what strict mode cannot carry.
fn closed(schema: &Schema) -> Result<Schema, String> {
    for key in ["$ref", "$defs", "prefixItems"] {
        if schema.get(key).is_some() {
            return Err(format!(
                "A strict json_schema cannot carry `{key}`; reshape the answer without it."
            ));
        }
    }
    if matches!(schema.get("additionalProperties"), Some(Value::Object(_))) {
        return Err(
            "A strict json_schema cannot carry an open record; name its keys as properties."
                .to_owned(),
        );
    }
    let required = required_of(schema);
    let properties = schema.get("properties").and_then(Value::as_object);
    let optional = properties.and_then(|held| held.keys().find(|key| !required.contains(key)));
    if let Some(optional) = optional {
        return Err(format!(
            "A strict json_schema cannot carry an optional field `{optional}`; make it `.nullable()`, which is what strict mode sends."
        ));
    }
    let mut out = schema.clone();
    if let Some(items) = schema.get("items").and_then(Value::as_object) {
        out.insert("items", Value::Object(closed(items)?));
    }
    let branches = branches_of(schema);
    if !branches.is_empty() {
        out = without_key(&out, "oneOf");
        let mapped = branches
            .into_iter()
            .map(|branch| closed(branch).map(Value::Object))
            .collect::<Result<Vec<_>, _>>()?;
        out.insert("anyOf", Value::Array(mapped));
    }
    if let Some(properties) = properties {
        let mut fields = Object::new();
        for (key, field) in properties.iter() {
            let held = match field {
                Value::Object(node) => Value::Object(closed(node)?),
                other => other.clone(),
            };
            fields.insert(key, held);
        }
        out.insert("properties", Value::Object(fields));
        out.insert("additionalProperties", Value::Bool(false));
    }
    Ok(out)
}

/// What a strict `json_schema` sends. An empty field is `.nullable()`, as the model answers it.
pub fn strict_schema(schema: &Schema) -> Result<Schema, String> {
    closed(&clean_schema(schema))
}
