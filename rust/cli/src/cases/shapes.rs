//! Generated from schema/cases.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use nyaaai_contract::json::{Object, Record, Value};
use nyaaai_contract::shape::{self, Path, Refused, Shape};

#[derive(Debug, Clone, PartialEq)]
pub struct JsonRoundTripInput {
    pub texts: Vec<String>,
}

impl Shape for JsonRoundTripInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonRoundTripResult {
    pub results: Vec<JsonRoundTripResultResultsItem>,
}

impl Shape for JsonRoundTripResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let results = shape::required(object, "results", at, |value, at| {
            shape::list(value, at, JsonRoundTripResultResultsItem::decode_at)
        })?;
        Ok(Self { results })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "results",
            Value::Array(self.results.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonRoundTripResultResultsItem {
    pub ok: bool,
    pub text: Option<String>,
}

impl Shape for JsonRoundTripResultResultsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let ok = shape::required(object, "ok", at, shape::flag)?;
        let text = shape::optional(object, "text", at, shape::text)?;
        Ok(Self { ok, text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("ok", Value::Bool(self.ok));
        if let Some(held) = &self.text {
            object.insert("text", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonCanonicalInput {
    pub texts: Vec<String>,
}

impl Shape for JsonCanonicalInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsonCanonicalResult {
    pub texts: Vec<Option<String>>,
}

impl Shape for JsonCanonicalResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsStringInput {
    pub texts: Vec<String>,
}

impl Shape for JsStringInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsStringResult {
    pub strings: Vec<String>,
}

impl Shape for JsStringResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let strings = shape::required(object, "strings", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { strings })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "strings",
            Value::Array(
                self.strings
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderSortedTextsInput {
    pub texts: Vec<String>,
}

impl Shape for OrderSortedTextsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderSortedTextsResult {
    pub texts: Vec<String>,
}

impl Shape for OrderSortedTextsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Base64DecodeInput {
    pub texts: Vec<String>,
}

impl Shape for Base64DecodeInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Base64DecodeResult {
    pub texts: Vec<Option<String>>,
}

impl Shape for Base64DecodeResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeadersOfInput {
    pub headers: Vec<(String, String)>,
    pub name: String,
}

impl Shape for HeadersOfInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let headers = shape::required(object, "headers", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::tuple(value, at, 2).and_then(|items| {
                    Ok((
                        shape::place(items, 0, at, shape::text)?,
                        shape::place(items, 1, at, shape::text)?,
                    ))
                })
            })
        })?;
        let name = shape::required(object, "name", at, shape::text)?;
        Ok(Self { headers, name })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "headers",
            Value::Array(
                self.headers
                    .iter()
                    .map(|held| {
                        Value::Array(vec![
                            Value::from(held.0.as_str()),
                            Value::from(held.1.as_str()),
                        ])
                    })
                    .collect(),
            ),
        );
        object.insert("name", Value::from(self.name.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeadersOfResult {
    pub value: Option<String>,
}

impl Shape for HeadersOfResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let value_field = shape::required(object, "value", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self { value: value_field })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "value",
            self.value
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeadersResponseInput {
    pub lists: Vec<Vec<(String, String)>>,
}

impl Shape for HeadersResponseInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let lists = shape::required(object, "lists", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, |value, at| {
                    shape::tuple(value, at, 2).and_then(|items| {
                        Ok((
                            shape::place(items, 0, at, shape::text)?,
                            shape::place(items, 1, at, shape::text)?,
                        ))
                    })
                })
            })
        })?;
        Ok(Self { lists })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "lists",
            Value::Array(
                self.lists
                    .iter()
                    .map(|held| {
                        Value::Array(
                            held.iter()
                                .map(|held| {
                                    Value::Array(vec![
                                        Value::from(held.0.as_str()),
                                        Value::from(held.1.as_str()),
                                    ])
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HeadersResponseResult {
    pub lists: Vec<Vec<(String, String)>>,
}

impl Shape for HeadersResponseResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let lists = shape::required(object, "lists", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, |value, at| {
                    shape::tuple(value, at, 2).and_then(|items| {
                        Ok((
                            shape::place(items, 0, at, shape::text)?,
                            shape::place(items, 1, at, shape::text)?,
                        ))
                    })
                })
            })
        })?;
        Ok(Self { lists })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "lists",
            Value::Array(
                self.lists
                    .iter()
                    .map(|held| {
                        Value::Array(
                            held.iter()
                                .map(|held| {
                                    Value::Array(vec![
                                        Value::from(held.0.as_str()),
                                        Value::from(held.1.as_str()),
                                    ])
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapeCompatibleVersionsInput {
    pub pairs: Vec<ShapeCompatibleVersionsInputPairsItem>,
}

impl Shape for ShapeCompatibleVersionsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let pairs = shape::required(object, "pairs", at, |value, at| {
            shape::list(value, at, ShapeCompatibleVersionsInputPairsItem::decode_at)
        })?;
        Ok(Self { pairs })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "pairs",
            Value::Array(self.pairs.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapeCompatibleVersionsInputPairsItem {
    pub left: String,
    pub right: String,
}

impl Shape for ShapeCompatibleVersionsInputPairsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let left = shape::required(object, "left", at, shape::text)?;
        let right = shape::required(object, "right", at, shape::text)?;
        Ok(Self { left, right })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("left", Value::from(self.left.as_str()));
        object.insert("right", Value::from(self.right.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapeCompatibleVersionsResult {
    pub compatible: Vec<bool>,
}

impl Shape for ShapeCompatibleVersionsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let compatible = shape::required(object, "compatible", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        Ok(Self { compatible })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "compatible",
            Value::Array(
                self.compatible
                    .iter()
                    .map(|held| Value::Bool(*held))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractRoundTripInput {
    pub set: String,
    pub shape: String,
    pub texts: Vec<String>,
}

impl Shape for ContractRoundTripInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let set = shape::required(object, "set", at, shape::text)?;
        let shape_field = shape::required(object, "shape", at, shape::text)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            set,
            shape: shape_field,
            texts,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("set", Value::from(self.set.as_str()));
        object.insert("shape", Value::from(self.shape.as_str()));
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractRoundTripResult {
    pub results: Vec<ContractRoundTripResultResultsItem>,
}

impl Shape for ContractRoundTripResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let results = shape::required(object, "results", at, |value, at| {
            shape::list(value, at, ContractRoundTripResultResultsItem::decode_at)
        })?;
        Ok(Self { results })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "results",
            Value::Array(self.results.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContractRoundTripResultResultsItem {
    pub ok: bool,
    pub text: Option<String>,
    pub path: Option<Vec<nyaaai_contract::shapes::frames::PathPart>>,
    pub message: Option<String>,
}

impl Shape for ContractRoundTripResultResultsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let ok = shape::required(object, "ok", at, shape::flag)?;
        let text = shape::optional(object, "text", at, shape::text)?;
        let path = shape::optional(object, "path", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::frames::PathPart::decode_at,
            )
        })?;
        let message = shape::optional(object, "message", at, shape::text)?;
        Ok(Self {
            ok,
            text,
            path,
            message,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("ok", Value::Bool(self.ok));
        if let Some(held) = &self.text {
            object.insert("text", Value::from(held.as_str()));
        }
        if let Some(held) = &self.path {
            object.insert(
                "path",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        if let Some(held) = &self.message {
            object.insert("message", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeParseIsoInput {
    pub texts: Vec<String>,
}

impl Shape for TimeParseIsoInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeParseIsoResult {
    pub results: Vec<Option<i64>>,
}

impl Shape for TimeParseIsoResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let results = shape::required(object, "results", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::integer)
            })
        })?;
        Ok(Self { results })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "results",
            Value::Array(
                self.results
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held as f64))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeIsoInput {
    pub times: Vec<i64>,
}

impl Shape for TimeIsoInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let times = shape::required(object, "times", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        Ok(Self { times })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "times",
            Value::Array(
                self.times
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeIsoResult {
    pub texts: Vec<Option<String>>,
}

impl Shape for TimeIsoResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeHttpDateInput {
    pub texts: Vec<String>,
    pub now_ms: i64,
}

impl Shape for TimeHttpDateInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        Ok(Self { texts, now_ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeHttpDateResult {
    pub results: Vec<Option<i64>>,
}

impl Shape for TimeHttpDateResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let results = shape::required(object, "results", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::integer)
            })
        })?;
        Ok(Self { results })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "results",
            Value::Array(
                self.results
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held as f64))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeRetryAfterInput {
    pub headers: Vec<Option<String>>,
    pub now_ms: i64,
}

impl Shape for TimeRetryAfterInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let headers = shape::required(object, "headers", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        Ok(Self { headers, now_ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "headers",
            Value::Array(
                self.headers
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeRetryAfterResult {
    pub results: Vec<Option<i64>>,
}

impl Shape for TimeRetryAfterResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let results = shape::required(object, "results", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::integer)
            })
        })?;
        Ok(Self { results })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "results",
            Value::Array(
                self.results
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held as f64))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeResetStampInput {
    pub values: Vec<Value>,
    pub now_ms: i64,
}

impl Shape for TimeResetStampInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        Ok(Self { values, now_ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("values", Value::Array(self.values.clone()));
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeResetStampResult {
    pub results: Vec<Option<i64>>,
}

impl Shape for TimeResetStampResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let results = shape::required(object, "results", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::integer)
            })
        })?;
        Ok(Self { results })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "results",
            Value::Array(
                self.results
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held as f64))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeDayKeyInput {
    pub times: Vec<i64>,
    pub offsets: Vec<i64>,
}

impl Shape for TimeDayKeyInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let times = shape::required(object, "times", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let offsets = shape::required(object, "offsets", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        Ok(Self { times, offsets })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "times",
            Value::Array(
                self.times
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "offsets",
            Value::Array(
                self.offsets
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeDayKeyResult {
    pub rows: Vec<Vec<Option<String>>>,
}

impl Shape for TimeDayKeyResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, |value, at| {
                    shape::nullable(value, at, shape::text)
                })
            })
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| {
                        Value::Array(
                            held.iter()
                                .map(|held| {
                                    held.as_ref()
                                        .map_or(Value::Null, |held| Value::from(held.as_str()))
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberTextInput {
    pub values: Vec<f64>,
}

impl Shape for NumberTextInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::number)
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(
                self.values
                    .iter()
                    .map(|held| Value::Number(*held))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberTextResult {
    pub texts: Vec<String>,
}

impl Shape for NumberTextResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberRoundInput {
    pub values: Vec<f64>,
}

impl Shape for NumberRoundInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::number)
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(
                self.values
                    .iter()
                    .map(|held| Value::Number(*held))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberRoundResult {
    pub texts: Vec<String>,
}

impl Shape for NumberRoundResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberFixedInput {
    pub values: Vec<f64>,
    pub digits: Vec<u64>,
}

impl Shape for NumberFixedInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::number)
        })?;
        let digits = shape::required(object, "digits", at, |value, at| {
            shape::list(value, at, shape::natural)
        })?;
        Ok(Self { values, digits })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(
                self.values
                    .iter()
                    .map(|held| Value::Number(*held))
                    .collect(),
            ),
        );
        object.insert(
            "digits",
            Value::Array(
                self.digits
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberFixedResult {
    pub rows: Vec<Vec<String>>,
}

impl Shape for NumberFixedResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| {
                        Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberOfTextInput {
    pub texts: Vec<String>,
}

impl Shape for NumberOfTextInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberOfTextResult {
    pub values: Vec<Option<f64>>,
}

impl Shape for NumberOfTextResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::number)
            })
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(
                self.values
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberIntegerOfTextInput {
    pub texts: Vec<String>,
}

impl Shape for NumberIntegerOfTextInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumberIntegerOfTextResult {
    pub values: Vec<Option<i64>>,
}

impl Shape for NumberIntegerOfTextResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::integer)
            })
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(
                self.values
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held as f64))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextCutsInput {
    pub texts: Vec<String>,
    pub points: Vec<u64>,
}

impl Shape for TextCutsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let points = shape::required(object, "points", at, |value, at| {
            shape::list(value, at, shape::natural)
        })?;
        Ok(Self { texts, points })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "points",
            Value::Array(
                self.points
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextCutsResult {
    pub rows: Vec<TextCutsResultRowsItem>,
}

impl Shape for TextCutsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, TextCutsResultRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextCutsResultRowsItem {
    pub length: i64,
    pub utf8: i64,
    pub before: Vec<i64>,
    pub after: Vec<i64>,
    pub heads: Vec<String>,
    pub tails: Vec<String>,
    pub bytes: Vec<String>,
    pub lower: String,
    pub upper: String,
}

impl Shape for TextCutsResultRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let length = shape::required(object, "length", at, shape::integer)?;
        let utf8 = shape::required(object, "utf8", at, shape::integer)?;
        let before = shape::required(object, "before", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let after = shape::required(object, "after", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let heads = shape::required(object, "heads", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let tails = shape::required(object, "tails", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let bytes = shape::required(object, "bytes", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let lower = shape::required(object, "lower", at, shape::text)?;
        let upper = shape::required(object, "upper", at, shape::text)?;
        Ok(Self {
            length,
            utf8,
            before,
            after,
            heads,
            tails,
            bytes,
            lower,
            upper,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("length", Value::Number(self.length as f64));
        object.insert("utf8", Value::Number(self.utf8 as f64));
        object.insert(
            "before",
            Value::Array(
                self.before
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "after",
            Value::Array(
                self.after
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "heads",
            Value::Array(
                self.heads
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "tails",
            Value::Array(
                self.tails
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "bytes",
            Value::Array(
                self.bytes
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("lower", Value::from(self.lower.as_str()));
        object.insert("upper", Value::from(self.upper.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSlicesInput {
    pub text: String,
    pub spans: Vec<TextSlicesInputSpansItem>,
}

impl Shape for TextSlicesInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let text = shape::required(object, "text", at, shape::text)?;
        let spans = shape::required(object, "spans", at, |value, at| {
            shape::list(value, at, TextSlicesInputSpansItem::decode_at)
        })?;
        Ok(Self { text, spans })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("text", Value::from(self.text.as_str()));
        object.insert(
            "spans",
            Value::Array(self.spans.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSlicesInputSpansItem {
    pub start: u64,
    pub end: u64,
}

impl Shape for TextSlicesInputSpansItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start = shape::required(object, "start", at, shape::natural)?;
        let end = shape::required(object, "end", at, shape::natural)?;
        Ok(Self { start, end })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("start", Value::Number(self.start as f64));
        object.insert("end", Value::Number(self.end as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSlicesResult {
    pub slices: Vec<String>,
}

impl Shape for TextSlicesResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let slices = shape::required(object, "slices", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { slices })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "slices",
            Value::Array(
                self.slices
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSpacesInput {
    pub texts: Vec<String>,
}

impl Shape for TextSpacesInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextSpacesResult {
    pub trimmed: Vec<String>,
    pub collapsed: Vec<String>,
}

impl Shape for TextSpacesResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let trimmed = shape::required(object, "trimmed", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let collapsed = shape::required(object, "collapsed", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { trimmed, collapsed })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "trimmed",
            Value::Array(
                self.trimmed
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "collapsed",
            Value::Array(
                self.collapsed
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextFindInput {
    pub text: String,
    pub needles: Vec<String>,
    pub froms: Vec<u64>,
}

impl Shape for TextFindInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let text = shape::required(object, "text", at, shape::text)?;
        let needles = shape::required(object, "needles", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let froms = shape::required(object, "froms", at, |value, at| {
            shape::list(value, at, shape::natural)
        })?;
        Ok(Self {
            text,
            needles,
            froms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("text", Value::from(self.text.as_str()));
        object.insert(
            "needles",
            Value::Array(
                self.needles
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "froms",
            Value::Array(
                self.froms
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextFindResult {
    pub first: Vec<Vec<i64>>,
    pub last: Vec<Vec<i64>>,
}

impl Shape for TextFindResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let first = shape::required(object, "first", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, shape::integer)
            })
        })?;
        let last = shape::required(object, "last", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, shape::integer)
            })
        })?;
        Ok(Self { first, last })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "first",
            Value::Array(
                self.first
                    .iter()
                    .map(|held| {
                        Value::Array(
                            held.iter()
                                .map(|held| Value::Number(*held as f64))
                                .collect(),
                        )
                    })
                    .collect(),
            ),
        );
        object.insert(
            "last",
            Value::Array(
                self.last
                    .iter()
                    .map(|held| {
                        Value::Array(
                            held.iter()
                                .map(|held| Value::Number(*held as f64))
                                .collect(),
                        )
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextUtf8ChunksInput {
    pub runs: Vec<TextUtf8ChunksInputRunsItem>,
}

impl Shape for TextUtf8ChunksInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let runs = shape::required(object, "runs", at, |value, at| {
            shape::list(value, at, TextUtf8ChunksInputRunsItem::decode_at)
        })?;
        Ok(Self { runs })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "runs",
            Value::Array(self.runs.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextUtf8ChunksInputRunsItem {
    pub chunks: Vec<String>,
    pub again: Option<Vec<String>>,
}

impl Shape for TextUtf8ChunksInputRunsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let chunks = shape::required(object, "chunks", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let again = shape::optional(object, "again", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { chunks, again })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "chunks",
            Value::Array(
                self.chunks
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        if let Some(held) = &self.again {
            object.insert(
                "again",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextUtf8ChunksResult {
    pub runs: Vec<TextUtf8ChunksResultRunsItem>,
}

impl Shape for TextUtf8ChunksResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let runs = shape::required(object, "runs", at, |value, at| {
            shape::list(value, at, TextUtf8ChunksResultRunsItem::decode_at)
        })?;
        Ok(Self { runs })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "runs",
            Value::Array(self.runs.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextUtf8ChunksResultRunsItem {
    pub pushed: Vec<String>,
    pub ended: String,
    pub second: Option<TextUtf8ChunksResultRunsItemSecond>,
}

impl Shape for TextUtf8ChunksResultRunsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let pushed = shape::required(object, "pushed", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let ended = shape::required(object, "ended", at, shape::text)?;
        let second = shape::optional(
            object,
            "second",
            at,
            TextUtf8ChunksResultRunsItemSecond::decode_at,
        )?;
        Ok(Self {
            pushed,
            ended,
            second,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "pushed",
            Value::Array(
                self.pushed
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("ended", Value::from(self.ended.as_str()));
        if let Some(held) = &self.second {
            object.insert("second", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextUtf8ChunksResultRunsItemSecond {
    pub pushed: Vec<String>,
    pub ended: String,
}

impl Shape for TextUtf8ChunksResultRunsItemSecond {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let pushed = shape::required(object, "pushed", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let ended = shape::required(object, "ended", at, shape::text)?;
        Ok(Self { pushed, ended })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "pushed",
            Value::Array(
                self.pushed
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("ended", Value::from(self.ended.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextHasControlInput {
    pub texts: Vec<String>,
}

impl Shape for TextHasControlInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextHasControlResult {
    pub results: Vec<bool>,
}

impl Shape for TextHasControlResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let results = shape::required(object, "results", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        Ok(Self { results })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "results",
            Value::Array(self.results.iter().map(|held| Value::Bool(*held)).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPathSegmentInput {
    pub texts: Vec<String>,
}

impl Shape for TextPathSegmentInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextPathSegmentResult {
    pub texts: Vec<String>,
}

impl Shape for TextPathSegmentResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextNfcInput {
    pub texts: Vec<String>,
}

impl Shape for TextNfcInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextNfcResult {
    pub texts: Vec<String>,
}

impl Shape for TextNfcResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UrlOriginInput {
    pub urls: Vec<String>,
}

impl Shape for UrlOriginInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let urls = shape::required(object, "urls", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { urls })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "urls",
            Value::Array(
                self.urls
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UrlOriginResult {
    pub origins: Vec<Option<String>>,
}

impl Shape for UrlOriginResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let origins = shape::required(object, "origins", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { origins })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "origins",
            Value::Array(
                self.origins
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UrlPartsInput {
    pub urls: Vec<String>,
}

impl Shape for UrlPartsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let urls = shape::required(object, "urls", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { urls })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "urls",
            Value::Array(
                self.urls
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UrlPartsResult {
    pub parts: Vec<Option<UrlPartsResultPartsItem>>,
}

impl Shape for UrlPartsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let parts = shape::required(object, "parts", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, UrlPartsResultPartsItem::decode_at)
            })
        })?;
        Ok(Self { parts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "parts",
            Value::Array(
                self.parts
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UrlPartsResultPartsItem {
    pub scheme: String,
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub origin: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
}

impl Shape for UrlPartsResultPartsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let scheme = shape::required(object, "scheme", at, shape::text)?;
        let username = shape::required(object, "username", at, shape::text)?;
        let password = shape::required(object, "password", at, shape::text)?;
        let host = shape::required(object, "host", at, shape::text)?;
        let port = shape::required(object, "port", at, shape::text)?;
        let origin = shape::required(object, "origin", at, shape::text)?;
        let path = shape::required(object, "path", at, shape::text)?;
        let query = shape::required(object, "query", at, shape::text)?;
        let fragment = shape::required(object, "fragment", at, shape::text)?;
        Ok(Self {
            scheme,
            username,
            password,
            host,
            port,
            origin,
            path,
            query,
            fragment,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("scheme", Value::from(self.scheme.as_str()));
        object.insert("username", Value::from(self.username.as_str()));
        object.insert("password", Value::from(self.password.as_str()));
        object.insert("host", Value::from(self.host.as_str()));
        object.insert("port", Value::from(self.port.as_str()));
        object.insert("origin", Value::from(self.origin.as_str()));
        object.insert("path", Value::from(self.path.as_str()));
        object.insert("query", Value::from(self.query.as_str()));
        object.insert("fragment", Value::from(self.fragment.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternCaseMatch {
    pub start: u64,
    pub end: u64,
    pub text: String,
    pub captures: Vec<Option<String>>,
}

impl Shape for PatternCaseMatch {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start = shape::required(object, "start", at, shape::natural)?;
        let end = shape::required(object, "end", at, shape::natural)?;
        let text = shape::required(object, "text", at, shape::text)?;
        let captures = shape::required(object, "captures", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self {
            start,
            end,
            text,
            captures,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("start", Value::Number(self.start as f64));
        object.insert("end", Value::Number(self.end as f64));
        object.insert("text", Value::from(self.text.as_str()));
        object.insert(
            "captures",
            Value::Array(
                self.captures
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternProblemsInput {
    pub patterns: Vec<PatternProblemsInputPatternsItem>,
}

impl Shape for PatternProblemsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let patterns = shape::required(object, "patterns", at, |value, at| {
            shape::list(value, at, PatternProblemsInputPatternsItem::decode_at)
        })?;
        Ok(Self { patterns })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "patterns",
            Value::Array(self.patterns.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternProblemsInputPatternsItem {
    pub source: String,
    pub flags: String,
}

impl Shape for PatternProblemsInputPatternsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let source = shape::required(object, "source", at, shape::text)?;
        let flags = shape::required(object, "flags", at, shape::text)?;
        Ok(Self { source, flags })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("source", Value::from(self.source.as_str()));
        object.insert("flags", Value::from(self.flags.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternProblemsResult {
    pub refused: Vec<bool>,
}

impl Shape for PatternProblemsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let refused = shape::required(object, "refused", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        Ok(Self { refused })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "refused",
            Value::Array(self.refused.iter().map(|held| Value::Bool(*held)).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternMatchesInput {
    pub source: String,
    pub flags: String,
    pub texts: Vec<String>,
}

impl Shape for PatternMatchesInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let source = shape::required(object, "source", at, shape::text)?;
        let flags = shape::required(object, "flags", at, shape::text)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            source,
            flags,
            texts,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("source", Value::from(self.source.as_str()));
        object.insert("flags", Value::from(self.flags.as_str()));
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternMatchesResult {
    pub results: Vec<PatternMatchesResultResultsItem>,
}

impl Shape for PatternMatchesResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let results = shape::required(object, "results", at, |value, at| {
            shape::list(value, at, PatternMatchesResultResultsItem::decode_at)
        })?;
        Ok(Self { results })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "results",
            Value::Array(self.results.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternMatchesResultResultsItem {
    pub test: bool,
    pub all: Vec<PatternCaseMatch>,
}

impl Shape for PatternMatchesResultResultsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let test = shape::required(object, "test", at, shape::flag)?;
        let all = shape::required(object, "all", at, |value, at| {
            shape::list(value, at, PatternCaseMatch::decode_at)
        })?;
        Ok(Self { test, all })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("test", Value::Bool(self.test));
        object.insert(
            "all",
            Value::Array(self.all.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternFindInput {
    pub source: String,
    pub flags: String,
    pub text: String,
    pub froms: Vec<u64>,
}

impl Shape for PatternFindInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let source = shape::required(object, "source", at, shape::text)?;
        let flags = shape::required(object, "flags", at, shape::text)?;
        let text = shape::required(object, "text", at, shape::text)?;
        let froms = shape::required(object, "froms", at, |value, at| {
            shape::list(value, at, shape::natural)
        })?;
        Ok(Self {
            source,
            flags,
            text,
            froms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("source", Value::from(self.source.as_str()));
        object.insert("flags", Value::from(self.flags.as_str()));
        object.insert("text", Value::from(self.text.as_str()));
        object.insert(
            "froms",
            Value::Array(
                self.froms
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternFindResult {
    pub found: Vec<Option<PatternCaseMatch>>,
}

impl Shape for PatternFindResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let found = shape::required(object, "found", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, PatternCaseMatch::decode_at)
            })
        })?;
        Ok(Self { found })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "found",
            Value::Array(
                self.found
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternEditsInput {
    pub source: String,
    pub flags: String,
    pub template: String,
    pub texts: Vec<String>,
}

impl Shape for PatternEditsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let source = shape::required(object, "source", at, shape::text)?;
        let flags = shape::required(object, "flags", at, shape::text)?;
        let template = shape::required(object, "template", at, shape::text)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            source,
            flags,
            template,
            texts,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("source", Value::from(self.source.as_str()));
        object.insert("flags", Value::from(self.flags.as_str()));
        object.insert("template", Value::from(self.template.as_str()));
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternEditsResult {
    pub results: Vec<PatternEditsResultResultsItem>,
}

impl Shape for PatternEditsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let results = shape::required(object, "results", at, |value, at| {
            shape::list(value, at, PatternEditsResultResultsItem::decode_at)
        })?;
        Ok(Self { results })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "results",
            Value::Array(self.results.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatternEditsResultResultsItem {
    pub replaced: String,
    pub pieces: Vec<String>,
}

impl Shape for PatternEditsResultResultsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let replaced = shape::required(object, "replaced", at, shape::text)?;
        let pieces = shape::required(object, "pieces", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { replaced, pieces })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("replaced", Value::from(self.replaced.as_str()));
        object.insert(
            "pieces",
            Value::Array(
                self.pieces
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSchemaIssue {
    pub path: Vec<nyaaai_contract::shapes::frames::PathPart>,
    pub code: StrictSchemaIssueCode,
    pub message: String,
}

impl Shape for StrictSchemaIssue {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let path = shape::required(object, "path", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::frames::PathPart::decode_at,
            )
        })?;
        let code = shape::required(object, "code", at, StrictSchemaIssueCode::decode_at)?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self {
            path,
            code,
            message,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "path",
            Value::Array(self.path.iter().map(|held| held.encode()).collect()),
        );
        object.insert("code", self.code.encode());
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrictSchemaIssueCode {
    Type,
    Enum,
    Const,
    Minimum,
    ExclusiveMinimum,
    Maximum,
    ExclusiveMaximum,
    MinLength,
    MaxLength,
    MinItems,
    MaxItems,
    Required,
    Additional,
    PropertyName,
    NoBranch,
    ManyBranches,
    UnknownKeyword,
    BadKeyword,
    BadReference,
    ReferenceLoop,
    Limit,
}

impl StrictSchemaIssueCode {
    pub const ALL: [Self; 21] = [
        Self::Type,
        Self::Enum,
        Self::Const,
        Self::Minimum,
        Self::ExclusiveMinimum,
        Self::Maximum,
        Self::ExclusiveMaximum,
        Self::MinLength,
        Self::MaxLength,
        Self::MinItems,
        Self::MaxItems,
        Self::Required,
        Self::Additional,
        Self::PropertyName,
        Self::NoBranch,
        Self::ManyBranches,
        Self::UnknownKeyword,
        Self::BadKeyword,
        Self::BadReference,
        Self::ReferenceLoop,
        Self::Limit,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Enum => "enum",
            Self::Const => "const",
            Self::Minimum => "minimum",
            Self::ExclusiveMinimum => "exclusive_minimum",
            Self::Maximum => "maximum",
            Self::ExclusiveMaximum => "exclusive_maximum",
            Self::MinLength => "min_length",
            Self::MaxLength => "max_length",
            Self::MinItems => "min_items",
            Self::MaxItems => "max_items",
            Self::Required => "required",
            Self::Additional => "additional",
            Self::PropertyName => "property_name",
            Self::NoBranch => "no_branch",
            Self::ManyBranches => "many_branches",
            Self::UnknownKeyword => "unknown_keyword",
            Self::BadKeyword => "bad_keyword",
            Self::BadReference => "bad_reference",
            Self::ReferenceLoop => "reference_loop",
            Self::Limit => "limit",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for StrictSchemaIssueCode {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSchemaNormalizeInput {
    pub schemas: Vec<Value>,
}

impl Shape for StrictSchemaNormalizeInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let schemas = shape::required(object, "schemas", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        Ok(Self { schemas })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("schemas", Value::Array(self.schemas.clone()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSchemaNormalizeResult {
    pub schemas: Vec<Value>,
}

impl Shape for StrictSchemaNormalizeResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let schemas = shape::required(object, "schemas", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        Ok(Self { schemas })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("schemas", Value::Array(self.schemas.clone()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSchemaProblemsInput {
    pub schemas: Vec<Value>,
}

impl Shape for StrictSchemaProblemsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let schemas = shape::required(object, "schemas", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        Ok(Self { schemas })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("schemas", Value::Array(self.schemas.clone()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSchemaProblemsResult {
    pub problems: Vec<Vec<StrictSchemaIssue>>,
}

impl Shape for StrictSchemaProblemsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let problems = shape::required(object, "problems", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, StrictSchemaIssue::decode_at)
            })
        })?;
        Ok(Self { problems })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "problems",
            Value::Array(
                self.problems
                    .iter()
                    .map(|held| Value::Array(held.iter().map(|held| held.encode()).collect()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSchemaValidateInput {
    pub schema: Value,
    pub texts: Vec<String>,
}

impl Shape for StrictSchemaValidateInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let schema = shape::required(object, "schema", at, shape::any)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { schema, texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("schema", self.schema.clone());
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSchemaValidateResult {
    pub issues: Vec<Vec<StrictSchemaIssue>>,
}

impl Shape for StrictSchemaValidateResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let issues = shape::required(object, "issues", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, StrictSchemaIssue::decode_at)
            })
        })?;
        Ok(Self { issues })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "issues",
            Value::Array(
                self.issues
                    .iter()
                    .map(|held| Value::Array(held.iter().map(|held| held.encode()).collect()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSchemaValidateListInput {
    pub schema: Value,
    pub item: String,
    pub counts: Vec<u64>,
}

impl Shape for StrictSchemaValidateListInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let schema = shape::required(object, "schema", at, shape::any)?;
        let item = shape::required(object, "item", at, shape::text)?;
        let counts = shape::required(object, "counts", at, |value, at| {
            shape::list(value, at, shape::natural)
        })?;
        Ok(Self {
            schema,
            item,
            counts,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("schema", self.schema.clone());
        object.insert("item", Value::from(self.item.as_str()));
        object.insert(
            "counts",
            Value::Array(
                self.counts
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSchemaValidateListResult {
    pub issues: Vec<Vec<StrictSchemaIssue>>,
}

impl Shape for StrictSchemaValidateListResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let issues = shape::required(object, "issues", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, StrictSchemaIssue::decode_at)
            })
        })?;
        Ok(Self { issues })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "issues",
            Value::Array(
                self.issues
                    .iter()
                    .map(|held| Value::Array(held.iter().map(|held| held.encode()).collect()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionSource {
    pub lines: Option<Vec<String>>,
    pub base64: Option<String>,
    pub file: Option<String>,
}

impl Shape for SessionSource {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let lines = shape::optional(object, "lines", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let base64 = shape::optional(object, "base64", at, shape::text)?;
        let file = shape::optional(object, "file", at, shape::text)?;
        Ok(Self {
            lines,
            base64,
            file,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.lines {
            object.insert(
                "lines",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.base64 {
            object.insert("base64", Value::from(held.as_str()));
        }
        if let Some(held) = &self.file {
            object.insert("file", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BodyOf {
    Whole,
    AfterFirstLine,
}

impl BodyOf {
    pub const ALL: [Self; 2] = [Self::Whole, Self::AfterFirstLine];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Whole => "whole",
            Self::AfterFirstLine => "afterFirstLine",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for BodyOf {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutRehomedInput {
    pub rows: Vec<String>,
    pub session_id: String,
}

impl Shape for SessionCutRehomedInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let session_id = shape::required(object, "sessionId", at, shape::text)?;
        Ok(Self { rows, session_id })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("sessionId", Value::from(self.session_id.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutRehomedResult {
    pub rows: Vec<String>,
}

impl Shape for SessionCutRehomedResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutIndexInput {
    pub source: SessionSource,
}

impl Shape for SessionCutIndexInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let source = shape::required(object, "source", at, SessionSource::decode_at)?;
        Ok(Self { source })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("source", self.source.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutIndexResult {
    pub index: nyaaai_contract::shapes::session_cut::SessionIndex,
}

impl Shape for SessionCutIndexResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let index = shape::required(
            object,
            "index",
            at,
            nyaaai_contract::shapes::session_cut::SessionIndex::decode_at,
        )?;
        Ok(Self { index })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("index", self.index.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutPlanInput {
    pub source: SessionSource,
    pub requests: Vec<nyaaai_contract::shapes::session_cut::CutRequest>,
    pub cli_version: String,
}

impl Shape for SessionCutPlanInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let source = shape::required(object, "source", at, SessionSource::decode_at)?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::session_cut::CutRequest::decode_at,
            )
        })?;
        let cli_version = shape::required(object, "cliVersion", at, shape::text)?;
        Ok(Self {
            source,
            requests,
            cli_version,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("source", self.source.encode());
        object.insert(
            "requests",
            Value::Array(self.requests.iter().map(|held| held.encode()).collect()),
        );
        object.insert("cliVersion", Value::from(self.cli_version.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutPlanResult {
    pub plans: Vec<SessionCutPlanResultPlansItem>,
}

impl Shape for SessionCutPlanResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let plans = shape::required(object, "plans", at, |value, at| {
            shape::list(value, at, SessionCutPlanResultPlansItem::decode_at)
        })?;
        Ok(Self { plans })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "plans",
            Value::Array(self.plans.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionCutPlanResultPlansItem {
    CutPlan(nyaaai_contract::shapes::session_cut::CutPlan),
    CutRefused(nyaaai_contract::shapes::session_cut::CutRefused),
}

impl Shape for SessionCutPlanResultPlansItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = nyaaai_contract::shapes::session_cut::CutPlan::decode_at(value, at) {
            return Ok(Self::CutPlan(held));
        }
        if let Ok(held) = nyaaai_contract::shapes::session_cut::CutRefused::decode_at(value, at) {
            return Ok(Self::CutRefused(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::CutPlan(held) => held.encode(),
            Self::CutRefused(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutCutInput {
    pub source: SessionSource,
    pub request: nyaaai_contract::shapes::session_cut::CutRequest,
    pub cli_version: String,
    pub source_path: String,
    pub id_prefix: String,
    pub now_ms: i64,
    pub summary: Option<String>,
}

impl Shape for SessionCutCutInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let source = shape::required(object, "source", at, SessionSource::decode_at)?;
        let request = shape::required(
            object,
            "request",
            at,
            nyaaai_contract::shapes::session_cut::CutRequest::decode_at,
        )?;
        let cli_version = shape::required(object, "cliVersion", at, shape::text)?;
        let source_path = shape::required(object, "sourcePath", at, shape::text)?;
        let id_prefix = shape::required(object, "idPrefix", at, shape::text)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let summary = shape::optional(object, "summary", at, shape::text)?;
        Ok(Self {
            source,
            request,
            cli_version,
            source_path,
            id_prefix,
            now_ms,
            summary,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("source", self.source.encode());
        object.insert("request", self.request.encode());
        object.insert("cliVersion", Value::from(self.cli_version.as_str()));
        object.insert("sourcePath", Value::from(self.source_path.as_str()));
        object.insert("idPrefix", Value::from(self.id_prefix.as_str()));
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        if let Some(held) = &self.summary {
            object.insert("summary", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutCutResult {
    pub error: Option<String>,
    pub result: Option<SessionCutCutResultResult>,
    pub built: Option<Option<Vec<String>>>,
    pub summarized: Option<Option<SessionCutCutResultSummarized>>,
}

impl Shape for SessionCutCutResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let error = shape::optional(object, "error", at, shape::text)?;
        let result = shape::optional(object, "result", at, SessionCutCutResultResult::decode_at)?;
        let built = shape::optional(object, "built", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        let summarized = shape::optional(object, "summarized", at, |value, at| {
            shape::nullable(value, at, SessionCutCutResultSummarized::decode_at)
        })?;
        Ok(Self {
            error,
            result,
            built,
            summarized,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.error {
            object.insert("error", Value::from(held.as_str()));
        }
        if let Some(held) = &self.result {
            object.insert("result", held.encode());
        }
        if let Some(held) = &self.built {
            object.insert(
                "built",
                held.as_ref().map_or(Value::Null, |held| {
                    Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
                }),
            );
        }
        if let Some(held) = &self.summarized {
            object.insert(
                "summarized",
                held.as_ref().map_or(Value::Null, |held| held.encode()),
            );
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionCutCutResultResult {
    CutDone(nyaaai_contract::shapes::session_cut::CutDone),
    CutRefused(nyaaai_contract::shapes::session_cut::CutRefused),
}

impl Shape for SessionCutCutResultResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = nyaaai_contract::shapes::session_cut::CutDone::decode_at(value, at) {
            return Ok(Self::CutDone(held));
        }
        if let Ok(held) = nyaaai_contract::shapes::session_cut::CutRefused::decode_at(value, at) {
            return Ok(Self::CutRefused(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::CutDone(held) => held.encode(),
            Self::CutRefused(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutCutResultSummarized {
    pub standing: Option<String>,
    pub rows: Vec<String>,
}

impl Shape for SessionCutCutResultSummarized {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let standing = shape::required(object, "standing", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { standing, rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "standing",
            self.standing
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutVerifyInput {
    pub source: SessionSource,
    pub session_id: String,
    pub paired: bool,
}

impl Shape for SessionCutVerifyInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let source = shape::required(object, "source", at, SessionSource::decode_at)?;
        let session_id = shape::required(object, "sessionId", at, shape::text)?;
        let paired = shape::required(object, "paired", at, shape::flag)?;
        Ok(Self {
            source,
            session_id,
            paired,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("source", self.source.encode());
        object.insert("sessionId", Value::from(self.session_id.as_str()));
        object.insert("paired", Value::Bool(self.paired));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutVerifyResult {
    pub problems: Vec<String>,
}

impl Shape for SessionCutVerifyResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let problems = shape::required(object, "problems", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { problems })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "problems",
            Value::Array(
                self.problems
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutPickTurnInput {
    pub source: SessionSource,
    pub picks: Vec<SessionCutPickTurnInputPicksItem>,
}

impl Shape for SessionCutPickTurnInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let source = shape::required(object, "source", at, SessionSource::decode_at)?;
        let picks = shape::required(object, "picks", at, |value, at| {
            shape::list(value, at, SessionCutPickTurnInputPicksItem::decode_at)
        })?;
        Ok(Self { source, picks })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("source", self.source.encode());
        object.insert(
            "picks",
            Value::Array(self.picks.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutPickTurnInputPicksItem {
    pub at: i64,
    pub text: Option<String>,
    pub body_of: BodyOf,
}

impl Shape for SessionCutPickTurnInputPicksItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let at_field = shape::required(object, "at", at, shape::integer)?;
        let text = shape::optional(object, "text", at, shape::text)?;
        let body_of = shape::required(object, "bodyOf", at, BodyOf::decode_at)?;
        Ok(Self {
            at: at_field,
            text,
            body_of,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("at", Value::Number(self.at as f64));
        if let Some(held) = &self.text {
            object.insert("text", Value::from(held.as_str()));
        }
        object.insert("bodyOf", self.body_of.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionCutPickTurnResult {
    pub picked: Vec<Option<nyaaai_contract::shapes::session_cut::PickedTurn>>,
}

impl Shape for SessionCutPickTurnResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let picked = shape::required(object, "picked", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(
                    value,
                    at,
                    nyaaai_contract::shapes::session_cut::PickedTurn::decode_at,
                )
            })
        })?;
        Ok(Self { picked })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "picked",
            Value::Array(
                self.picked
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionFileLinesOfInput {
    pub base64: String,
}

impl Shape for SessionFileLinesOfInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let base64 = shape::required(object, "base64", at, shape::text)?;
        Ok(Self { base64 })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("base64", Value::from(self.base64.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionFileLinesOfResult {
    pub rows: Vec<String>,
}

impl Shape for SessionFileLinesOfResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathsIsInsideInput {
    pub tree: Vec<nyaaai_testkit::tree_shapes::TreeEntry>,
    pub checks: Vec<PathsIsInsideInputChecksItem>,
    pub relative: Option<()>,
}

impl Shape for PathsIsInsideInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tree = shape::required(object, "tree", at, |value, at| {
            shape::list(value, at, nyaaai_testkit::tree_shapes::TreeEntry::decode_at)
        })?;
        let checks = shape::required(object, "checks", at, |value, at| {
            shape::list(value, at, PathsIsInsideInputChecksItem::decode_at)
        })?;
        let relative = shape::optional(object, "relative", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        Ok(Self {
            tree,
            checks,
            relative,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "tree",
            Value::Array(self.tree.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "checks",
            Value::Array(self.checks.iter().map(|held| held.encode()).collect()),
        );
        if self.relative.is_some() {
            object.insert("relative", Value::Bool(true));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathsIsInsideInputChecksItem {
    pub child: String,
    pub parent: String,
}

impl Shape for PathsIsInsideInputChecksItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let child = shape::required(object, "child", at, shape::text)?;
        let parent = shape::required(object, "parent", at, shape::text)?;
        Ok(Self { child, parent })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("child", Value::from(self.child.as_str()));
        object.insert("parent", Value::from(self.parent.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PathsIsInsideResult {
    pub results: Vec<PathsIsInsideResultResultsItem>,
}

impl Shape for PathsIsInsideResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let results = shape::required(object, "results", at, |value, at| {
            shape::list(value, at, PathsIsInsideResultResultsItem::decode_at)
        })?;
        Ok(Self { results })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "results",
            Value::Array(self.results.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathsIsInsideResultResultsItem {
    Flag(bool),
    Error,
}

impl Shape for PathsIsInsideResultResultsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = shape::flag(value, at) {
            return Ok(Self::Flag(held));
        }
        if shape::is_text(value, at, "error").is_ok() {
            return Ok(Self::Error);
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Flag(held) => Value::Bool(*held),
            Self::Error => Value::from("error"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClaudeVersionInInput {
    pub texts: Vec<String>,
}

impl Shape for ClaudeVersionInInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClaudeVersionInResult {
    pub found: Vec<Option<String>>,
}

impl Shape for ClaudeVersionInResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let found = shape::required(object, "found", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { found })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "found",
            Value::Array(
                self.found
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitConverseInput {
    pub requests: u64,
    pub together: bool,
}

impl Shape for TestkitConverseInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let requests = shape::required(object, "requests", at, shape::natural)?;
        let together = shape::required(object, "together", at, shape::flag)?;
        Ok(Self { requests, together })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("requests", Value::Number(self.requests as f64));
        object.insert("together", Value::Bool(self.together));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitConverseResult {
    pub ids: Vec<String>,
    pub now_ms: i64,
    pub steers: Vec<String>,
}

impl Shape for TestkitConverseResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let ids = shape::required(object, "ids", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let steers = shape::required(object, "steers", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            ids,
            now_ms,
            steers,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "ids",
            Value::Array(
                self.ids
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert(
            "steers",
            Value::Array(
                self.steers
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitMisfitInput {
    pub fits: bool,
}

impl Shape for TestkitMisfitInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let fits = shape::required(object, "fits", at, shape::flag)?;
        Ok(Self { fits })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("fits", Value::Bool(self.fits));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitMisfitResult {
    pub text: String,
}

impl Shape for TestkitMisfitResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let text = shape::required(object, "text", at, |value, at| {
            shape::text_within(value, at, Some(1), None)
        })?;
        Ok(Self { text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("text", Value::from(self.text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitEntryKeysInput {
    pub steps: Vec<TestkitEntryKeysInputStepsItem>,
}

impl Shape for TestkitEntryKeysInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitEntryKeysInputStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitEntryKeysInputStepsItem {
    pub op: TestkitEntryKeysInputStepsItemOp,
    pub name: Option<String>,
    pub id: Option<String>,
    pub answer: Option<TestkitEntryKeysInputStepsItemAnswer>,
    pub key: Option<String>,
    pub unlistens: Option<String>,
}

impl Shape for TestkitEntryKeysInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let op = shape::required(
            object,
            "op",
            at,
            TestkitEntryKeysInputStepsItemOp::decode_at,
        )?;
        let name = shape::optional(object, "name", at, shape::text)?;
        let id = shape::optional(object, "id", at, shape::text)?;
        let answer = shape::optional(
            object,
            "answer",
            at,
            TestkitEntryKeysInputStepsItemAnswer::decode_at,
        )?;
        let key = shape::optional(object, "key", at, shape::text)?;
        let unlistens = shape::optional(object, "unlistens", at, shape::text)?;
        Ok(Self {
            op,
            name,
            id,
            answer,
            key,
            unlistens,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("op", self.op.encode());
        if let Some(held) = &self.name {
            object.insert("name", Value::from(held.as_str()));
        }
        if let Some(held) = &self.id {
            object.insert("id", Value::from(held.as_str()));
        }
        if let Some(held) = &self.answer {
            object.insert("answer", held.encode());
        }
        if let Some(held) = &self.key {
            object.insert("key", Value::from(held.as_str()));
        }
        if let Some(held) = &self.unlistens {
            object.insert("unlistens", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestkitEntryKeysInputStepsItemOp {
    Listen,
    Unlisten,
    Set,
    Get,
}

impl TestkitEntryKeysInputStepsItemOp {
    pub const ALL: [Self; 4] = [Self::Listen, Self::Unlisten, Self::Set, Self::Get];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Listen => "listen",
            Self::Unlisten => "unlisten",
            Self::Set => "set",
            Self::Get => "get",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestkitEntryKeysInputStepsItemOp {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestkitEntryKeysInputStepsItemAnswer {
    Key,
    None,
    Refused,
    Unready,
}

impl TestkitEntryKeysInputStepsItemAnswer {
    pub const ALL: [Self; 4] = [Self::Key, Self::None, Self::Refused, Self::Unready];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::None => "none",
            Self::Refused => "refused",
            Self::Unready => "unready",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestkitEntryKeysInputStepsItemAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitEntryKeysResult {
    pub steps: Vec<TestkitEntryKeysResultStepsItem>,
}

impl Shape for TestkitEntryKeysResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitEntryKeysResultStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitEntryKeysResultStepsItem {
    pub heard: Vec<TestkitEntryKeysResultStepsItemHeardItem>,
    pub got: Option<TestkitEntryKeysResultStepsItemGot>,
    pub reads: i64,
}

impl Shape for TestkitEntryKeysResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let heard = shape::required(object, "heard", at, |value, at| {
            shape::list(
                value,
                at,
                TestkitEntryKeysResultStepsItemHeardItem::decode_at,
            )
        })?;
        let got = shape::optional(
            object,
            "got",
            at,
            TestkitEntryKeysResultStepsItemGot::decode_at,
        )?;
        let reads = shape::required(object, "reads", at, shape::integer)?;
        Ok(Self { heard, got, reads })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "heard",
            Value::Array(self.heard.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.got {
            object.insert("got", held.encode());
        }
        object.insert("reads", Value::Number(self.reads as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitEntryKeysResultStepsItemHeardItem {
    pub name: String,
    pub id: String,
}

impl Shape for TestkitEntryKeysResultStepsItemHeardItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let name = shape::required(object, "name", at, shape::text)?;
        let id = shape::required(object, "id", at, shape::text)?;
        Ok(Self { name, id })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("name", Value::from(self.name.as_str()));
        object.insert("id", Value::from(self.id.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitEntryKeysResultStepsItemGot {
    pub kind: String,
    pub key: Option<String>,
}

impl Shape for TestkitEntryKeysResultStepsItemGot {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, shape::text)?;
        let key = shape::optional(object, "key", at, shape::text)?;
        Ok(Self { kind, key })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from(self.kind.as_str()));
        if let Some(held) = &self.key {
            object.insert("key", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitProviderListInput {
    pub steps: Vec<TestkitProviderListInputStepsItem>,
}

impl Shape for TestkitProviderListInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitProviderListInputStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitProviderListInputStepsItem {
    pub op: TestkitProviderListInputStepsItemOp,
    pub name: Option<String>,
    pub ids: Option<Vec<String>>,
    pub unlistens: Option<String>,
}

impl Shape for TestkitProviderListInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let op = shape::required(
            object,
            "op",
            at,
            TestkitProviderListInputStepsItemOp::decode_at,
        )?;
        let name = shape::optional(object, "name", at, shape::text)?;
        let ids = shape::optional(object, "ids", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let unlistens = shape::optional(object, "unlistens", at, shape::text)?;
        Ok(Self {
            op,
            name,
            ids,
            unlistens,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("op", self.op.encode());
        if let Some(held) = &self.name {
            object.insert("name", Value::from(held.as_str()));
        }
        if let Some(held) = &self.ids {
            object.insert(
                "ids",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.unlistens {
            object.insert("unlistens", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestkitProviderListInputStepsItemOp {
    Listen,
    Unlisten,
    Set,
    Read,
}

impl TestkitProviderListInputStepsItemOp {
    pub const ALL: [Self; 4] = [Self::Listen, Self::Unlisten, Self::Set, Self::Read];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Listen => "listen",
            Self::Unlisten => "unlisten",
            Self::Set => "set",
            Self::Read => "read",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestkitProviderListInputStepsItemOp {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitProviderListResult {
    pub steps: Vec<TestkitProviderListResultStepsItem>,
}

impl Shape for TestkitProviderListResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitProviderListResultStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitProviderListResultStepsItem {
    pub heard: Vec<String>,
    pub ids: Option<Vec<String>>,
    pub reads: i64,
}

impl Shape for TestkitProviderListResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let heard = shape::required(object, "heard", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let ids = shape::optional(object, "ids", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let reads = shape::required(object, "reads", at, shape::integer)?;
        Ok(Self { heard, ids, reads })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "heard",
            Value::Array(
                self.heard
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        if let Some(held) = &self.ids {
            object.insert(
                "ids",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        object.insert("reads", Value::Number(self.reads as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitModelRegistryInput {
    pub steps: Vec<TestkitModelRegistryInputStepsItem>,
}

impl Shape for TestkitModelRegistryInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitModelRegistryInputStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitModelRegistryInputStepsItem {
    pub op: TestkitModelRegistryInputStepsItemOp,
    pub name: Option<String>,
    pub keys: Option<Vec<String>>,
    pub entry: Option<String>,
    pub id: Option<String>,
    pub label: Option<String>,
    pub why: Option<String>,
    pub unlistens: Option<String>,
}

impl Shape for TestkitModelRegistryInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let op = shape::required(
            object,
            "op",
            at,
            TestkitModelRegistryInputStepsItemOp::decode_at,
        )?;
        let name = shape::optional(object, "name", at, shape::text)?;
        let keys = shape::optional(object, "keys", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let entry = shape::optional(object, "entry", at, shape::text)?;
        let id = shape::optional(object, "id", at, shape::text)?;
        let label = shape::optional(object, "label", at, shape::text)?;
        let why = shape::optional(object, "why", at, shape::text)?;
        let unlistens = shape::optional(object, "unlistens", at, shape::text)?;
        Ok(Self {
            op,
            name,
            keys,
            entry,
            id,
            label,
            why,
            unlistens,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("op", self.op.encode());
        if let Some(held) = &self.name {
            object.insert("name", Value::from(held.as_str()));
        }
        if let Some(held) = &self.keys {
            object.insert(
                "keys",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.entry {
            object.insert("entry", Value::from(held.as_str()));
        }
        if let Some(held) = &self.id {
            object.insert("id", Value::from(held.as_str()));
        }
        if let Some(held) = &self.label {
            object.insert("label", Value::from(held.as_str()));
        }
        if let Some(held) = &self.why {
            object.insert("why", Value::from(held.as_str()));
        }
        if let Some(held) = &self.unlistens {
            object.insert("unlistens", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestkitModelRegistryInputStepsItemOp {
    Listen,
    Unlisten,
    SetRows,
    SetHidden,
    Read,
    Record,
    Refuse,
    Allow,
}

impl TestkitModelRegistryInputStepsItemOp {
    pub const ALL: [Self; 8] = [
        Self::Listen,
        Self::Unlisten,
        Self::SetRows,
        Self::SetHidden,
        Self::Read,
        Self::Record,
        Self::Refuse,
        Self::Allow,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Listen => "listen",
            Self::Unlisten => "unlisten",
            Self::SetRows => "setRows",
            Self::SetHidden => "setHidden",
            Self::Read => "read",
            Self::Record => "record",
            Self::Refuse => "refuse",
            Self::Allow => "allow",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestkitModelRegistryInputStepsItemOp {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitModelRegistryResult {
    pub steps: Vec<TestkitModelRegistryResultStepsItem>,
}

impl Shape for TestkitModelRegistryResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitModelRegistryResultStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitModelRegistryResultStepsItem {
    pub heard: Vec<String>,
    pub rows: Option<Vec<String>>,
    pub hidden: Option<Vec<String>>,
    pub tested: Option<Vec<String>>,
    pub outcome: Option<Option<String>>,
    pub reads: i64,
    pub written: i64,
}

impl Shape for TestkitModelRegistryResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let heard = shape::required(object, "heard", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let rows = shape::optional(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let hidden = shape::optional(object, "hidden", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let tested = shape::optional(object, "tested", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let outcome = shape::optional(object, "outcome", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let reads = shape::required(object, "reads", at, shape::integer)?;
        let written = shape::required(object, "written", at, shape::integer)?;
        Ok(Self {
            heard,
            rows,
            hidden,
            tested,
            outcome,
            reads,
            written,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "heard",
            Value::Array(
                self.heard
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        if let Some(held) = &self.rows {
            object.insert(
                "rows",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.hidden {
            object.insert(
                "hidden",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.tested {
            object.insert(
                "tested",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.outcome {
            object.insert(
                "outcome",
                held.as_ref()
                    .map_or(Value::Null, |held| Value::from(held.as_str())),
            );
        }
        object.insert("reads", Value::Number(self.reads as f64));
        object.insert("written", Value::Number(self.written as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitDeadlineInput {
    pub start_ms: i64,
    pub steps: Vec<TestkitDeadlineInputStepsItem>,
}

impl Shape for TestkitDeadlineInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitDeadlineInputStepsItem::decode_at)
        })?;
        Ok(Self { start_ms, steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitDeadlineInputStepsItem {
    pub op: TestkitDeadlineInputStepsItemOp,
    pub id: Option<String>,
    pub ms: Option<i64>,
    pub caller: Option<String>,
}

impl Shape for TestkitDeadlineInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let op = shape::required(object, "op", at, TestkitDeadlineInputStepsItemOp::decode_at)?;
        let id = shape::optional(object, "id", at, shape::text)?;
        let ms = shape::optional(object, "ms", at, shape::integer)?;
        let caller = shape::optional(object, "caller", at, shape::text)?;
        Ok(Self { op, id, ms, caller })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("op", self.op.encode());
        if let Some(held) = &self.id {
            object.insert("id", Value::from(held.as_str()));
        }
        if let Some(held) = &self.ms {
            object.insert("ms", Value::Number(*held as f64));
        }
        if let Some(held) = &self.caller {
            object.insert("caller", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestkitDeadlineInputStepsItemOp {
    Caller,
    AbortCaller,
    AbortAt,
    Deadline,
    Clear,
    Advance,
}

impl TestkitDeadlineInputStepsItemOp {
    pub const ALL: [Self; 6] = [
        Self::Caller,
        Self::AbortCaller,
        Self::AbortAt,
        Self::Deadline,
        Self::Clear,
        Self::Advance,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Caller => "caller",
            Self::AbortCaller => "abortCaller",
            Self::AbortAt => "abortAt",
            Self::Deadline => "deadline",
            Self::Clear => "clear",
            Self::Advance => "advance",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestkitDeadlineInputStepsItemOp {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitDeadlineResult {
    pub steps: Vec<TestkitDeadlineResultStepsItem>,
}

impl Shape for TestkitDeadlineResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitDeadlineResultStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitDeadlineResultStepsItem {
    pub now_ms: i64,
    pub waiting: Vec<String>,
    pub deadlines: Vec<TestkitDeadlineResultStepsItemDeadlinesItem>,
}

impl Shape for TestkitDeadlineResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let deadlines = shape::required(object, "deadlines", at, |value, at| {
            shape::list(
                value,
                at,
                TestkitDeadlineResultStepsItemDeadlinesItem::decode_at,
            )
        })?;
        Ok(Self {
            now_ms,
            waiting,
            deadlines,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "deadlines",
            Value::Array(self.deadlines.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitDeadlineResultStepsItemDeadlinesItem {
    pub id: String,
    pub aborted: bool,
    pub ended: Option<String>,
}

impl Shape for TestkitDeadlineResultStepsItemDeadlinesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let aborted = shape::required(object, "aborted", at, shape::flag)?;
        let ended = shape::required(object, "ended", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self { id, aborted, ended })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("aborted", Value::Bool(self.aborted));
        object.insert(
            "ended",
            self.ended
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitUsageSinkInput {
    pub steps: Vec<TestkitUsageSinkInputStepsItem>,
}

impl Shape for TestkitUsageSinkInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitUsageSinkInputStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitUsageSinkInputStepsItem {
    pub op: TestkitUsageSinkInputStepsItemOp,
    pub ids: Option<Vec<String>>,
    pub on: Option<bool>,
    pub model: Option<String>,
    pub generation_id: Option<String>,
    pub id: Option<String>,
    pub at: Option<i64>,
    pub provider: Option<String>,
    pub rows: Option<Vec<nyaaai_contract::shapes::usage::UnfilledRecord>>,
}

impl Shape for TestkitUsageSinkInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let op = shape::required(
            object,
            "op",
            at,
            TestkitUsageSinkInputStepsItemOp::decode_at,
        )?;
        let ids = shape::optional(object, "ids", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let on = shape::optional(object, "on", at, shape::flag)?;
        let model = shape::optional(object, "model", at, shape::text)?;
        let generation_id = shape::optional(object, "generationId", at, shape::text)?;
        let id = shape::optional(object, "id", at, shape::text)?;
        let at_field = shape::optional(object, "at", at, shape::integer)?;
        let provider = shape::optional(object, "provider", at, shape::text)?;
        let rows = shape::optional(object, "rows", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::usage::UnfilledRecord::decode_at,
            )
        })?;
        Ok(Self {
            op,
            ids,
            on,
            model,
            generation_id,
            id,
            at: at_field,
            provider,
            rows,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("op", self.op.encode());
        if let Some(held) = &self.ids {
            object.insert(
                "ids",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.on {
            object.insert("on", Value::Bool(*held));
        }
        if let Some(held) = &self.model {
            object.insert("model", Value::from(held.as_str()));
        }
        if let Some(held) = &self.generation_id {
            object.insert("generationId", Value::from(held.as_str()));
        }
        if let Some(held) = &self.id {
            object.insert("id", Value::from(held.as_str()));
        }
        if let Some(held) = &self.at {
            object.insert("at", Value::Number(*held as f64));
        }
        if let Some(held) = &self.provider {
            object.insert("provider", Value::from(held.as_str()));
        }
        if let Some(held) = &self.rows {
            object.insert(
                "rows",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestkitUsageSinkInputStepsItemOp {
    Ids,
    None,
    Record,
    Edit,
    Fill,
    MarkFilled,
    Rows,
    Filled,
    Unfilled,
}

impl TestkitUsageSinkInputStepsItemOp {
    pub const ALL: [Self; 9] = [
        Self::Ids,
        Self::None,
        Self::Record,
        Self::Edit,
        Self::Fill,
        Self::MarkFilled,
        Self::Rows,
        Self::Filled,
        Self::Unfilled,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ids => "ids",
            Self::None => "none",
            Self::Record => "record",
            Self::Edit => "edit",
            Self::Fill => "fill",
            Self::MarkFilled => "markFilled",
            Self::Rows => "rows",
            Self::Filled => "filled",
            Self::Unfilled => "unfilled",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestkitUsageSinkInputStepsItemOp {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitUsageSinkResult {
    pub steps: Vec<TestkitUsageSinkResultStepsItem>,
}

impl Shape for TestkitUsageSinkResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitUsageSinkResultStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitUsageSinkResultStepsItem {
    pub answer: Option<Option<String>>,
    pub filled: Option<bool>,
    pub unfilled: Option<Vec<nyaaai_contract::shapes::usage::UnfilledRecord>>,
    pub calls: TestkitUsageSinkResultStepsItemCalls,
    pub recorded: Vec<String>,
    pub fills: Vec<String>,
    pub filled_asks: Vec<String>,
    pub unfilled_asks: Vec<i64>,
}

impl Shape for TestkitUsageSinkResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let answer = shape::optional(object, "answer", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let filled = shape::optional(object, "filled", at, shape::flag)?;
        let unfilled = shape::optional(object, "unfilled", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::usage::UnfilledRecord::decode_at,
            )
        })?;
        let calls = shape::required(
            object,
            "calls",
            at,
            TestkitUsageSinkResultStepsItemCalls::decode_at,
        )?;
        let recorded = shape::required(object, "recorded", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let fills = shape::required(object, "fills", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let filled_asks = shape::required(object, "filledAsks", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let unfilled_asks = shape::required(object, "unfilledAsks", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        Ok(Self {
            answer,
            filled,
            unfilled,
            calls,
            recorded,
            fills,
            filled_asks,
            unfilled_asks,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.answer {
            object.insert(
                "answer",
                held.as_ref()
                    .map_or(Value::Null, |held| Value::from(held.as_str())),
            );
        }
        if let Some(held) = &self.filled {
            object.insert("filled", Value::Bool(*held));
        }
        if let Some(held) = &self.unfilled {
            object.insert(
                "unfilled",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        object.insert("calls", self.calls.encode());
        object.insert(
            "recorded",
            Value::Array(
                self.recorded
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "fills",
            Value::Array(
                self.fills
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "filledAsks",
            Value::Array(
                self.filled_asks
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "unfilledAsks",
            Value::Array(
                self.unfilled_asks
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitUsageSinkResultStepsItemCalls {
    pub record: i64,
    pub fill: i64,
    pub filled: i64,
    pub unfilled: i64,
}

impl Shape for TestkitUsageSinkResultStepsItemCalls {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let record = shape::required(object, "record", at, shape::integer)?;
        let fill = shape::required(object, "fill", at, shape::integer)?;
        let filled = shape::required(object, "filled", at, shape::integer)?;
        let unfilled = shape::required(object, "unfilled", at, shape::integer)?;
        Ok(Self {
            record,
            fill,
            filled,
            unfilled,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("record", Value::Number(self.record as f64));
        object.insert("fill", Value::Number(self.fill as f64));
        object.insert("filled", Value::Number(self.filled as f64));
        object.insert("unfilled", Value::Number(self.unfilled as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitTimeOffsetInput {
    pub clock: TestkitTimeOffsetInputClock,
    pub start_ms: i64,
    pub offsets: Vec<TestkitTimeOffsetInputOffsetsItem>,
    pub asks: Vec<i64>,
}

impl Shape for TestkitTimeOffsetInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let clock = shape::required(object, "clock", at, TestkitTimeOffsetInputClock::decode_at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let offsets = shape::required(object, "offsets", at, |value, at| {
            shape::list(value, at, TestkitTimeOffsetInputOffsetsItem::decode_at)
        })?;
        let asks = shape::required(object, "asks", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        Ok(Self {
            clock,
            start_ms,
            offsets,
            asks,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("clock", self.clock.encode());
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "offsets",
            Value::Array(self.offsets.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "asks",
            Value::Array(
                self.asks
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestkitTimeOffsetInputClock {
    Stepped,
    Wheel,
    Scheduler,
}

impl TestkitTimeOffsetInputClock {
    pub const ALL: [Self; 3] = [Self::Stepped, Self::Wheel, Self::Scheduler];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stepped => "stepped",
            Self::Wheel => "wheel",
            Self::Scheduler => "scheduler",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestkitTimeOffsetInputClock {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitTimeOffsetInputOffsetsItem {
    pub from_ms: i64,
    pub minutes: i64,
}

impl Shape for TestkitTimeOffsetInputOffsetsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let from_ms = shape::required(object, "fromMs", at, shape::integer)?;
        let minutes = shape::required(object, "minutes", at, shape::integer)?;
        Ok(Self { from_ms, minutes })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("fromMs", Value::Number(self.from_ms as f64));
        object.insert("minutes", Value::Number(self.minutes as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitTimeOffsetResult {
    pub rows: Vec<TestkitTimeOffsetResultRowsItem>,
}

impl Shape for TestkitTimeOffsetResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, TestkitTimeOffsetResultRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitTimeOffsetResultRowsItem {
    pub at_ms: i64,
    pub minutes: i64,
    pub day: Option<String>,
}

impl Shape for TestkitTimeOffsetResultRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let at_ms = shape::required(object, "atMs", at, shape::integer)?;
        let minutes = shape::required(object, "minutes", at, shape::integer)?;
        let day = shape::required(object, "day", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            at_ms,
            minutes,
            day,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("atMs", Value::Number(self.at_ms as f64));
        object.insert("minutes", Value::Number(self.minutes as f64));
        object.insert(
            "day",
            self.day
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitTimeOffsetRunInput {
    pub start_ms: i64,
    pub offsets: Vec<TestkitTimeOffsetRunInputOffsetsItem>,
    pub steps: Vec<TestkitTimeOffsetRunInputStepsItem>,
}

impl Shape for TestkitTimeOffsetRunInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let offsets = shape::required(object, "offsets", at, |value, at| {
            shape::list(value, at, TestkitTimeOffsetRunInputOffsetsItem::decode_at)
        })?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitTimeOffsetRunInputStepsItem::decode_at)
        })?;
        Ok(Self {
            start_ms,
            offsets,
            steps,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "offsets",
            Value::Array(self.offsets.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitTimeOffsetRunInputOffsetsItem {
    pub from_ms: i64,
    pub minutes: i64,
}

impl Shape for TestkitTimeOffsetRunInputOffsetsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let from_ms = shape::required(object, "fromMs", at, shape::integer)?;
        let minutes = shape::required(object, "minutes", at, shape::integer)?;
        Ok(Self { from_ms, minutes })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("fromMs", Value::Number(self.from_ms as f64));
        object.insert("minutes", Value::Number(self.minutes as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitTimeOffsetRunInputStepsItem {
    pub op: TestkitTimeOffsetRunInputStepsItemOp,
    pub offsets: Option<Vec<TestkitTimeOffsetRunInputStepsItemOffsetsItem>>,
    pub ms: Option<i64>,
    pub at_ms: Option<i64>,
}

impl Shape for TestkitTimeOffsetRunInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let op = shape::required(
            object,
            "op",
            at,
            TestkitTimeOffsetRunInputStepsItemOp::decode_at,
        )?;
        let offsets = shape::optional(object, "offsets", at, |value, at| {
            shape::list(
                value,
                at,
                TestkitTimeOffsetRunInputStepsItemOffsetsItem::decode_at,
            )
        })?;
        let ms = shape::optional(object, "ms", at, shape::integer)?;
        let at_ms = shape::optional(object, "atMs", at, shape::integer)?;
        Ok(Self {
            op,
            offsets,
            ms,
            at_ms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("op", self.op.encode());
        if let Some(held) = &self.offsets {
            object.insert(
                "offsets",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        if let Some(held) = &self.ms {
            object.insert("ms", Value::Number(*held as f64));
        }
        if let Some(held) = &self.at_ms {
            object.insert("atMs", Value::Number(*held as f64));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestkitTimeOffsetRunInputStepsItemOp {
    Set,
    Advance,
    Ask,
}

impl TestkitTimeOffsetRunInputStepsItemOp {
    pub const ALL: [Self; 3] = [Self::Set, Self::Advance, Self::Ask];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Set => "set",
            Self::Advance => "advance",
            Self::Ask => "ask",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestkitTimeOffsetRunInputStepsItemOp {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitTimeOffsetRunInputStepsItemOffsetsItem {
    pub from_ms: i64,
    pub minutes: i64,
}

impl Shape for TestkitTimeOffsetRunInputStepsItemOffsetsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let from_ms = shape::required(object, "fromMs", at, shape::integer)?;
        let minutes = shape::required(object, "minutes", at, shape::integer)?;
        Ok(Self { from_ms, minutes })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("fromMs", Value::Number(self.from_ms as f64));
        object.insert("minutes", Value::Number(self.minutes as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitTimeOffsetRunResult {
    pub steps: Vec<TestkitTimeOffsetRunResultStepsItem>,
}

impl Shape for TestkitTimeOffsetRunResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitTimeOffsetRunResultStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitTimeOffsetRunResultStepsItem {
    pub now_ms: i64,
    pub minutes: Option<i64>,
}

impl Shape for TestkitTimeOffsetRunResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let minutes = shape::optional(object, "minutes", at, shape::integer)?;
        Ok(Self { now_ms, minutes })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        if let Some(held) = &self.minutes {
            object.insert("minutes", Value::Number(*held as f64));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitCompactedSessionInput {}

impl Shape for TestkitCompactedSessionInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        shape::object(value, at)?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        Value::Object(Object::new())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitCompactedSessionResult {
    pub lines: Vec<String>,
}

impl Shape for TestkitCompactedSessionResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let lines = shape::required(object, "lines", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { lines })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "lines",
            Value::Array(
                self.lines
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitWheelInput {
    pub start_ms: i64,
    pub steps: Vec<TestkitWheelInputStepsItem>,
}

impl Shape for TestkitWheelInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitWheelInputStepsItem::decode_at)
        })?;
        Ok(Self { start_ms, steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitWheelInputStepsItem {
    pub op: TestkitWheelInputStepsItemOp,
    pub ms: Option<i64>,
    pub tag: Option<String>,
}

impl Shape for TestkitWheelInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let op = shape::required(object, "op", at, TestkitWheelInputStepsItemOp::decode_at)?;
        let ms = shape::optional(object, "ms", at, shape::integer)?;
        let tag = shape::optional(object, "tag", at, shape::text)?;
        Ok(Self { op, ms, tag })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("op", self.op.encode());
        if let Some(held) = &self.ms {
            object.insert("ms", Value::Number(*held as f64));
        }
        if let Some(held) = &self.tag {
            object.insert("tag", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestkitWheelInputStepsItemOp {
    After,
    Cancel,
    Advance,
}

impl TestkitWheelInputStepsItemOp {
    pub const ALL: [Self; 3] = [Self::After, Self::Cancel, Self::Advance];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::After => "after",
            Self::Cancel => "cancel",
            Self::Advance => "advance",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestkitWheelInputStepsItemOp {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitWheelResult {
    pub steps: Vec<TestkitWheelResultStepsItem>,
}

impl Shape for TestkitWheelResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitWheelResultStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitWheelResultStepsItem {
    pub now_ms: i64,
    pub waiting: Vec<String>,
    pub fired: Vec<TestkitWheelResultStepsItemFiredItem>,
    pub cancelled: bool,
}

impl Shape for TestkitWheelResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let fired = shape::required(object, "fired", at, |value, at| {
            shape::list(value, at, TestkitWheelResultStepsItemFiredItem::decode_at)
        })?;
        let cancelled = shape::required(object, "cancelled", at, shape::flag)?;
        Ok(Self {
            now_ms,
            waiting,
            fired,
            cancelled,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "fired",
            Value::Array(self.fired.iter().map(|held| held.encode()).collect()),
        );
        object.insert("cancelled", Value::Bool(self.cancelled));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitWheelResultStepsItemFiredItem {
    pub tag: String,
    pub at_ms: i64,
}

impl Shape for TestkitWheelResultStepsItemFiredItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let at_ms = shape::required(object, "atMs", at, shape::integer)?;
        Ok(Self { tag, at_ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("tag", Value::from(self.tag.as_str()));
        object.insert("atMs", Value::Number(self.at_ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitSchedulerInput {
    pub start_ms: i64,
    pub steps: Vec<TestkitSchedulerInputStepsItem>,
}

impl Shape for TestkitSchedulerInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitSchedulerInputStepsItem::decode_at)
        })?;
        Ok(Self { start_ms, steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitSchedulerInputStepsItem {
    pub op: TestkitSchedulerInputStepsItemOp,
    pub id: Option<String>,
    pub ms: Option<i64>,
    pub tag: Option<String>,
}

impl Shape for TestkitSchedulerInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let op = shape::required(
            object,
            "op",
            at,
            TestkitSchedulerInputStepsItemOp::decode_at,
        )?;
        let id = shape::optional(object, "id", at, shape::text)?;
        let ms = shape::optional(object, "ms", at, shape::integer)?;
        let tag = shape::optional(object, "tag", at, shape::text)?;
        Ok(Self { op, id, ms, tag })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("op", self.op.encode());
        if let Some(held) = &self.id {
            object.insert("id", Value::from(held.as_str()));
        }
        if let Some(held) = &self.ms {
            object.insert("ms", Value::Number(*held as f64));
        }
        if let Some(held) = &self.tag {
            object.insert("tag", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestkitSchedulerInputStepsItemOp {
    Sleep,
    Schedule,
    Cancel,
    Advance,
    Settle,
    WheelCancel,
    WheelAfter,
}

impl TestkitSchedulerInputStepsItemOp {
    pub const ALL: [Self; 7] = [
        Self::Sleep,
        Self::Schedule,
        Self::Cancel,
        Self::Advance,
        Self::Settle,
        Self::WheelCancel,
        Self::WheelAfter,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sleep => "sleep",
            Self::Schedule => "schedule",
            Self::Cancel => "cancel",
            Self::Advance => "advance",
            Self::Settle => "settle",
            Self::WheelCancel => "wheelCancel",
            Self::WheelAfter => "wheelAfter",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TestkitSchedulerInputStepsItemOp {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitSchedulerResult {
    pub steps: Vec<TestkitSchedulerResultStepsItem>,
}

impl Shape for TestkitSchedulerResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, TestkitSchedulerResultStepsItem::decode_at)
        })?;
        Ok(Self { steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitSchedulerResultStepsItem {
    pub now_ms: i64,
    pub waiting: Vec<String>,
    pub fired: Vec<TestkitSchedulerResultStepsItemFiredItem>,
    pub ran: Vec<String>,
    pub ended: Vec<TestkitSchedulerResultStepsItemEndedItem>,
}

impl Shape for TestkitSchedulerResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let fired = shape::required(object, "fired", at, |value, at| {
            shape::list(
                value,
                at,
                TestkitSchedulerResultStepsItemFiredItem::decode_at,
            )
        })?;
        let ran = shape::required(object, "ran", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let ended = shape::required(object, "ended", at, |value, at| {
            shape::list(
                value,
                at,
                TestkitSchedulerResultStepsItemEndedItem::decode_at,
            )
        })?;
        Ok(Self {
            now_ms,
            waiting,
            fired,
            ran,
            ended,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "fired",
            Value::Array(self.fired.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "ran",
            Value::Array(
                self.ran
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "ended",
            Value::Array(self.ended.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitSchedulerResultStepsItemFiredItem {
    pub tag: String,
    pub at_ms: i64,
}

impl Shape for TestkitSchedulerResultStepsItemFiredItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let at_ms = shape::required(object, "atMs", at, shape::integer)?;
        Ok(Self { tag, at_ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("tag", Value::from(self.tag.as_str()));
        object.insert("atMs", Value::Number(self.at_ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitSchedulerResultStepsItemEndedItem {
    pub id: String,
    pub outcome: String,
}

impl Shape for TestkitSchedulerResultStepsItemEndedItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let outcome = shape::required(object, "outcome", at, shape::text)?;
        Ok(Self { id, outcome })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("outcome", Value::from(self.outcome.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitRandomInput {
    pub values: Vec<f64>,
    pub draws: u64,
}

impl Shape for TestkitRandomInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::number)
        })?;
        let draws = shape::required(object, "draws", at, shape::natural)?;
        Ok(Self { values, draws })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(
                self.values
                    .iter()
                    .map(|held| Value::Number(*held))
                    .collect(),
            ),
        );
        object.insert("draws", Value::Number(self.draws as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitRandomResult {
    pub drawn: Vec<f64>,
}

impl Shape for TestkitRandomResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let drawn = shape::required(object, "drawn", at, |value, at| {
            shape::list(value, at, shape::number)
        })?;
        Ok(Self { drawn })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "drawn",
            Value::Array(self.drawn.iter().map(|held| Value::Number(*held)).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitReplayInput {
    pub file: String,
}

impl Shape for TestkitReplayInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let file = shape::required(object, "file", at, shape::text)?;
        Ok(Self { file })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("file", Value::from(self.file.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitReplayResult {
    pub requests: Vec<Value>,
    pub exchanges: Vec<TestkitReplayResultExchangesItem>,
}

impl Shape for TestkitReplayResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, TestkitReplayResultExchangesItem::decode_at)
        })?;
        Ok(Self {
            requests,
            exchanges,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("requests", Value::Array(self.requests.clone()));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitReplayResultExchangesItem {
    pub status: i64,
    pub headers: Record<String>,
    pub chunks: Vec<i64>,
    pub body: String,
}

impl Shape for TestkitReplayResultExchangesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let status = shape::required(object, "status", at, shape::integer)?;
        let headers = shape::required(object, "headers", at, |value, at| {
            shape::record(value, at, shape::text)
        })?;
        let chunks = shape::required(object, "chunks", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let body = shape::required(object, "body", at, shape::text)?;
        Ok(Self {
            status,
            headers,
            chunks,
            body,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("status", Value::Number(self.status as f64));
        object.insert(
            "headers",
            shape::record_value(&self.headers, |held| Value::from(held.as_str())),
        );
        object.insert(
            "chunks",
            Value::Array(
                self.chunks
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert("body", Value::from(self.body.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitContextInput {
    pub folder: String,
}

impl Shape for TestkitContextInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let folder = shape::required(object, "folder", at, shape::text)?;
        Ok(Self { folder })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("folder", Value::from(self.folder.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestkitContextResult {
    pub ids: Vec<String>,
    pub times: Vec<i64>,
    pub tmp: String,
    pub root: String,
    pub planted: bool,
}

impl Shape for TestkitContextResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let ids = shape::required(object, "ids", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let times = shape::required(object, "times", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let tmp = shape::required(object, "tmp", at, shape::text)?;
        let root = shape::required(object, "root", at, shape::text)?;
        let planted = shape::required(object, "planted", at, shape::flag)?;
        Ok(Self {
            ids,
            times,
            tmp,
            root,
            planted,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "ids",
            Value::Array(
                self.ids
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "times",
            Value::Array(
                self.times
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert("tmp", Value::from(self.tmp.as_str()));
        object.insert("root", Value::from(self.root.as_str()));
        object.insert("planted", Value::Bool(self.planted));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoticeTextTextsInput {
    pub rows: Vec<NoticeTextTextsInputRowsItem>,
}

impl Shape for NoticeTextTextsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, NoticeTextTextsInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoticeTextTextsInputRowsItem {
    pub model: Option<String>,
    pub entry: Option<String>,
}

impl Shape for NoticeTextTextsInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let model = shape::required(object, "model", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let entry = shape::required(object, "entry", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self { model, entry })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "model",
            self.model
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "entry",
            self.entry
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoticeTextTextsResult {
    pub overloaded: Vec<String>,
    pub unavailable: Vec<String>,
}

impl Shape for NoticeTextTextsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let overloaded = shape::required(object, "overloaded", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let unavailable = shape::required(object, "unavailable", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            overloaded,
            unavailable,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "overloaded",
            Value::Array(
                self.overloaded
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "unavailable",
            Value::Array(
                self.unavailable
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffortLadderInput {}

impl Shape for EffortLadderInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        shape::object(value, at)?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        Value::Object(Object::new())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffortLadderResult {
    pub efforts: Vec<nyaaai_contract::shapes::step::Effort>,
}

impl Shape for EffortLadderResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
        })?;
        Ok(Self { efforts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffortResolveInput {
    pub offered: Vec<String>,
    pub wanted: Vec<String>,
}

impl Shape for EffortResolveInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let offered = shape::required(object, "offered", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let wanted = shape::required(object, "wanted", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { offered, wanted })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "offered",
            Value::Array(
                self.offered
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "wanted",
            Value::Array(
                self.wanted
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffortResolveResult {
    pub values: Vec<Option<String>>,
}

impl Shape for EffortResolveResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(
                self.values
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SseItemsInput {
    pub chunks: Vec<String>,
    pub again: Option<Vec<String>>,
}

impl Shape for SseItemsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let chunks = shape::required(object, "chunks", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let again = shape::optional(object, "again", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { chunks, again })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "chunks",
            Value::Array(
                self.chunks
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        if let Some(held) = &self.again {
            object.insert(
                "again",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SseItemsResult {
    pub pushed: Vec<Vec<nyaaai_contract::shapes::step::SseItem>>,
    pub ended: Vec<nyaaai_contract::shapes::step::SseItem>,
    pub pushed_again: Option<Vec<Vec<nyaaai_contract::shapes::step::SseItem>>>,
    pub ended_again: Option<Vec<nyaaai_contract::shapes::step::SseItem>>,
}

impl Shape for SseItemsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let pushed = shape::required(object, "pushed", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, nyaaai_contract::shapes::step::SseItem::decode_at)
            })
        })?;
        let ended = shape::required(object, "ended", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::SseItem::decode_at)
        })?;
        let pushed_again = shape::optional(object, "pushedAgain", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, nyaaai_contract::shapes::step::SseItem::decode_at)
            })
        })?;
        let ended_again = shape::optional(object, "endedAgain", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::SseItem::decode_at)
        })?;
        Ok(Self {
            pushed,
            ended,
            pushed_again,
            ended_again,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "pushed",
            Value::Array(
                self.pushed
                    .iter()
                    .map(|held| Value::Array(held.iter().map(|held| held.encode()).collect()))
                    .collect(),
            ),
        );
        object.insert(
            "ended",
            Value::Array(self.ended.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.pushed_again {
            object.insert(
                "pushedAgain",
                Value::Array(
                    held.iter()
                        .map(|held| Value::Array(held.iter().map(|held| held.encode()).collect()))
                        .collect(),
                ),
            );
        }
        if let Some(held) = &self.ended_again {
            object.insert(
                "endedAgain",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SseSplitsInput {
    pub bytes: String,
}

impl Shape for SseSplitsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let bytes = shape::required(object, "bytes", at, shape::text)?;
        Ok(Self { bytes })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("bytes", Value::from(self.bytes.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SseSplitsResult {
    pub items: Vec<nyaaai_contract::shapes::step::SseItem>,
    pub agree: bool,
    pub first_different: Option<i64>,
    pub bytewise: bool,
}

impl Shape for SseSplitsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let items = shape::required(object, "items", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::SseItem::decode_at)
        })?;
        let agree = shape::required(object, "agree", at, shape::flag)?;
        let first_different = shape::required(object, "firstDifferent", at, |value, at| {
            shape::nullable(value, at, shape::integer)
        })?;
        let bytewise = shape::required(object, "bytewise", at, shape::flag)?;
        Ok(Self {
            items,
            agree,
            first_different,
            bytewise,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "items",
            Value::Array(self.items.iter().map(|held| held.encode()).collect()),
        );
        object.insert("agree", Value::Bool(self.agree));
        object.insert(
            "firstDifferent",
            self.first_different
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held as f64)),
        );
        object.insert("bytewise", Value::Bool(self.bytewise));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepBilled {
    pub usage: Option<nyaaai_contract::shapes::step::NormalizedUsage>,
    pub host: Option<String>,
    pub generation_id: Option<String>,
}

impl Shape for StepBilled {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let usage = shape::optional(
            object,
            "usage",
            at,
            nyaaai_contract::shapes::step::NormalizedUsage::decode_at,
        )?;
        let host = shape::optional(object, "host", at, shape::text)?;
        let generation_id = shape::optional(object, "generationId", at, shape::text)?;
        Ok(Self {
            usage,
            host,
            generation_id,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.usage {
            object.insert("usage", held.encode());
        }
        if let Some(held) = &self.host {
            object.insert("host", Value::from(held.as_str()));
        }
        if let Some(held) = &self.generation_id {
            object.insert("generationId", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepAssembleInput {
    pub events: Vec<nyaaai_contract::shapes::step::StepEvent>,
    pub more: Option<Vec<String>>,
}

impl Shape for StepAssembleInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let events = shape::required(object, "events", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::step::StepEvent::decode_at,
            )
        })?;
        let more = shape::optional(object, "more", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { events, more })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "events",
            Value::Array(self.events.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.more {
            object.insert(
                "more",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepAssembleResult {
    pub added: Vec<bool>,
    pub failed_at: Option<i64>,
    pub failure: Option<nyaaai_contract::shapes::step::Failure>,
    pub output_seen: bool,
    pub billed: Option<StepBilled>,
    pub snapshot: Vec<nyaaai_contract::shapes::step::ReplyPart>,
    pub result: Option<nyaaai_contract::shapes::step::StepResult>,
}

impl Shape for StepAssembleResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let added = shape::required(object, "added", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        let failed_at = shape::required(object, "failedAt", at, |value, at| {
            shape::nullable(value, at, shape::integer)
        })?;
        let failure = shape::required(object, "failure", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::step::Failure::decode_at)
        })?;
        let output_seen = shape::required(object, "outputSeen", at, shape::flag)?;
        let billed = shape::required(object, "billed", at, |value, at| {
            shape::nullable(value, at, StepBilled::decode_at)
        })?;
        let snapshot = shape::required(object, "snapshot", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::step::ReplyPart::decode_at,
            )
        })?;
        let result = shape::required(object, "result", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::step::StepResult::decode_at,
            )
        })?;
        Ok(Self {
            added,
            failed_at,
            failure,
            output_seen,
            billed,
            snapshot,
            result,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "added",
            Value::Array(self.added.iter().map(|held| Value::Bool(*held)).collect()),
        );
        object.insert(
            "failedAt",
            self.failed_at
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held as f64)),
        );
        object.insert(
            "failure",
            self.failure
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert("outputSeen", Value::Bool(self.output_seen));
        object.insert(
            "billed",
            self.billed
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "snapshot",
            Value::Array(self.snapshot.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "result",
            self.result
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepRepliesInput {
    pub result: nyaaai_contract::shapes::step::StepResult,
}

impl Shape for StepRepliesInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let result = shape::required(
            object,
            "result",
            at,
            nyaaai_contract::shapes::step::StepResult::decode_at,
        )?;
        Ok(Self { result })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("result", self.result.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepRepliesResult {
    pub text: String,
    pub calls: Vec<nyaaai_contract::shapes::step::ReplyPart>,
    pub thoughts: Vec<nyaaai_contract::shapes::step::ThoughtPart>,
}

impl Shape for StepRepliesResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let text = shape::required(object, "text", at, shape::text)?;
        let calls = shape::required(object, "calls", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::step::ReplyPart::decode_at,
            )
        })?;
        let thoughts = shape::required(object, "thoughts", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::step::ThoughtPart::decode_at,
            )
        })?;
        Ok(Self {
            text,
            calls,
            thoughts,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("text", Value::from(self.text.as_str()));
        object.insert(
            "calls",
            Value::Array(self.calls.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "thoughts",
            Value::Array(self.thoughts.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WireError {
    pub code: Option<WireErrorCode>,
    pub type_: Option<String>,
    pub message: Option<String>,
    pub metadata: Option<Record<Value>>,
    pub resets_at: Option<WireErrorResetsAt>,
}

impl Shape for WireError {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let code = shape::optional(object, "code", at, WireErrorCode::decode_at)?;
        let type_ = shape::optional(object, "type", at, shape::text)?;
        let message = shape::optional(object, "message", at, shape::text)?;
        let metadata = shape::optional(object, "metadata", at, |value, at| {
            shape::record(value, at, shape::any)
        })?;
        let resets_at = shape::optional(object, "resetsAt", at, WireErrorResetsAt::decode_at)?;
        Ok(Self {
            code,
            type_,
            message,
            metadata,
            resets_at,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.code {
            object.insert("code", held.encode());
        }
        if let Some(held) = &self.type_ {
            object.insert("type", Value::from(held.as_str()));
        }
        if let Some(held) = &self.message {
            object.insert("message", Value::from(held.as_str()));
        }
        if let Some(held) = &self.metadata {
            object.insert("metadata", shape::record_value(held, |held| held.clone()));
        }
        if let Some(held) = &self.resets_at {
            object.insert("resetsAt", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WireErrorCode {
    Number(f64),
    Text(String),
}

impl Shape for WireErrorCode {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = shape::number(value, at) {
            return Ok(Self::Number(held));
        }
        if let Ok(held) = shape::text(value, at) {
            return Ok(Self::Text(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Number(held) => Value::Number(*held),
            Self::Text(held) => Value::from(held.as_str()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WireErrorResetsAt {
    Number(f64),
    Text(String),
}

impl Shape for WireErrorResetsAt {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = shape::number(value, at) {
            return Ok(Self::Number(held));
        }
        if let Ok(held) = shape::text(value, at) {
            return Ok(Self::Text(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Number(held) => Value::Number(*held),
            Self::Text(held) => Value::from(held.as_str()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailureExtra {
    pub status: Option<FailureExtraStatus>,
    pub retry_after_ms: Option<i64>,
    pub resets_at: Option<i64>,
    pub reasons: Option<Vec<String>>,
    pub billed: Option<FailureExtraBilled>,
}

impl Shape for FailureExtra {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let status = shape::optional(object, "status", at, FailureExtraStatus::decode_at)?;
        let retry_after_ms = shape::optional(object, "retryAfterMs", at, shape::integer)?;
        let resets_at = shape::optional(object, "resetsAt", at, shape::integer)?;
        let reasons = shape::optional(object, "reasons", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let billed = shape::optional(object, "billed", at, FailureExtraBilled::decode_at)?;
        Ok(Self {
            status,
            retry_after_ms,
            resets_at,
            reasons,
            billed,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.status {
            object.insert("status", held.encode());
        }
        if let Some(held) = &self.retry_after_ms {
            object.insert("retryAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.resets_at {
            object.insert("resetsAt", Value::Number(*held as f64));
        }
        if let Some(held) = &self.reasons {
            object.insert(
                "reasons",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.billed {
            object.insert("billed", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FailureExtraStatus {
    Number(f64),
    Text(String),
}

impl Shape for FailureExtraStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = shape::number(value, at) {
            return Ok(Self::Number(held));
        }
        if let Ok(held) = shape::text(value, at) {
            return Ok(Self::Text(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Number(held) => Value::Number(*held),
            Self::Text(held) => Value::from(held.as_str()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailureExtraBilled {
    pub usage: Option<nyaaai_contract::shapes::step::NormalizedUsage>,
    pub host: Option<String>,
    pub generation_id: Option<String>,
}

impl Shape for FailureExtraBilled {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let usage = shape::optional(
            object,
            "usage",
            at,
            nyaaai_contract::shapes::step::NormalizedUsage::decode_at,
        )?;
        let host = shape::optional(object, "host", at, shape::text)?;
        let generation_id = shape::optional(object, "generationId", at, shape::text)?;
        Ok(Self {
            usage,
            host,
            generation_id,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.usage {
            object.insert("usage", held.encode());
        }
        if let Some(held) = &self.host {
            object.insert("host", Value::from(held.as_str()));
        }
        if let Some(held) = &self.generation_id {
            object.insert("generationId", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailureMessageInput {
    pub values: Vec<Value>,
}

impl Shape for FailureMessageInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("values", Value::Array(self.values.clone()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailureMessageResult {
    pub messages: Vec<Option<String>>,
}

impl Shape for FailureMessageResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let messages = shape::required(object, "messages", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { messages })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "messages",
            Value::Array(
                self.messages
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailureOfInput {
    pub thrown: Vec<FailureOfInputThrownItem>,
}

impl Shape for FailureOfInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let thrown = shape::required(object, "thrown", at, |value, at| {
            shape::list(value, at, FailureOfInputThrownItem::decode_at)
        })?;
        Ok(Self { thrown })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "thrown",
            Value::Array(self.thrown.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FailureOfInputThrownItem {
    Failed(FailureOfInputThrownItemFailed),
    Provider(FailureOfInputThrownItemProvider),
    Error(FailureOfInputThrownItemError),
}

impl Shape for FailureOfInputThrownItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "make", at)? {
            "failed" => FailureOfInputThrownItemFailed::decode_at(value, at).map(Self::Failed),
            "provider" => {
                FailureOfInputThrownItemProvider::decode_at(value, at).map(Self::Provider)
            }
            "error" => FailureOfInputThrownItemError::decode_at(value, at).map(Self::Error),
            _ => Err(shape::no_branch(at, "make")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Failed(held) => held.encode(),
            Self::Provider(held) => held.encode(),
            Self::Error(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailureOfInputThrownItemFailed {
    pub category: nyaaai_contract::shapes::step::FailureCategory,
    pub message: String,
    pub extra: Option<FailureExtra>,
}

impl Shape for FailureOfInputThrownItemFailed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "make", at, |value, at| {
            shape::is_text(value, at, "failed")
        })?;
        let category = shape::required(
            object,
            "category",
            at,
            nyaaai_contract::shapes::step::FailureCategory::decode_at,
        )?;
        let message = shape::required(object, "message", at, shape::text)?;
        let extra = shape::optional(object, "extra", at, FailureExtra::decode_at)?;
        Ok(Self {
            category,
            message,
            extra,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("make", Value::from("failed"));
        object.insert("category", self.category.encode());
        object.insert("message", Value::from(self.message.as_str()));
        if let Some(held) = &self.extra {
            object.insert("extra", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailureOfInputThrownItemProvider {
    pub failure: nyaaai_contract::shapes::step::Failure,
}

impl Shape for FailureOfInputThrownItemProvider {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "make", at, |value, at| {
            shape::is_text(value, at, "provider")
        })?;
        let failure = shape::required(
            object,
            "failure",
            at,
            nyaaai_contract::shapes::step::Failure::decode_at,
        )?;
        Ok(Self { failure })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("make", Value::from("provider"));
        object.insert("failure", self.failure.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailureOfInputThrownItemError {
    pub message: String,
}

impl Shape for FailureOfInputThrownItemError {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "make", at, |value, at| {
            shape::is_text(value, at, "error")
        })?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self { message })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("make", Value::from("error"));
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailureOfResult {
    pub failures: Vec<Option<nyaaai_contract::shapes::step::Failure>>,
}

impl Shape for FailureOfResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let failures = shape::required(object, "failures", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, nyaaai_contract::shapes::step::Failure::decode_at)
            })
        })?;
        Ok(Self { failures })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "failures",
            Value::Array(
                self.failures
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeReadersInput {
    pub texts: Vec<String>,
}

impl Shape for NativeReadersInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeReadersResult {
    pub rows: Vec<NativeReadersResultRowsItem>,
}

impl Shape for NativeReadersResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, NativeReadersResultRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeReadersResultRowsItem {
    pub is_json: bool,
    pub string: String,
    pub number: Option<f64>,
}

impl Shape for NativeReadersResultRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let is_json = shape::required(object, "isJson", at, shape::flag)?;
        let string = shape::required(object, "string", at, shape::text)?;
        let number = shape::required(object, "number", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        Ok(Self {
            is_json,
            string,
            number,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("isJson", Value::Bool(self.is_json));
        object.insert("string", Value::from(self.string.as_str()));
        object.insert(
            "number",
            self.number
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeSchemaPromptInput {
    pub schemas: Vec<Record<Value>>,
}

impl Shape for NativeSchemaPromptInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let schemas = shape::required(object, "schemas", at, |value, at| {
            shape::list(value, at, |value, at| shape::record(value, at, shape::any))
        })?;
        Ok(Self { schemas })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "schemas",
            Value::Array(
                self.schemas
                    .iter()
                    .map(|held| shape::record_value(held, |held| held.clone()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeSchemaPromptResult {
    pub texts: Vec<String>,
}

impl Shape for NativeSchemaPromptResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeSystemTextsInput {
    pub rows: Vec<NativeSystemTextsInputRowsItem>,
}

impl Shape for NativeSystemTextsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, NativeSystemTextsInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeSystemTextsInputRowsItem {
    pub system: Vec<String>,
    pub summary: Option<String>,
    pub prompted: Option<String>,
}

impl Shape for NativeSystemTextsInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let system = shape::required(object, "system", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let summary = shape::required(object, "summary", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let prompted = shape::required(object, "prompted", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            system,
            summary,
            prompted,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "system",
            Value::Array(
                self.system
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "summary",
            self.summary
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "prompted",
            self.prompted
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeSystemTextsResult {
    pub texts: Vec<Vec<String>>,
}

impl Shape for NativeSystemTextsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| {
                        Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeArgumentsInput {
    pub texts: Vec<String>,
}

impl Shape for NativeArgumentsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeArgumentsResult {
    pub values: Vec<Record<Value>>,
}

impl Shape for NativeArgumentsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, |value, at| shape::record(value, at, shape::any))
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(
                self.values
                    .iter()
                    .map(|held| shape::record_value(held, |held| held.clone()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeCurrentTurnInput {
    pub histories: Vec<Vec<nyaaai_contract::shapes::step::Turn>>,
}

impl Shape for NativeCurrentTurnInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let histories = shape::required(object, "histories", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, nyaaai_contract::shapes::step::Turn::decode_at)
            })
        })?;
        Ok(Self { histories })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "histories",
            Value::Array(
                self.histories
                    .iter()
                    .map(|held| Value::Array(held.iter().map(|held| held.encode()).collect()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeCurrentTurnResult {
    pub indexes: Vec<i64>,
}

impl Shape for NativeCurrentTurnResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let indexes = shape::required(object, "indexes", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        Ok(Self { indexes })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "indexes",
            Value::Array(
                self.indexes
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeChunkInput {
    pub texts: Vec<String>,
}

impl Shape for NativeChunkInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeChunkResult {
    pub rows: Vec<NativeChunkResultRowsItem>,
}

impl Shape for NativeChunkResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, NativeChunkResultRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeChunkResultRowsItem {
    pub chunk: Option<Record<Value>>,
    pub failure: Option<nyaaai_contract::shapes::step::Failure>,
}

impl Shape for NativeChunkResultRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let chunk = shape::required(object, "chunk", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::record(value, at, shape::any))
        })?;
        let failure = shape::required(object, "failure", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::step::Failure::decode_at)
        })?;
        Ok(Self { chunk, failure })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "chunk",
            self.chunk.as_ref().map_or(Value::Null, |held| {
                shape::record_value(held, |held| held.clone())
            }),
        );
        object.insert(
            "failure",
            self.failure
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeWireErrorInput {
    pub rows: Vec<NativeWireErrorInputRowsItem>,
}

impl Shape for NativeWireErrorInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, NativeWireErrorInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeWireErrorInputRowsItem {
    pub status: i64,
    pub body: String,
}

impl Shape for NativeWireErrorInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let status = shape::required(object, "status", at, shape::integer)?;
        let body = shape::required(object, "body", at, shape::text)?;
        Ok(Self { status, body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("status", Value::Number(self.status as f64));
        object.insert("body", Value::from(self.body.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeWireErrorResult {
    pub errors: Vec<WireError>,
}

impl Shape for NativeWireErrorResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let errors = shape::required(object, "errors", at, |value, at| {
            shape::list(value, at, WireError::decode_at)
        })?;
        Ok(Self { errors })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "errors",
            Value::Array(self.errors.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeStreamFailureInput {
    pub error: WireError,
    pub usage: Option<nyaaai_contract::shapes::step::NormalizedUsage>,
    pub serving: nyaaai_contract::shapes::step::Serving,
}

impl Shape for NativeStreamFailureInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let error = shape::required(object, "error", at, WireError::decode_at)?;
        let usage = shape::required(object, "usage", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::step::NormalizedUsage::decode_at,
            )
        })?;
        let serving = shape::required(
            object,
            "serving",
            at,
            nyaaai_contract::shapes::step::Serving::decode_at,
        )?;
        Ok(Self {
            error,
            usage,
            serving,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("error", self.error.encode());
        object.insert(
            "usage",
            self.usage
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert("serving", self.serving.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeStreamFailureResult {
    pub failure: nyaaai_contract::shapes::step::Failure,
}

impl Shape for NativeStreamFailureResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let failure = shape::required(
            object,
            "failure",
            at,
            nyaaai_contract::shapes::step::Failure::decode_at,
        )?;
        Ok(Self { failure })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("failure", self.failure.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeAssembledInput {
    pub events: Vec<nyaaai_contract::shapes::step::StepEvent>,
}

impl Shape for NativeAssembledInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let events = shape::required(object, "events", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::step::StepEvent::decode_at,
            )
        })?;
        Ok(Self { events })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "events",
            Value::Array(self.events.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeAssembledResult {
    pub result: Option<nyaaai_contract::shapes::step::StepResult>,
    pub failure: Option<nyaaai_contract::shapes::step::Failure>,
}

impl Shape for NativeAssembledResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let result = shape::required(object, "result", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::step::StepResult::decode_at,
            )
        })?;
        let failure = shape::required(object, "failure", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::step::Failure::decode_at)
        })?;
        Ok(Self { result, failure })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "result",
            self.result
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "failure",
            self.failure
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeServingInput {
    pub ids: Vec<Value>,
}

impl Shape for NativeServingInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let ids = shape::required(object, "ids", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        Ok(Self { ids })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("ids", Value::Array(self.ids.clone()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NativeServingResult {
    pub events: Vec<Vec<nyaaai_contract::shapes::step::StepEvent>>,
}

impl Shape for NativeServingResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let events = shape::required(object, "events", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(
                    value,
                    at,
                    nyaaai_contract::shapes::step::StepEvent::decode_at,
                )
            })
        })?;
        Ok(Self { events })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "events",
            Value::Array(
                self.events
                    .iter()
                    .map(|held| Value::Array(held.iter().map(|held| held.encode()).collect()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaProjectionProjectInput {
    pub projection: SchemaProjectionProjectInputProjection,
    pub schema: Record<Value>,
    pub value: Value,
}

impl Shape for SchemaProjectionProjectInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let projection = shape::required(
            object,
            "projection",
            at,
            SchemaProjectionProjectInputProjection::decode_at,
        )?;
        let schema = shape::required(object, "schema", at, |value, at| {
            shape::record(value, at, shape::any)
        })?;
        let value_field = shape::required(object, "value", at, shape::any)?;
        Ok(Self {
            projection,
            schema,
            value: value_field,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("projection", self.projection.encode());
        object.insert(
            "schema",
            shape::record_value(&self.schema, |held| held.clone()),
        );
        object.insert("value", self.value.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaProjectionProjectInputProjection {
    Clean,
    Nullable,
    WithoutNulls,
    Strict,
}

impl SchemaProjectionProjectInputProjection {
    pub const ALL: [Self; 4] = [
        Self::Clean,
        Self::Nullable,
        Self::WithoutNulls,
        Self::Strict,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Nullable => "nullable",
            Self::WithoutNulls => "withoutNulls",
            Self::Strict => "strict",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for SchemaProjectionProjectInputProjection {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaProjectionProjectResult {
    pub output: Value,
    pub refusal: Option<String>,
}

impl Shape for SchemaProjectionProjectResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let output = shape::required(object, "output", at, shape::any)?;
        let refusal = shape::required(object, "refusal", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self { output, refusal })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("output", self.output.clone());
        object.insert(
            "refusal",
            self.refusal
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalName {
    pub host: String,
    pub label: String,
    pub replay: Option<LocalNameReplay>,
    pub chat_path: Option<String>,
    pub optional_key: Option<bool>,
}

impl Shape for LocalName {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let host = shape::required(object, "host", at, shape::text)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let replay = shape::optional(object, "replay", at, LocalNameReplay::decode_at)?;
        let chat_path = shape::optional(object, "chatPath", at, shape::text)?;
        let optional_key = shape::optional(object, "optionalKey", at, shape::flag)?;
        Ok(Self {
            host,
            label,
            replay,
            chat_path,
            optional_key,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("host", Value::from(self.host.as_str()));
        object.insert("label", Value::from(self.label.as_str()));
        if let Some(held) = &self.replay {
            object.insert("replay", held.encode());
        }
        if let Some(held) = &self.chat_path {
            object.insert("chatPath", Value::from(held.as_str()));
        }
        if let Some(held) = &self.optional_key {
            object.insert("optionalKey", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalNameReplay {
    Reasoning,
    ReasoningContent,
}

impl LocalNameReplay {
    pub const ALL: [Self; 2] = [Self::Reasoning, Self::ReasoningContent];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reasoning => "reasoning",
            Self::ReasoningContent => "reasoning_content",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalNameReplay {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodecName {
    pub family: nyaaai_contract::shapes::step::ApiFamily,
    pub dialect: String,
    pub facts: Option<nyaaai_contract::shapes::step::CodecFacts>,
    pub local: Option<LocalName>,
    pub now_ms: Option<i64>,
    pub prefix: Option<String>,
}

impl Shape for CodecName {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let family = shape::required(
            object,
            "family",
            at,
            nyaaai_contract::shapes::step::ApiFamily::decode_at,
        )?;
        let dialect = shape::required(object, "dialect", at, shape::text)?;
        let facts = shape::optional(
            object,
            "facts",
            at,
            nyaaai_contract::shapes::step::CodecFacts::decode_at,
        )?;
        let local = shape::optional(object, "local", at, LocalName::decode_at)?;
        let now_ms = shape::optional(object, "nowMs", at, shape::integer)?;
        let prefix = shape::optional(object, "prefix", at, shape::text)?;
        Ok(Self {
            family,
            dialect,
            facts,
            local,
            now_ms,
            prefix,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("family", self.family.encode());
        object.insert("dialect", Value::from(self.dialect.as_str()));
        if let Some(held) = &self.facts {
            object.insert("facts", held.encode());
        }
        if let Some(held) = &self.local {
            object.insert("local", held.encode());
        }
        if let Some(held) = &self.now_ms {
            object.insert("nowMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.prefix {
            object.insert("prefix", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodecEncodeInput {
    pub codec: CodecName,
    pub request: nyaaai_contract::shapes::step::CanonicalRequest,
    pub mode: CodecEncodeInputMode,
}

impl Shape for CodecEncodeInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let codec = shape::required(object, "codec", at, CodecName::decode_at)?;
        let request = shape::required(
            object,
            "request",
            at,
            nyaaai_contract::shapes::step::CanonicalRequest::decode_at,
        )?;
        let mode = shape::required(object, "mode", at, CodecEncodeInputMode::decode_at)?;
        Ok(Self {
            codec,
            request,
            mode,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("codec", self.codec.encode());
        object.insert("request", self.request.encode());
        object.insert("mode", self.mode.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodecEncodeInputMode {
    Stream,
    Complete,
}

impl CodecEncodeInputMode {
    pub const ALL: [Self; 2] = [Self::Stream, Self::Complete];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stream => "stream",
            Self::Complete => "complete",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for CodecEncodeInputMode {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodecEncodeResult {
    pub encoded: Option<nyaaai_contract::shapes::step::Encoded>,
    pub refusal: Option<String>,
}

impl Shape for CodecEncodeResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let encoded = shape::required(object, "encoded", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::step::Encoded::decode_at)
        })?;
        let refusal = shape::required(object, "refusal", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self { encoded, refusal })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "encoded",
            self.encoded
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "refusal",
            self.refusal
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodecStreamInput {
    pub codec: CodecName,
    pub items: Vec<nyaaai_contract::shapes::step::SseItem>,
}

impl Shape for CodecStreamInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let codec = shape::required(object, "codec", at, CodecName::decode_at)?;
        let items = shape::required(object, "items", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::SseItem::decode_at)
        })?;
        Ok(Self { codec, items })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("codec", self.codec.encode());
        object.insert(
            "items",
            Value::Array(self.items.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodecStreamResult {
    pub events: Vec<nyaaai_contract::shapes::step::StepEvent>,
    pub result: Option<nyaaai_contract::shapes::step::StepResult>,
    pub failure: Option<nyaaai_contract::shapes::step::Failure>,
}

impl Shape for CodecStreamResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let events = shape::required(object, "events", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::step::StepEvent::decode_at,
            )
        })?;
        let result = shape::required(object, "result", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::step::StepResult::decode_at,
            )
        })?;
        let failure = shape::required(object, "failure", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::step::Failure::decode_at)
        })?;
        Ok(Self {
            events,
            result,
            failure,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "events",
            Value::Array(self.events.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "result",
            self.result
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "failure",
            self.failure
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodecCompleteInput {
    pub codec: CodecName,
    pub body: Value,
}

impl Shape for CodecCompleteInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let codec = shape::required(object, "codec", at, CodecName::decode_at)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { codec, body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("codec", self.codec.encode());
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodecCompleteResult {
    pub result: Option<nyaaai_contract::shapes::step::StepResult>,
    pub failure: Option<nyaaai_contract::shapes::step::Failure>,
}

impl Shape for CodecCompleteResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let result = shape::required(object, "result", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::step::StepResult::decode_at,
            )
        })?;
        let failure = shape::required(object, "failure", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::step::Failure::decode_at)
        })?;
        Ok(Self { result, failure })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "result",
            self.result
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "failure",
            self.failure
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodecFailureInput {
    pub codec: CodecName,
    pub status: i64,
    pub body: Value,
    pub headers: nyaaai_contract::shapes::step::HeaderList,
}

impl Shape for CodecFailureInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let codec = shape::required(object, "codec", at, CodecName::decode_at)?;
        let status = shape::required(object, "status", at, shape::integer)?;
        let body = shape::required(object, "body", at, shape::any)?;
        let headers = shape::required(
            object,
            "headers",
            at,
            nyaaai_contract::shapes::step::decode_header_list,
        )?;
        Ok(Self {
            codec,
            status,
            body,
            headers,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("codec", self.codec.encode());
        object.insert("status", Value::Number(self.status as f64));
        object.insert("body", self.body.clone());
        object.insert(
            "headers",
            nyaaai_contract::shapes::step::encode_header_list(&self.headers),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodecFailureResult {
    pub failure: nyaaai_contract::shapes::step::Failure,
}

impl Shape for CodecFailureResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let failure = shape::required(
            object,
            "failure",
            at,
            nyaaai_contract::shapes::step::Failure::decode_at,
        )?;
        Ok(Self { failure })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("failure", self.failure.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransportChunk {
    Text(TransportChunkText),
    Bytes(TransportChunkBytes),
    Repeat(TransportChunkRepeat),
}

impl Shape for TransportChunk {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "text" => TransportChunkText::decode_at(value, at).map(Self::Text),
            "bytes" => TransportChunkBytes::decode_at(value, at).map(Self::Bytes),
            "repeat" => TransportChunkRepeat::decode_at(value, at).map(Self::Repeat),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Text(held) => held.encode(),
            Self::Bytes(held) => held.encode(),
            Self::Repeat(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportChunkText {
    pub text: String,
    pub delay_ms: Option<i64>,
}

impl Shape for TransportChunkText {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "text")
        })?;
        let text = shape::required(object, "text", at, shape::text)?;
        let delay_ms = shape::optional(object, "delayMs", at, shape::integer)?;
        Ok(Self { text, delay_ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("text"));
        object.insert("text", Value::from(self.text.as_str()));
        if let Some(held) = &self.delay_ms {
            object.insert("delayMs", Value::Number(*held as f64));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportChunkBytes {
    pub base64: String,
    pub delay_ms: Option<i64>,
}

impl Shape for TransportChunkBytes {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "bytes")
        })?;
        let base64 = shape::required(object, "base64", at, shape::text)?;
        let delay_ms = shape::optional(object, "delayMs", at, shape::integer)?;
        Ok(Self { base64, delay_ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("bytes"));
        object.insert("base64", Value::from(self.base64.as_str()));
        if let Some(held) = &self.delay_ms {
            object.insert("delayMs", Value::Number(*held as f64));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportChunkRepeat {
    pub text: String,
    pub times: u64,
    pub delay_ms: Option<i64>,
}

impl Shape for TransportChunkRepeat {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "repeat")
        })?;
        let text = shape::required(object, "text", at, shape::text)?;
        let times = shape::required(object, "times", at, shape::natural)?;
        let delay_ms = shape::optional(object, "delayMs", at, shape::integer)?;
        Ok(Self {
            text,
            times,
            delay_ms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("repeat"));
        object.insert("text", Value::from(self.text.as_str()));
        object.insert("times", Value::Number(self.times as f64));
        if let Some(held) = &self.delay_ms {
            object.insert("delayMs", Value::Number(*held as f64));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportExchange {
    pub status: i64,
    pub headers: nyaaai_contract::shapes::step::HeaderList,
    pub chunks: Vec<TransportChunk>,
    pub head_ms: Option<i64>,
    pub end: Option<TransportExchangeEnd>,
    pub error: Option<String>,
}

impl Shape for TransportExchange {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let status = shape::required(object, "status", at, shape::integer)?;
        let headers = shape::required(
            object,
            "headers",
            at,
            nyaaai_contract::shapes::step::decode_header_list,
        )?;
        let chunks = shape::required(object, "chunks", at, |value, at| {
            shape::list(value, at, TransportChunk::decode_at)
        })?;
        let head_ms = shape::optional(object, "headMs", at, shape::integer)?;
        let end = shape::optional(object, "end", at, TransportExchangeEnd::decode_at)?;
        let error = shape::optional(object, "error", at, shape::text)?;
        Ok(Self {
            status,
            headers,
            chunks,
            head_ms,
            end,
            error,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("status", Value::Number(self.status as f64));
        object.insert(
            "headers",
            nyaaai_contract::shapes::step::encode_header_list(&self.headers),
        );
        object.insert(
            "chunks",
            Value::Array(self.chunks.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.head_ms {
            object.insert("headMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.end {
            object.insert("end", held.encode());
        }
        if let Some(held) = &self.error {
            object.insert("error", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportExchangeEnd {
    Closed,
    Dropped,
    Refused,
}

impl TransportExchangeEnd {
    pub const ALL: [Self; 3] = [Self::Closed, Self::Dropped, Self::Refused];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Dropped => "dropped",
            Self::Refused => "refused",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TransportExchangeEnd {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransportKeyAuth {
    None(TransportKeyAuthNone),
    Stored(TransportKeyAuthStored),
    Key(TransportKeyAuthKey),
}

impl Shape for TransportKeyAuth {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "none" => TransportKeyAuthNone::decode_at(value, at).map(Self::None),
            "stored" => TransportKeyAuthStored::decode_at(value, at).map(Self::Stored),
            "key" => TransportKeyAuthKey::decode_at(value, at).map(Self::Key),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::None(held) => held.encode(),
            Self::Stored(held) => held.encode(),
            Self::Key(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportKeyAuthNone {}

impl Shape for TransportKeyAuthNone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "none")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("none"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportKeyAuthStored {}

impl Shape for TransportKeyAuthStored {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "stored")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("stored"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportKeyAuthKey {
    pub key: String,
}

impl Shape for TransportKeyAuthKey {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "key")
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        Ok(Self { key })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("key"));
        object.insert("key", Value::from(self.key.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransportCall {
    Stream(TransportCallStream),
    Complete(TransportCallComplete),
    GetJson(TransportCallGetJson),
    PostJson(TransportCallPostJson),
}

impl Shape for TransportCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "stream" => TransportCallStream::decode_at(value, at).map(Self::Stream),
            "complete" => TransportCallComplete::decode_at(value, at).map(Self::Complete),
            "getJson" => TransportCallGetJson::decode_at(value, at).map(Self::GetJson),
            "postJson" => TransportCallPostJson::decode_at(value, at).map(Self::PostJson),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Stream(held) => held.encode(),
            Self::Complete(held) => held.encode(),
            Self::GetJson(held) => held.encode(),
            Self::PostJson(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportCallStream {
    pub request: nyaaai_contract::shapes::step::CanonicalRequest,
    pub stop_after: Option<u64>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for TransportCallStream {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "stream")
        })?;
        let request = shape::required(
            object,
            "request",
            at,
            nyaaai_contract::shapes::step::CanonicalRequest::decode_at,
        )?;
        let stop_after = shape::optional(object, "stopAfter", at, |value, at| {
            shape::natural(value, at).and_then(|held| shape::at_least(held, 1, at))
        })?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            request,
            stop_after,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("stream"));
        object.insert("request", self.request.encode());
        if let Some(held) = &self.stop_after {
            object.insert("stopAfter", Value::Number(*held as f64));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportCallComplete {
    pub request: nyaaai_contract::shapes::step::CanonicalRequest,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for TransportCallComplete {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "complete")
        })?;
        let request = shape::required(
            object,
            "request",
            at,
            nyaaai_contract::shapes::step::CanonicalRequest::decode_at,
        )?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            request,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("complete"));
        object.insert("request", self.request.encode());
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportCallGetJson {
    pub path: String,
    pub key_auth: Option<TransportKeyAuth>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for TransportCallGetJson {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "getJson")
        })?;
        let path = shape::required(object, "path", at, shape::text)?;
        let key_auth = shape::optional(object, "keyAuth", at, TransportKeyAuth::decode_at)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            path,
            key_auth,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("getJson"));
        object.insert("path", Value::from(self.path.as_str()));
        if let Some(held) = &self.key_auth {
            object.insert("keyAuth", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportCallPostJson {
    pub path: String,
    pub body: Option<Value>,
    pub key_auth: Option<TransportKeyAuth>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for TransportCallPostJson {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "postJson")
        })?;
        let path = shape::required(object, "path", at, shape::text)?;
        let body = shape::optional(object, "body", at, shape::any)?;
        let key_auth = shape::optional(object, "keyAuth", at, TransportKeyAuth::decode_at)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            path,
            body,
            key_auth,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("postJson"));
        object.insert("path", Value::from(self.path.as_str()));
        if let Some(held) = &self.body {
            object.insert("body", held.clone());
        }
        if let Some(held) = &self.key_auth {
            object.insert("keyAuth", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransportScript {
    Inline(TransportScriptInline),
    Trace(TransportScriptTrace),
}

impl Shape for TransportScript {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "inline" => TransportScriptInline::decode_at(value, at).map(Self::Inline),
            "trace" => TransportScriptTrace::decode_at(value, at).map(Self::Trace),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Inline(held) => held.encode(),
            Self::Trace(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportScriptInline {
    pub exchanges: Vec<TransportExchange>,
}

impl Shape for TransportScriptInline {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "inline")
        })?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, TransportExchange::decode_at)
        })?;
        Ok(Self { exchanges })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("inline"));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportScriptTrace {
    pub file: String,
}

impl Shape for TransportScriptTrace {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "trace")
        })?;
        let file = shape::required(object, "file", at, shape::text)?;
        Ok(Self { file })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("trace"));
        object.insert("file", Value::from(self.file.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportSent {
    pub method: String,
    pub url: String,
    pub headers: nyaaai_contract::shapes::step::HeaderList,
    pub body: Option<String>,
}

impl Shape for TransportSent {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let method = shape::required(object, "method", at, shape::text)?;
        let url = shape::required(object, "url", at, shape::text)?;
        let headers = shape::required(
            object,
            "headers",
            at,
            nyaaai_contract::shapes::step::decode_header_list,
        )?;
        let body = shape::required(object, "body", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            method,
            url,
            headers,
            body,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("method", Value::from(self.method.as_str()));
        object.insert("url", Value::from(self.url.as_str()));
        object.insert(
            "headers",
            nyaaai_contract::shapes::step::encode_header_list(&self.headers),
        );
        object.insert(
            "body",
            self.body
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportOutcome {
    pub events: Vec<nyaaai_contract::shapes::step::StepEvent>,
    pub result: Option<nyaaai_contract::shapes::step::StepResult>,
    pub answer: Value,
    pub failure: Option<nyaaai_contract::shapes::step::Failure>,
    pub thrown: Option<String>,
    pub aborted: bool,
    pub now_ms: i64,
}

impl Shape for TransportOutcome {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let events = shape::required(object, "events", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::step::StepEvent::decode_at,
            )
        })?;
        let result = shape::required(object, "result", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::step::StepResult::decode_at,
            )
        })?;
        let answer = shape::required(object, "answer", at, shape::any)?;
        let failure = shape::required(object, "failure", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::step::Failure::decode_at)
        })?;
        let thrown = shape::required(object, "thrown", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let aborted = shape::required(object, "aborted", at, shape::flag)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        Ok(Self {
            events,
            result,
            answer,
            failure,
            thrown,
            aborted,
            now_ms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "events",
            Value::Array(self.events.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "result",
            self.result
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert("answer", self.answer.clone());
        object.insert(
            "failure",
            self.failure
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "thrown",
            self.thrown
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert("aborted", Value::Bool(self.aborted));
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportExchangeInput {
    pub codec: CodecName,
    pub codec_headers: Option<Record<String>>,
    pub base_url: String,
    pub label: String,
    pub user_agent: String,
    pub auth: Option<TransportExchangeInputAuth>,
    pub idle_ms: Option<i64>,
    pub max_bytes: Option<i64>,
    pub keys: Vec<Option<String>>,
    pub start_ms: i64,
    pub script: TransportScript,
    pub calls: Vec<TransportCall>,
}

impl Shape for TransportExchangeInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let codec = shape::required(object, "codec", at, CodecName::decode_at)?;
        let codec_headers = shape::optional(object, "codecHeaders", at, |value, at| {
            shape::record(value, at, shape::text)
        })?;
        let base_url = shape::required(object, "baseUrl", at, shape::text)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let user_agent = shape::required(object, "userAgent", at, shape::text)?;
        let auth = shape::optional(object, "auth", at, TransportExchangeInputAuth::decode_at)?;
        let idle_ms = shape::optional(object, "idleMs", at, shape::integer)?;
        let max_bytes = shape::optional(object, "maxBytes", at, shape::integer)?;
        let keys = shape::required(object, "keys", at, |value, at| {
            shape::list_within(value, at, Some(1), None, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let script = shape::required(object, "script", at, TransportScript::decode_at)?;
        let calls = shape::required(object, "calls", at, |value, at| {
            shape::list_within(value, at, Some(1), None, TransportCall::decode_at)
        })?;
        Ok(Self {
            codec,
            codec_headers,
            base_url,
            label,
            user_agent,
            auth,
            idle_ms,
            max_bytes,
            keys,
            start_ms,
            script,
            calls,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("codec", self.codec.encode());
        if let Some(held) = &self.codec_headers {
            object.insert(
                "codecHeaders",
                shape::record_value(held, |held| Value::from(held.as_str())),
            );
        }
        object.insert("baseUrl", Value::from(self.base_url.as_str()));
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("userAgent", Value::from(self.user_agent.as_str()));
        if let Some(held) = &self.auth {
            object.insert("auth", held.encode());
        }
        if let Some(held) = &self.idle_ms {
            object.insert("idleMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.max_bytes {
            object.insert("maxBytes", Value::Number(*held as f64));
        }
        object.insert(
            "keys",
            Value::Array(
                self.keys
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert("script", self.script.encode());
        object.insert(
            "calls",
            Value::Array(self.calls.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportExchangeInputAuth {
    Bearer,
    None,
    Optional,
}

impl TransportExchangeInputAuth {
    pub const ALL: [Self; 3] = [Self::Bearer, Self::None, Self::Optional];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bearer => "bearer",
            Self::None => "none",
            Self::Optional => "optional",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for TransportExchangeInputAuth {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportExchangeResult {
    pub calls: Vec<TransportOutcome>,
    pub requests: Vec<TransportSent>,
    pub log: Vec<String>,
    pub settled: Vec<String>,
    pub key_reads: i64,
    pub read: Vec<i64>,
    pub waiting: Vec<String>,
}

impl Shape for TransportExchangeResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let calls = shape::required(object, "calls", at, |value, at| {
            shape::list(value, at, TransportOutcome::decode_at)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, TransportSent::decode_at)
        })?;
        let log = shape::required(object, "log", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let settled = shape::required(object, "settled", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let key_reads = shape::required(object, "keyReads", at, shape::integer)?;
        let read = shape::required(object, "read", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            calls,
            requests,
            log,
            settled,
            key_reads,
            read,
            waiting,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "calls",
            Value::Array(self.calls.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "requests",
            Value::Array(self.requests.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "log",
            Value::Array(
                self.log
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "settled",
            Value::Array(
                self.settled
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("keyReads", Value::Number(self.key_reads as f64));
        object.insert(
            "read",
            Value::Array(
                self.read
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportKeyInput {
    pub text: String,
}

impl Shape for TransportKeyInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let text = shape::required(object, "text", at, shape::text)?;
        Ok(Self { text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("text", Value::from(self.text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransportKeyResult {
    pub shape: bool,
    pub normalized: Option<String>,
}

impl Shape for TransportKeyResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let shape_field = shape::required(object, "shape", at, shape::flag)?;
        let normalized = shape::required(object, "normalized", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            shape: shape_field,
            normalized,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("shape", Value::Bool(self.shape));
        object.insert(
            "normalized",
            self.normalized
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SecretSpellingsInput {
    pub secret: String,
}

impl Shape for SecretSpellingsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let secret = shape::required(object, "secret", at, shape::text)?;
        Ok(Self { secret })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("secret", Value::from(self.secret.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SecretSpellingsResult {
    pub spellings: Vec<SecretSpellingsResultSpellingsItem>,
}

impl Shape for SecretSpellingsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let spellings = shape::required(object, "spellings", at, |value, at| {
            shape::list(value, at, SecretSpellingsResultSpellingsItem::decode_at)
        })?;
        Ok(Self { spellings })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "spellings",
            Value::Array(self.spellings.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SecretSpellingsResultSpellingsItem {
    pub text: String,
    pub folded: Vec<i64>,
}

impl Shape for SecretSpellingsResultSpellingsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let text = shape::required(object, "text", at, shape::text)?;
        let folded = shape::required(object, "folded", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        Ok(Self { text, folded })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("text", Value::from(self.text.as_str()));
        object.insert(
            "folded",
            Value::Array(
                self.folded
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SecretScrubInput {
    pub secret: String,
    pub text: String,
}

impl Shape for SecretScrubInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let secret = shape::required(object, "secret", at, shape::text)?;
        let text = shape::required(object, "text", at, shape::text)?;
        Ok(Self { secret, text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("secret", Value::from(self.secret.as_str()));
        object.insert("text", Value::from(self.text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SecretScrubResult {
    pub scrubbed: String,
    pub leaks: bool,
}

impl Shape for SecretScrubResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let scrubbed = shape::required(object, "scrubbed", at, shape::text)?;
        let leaks = shape::required(object, "leaks", at, shape::flag)?;
        Ok(Self { scrubbed, leaks })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("scrubbed", Value::from(self.scrubbed.as_str()));
        object.insert("leaks", Value::Bool(self.leaks));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryParseInput {
    pub value: Value,
}

impl Shape for EntryParseInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let value_field = shape::required(object, "value", at, shape::any)?;
        Ok(Self { value: value_field })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("value", self.value.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryParseResult {
    pub entry: Option<nyaaai_contract::shapes::provider::ProviderEntry>,
}

impl Shape for EntryParseResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entry = shape::required(object, "entry", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
            )
        })?;
        Ok(Self { entry })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "entry",
            self.entry
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryParseListInput {
    pub value: Value,
}

impl Shape for EntryParseListInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let value_field = shape::required(object, "value", at, shape::any)?;
        Ok(Self { value: value_field })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("value", self.value.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryParseListResult {
    pub entries: Vec<nyaaai_contract::shapes::provider::ProviderEntry>,
}

impl Shape for EntryParseListResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entries = shape::required(object, "entries", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
            )
        })?;
        Ok(Self { entries })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "entries",
            Value::Array(self.entries.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntrySameOriginInput {
    pub rows: Vec<EntrySameOriginInputRowsItem>,
}

impl Shape for EntrySameOriginInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, EntrySameOriginInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntrySameOriginInputRowsItem {
    pub left: Option<String>,
    pub right: Option<String>,
}

impl Shape for EntrySameOriginInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let left = shape::required(object, "left", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let right = shape::required(object, "right", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self { left, right })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "left",
            self.left
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "right",
            self.right
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntrySameOriginResult {
    pub same: Vec<bool>,
}

impl Shape for EntrySameOriginResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let same = shape::required(object, "same", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        Ok(Self { same })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "same",
            Value::Array(self.same.iter().map(|held| Value::Bool(*held)).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryIdInput {
    pub values: Vec<Value>,
}

impl Shape for EntryIdInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("values", Value::Array(self.values.clone()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntryIdResult {
    pub valid: Vec<bool>,
}

impl Shape for EntryIdResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let valid = shape::required(object, "valid", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        Ok(Self { valid })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "valid",
            Value::Array(self.valid.iter().map(|held| Value::Bool(*held)).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntrySecretKeyInput {
    pub ids: Vec<String>,
    pub namespace: String,
}

impl Shape for EntrySecretKeyInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let ids = shape::required(object, "ids", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let namespace = shape::required(object, "namespace", at, shape::text)?;
        Ok(Self { ids, namespace })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "ids",
            Value::Array(
                self.ids
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("namespace", Value::from(self.namespace.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntrySecretKeyResult {
    pub keys: Vec<String>,
}

impl Shape for EntrySecretKeyResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let keys = shape::required(object, "keys", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { keys })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "keys",
            Value::Array(
                self.keys
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowKeyBuildInput {
    pub identities: Vec<nyaaai_contract::shapes::model::RowIdentity>,
}

impl Shape for RowKeyBuildInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let identities = shape::required(object, "identities", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::model::RowIdentity::decode_at,
            )
        })?;
        Ok(Self { identities })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "identities",
            Value::Array(self.identities.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowKeyBuildResult {
    pub keys: Vec<String>,
    pub back: Vec<Option<nyaaai_contract::shapes::model::RowIdentity>>,
}

impl Shape for RowKeyBuildResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let keys = shape::required(object, "keys", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let back = shape::required(object, "back", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(
                    value,
                    at,
                    nyaaai_contract::shapes::model::RowIdentity::decode_at,
                )
            })
        })?;
        Ok(Self { keys, back })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "keys",
            Value::Array(
                self.keys
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "back",
            Value::Array(
                self.back
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowKeyParseInput {
    pub values: Vec<Value>,
}

impl Shape for RowKeyParseInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("values", Value::Array(self.values.clone()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowKeyParseResult {
    pub identities: Vec<Option<nyaaai_contract::shapes::model::RowIdentity>>,
}

impl Shape for RowKeyParseResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let identities = shape::required(object, "identities", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(
                    value,
                    at,
                    nyaaai_contract::shapes::model::RowIdentity::decode_at,
                )
            })
        })?;
        Ok(Self { identities })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "identities",
            Value::Array(
                self.identities
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowKeyShownInput {
    pub identities: Vec<nyaaai_contract::shapes::model::RowIdentity>,
    pub keys: Vec<String>,
}

impl Shape for RowKeyShownInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let identities = shape::required(object, "identities", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::model::RowIdentity::decode_at,
            )
        })?;
        let keys = shape::required(object, "keys", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { identities, keys })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "identities",
            Value::Array(self.identities.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "keys",
            Value::Array(
                self.keys
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowKeyShownResult {
    pub ids: Vec<String>,
    pub keyed: Vec<String>,
}

impl Shape for RowKeyShownResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let ids = shape::required(object, "ids", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let keyed = shape::required(object, "keyed", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { ids, keyed })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "ids",
            Value::Array(
                self.ids
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "keyed",
            Value::Array(
                self.keyed
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowKeyServedIdInput {
    pub identities: Vec<nyaaai_contract::shapes::model::RowIdentity>,
}

impl Shape for RowKeyServedIdInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let identities = shape::required(object, "identities", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::model::RowIdentity::decode_at,
            )
        })?;
        Ok(Self { identities })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "identities",
            Value::Array(self.identities.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowKeyServedIdResult {
    pub ids: Vec<Option<String>>,
}

impl Shape for RowKeyServedIdResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let ids = shape::required(object, "ids", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { ids })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "ids",
            Value::Array(
                self.ids
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CuratedRowsInput {}

impl Shape for CuratedRowsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        shape::object(value, at)?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        Value::Object(Object::new())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CuratedRowsResult {
    pub rows: Vec<nyaaai_contract::shapes::model::ModelRow>,
}

impl Shape for CuratedRowsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::model::ModelRow::decode_at,
            )
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryTestRecordInput {
    pub values: Vec<Value>,
}

impl Shape for RegistryTestRecordInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("values", Value::Array(self.values.clone()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryTestRecordResult {
    pub records: Vec<Option<nyaaai_contract::shapes::model::TestRecord>>,
}

impl Shape for RegistryTestRecordResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let records = shape::required(object, "records", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(
                    value,
                    at,
                    nyaaai_contract::shapes::model::TestRecord::decode_at,
                )
            })
        })?;
        Ok(Self { records })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "records",
            Value::Array(
                self.records
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryParseInput {
    pub raw: RegistryParseInputRaw,
    pub entries: Vec<RegistryParseInputEntriesItem>,
}

impl Shape for RegistryParseInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let raw = shape::required(object, "raw", at, RegistryParseInputRaw::decode_at)?;
        let entries = shape::required(object, "entries", at, |value, at| {
            shape::list(value, at, RegistryParseInputEntriesItem::decode_at)
        })?;
        Ok(Self { raw, entries })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("raw", self.raw.encode());
        object.insert(
            "entries",
            Value::Array(self.entries.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryParseInputRaw {
    pub records: Option<Value>,
    pub added: Option<Value>,
    pub hidden: Option<Value>,
}

impl Shape for RegistryParseInputRaw {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let records = shape::optional(object, "records", at, shape::any)?;
        let added = shape::optional(object, "added", at, shape::any)?;
        let hidden = shape::optional(object, "hidden", at, shape::any)?;
        Ok(Self {
            records,
            added,
            hidden,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.records {
            object.insert("records", held.clone());
        }
        if let Some(held) = &self.added {
            object.insert("added", held.clone());
        }
        if let Some(held) = &self.hidden {
            object.insert("hidden", held.clone());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryParseInputEntriesItem {
    pub id: String,
}

impl Shape for RegistryParseInputEntriesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        Ok(Self { id })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryParseResult {
    pub stored: nyaaai_contract::shapes::model::StoredRegistry,
}

impl Shape for RegistryParseResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let stored = shape::required(
            object,
            "stored",
            at,
            nyaaai_contract::shapes::model::StoredRegistry::decode_at,
        )?;
        Ok(Self { stored })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("stored", self.stored.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryScopedInput {
    pub raw: RegistryScopedInputRaw,
    pub entries: Vec<RegistryScopedInputEntriesItem>,
    pub scopes: Vec<Option<String>>,
}

impl Shape for RegistryScopedInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let raw = shape::required(object, "raw", at, RegistryScopedInputRaw::decode_at)?;
        let entries = shape::required(object, "entries", at, |value, at| {
            shape::list(value, at, RegistryScopedInputEntriesItem::decode_at)
        })?;
        let scopes = shape::required(object, "scopes", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self {
            raw,
            entries,
            scopes,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("raw", self.raw.encode());
        object.insert(
            "entries",
            Value::Array(self.entries.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "scopes",
            Value::Array(
                self.scopes
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryScopedInputRaw {
    pub records: Option<Value>,
    pub added: Option<Value>,
    pub hidden: Option<Value>,
}

impl Shape for RegistryScopedInputRaw {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let records = shape::optional(object, "records", at, shape::any)?;
        let added = shape::optional(object, "added", at, shape::any)?;
        let hidden = shape::optional(object, "hidden", at, shape::any)?;
        Ok(Self {
            records,
            added,
            hidden,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.records {
            object.insert("records", held.clone());
        }
        if let Some(held) = &self.added {
            object.insert("added", held.clone());
        }
        if let Some(held) = &self.hidden {
            object.insert("hidden", held.clone());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryScopedInputEntriesItem {
    pub id: String,
    pub kind: nyaaai_contract::shapes::provider::ProviderKindName,
}

impl Shape for RegistryScopedInputEntriesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let kind = shape::required(
            object,
            "kind",
            at,
            nyaaai_contract::shapes::provider::ProviderKindName::decode_at,
        )?;
        Ok(Self { id, kind })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("kind", self.kind.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryScopedResult {
    pub scoped: Vec<RegistryScopedResultScopedItem>,
    pub merged: RegistryScopedResultMerged,
}

impl Shape for RegistryScopedResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let scoped = shape::required(object, "scoped", at, |value, at| {
            shape::list(value, at, RegistryScopedResultScopedItem::decode_at)
        })?;
        let merged = shape::required(object, "merged", at, RegistryScopedResultMerged::decode_at)?;
        Ok(Self { scoped, merged })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "scoped",
            Value::Array(self.scoped.iter().map(|held| held.encode()).collect()),
        );
        object.insert("merged", self.merged.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryScopedResultScopedItem {
    pub tested: Record<nyaaai_contract::shapes::model::TestRecord>,
    pub added: Vec<String>,
    pub hidden: Vec<String>,
}

impl Shape for RegistryScopedResultScopedItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tested = shape::required(object, "tested", at, |value, at| {
            shape::record(
                value,
                at,
                nyaaai_contract::shapes::model::TestRecord::decode_at,
            )
        })?;
        let added = shape::required(object, "added", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let hidden = shape::required(object, "hidden", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            tested,
            added,
            hidden,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "tested",
            shape::record_value(&self.tested, |held| held.encode()),
        );
        object.insert(
            "added",
            Value::Array(
                self.added
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "hidden",
            Value::Array(
                self.hidden
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryScopedResultMerged {
    pub tested: Record<nyaaai_contract::shapes::model::TestRecord>,
    pub added: Vec<String>,
    pub hidden: Vec<String>,
}

impl Shape for RegistryScopedResultMerged {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tested = shape::required(object, "tested", at, |value, at| {
            shape::record(
                value,
                at,
                nyaaai_contract::shapes::model::TestRecord::decode_at,
            )
        })?;
        let added = shape::required(object, "added", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let hidden = shape::required(object, "hidden", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            tested,
            added,
            hidden,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "tested",
            shape::record_value(&self.tested, |held| held.encode()),
        );
        object.insert(
            "added",
            Value::Array(
                self.added
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "hidden",
            Value::Array(
                self.hidden
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryRowsInput {
    pub raw: RegistryRowsInputRaw,
    pub entries: Vec<RegistryRowsInputEntriesItem>,
}

impl Shape for RegistryRowsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let raw = shape::required(object, "raw", at, RegistryRowsInputRaw::decode_at)?;
        let entries = shape::required(object, "entries", at, |value, at| {
            shape::list(value, at, RegistryRowsInputEntriesItem::decode_at)
        })?;
        Ok(Self { raw, entries })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("raw", self.raw.encode());
        object.insert(
            "entries",
            Value::Array(self.entries.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryRowsInputRaw {
    pub records: Option<Value>,
    pub added: Option<Value>,
    pub hidden: Option<Value>,
}

impl Shape for RegistryRowsInputRaw {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let records = shape::optional(object, "records", at, shape::any)?;
        let added = shape::optional(object, "added", at, shape::any)?;
        let hidden = shape::optional(object, "hidden", at, shape::any)?;
        Ok(Self {
            records,
            added,
            hidden,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.records {
            object.insert("records", held.clone());
        }
        if let Some(held) = &self.added {
            object.insert("added", held.clone());
        }
        if let Some(held) = &self.hidden {
            object.insert("hidden", held.clone());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryRowsInputEntriesItem {
    pub id: String,
    pub kind: nyaaai_contract::shapes::provider::ProviderKindName,
}

impl Shape for RegistryRowsInputEntriesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let kind = shape::required(
            object,
            "kind",
            at,
            nyaaai_contract::shapes::provider::ProviderKindName::decode_at,
        )?;
        Ok(Self { id, kind })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("kind", self.kind.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryRowsResult {
    pub curated_first: bool,
    pub rest: Vec<nyaaai_contract::shapes::model::ModelRow>,
}

impl Shape for RegistryRowsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let curated_first = shape::required(object, "curatedFirst", at, shape::flag)?;
        let rest = shape::required(object, "rest", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::model::ModelRow::decode_at,
            )
        })?;
        Ok(Self {
            curated_first,
            rest,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("curatedFirst", Value::Bool(self.curated_first));
        object.insert(
            "rest",
            Value::Array(self.rest.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryOfferedInput {
    pub raw: RegistryOfferedInputRaw,
    pub entries: Vec<RegistryOfferedInputEntriesItem>,
    pub scope: Option<String>,
    pub keys: Vec<String>,
}

impl Shape for RegistryOfferedInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let raw = shape::required(object, "raw", at, RegistryOfferedInputRaw::decode_at)?;
        let entries = shape::required(object, "entries", at, |value, at| {
            shape::list(value, at, RegistryOfferedInputEntriesItem::decode_at)
        })?;
        let scope = shape::required(object, "scope", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let keys = shape::required(object, "keys", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            raw,
            entries,
            scope,
            keys,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("raw", self.raw.encode());
        object.insert(
            "entries",
            Value::Array(self.entries.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "scope",
            self.scope
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "keys",
            Value::Array(
                self.keys
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryOfferedInputRaw {
    pub records: Option<Value>,
    pub added: Option<Value>,
    pub hidden: Option<Value>,
}

impl Shape for RegistryOfferedInputRaw {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let records = shape::optional(object, "records", at, shape::any)?;
        let added = shape::optional(object, "added", at, shape::any)?;
        let hidden = shape::optional(object, "hidden", at, shape::any)?;
        Ok(Self {
            records,
            added,
            hidden,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.records {
            object.insert("records", held.clone());
        }
        if let Some(held) = &self.added {
            object.insert("added", held.clone());
        }
        if let Some(held) = &self.hidden {
            object.insert("hidden", held.clone());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryOfferedInputEntriesItem {
    pub id: String,
    pub kind: nyaaai_contract::shapes::provider::ProviderKindName,
}

impl Shape for RegistryOfferedInputEntriesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let kind = shape::required(
            object,
            "kind",
            at,
            nyaaai_contract::shapes::provider::ProviderKindName::decode_at,
        )?;
        Ok(Self { id, kind })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("kind", self.kind.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryOfferedResult {
    pub offered: Vec<Option<bool>>,
}

impl Shape for RegistryOfferedResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let offered = shape::required(object, "offered", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::flag)
            })
        })?;
        Ok(Self { offered })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "offered",
            Value::Array(
                self.offered
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| Value::Bool(*held)))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryListInput {
    pub raw: RegistryListInputRaw,
    pub entries: Vec<RegistryListInputEntriesItem>,
    pub scope: Option<String>,
    pub use_: nyaaai_contract::shapes::model::ModelUse,
    pub views: Vec<RegistryListInputViewsItem>,
}

impl Shape for RegistryListInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let raw = shape::required(object, "raw", at, RegistryListInputRaw::decode_at)?;
        let entries = shape::required(object, "entries", at, |value, at| {
            shape::list(value, at, RegistryListInputEntriesItem::decode_at)
        })?;
        let scope = shape::required(object, "scope", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let views = shape::required(object, "views", at, |value, at| {
            shape::list(value, at, RegistryListInputViewsItem::decode_at)
        })?;
        Ok(Self {
            raw,
            entries,
            scope,
            use_,
            views,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("raw", self.raw.encode());
        object.insert(
            "entries",
            Value::Array(self.entries.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "scope",
            self.scope
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert("use", self.use_.encode());
        object.insert(
            "views",
            Value::Array(self.views.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryListInputRaw {
    pub records: Option<Value>,
    pub added: Option<Value>,
    pub hidden: Option<Value>,
}

impl Shape for RegistryListInputRaw {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let records = shape::optional(object, "records", at, shape::any)?;
        let added = shape::optional(object, "added", at, shape::any)?;
        let hidden = shape::optional(object, "hidden", at, shape::any)?;
        Ok(Self {
            records,
            added,
            hidden,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.records {
            object.insert("records", held.clone());
        }
        if let Some(held) = &self.added {
            object.insert("added", held.clone());
        }
        if let Some(held) = &self.hidden {
            object.insert("hidden", held.clone());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryListInputEntriesItem {
    pub id: String,
    pub kind: nyaaai_contract::shapes::provider::ProviderKindName,
}

impl Shape for RegistryListInputEntriesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let kind = shape::required(
            object,
            "kind",
            at,
            nyaaai_contract::shapes::provider::ProviderKindName::decode_at,
        )?;
        Ok(Self { id, kind })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("kind", self.kind.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryListInputViewsItem {
    pub key: String,
    pub status: RegistryListInputViewsItemStatus,
}

impl Shape for RegistryListInputViewsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let key = shape::required(object, "key", at, shape::text)?;
        let status = shape::required(
            object,
            "status",
            at,
            RegistryListInputViewsItemStatus::decode_at,
        )?;
        Ok(Self { key, status })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("key", Value::from(self.key.as_str()));
        object.insert("status", self.status.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistryListInputViewsItemStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl RegistryListInputViewsItemStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for RegistryListInputViewsItemStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryListResult {
    pub keys: Vec<String>,
}

impl Shape for RegistryListResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let keys = shape::required(object, "keys", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { keys })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "keys",
            Value::Array(
                self.keys
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistrySelectInput {
    pub wanted: String,
    pub list: Vec<RegistrySelectInputListItem>,
}

impl Shape for RegistrySelectInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let wanted = shape::required(object, "wanted", at, shape::text)?;
        let list = shape::required(object, "list", at, |value, at| {
            shape::list(value, at, RegistrySelectInputListItem::decode_at)
        })?;
        Ok(Self { wanted, list })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("wanted", Value::from(self.wanted.as_str()));
        object.insert(
            "list",
            Value::Array(self.list.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistrySelectInputListItem {
    pub key: String,
    pub verified_completion: f64,
    pub price: Option<RegistrySelectInputListItemPrice>,
}

impl Shape for RegistrySelectInputListItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let key = shape::required(object, "key", at, shape::text)?;
        let verified_completion = shape::required(object, "verifiedCompletion", at, shape::number)?;
        let price = shape::optional(
            object,
            "price",
            at,
            RegistrySelectInputListItemPrice::decode_at,
        )?;
        Ok(Self {
            key,
            verified_completion,
            price,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("key", Value::from(self.key.as_str()));
        object.insert(
            "verifiedCompletion",
            Value::Number(self.verified_completion),
        );
        if let Some(held) = &self.price {
            object.insert("price", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegistrySelectInputListItemPrice {
    Price(nyaaai_contract::shapes::model::Price),
    Free,
    Unknown,
}

impl Shape for RegistrySelectInputListItemPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = nyaaai_contract::shapes::model::Price::decode_at(value, at) {
            return Ok(Self::Price(held));
        }
        if shape::is_text(value, at, "free").is_ok() {
            return Ok(Self::Free);
        }
        if shape::is_text(value, at, "unknown").is_ok() {
            return Ok(Self::Unknown);
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Price(held) => held.encode(),
            Self::Free => Value::from("free"),
            Self::Unknown => Value::from("unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistrySelectResult {
    pub id: Option<String>,
    pub marked: bool,
}

impl Shape for RegistrySelectResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let marked = shape::required(object, "marked", at, shape::flag)?;
        Ok(Self { id, marked })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "id",
            self.id
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert("marked", Value::Bool(self.marked));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryPassedInput {
    pub rows: Vec<RegistryPassedInputRowsItem>,
}

impl Shape for RegistryPassedInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, RegistryPassedInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryPassedInputRowsItem {
    pub tool_plus: RegistryPassedInputRowsItemToolPlus,
    pub recovery: RegistryPassedInputRowsItemRecovery,
    pub reasoning: RegistryPassedInputRowsItemReasoning,
    pub schema: RegistryPassedInputRowsItemSchema,
}

impl Shape for RegistryPassedInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tool_plus = shape::required(
            object,
            "toolPlus",
            at,
            RegistryPassedInputRowsItemToolPlus::decode_at,
        )?;
        let recovery = shape::required(
            object,
            "recovery",
            at,
            RegistryPassedInputRowsItemRecovery::decode_at,
        )?;
        let reasoning = shape::required(
            object,
            "reasoning",
            at,
            RegistryPassedInputRowsItemReasoning::decode_at,
        )?;
        let schema = shape::required(
            object,
            "schema",
            at,
            RegistryPassedInputRowsItemSchema::decode_at,
        )?;
        Ok(Self {
            tool_plus,
            recovery,
            reasoning,
            schema,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("toolPlus", self.tool_plus.encode());
        object.insert("recovery", self.recovery.encode());
        object.insert("reasoning", self.reasoning.encode());
        object.insert("schema", self.schema.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistryPassedInputRowsItemToolPlus {
    Pass,
    Fail,
    NA,
}

impl RegistryPassedInputRowsItemToolPlus {
    pub const ALL: [Self; 3] = [Self::Pass, Self::Fail, Self::NA];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::NA => "n/a",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for RegistryPassedInputRowsItemToolPlus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistryPassedInputRowsItemRecovery {
    Pass,
    Fail,
    NA,
}

impl RegistryPassedInputRowsItemRecovery {
    pub const ALL: [Self; 3] = [Self::Pass, Self::Fail, Self::NA];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::NA => "n/a",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for RegistryPassedInputRowsItemRecovery {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistryPassedInputRowsItemReasoning {
    Pass,
    Fail,
    NA,
}

impl RegistryPassedInputRowsItemReasoning {
    pub const ALL: [Self; 3] = [Self::Pass, Self::Fail, Self::NA];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::NA => "n/a",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for RegistryPassedInputRowsItemReasoning {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistryPassedInputRowsItemSchema {
    Pass,
    Fail,
    NA,
}

impl RegistryPassedInputRowsItemSchema {
    pub const ALL: [Self; 3] = [Self::Pass, Self::Fail, Self::NA];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::NA => "n/a",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for RegistryPassedInputRowsItemSchema {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryPassedResult {
    pub passed: Vec<bool>,
    pub gates: Vec<Option<String>>,
}

impl Shape for RegistryPassedResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let passed = shape::required(object, "passed", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        let gates = shape::required(object, "gates", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { passed, gates })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "passed",
            Value::Array(self.passed.iter().map(|held| Value::Bool(*held)).collect()),
        );
        object.insert(
            "gates",
            Value::Array(
                self.gates
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryIdsInput {
    pub model_ids: Vec<Value>,
    pub curated: Vec<RegistryIdsInputCuratedItem>,
    pub tested: Vec<RegistryIdsInputTestedItem>,
}

impl Shape for RegistryIdsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let model_ids = shape::required(object, "modelIds", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        let curated = shape::required(object, "curated", at, |value, at| {
            shape::list(value, at, RegistryIdsInputCuratedItem::decode_at)
        })?;
        let tested = shape::required(object, "tested", at, |value, at| {
            shape::list(value, at, RegistryIdsInputTestedItem::decode_at)
        })?;
        Ok(Self {
            model_ids,
            curated,
            tested,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("modelIds", Value::Array(self.model_ids.clone()));
        object.insert(
            "curated",
            Value::Array(self.curated.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "tested",
            Value::Array(self.tested.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryIdsInputCuratedItem {
    pub kind: nyaaai_contract::shapes::provider::ProviderKindName,
    pub id: String,
}

impl Shape for RegistryIdsInputCuratedItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(
            object,
            "kind",
            at,
            nyaaai_contract::shapes::provider::ProviderKindName::decode_at,
        )?;
        let id = shape::required(object, "id", at, shape::text)?;
        Ok(Self { kind, id })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("id", Value::from(self.id.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryIdsInputTestedItem {
    pub key: String,
    pub entry: String,
}

impl Shape for RegistryIdsInputTestedItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let key = shape::required(object, "key", at, shape::text)?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        Ok(Self { key, entry })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("key", Value::from(self.key.as_str()));
        object.insert("entry", Value::from(self.entry.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegistryIdsResult {
    pub model_ids: Vec<bool>,
    pub curated: Vec<bool>,
    pub tested: Vec<bool>,
}

impl Shape for RegistryIdsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let model_ids = shape::required(object, "modelIds", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        let curated = shape::required(object, "curated", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        let tested = shape::required(object, "tested", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        Ok(Self {
            model_ids,
            curated,
            tested,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "modelIds",
            Value::Array(
                self.model_ids
                    .iter()
                    .map(|held| Value::Bool(*held))
                    .collect(),
            ),
        );
        object.insert(
            "curated",
            Value::Array(self.curated.iter().map(|held| Value::Bool(*held)).collect()),
        );
        object.insert(
            "tested",
            Value::Array(self.tested.iter().map(|held| Value::Bool(*held)).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewUnknownInput {
    pub keys: Vec<String>,
}

impl Shape for ViewUnknownInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let keys = shape::required(object, "keys", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { keys })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "keys",
            Value::Array(
                self.keys
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewUnknownResult {
    pub views: Vec<nyaaai_contract::shapes::model::ModelView>,
}

impl Shape for ViewUnknownResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let views = shape::required(object, "views", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::model::ModelView::decode_at,
            )
        })?;
        Ok(Self { views })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "views",
            Value::Array(self.views.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewTiersInput {
    pub rows: Vec<ViewTiersInputRowsItem>,
}

impl Shape for ViewTiersInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, ViewTiersInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewTiersInputRowsItem {
    pub view: Option<ViewTiersInputRowsItemView>,
}

impl Shape for ViewTiersInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let view = shape::required(object, "view", at, |value, at| {
            shape::nullable(value, at, ViewTiersInputRowsItemView::decode_at)
        })?;
        Ok(Self { view })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "view",
            self.view.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewTiersInputRowsItemView {
    pub price: Option<ViewTiersInputRowsItemViewPrice>,
}

impl Shape for ViewTiersInputRowsItemView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let price = shape::optional(
            object,
            "price",
            at,
            ViewTiersInputRowsItemViewPrice::decode_at,
        )?;
        Ok(Self { price })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.price {
            object.insert("price", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewTiersInputRowsItemViewPrice {
    Price(nyaaai_contract::shapes::model::Price),
    Free,
    Unknown,
}

impl Shape for ViewTiersInputRowsItemViewPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = nyaaai_contract::shapes::model::Price::decode_at(value, at) {
            return Ok(Self::Price(held));
        }
        if shape::is_text(value, at, "free").is_ok() {
            return Ok(Self::Free);
        }
        if shape::is_text(value, at, "unknown").is_ok() {
            return Ok(Self::Unknown);
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Price(held) => held.encode(),
            Self::Free => Value::from("free"),
            Self::Unknown => Value::from("unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewTiersResult {
    pub tiers: Vec<Option<Vec<nyaaai_contract::shapes::model::PriceTier>>>,
}

impl Shape for ViewTiersResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tiers = shape::required(object, "tiers", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, |value, at| {
                    shape::list(
                        value,
                        at,
                        nyaaai_contract::shapes::model::PriceTier::decode_at,
                    )
                })
            })
        })?;
        Ok(Self { tiers })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "tiers",
            Value::Array(
                self.tiers
                    .iter()
                    .map(|held| {
                        held.as_ref().map_or(Value::Null, |held| {
                            Value::Array(held.iter().map(|held| held.encode()).collect())
                        })
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewCodecFactsInput {
    pub views: Vec<Option<ViewCodecFactsInputViewsItem>>,
}

impl Shape for ViewCodecFactsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let views = shape::required(object, "views", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, ViewCodecFactsInputViewsItem::decode_at)
            })
        })?;
        Ok(Self { views })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "views",
            Value::Array(
                self.views
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewCodecFactsInputViewsItem {
    pub reasoning: bool,
    pub status: ViewCodecFactsInputViewsItemStatus,
    pub efforts: Vec<nyaaai_contract::shapes::step::Effort>,
    pub output_limit: Option<f64>,
    pub price: Option<ViewCodecFactsInputViewsItemPrice>,
    pub api: Option<nyaaai_contract::shapes::step::ApiFacts>,
}

impl Shape for ViewCodecFactsInputViewsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let reasoning = shape::required(object, "reasoning", at, shape::flag)?;
        let status = shape::required(
            object,
            "status",
            at,
            ViewCodecFactsInputViewsItemStatus::decode_at,
        )?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
        })?;
        let output_limit = shape::optional(object, "outputLimit", at, shape::number)?;
        let price = shape::optional(
            object,
            "price",
            at,
            ViewCodecFactsInputViewsItemPrice::decode_at,
        )?;
        let api = shape::optional(
            object,
            "api",
            at,
            nyaaai_contract::shapes::step::ApiFacts::decode_at,
        )?;
        Ok(Self {
            reasoning,
            status,
            efforts,
            output_limit,
            price,
            api,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("reasoning", Value::Bool(self.reasoning));
        object.insert("status", self.status.encode());
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.output_limit {
            object.insert("outputLimit", Value::Number(*held));
        }
        if let Some(held) = &self.price {
            object.insert("price", held.encode());
        }
        if let Some(held) = &self.api {
            object.insert("api", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewCodecFactsInputViewsItemStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl ViewCodecFactsInputViewsItemStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ViewCodecFactsInputViewsItemStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewCodecFactsInputViewsItemPrice {
    Price(nyaaai_contract::shapes::model::Price),
    Free,
    Unknown,
}

impl Shape for ViewCodecFactsInputViewsItemPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = nyaaai_contract::shapes::model::Price::decode_at(value, at) {
            return Ok(Self::Price(held));
        }
        if shape::is_text(value, at, "free").is_ok() {
            return Ok(Self::Free);
        }
        if shape::is_text(value, at, "unknown").is_ok() {
            return Ok(Self::Unknown);
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Price(held) => held.encode(),
            Self::Free => Value::from("free"),
            Self::Unknown => Value::from("unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewCodecFactsResult {
    pub facts: Vec<Option<nyaaai_contract::shapes::step::CodecFacts>>,
}

impl Shape for ViewCodecFactsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let facts = shape::required(object, "facts", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(
                    value,
                    at,
                    nyaaai_contract::shapes::step::CodecFacts::decode_at,
                )
            })
        })?;
        Ok(Self { facts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "facts",
            Value::Array(
                self.facts
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewOutputLimitInput {
    pub texts: Vec<String>,
}

impl Shape for ViewOutputLimitInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewOutputLimitResult {
    pub limits: Vec<Option<i64>>,
}

impl Shape for ViewOutputLimitResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let limits = shape::required(object, "limits", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::integer)
            })
        })?;
        Ok(Self { limits })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "limits",
            Value::Array(
                self.limits
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held as f64))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewRunEffortInput {
    pub rows: Vec<ViewRunEffortInputRowsItem>,
}

impl Shape for ViewRunEffortInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, ViewRunEffortInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewRunEffortInputRowsItem {
    pub view: Option<ViewRunEffortInputRowsItemView>,
    pub wanted: Option<nyaaai_contract::shapes::step::Effort>,
}

impl Shape for ViewRunEffortInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let view = shape::required(object, "view", at, |value, at| {
            shape::nullable(value, at, ViewRunEffortInputRowsItemView::decode_at)
        })?;
        let wanted = shape::required(object, "wanted", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
        })?;
        Ok(Self { view, wanted })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "view",
            self.view.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "wanted",
            self.wanted
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewRunEffortInputRowsItemView {
    pub reasoning: bool,
    pub status: ViewRunEffortInputRowsItemViewStatus,
    pub efforts: Vec<nyaaai_contract::shapes::step::Effort>,
    pub output_limit: Option<f64>,
    pub price: Option<ViewRunEffortInputRowsItemViewPrice>,
    pub api: Option<nyaaai_contract::shapes::step::ApiFacts>,
}

impl Shape for ViewRunEffortInputRowsItemView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let reasoning = shape::required(object, "reasoning", at, shape::flag)?;
        let status = shape::required(
            object,
            "status",
            at,
            ViewRunEffortInputRowsItemViewStatus::decode_at,
        )?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
        })?;
        let output_limit = shape::optional(object, "outputLimit", at, shape::number)?;
        let price = shape::optional(
            object,
            "price",
            at,
            ViewRunEffortInputRowsItemViewPrice::decode_at,
        )?;
        let api = shape::optional(
            object,
            "api",
            at,
            nyaaai_contract::shapes::step::ApiFacts::decode_at,
        )?;
        Ok(Self {
            reasoning,
            status,
            efforts,
            output_limit,
            price,
            api,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("reasoning", Value::Bool(self.reasoning));
        object.insert("status", self.status.encode());
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.output_limit {
            object.insert("outputLimit", Value::Number(*held));
        }
        if let Some(held) = &self.price {
            object.insert("price", held.encode());
        }
        if let Some(held) = &self.api {
            object.insert("api", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewRunEffortInputRowsItemViewStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl ViewRunEffortInputRowsItemViewStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ViewRunEffortInputRowsItemViewStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewRunEffortInputRowsItemViewPrice {
    Price(nyaaai_contract::shapes::model::Price),
    Free,
    Unknown,
}

impl Shape for ViewRunEffortInputRowsItemViewPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = nyaaai_contract::shapes::model::Price::decode_at(value, at) {
            return Ok(Self::Price(held));
        }
        if shape::is_text(value, at, "free").is_ok() {
            return Ok(Self::Free);
        }
        if shape::is_text(value, at, "unknown").is_ok() {
            return Ok(Self::Unknown);
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Price(held) => held.encode(),
            Self::Free => Value::from("free"),
            Self::Unknown => Value::from("unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewRunEffortResult {
    pub efforts: Vec<Option<nyaaai_contract::shapes::step::Effort>>,
}

impl Shape for ViewRunEffortResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
            })
        })?;
        Ok(Self { efforts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "efforts",
            Value::Array(
                self.efforts
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOfInput {
    pub rows: Vec<RetentionOfInputRowsItem>,
}

impl Shape for RetentionOfInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, RetentionOfInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOfInputRowsItem {
    pub plan: nyaaai_contract::shapes::provider::ProviderPlan,
    pub id: String,
}

impl Shape for RetentionOfInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let plan = shape::required(
            object,
            "plan",
            at,
            nyaaai_contract::shapes::provider::ProviderPlan::decode_at,
        )?;
        let id = shape::required(object, "id", at, shape::text)?;
        Ok(Self { plan, id })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("plan", self.plan.encode());
        object.insert("id", Value::from(self.id.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOfResult {
    pub rows: Vec<Option<RetentionOfResultRowsItem>>,
}

impl Shape for RetentionOfResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, RetentionOfResultRowsItem::decode_at)
            })
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionOfResultRowsItem {
    pub zdr: bool,
    pub until: Option<String>,
}

impl Shape for RetentionOfResultRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let zdr = shape::required(object, "zdr", at, shape::flag)?;
        let until = shape::optional(object, "until", at, shape::text)?;
        Ok(Self { zdr, until })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("zdr", Value::Bool(self.zdr));
        if let Some(held) = &self.until {
            object.insert("until", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionIsZdrInput {
    pub rows: Vec<RetentionIsZdrInputRowsItem>,
}

impl Shape for RetentionIsZdrInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, RetentionIsZdrInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionIsZdrInputRowsItem {
    pub plan: nyaaai_contract::shapes::provider::ProviderPlan,
    pub id: String,
    pub now: i64,
}

impl Shape for RetentionIsZdrInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let plan = shape::required(
            object,
            "plan",
            at,
            nyaaai_contract::shapes::provider::ProviderPlan::decode_at,
        )?;
        let id = shape::required(object, "id", at, shape::text)?;
        let now = shape::required(object, "now", at, shape::integer)?;
        Ok(Self { plan, id, now })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("plan", self.plan.encode());
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("now", Value::Number(self.now as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionIsZdrResult {
    pub values: Vec<bool>,
}

impl Shape for RetentionIsZdrResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(self.values.iter().map(|held| Value::Bool(*held)).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispositionOfInput {
    pub rows: Vec<DispositionOfInputRowsItem>,
}

impl Shape for DispositionOfInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, DispositionOfInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispositionOfInputRowsItem {
    pub failure: nyaaai_contract::shapes::step::Failure,
    pub status: Option<DispositionOfInputRowsItemStatus>,
}

impl Shape for DispositionOfInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let failure = shape::required(
            object,
            "failure",
            at,
            nyaaai_contract::shapes::step::Failure::decode_at,
        )?;
        let status = shape::required(object, "status", at, |value, at| {
            shape::nullable(value, at, DispositionOfInputRowsItemStatus::decode_at)
        })?;
        Ok(Self { failure, status })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("failure", self.failure.encode());
        object.insert(
            "status",
            self.status
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DispositionOfInputRowsItemStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl DispositionOfInputRowsItemStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for DispositionOfInputRowsItemStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispositionOfResult {
    pub dispositions: Vec<nyaaai_contract::shapes::provider::Disposition>,
}

impl Shape for DispositionOfResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let dispositions = shape::required(object, "dispositions", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::Disposition::decode_at,
            )
        })?;
        Ok(Self { dispositions })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "dispositions",
            Value::Array(self.dispositions.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispositionSkipReasonInput {
    pub failures: Vec<nyaaai_contract::shapes::step::Failure>,
}

impl Shape for DispositionSkipReasonInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let failures = shape::required(object, "failures", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Failure::decode_at)
        })?;
        Ok(Self { failures })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "failures",
            Value::Array(self.failures.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispositionSkipReasonResult {
    pub reasons: Vec<nyaaai_contract::shapes::provider::SkipReason>,
    pub all: Vec<nyaaai_contract::shapes::provider::SkipReason>,
}

impl Shape for DispositionSkipReasonResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let reasons = shape::required(object, "reasons", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::SkipReason::decode_at,
            )
        })?;
        let all = shape::required(object, "all", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::SkipReason::decode_at,
            )
        })?;
        Ok(Self { reasons, all })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "reasons",
            Value::Array(self.reasons.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "all",
            Value::Array(self.all.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetExchange {
    pub status: i64,
    pub json: Option<Value>,
    pub text: Option<String>,
    pub head_ms: Option<i64>,
    pub end: Option<NetExchangeEnd>,
}

impl Shape for NetExchange {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let status = shape::required(object, "status", at, shape::integer)?;
        let json = shape::optional(object, "json", at, shape::any)?;
        let text = shape::optional(object, "text", at, shape::text)?;
        let head_ms = shape::optional(object, "headMs", at, shape::integer)?;
        let end = shape::optional(object, "end", at, NetExchangeEnd::decode_at)?;
        Ok(Self {
            status,
            json,
            text,
            head_ms,
            end,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("status", Value::Number(self.status as f64));
        if let Some(held) = &self.json {
            object.insert("json", held.clone());
        }
        if let Some(held) = &self.text {
            object.insert("text", Value::from(held.as_str()));
        }
        if let Some(held) = &self.head_ms {
            object.insert("headMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.end {
            object.insert("end", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetExchangeEnd {
    Closed,
    Dropped,
    Refused,
}

impl NetExchangeEnd {
    pub const ALL: [Self; 3] = [Self::Closed, Self::Dropped, Self::Refused];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Dropped => "dropped",
            Self::Refused => "refused",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for NetExchangeEnd {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewLine {
    pub key: String,
    pub status: ViewLineStatus,
    pub retired_on: Option<String>,
    pub efforts: Vec<nyaaai_contract::shapes::step::Effort>,
    pub prompt_limit: Option<f64>,
    pub output_limit: Option<f64>,
    pub price: Option<ViewLinePrice>,
    pub cache_write1h: Option<()>,
    pub hosts: Vec<String>,
    pub data_kept: Option<()>,
    pub api: Option<nyaaai_contract::shapes::step::ApiFacts>,
    pub cache: Option<ViewLineCache>,
}

impl Shape for ViewLine {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let key = shape::required(object, "key", at, shape::text)?;
        let status = shape::required(object, "status", at, ViewLineStatus::decode_at)?;
        let retired_on = shape::optional(object, "retiredOn", at, shape::text)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
        })?;
        let prompt_limit = shape::optional(object, "promptLimit", at, shape::number)?;
        let output_limit = shape::optional(object, "outputLimit", at, shape::number)?;
        let price = shape::optional(object, "price", at, ViewLinePrice::decode_at)?;
        let cache_write1h = shape::optional(object, "cacheWrite1h", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        let hosts = shape::required(object, "hosts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let data_kept = shape::optional(object, "dataKept", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        let api = shape::optional(
            object,
            "api",
            at,
            nyaaai_contract::shapes::step::ApiFacts::decode_at,
        )?;
        let cache = shape::optional(object, "cache", at, ViewLineCache::decode_at)?;
        Ok(Self {
            key,
            status,
            retired_on,
            efforts,
            prompt_limit,
            output_limit,
            price,
            cache_write1h,
            hosts,
            data_kept,
            api,
            cache,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("key", Value::from(self.key.as_str()));
        object.insert("status", self.status.encode());
        if let Some(held) = &self.retired_on {
            object.insert("retiredOn", Value::from(held.as_str()));
        }
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.prompt_limit {
            object.insert("promptLimit", Value::Number(*held));
        }
        if let Some(held) = &self.output_limit {
            object.insert("outputLimit", Value::Number(*held));
        }
        if let Some(held) = &self.price {
            object.insert("price", held.encode());
        }
        if self.cache_write1h.is_some() {
            object.insert("cacheWrite1h", Value::Bool(true));
        }
        object.insert(
            "hosts",
            Value::Array(
                self.hosts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        if self.data_kept.is_some() {
            object.insert("dataKept", Value::Bool(true));
        }
        if let Some(held) = &self.api {
            object.insert("api", held.encode());
        }
        if let Some(held) = &self.cache {
            object.insert("cache", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewLineStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl ViewLineStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ViewLineStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewLinePrice {
    Price(nyaaai_contract::shapes::model::Price),
    Free,
    Unknown,
}

impl Shape for ViewLinePrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = nyaaai_contract::shapes::model::Price::decode_at(value, at) {
            return Ok(Self::Price(held));
        }
        if shape::is_text(value, at, "free").is_ok() {
            return Ok(Self::Free);
        }
        if shape::is_text(value, at, "unknown").is_ok() {
            return Ok(Self::Unknown);
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Price(held) => held.encode(),
            Self::Free => Value::from("free"),
            Self::Unknown => Value::from("unknown"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewLineCache {
    Implicit,
    Explicit,
}

impl ViewLineCache {
    pub const ALL: [Self; 2] = [Self::Implicit, Self::Explicit];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Implicit => "implicit",
            Self::Explicit => "explicit",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ViewLineCache {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterModelsInput {
    pub body: Value,
}

impl Shape for CatalogOpenrouterModelsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterModelsResult {
    pub models: Option<Vec<CatalogOpenrouterModelsResultModelsItem>>,
}

impl Shape for CatalogOpenrouterModelsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(
                    value,
                    at,
                    CatalogOpenrouterModelsResultModelsItem::decode_at,
                )
            })
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterModelsResultModelsItem {
    pub id: String,
    pub name: Option<String>,
    pub created: Option<f64>,
    pub context_length: Option<f64>,
    pub expiration_date: Option<String>,
    pub pricing: Option<CatalogOpenrouterModelsResultModelsItemPricing>,
    pub reasoning: Option<CatalogOpenrouterModelsResultModelsItemReasoning>,
    pub supported_parameters: Option<Vec<String>>,
}

impl Shape for CatalogOpenrouterModelsResultModelsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let name = shape::required(object, "name", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let created = shape::required(object, "created", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let context_length = shape::required(object, "context_length", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let expiration_date = shape::required(object, "expiration_date", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let pricing = shape::required(object, "pricing", at, |value, at| {
            shape::nullable(
                value,
                at,
                CatalogOpenrouterModelsResultModelsItemPricing::decode_at,
            )
        })?;
        let reasoning = shape::required(object, "reasoning", at, |value, at| {
            shape::nullable(
                value,
                at,
                CatalogOpenrouterModelsResultModelsItemReasoning::decode_at,
            )
        })?;
        let supported_parameters =
            shape::required(object, "supported_parameters", at, |value, at| {
                shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
            })?;
        Ok(Self {
            id,
            name,
            created,
            context_length,
            expiration_date,
            pricing,
            reasoning,
            supported_parameters,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert(
            "name",
            self.name
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "created",
            self.created
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert(
            "context_length",
            self.context_length
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert(
            "expiration_date",
            self.expiration_date
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "pricing",
            self.pricing
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "reasoning",
            self.reasoning
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "supported_parameters",
            self.supported_parameters
                .as_ref()
                .map_or(Value::Null, |held| {
                    Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
                }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterModelsResultModelsItemPricing {
    pub prompt: Value,
    pub completion: Value,
    pub input_cache_read: Value,
    pub input_cache_write: Value,
    pub input_cache_write_1h: Value,
    pub overrides: Option<Vec<CatalogOpenrouterModelsResultModelsItemPricingOverridesItem>>,
}

impl Shape for CatalogOpenrouterModelsResultModelsItemPricing {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let prompt = shape::required(object, "prompt", at, shape::any)?;
        let completion = shape::required(object, "completion", at, shape::any)?;
        let input_cache_read = shape::required(object, "input_cache_read", at, shape::any)?;
        let input_cache_write = shape::required(object, "input_cache_write", at, shape::any)?;
        let input_cache_write_1h = shape::required(object, "input_cache_write_1h", at, shape::any)?;
        let overrides = shape::required(object, "overrides", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(
                    value,
                    at,
                    CatalogOpenrouterModelsResultModelsItemPricingOverridesItem::decode_at,
                )
            })
        })?;
        Ok(Self {
            prompt,
            completion,
            input_cache_read,
            input_cache_write,
            input_cache_write_1h,
            overrides,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("prompt", self.prompt.clone());
        object.insert("completion", self.completion.clone());
        object.insert("input_cache_read", self.input_cache_read.clone());
        object.insert("input_cache_write", self.input_cache_write.clone());
        object.insert("input_cache_write_1h", self.input_cache_write_1h.clone());
        object.insert(
            "overrides",
            self.overrides.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterModelsResultModelsItemReasoning {
    pub supported_efforts: Option<Vec<String>>,
}

impl Shape for CatalogOpenrouterModelsResultModelsItemReasoning {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let supported_efforts = shape::required(object, "supported_efforts", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        Ok(Self { supported_efforts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "supported_efforts",
            self.supported_efforts.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterModelsResultModelsItemPricingOverridesItem {
    pub min_prompt_tokens: f64,
    pub prompt: Value,
    pub completion: Value,
    pub input_cache_read: Value,
    pub input_cache_write: Value,
    pub input_cache_write_1h: Value,
}

impl Shape for CatalogOpenrouterModelsResultModelsItemPricingOverridesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let min_prompt_tokens = shape::required(object, "min_prompt_tokens", at, shape::number)?;
        let prompt = shape::required(object, "prompt", at, shape::any)?;
        let completion = shape::required(object, "completion", at, shape::any)?;
        let input_cache_read = shape::required(object, "input_cache_read", at, shape::any)?;
        let input_cache_write = shape::required(object, "input_cache_write", at, shape::any)?;
        let input_cache_write_1h = shape::required(object, "input_cache_write_1h", at, shape::any)?;
        Ok(Self {
            min_prompt_tokens,
            prompt,
            completion,
            input_cache_read,
            input_cache_write,
            input_cache_write_1h,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("min_prompt_tokens", Value::Number(self.min_prompt_tokens));
        object.insert("prompt", self.prompt.clone());
        object.insert("completion", self.completion.clone());
        object.insert("input_cache_read", self.input_cache_read.clone());
        object.insert("input_cache_write", self.input_cache_write.clone());
        object.insert("input_cache_write_1h", self.input_cache_write_1h.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterEndpointsInput {
    pub body: Value,
}

impl Shape for CatalogOpenrouterEndpointsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterEndpointsResult {
    pub endpoints: Option<Vec<CatalogOpenrouterEndpointsResultEndpointsItem>>,
}

impl Shape for CatalogOpenrouterEndpointsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let endpoints = shape::required(object, "endpoints", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(
                    value,
                    at,
                    CatalogOpenrouterEndpointsResultEndpointsItem::decode_at,
                )
            })
        })?;
        Ok(Self { endpoints })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "endpoints",
            self.endpoints.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterEndpointsResultEndpointsItem {
    pub model: String,
    pub hosts: Vec<CatalogOpenrouterEndpointsResultEndpointsItemHostsItem>,
}

impl Shape for CatalogOpenrouterEndpointsResultEndpointsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let model = shape::required(object, "model", at, shape::text)?;
        let hosts = shape::required(object, "hosts", at, |value, at| {
            shape::list(
                value,
                at,
                CatalogOpenrouterEndpointsResultEndpointsItemHostsItem::decode_at,
            )
        })?;
        Ok(Self { model, hosts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("model", Value::from(self.model.as_str()));
        object.insert(
            "hosts",
            Value::Array(self.hosts.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterEndpointsResultEndpointsItemHostsItem {
    pub model_id: String,
    pub tag: Option<String>,
    pub provider_name: Option<String>,
    pub context_length: Option<f64>,
    pub max_prompt_tokens: Option<f64>,
    pub supported_parameters: Option<Vec<String>>,
    pub pricing: Option<CatalogOpenrouterEndpointsResultEndpointsItemHostsItemPricing>,
}

impl Shape for CatalogOpenrouterEndpointsResultEndpointsItemHostsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let model_id = shape::required(object, "model_id", at, shape::text)?;
        let tag = shape::required(object, "tag", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let provider_name = shape::required(object, "provider_name", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let context_length = shape::required(object, "context_length", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let max_prompt_tokens = shape::required(object, "max_prompt_tokens", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let supported_parameters =
            shape::required(object, "supported_parameters", at, |value, at| {
                shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
            })?;
        let pricing = shape::required(object, "pricing", at, |value, at| {
            shape::nullable(
                value,
                at,
                CatalogOpenrouterEndpointsResultEndpointsItemHostsItemPricing::decode_at,
            )
        })?;
        Ok(Self {
            model_id,
            tag,
            provider_name,
            context_length,
            max_prompt_tokens,
            supported_parameters,
            pricing,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("model_id", Value::from(self.model_id.as_str()));
        object.insert(
            "tag",
            self.tag
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "provider_name",
            self.provider_name
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "context_length",
            self.context_length
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert(
            "max_prompt_tokens",
            self.max_prompt_tokens
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert(
            "supported_parameters",
            self.supported_parameters
                .as_ref()
                .map_or(Value::Null, |held| {
                    Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
                }),
        );
        object.insert(
            "pricing",
            self.pricing
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterEndpointsResultEndpointsItemHostsItemPricing {
    pub prompt: Value,
    pub completion: Value,
    pub input_cache_read: Value,
    pub input_cache_write: Value,
    pub input_cache_write_1h: Value,
    pub overrides:
        Option<Vec<CatalogOpenrouterEndpointsResultEndpointsItemHostsItemPricingOverridesItem>>,
}

impl Shape for CatalogOpenrouterEndpointsResultEndpointsItemHostsItemPricing {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let prompt = shape::required(object, "prompt", at, shape::any)?;
        let completion = shape::required(object, "completion", at, shape::any)?;
        let input_cache_read = shape::required(object, "input_cache_read", at, shape::any)?;
        let input_cache_write = shape::required(object, "input_cache_write", at, shape::any)?;
        let input_cache_write_1h = shape::required(object, "input_cache_write_1h", at, shape::any)?;
        let overrides = shape::required(object, "overrides", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(value, at, CatalogOpenrouterEndpointsResultEndpointsItemHostsItemPricingOverridesItem::decode_at)
            })
        })?;
        Ok(Self {
            prompt,
            completion,
            input_cache_read,
            input_cache_write,
            input_cache_write_1h,
            overrides,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("prompt", self.prompt.clone());
        object.insert("completion", self.completion.clone());
        object.insert("input_cache_read", self.input_cache_read.clone());
        object.insert("input_cache_write", self.input_cache_write.clone());
        object.insert("input_cache_write_1h", self.input_cache_write_1h.clone());
        object.insert(
            "overrides",
            self.overrides.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterEndpointsResultEndpointsItemHostsItemPricingOverridesItem {
    pub min_prompt_tokens: f64,
    pub prompt: Value,
    pub completion: Value,
    pub input_cache_read: Value,
    pub input_cache_write: Value,
    pub input_cache_write_1h: Value,
}

impl Shape for CatalogOpenrouterEndpointsResultEndpointsItemHostsItemPricingOverridesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let min_prompt_tokens = shape::required(object, "min_prompt_tokens", at, shape::number)?;
        let prompt = shape::required(object, "prompt", at, shape::any)?;
        let completion = shape::required(object, "completion", at, shape::any)?;
        let input_cache_read = shape::required(object, "input_cache_read", at, shape::any)?;
        let input_cache_write = shape::required(object, "input_cache_write", at, shape::any)?;
        let input_cache_write_1h = shape::required(object, "input_cache_write_1h", at, shape::any)?;
        Ok(Self {
            min_prompt_tokens,
            prompt,
            completion,
            input_cache_read,
            input_cache_write,
            input_cache_write_1h,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("min_prompt_tokens", Value::Number(self.min_prompt_tokens));
        object.insert("prompt", self.prompt.clone());
        object.insert("completion", self.completion.clone());
        object.insert("input_cache_read", self.input_cache_read.clone());
        object.insert("input_cache_write", self.input_cache_write.clone());
        object.insert("input_cache_write_1h", self.input_cache_write_1h.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeModelsInput {
    pub body: Value,
}

impl Shape for CatalogOpencodeModelsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeModelsResult {
    pub models: Option<Vec<CatalogOpencodeModelsResultModelsItem>>,
}

impl Shape for CatalogOpencodeModelsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(value, at, CatalogOpencodeModelsResultModelsItem::decode_at)
            })
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeModelsResultModelsItem {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub input: Option<Vec<String>>,
    pub output: Option<Vec<String>>,
    pub tools: Option<bool>,
    pub reasoning: Option<bool>,
    pub efforts: Option<Vec<String>>,
    pub toggle: Option<bool>,
    pub structured_output: Option<bool>,
    pub cache: Option<CatalogOpencodeModelsResultModelsItemCache>,
    pub context: Option<f64>,
    pub input_limit: Option<f64>,
    pub output_limit: Option<f64>,
    pub price: Option<CatalogOpencodeModelsResultModelsItemPrice>,
    pub api: Vec<CatalogOpencodeModelsResultModelsItemApiItem>,
}

impl Shape for CatalogOpencodeModelsResultModelsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let name = shape::required(object, "name", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let description = shape::required(object, "description", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let input = shape::required(object, "input", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        let output = shape::required(object, "output", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        let tools = shape::required(object, "tools", at, |value, at| {
            shape::nullable(value, at, shape::flag)
        })?;
        let reasoning = shape::required(object, "reasoning", at, |value, at| {
            shape::nullable(value, at, shape::flag)
        })?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        let toggle = shape::required(object, "toggle", at, |value, at| {
            shape::nullable(value, at, shape::flag)
        })?;
        let structured_output = shape::required(object, "structured_output", at, |value, at| {
            shape::nullable(value, at, shape::flag)
        })?;
        let cache = shape::required(object, "cache", at, |value, at| {
            shape::nullable(
                value,
                at,
                CatalogOpencodeModelsResultModelsItemCache::decode_at,
            )
        })?;
        let context = shape::required(object, "context", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let input_limit = shape::required(object, "input_limit", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let output_limit = shape::required(object, "output_limit", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let price = shape::required(object, "price", at, |value, at| {
            shape::nullable(
                value,
                at,
                CatalogOpencodeModelsResultModelsItemPrice::decode_at,
            )
        })?;
        let api = shape::required(object, "api", at, |value, at| {
            shape::list(
                value,
                at,
                CatalogOpencodeModelsResultModelsItemApiItem::decode_at,
            )
        })?;
        Ok(Self {
            id,
            name,
            description,
            input,
            output,
            tools,
            reasoning,
            efforts,
            toggle,
            structured_output,
            cache,
            context,
            input_limit,
            output_limit,
            price,
            api,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert(
            "name",
            self.name
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "description",
            self.description
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "input",
            self.input.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }),
        );
        object.insert(
            "output",
            self.output.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }),
        );
        object.insert(
            "tools",
            self.tools
                .as_ref()
                .map_or(Value::Null, |held| Value::Bool(*held)),
        );
        object.insert(
            "reasoning",
            self.reasoning
                .as_ref()
                .map_or(Value::Null, |held| Value::Bool(*held)),
        );
        object.insert(
            "efforts",
            self.efforts.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }),
        );
        object.insert(
            "toggle",
            self.toggle
                .as_ref()
                .map_or(Value::Null, |held| Value::Bool(*held)),
        );
        object.insert(
            "structured_output",
            self.structured_output
                .as_ref()
                .map_or(Value::Null, |held| Value::Bool(*held)),
        );
        object.insert(
            "cache",
            self.cache
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "context",
            self.context
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert(
            "input_limit",
            self.input_limit
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert(
            "output_limit",
            self.output_limit
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert(
            "price",
            self.price
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "api",
            Value::Array(self.api.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeModelsResultModelsItemCache {
    pub implicit: Option<bool>,
    pub explicit: bool,
}

impl Shape for CatalogOpencodeModelsResultModelsItemCache {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let implicit = shape::required(object, "implicit", at, |value, at| {
            shape::nullable(value, at, shape::flag)
        })?;
        let explicit = shape::required(object, "explicit", at, shape::flag)?;
        Ok(Self { implicit, explicit })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "implicit",
            self.implicit
                .as_ref()
                .map_or(Value::Null, |held| Value::Bool(*held)),
        );
        object.insert("explicit", Value::Bool(self.explicit));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeModelsResultModelsItemPrice {
    pub input: f64,
    pub output: f64,
}

impl Shape for CatalogOpencodeModelsResultModelsItemPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let input = shape::required(object, "input", at, shape::number)?;
        let output = shape::required(object, "output", at, shape::number)?;
        Ok(Self { input, output })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("input", Value::Number(self.input));
        object.insert("output", Value::Number(self.output));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeModelsResultModelsItemApiItem {
    pub family: nyaaai_contract::shapes::step::ApiFamily,
    pub reasoning_field: Option<String>,
    pub choice: Option<Vec<String>>,
    pub strict: Option<bool>,
}

impl Shape for CatalogOpencodeModelsResultModelsItemApiItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let family = shape::required(
            object,
            "family",
            at,
            nyaaai_contract::shapes::step::ApiFamily::decode_at,
        )?;
        let reasoning_field = shape::required(object, "reasoning_field", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let choice = shape::required(object, "choice", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        let strict = shape::required(object, "strict", at, |value, at| {
            shape::nullable(value, at, shape::flag)
        })?;
        Ok(Self {
            family,
            reasoning_field,
            choice,
            strict,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("family", self.family.encode());
        object.insert(
            "reasoning_field",
            self.reasoning_field
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "choice",
            self.choice.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }),
        );
        object.insert(
            "strict",
            self.strict
                .as_ref()
                .map_or(Value::Null, |held| Value::Bool(*held)),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterJoinInput {
    pub rows: Vec<String>,
    pub models: Value,
    pub zdr: Value,
    pub now_ms: i64,
    pub passes: Vec<CatalogOpenrouterJoinInputPassesItem>,
}

impl Shape for CatalogOpenrouterJoinInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let models = shape::required(object, "models", at, shape::any)?;
        let zdr = shape::required(object, "zdr", at, shape::any)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let passes = shape::required(object, "passes", at, |value, at| {
            shape::list(value, at, CatalogOpenrouterJoinInputPassesItem::decode_at)
        })?;
        Ok(Self {
            rows,
            models,
            zdr,
            now_ms,
            passes,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("models", self.models.clone());
        object.insert("zdr", self.zdr.clone());
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert(
            "passes",
            Value::Array(self.passes.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterJoinInputPassesItem {
    pub use_: nyaaai_contract::shapes::model::ModelUse,
    pub zdr_only: bool,
}

impl Shape for CatalogOpenrouterJoinInputPassesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::required(object, "zdrOnly", at, shape::flag)?;
        Ok(Self { use_, zdr_only })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("use", self.use_.encode());
        object.insert("zdrOnly", Value::Bool(self.zdr_only));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterJoinResult {
    pub passes: Vec<Vec<ViewLine>>,
}

impl Shape for CatalogOpenrouterJoinResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let passes = shape::required(object, "passes", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, ViewLine::decode_at)
            })
        })?;
        Ok(Self { passes })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "passes",
            Value::Array(
                self.passes
                    .iter()
                    .map(|held| Value::Array(held.iter().map(|held| held.encode()).collect()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeJoinInput {
    pub rows: Vec<String>,
    pub plan: nyaaai_contract::shapes::provider::ProviderPlan,
    pub models: Value,
    pub now_ms: i64,
    pub passes: Vec<CatalogOpencodeJoinInputPassesItem>,
}

impl Shape for CatalogOpencodeJoinInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let plan = shape::required(
            object,
            "plan",
            at,
            nyaaai_contract::shapes::provider::ProviderPlan::decode_at,
        )?;
        let models = shape::required(object, "models", at, shape::any)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let passes = shape::required(object, "passes", at, |value, at| {
            shape::list(value, at, CatalogOpencodeJoinInputPassesItem::decode_at)
        })?;
        Ok(Self {
            rows,
            plan,
            models,
            now_ms,
            passes,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("plan", self.plan.encode());
        object.insert("models", self.models.clone());
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert(
            "passes",
            Value::Array(self.passes.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeJoinInputPassesItem {
    pub use_: nyaaai_contract::shapes::model::ModelUse,
    pub zdr_only: bool,
}

impl Shape for CatalogOpencodeJoinInputPassesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::required(object, "zdrOnly", at, shape::flag)?;
        Ok(Self { use_, zdr_only })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("use", self.use_.encode());
        object.insert("zdrOnly", Value::Bool(self.zdr_only));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeJoinResult {
    pub passes: Vec<Vec<ViewLine>>,
}

impl Shape for CatalogOpencodeJoinResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let passes = shape::required(object, "passes", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::list(value, at, ViewLine::decode_at)
            })
        })?;
        Ok(Self { passes })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "passes",
            Value::Array(
                self.passes
                    .iter()
                    .map(|held| Value::Array(held.iter().map(|held| held.encode()).collect()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogPerMillionInput {
    pub values: Vec<Value>,
}

impl Shape for CatalogPerMillionInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("values", Value::Array(self.values.clone()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogPerMillionResult {
    pub prices: Vec<Option<f64>>,
}

impl Shape for CatalogPerMillionResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let prices = shape::required(object, "prices", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::number)
            })
        })?;
        Ok(Self { prices })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "prices",
            Value::Array(
                self.prices
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogExpiryInput {
    pub texts: Vec<String>,
}

impl Shape for CatalogExpiryInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogExpiryResult {
    pub ms: Vec<Option<i64>>,
}

impl Shape for CatalogExpiryResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let ms = shape::required(object, "ms", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::integer)
            })
        })?;
        Ok(Self { ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "ms",
            Value::Array(
                self.ms
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held as f64))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogStandingInput {
    pub rows: Vec<CatalogStandingInputRowsItem>,
}

impl Shape for CatalogStandingInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, CatalogStandingInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogStandingInputRowsItem {
    pub expiration: Option<String>,
    pub hosts: Vec<CatalogStandingInputRowsItemHostsItem>,
    pub needs: Vec<CatalogStandingInputRowsItemNeedsItem>,
    pub now_ms: i64,
}

impl Shape for CatalogStandingInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let expiration = shape::required(object, "expiration", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let hosts = shape::required(object, "hosts", at, |value, at| {
            shape::list(value, at, CatalogStandingInputRowsItemHostsItem::decode_at)
        })?;
        let needs = shape::required(object, "needs", at, |value, at| {
            shape::list(value, at, CatalogStandingInputRowsItemNeedsItem::decode_at)
        })?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        Ok(Self {
            expiration,
            hosts,
            needs,
            now_ms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "expiration",
            self.expiration
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "hosts",
            Value::Array(self.hosts.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "needs",
            Value::Array(self.needs.iter().map(|held| held.encode()).collect()),
        );
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogStandingInputRowsItemHostsItem {
    pub params: Option<Vec<String>>,
}

impl Shape for CatalogStandingInputRowsItemHostsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let params = shape::required(object, "params", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        Ok(Self { params })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "params",
            self.params.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CatalogStandingInputRowsItemNeedsItem {
    Text(String),
    List(Vec<String>),
}

impl Shape for CatalogStandingInputRowsItemNeedsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = shape::text(value, at) {
            return Ok(Self::Text(held));
        }
        if let Ok(held) = shape::list(value, at, shape::text) {
            return Ok(Self::List(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Text(held) => Value::from(held.as_str()),
            Self::List(held) => {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogStandingResult {
    pub rows: Vec<CatalogStandingResultRowsItem>,
}

impl Shape for CatalogStandingResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, CatalogStandingResultRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogStandingResultRowsItem {
    pub standing: CatalogStandingResultRowsItemStanding,
    pub usable: Vec<i64>,
}

impl Shape for CatalogStandingResultRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let standing = shape::required(
            object,
            "standing",
            at,
            CatalogStandingResultRowsItemStanding::decode_at,
        )?;
        let usable = shape::required(object, "usable", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        Ok(Self { standing, usable })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("standing", self.standing.encode());
        object.insert(
            "usable",
            Value::Array(
                self.usable
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogStandingResultRowsItemStanding {
    Ok,
    Retired,
    NoZdrHost,
}

impl CatalogStandingResultRowsItemStanding {
    pub const ALL: [Self; 3] = [Self::Ok, Self::Retired, Self::NoZdrHost];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for CatalogStandingResultRowsItemStanding {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInput {
    pub start_ms: i64,
    pub exchanges: Vec<NetExchange>,
    pub rows: Vec<String>,
    pub ttl_ms: Option<i64>,
    pub on_refreshed: Option<CatalogOpenrouterRunInputOnRefreshed>,
    pub steps: Vec<CatalogOpenrouterRunInputStepsItem>,
}

impl Shape for CatalogOpenrouterRunInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let ttl_ms = shape::optional(object, "ttlMs", at, shape::integer)?;
        let on_refreshed = shape::optional(
            object,
            "onRefreshed",
            at,
            CatalogOpenrouterRunInputOnRefreshed::decode_at,
        )?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list_within(
                value,
                at,
                Some(1),
                None,
                CatalogOpenrouterRunInputStepsItem::decode_at,
            )
        })?;
        Ok(Self {
            start_ms,
            exchanges,
            rows,
            ttl_ms,
            on_refreshed,
            steps,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        if let Some(held) = &self.ttl_ms {
            object.insert("ttlMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.on_refreshed {
            object.insert("onRefreshed", held.encode());
        }
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogOpenrouterRunInputOnRefreshed {
    Reads,
    ReadsAndThrows,
}

impl CatalogOpenrouterRunInputOnRefreshed {
    pub const ALL: [Self; 2] = [Self::Reads, Self::ReadsAndThrows];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reads => "reads",
            Self::ReadsAndThrows => "readsAndThrows",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for CatalogOpenrouterRunInputOnRefreshed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CatalogOpenrouterRunInputStepsItem {
    Call(CatalogOpenrouterRunInputStepsItemCall),
    Together(CatalogOpenrouterRunInputStepsItemTogether),
    Advance(CatalogOpenrouterRunInputStepsItemAdvance),
    Rows(CatalogOpenrouterRunInputStepsItemRows),
}

impl Shape for CatalogOpenrouterRunInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "call" => CatalogOpenrouterRunInputStepsItemCall::decode_at(value, at).map(Self::Call),
            "together" => {
                CatalogOpenrouterRunInputStepsItemTogether::decode_at(value, at).map(Self::Together)
            }
            "advance" => {
                CatalogOpenrouterRunInputStepsItemAdvance::decode_at(value, at).map(Self::Advance)
            }
            "rows" => CatalogOpenrouterRunInputStepsItemRows::decode_at(value, at).map(Self::Rows),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Call(held) => held.encode(),
            Self::Together(held) => held.encode(),
            Self::Advance(held) => held.encode(),
            Self::Rows(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemCall {
    pub call: CatalogOpenrouterRunInputStepsItemCallCall,
}

impl Shape for CatalogOpenrouterRunInputStepsItemCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "call")
        })?;
        let call = shape::required(
            object,
            "call",
            at,
            CatalogOpenrouterRunInputStepsItemCallCall::decode_at,
        )?;
        Ok(Self { call })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("call"));
        object.insert("call", self.call.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemTogether {
    pub calls: Vec<CatalogOpenrouterRunInputStepsItemTogetherCallsItem>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemTogether {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "together")
        })?;
        let calls = shape::required(object, "calls", at, |value, at| {
            shape::list_within(
                value,
                at,
                Some(2),
                None,
                CatalogOpenrouterRunInputStepsItemTogetherCallsItem::decode_at,
            )
        })?;
        Ok(Self { calls })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("together"));
        object.insert(
            "calls",
            Value::Array(self.calls.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemAdvance {
    pub ms: u64,
}

impl Shape for CatalogOpenrouterRunInputStepsItemAdvance {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "advance")
        })?;
        let ms = shape::required(object, "ms", at, shape::natural)?;
        Ok(Self { ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("advance"));
        object.insert("ms", Value::Number(self.ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemRows {
    pub rows: Vec<String>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemRows {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "rows")
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("rows"));
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CatalogOpenrouterRunInputStepsItemCallCall {
    Views(CatalogOpenrouterRunInputStepsItemCallCallViews),
    View(CatalogOpenrouterRunInputStepsItemCallCallView),
    ViewFor(CatalogOpenrouterRunInputStepsItemCallCallViewFor),
    Entry(CatalogOpenrouterRunInputStepsItemCallCallEntry),
    Listing(CatalogOpenrouterRunInputStepsItemCallCallListing),
}

impl Shape for CatalogOpenrouterRunInputStepsItemCallCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => CatalogOpenrouterRunInputStepsItemCallCallViews::decode_at(value, at)
                .map(Self::Views),
            "view" => {
                CatalogOpenrouterRunInputStepsItemCallCallView::decode_at(value, at).map(Self::View)
            }
            "viewFor" => CatalogOpenrouterRunInputStepsItemCallCallViewFor::decode_at(value, at)
                .map(Self::ViewFor),
            "entry" => CatalogOpenrouterRunInputStepsItemCallCallEntry::decode_at(value, at)
                .map(Self::Entry),
            "listing" => CatalogOpenrouterRunInputStepsItemCallCallListing::decode_at(value, at)
                .map(Self::Listing),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::ViewFor(held) => held.encode(),
            Self::Entry(held) => held.encode(),
            Self::Listing(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CatalogOpenrouterRunInputStepsItemTogetherCallsItem {
    Views(CatalogOpenrouterRunInputStepsItemTogetherCallsItemViews),
    View(CatalogOpenrouterRunInputStepsItemTogetherCallsItemView),
    ViewFor(CatalogOpenrouterRunInputStepsItemTogetherCallsItemViewFor),
    Entry(CatalogOpenrouterRunInputStepsItemTogetherCallsItemEntry),
    Listing(CatalogOpenrouterRunInputStepsItemTogetherCallsItemListing),
}

impl Shape for CatalogOpenrouterRunInputStepsItemTogetherCallsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => {
                CatalogOpenrouterRunInputStepsItemTogetherCallsItemViews::decode_at(value, at)
                    .map(Self::Views)
            }
            "view" => CatalogOpenrouterRunInputStepsItemTogetherCallsItemView::decode_at(value, at)
                .map(Self::View),
            "viewFor" => {
                CatalogOpenrouterRunInputStepsItemTogetherCallsItemViewFor::decode_at(value, at)
                    .map(Self::ViewFor)
            }
            "entry" => {
                CatalogOpenrouterRunInputStepsItemTogetherCallsItemEntry::decode_at(value, at)
                    .map(Self::Entry)
            }
            "listing" => {
                CatalogOpenrouterRunInputStepsItemTogetherCallsItemListing::decode_at(value, at)
                    .map(Self::Listing)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::ViewFor(held) => held.encode(),
            Self::Entry(held) => held.encode(),
            Self::Listing(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemCallCallViews {
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub zdr_only: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemCallCallViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::optional(object, "zdrOnly", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            use_,
            zdr_only,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.zdr_only {
            object.insert("zdrOnly", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemCallCallView {
    pub key: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub zdr_only: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemCallCallView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::optional(object, "zdrOnly", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            key,
            use_,
            zdr_only,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("key", Value::from(self.key.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.zdr_only {
            object.insert("zdrOnly", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemCallCallViewFor {
    pub row: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemCallCallViewFor {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "viewFor")
        })?;
        let row = shape::required(object, "row", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            row,
            use_,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("viewFor"));
        object.insert("row", Value::from(self.row.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemCallCallEntry {
    pub id: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemCallCallEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "entry")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            id,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("entry"));
        object.insert("id", Value::from(self.id.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemCallCallListing {
    pub retry: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemCallCallListing {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "listing")
        })?;
        let retry = shape::optional(object, "retry", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            retry,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("listing"));
        if let Some(held) = &self.retry {
            object.insert("retry", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemTogetherCallsItemViews {
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub zdr_only: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemTogetherCallsItemViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::optional(object, "zdrOnly", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            use_,
            zdr_only,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.zdr_only {
            object.insert("zdrOnly", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemTogetherCallsItemView {
    pub key: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub zdr_only: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemTogetherCallsItemView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::optional(object, "zdrOnly", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            key,
            use_,
            zdr_only,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("key", Value::from(self.key.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.zdr_only {
            object.insert("zdrOnly", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemTogetherCallsItemViewFor {
    pub row: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemTogetherCallsItemViewFor {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "viewFor")
        })?;
        let row = shape::required(object, "row", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            row,
            use_,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("viewFor"));
        object.insert("row", Value::from(self.row.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemTogetherCallsItemEntry {
    pub id: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemTogetherCallsItemEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "entry")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            id,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("entry"));
        object.insert("id", Value::from(self.id.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunInputStepsItemTogetherCallsItemListing {
    pub retry: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpenrouterRunInputStepsItemTogetherCallsItemListing {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "listing")
        })?;
        let retry = shape::optional(object, "retry", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            retry,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("listing"));
        if let Some(held) = &self.retry {
            object.insert("retry", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunResult {
    pub steps: Vec<CatalogOpenrouterRunResultStepsItem>,
    pub requests: Vec<String>,
    pub read: Vec<i64>,
    pub waiting: Vec<String>,
    pub refreshed: i64,
    pub saw: Vec<String>,
}

impl Shape for CatalogOpenrouterRunResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, CatalogOpenrouterRunResultStepsItem::decode_at)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let read = shape::required(object, "read", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let refreshed = shape::required(object, "refreshed", at, shape::integer)?;
        let saw = shape::required(object, "saw", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            steps,
            requests,
            read,
            waiting,
            refreshed,
            saw,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "requests",
            Value::Array(
                self.requests
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "read",
            Value::Array(
                self.read
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("refreshed", Value::Number(self.refreshed as f64));
        object.insert(
            "saw",
            Value::Array(
                self.saw
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunResultStepsItem {
    pub now_ms: i64,
    pub answers: Vec<CatalogOpenrouterRunResultStepsItemAnswersItem>,
    pub row_reads: i64,
}

impl Shape for CatalogOpenrouterRunResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let answers = shape::required(object, "answers", at, |value, at| {
            shape::list(
                value,
                at,
                CatalogOpenrouterRunResultStepsItemAnswersItem::decode_at,
            )
        })?;
        let row_reads = shape::required(object, "rowReads", at, shape::integer)?;
        Ok(Self {
            now_ms,
            answers,
            row_reads,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert(
            "answers",
            Value::Array(self.answers.iter().map(|held| held.encode()).collect()),
        );
        object.insert("rowReads", Value::Number(self.row_reads as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CatalogOpenrouterRunResultStepsItemAnswersItem {
    Views(CatalogOpenrouterRunResultStepsItemAnswersItemViews),
    View(CatalogOpenrouterRunResultStepsItemAnswersItemView),
    Entry(CatalogOpenrouterRunResultStepsItemAnswersItemEntry),
    Listing(CatalogOpenrouterRunResultStepsItemAnswersItemListing),
    Thrown(CatalogOpenrouterRunResultStepsItemAnswersItemThrown),
}

impl Shape for CatalogOpenrouterRunResultStepsItemAnswersItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => CatalogOpenrouterRunResultStepsItemAnswersItemViews::decode_at(value, at)
                .map(Self::Views),
            "view" => CatalogOpenrouterRunResultStepsItemAnswersItemView::decode_at(value, at)
                .map(Self::View),
            "entry" => CatalogOpenrouterRunResultStepsItemAnswersItemEntry::decode_at(value, at)
                .map(Self::Entry),
            "listing" => {
                CatalogOpenrouterRunResultStepsItemAnswersItemListing::decode_at(value, at)
                    .map(Self::Listing)
            }
            "thrown" => CatalogOpenrouterRunResultStepsItemAnswersItemThrown::decode_at(value, at)
                .map(Self::Thrown),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::Entry(held) => held.encode(),
            Self::Listing(held) => held.encode(),
            Self::Thrown(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunResultStepsItemAnswersItemViews {
    pub views: Vec<ViewLine>,
}

impl Shape for CatalogOpenrouterRunResultStepsItemAnswersItemViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let views = shape::required(object, "views", at, |value, at| {
            shape::list(value, at, ViewLine::decode_at)
        })?;
        Ok(Self { views })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        object.insert(
            "views",
            Value::Array(self.views.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunResultStepsItemAnswersItemView {
    pub view: Option<ViewLine>,
}

impl Shape for CatalogOpenrouterRunResultStepsItemAnswersItemView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let view = shape::required(object, "view", at, |value, at| {
            shape::nullable(value, at, ViewLine::decode_at)
        })?;
        Ok(Self { view })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert(
            "view",
            self.view.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunResultStepsItemAnswersItemEntry {
    pub entry: Option<CatalogOpenrouterRunResultStepsItemAnswersItemEntryEntry>,
}

impl Shape for CatalogOpenrouterRunResultStepsItemAnswersItemEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "entry")
        })?;
        let entry = shape::required(object, "entry", at, |value, at| {
            shape::nullable(
                value,
                at,
                CatalogOpenrouterRunResultStepsItemAnswersItemEntryEntry::decode_at,
            )
        })?;
        Ok(Self { entry })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("entry"));
        object.insert(
            "entry",
            self.entry
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunResultStepsItemAnswersItemListing {
    pub listing: Option<CatalogOpenrouterRunResultStepsItemAnswersItemListingListing>,
}

impl Shape for CatalogOpenrouterRunResultStepsItemAnswersItemListing {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "listing")
        })?;
        let listing = shape::required(object, "listing", at, |value, at| {
            shape::nullable(
                value,
                at,
                CatalogOpenrouterRunResultStepsItemAnswersItemListingListing::decode_at,
            )
        })?;
        Ok(Self { listing })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("listing"));
        object.insert(
            "listing",
            self.listing
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunResultStepsItemAnswersItemThrown {
    pub message: String,
}

impl Shape for CatalogOpenrouterRunResultStepsItemAnswersItemThrown {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "thrown")
        })?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self { message })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("thrown"));
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunResultStepsItemAnswersItemEntryEntry {
    pub id: String,
    pub name: Option<String>,
    pub efforts: Option<Vec<String>>,
}

impl Shape for CatalogOpenrouterRunResultStepsItemAnswersItemEntryEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let name = shape::required(object, "name", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        Ok(Self { id, name, efforts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert(
            "name",
            self.name
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "efforts",
            self.efforts.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpenrouterRunResultStepsItemAnswersItemListingListing {
    pub raw: i64,
    pub endpoints: Vec<String>,
}

impl Shape for CatalogOpenrouterRunResultStepsItemAnswersItemListingListing {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let raw = shape::required(object, "raw", at, shape::integer)?;
        let endpoints = shape::required(object, "endpoints", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { raw, endpoints })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("raw", Value::Number(self.raw as f64));
        object.insert(
            "endpoints",
            Value::Array(
                self.endpoints
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunInput {
    pub start_ms: i64,
    pub exchanges: Vec<NetExchange>,
    pub rows: Vec<String>,
    pub plan: nyaaai_contract::shapes::provider::ProviderPlan,
    pub ttl_ms: Option<i64>,
    pub on_refreshed: Option<CatalogOpencodeRunInputOnRefreshed>,
    pub steps: Vec<CatalogOpencodeRunInputStepsItem>,
}

impl Shape for CatalogOpencodeRunInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let plan = shape::required(
            object,
            "plan",
            at,
            nyaaai_contract::shapes::provider::ProviderPlan::decode_at,
        )?;
        let ttl_ms = shape::optional(object, "ttlMs", at, shape::integer)?;
        let on_refreshed = shape::optional(
            object,
            "onRefreshed",
            at,
            CatalogOpencodeRunInputOnRefreshed::decode_at,
        )?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list_within(
                value,
                at,
                Some(1),
                None,
                CatalogOpencodeRunInputStepsItem::decode_at,
            )
        })?;
        Ok(Self {
            start_ms,
            exchanges,
            rows,
            plan,
            ttl_ms,
            on_refreshed,
            steps,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("plan", self.plan.encode());
        if let Some(held) = &self.ttl_ms {
            object.insert("ttlMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.on_refreshed {
            object.insert("onRefreshed", held.encode());
        }
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogOpencodeRunInputOnRefreshed {
    Reads,
    ReadsAndThrows,
}

impl CatalogOpencodeRunInputOnRefreshed {
    pub const ALL: [Self; 2] = [Self::Reads, Self::ReadsAndThrows];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reads => "reads",
            Self::ReadsAndThrows => "readsAndThrows",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for CatalogOpencodeRunInputOnRefreshed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CatalogOpencodeRunInputStepsItem {
    Call(CatalogOpencodeRunInputStepsItemCall),
    Together(CatalogOpencodeRunInputStepsItemTogether),
    Advance(CatalogOpencodeRunInputStepsItemAdvance),
    Rows(CatalogOpencodeRunInputStepsItemRows),
}

impl Shape for CatalogOpencodeRunInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "call" => CatalogOpencodeRunInputStepsItemCall::decode_at(value, at).map(Self::Call),
            "together" => {
                CatalogOpencodeRunInputStepsItemTogether::decode_at(value, at).map(Self::Together)
            }
            "advance" => {
                CatalogOpencodeRunInputStepsItemAdvance::decode_at(value, at).map(Self::Advance)
            }
            "rows" => CatalogOpencodeRunInputStepsItemRows::decode_at(value, at).map(Self::Rows),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Call(held) => held.encode(),
            Self::Together(held) => held.encode(),
            Self::Advance(held) => held.encode(),
            Self::Rows(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunInputStepsItemCall {
    pub call: CatalogOpencodeRunInputStepsItemCallCall,
}

impl Shape for CatalogOpencodeRunInputStepsItemCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "call")
        })?;
        let call = shape::required(
            object,
            "call",
            at,
            CatalogOpencodeRunInputStepsItemCallCall::decode_at,
        )?;
        Ok(Self { call })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("call"));
        object.insert("call", self.call.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunInputStepsItemTogether {
    pub calls: Vec<CatalogOpencodeRunInputStepsItemTogetherCallsItem>,
}

impl Shape for CatalogOpencodeRunInputStepsItemTogether {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "together")
        })?;
        let calls = shape::required(object, "calls", at, |value, at| {
            shape::list_within(
                value,
                at,
                Some(2),
                None,
                CatalogOpencodeRunInputStepsItemTogetherCallsItem::decode_at,
            )
        })?;
        Ok(Self { calls })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("together"));
        object.insert(
            "calls",
            Value::Array(self.calls.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunInputStepsItemAdvance {
    pub ms: u64,
}

impl Shape for CatalogOpencodeRunInputStepsItemAdvance {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "advance")
        })?;
        let ms = shape::required(object, "ms", at, shape::natural)?;
        Ok(Self { ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("advance"));
        object.insert("ms", Value::Number(self.ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunInputStepsItemRows {
    pub rows: Vec<String>,
}

impl Shape for CatalogOpencodeRunInputStepsItemRows {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "rows")
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("rows"));
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CatalogOpencodeRunInputStepsItemCallCall {
    Views(CatalogOpencodeRunInputStepsItemCallCallViews),
    ViewFor(CatalogOpencodeRunInputStepsItemCallCallViewFor),
    Models(CatalogOpencodeRunInputStepsItemCallCallModels),
}

impl Shape for CatalogOpencodeRunInputStepsItemCallCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => {
                CatalogOpencodeRunInputStepsItemCallCallViews::decode_at(value, at).map(Self::Views)
            }
            "viewFor" => CatalogOpencodeRunInputStepsItemCallCallViewFor::decode_at(value, at)
                .map(Self::ViewFor),
            "models" => CatalogOpencodeRunInputStepsItemCallCallModels::decode_at(value, at)
                .map(Self::Models),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::ViewFor(held) => held.encode(),
            Self::Models(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CatalogOpencodeRunInputStepsItemTogetherCallsItem {
    Views(CatalogOpencodeRunInputStepsItemTogetherCallsItemViews),
    ViewFor(CatalogOpencodeRunInputStepsItemTogetherCallsItemViewFor),
    Models(CatalogOpencodeRunInputStepsItemTogetherCallsItemModels),
}

impl Shape for CatalogOpencodeRunInputStepsItemTogetherCallsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => CatalogOpencodeRunInputStepsItemTogetherCallsItemViews::decode_at(value, at)
                .map(Self::Views),
            "viewFor" => {
                CatalogOpencodeRunInputStepsItemTogetherCallsItemViewFor::decode_at(value, at)
                    .map(Self::ViewFor)
            }
            "models" => {
                CatalogOpencodeRunInputStepsItemTogetherCallsItemModels::decode_at(value, at)
                    .map(Self::Models)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::ViewFor(held) => held.encode(),
            Self::Models(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunInputStepsItemCallCallViews {
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub zdr_only: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpencodeRunInputStepsItemCallCallViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::optional(object, "zdrOnly", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            use_,
            zdr_only,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.zdr_only {
            object.insert("zdrOnly", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunInputStepsItemCallCallViewFor {
    pub row: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpencodeRunInputStepsItemCallCallViewFor {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "viewFor")
        })?;
        let row = shape::required(object, "row", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            row,
            use_,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("viewFor"));
        object.insert("row", Value::from(self.row.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunInputStepsItemCallCallModels {
    pub fresh: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpencodeRunInputStepsItemCallCallModels {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "models")
        })?;
        let fresh = shape::optional(object, "fresh", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            fresh,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("models"));
        if let Some(held) = &self.fresh {
            object.insert("fresh", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunInputStepsItemTogetherCallsItemViews {
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub zdr_only: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpencodeRunInputStepsItemTogetherCallsItemViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::optional(object, "zdrOnly", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            use_,
            zdr_only,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.zdr_only {
            object.insert("zdrOnly", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunInputStepsItemTogetherCallsItemViewFor {
    pub row: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpencodeRunInputStepsItemTogetherCallsItemViewFor {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "viewFor")
        })?;
        let row = shape::required(object, "row", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            row,
            use_,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("viewFor"));
        object.insert("row", Value::from(self.row.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunInputStepsItemTogetherCallsItemModels {
    pub fresh: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for CatalogOpencodeRunInputStepsItemTogetherCallsItemModels {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "models")
        })?;
        let fresh = shape::optional(object, "fresh", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            fresh,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("models"));
        if let Some(held) = &self.fresh {
            object.insert("fresh", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunResult {
    pub steps: Vec<CatalogOpencodeRunResultStepsItem>,
    pub requests: Vec<String>,
    pub read: Vec<i64>,
    pub waiting: Vec<String>,
    pub refreshed: i64,
    pub saw: Vec<String>,
}

impl Shape for CatalogOpencodeRunResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, CatalogOpencodeRunResultStepsItem::decode_at)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let read = shape::required(object, "read", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let refreshed = shape::required(object, "refreshed", at, shape::integer)?;
        let saw = shape::required(object, "saw", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            steps,
            requests,
            read,
            waiting,
            refreshed,
            saw,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "requests",
            Value::Array(
                self.requests
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "read",
            Value::Array(
                self.read
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("refreshed", Value::Number(self.refreshed as f64));
        object.insert(
            "saw",
            Value::Array(
                self.saw
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunResultStepsItem {
    pub now_ms: i64,
    pub answers: Vec<CatalogOpencodeRunResultStepsItemAnswersItem>,
    pub row_reads: i64,
}

impl Shape for CatalogOpencodeRunResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let answers = shape::required(object, "answers", at, |value, at| {
            shape::list(
                value,
                at,
                CatalogOpencodeRunResultStepsItemAnswersItem::decode_at,
            )
        })?;
        let row_reads = shape::required(object, "rowReads", at, shape::integer)?;
        Ok(Self {
            now_ms,
            answers,
            row_reads,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert(
            "answers",
            Value::Array(self.answers.iter().map(|held| held.encode()).collect()),
        );
        object.insert("rowReads", Value::Number(self.row_reads as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CatalogOpencodeRunResultStepsItemAnswersItem {
    Views(CatalogOpencodeRunResultStepsItemAnswersItemViews),
    View(CatalogOpencodeRunResultStepsItemAnswersItemView),
    Models(CatalogOpencodeRunResultStepsItemAnswersItemModels),
    Thrown(CatalogOpencodeRunResultStepsItemAnswersItemThrown),
}

impl Shape for CatalogOpencodeRunResultStepsItemAnswersItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => CatalogOpencodeRunResultStepsItemAnswersItemViews::decode_at(value, at)
                .map(Self::Views),
            "view" => CatalogOpencodeRunResultStepsItemAnswersItemView::decode_at(value, at)
                .map(Self::View),
            "models" => CatalogOpencodeRunResultStepsItemAnswersItemModels::decode_at(value, at)
                .map(Self::Models),
            "thrown" => CatalogOpencodeRunResultStepsItemAnswersItemThrown::decode_at(value, at)
                .map(Self::Thrown),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::Models(held) => held.encode(),
            Self::Thrown(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunResultStepsItemAnswersItemViews {
    pub views: Vec<ViewLine>,
}

impl Shape for CatalogOpencodeRunResultStepsItemAnswersItemViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let views = shape::required(object, "views", at, |value, at| {
            shape::list(value, at, ViewLine::decode_at)
        })?;
        Ok(Self { views })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        object.insert(
            "views",
            Value::Array(self.views.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunResultStepsItemAnswersItemView {
    pub view: ViewLine,
}

impl Shape for CatalogOpencodeRunResultStepsItemAnswersItemView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let view = shape::required(object, "view", at, ViewLine::decode_at)?;
        Ok(Self { view })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("view", self.view.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunResultStepsItemAnswersItemModels {
    pub models: Option<Vec<String>>,
}

impl Shape for CatalogOpencodeRunResultStepsItemAnswersItemModels {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "models")
        })?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("models"));
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CatalogOpencodeRunResultStepsItemAnswersItemThrown {
    pub message: String,
}

impl Shape for CatalogOpencodeRunResultStepsItemAnswersItemThrown {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "thrown")
        })?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self { message })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("thrown"));
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterModelsInput {
    pub raw: Vec<Value>,
    pub zdr: Value,
    pub now_ms: i64,
}

impl Shape for BrowseOpenrouterModelsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let raw = shape::required(object, "raw", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        let zdr = shape::required(object, "zdr", at, shape::any)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        Ok(Self { raw, zdr, now_ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("raw", Value::Array(self.raw.clone()));
        object.insert("zdr", self.zdr.clone());
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterModelsResult {
    pub models: Vec<nyaaai_contract::shapes::model::BrowseModel>,
}

impl Shape for BrowseOpenrouterModelsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::model::BrowseModel::decode_at,
            )
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "models",
            Value::Array(self.models.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterProvidersInput {
    pub raw: Value,
    pub zdr: Value,
}

impl Shape for BrowseOpenrouterProvidersInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let raw = shape::required(object, "raw", at, shape::any)?;
        let zdr = shape::required(object, "zdr", at, shape::any)?;
        Ok(Self { raw, zdr })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("raw", self.raw.clone());
        object.insert("zdr", self.zdr.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterProvidersResult {
    pub providers: Option<Vec<nyaaai_contract::shapes::model::ProviderRow>>,
}

impl Shape for BrowseOpenrouterProvidersResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let providers = shape::required(object, "providers", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(
                    value,
                    at,
                    nyaaai_contract::shapes::model::ProviderRow::decode_at,
                )
            })
        })?;
        Ok(Self { providers })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "providers",
            self.providers.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserInput {
    pub start_ms: i64,
    pub exchanges: Vec<NetExchange>,
    pub steps: Vec<BrowseOpenrouterBrowserInputStepsItem>,
}

impl Shape for BrowseOpenrouterBrowserInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list_within(
                value,
                at,
                Some(1),
                None,
                BrowseOpenrouterBrowserInputStepsItem::decode_at,
            )
        })?;
        Ok(Self {
            start_ms,
            exchanges,
            steps,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowseOpenrouterBrowserInputStepsItem {
    List(BrowseOpenrouterBrowserInputStepsItemList),
    Detail(BrowseOpenrouterBrowserInputStepsItemDetail),
    Listed(BrowseOpenrouterBrowserInputStepsItemListed),
    View(BrowseOpenrouterBrowserInputStepsItemView),
    Advance(BrowseOpenrouterBrowserInputStepsItemAdvance),
}

impl Shape for BrowseOpenrouterBrowserInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "list" => {
                BrowseOpenrouterBrowserInputStepsItemList::decode_at(value, at).map(Self::List)
            }
            "detail" => {
                BrowseOpenrouterBrowserInputStepsItemDetail::decode_at(value, at).map(Self::Detail)
            }
            "listed" => {
                BrowseOpenrouterBrowserInputStepsItemListed::decode_at(value, at).map(Self::Listed)
            }
            "view" => {
                BrowseOpenrouterBrowserInputStepsItemView::decode_at(value, at).map(Self::View)
            }
            "advance" => BrowseOpenrouterBrowserInputStepsItemAdvance::decode_at(value, at)
                .map(Self::Advance),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::List(held) => held.encode(),
            Self::Detail(held) => held.encode(),
            Self::Listed(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::Advance(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserInputStepsItemList {
    pub fresh: Option<bool>,
}

impl Shape for BrowseOpenrouterBrowserInputStepsItemList {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "list")
        })?;
        let fresh = shape::optional(object, "fresh", at, shape::flag)?;
        Ok(Self { fresh })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("list"));
        if let Some(held) = &self.fresh {
            object.insert("fresh", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserInputStepsItemDetail {
    pub id: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for BrowseOpenrouterBrowserInputStepsItemDetail {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "detail")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            id,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("detail"));
        object.insert("id", Value::from(self.id.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserInputStepsItemListed {
    pub id: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for BrowseOpenrouterBrowserInputStepsItemListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "listed")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            id,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("listed"));
        object.insert("id", Value::from(self.id.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserInputStepsItemView {
    pub row: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for BrowseOpenrouterBrowserInputStepsItemView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let row = shape::required(object, "row", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            row,
            use_,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("row", Value::from(self.row.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserInputStepsItemAdvance {
    pub ms: u64,
}

impl Shape for BrowseOpenrouterBrowserInputStepsItemAdvance {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "advance")
        })?;
        let ms = shape::required(object, "ms", at, shape::natural)?;
        Ok(Self { ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("advance"));
        object.insert("ms", Value::Number(self.ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserResult {
    pub steps: Vec<BrowseOpenrouterBrowserResultStepsItem>,
    pub requests: Vec<String>,
    pub read: Vec<i64>,
    pub waiting: Vec<String>,
}

impl Shape for BrowseOpenrouterBrowserResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, BrowseOpenrouterBrowserResultStepsItem::decode_at)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let read = shape::required(object, "read", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            steps,
            requests,
            read,
            waiting,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "requests",
            Value::Array(
                self.requests
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "read",
            Value::Array(
                self.read
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserResultStepsItem {
    pub now_ms: i64,
    pub answer: BrowseOpenrouterBrowserResultStepsItemAnswer,
}

impl Shape for BrowseOpenrouterBrowserResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let answer = shape::required(
            object,
            "answer",
            at,
            BrowseOpenrouterBrowserResultStepsItemAnswer::decode_at,
        )?;
        Ok(Self { now_ms, answer })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert("answer", self.answer.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowseOpenrouterBrowserResultStepsItemAnswer {
    List(BrowseOpenrouterBrowserResultStepsItemAnswerList),
    Detail(BrowseOpenrouterBrowserResultStepsItemAnswerDetail),
    Listed(BrowseOpenrouterBrowserResultStepsItemAnswerListed),
    View(BrowseOpenrouterBrowserResultStepsItemAnswerView),
    Advanced(BrowseOpenrouterBrowserResultStepsItemAnswerAdvanced),
    Thrown(BrowseOpenrouterBrowserResultStepsItemAnswerThrown),
}

impl Shape for BrowseOpenrouterBrowserResultStepsItemAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "list" => BrowseOpenrouterBrowserResultStepsItemAnswerList::decode_at(value, at)
                .map(Self::List),
            "detail" => BrowseOpenrouterBrowserResultStepsItemAnswerDetail::decode_at(value, at)
                .map(Self::Detail),
            "listed" => BrowseOpenrouterBrowserResultStepsItemAnswerListed::decode_at(value, at)
                .map(Self::Listed),
            "view" => BrowseOpenrouterBrowserResultStepsItemAnswerView::decode_at(value, at)
                .map(Self::View),
            "advanced" => {
                BrowseOpenrouterBrowserResultStepsItemAnswerAdvanced::decode_at(value, at)
                    .map(Self::Advanced)
            }
            "thrown" => BrowseOpenrouterBrowserResultStepsItemAnswerThrown::decode_at(value, at)
                .map(Self::Thrown),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::List(held) => held.encode(),
            Self::Detail(held) => held.encode(),
            Self::Listed(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::Advanced(held) => held.encode(),
            Self::Thrown(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserResultStepsItemAnswerList {
    pub models: Option<Vec<nyaaai_contract::shapes::model::BrowseModel>>,
}

impl Shape for BrowseOpenrouterBrowserResultStepsItemAnswerList {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "list")
        })?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(
                    value,
                    at,
                    nyaaai_contract::shapes::model::BrowseModel::decode_at,
                )
            })
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("list"));
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserResultStepsItemAnswerDetail {
    pub detail: Option<nyaaai_contract::shapes::model::BrowseDetail>,
}

impl Shape for BrowseOpenrouterBrowserResultStepsItemAnswerDetail {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "detail")
        })?;
        let detail = shape::required(object, "detail", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::model::BrowseDetail::decode_at,
            )
        })?;
        Ok(Self { detail })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("detail"));
        object.insert(
            "detail",
            self.detail
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserResultStepsItemAnswerListed {
    pub listed: Option<BrowseOpenrouterBrowserResultStepsItemAnswerListedListed>,
}

impl Shape for BrowseOpenrouterBrowserResultStepsItemAnswerListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "listed")
        })?;
        let listed = shape::required(object, "listed", at, |value, at| {
            shape::nullable(
                value,
                at,
                BrowseOpenrouterBrowserResultStepsItemAnswerListedListed::decode_at,
            )
        })?;
        Ok(Self { listed })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("listed"));
        object.insert(
            "listed",
            self.listed
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserResultStepsItemAnswerView {
    pub view: ViewLine,
}

impl Shape for BrowseOpenrouterBrowserResultStepsItemAnswerView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let view = shape::required(object, "view", at, ViewLine::decode_at)?;
        Ok(Self { view })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("view", self.view.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserResultStepsItemAnswerAdvanced {}

impl Shape for BrowseOpenrouterBrowserResultStepsItemAnswerAdvanced {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "advanced")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("advanced"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserResultStepsItemAnswerThrown {
    pub message: String,
}

impl Shape for BrowseOpenrouterBrowserResultStepsItemAnswerThrown {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "thrown")
        })?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self { message })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("thrown"));
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterBrowserResultStepsItemAnswerListedListed {
    pub label: String,
    pub reasoning: bool,
}

impl Shape for BrowseOpenrouterBrowserResultStepsItemAnswerListedListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let reasoning = shape::required(object, "reasoning", at, shape::flag)?;
        Ok(Self { label, reasoning })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("reasoning", Value::Bool(self.reasoning));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterHopInput {
    pub start_ms: i64,
    pub exchanges: Vec<NetExchange>,
    pub id: String,
    pub view_hosts: Vec<String>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for BrowseOpenrouterHopInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let view_hosts = shape::required(object, "viewHosts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            start_ms,
            exchanges,
            id,
            view_hosts,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        object.insert("id", Value::from(self.id.as_str()));
        object.insert(
            "viewHosts",
            Value::Array(
                self.view_hosts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterHopResult {
    pub hosts: Option<Vec<String>>,
    pub thrown: Option<String>,
    pub requests: Vec<String>,
    pub waiting: Vec<String>,
    pub now_ms: i64,
}

impl Shape for BrowseOpenrouterHopResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let hosts = shape::required(object, "hosts", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        let thrown = shape::required(object, "thrown", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        Ok(Self {
            hosts,
            thrown,
            requests,
            waiting,
            now_ms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "hosts",
            self.hosts.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }),
        );
        object.insert(
            "thrown",
            self.thrown
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "requests",
            Value::Array(
                self.requests
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterPinsInput {
    pub calls: Vec<BrowseOpenrouterPinsInputCallsItem>,
}

impl Shape for BrowseOpenrouterPinsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let calls = shape::required(object, "calls", at, |value, at| {
            shape::list(value, at, BrowseOpenrouterPinsInputCallsItem::decode_at)
        })?;
        Ok(Self { calls })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "calls",
            Value::Array(self.calls.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowseOpenrouterPinsInputCallsItem {
    Pin(BrowseOpenrouterPinsInputCallsItemPin),
    Encode(BrowseOpenrouterPinsInputCallsItemEncode),
    PassedBack(BrowseOpenrouterPinsInputCallsItemPassedBack),
}

impl Shape for BrowseOpenrouterPinsInputCallsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "pin" => BrowseOpenrouterPinsInputCallsItemPin::decode_at(value, at).map(Self::Pin),
            "encode" => {
                BrowseOpenrouterPinsInputCallsItemEncode::decode_at(value, at).map(Self::Encode)
            }
            "passedBack" => BrowseOpenrouterPinsInputCallsItemPassedBack::decode_at(value, at)
                .map(Self::PassedBack),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Pin(held) => held.encode(),
            Self::Encode(held) => held.encode(),
            Self::PassedBack(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterPinsInputCallsItemPin {
    pub hosts: Vec<String>,
}

impl Shape for BrowseOpenrouterPinsInputCallsItemPin {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "pin")
        })?;
        let hosts = shape::required(object, "hosts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { hosts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("pin"));
        object.insert(
            "hosts",
            Value::Array(
                self.hosts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterPinsInputCallsItemEncode {
    pub request: nyaaai_contract::shapes::step::CanonicalRequest,
    pub mode: BrowseOpenrouterPinsInputCallsItemEncodeMode,
}

impl Shape for BrowseOpenrouterPinsInputCallsItemEncode {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "encode")
        })?;
        let request = shape::required(
            object,
            "request",
            at,
            nyaaai_contract::shapes::step::CanonicalRequest::decode_at,
        )?;
        let mode = shape::required(
            object,
            "mode",
            at,
            BrowseOpenrouterPinsInputCallsItemEncodeMode::decode_at,
        )?;
        Ok(Self { request, mode })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("encode"));
        object.insert("request", self.request.encode());
        object.insert("mode", self.mode.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterPinsInputCallsItemPassedBack {}

impl Shape for BrowseOpenrouterPinsInputCallsItemPassedBack {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "passedBack")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("passedBack"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BrowseOpenrouterPinsInputCallsItemEncodeMode {
    Stream,
    Complete,
}

impl BrowseOpenrouterPinsInputCallsItemEncodeMode {
    pub const ALL: [Self; 2] = [Self::Stream, Self::Complete];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stream => "stream",
            Self::Complete => "complete",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for BrowseOpenrouterPinsInputCallsItemEncodeMode {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterPinsResult {
    pub answers: Vec<BrowseOpenrouterPinsResultAnswersItem>,
}

impl Shape for BrowseOpenrouterPinsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let answers = shape::required(object, "answers", at, |value, at| {
            shape::list(value, at, BrowseOpenrouterPinsResultAnswersItem::decode_at)
        })?;
        Ok(Self { answers })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "answers",
            Value::Array(self.answers.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowseOpenrouterPinsResultAnswersItem {
    Pinned(BrowseOpenrouterPinsResultAnswersItemPinned),
    Encoded(BrowseOpenrouterPinsResultAnswersItemEncoded),
    PassedBack(BrowseOpenrouterPinsResultAnswersItemPassedBack),
}

impl Shape for BrowseOpenrouterPinsResultAnswersItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "pinned" => {
                BrowseOpenrouterPinsResultAnswersItemPinned::decode_at(value, at).map(Self::Pinned)
            }
            "encoded" => BrowseOpenrouterPinsResultAnswersItemEncoded::decode_at(value, at)
                .map(Self::Encoded),
            "passedBack" => BrowseOpenrouterPinsResultAnswersItemPassedBack::decode_at(value, at)
                .map(Self::PassedBack),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Pinned(held) => held.encode(),
            Self::Encoded(held) => held.encode(),
            Self::PassedBack(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterPinsResultAnswersItemPinned {}

impl Shape for BrowseOpenrouterPinsResultAnswersItemPinned {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "pinned")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("pinned"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterPinsResultAnswersItemEncoded {
    pub provider: Value,
    pub replayed: bool,
}

impl Shape for BrowseOpenrouterPinsResultAnswersItemEncoded {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "encoded")
        })?;
        let provider = shape::required(object, "provider", at, shape::any)?;
        let replayed = shape::required(object, "replayed", at, shape::flag)?;
        Ok(Self { provider, replayed })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("encoded"));
        object.insert("provider", self.provider.clone());
        object.insert("replayed", Value::Bool(self.replayed));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpenrouterPinsResultAnswersItemPassedBack {
    pub value: bool,
}

impl Shape for BrowseOpenrouterPinsResultAnswersItemPassedBack {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "passedBack")
        })?;
        let value_field = shape::required(object, "value", at, shape::flag)?;
        Ok(Self { value: value_field })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("passedBack"));
        object.insert("value", Value::Bool(self.value));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserInput {
    pub start_ms: i64,
    pub plan: nyaaai_contract::shapes::provider::ProviderPlan,
    pub exchanges: Vec<NetExchange>,
    pub steps: Vec<BrowseOpencodeBrowserInputStepsItem>,
}

impl Shape for BrowseOpencodeBrowserInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let plan = shape::required(
            object,
            "plan",
            at,
            nyaaai_contract::shapes::provider::ProviderPlan::decode_at,
        )?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list_within(
                value,
                at,
                Some(1),
                None,
                BrowseOpencodeBrowserInputStepsItem::decode_at,
            )
        })?;
        Ok(Self {
            start_ms,
            plan,
            exchanges,
            steps,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert("plan", self.plan.encode());
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowseOpencodeBrowserInputStepsItem {
    List(BrowseOpencodeBrowserInputStepsItemList),
    Detail(BrowseOpencodeBrowserInputStepsItemDetail),
    Listed(BrowseOpencodeBrowserInputStepsItemListed),
    View(BrowseOpencodeBrowserInputStepsItemView),
    Advance(BrowseOpencodeBrowserInputStepsItemAdvance),
}

impl Shape for BrowseOpencodeBrowserInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "list" => BrowseOpencodeBrowserInputStepsItemList::decode_at(value, at).map(Self::List),
            "detail" => {
                BrowseOpencodeBrowserInputStepsItemDetail::decode_at(value, at).map(Self::Detail)
            }
            "listed" => {
                BrowseOpencodeBrowserInputStepsItemListed::decode_at(value, at).map(Self::Listed)
            }
            "view" => BrowseOpencodeBrowserInputStepsItemView::decode_at(value, at).map(Self::View),
            "advance" => {
                BrowseOpencodeBrowserInputStepsItemAdvance::decode_at(value, at).map(Self::Advance)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::List(held) => held.encode(),
            Self::Detail(held) => held.encode(),
            Self::Listed(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::Advance(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserInputStepsItemList {
    pub fresh: Option<bool>,
}

impl Shape for BrowseOpencodeBrowserInputStepsItemList {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "list")
        })?;
        let fresh = shape::optional(object, "fresh", at, shape::flag)?;
        Ok(Self { fresh })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("list"));
        if let Some(held) = &self.fresh {
            object.insert("fresh", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserInputStepsItemDetail {
    pub id: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for BrowseOpencodeBrowserInputStepsItemDetail {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "detail")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            id,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("detail"));
        object.insert("id", Value::from(self.id.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserInputStepsItemListed {
    pub id: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for BrowseOpencodeBrowserInputStepsItemListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "listed")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            id,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("listed"));
        object.insert("id", Value::from(self.id.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserInputStepsItemView {
    pub row: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for BrowseOpencodeBrowserInputStepsItemView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let row = shape::required(object, "row", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            row,
            use_,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("row", Value::from(self.row.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserInputStepsItemAdvance {
    pub ms: u64,
}

impl Shape for BrowseOpencodeBrowserInputStepsItemAdvance {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "advance")
        })?;
        let ms = shape::required(object, "ms", at, shape::natural)?;
        Ok(Self { ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("advance"));
        object.insert("ms", Value::Number(self.ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserResult {
    pub steps: Vec<BrowseOpencodeBrowserResultStepsItem>,
    pub requests: Vec<String>,
    pub read: Vec<i64>,
    pub waiting: Vec<String>,
}

impl Shape for BrowseOpencodeBrowserResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, BrowseOpencodeBrowserResultStepsItem::decode_at)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let read = shape::required(object, "read", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            steps,
            requests,
            read,
            waiting,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "requests",
            Value::Array(
                self.requests
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "read",
            Value::Array(
                self.read
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserResultStepsItem {
    pub now_ms: i64,
    pub answer: BrowseOpencodeBrowserResultStepsItemAnswer,
}

impl Shape for BrowseOpencodeBrowserResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let answer = shape::required(
            object,
            "answer",
            at,
            BrowseOpencodeBrowserResultStepsItemAnswer::decode_at,
        )?;
        Ok(Self { now_ms, answer })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert("answer", self.answer.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BrowseOpencodeBrowserResultStepsItemAnswer {
    List(BrowseOpencodeBrowserResultStepsItemAnswerList),
    Detail(BrowseOpencodeBrowserResultStepsItemAnswerDetail),
    Listed(BrowseOpencodeBrowserResultStepsItemAnswerListed),
    View(BrowseOpencodeBrowserResultStepsItemAnswerView),
    Advanced(BrowseOpencodeBrowserResultStepsItemAnswerAdvanced),
    Thrown(BrowseOpencodeBrowserResultStepsItemAnswerThrown),
}

impl Shape for BrowseOpencodeBrowserResultStepsItemAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "list" => {
                BrowseOpencodeBrowserResultStepsItemAnswerList::decode_at(value, at).map(Self::List)
            }
            "detail" => BrowseOpencodeBrowserResultStepsItemAnswerDetail::decode_at(value, at)
                .map(Self::Detail),
            "listed" => BrowseOpencodeBrowserResultStepsItemAnswerListed::decode_at(value, at)
                .map(Self::Listed),
            "view" => {
                BrowseOpencodeBrowserResultStepsItemAnswerView::decode_at(value, at).map(Self::View)
            }
            "advanced" => BrowseOpencodeBrowserResultStepsItemAnswerAdvanced::decode_at(value, at)
                .map(Self::Advanced),
            "thrown" => BrowseOpencodeBrowserResultStepsItemAnswerThrown::decode_at(value, at)
                .map(Self::Thrown),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::List(held) => held.encode(),
            Self::Detail(held) => held.encode(),
            Self::Listed(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::Advanced(held) => held.encode(),
            Self::Thrown(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserResultStepsItemAnswerList {
    pub models: Option<Vec<nyaaai_contract::shapes::model::BrowseModel>>,
}

impl Shape for BrowseOpencodeBrowserResultStepsItemAnswerList {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "list")
        })?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(
                    value,
                    at,
                    nyaaai_contract::shapes::model::BrowseModel::decode_at,
                )
            })
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("list"));
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserResultStepsItemAnswerDetail {
    pub detail: Option<nyaaai_contract::shapes::model::BrowseDetail>,
}

impl Shape for BrowseOpencodeBrowserResultStepsItemAnswerDetail {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "detail")
        })?;
        let detail = shape::required(object, "detail", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::model::BrowseDetail::decode_at,
            )
        })?;
        Ok(Self { detail })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("detail"));
        object.insert(
            "detail",
            self.detail
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserResultStepsItemAnswerListed {
    pub listed: Option<BrowseOpencodeBrowserResultStepsItemAnswerListedListed>,
}

impl Shape for BrowseOpencodeBrowserResultStepsItemAnswerListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "listed")
        })?;
        let listed = shape::required(object, "listed", at, |value, at| {
            shape::nullable(
                value,
                at,
                BrowseOpencodeBrowserResultStepsItemAnswerListedListed::decode_at,
            )
        })?;
        Ok(Self { listed })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("listed"));
        object.insert(
            "listed",
            self.listed
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserResultStepsItemAnswerView {
    pub view: ViewLine,
}

impl Shape for BrowseOpencodeBrowserResultStepsItemAnswerView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let view = shape::required(object, "view", at, ViewLine::decode_at)?;
        Ok(Self { view })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("view", self.view.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserResultStepsItemAnswerAdvanced {}

impl Shape for BrowseOpencodeBrowserResultStepsItemAnswerAdvanced {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "advanced")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("advanced"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserResultStepsItemAnswerThrown {
    pub message: String,
}

impl Shape for BrowseOpencodeBrowserResultStepsItemAnswerThrown {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "thrown")
        })?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self { message })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("thrown"));
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BrowseOpencodeBrowserResultStepsItemAnswerListedListed {
    pub label: String,
    pub reasoning: bool,
}

impl Shape for BrowseOpencodeBrowserResultStepsItemAnswerListedListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let reasoning = shape::required(object, "reasoning", at, shape::flag)?;
        Ok(Self { label, reasoning })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("reasoning", Value::Bool(self.reasoning));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindOpenrouterKeyInput {
    pub start_ms: i64,
    pub exchanges: Vec<NetExchange>,
    pub stored: Option<String>,
    pub checks: Vec<KindOpenrouterKeyInputChecksItem>,
}

impl Shape for KindOpenrouterKeyInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let stored = shape::required(object, "stored", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let checks = shape::required(object, "checks", at, |value, at| {
            shape::list(value, at, KindOpenrouterKeyInputChecksItem::decode_at)
        })?;
        Ok(Self {
            start_ms,
            exchanges,
            stored,
            checks,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "stored",
            self.stored
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "checks",
            Value::Array(self.checks.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KindOpenrouterKeyInputChecksItem {
    Stored(KindOpenrouterKeyInputChecksItemStored),
    Key(KindOpenrouterKeyInputChecksItemKey),
}

impl Shape for KindOpenrouterKeyInputChecksItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "auth", at)? {
            "stored" => {
                KindOpenrouterKeyInputChecksItemStored::decode_at(value, at).map(Self::Stored)
            }
            "key" => KindOpenrouterKeyInputChecksItemKey::decode_at(value, at).map(Self::Key),
            _ => Err(shape::no_branch(at, "auth")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Stored(held) => held.encode(),
            Self::Key(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindOpenrouterKeyInputChecksItemStored {
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for KindOpenrouterKeyInputChecksItemStored {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "auth", at, |value, at| {
            shape::is_text(value, at, "stored")
        })?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("auth", Value::from("stored"));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindOpenrouterKeyInputChecksItemKey {
    pub key: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for KindOpenrouterKeyInputChecksItemKey {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "auth", at, |value, at| {
            shape::is_text(value, at, "key")
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            key,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("auth", Value::from("key"));
        object.insert("key", Value::from(self.key.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindOpenrouterKeyResult {
    pub checks: Vec<KindOpenrouterKeyResultChecksItem>,
    pub requests: Vec<KindOpenrouterKeyResultRequestsItem>,
    pub waiting: Vec<String>,
}

impl Shape for KindOpenrouterKeyResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let checks = shape::required(object, "checks", at, |value, at| {
            shape::list(value, at, KindOpenrouterKeyResultChecksItem::decode_at)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, KindOpenrouterKeyResultRequestsItem::decode_at)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            checks,
            requests,
            waiting,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "checks",
            Value::Array(self.checks.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "requests",
            Value::Array(self.requests.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindOpenrouterKeyResultChecksItem {
    pub ok: Option<bool>,
    pub reason: Option<KindOpenrouterKeyResultChecksItemReason>,
    pub usage: Option<f64>,
    pub limit: Option<f64>,
    pub reset: Option<String>,
    pub thrown: Option<String>,
}

impl Shape for KindOpenrouterKeyResultChecksItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let ok = shape::required(object, "ok", at, |value, at| {
            shape::nullable(value, at, shape::flag)
        })?;
        let reason = shape::required(object, "reason", at, |value, at| {
            shape::nullable(
                value,
                at,
                KindOpenrouterKeyResultChecksItemReason::decode_at,
            )
        })?;
        let usage = shape::required(object, "usage", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let limit = shape::required(object, "limit", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let reset = shape::required(object, "reset", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let thrown = shape::required(object, "thrown", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            ok,
            reason,
            usage,
            limit,
            reset,
            thrown,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "ok",
            self.ok
                .as_ref()
                .map_or(Value::Null, |held| Value::Bool(*held)),
        );
        object.insert(
            "reason",
            self.reason
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "usage",
            self.usage
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert(
            "limit",
            self.limit
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert(
            "reset",
            self.reset
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "thrown",
            self.thrown
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindOpenrouterKeyResultRequestsItem {
    pub path: String,
    pub authorization: Option<String>,
}

impl Shape for KindOpenrouterKeyResultRequestsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let path = shape::required(object, "path", at, shape::text)?;
        let authorization = shape::required(object, "authorization", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            path,
            authorization,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("path", Value::from(self.path.as_str()));
        object.insert(
            "authorization",
            self.authorization
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindOpenrouterKeyResultChecksItemReason {
    Rejected,
    Unreachable,
}

impl KindOpenrouterKeyResultChecksItemReason {
    pub const ALL: [Self; 2] = [Self::Rejected, Self::Unreachable];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Rejected => "rejected",
            Self::Unreachable => "unreachable",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KindOpenrouterKeyResultChecksItemReason {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindLinesInput {
    pub kinds: Option<Vec<nyaaai_contract::shapes::provider::ProviderKindName>>,
}

impl Shape for KindLinesInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kinds = shape::optional(object, "kinds", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::ProviderKindName::decode_at,
            )
        })?;
        Ok(Self { kinds })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.kinds {
            object.insert(
                "kinds",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindLinesResult {
    pub lines: Vec<nyaaai_contract::shapes::provider::KindLine>,
}

impl Shape for KindLinesResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let lines = shape::required(object, "lines", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::KindLine::decode_at,
            )
        })?;
        Ok(Self { lines })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "lines",
            Value::Array(self.lines.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindProviderViewInput {
    pub entry: nyaaai_contract::shapes::provider::ProviderEntry,
    pub status: nyaaai_contract::shapes::provider::KeyState,
    pub kinds: Option<Vec<nyaaai_contract::shapes::provider::ProviderKindName>>,
    pub bill: Option<nyaaai_contract::shapes::provider::Billed>,
}

impl Shape for KindProviderViewInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entry = shape::required(
            object,
            "entry",
            at,
            nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
        )?;
        let status = shape::required(
            object,
            "status",
            at,
            nyaaai_contract::shapes::provider::KeyState::decode_at,
        )?;
        let kinds = shape::optional(object, "kinds", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::ProviderKindName::decode_at,
            )
        })?;
        let bill = shape::optional(
            object,
            "bill",
            at,
            nyaaai_contract::shapes::provider::Billed::decode_at,
        )?;
        Ok(Self {
            entry,
            status,
            kinds,
            bill,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("entry", self.entry.encode());
        object.insert("status", self.status.encode());
        if let Some(held) = &self.kinds {
            object.insert(
                "kinds",
                Value::Array(held.iter().map(|held| held.encode()).collect()),
            );
        }
        if let Some(held) = &self.bill {
            object.insert("bill", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindProviderViewResult {
    pub view: nyaaai_contract::shapes::provider::ProviderView,
}

impl Shape for KindProviderViewResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let view = shape::required(
            object,
            "view",
            at,
            nyaaai_contract::shapes::provider::ProviderView::decode_at,
        )?;
        Ok(Self { view })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("view", self.view.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindFactsInput {
    pub kind: KindFactsInputKind,
    pub entry: KindFactsInputEntry,
    pub rows: Vec<KindFactsInputRowsItem>,
}

impl Shape for KindFactsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, KindFactsInputKind::decode_at)?;
        let entry = shape::required(object, "entry", at, KindFactsInputEntry::decode_at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, KindFactsInputRowsItem::decode_at)
        })?;
        Ok(Self { kind, entry, rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("entry", self.entry.encode());
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindFactsInputKind {
    Openrouter,
    Opencode,
}

impl KindFactsInputKind {
    pub const ALL: [Self; 2] = [Self::Openrouter, Self::Opencode];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openrouter => "openrouter",
            Self::Opencode => "opencode",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KindFactsInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindFactsInputEntry {
    pub label: String,
    pub plan: Option<nyaaai_contract::shapes::provider::ProviderPlan>,
}

impl Shape for KindFactsInputEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let plan = shape::optional(
            object,
            "plan",
            at,
            nyaaai_contract::shapes::provider::ProviderPlan::decode_at,
        )?;
        Ok(Self { label, plan })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("label", Value::from(self.label.as_str()));
        if let Some(held) = &self.plan {
            object.insert("plan", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindFactsInputRowsItem {
    pub state: nyaaai_contract::shapes::provider::KeyState,
    pub bill: Option<nyaaai_contract::shapes::provider::Billed>,
}

impl Shape for KindFactsInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let state = shape::required(
            object,
            "state",
            at,
            nyaaai_contract::shapes::provider::KeyState::decode_at,
        )?;
        let bill = shape::optional(
            object,
            "bill",
            at,
            nyaaai_contract::shapes::provider::Billed::decode_at,
        )?;
        Ok(Self { state, bill })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("state", self.state.encode());
        if let Some(held) = &self.bill {
            object.insert("bill", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindFactsResult {
    pub facts: Vec<Option<String>>,
}

impl Shape for KindFactsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let facts = shape::required(object, "facts", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { facts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "facts",
            Value::Array(
                self.facts
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindCheckInput {
    pub kind: KindCheckInputKind,
    pub entry: KindCheckInputEntry,
    pub secret: Option<String>,
    pub base_url: Option<String>,
    pub start_ms: i64,
    pub exchanges: Vec<NetExchange>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for KindCheckInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, KindCheckInputKind::decode_at)?;
        let entry = shape::required(object, "entry", at, KindCheckInputEntry::decode_at)?;
        let secret = shape::required(object, "secret", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let base_url = shape::optional(object, "baseUrl", at, shape::text)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            kind,
            entry,
            secret,
            base_url,
            start_ms,
            exchanges,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("entry", self.entry.encode());
        object.insert(
            "secret",
            self.secret
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        if let Some(held) = &self.base_url {
            object.insert("baseUrl", Value::from(held.as_str()));
        }
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindCheckInputKind {
    Openrouter,
    Opencode,
}

impl KindCheckInputKind {
    pub const ALL: [Self; 2] = [Self::Openrouter, Self::Opencode];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openrouter => "openrouter",
            Self::Opencode => "opencode",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KindCheckInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindCheckInputEntry {
    pub label: String,
    pub plan: Option<nyaaai_contract::shapes::provider::ProviderPlan>,
}

impl Shape for KindCheckInputEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let plan = shape::optional(
            object,
            "plan",
            at,
            nyaaai_contract::shapes::provider::ProviderPlan::decode_at,
        )?;
        Ok(Self { label, plan })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("label", Value::from(self.label.as_str()));
        if let Some(held) = &self.plan {
            object.insert("plan", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindCheckResult {
    pub state: Option<nyaaai_contract::shapes::provider::KeyState>,
    pub thrown: Option<String>,
    pub facts: Option<String>,
    pub requests: Vec<KindCheckResultRequestsItem>,
    pub waiting: Vec<String>,
}

impl Shape for KindCheckResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let state = shape::required(object, "state", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::provider::KeyState::decode_at,
            )
        })?;
        let thrown = shape::required(object, "thrown", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let facts = shape::required(object, "facts", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, KindCheckResultRequestsItem::decode_at)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            state,
            thrown,
            facts,
            requests,
            waiting,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "state",
            self.state
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "thrown",
            self.thrown
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "facts",
            self.facts
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "requests",
            Value::Array(self.requests.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindCheckResultRequestsItem {
    pub method: String,
    pub url: String,
    pub authorization: Option<String>,
    pub user_agent: Option<String>,
}

impl Shape for KindCheckResultRequestsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let method = shape::required(object, "method", at, shape::text)?;
        let url = shape::required(object, "url", at, shape::text)?;
        let authorization = shape::required(object, "authorization", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let user_agent = shape::required(object, "userAgent", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            method,
            url,
            authorization,
            user_agent,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("method", Value::from(self.method.as_str()));
        object.insert("url", Value::from(self.url.as_str()));
        object.insert(
            "authorization",
            self.authorization
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "userAgent",
            self.user_agent
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindLimitsInput {
    pub kind: KindLimitsInputKind,
    pub entry: KindLimitsInputEntry,
    pub stored: Option<String>,
    pub start_ms: i64,
    pub exchanges: Vec<NetExchange>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for KindLimitsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, KindLimitsInputKind::decode_at)?;
        let entry = shape::required(object, "entry", at, KindLimitsInputEntry::decode_at)?;
        let stored = shape::required(object, "stored", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            kind,
            entry,
            stored,
            start_ms,
            exchanges,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("entry", self.entry.encode());
        object.insert(
            "stored",
            self.stored
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindLimitsInputKind {
    Openrouter,
    Opencode,
}

impl KindLimitsInputKind {
    pub const ALL: [Self; 2] = [Self::Openrouter, Self::Opencode];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openrouter => "openrouter",
            Self::Opencode => "opencode",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KindLimitsInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindLimitsInputEntry {
    pub label: String,
    pub plan: Option<nyaaai_contract::shapes::provider::ProviderPlan>,
}

impl Shape for KindLimitsInputEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let plan = shape::optional(
            object,
            "plan",
            at,
            nyaaai_contract::shapes::provider::ProviderPlan::decode_at,
        )?;
        Ok(Self { label, plan })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("label", Value::from(self.label.as_str()));
        if let Some(held) = &self.plan {
            object.insert("plan", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindLimitsResult {
    pub present: bool,
    pub limits: Option<nyaaai_contract::shapes::provider::Limits>,
    pub requests: Vec<KindLimitsResultRequestsItem>,
    pub waiting: Vec<String>,
}

impl Shape for KindLimitsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let present = shape::required(object, "present", at, shape::flag)?;
        let limits = shape::required(object, "limits", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::provider::Limits::decode_at,
            )
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, KindLimitsResultRequestsItem::decode_at)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            present,
            limits,
            requests,
            waiting,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("present", Value::Bool(self.present));
        object.insert(
            "limits",
            self.limits
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "requests",
            Value::Array(self.requests.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindLimitsResultRequestsItem {
    pub method: String,
    pub url: String,
    pub authorization: Option<String>,
    pub user_agent: Option<String>,
}

impl Shape for KindLimitsResultRequestsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let method = shape::required(object, "method", at, shape::text)?;
        let url = shape::required(object, "url", at, shape::text)?;
        let authorization = shape::required(object, "authorization", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let user_agent = shape::required(object, "userAgent", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            method,
            url,
            authorization,
            user_agent,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("method", Value::from(self.method.as_str()));
        object.insert("url", Value::from(self.url.as_str()));
        object.insert(
            "authorization",
            self.authorization
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "userAgent",
            self.user_agent
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindCodecInput {
    pub kind: KindCodecInputKind,
    pub label: String,
    pub view: Option<KindCodecInputView>,
    pub request: nyaaai_contract::shapes::step::CanonicalRequest,
    pub mode: KindCodecInputMode,
    pub now_ms: i64,
    pub failures: Vec<KindCodecInputFailuresItem>,
}

impl Shape for KindCodecInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, KindCodecInputKind::decode_at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let view = shape::required(object, "view", at, |value, at| {
            shape::nullable(value, at, KindCodecInputView::decode_at)
        })?;
        let request = shape::required(
            object,
            "request",
            at,
            nyaaai_contract::shapes::step::CanonicalRequest::decode_at,
        )?;
        let mode = shape::required(object, "mode", at, KindCodecInputMode::decode_at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let failures = shape::required(object, "failures", at, |value, at| {
            shape::list(value, at, KindCodecInputFailuresItem::decode_at)
        })?;
        Ok(Self {
            kind,
            label,
            view,
            request,
            mode,
            now_ms,
            failures,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("label", Value::from(self.label.as_str()));
        object.insert(
            "view",
            self.view.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        object.insert("request", self.request.encode());
        object.insert("mode", self.mode.encode());
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert(
            "failures",
            Value::Array(self.failures.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindCodecInputKind {
    Openrouter,
    Opencode,
}

impl KindCodecInputKind {
    pub const ALL: [Self; 2] = [Self::Openrouter, Self::Opencode];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openrouter => "openrouter",
            Self::Opencode => "opencode",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KindCodecInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindCodecInputView {
    pub efforts: Vec<nyaaai_contract::shapes::step::Effort>,
    pub api: Option<nyaaai_contract::shapes::step::ApiFacts>,
    pub output_limit: Option<f64>,
    pub price: Option<KindCodecInputViewPrice>,
}

impl Shape for KindCodecInputView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
        })?;
        let api = shape::optional(
            object,
            "api",
            at,
            nyaaai_contract::shapes::step::ApiFacts::decode_at,
        )?;
        let output_limit = shape::optional(object, "outputLimit", at, shape::number)?;
        let price = shape::optional(object, "price", at, KindCodecInputViewPrice::decode_at)?;
        Ok(Self {
            efforts,
            api,
            output_limit,
            price,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.api {
            object.insert("api", held.encode());
        }
        if let Some(held) = &self.output_limit {
            object.insert("outputLimit", Value::Number(*held));
        }
        if let Some(held) = &self.price {
            object.insert("price", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindCodecInputMode {
    Stream,
    Complete,
}

impl KindCodecInputMode {
    pub const ALL: [Self; 2] = [Self::Stream, Self::Complete];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stream => "stream",
            Self::Complete => "complete",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KindCodecInputMode {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindCodecInputFailuresItem {
    pub status: i64,
    pub body: Value,
    pub headers: Option<Vec<(String, String)>>,
}

impl Shape for KindCodecInputFailuresItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let status = shape::required(object, "status", at, shape::integer)?;
        let body = shape::required(object, "body", at, shape::any)?;
        let headers = shape::optional(object, "headers", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::tuple(value, at, 2).and_then(|items| {
                    Ok((
                        shape::place(items, 0, at, shape::text)?,
                        shape::place(items, 1, at, shape::text)?,
                    ))
                })
            })
        })?;
        Ok(Self {
            status,
            body,
            headers,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("status", Value::Number(self.status as f64));
        object.insert("body", self.body.clone());
        if let Some(held) = &self.headers {
            object.insert(
                "headers",
                Value::Array(
                    held.iter()
                        .map(|held| {
                            Value::Array(vec![
                                Value::from(held.0.as_str()),
                                Value::from(held.1.as_str()),
                            ])
                        })
                        .collect(),
                ),
            );
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindCodecInputViewPrice {
    Free,
    Unknown,
}

impl KindCodecInputViewPrice {
    pub const ALL: [Self; 2] = [Self::Free, Self::Unknown];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KindCodecInputViewPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindCodecResult {
    pub encoded: Option<nyaaai_contract::shapes::step::Encoded>,
    pub refusal: Option<String>,
    pub facts: Option<KindCodecResultFacts>,
    pub failures: Vec<nyaaai_contract::shapes::step::Failure>,
}

impl Shape for KindCodecResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let encoded = shape::required(object, "encoded", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::step::Encoded::decode_at)
        })?;
        let refusal = shape::required(object, "refusal", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let facts = shape::required(object, "facts", at, |value, at| {
            shape::nullable(value, at, KindCodecResultFacts::decode_at)
        })?;
        let failures = shape::required(object, "failures", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Failure::decode_at)
        })?;
        Ok(Self {
            encoded,
            refusal,
            facts,
            failures,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "encoded",
            self.encoded
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "refusal",
            self.refusal
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "facts",
            self.facts
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "failures",
            Value::Array(self.failures.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindCodecResultFacts {
    pub efforts: Vec<nyaaai_contract::shapes::step::Effort>,
    pub free: bool,
    pub output_limit: Option<f64>,
}

impl Shape for KindCodecResultFacts {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
        })?;
        let free = shape::required(object, "free", at, shape::flag)?;
        let output_limit = shape::required(object, "outputLimit", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        Ok(Self {
            efforts,
            free,
            output_limit,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        object.insert("free", Value::Bool(self.free));
        object.insert(
            "outputLimit",
            self.output_limit
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindDispositionInput {
    pub kind: KindDispositionInputKind,
    pub rows: Vec<KindDispositionInputRowsItem>,
}

impl Shape for KindDispositionInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, KindDispositionInputKind::decode_at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, KindDispositionInputRowsItem::decode_at)
        })?;
        Ok(Self { kind, rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindDispositionInputKind {
    Openrouter,
    Opencode,
}

impl KindDispositionInputKind {
    pub const ALL: [Self; 2] = [Self::Openrouter, Self::Opencode];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openrouter => "openrouter",
            Self::Opencode => "opencode",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KindDispositionInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindDispositionInputRowsItem {
    pub failure: nyaaai_contract::shapes::step::Failure,
    pub status: Option<KindDispositionInputRowsItemStatus>,
}

impl Shape for KindDispositionInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let failure = shape::required(
            object,
            "failure",
            at,
            nyaaai_contract::shapes::step::Failure::decode_at,
        )?;
        let status = shape::required(object, "status", at, |value, at| {
            shape::nullable(value, at, KindDispositionInputRowsItemStatus::decode_at)
        })?;
        Ok(Self { failure, status })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("failure", self.failure.encode());
        object.insert(
            "status",
            self.status
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindDispositionInputRowsItemStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl KindDispositionInputRowsItemStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KindDispositionInputRowsItemStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindDispositionResult {
    pub dispositions: Vec<nyaaai_contract::shapes::provider::Disposition>,
}

impl Shape for KindDispositionResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let dispositions = shape::required(object, "dispositions", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::Disposition::decode_at,
            )
        })?;
        Ok(Self { dispositions })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "dispositions",
            Value::Array(self.dispositions.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteInput {
    pub kind: KindRouteInputKind,
    pub entry: KindRouteInputEntry,
    pub start_ms: i64,
    pub exchanges: Vec<NetExchange>,
    pub rows: Vec<String>,
    pub steps: Vec<KindRouteInputStepsItem>,
}

impl Shape for KindRouteInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, KindRouteInputKind::decode_at)?;
        let entry = shape::required(object, "entry", at, KindRouteInputEntry::decode_at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list_within(value, at, Some(1), None, KindRouteInputStepsItem::decode_at)
        })?;
        Ok(Self {
            kind,
            entry,
            start_ms,
            exchanges,
            rows,
            steps,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("entry", self.entry.encode());
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KindRouteInputKind {
    Openrouter,
    Opencode,
}

impl KindRouteInputKind {
    pub const ALL: [Self; 2] = [Self::Openrouter, Self::Opencode];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openrouter => "openrouter",
            Self::Opencode => "opencode",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for KindRouteInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteInputEntry {
    pub label: String,
    pub plan: Option<nyaaai_contract::shapes::provider::ProviderPlan>,
}

impl Shape for KindRouteInputEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let plan = shape::optional(
            object,
            "plan",
            at,
            nyaaai_contract::shapes::provider::ProviderPlan::decode_at,
        )?;
        Ok(Self { label, plan })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("label", Value::from(self.label.as_str()));
        if let Some(held) = &self.plan {
            object.insert("plan", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KindRouteInputStepsItem {
    Views(KindRouteInputStepsItemViews),
    List(KindRouteInputStepsItemList),
    Detail(KindRouteInputStepsItemDetail),
    Listed(KindRouteInputStepsItemListed),
    View(KindRouteInputStepsItemView),
    Advance(KindRouteInputStepsItemAdvance),
}

impl Shape for KindRouteInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => KindRouteInputStepsItemViews::decode_at(value, at).map(Self::Views),
            "list" => KindRouteInputStepsItemList::decode_at(value, at).map(Self::List),
            "detail" => KindRouteInputStepsItemDetail::decode_at(value, at).map(Self::Detail),
            "listed" => KindRouteInputStepsItemListed::decode_at(value, at).map(Self::Listed),
            "view" => KindRouteInputStepsItemView::decode_at(value, at).map(Self::View),
            "advance" => KindRouteInputStepsItemAdvance::decode_at(value, at).map(Self::Advance),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::List(held) => held.encode(),
            Self::Detail(held) => held.encode(),
            Self::Listed(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::Advance(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteInputStepsItemViews {
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub zdr_only: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for KindRouteInputStepsItemViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::optional(object, "zdrOnly", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            use_,
            zdr_only,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.zdr_only {
            object.insert("zdrOnly", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteInputStepsItemList {
    pub fresh: Option<bool>,
}

impl Shape for KindRouteInputStepsItemList {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "list")
        })?;
        let fresh = shape::optional(object, "fresh", at, shape::flag)?;
        Ok(Self { fresh })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("list"));
        if let Some(held) = &self.fresh {
            object.insert("fresh", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteInputStepsItemDetail {
    pub id: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for KindRouteInputStepsItemDetail {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "detail")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            id,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("detail"));
        object.insert("id", Value::from(self.id.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteInputStepsItemListed {
    pub id: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for KindRouteInputStepsItemListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "listed")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            id,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("listed"));
        object.insert("id", Value::from(self.id.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteInputStepsItemView {
    pub row: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for KindRouteInputStepsItemView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let row = shape::required(object, "row", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            row,
            use_,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("row", Value::from(self.row.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteInputStepsItemAdvance {
    pub ms: u64,
}

impl Shape for KindRouteInputStepsItemAdvance {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "advance")
        })?;
        let ms = shape::required(object, "ms", at, shape::natural)?;
        Ok(Self { ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("advance"));
        object.insert("ms", Value::Number(self.ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteResult {
    pub steps: Vec<KindRouteResultStepsItem>,
    pub requests: Vec<String>,
    pub read: Vec<i64>,
    pub waiting: Vec<String>,
}

impl Shape for KindRouteResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, KindRouteResultStepsItem::decode_at)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let read = shape::required(object, "read", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            steps,
            requests,
            read,
            waiting,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "requests",
            Value::Array(
                self.requests
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "read",
            Value::Array(
                self.read
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteResultStepsItem {
    pub now_ms: i64,
    pub answer: KindRouteResultStepsItemAnswer,
}

impl Shape for KindRouteResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let answer = shape::required(
            object,
            "answer",
            at,
            KindRouteResultStepsItemAnswer::decode_at,
        )?;
        Ok(Self { now_ms, answer })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert("answer", self.answer.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum KindRouteResultStepsItemAnswer {
    Views(KindRouteResultStepsItemAnswerViews),
    List(KindRouteResultStepsItemAnswerList),
    Detail(KindRouteResultStepsItemAnswerDetail),
    Listed(KindRouteResultStepsItemAnswerListed),
    View(KindRouteResultStepsItemAnswerView),
    Advanced(KindRouteResultStepsItemAnswerAdvanced),
    Thrown(KindRouteResultStepsItemAnswerThrown),
}

impl Shape for KindRouteResultStepsItemAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => KindRouteResultStepsItemAnswerViews::decode_at(value, at).map(Self::Views),
            "list" => KindRouteResultStepsItemAnswerList::decode_at(value, at).map(Self::List),
            "detail" => {
                KindRouteResultStepsItemAnswerDetail::decode_at(value, at).map(Self::Detail)
            }
            "listed" => {
                KindRouteResultStepsItemAnswerListed::decode_at(value, at).map(Self::Listed)
            }
            "view" => KindRouteResultStepsItemAnswerView::decode_at(value, at).map(Self::View),
            "advanced" => {
                KindRouteResultStepsItemAnswerAdvanced::decode_at(value, at).map(Self::Advanced)
            }
            "thrown" => {
                KindRouteResultStepsItemAnswerThrown::decode_at(value, at).map(Self::Thrown)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::List(held) => held.encode(),
            Self::Detail(held) => held.encode(),
            Self::Listed(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::Advanced(held) => held.encode(),
            Self::Thrown(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteResultStepsItemAnswerViews {
    pub views: Vec<ViewLine>,
}

impl Shape for KindRouteResultStepsItemAnswerViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let views = shape::required(object, "views", at, |value, at| {
            shape::list(value, at, ViewLine::decode_at)
        })?;
        Ok(Self { views })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        object.insert(
            "views",
            Value::Array(self.views.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteResultStepsItemAnswerList {
    pub models: Option<Vec<nyaaai_contract::shapes::model::BrowseModel>>,
}

impl Shape for KindRouteResultStepsItemAnswerList {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "list")
        })?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(
                    value,
                    at,
                    nyaaai_contract::shapes::model::BrowseModel::decode_at,
                )
            })
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("list"));
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteResultStepsItemAnswerDetail {
    pub detail: Option<nyaaai_contract::shapes::model::BrowseDetail>,
}

impl Shape for KindRouteResultStepsItemAnswerDetail {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "detail")
        })?;
        let detail = shape::required(object, "detail", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::model::BrowseDetail::decode_at,
            )
        })?;
        Ok(Self { detail })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("detail"));
        object.insert(
            "detail",
            self.detail
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteResultStepsItemAnswerListed {
    pub listed: Option<KindRouteResultStepsItemAnswerListedListed>,
}

impl Shape for KindRouteResultStepsItemAnswerListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "listed")
        })?;
        let listed = shape::required(object, "listed", at, |value, at| {
            shape::nullable(
                value,
                at,
                KindRouteResultStepsItemAnswerListedListed::decode_at,
            )
        })?;
        Ok(Self { listed })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("listed"));
        object.insert(
            "listed",
            self.listed
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteResultStepsItemAnswerView {
    pub view: ViewLine,
}

impl Shape for KindRouteResultStepsItemAnswerView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let view = shape::required(object, "view", at, ViewLine::decode_at)?;
        Ok(Self { view })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("view", self.view.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteResultStepsItemAnswerAdvanced {}

impl Shape for KindRouteResultStepsItemAnswerAdvanced {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "advanced")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("advanced"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteResultStepsItemAnswerThrown {
    pub message: String,
}

impl Shape for KindRouteResultStepsItemAnswerThrown {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "thrown")
        })?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self { message })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("thrown"));
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KindRouteResultStepsItemAnswerListedListed {
    pub label: String,
    pub reasoning: bool,
}

impl Shape for KindRouteResultStepsItemAnswerListedListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let reasoning = shape::required(object, "reasoning", at, shape::flag)?;
        Ok(Self { label, reasoning })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("reasoning", Value::Bool(self.reasoning));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServedModel {
    pub tag: String,
    pub tools: bool,
    pub thinking: bool,
    pub efforts: Vec<nyaaai_contract::shapes::step::Effort>,
    pub context: Option<f64>,
    pub loaded: Option<f64>,
    pub trained: Option<f64>,
    pub probed: Option<()>,
}

impl Shape for ServedModel {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let tools = shape::required(object, "tools", at, shape::flag)?;
        let thinking = shape::required(object, "thinking", at, shape::flag)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
        })?;
        let context = shape::optional(object, "context", at, shape::number)?;
        let loaded = shape::optional(object, "loaded", at, shape::number)?;
        let trained = shape::optional(object, "trained", at, shape::number)?;
        let probed = shape::optional(object, "probed", at, |value, at| {
            shape::is_flag(value, at, false)
        })?;
        Ok(Self {
            tag,
            tools,
            thinking,
            efforts,
            context,
            loaded,
            trained,
            probed,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("tag", Value::from(self.tag.as_str()));
        object.insert("tools", Value::Bool(self.tools));
        object.insert("thinking", Value::Bool(self.thinking));
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.context {
            object.insert("context", Value::Number(*held));
        }
        if let Some(held) = &self.loaded {
            object.insert("loaded", Value::Number(*held));
        }
        if let Some(held) = &self.trained {
            object.insert("trained", Value::Number(*held));
        }
        if self.probed.is_some() {
            object.insert("probed", Value::Bool(false));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServedTerms {
    pub kept: bool,
    pub price: ServedTermsPrice,
}

impl Shape for ServedTerms {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kept = shape::required(object, "kept", at, shape::flag)?;
        let price = shape::required(object, "price", at, ServedTermsPrice::decode_at)?;
        Ok(Self { kept, price })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kept", Value::Bool(self.kept));
        object.insert("price", self.price.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServedTermsPrice {
    Free,
    Unknown,
}

impl ServedTermsPrice {
    pub const ALL: [Self; 2] = [Self::Free, Self::Unknown];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ServedTermsPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalEntry {
    pub label: String,
    pub url: Option<String>,
}

impl Shape for LocalEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let url = shape::optional(object, "url", at, shape::text)?;
        Ok(Self { label, url })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("label", Value::from(self.label.as_str()));
        if let Some(held) = &self.url {
            object.insert("url", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalEffortsInput {
    pub values: Vec<LocalEffortsInputValuesItem>,
}

impl Shape for LocalEffortsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, LocalEffortsInputValuesItem::decode_at)
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(self.values.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocalEffortsInputValuesItem {
    Flag(bool),
    Text(String),
}

impl Shape for LocalEffortsInputValuesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = shape::flag(value, at) {
            return Ok(Self::Flag(held));
        }
        if let Ok(held) = shape::text(value, at) {
            return Ok(Self::Text(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Flag(held) => Value::Bool(*held),
            Self::Text(held) => Value::from(held.as_str()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalEffortsResult {
    pub efforts: Vec<nyaaai_contract::shapes::step::Effort>,
}

impl Shape for LocalEffortsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
        })?;
        Ok(Self { efforts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLabelInput {
    pub tags: Vec<String>,
}

impl Shape for LocalLabelInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tags = shape::required(object, "tags", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { tags })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "tags",
            Value::Array(
                self.tags
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLabelResult {
    pub labels: Vec<String>,
}

impl Shape for LocalLabelResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let labels = shape::required(object, "labels", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { labels })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "labels",
            Value::Array(
                self.labels
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalViewInput {
    pub row: String,
    pub model: ServedModel,
    pub loaded: Option<f64>,
    pub terms: ServedTerms,
    pub zdr_only: bool,
}

impl Shape for LocalViewInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let row = shape::required(object, "row", at, shape::text)?;
        let model = shape::required(object, "model", at, ServedModel::decode_at)?;
        let loaded = shape::optional(object, "loaded", at, shape::number)?;
        let terms = shape::required(object, "terms", at, ServedTerms::decode_at)?;
        let zdr_only = shape::required(object, "zdrOnly", at, shape::flag)?;
        Ok(Self {
            row,
            model,
            loaded,
            terms,
            zdr_only,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("row", Value::from(self.row.as_str()));
        object.insert("model", self.model.encode());
        if let Some(held) = &self.loaded {
            object.insert("loaded", Value::Number(*held));
        }
        object.insert("terms", self.terms.encode());
        object.insert("zdrOnly", Value::Bool(self.zdr_only));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalViewResult {
    pub view: ViewLine,
    pub context: Option<f64>,
}

impl Shape for LocalViewResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let view = shape::required(object, "view", at, ViewLine::decode_at)?;
        let context = shape::required(object, "context", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        Ok(Self { view, context })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("view", self.view.encode());
        object.insert(
            "context",
            self.context
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalOllamaTagsInput {
    pub body: Value,
}

impl Shape for LocalOllamaTagsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalOllamaTagsResult {
    pub tags: Option<Vec<String>>,
}

impl Shape for LocalOllamaTagsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tags = shape::required(object, "tags", at, |value, at| {
            shape::nullable(value, at, |value, at| shape::list(value, at, shape::text))
        })?;
        Ok(Self { tags })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "tags",
            self.tags.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalOllamaShowInput {
    pub tag: String,
    pub body: Value,
}

impl Shape for LocalOllamaShowInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { tag, body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("tag", Value::from(self.tag.as_str()));
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalOllamaShowResult {
    pub model: Option<ServedModel>,
}

impl Shape for LocalOllamaShowResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let model = shape::required(object, "model", at, |value, at| {
            shape::nullable(value, at, ServedModel::decode_at)
        })?;
        Ok(Self { model })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "model",
            self.model
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalOllamaNumCtxInput {
    pub texts: Vec<String>,
}

impl Shape for LocalOllamaNumCtxInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let texts = shape::required(object, "texts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { texts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "texts",
            Value::Array(
                self.texts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalOllamaNumCtxResult {
    pub contexts: Vec<Option<f64>>,
}

impl Shape for LocalOllamaNumCtxResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let contexts = shape::required(object, "contexts", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::number)
            })
        })?;
        Ok(Self { contexts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "contexts",
            Value::Array(
                self.contexts
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalOllamaRunningInput {
    pub body: Value,
}

impl Shape for LocalOllamaRunningInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalOllamaRunningResult {
    pub contexts: Vec<LocalOllamaRunningResultContextsItem>,
}

impl Shape for LocalOllamaRunningResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let contexts = shape::required(object, "contexts", at, |value, at| {
            shape::list(value, at, LocalOllamaRunningResultContextsItem::decode_at)
        })?;
        Ok(Self { contexts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "contexts",
            Value::Array(self.contexts.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalOllamaRunningResultContextsItem {
    pub tag: String,
    pub context: f64,
}

impl Shape for LocalOllamaRunningResultContextsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let context = shape::required(object, "context", at, shape::number)?;
        Ok(Self { tag, context })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("tag", Value::from(self.tag.as_str()));
        object.insert("context", Value::Number(self.context));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLmstudioModelsInput {
    pub body: Value,
}

impl Shape for LocalLmstudioModelsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLmstudioModelsResult {
    pub models: Option<Vec<ServedModel>>,
}

impl Shape for LocalLmstudioModelsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(value, at, ServedModel::decode_at)
            })
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLmstudioLoadedInput {
    pub body: Value,
}

impl Shape for LocalLmstudioLoadedInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLmstudioLoadedResult {
    pub contexts: Vec<LocalLmstudioLoadedResultContextsItem>,
}

impl Shape for LocalLmstudioLoadedResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let contexts = shape::required(object, "contexts", at, |value, at| {
            shape::list(value, at, LocalLmstudioLoadedResultContextsItem::decode_at)
        })?;
        Ok(Self { contexts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "contexts",
            Value::Array(self.contexts.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLmstudioLoadedResultContextsItem {
    pub tag: String,
    pub context: f64,
}

impl Shape for LocalLmstudioLoadedResultContextsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let context = shape::required(object, "context", at, shape::number)?;
        Ok(Self { tag, context })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("tag", Value::from(self.tag.as_str()));
        object.insert("context", Value::Number(self.context));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLlamacppModelsInput {
    pub body: Value,
}

impl Shape for LocalLlamacppModelsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLlamacppModelsResult {
    pub models: Option<Vec<LocalLlamacppModelsResultModelsItem>>,
}

impl Shape for LocalLlamacppModelsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(value, at, LocalLlamacppModelsResultModelsItem::decode_at)
            })
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLlamacppModelsResultModelsItem {
    pub id: String,
    pub trained: Option<f64>,
}

impl Shape for LocalLlamacppModelsResultModelsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let trained = shape::required(object, "trained", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        Ok(Self { id, trained })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert(
            "trained",
            self.trained
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLlamacppPropsInput {
    pub body: Value,
}

impl Shape for LocalLlamacppPropsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLlamacppPropsResult {
    pub props: Option<LocalLlamacppPropsResultProps>,
}

impl Shape for LocalLlamacppPropsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let props = shape::required(object, "props", at, |value, at| {
            shape::nullable(value, at, LocalLlamacppPropsResultProps::decode_at)
        })?;
        Ok(Self { props })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "props",
            self.props
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLlamacppPropsResultProps {
    pub context: Option<f64>,
    pub efforts: Vec<String>,
}

impl Shape for LocalLlamacppPropsResultProps {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let context = shape::required(object, "context", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { context, efforts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "context",
            self.context
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert(
            "efforts",
            Value::Array(
                self.efforts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLlamacppModelInput {
    pub id: String,
    pub trained: Option<f64>,
    pub props: Option<LocalLlamacppModelInputProps>,
    pub known: Option<ServedModel>,
}

impl Shape for LocalLlamacppModelInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let trained = shape::optional(object, "trained", at, shape::number)?;
        let props = shape::optional(object, "props", at, LocalLlamacppModelInputProps::decode_at)?;
        let known = shape::optional(object, "known", at, ServedModel::decode_at)?;
        Ok(Self {
            id,
            trained,
            props,
            known,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        if let Some(held) = &self.trained {
            object.insert("trained", Value::Number(*held));
        }
        if let Some(held) = &self.props {
            object.insert("props", held.encode());
        }
        if let Some(held) = &self.known {
            object.insert("known", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLlamacppModelInputProps {
    pub context: Option<f64>,
    pub efforts: Vec<String>,
}

impl Shape for LocalLlamacppModelInputProps {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let context = shape::optional(object, "context", at, shape::number)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { context, efforts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.context {
            object.insert("context", Value::Number(*held));
        }
        object.insert(
            "efforts",
            Value::Array(
                self.efforts
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalLlamacppModelResult {
    pub model: ServedModel,
}

impl Shape for LocalLlamacppModelResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let model = shape::required(object, "model", at, ServedModel::decode_at)?;
        Ok(Self { model })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("model", self.model.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCompatibleModelsInput {
    pub body: Value,
}

impl Shape for LocalCompatibleModelsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCompatibleModelsResult {
    pub models: Option<Vec<ServedModel>>,
}

impl Shape for LocalCompatibleModelsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(value, at, ServedModel::decode_at)
            })
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCompatibleTermsInput {
    pub urls: Vec<Option<String>>,
}

impl Shape for LocalCompatibleTermsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let urls = shape::required(object, "urls", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { urls })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "urls",
            Value::Array(
                self.urls
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCompatibleTermsResult {
    pub terms: Vec<LocalCompatibleTermsResultTermsItem>,
}

impl Shape for LocalCompatibleTermsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let terms = shape::required(object, "terms", at, |value, at| {
            shape::list(value, at, LocalCompatibleTermsResultTermsItem::decode_at)
        })?;
        Ok(Self { terms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "terms",
            Value::Array(self.terms.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCompatibleTermsResultTermsItem {
    pub remote: bool,
    pub kept: bool,
    pub price: LocalCompatibleTermsResultTermsItemPrice,
}

impl Shape for LocalCompatibleTermsResultTermsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let remote = shape::required(object, "remote", at, shape::flag)?;
        let kept = shape::required(object, "kept", at, shape::flag)?;
        let price = shape::required(
            object,
            "price",
            at,
            LocalCompatibleTermsResultTermsItemPrice::decode_at,
        )?;
        Ok(Self {
            remote,
            kept,
            price,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("remote", Value::Bool(self.remote));
        object.insert("kept", Value::Bool(self.kept));
        object.insert("price", self.price.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalCompatibleTermsResultTermsItemPrice {
    Free,
    Unknown,
}

impl LocalCompatibleTermsResultTermsItemPrice {
    pub const ALL: [Self; 2] = [Self::Free, Self::Unknown];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalCompatibleTermsResultTermsItemPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalReachInput {
    pub kind: LocalReachInputKind,
    pub entry: LocalEntry,
    pub secret: Option<String>,
    pub start_ms: i64,
    pub exchanges: Vec<NetExchange>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalReachInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, LocalReachInputKind::decode_at)?;
        let entry = shape::required(object, "entry", at, LocalEntry::decode_at)?;
        let secret = shape::required(object, "secret", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            kind,
            entry,
            secret,
            start_ms,
            exchanges,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("entry", self.entry.encode());
        object.insert(
            "secret",
            self.secret
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalReachInputKind {
    Ollama,
    Lmstudio,
    Llamacpp,
    Compatible,
}

impl LocalReachInputKind {
    pub const ALL: [Self; 4] = [
        Self::Ollama,
        Self::Lmstudio,
        Self::Llamacpp,
        Self::Compatible,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::Lmstudio => "lmstudio",
            Self::Llamacpp => "llamacpp",
            Self::Compatible => "compatible",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalReachInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalReachResult {
    pub state: Option<nyaaai_contract::shapes::provider::KeyState>,
    pub thrown: Option<String>,
    pub facts: Option<String>,
    pub requests: Vec<LocalReachResultRequestsItem>,
    pub waiting: Vec<String>,
}

impl Shape for LocalReachResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let state = shape::required(object, "state", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::provider::KeyState::decode_at,
            )
        })?;
        let thrown = shape::required(object, "thrown", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let facts = shape::required(object, "facts", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, LocalReachResultRequestsItem::decode_at)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            state,
            thrown,
            facts,
            requests,
            waiting,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "state",
            self.state
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "thrown",
            self.thrown
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "facts",
            self.facts
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "requests",
            Value::Array(self.requests.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalReachResultRequestsItem {
    pub method: String,
    pub url: String,
    pub authorization: Option<String>,
    pub body: Value,
}

impl Shape for LocalReachResultRequestsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let method = shape::required(object, "method", at, shape::text)?;
        let url = shape::required(object, "url", at, shape::text)?;
        let authorization = shape::required(object, "authorization", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self {
            method,
            url,
            authorization,
            body,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("method", Value::from(self.method.as_str()));
        object.insert("url", Value::from(self.url.as_str()));
        object.insert(
            "authorization",
            self.authorization
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalServerInput {
    pub kind: LocalServerInputKind,
    pub entry: LocalEntry,
    pub stored: Option<String>,
    pub start_ms: i64,
    pub exchanges: Vec<NetExchange>,
    pub steps: Vec<LocalServerInputStepsItem>,
}

impl Shape for LocalServerInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, LocalServerInputKind::decode_at)?;
        let entry = shape::required(object, "entry", at, LocalEntry::decode_at)?;
        let stored = shape::required(object, "stored", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list_within(
                value,
                at,
                Some(1),
                None,
                LocalServerInputStepsItem::decode_at,
            )
        })?;
        Ok(Self {
            kind,
            entry,
            stored,
            start_ms,
            exchanges,
            steps,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("entry", self.entry.encode());
        object.insert(
            "stored",
            self.stored
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalServerInputKind {
    Ollama,
    Lmstudio,
    Llamacpp,
    Compatible,
}

impl LocalServerInputKind {
    pub const ALL: [Self; 4] = [
        Self::Ollama,
        Self::Lmstudio,
        Self::Llamacpp,
        Self::Compatible,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::Lmstudio => "lmstudio",
            Self::Llamacpp => "llamacpp",
            Self::Compatible => "compatible",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalServerInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocalServerInputStepsItem {
    Models(LocalServerInputStepsItemModels),
    Loaded(LocalServerInputStepsItemLoaded),
    Warm(LocalServerInputStepsItemWarm),
}

impl Shape for LocalServerInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "models" => LocalServerInputStepsItemModels::decode_at(value, at).map(Self::Models),
            "loaded" => LocalServerInputStepsItemLoaded::decode_at(value, at).map(Self::Loaded),
            "warm" => LocalServerInputStepsItemWarm::decode_at(value, at).map(Self::Warm),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Models(held) => held.encode(),
            Self::Loaded(held) => held.encode(),
            Self::Warm(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalServerInputStepsItemModels {
    pub known: Vec<ServedModel>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalServerInputStepsItemModels {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "models")
        })?;
        let known = shape::required(object, "known", at, |value, at| {
            shape::list(value, at, ServedModel::decode_at)
        })?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            known,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("models"));
        object.insert(
            "known",
            Value::Array(self.known.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalServerInputStepsItemLoaded {
    pub tag: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalServerInputStepsItemLoaded {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "loaded")
        })?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            tag,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("loaded"));
        object.insert("tag", Value::from(self.tag.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalServerInputStepsItemWarm {
    pub tag: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalServerInputStepsItemWarm {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "warm")
        })?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            tag,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("warm"));
        object.insert("tag", Value::from(self.tag.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalServerResult {
    pub steps: Vec<LocalServerResultStepsItem>,
    pub requests: Vec<LocalServerResultRequestsItem>,
    pub read: Vec<i64>,
    pub waiting: Vec<String>,
}

impl Shape for LocalServerResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, LocalServerResultStepsItem::decode_at)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, LocalServerResultRequestsItem::decode_at)
        })?;
        let read = shape::required(object, "read", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            steps,
            requests,
            read,
            waiting,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "requests",
            Value::Array(self.requests.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "read",
            Value::Array(
                self.read
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalServerResultStepsItem {
    pub models: Option<Vec<ServedModel>>,
    pub contexts: Option<Vec<LocalServerResultStepsItemContextsItem>>,
    pub done: bool,
    pub thrown: Option<String>,
}

impl Shape for LocalServerResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(value, at, ServedModel::decode_at)
            })
        })?;
        let contexts = shape::required(object, "contexts", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(value, at, LocalServerResultStepsItemContextsItem::decode_at)
            })
        })?;
        let done = shape::required(object, "done", at, shape::flag)?;
        let thrown = shape::required(object, "thrown", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            models,
            contexts,
            done,
            thrown,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        object.insert(
            "contexts",
            self.contexts.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        object.insert("done", Value::Bool(self.done));
        object.insert(
            "thrown",
            self.thrown
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalServerResultRequestsItem {
    pub method: String,
    pub url: String,
    pub authorization: Option<String>,
    pub body: Value,
}

impl Shape for LocalServerResultRequestsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let method = shape::required(object, "method", at, shape::text)?;
        let url = shape::required(object, "url", at, shape::text)?;
        let authorization = shape::required(object, "authorization", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self {
            method,
            url,
            authorization,
            body,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("method", Value::from(self.method.as_str()));
        object.insert("url", Value::from(self.url.as_str()));
        object.insert(
            "authorization",
            self.authorization
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalServerResultStepsItemContextsItem {
    pub tag: String,
    pub context: f64,
}

impl Shape for LocalServerResultStepsItemContextsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let context = shape::required(object, "context", at, shape::number)?;
        Ok(Self { tag, context })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("tag", Value::from(self.tag.as_str()));
        object.insert("context", Value::Number(self.context));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInput {
    pub start_ms: i64,
    pub entry: String,
    pub terms: ServedTerms,
    pub server: LocalCatalogRunInputServer,
    pub rows: Vec<String>,
    pub on_refreshed: Option<LocalCatalogRunInputOnRefreshed>,
    pub steps: Vec<LocalCatalogRunInputStepsItem>,
}

impl Shape for LocalCatalogRunInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let terms = shape::required(object, "terms", at, ServedTerms::decode_at)?;
        let server = shape::required(object, "server", at, LocalCatalogRunInputServer::decode_at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let on_refreshed = shape::optional(
            object,
            "onRefreshed",
            at,
            LocalCatalogRunInputOnRefreshed::decode_at,
        )?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list_within(
                value,
                at,
                Some(1),
                None,
                LocalCatalogRunInputStepsItem::decode_at,
            )
        })?;
        Ok(Self {
            start_ms,
            entry,
            terms,
            server,
            rows,
            on_refreshed,
            steps,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("terms", self.terms.encode());
        object.insert("server", self.server.encode());
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        if let Some(held) = &self.on_refreshed {
            object.insert("onRefreshed", held.encode());
        }
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputServer {
    pub live: bool,
    pub loaded: bool,
    pub warm: bool,
    pub models: Vec<LocalCatalogRunInputServerModelsItem>,
    pub loads: Vec<LocalCatalogRunInputServerLoadsItem>,
    pub warms: Vec<LocalCatalogRunInputServerWarmsItem>,
}

impl Shape for LocalCatalogRunInputServer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let live = shape::required(object, "live", at, shape::flag)?;
        let loaded = shape::required(object, "loaded", at, shape::flag)?;
        let warm = shape::required(object, "warm", at, shape::flag)?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::list(value, at, LocalCatalogRunInputServerModelsItem::decode_at)
        })?;
        let loads = shape::required(object, "loads", at, |value, at| {
            shape::list(value, at, LocalCatalogRunInputServerLoadsItem::decode_at)
        })?;
        let warms = shape::required(object, "warms", at, |value, at| {
            shape::list(value, at, LocalCatalogRunInputServerWarmsItem::decode_at)
        })?;
        Ok(Self {
            live,
            loaded,
            warm,
            models,
            loads,
            warms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("live", Value::Bool(self.live));
        object.insert("loaded", Value::Bool(self.loaded));
        object.insert("warm", Value::Bool(self.warm));
        object.insert(
            "models",
            Value::Array(self.models.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "loads",
            Value::Array(self.loads.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "warms",
            Value::Array(self.warms.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalCatalogRunInputOnRefreshed {
    Reads,
    ReadsAndThrows,
}

impl LocalCatalogRunInputOnRefreshed {
    pub const ALL: [Self; 2] = [Self::Reads, Self::ReadsAndThrows];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reads => "reads",
            Self::ReadsAndThrows => "readsAndThrows",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalCatalogRunInputOnRefreshed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocalCatalogRunInputStepsItem {
    Call(LocalCatalogRunInputStepsItemCall),
    Together(LocalCatalogRunInputStepsItemTogether),
    Settled(LocalCatalogRunInputStepsItemSettled),
    Rediscover(LocalCatalogRunInputStepsItemRediscover),
    Advance(LocalCatalogRunInputStepsItemAdvance),
    Rows(LocalCatalogRunInputStepsItemRows),
}

impl Shape for LocalCatalogRunInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "call" => LocalCatalogRunInputStepsItemCall::decode_at(value, at).map(Self::Call),
            "together" => {
                LocalCatalogRunInputStepsItemTogether::decode_at(value, at).map(Self::Together)
            }
            "settled" => {
                LocalCatalogRunInputStepsItemSettled::decode_at(value, at).map(Self::Settled)
            }
            "rediscover" => {
                LocalCatalogRunInputStepsItemRediscover::decode_at(value, at).map(Self::Rediscover)
            }
            "advance" => {
                LocalCatalogRunInputStepsItemAdvance::decode_at(value, at).map(Self::Advance)
            }
            "rows" => LocalCatalogRunInputStepsItemRows::decode_at(value, at).map(Self::Rows),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Call(held) => held.encode(),
            Self::Together(held) => held.encode(),
            Self::Settled(held) => held.encode(),
            Self::Rediscover(held) => held.encode(),
            Self::Advance(held) => held.encode(),
            Self::Rows(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputServerModelsItem {
    pub models: Vec<ServedModel>,
    pub delay_ms: Option<i64>,
    pub fails: Option<()>,
}

impl Shape for LocalCatalogRunInputServerModelsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::list(value, at, ServedModel::decode_at)
        })?;
        let delay_ms = shape::optional(object, "delayMs", at, shape::integer)?;
        let fails = shape::optional(object, "fails", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        Ok(Self {
            models,
            delay_ms,
            fails,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "models",
            Value::Array(self.models.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.delay_ms {
            object.insert("delayMs", Value::Number(*held as f64));
        }
        if self.fails.is_some() {
            object.insert("fails", Value::Bool(true));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputServerLoadsItem {
    pub contexts: Vec<LocalCatalogRunInputServerLoadsItemContextsItem>,
    pub delay_ms: Option<i64>,
    pub fails: Option<()>,
}

impl Shape for LocalCatalogRunInputServerLoadsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let contexts = shape::required(object, "contexts", at, |value, at| {
            shape::list(
                value,
                at,
                LocalCatalogRunInputServerLoadsItemContextsItem::decode_at,
            )
        })?;
        let delay_ms = shape::optional(object, "delayMs", at, shape::integer)?;
        let fails = shape::optional(object, "fails", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        Ok(Self {
            contexts,
            delay_ms,
            fails,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "contexts",
            Value::Array(self.contexts.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.delay_ms {
            object.insert("delayMs", Value::Number(*held as f64));
        }
        if self.fails.is_some() {
            object.insert("fails", Value::Bool(true));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputServerWarmsItem {
    pub delay_ms: Option<i64>,
    pub fails: Option<()>,
}

impl Shape for LocalCatalogRunInputServerWarmsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let delay_ms = shape::optional(object, "delayMs", at, shape::integer)?;
        let fails = shape::optional(object, "fails", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        Ok(Self { delay_ms, fails })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.delay_ms {
            object.insert("delayMs", Value::Number(*held as f64));
        }
        if self.fails.is_some() {
            object.insert("fails", Value::Bool(true));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemCall {
    pub call: LocalCatalogRunInputStepsItemCallCall,
}

impl Shape for LocalCatalogRunInputStepsItemCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "call")
        })?;
        let call = shape::required(
            object,
            "call",
            at,
            LocalCatalogRunInputStepsItemCallCall::decode_at,
        )?;
        Ok(Self { call })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("call"));
        object.insert("call", self.call.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemTogether {
    pub calls: Vec<LocalCatalogRunInputStepsItemTogetherCallsItem>,
}

impl Shape for LocalCatalogRunInputStepsItemTogether {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "together")
        })?;
        let calls = shape::required(object, "calls", at, |value, at| {
            shape::list_within(
                value,
                at,
                Some(2),
                None,
                LocalCatalogRunInputStepsItemTogetherCallsItem::decode_at,
            )
        })?;
        Ok(Self { calls })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("together"));
        object.insert(
            "calls",
            Value::Array(self.calls.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemSettled {
    pub tag: String,
    pub times: Option<u64>,
}

impl Shape for LocalCatalogRunInputStepsItemSettled {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "settled")
        })?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let times = shape::optional(object, "times", at, |value, at| {
            shape::natural(value, at).and_then(|held| shape::at_least(held, 1, at))
        })?;
        Ok(Self { tag, times })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("settled"));
        object.insert("tag", Value::from(self.tag.as_str()));
        if let Some(held) = &self.times {
            object.insert("times", Value::Number(*held as f64));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemRediscover {}

impl Shape for LocalCatalogRunInputStepsItemRediscover {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "rediscover")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("rediscover"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemAdvance {
    pub ms: u64,
}

impl Shape for LocalCatalogRunInputStepsItemAdvance {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "advance")
        })?;
        let ms = shape::required(object, "ms", at, shape::natural)?;
        Ok(Self { ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("advance"));
        object.insert("ms", Value::Number(self.ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemRows {
    pub rows: Vec<String>,
}

impl Shape for LocalCatalogRunInputStepsItemRows {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "rows")
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("rows"));
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputServerLoadsItemContextsItem {
    pub tag: String,
    pub context: f64,
}

impl Shape for LocalCatalogRunInputServerLoadsItemContextsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let context = shape::required(object, "context", at, shape::number)?;
        Ok(Self { tag, context })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("tag", Value::from(self.tag.as_str()));
        object.insert("context", Value::Number(self.context));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocalCatalogRunInputStepsItemCallCall {
    Views(LocalCatalogRunInputStepsItemCallCallViews),
    Discovered(LocalCatalogRunInputStepsItemCallCallDiscovered),
    ViewFor(LocalCatalogRunInputStepsItemCallCallViewFor),
    Warmed(LocalCatalogRunInputStepsItemCallCallWarmed),
}

impl Shape for LocalCatalogRunInputStepsItemCallCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => {
                LocalCatalogRunInputStepsItemCallCallViews::decode_at(value, at).map(Self::Views)
            }
            "discovered" => LocalCatalogRunInputStepsItemCallCallDiscovered::decode_at(value, at)
                .map(Self::Discovered),
            "viewFor" => LocalCatalogRunInputStepsItemCallCallViewFor::decode_at(value, at)
                .map(Self::ViewFor),
            "warmed" => {
                LocalCatalogRunInputStepsItemCallCallWarmed::decode_at(value, at).map(Self::Warmed)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::Discovered(held) => held.encode(),
            Self::ViewFor(held) => held.encode(),
            Self::Warmed(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocalCatalogRunInputStepsItemTogetherCallsItem {
    Views(LocalCatalogRunInputStepsItemTogetherCallsItemViews),
    Discovered(LocalCatalogRunInputStepsItemTogetherCallsItemDiscovered),
    ViewFor(LocalCatalogRunInputStepsItemTogetherCallsItemViewFor),
    Warmed(LocalCatalogRunInputStepsItemTogetherCallsItemWarmed),
}

impl Shape for LocalCatalogRunInputStepsItemTogetherCallsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => LocalCatalogRunInputStepsItemTogetherCallsItemViews::decode_at(value, at)
                .map(Self::Views),
            "discovered" => {
                LocalCatalogRunInputStepsItemTogetherCallsItemDiscovered::decode_at(value, at)
                    .map(Self::Discovered)
            }
            "viewFor" => {
                LocalCatalogRunInputStepsItemTogetherCallsItemViewFor::decode_at(value, at)
                    .map(Self::ViewFor)
            }
            "warmed" => LocalCatalogRunInputStepsItemTogetherCallsItemWarmed::decode_at(value, at)
                .map(Self::Warmed),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::Discovered(held) => held.encode(),
            Self::ViewFor(held) => held.encode(),
            Self::Warmed(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemCallCallViews {
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub zdr_only: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalCatalogRunInputStepsItemCallCallViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::optional(object, "zdrOnly", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            use_,
            zdr_only,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.zdr_only {
            object.insert("zdrOnly", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemCallCallDiscovered {
    pub fresh: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalCatalogRunInputStepsItemCallCallDiscovered {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "discovered")
        })?;
        let fresh = shape::optional(object, "fresh", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            fresh,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("discovered"));
        if let Some(held) = &self.fresh {
            object.insert("fresh", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemCallCallViewFor {
    pub row: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalCatalogRunInputStepsItemCallCallViewFor {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "viewFor")
        })?;
        let row = shape::required(object, "row", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            row,
            use_,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("viewFor"));
        object.insert("row", Value::from(self.row.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemCallCallWarmed {
    pub tag: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalCatalogRunInputStepsItemCallCallWarmed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "warmed")
        })?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            tag,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("warmed"));
        object.insert("tag", Value::from(self.tag.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemTogetherCallsItemViews {
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub zdr_only: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalCatalogRunInputStepsItemTogetherCallsItemViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::optional(object, "zdrOnly", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            use_,
            zdr_only,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.zdr_only {
            object.insert("zdrOnly", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemTogetherCallsItemDiscovered {
    pub fresh: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalCatalogRunInputStepsItemTogetherCallsItemDiscovered {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "discovered")
        })?;
        let fresh = shape::optional(object, "fresh", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            fresh,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("discovered"));
        if let Some(held) = &self.fresh {
            object.insert("fresh", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemTogetherCallsItemViewFor {
    pub row: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalCatalogRunInputStepsItemTogetherCallsItemViewFor {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "viewFor")
        })?;
        let row = shape::required(object, "row", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            row,
            use_,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("viewFor"));
        object.insert("row", Value::from(self.row.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunInputStepsItemTogetherCallsItemWarmed {
    pub tag: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalCatalogRunInputStepsItemTogetherCallsItemWarmed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "warmed")
        })?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            tag,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("warmed"));
        object.insert("tag", Value::from(self.tag.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunResult {
    pub steps: Vec<LocalCatalogRunResultStepsItem>,
    pub waiting: Vec<String>,
    pub saw: Vec<String>,
}

impl Shape for LocalCatalogRunResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, LocalCatalogRunResultStepsItem::decode_at)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let saw = shape::required(object, "saw", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            steps,
            waiting,
            saw,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "saw",
            Value::Array(
                self.saw
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunResultStepsItem {
    pub now_ms: i64,
    pub answers: Vec<LocalCatalogRunResultStepsItemAnswersItem>,
    pub served: Vec<LocalCatalogRunResultStepsItemServedItem>,
    pub refreshed: i64,
}

impl Shape for LocalCatalogRunResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let answers = shape::required(object, "answers", at, |value, at| {
            shape::list(
                value,
                at,
                LocalCatalogRunResultStepsItemAnswersItem::decode_at,
            )
        })?;
        let served = shape::required(object, "served", at, |value, at| {
            shape::list(
                value,
                at,
                LocalCatalogRunResultStepsItemServedItem::decode_at,
            )
        })?;
        let refreshed = shape::required(object, "refreshed", at, shape::integer)?;
        Ok(Self {
            now_ms,
            answers,
            served,
            refreshed,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert(
            "answers",
            Value::Array(self.answers.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "served",
            Value::Array(self.served.iter().map(|held| held.encode()).collect()),
        );
        object.insert("refreshed", Value::Number(self.refreshed as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocalCatalogRunResultStepsItemAnswersItem {
    Views(LocalCatalogRunResultStepsItemAnswersItemViews),
    Discovered(LocalCatalogRunResultStepsItemAnswersItemDiscovered),
    View(LocalCatalogRunResultStepsItemAnswersItemView),
    Warmed(LocalCatalogRunResultStepsItemAnswersItemWarmed),
    Thrown(LocalCatalogRunResultStepsItemAnswersItemThrown),
}

impl Shape for LocalCatalogRunResultStepsItemAnswersItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => LocalCatalogRunResultStepsItemAnswersItemViews::decode_at(value, at)
                .map(Self::Views),
            "discovered" => {
                LocalCatalogRunResultStepsItemAnswersItemDiscovered::decode_at(value, at)
                    .map(Self::Discovered)
            }
            "view" => {
                LocalCatalogRunResultStepsItemAnswersItemView::decode_at(value, at).map(Self::View)
            }
            "warmed" => LocalCatalogRunResultStepsItemAnswersItemWarmed::decode_at(value, at)
                .map(Self::Warmed),
            "thrown" => LocalCatalogRunResultStepsItemAnswersItemThrown::decode_at(value, at)
                .map(Self::Thrown),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::Discovered(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::Warmed(held) => held.encode(),
            Self::Thrown(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunResultStepsItemServedItem {
    pub method: LocalCatalogRunResultStepsItemServedItemMethod,
    pub tag: Option<String>,
    pub known: Vec<String>,
    pub start_ms: i64,
    pub end_ms: i64,
    pub aborted: bool,
}

impl Shape for LocalCatalogRunResultStepsItemServedItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let method = shape::required(
            object,
            "method",
            at,
            LocalCatalogRunResultStepsItemServedItemMethod::decode_at,
        )?;
        let tag = shape::required(object, "tag", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let known = shape::required(object, "known", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let end_ms = shape::required(object, "endMs", at, shape::integer)?;
        let aborted = shape::required(object, "aborted", at, shape::flag)?;
        Ok(Self {
            method,
            tag,
            known,
            start_ms,
            end_ms,
            aborted,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("method", self.method.encode());
        object.insert(
            "tag",
            self.tag
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "known",
            Value::Array(
                self.known
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert("endMs", Value::Number(self.end_ms as f64));
        object.insert("aborted", Value::Bool(self.aborted));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunResultStepsItemAnswersItemViews {
    pub views: Vec<ViewLine>,
}

impl Shape for LocalCatalogRunResultStepsItemAnswersItemViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let views = shape::required(object, "views", at, |value, at| {
            shape::list(value, at, ViewLine::decode_at)
        })?;
        Ok(Self { views })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        object.insert(
            "views",
            Value::Array(self.views.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunResultStepsItemAnswersItemDiscovered {
    pub models: Option<Vec<LocalCatalogRunResultStepsItemAnswersItemDiscoveredModelsItem>>,
}

impl Shape for LocalCatalogRunResultStepsItemAnswersItemDiscovered {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "discovered")
        })?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(
                    value,
                    at,
                    LocalCatalogRunResultStepsItemAnswersItemDiscoveredModelsItem::decode_at,
                )
            })
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("discovered"));
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunResultStepsItemAnswersItemView {
    pub view: ViewLine,
}

impl Shape for LocalCatalogRunResultStepsItemAnswersItemView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let view = shape::required(object, "view", at, ViewLine::decode_at)?;
        Ok(Self { view })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("view", self.view.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunResultStepsItemAnswersItemWarmed {}

impl Shape for LocalCatalogRunResultStepsItemAnswersItemWarmed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "warmed")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("warmed"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunResultStepsItemAnswersItemThrown {
    pub message: String,
}

impl Shape for LocalCatalogRunResultStepsItemAnswersItemThrown {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "thrown")
        })?;
        let message = shape::required(object, "message", at, shape::text)?;
        Ok(Self { message })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("thrown"));
        object.insert("message", Value::from(self.message.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalCatalogRunResultStepsItemServedItemMethod {
    Models,
    Loaded,
    Warm,
}

impl LocalCatalogRunResultStepsItemServedItemMethod {
    pub const ALL: [Self; 3] = [Self::Models, Self::Loaded, Self::Warm];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Models => "models",
            Self::Loaded => "loaded",
            Self::Warm => "warm",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalCatalogRunResultStepsItemServedItemMethod {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalCatalogRunResultStepsItemAnswersItemDiscoveredModelsItem {
    pub key: String,
    pub display: String,
    pub tag: String,
    pub tools: bool,
    pub reasoning: bool,
    pub context: Option<f64>,
}

impl Shape for LocalCatalogRunResultStepsItemAnswersItemDiscoveredModelsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let key = shape::required(object, "key", at, shape::text)?;
        let display = shape::required(object, "display", at, shape::text)?;
        let tag = shape::required(object, "tag", at, shape::text)?;
        let tools = shape::required(object, "tools", at, shape::flag)?;
        let reasoning = shape::required(object, "reasoning", at, shape::flag)?;
        let context = shape::optional(object, "context", at, shape::number)?;
        Ok(Self {
            key,
            display,
            tag,
            tools,
            reasoning,
            context,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("key", Value::from(self.key.as_str()));
        object.insert("display", Value::from(self.display.as_str()));
        object.insert("tag", Value::from(self.tag.as_str()));
        object.insert("tools", Value::Bool(self.tools));
        object.insert("reasoning", Value::Bool(self.reasoning));
        if let Some(held) = &self.context {
            object.insert("context", Value::Number(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowInput {
    pub kind: LocalFlowInputKind,
    pub entry: LocalFlowInputEntry,
    pub stored: Option<String>,
    pub start_ms: i64,
    pub exchanges: Vec<NetExchange>,
    pub rows: Vec<String>,
    pub steps: Vec<LocalFlowInputStepsItem>,
}

impl Shape for LocalFlowInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, LocalFlowInputKind::decode_at)?;
        let entry = shape::required(object, "entry", at, LocalFlowInputEntry::decode_at)?;
        let stored = shape::required(object, "stored", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let start_ms = shape::required(object, "startMs", at, shape::integer)?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, NetExchange::decode_at)
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list_within(value, at, Some(1), None, LocalFlowInputStepsItem::decode_at)
        })?;
        Ok(Self {
            kind,
            entry,
            stored,
            start_ms,
            exchanges,
            rows,
            steps,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("entry", self.entry.encode());
        object.insert(
            "stored",
            self.stored
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert("startMs", Value::Number(self.start_ms as f64));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalFlowInputKind {
    Ollama,
    Lmstudio,
    Llamacpp,
    Compatible,
}

impl LocalFlowInputKind {
    pub const ALL: [Self; 4] = [
        Self::Ollama,
        Self::Lmstudio,
        Self::Llamacpp,
        Self::Compatible,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::Lmstudio => "lmstudio",
            Self::Llamacpp => "llamacpp",
            Self::Compatible => "compatible",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalFlowInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowInputEntry {
    pub id: String,
    pub label: String,
    pub url: Option<String>,
}

impl Shape for LocalFlowInputEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let url = shape::optional(object, "url", at, shape::text)?;
        Ok(Self { id, label, url })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("label", Value::from(self.label.as_str()));
        if let Some(held) = &self.url {
            object.insert("url", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocalFlowInputStepsItem {
    Views(LocalFlowInputStepsItemViews),
    Chat(LocalFlowInputStepsItemChat),
    List(LocalFlowInputStepsItemList),
    Detail(LocalFlowInputStepsItemDetail),
    Listed(LocalFlowInputStepsItemListed),
    View(LocalFlowInputStepsItemView),
    Advance(LocalFlowInputStepsItemAdvance),
    Rows(LocalFlowInputStepsItemRows),
}

impl Shape for LocalFlowInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => LocalFlowInputStepsItemViews::decode_at(value, at).map(Self::Views),
            "chat" => LocalFlowInputStepsItemChat::decode_at(value, at).map(Self::Chat),
            "list" => LocalFlowInputStepsItemList::decode_at(value, at).map(Self::List),
            "detail" => LocalFlowInputStepsItemDetail::decode_at(value, at).map(Self::Detail),
            "listed" => LocalFlowInputStepsItemListed::decode_at(value, at).map(Self::Listed),
            "view" => LocalFlowInputStepsItemView::decode_at(value, at).map(Self::View),
            "advance" => LocalFlowInputStepsItemAdvance::decode_at(value, at).map(Self::Advance),
            "rows" => LocalFlowInputStepsItemRows::decode_at(value, at).map(Self::Rows),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::Chat(held) => held.encode(),
            Self::List(held) => held.encode(),
            Self::Detail(held) => held.encode(),
            Self::Listed(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::Advance(held) => held.encode(),
            Self::Rows(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowInputStepsItemViews {
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub zdr_only: Option<bool>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalFlowInputStepsItemViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let zdr_only = shape::optional(object, "zdrOnly", at, shape::flag)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            use_,
            zdr_only,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.zdr_only {
            object.insert("zdrOnly", Value::Bool(*held));
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowInputStepsItemChat {
    pub model: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalFlowInputStepsItemChat {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "chat")
        })?;
        let model = shape::required(object, "model", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            model,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("chat"));
        object.insert("model", Value::from(self.model.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowInputStepsItemList {
    pub fresh: Option<bool>,
}

impl Shape for LocalFlowInputStepsItemList {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "list")
        })?;
        let fresh = shape::optional(object, "fresh", at, shape::flag)?;
        Ok(Self { fresh })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("list"));
        if let Some(held) = &self.fresh {
            object.insert("fresh", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowInputStepsItemDetail {
    pub id: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalFlowInputStepsItemDetail {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "detail")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            id,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("detail"));
        object.insert("id", Value::from(self.id.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowInputStepsItemListed {
    pub id: String,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalFlowInputStepsItemListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "listed")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            id,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("listed"));
        object.insert("id", Value::from(self.id.as_str()));
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowInputStepsItemView {
    pub row: String,
    pub use_: Option<nyaaai_contract::shapes::model::ModelUse>,
    pub abort_after_ms: Option<i64>,
    pub aborted_before: Option<bool>,
}

impl Shape for LocalFlowInputStepsItemView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let row = shape::required(object, "row", at, shape::text)?;
        let use_ = shape::optional(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let abort_after_ms = shape::optional(object, "abortAfterMs", at, shape::integer)?;
        let aborted_before = shape::optional(object, "abortedBefore", at, shape::flag)?;
        Ok(Self {
            row,
            use_,
            abort_after_ms,
            aborted_before,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("row", Value::from(self.row.as_str()));
        if let Some(held) = &self.use_ {
            object.insert("use", held.encode());
        }
        if let Some(held) = &self.abort_after_ms {
            object.insert("abortAfterMs", Value::Number(*held as f64));
        }
        if let Some(held) = &self.aborted_before {
            object.insert("abortedBefore", Value::Bool(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowInputStepsItemAdvance {
    pub ms: u64,
}

impl Shape for LocalFlowInputStepsItemAdvance {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "advance")
        })?;
        let ms = shape::required(object, "ms", at, shape::natural)?;
        Ok(Self { ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("advance"));
        object.insert("ms", Value::Number(self.ms as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowInputStepsItemRows {
    pub rows: Vec<String>,
}

impl Shape for LocalFlowInputStepsItemRows {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "rows")
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("rows"));
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResult {
    pub steps: Vec<LocalFlowResultStepsItem>,
    pub requests: Vec<LocalFlowResultRequestsItem>,
    pub read: Vec<i64>,
    pub waiting: Vec<String>,
    pub refreshed: i64,
}

impl Shape for LocalFlowResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, LocalFlowResultStepsItem::decode_at)
        })?;
        let requests = shape::required(object, "requests", at, |value, at| {
            shape::list(value, at, LocalFlowResultRequestsItem::decode_at)
        })?;
        let read = shape::required(object, "read", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        let waiting = shape::required(object, "waiting", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let refreshed = shape::required(object, "refreshed", at, shape::integer)?;
        Ok(Self {
            steps,
            requests,
            read,
            waiting,
            refreshed,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "requests",
            Value::Array(self.requests.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "read",
            Value::Array(
                self.read
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        object.insert(
            "waiting",
            Value::Array(
                self.waiting
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert("refreshed", Value::Number(self.refreshed as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResultStepsItem {
    pub now_ms: i64,
    pub answer: LocalFlowResultStepsItemAnswer,
}

impl Shape for LocalFlowResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let answer = shape::required(
            object,
            "answer",
            at,
            LocalFlowResultStepsItemAnswer::decode_at,
        )?;
        Ok(Self { now_ms, answer })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert("answer", self.answer.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResultRequestsItem {
    pub method: String,
    pub url: String,
    pub authorization: Option<String>,
    pub body: Value,
}

impl Shape for LocalFlowResultRequestsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let method = shape::required(object, "method", at, shape::text)?;
        let url = shape::required(object, "url", at, shape::text)?;
        let authorization = shape::required(object, "authorization", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self {
            method,
            url,
            authorization,
            body,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("method", Value::from(self.method.as_str()));
        object.insert("url", Value::from(self.url.as_str()));
        object.insert(
            "authorization",
            self.authorization
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocalFlowResultStepsItemAnswer {
    Views(LocalFlowResultStepsItemAnswerViews),
    Chat(LocalFlowResultStepsItemAnswerChat),
    List(LocalFlowResultStepsItemAnswerList),
    Detail(LocalFlowResultStepsItemAnswerDetail),
    Listed(LocalFlowResultStepsItemAnswerListed),
    View(LocalFlowResultStepsItemAnswerView),
    None(LocalFlowResultStepsItemAnswerNone),
    Thrown(LocalFlowResultStepsItemAnswerThrown),
}

impl Shape for LocalFlowResultStepsItemAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "views" => LocalFlowResultStepsItemAnswerViews::decode_at(value, at).map(Self::Views),
            "chat" => LocalFlowResultStepsItemAnswerChat::decode_at(value, at).map(Self::Chat),
            "list" => LocalFlowResultStepsItemAnswerList::decode_at(value, at).map(Self::List),
            "detail" => {
                LocalFlowResultStepsItemAnswerDetail::decode_at(value, at).map(Self::Detail)
            }
            "listed" => {
                LocalFlowResultStepsItemAnswerListed::decode_at(value, at).map(Self::Listed)
            }
            "view" => LocalFlowResultStepsItemAnswerView::decode_at(value, at).map(Self::View),
            "none" => LocalFlowResultStepsItemAnswerNone::decode_at(value, at).map(Self::None),
            "thrown" => {
                LocalFlowResultStepsItemAnswerThrown::decode_at(value, at).map(Self::Thrown)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Views(held) => held.encode(),
            Self::Chat(held) => held.encode(),
            Self::List(held) => held.encode(),
            Self::Detail(held) => held.encode(),
            Self::Listed(held) => held.encode(),
            Self::View(held) => held.encode(),
            Self::None(held) => held.encode(),
            Self::Thrown(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResultStepsItemAnswerViews {
    pub views: Vec<ViewLine>,
}

impl Shape for LocalFlowResultStepsItemAnswerViews {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "views")
        })?;
        let views = shape::required(object, "views", at, |value, at| {
            shape::list(value, at, ViewLine::decode_at)
        })?;
        Ok(Self { views })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("views"));
        object.insert(
            "views",
            Value::Array(self.views.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResultStepsItemAnswerChat {
    pub text: String,
}

impl Shape for LocalFlowResultStepsItemAnswerChat {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "chat")
        })?;
        let text = shape::required(object, "text", at, shape::text)?;
        Ok(Self { text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("chat"));
        object.insert("text", Value::from(self.text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResultStepsItemAnswerList {
    pub models: Option<Vec<nyaaai_contract::shapes::model::BrowseModel>>,
}

impl Shape for LocalFlowResultStepsItemAnswerList {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "list")
        })?;
        let models = shape::required(object, "models", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                shape::list(
                    value,
                    at,
                    nyaaai_contract::shapes::model::BrowseModel::decode_at,
                )
            })
        })?;
        Ok(Self { models })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("list"));
        object.insert(
            "models",
            self.models.as_ref().map_or(Value::Null, |held| {
                Value::Array(held.iter().map(|held| held.encode()).collect())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResultStepsItemAnswerDetail {
    pub detail: Option<nyaaai_contract::shapes::model::BrowseDetail>,
}

impl Shape for LocalFlowResultStepsItemAnswerDetail {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "detail")
        })?;
        let detail = shape::required(object, "detail", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::model::BrowseDetail::decode_at,
            )
        })?;
        Ok(Self { detail })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("detail"));
        object.insert(
            "detail",
            self.detail
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResultStepsItemAnswerListed {
    pub listed: Option<LocalFlowResultStepsItemAnswerListedListed>,
}

impl Shape for LocalFlowResultStepsItemAnswerListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "listed")
        })?;
        let listed = shape::required(object, "listed", at, |value, at| {
            shape::nullable(
                value,
                at,
                LocalFlowResultStepsItemAnswerListedListed::decode_at,
            )
        })?;
        Ok(Self { listed })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("listed"));
        object.insert(
            "listed",
            self.listed
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResultStepsItemAnswerView {
    pub view: ViewLine,
}

impl Shape for LocalFlowResultStepsItemAnswerView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "view")
        })?;
        let view = shape::required(object, "view", at, ViewLine::decode_at)?;
        Ok(Self { view })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("view"));
        object.insert("view", self.view.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResultStepsItemAnswerNone {}

impl Shape for LocalFlowResultStepsItemAnswerNone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "none")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("none"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResultStepsItemAnswerThrown {
    pub message: String,
    pub category: Option<String>,
}

impl Shape for LocalFlowResultStepsItemAnswerThrown {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "thrown")
        })?;
        let message = shape::required(object, "message", at, shape::text)?;
        let category = shape::required(object, "category", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self { message, category })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("thrown"));
        object.insert("message", Value::from(self.message.as_str()));
        object.insert(
            "category",
            self.category
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalFlowResultStepsItemAnswerListedListed {
    pub label: String,
    pub reasoning: bool,
}

impl Shape for LocalFlowResultStepsItemAnswerListedListed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let reasoning = shape::required(object, "reasoning", at, shape::flag)?;
        Ok(Self { label, reasoning })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("label", Value::from(self.label.as_str()));
        object.insert("reasoning", Value::Bool(self.reasoning));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalKindFactsInput {
    pub kind: LocalKindFactsInputKind,
    pub url: Option<String>,
    pub states: Vec<nyaaai_contract::shapes::provider::KeyState>,
}

impl Shape for LocalKindFactsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, LocalKindFactsInputKind::decode_at)?;
        let url = shape::optional(object, "url", at, shape::text)?;
        let states = shape::required(object, "states", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::KeyState::decode_at,
            )
        })?;
        Ok(Self { kind, url, states })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        if let Some(held) = &self.url {
            object.insert("url", Value::from(held.as_str()));
        }
        object.insert(
            "states",
            Value::Array(self.states.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalKindFactsInputKind {
    Ollama,
    Lmstudio,
    Llamacpp,
    Compatible,
}

impl LocalKindFactsInputKind {
    pub const ALL: [Self; 4] = [
        Self::Ollama,
        Self::Lmstudio,
        Self::Llamacpp,
        Self::Compatible,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::Lmstudio => "lmstudio",
            Self::Llamacpp => "llamacpp",
            Self::Compatible => "compatible",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalKindFactsInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalKindFactsResult {
    pub facts: Vec<Option<String>>,
}

impl Shape for LocalKindFactsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let facts = shape::required(object, "facts", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { facts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "facts",
            Value::Array(
                self.facts
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalKindCodecInput {
    pub kind: LocalKindCodecInputKind,
    pub label: String,
    pub view: Option<LocalKindCodecInputView>,
    pub request: nyaaai_contract::shapes::step::CanonicalRequest,
    pub mode: LocalKindCodecInputMode,
    pub failures: Vec<LocalKindCodecInputFailuresItem>,
}

impl Shape for LocalKindCodecInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, LocalKindCodecInputKind::decode_at)?;
        let label = shape::required(object, "label", at, shape::text)?;
        let view = shape::required(object, "view", at, |value, at| {
            shape::nullable(value, at, LocalKindCodecInputView::decode_at)
        })?;
        let request = shape::required(
            object,
            "request",
            at,
            nyaaai_contract::shapes::step::CanonicalRequest::decode_at,
        )?;
        let mode = shape::required(object, "mode", at, LocalKindCodecInputMode::decode_at)?;
        let failures = shape::required(object, "failures", at, |value, at| {
            shape::list(value, at, LocalKindCodecInputFailuresItem::decode_at)
        })?;
        Ok(Self {
            kind,
            label,
            view,
            request,
            mode,
            failures,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("label", Value::from(self.label.as_str()));
        object.insert(
            "view",
            self.view.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        object.insert("request", self.request.encode());
        object.insert("mode", self.mode.encode());
        object.insert(
            "failures",
            Value::Array(self.failures.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalKindCodecInputKind {
    Ollama,
    Lmstudio,
    Llamacpp,
    Compatible,
}

impl LocalKindCodecInputKind {
    pub const ALL: [Self; 4] = [
        Self::Ollama,
        Self::Lmstudio,
        Self::Llamacpp,
        Self::Compatible,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::Lmstudio => "lmstudio",
            Self::Llamacpp => "llamacpp",
            Self::Compatible => "compatible",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalKindCodecInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalKindCodecInputView {
    pub efforts: Vec<nyaaai_contract::shapes::step::Effort>,
    pub api: Option<nyaaai_contract::shapes::step::ApiFacts>,
    pub output_limit: Option<f64>,
    pub price: Option<LocalKindCodecInputViewPrice>,
}

impl Shape for LocalKindCodecInputView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
        })?;
        let api = shape::optional(
            object,
            "api",
            at,
            nyaaai_contract::shapes::step::ApiFacts::decode_at,
        )?;
        let output_limit = shape::optional(object, "outputLimit", at, shape::number)?;
        let price = shape::optional(object, "price", at, LocalKindCodecInputViewPrice::decode_at)?;
        Ok(Self {
            efforts,
            api,
            output_limit,
            price,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        if let Some(held) = &self.api {
            object.insert("api", held.encode());
        }
        if let Some(held) = &self.output_limit {
            object.insert("outputLimit", Value::Number(*held));
        }
        if let Some(held) = &self.price {
            object.insert("price", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalKindCodecInputMode {
    Stream,
    Complete,
}

impl LocalKindCodecInputMode {
    pub const ALL: [Self; 2] = [Self::Stream, Self::Complete];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stream => "stream",
            Self::Complete => "complete",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalKindCodecInputMode {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalKindCodecInputFailuresItem {
    pub status: i64,
    pub body: Value,
}

impl Shape for LocalKindCodecInputFailuresItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let status = shape::required(object, "status", at, shape::integer)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { status, body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("status", Value::Number(self.status as f64));
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalKindCodecInputViewPrice {
    Free,
    Unknown,
}

impl LocalKindCodecInputViewPrice {
    pub const ALL: [Self; 2] = [Self::Free, Self::Unknown];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalKindCodecInputViewPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalKindCodecResult {
    pub encoded: Option<nyaaai_contract::shapes::step::Encoded>,
    pub refusal: Option<String>,
    pub facts: Option<LocalKindCodecResultFacts>,
    pub failures: Vec<nyaaai_contract::shapes::step::Failure>,
}

impl Shape for LocalKindCodecResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let encoded = shape::required(object, "encoded", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::step::Encoded::decode_at)
        })?;
        let refusal = shape::required(object, "refusal", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let facts = shape::required(object, "facts", at, |value, at| {
            shape::nullable(value, at, LocalKindCodecResultFacts::decode_at)
        })?;
        let failures = shape::required(object, "failures", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Failure::decode_at)
        })?;
        Ok(Self {
            encoded,
            refusal,
            facts,
            failures,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "encoded",
            self.encoded
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "refusal",
            self.refusal
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "facts",
            self.facts
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "failures",
            Value::Array(self.failures.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalKindCodecResultFacts {
    pub efforts: Vec<nyaaai_contract::shapes::step::Effort>,
    pub free: bool,
    pub output_limit: Option<f64>,
}

impl Shape for LocalKindCodecResultFacts {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let efforts = shape::required(object, "efforts", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::step::Effort::decode_at)
        })?;
        let free = shape::required(object, "free", at, shape::flag)?;
        let output_limit = shape::required(object, "outputLimit", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        Ok(Self {
            efforts,
            free,
            output_limit,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "efforts",
            Value::Array(self.efforts.iter().map(|held| held.encode()).collect()),
        );
        object.insert("free", Value::Bool(self.free));
        object.insert(
            "outputLimit",
            self.output_limit
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalKindDispositionInput {
    pub kind: LocalKindDispositionInputKind,
    pub rows: Vec<LocalKindDispositionInputRowsItem>,
}

impl Shape for LocalKindDispositionInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(object, "kind", at, LocalKindDispositionInputKind::decode_at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, LocalKindDispositionInputRowsItem::decode_at)
        })?;
        Ok(Self { kind, rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalKindDispositionInputKind {
    Ollama,
    Lmstudio,
    Llamacpp,
    Compatible,
}

impl LocalKindDispositionInputKind {
    pub const ALL: [Self; 4] = [
        Self::Ollama,
        Self::Lmstudio,
        Self::Llamacpp,
        Self::Compatible,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ollama => "ollama",
            Self::Lmstudio => "lmstudio",
            Self::Llamacpp => "llamacpp",
            Self::Compatible => "compatible",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalKindDispositionInputKind {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalKindDispositionInputRowsItem {
    pub failure: nyaaai_contract::shapes::step::Failure,
    pub status: Option<LocalKindDispositionInputRowsItemStatus>,
}

impl Shape for LocalKindDispositionInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let failure = shape::required(
            object,
            "failure",
            at,
            nyaaai_contract::shapes::step::Failure::decode_at,
        )?;
        let status = shape::required(object, "status", at, |value, at| {
            shape::nullable(
                value,
                at,
                LocalKindDispositionInputRowsItemStatus::decode_at,
            )
        })?;
        Ok(Self { failure, status })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("failure", self.failure.encode());
        object.insert(
            "status",
            self.status
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocalKindDispositionInputRowsItemStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl LocalKindDispositionInputRowsItemStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for LocalKindDispositionInputRowsItemStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalKindDispositionResult {
    pub dispositions: Vec<nyaaai_contract::shapes::provider::Disposition>,
}

impl Shape for LocalKindDispositionResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let dispositions = shape::required(object, "dispositions", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::Disposition::decode_at,
            )
        })?;
        Ok(Self { dispositions })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "dispositions",
            Value::Array(self.dispositions.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverReachInput {
    pub rows: Vec<ResolverReachInputRowsItem>,
}

impl Shape for ResolverReachInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, ResolverReachInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverReachInputRowsItem {
    pub field: ResolverReachInputRowsItemField,
    pub answer: ResolverReachInputRowsItemAnswer,
}

impl Shape for ResolverReachInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let field = shape::required(
            object,
            "field",
            at,
            ResolverReachInputRowsItemField::decode_at,
        )?;
        let answer = shape::required(
            object,
            "answer",
            at,
            ResolverReachInputRowsItemAnswer::decode_at,
        )?;
        Ok(Self { field, answer })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("field", self.field.encode());
        object.insert("answer", self.answer.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolverReachInputRowsItemField {
    Key,
    Url,
}

impl ResolverReachInputRowsItemField {
    pub const ALL: [Self; 2] = [Self::Key, Self::Url];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Url => "url",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ResolverReachInputRowsItemField {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverReachInputRowsItemAnswer {
    Key(ResolverReachInputRowsItemAnswerKey),
    None(ResolverReachInputRowsItemAnswerNone),
    Refused(ResolverReachInputRowsItemAnswerRefused),
    Unready(ResolverReachInputRowsItemAnswerUnready),
}

impl Shape for ResolverReachInputRowsItemAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "key" => ResolverReachInputRowsItemAnswerKey::decode_at(value, at).map(Self::Key),
            "none" => ResolverReachInputRowsItemAnswerNone::decode_at(value, at).map(Self::None),
            "refused" => {
                ResolverReachInputRowsItemAnswerRefused::decode_at(value, at).map(Self::Refused)
            }
            "unready" => {
                ResolverReachInputRowsItemAnswerUnready::decode_at(value, at).map(Self::Unready)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Key(held) => held.encode(),
            Self::None(held) => held.encode(),
            Self::Refused(held) => held.encode(),
            Self::Unready(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverReachInputRowsItemAnswerKey {
    pub key: String,
}

impl Shape for ResolverReachInputRowsItemAnswerKey {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "key")
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        Ok(Self { key })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("key"));
        object.insert("key", Value::from(self.key.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverReachInputRowsItemAnswerNone {}

impl Shape for ResolverReachInputRowsItemAnswerNone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "none")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("none"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverReachInputRowsItemAnswerRefused {}

impl Shape for ResolverReachInputRowsItemAnswerRefused {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "refused")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("refused"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverReachInputRowsItemAnswerUnready {}

impl Shape for ResolverReachInputRowsItemAnswerUnready {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "unready")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("unready"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverReachResult {
    pub reaches: Vec<nyaaai_contract::shapes::provider::KeyReach>,
}

impl Shape for ResolverReachResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let reaches = shape::required(object, "reaches", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::KeyReach::decode_at,
            )
        })?;
        Ok(Self { reaches })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "reaches",
            Value::Array(self.reaches.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRouteKeyInput {
    pub entries: Vec<nyaaai_contract::shapes::provider::ProviderEntry>,
}

impl Shape for ResolverRouteKeyInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entries = shape::required(object, "entries", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
            )
        })?;
        Ok(Self { entries })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "entries",
            Value::Array(self.entries.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRouteKeyResult {
    pub keys: Vec<String>,
}

impl Shape for ResolverRouteKeyResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let keys = shape::required(object, "keys", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { keys })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "keys",
            Value::Array(
                self.keys
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInput {
    pub entry: nyaaai_contract::shapes::provider::ProviderEntry,
    pub list: Vec<nyaaai_contract::shapes::provider::ProviderEntry>,
    pub answers: Record<ResolverKeySourceInputAnswersValue>,
    pub steps: Vec<ResolverKeySourceInputStepsItem>,
}

impl Shape for ResolverKeySourceInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entry = shape::required(
            object,
            "entry",
            at,
            nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
        )?;
        let list = shape::required(object, "list", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
            )
        })?;
        let answers = shape::required(object, "answers", at, |value, at| {
            shape::record(value, at, ResolverKeySourceInputAnswersValue::decode_at)
        })?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, ResolverKeySourceInputStepsItem::decode_at)
        })?;
        Ok(Self {
            entry,
            list,
            answers,
            steps,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("entry", self.entry.encode());
        object.insert(
            "list",
            Value::Array(self.list.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "answers",
            shape::record_value(&self.answers, |held| held.encode()),
        );
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverKeySourceInputAnswersValue {
    Key(ResolverKeySourceInputAnswersValueKey),
    None(ResolverKeySourceInputAnswersValueNone),
    Refused(ResolverKeySourceInputAnswersValueRefused),
    Unready(ResolverKeySourceInputAnswersValueUnready),
}

impl Shape for ResolverKeySourceInputAnswersValue {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "key" => ResolverKeySourceInputAnswersValueKey::decode_at(value, at).map(Self::Key),
            "none" => ResolverKeySourceInputAnswersValueNone::decode_at(value, at).map(Self::None),
            "refused" => {
                ResolverKeySourceInputAnswersValueRefused::decode_at(value, at).map(Self::Refused)
            }
            "unready" => {
                ResolverKeySourceInputAnswersValueUnready::decode_at(value, at).map(Self::Unready)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Key(held) => held.encode(),
            Self::None(held) => held.encode(),
            Self::Refused(held) => held.encode(),
            Self::Unready(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverKeySourceInputStepsItem {
    Get(ResolverKeySourceInputStepsItemGet),
    List(ResolverKeySourceInputStepsItemList),
    Answer(ResolverKeySourceInputStepsItemAnswer),
}

impl Shape for ResolverKeySourceInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "get" => ResolverKeySourceInputStepsItemGet::decode_at(value, at).map(Self::Get),
            "list" => ResolverKeySourceInputStepsItemList::decode_at(value, at).map(Self::List),
            "answer" => {
                ResolverKeySourceInputStepsItemAnswer::decode_at(value, at).map(Self::Answer)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Get(held) => held.encode(),
            Self::List(held) => held.encode(),
            Self::Answer(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInputAnswersValueKey {
    pub key: String,
}

impl Shape for ResolverKeySourceInputAnswersValueKey {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "key")
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        Ok(Self { key })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("key"));
        object.insert("key", Value::from(self.key.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInputAnswersValueNone {}

impl Shape for ResolverKeySourceInputAnswersValueNone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "none")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("none"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInputAnswersValueRefused {}

impl Shape for ResolverKeySourceInputAnswersValueRefused {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "refused")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("refused"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInputAnswersValueUnready {}

impl Shape for ResolverKeySourceInputAnswersValueUnready {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "unready")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("unready"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInputStepsItemGet {}

impl Shape for ResolverKeySourceInputStepsItemGet {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "get")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("get"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInputStepsItemList {
    pub entries: Vec<nyaaai_contract::shapes::provider::ProviderEntry>,
}

impl Shape for ResolverKeySourceInputStepsItemList {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "list")
        })?;
        let entries = shape::required(object, "entries", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
            )
        })?;
        Ok(Self { entries })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("list"));
        object.insert(
            "entries",
            Value::Array(self.entries.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInputStepsItemAnswer {
    pub id: String,
    pub answer: ResolverKeySourceInputStepsItemAnswerAnswer,
}

impl Shape for ResolverKeySourceInputStepsItemAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "answer")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let answer = shape::required(
            object,
            "answer",
            at,
            ResolverKeySourceInputStepsItemAnswerAnswer::decode_at,
        )?;
        Ok(Self { id, answer })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("answer"));
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("answer", self.answer.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverKeySourceInputStepsItemAnswerAnswer {
    Key(ResolverKeySourceInputStepsItemAnswerAnswerKey),
    None(ResolverKeySourceInputStepsItemAnswerAnswerNone),
    Refused(ResolverKeySourceInputStepsItemAnswerAnswerRefused),
    Unready(ResolverKeySourceInputStepsItemAnswerAnswerUnready),
}

impl Shape for ResolverKeySourceInputStepsItemAnswerAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "key" => {
                ResolverKeySourceInputStepsItemAnswerAnswerKey::decode_at(value, at).map(Self::Key)
            }
            "none" => ResolverKeySourceInputStepsItemAnswerAnswerNone::decode_at(value, at)
                .map(Self::None),
            "refused" => ResolverKeySourceInputStepsItemAnswerAnswerRefused::decode_at(value, at)
                .map(Self::Refused),
            "unready" => ResolverKeySourceInputStepsItemAnswerAnswerUnready::decode_at(value, at)
                .map(Self::Unready),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Key(held) => held.encode(),
            Self::None(held) => held.encode(),
            Self::Refused(held) => held.encode(),
            Self::Unready(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInputStepsItemAnswerAnswerKey {
    pub key: String,
}

impl Shape for ResolverKeySourceInputStepsItemAnswerAnswerKey {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "key")
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        Ok(Self { key })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("key"));
        object.insert("key", Value::from(self.key.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInputStepsItemAnswerAnswerNone {}

impl Shape for ResolverKeySourceInputStepsItemAnswerAnswerNone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "none")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("none"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInputStepsItemAnswerAnswerRefused {}

impl Shape for ResolverKeySourceInputStepsItemAnswerAnswerRefused {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "refused")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("refused"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceInputStepsItemAnswerAnswerUnready {}

impl Shape for ResolverKeySourceInputStepsItemAnswerAnswerUnready {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "unready")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("unready"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverKeySourceResult {
    pub gets: Vec<Option<String>>,
}

impl Shape for ResolverKeySourceResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let gets = shape::required(object, "gets", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { gets })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "gets",
            Value::Array(
                self.gets
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverServedByInput {
    pub kind: nyaaai_contract::shapes::provider::ProviderKindName,
    pub entry: nyaaai_contract::shapes::provider::ProviderEntry,
    pub key: String,
    pub use_: nyaaai_contract::shapes::model::ModelUse,
    pub view: Option<ResolverServedByInputView>,
    pub zdr_only: bool,
    pub flipped_to: bool,
}

impl Shape for ResolverServedByInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let kind = shape::required(
            object,
            "kind",
            at,
            nyaaai_contract::shapes::provider::ProviderKindName::decode_at,
        )?;
        let entry = shape::required(
            object,
            "entry",
            at,
            nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
        )?;
        let key = shape::required(object, "key", at, shape::text)?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let view = shape::required(object, "view", at, |value, at| {
            shape::nullable(value, at, ResolverServedByInputView::decode_at)
        })?;
        let zdr_only = shape::required(object, "zdrOnly", at, shape::flag)?;
        let flipped_to = shape::required(object, "flippedTo", at, shape::flag)?;
        Ok(Self {
            kind,
            entry,
            key,
            use_,
            view,
            zdr_only,
            flipped_to,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", self.kind.encode());
        object.insert("entry", self.entry.encode());
        object.insert("key", Value::from(self.key.as_str()));
        object.insert("use", self.use_.encode());
        object.insert(
            "view",
            self.view.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        object.insert("zdrOnly", Value::Bool(self.zdr_only));
        object.insert("flippedTo", Value::Bool(self.flipped_to));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolverServedByInputView {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl ResolverServedByInputView {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ResolverServedByInputView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverServedByResult {
    pub key: String,
    pub entry: String,
    pub binding: String,
    pub view: nyaaai_contract::shapes::model::ModelView,
    pub path: String,
    pub passed: Vec<nyaaai_contract::shapes::provider::Passed>,
    pub candidates: i64,
    pub zdr_on_resolve: bool,
    pub sent: ResolverServedByResultSent,
}

impl Shape for ResolverServedByResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let key = shape::required(object, "key", at, shape::text)?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let binding = shape::required(object, "binding", at, shape::text)?;
        let view = shape::required(
            object,
            "view",
            at,
            nyaaai_contract::shapes::model::ModelView::decode_at,
        )?;
        let path = shape::required(object, "path", at, shape::text)?;
        let passed = shape::required(object, "passed", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::Passed::decode_at,
            )
        })?;
        let candidates = shape::required(object, "candidates", at, shape::integer)?;
        let zdr_on_resolve = shape::required(object, "zdrOnResolve", at, shape::flag)?;
        let sent = shape::required(object, "sent", at, ResolverServedByResultSent::decode_at)?;
        Ok(Self {
            key,
            entry,
            binding,
            view,
            path,
            passed,
            candidates,
            zdr_on_resolve,
            sent,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("key", Value::from(self.key.as_str()));
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("binding", Value::from(self.binding.as_str()));
        object.insert("view", self.view.encode());
        object.insert("path", Value::from(self.path.as_str()));
        object.insert(
            "passed",
            Value::Array(self.passed.iter().map(|held| held.encode()).collect()),
        );
        object.insert("candidates", Value::Number(self.candidates as f64));
        object.insert("zdrOnResolve", Value::Bool(self.zdr_on_resolve));
        object.insert("sent", self.sent.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverServedByResultSent {
    pub zdr_only: bool,
    pub path: String,
}

impl Shape for ResolverServedByResultSent {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let zdr_only = shape::required(object, "zdrOnly", at, shape::flag)?;
        let path = shape::required(object, "path", at, shape::text)?;
        Ok(Self { zdr_only, path })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("zdrOnly", Value::Bool(self.zdr_only));
        object.insert("path", Value::from(self.path.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverUsesInput {
    pub rows: Vec<String>,
    pub uses: Vec<nyaaai_contract::shapes::model::ModelUse>,
}

impl Shape for ResolverUsesInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let uses = shape::required(object, "uses", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::model::ModelUse::decode_at,
            )
        })?;
        Ok(Self { rows, uses })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "uses",
            Value::Array(self.uses.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverUsesResult {
    pub calls: Vec<ResolverUsesResultCallsItem>,
}

impl Shape for ResolverUsesResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let calls = shape::required(object, "calls", at, |value, at| {
            shape::list(value, at, ResolverUsesResultCallsItem::decode_at)
        })?;
        Ok(Self { calls })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "calls",
            Value::Array(self.calls.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverUsesResultCallsItem {
    pub call: ResolverUsesResultCallsItemCall,
    pub use_: nyaaai_contract::shapes::model::ModelUse,
    pub asked: Vec<nyaaai_contract::shapes::model::ModelUse>,
}

impl Shape for ResolverUsesResultCallsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let call = shape::required(
            object,
            "call",
            at,
            ResolverUsesResultCallsItemCall::decode_at,
        )?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        let asked = shape::required(object, "asked", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::model::ModelUse::decode_at,
            )
        })?;
        Ok(Self { call, use_, asked })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("call", self.call.encode());
        object.insert("use", self.use_.encode());
        object.insert(
            "asked",
            Value::Array(self.asked.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolverUsesResultCallsItemCall {
    Routes,
    Resolve,
    Rows,
    Candidates,
}

impl ResolverUsesResultCallsItemCall {
    pub const ALL: [Self; 4] = [Self::Routes, Self::Resolve, Self::Rows, Self::Candidates];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Routes => "routes",
            Self::Resolve => "resolve",
            Self::Rows => "rows",
            Self::Candidates => "candidates",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ResolverUsesResultCallsItemCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInput {
    pub entries: Vec<nyaaai_contract::shapes::provider::ProviderEntry>,
    pub answers: Record<ResolverRoutesInputAnswersValue>,
    pub failing: Vec<String>,
    pub rows: Vec<String>,
    pub kinds: Vec<nyaaai_contract::shapes::provider::ProviderKindName>,
    pub listings: Record<Record<ResolverRoutesInputListingsValueValue>>,
    pub dispose_fails: Option<Vec<String>>,
    pub start_ms: Option<i64>,
    pub zdr_only: bool,
    pub steps: Vec<ResolverRoutesInputStepsItem>,
}

impl Shape for ResolverRoutesInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entries = shape::required(object, "entries", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
            )
        })?;
        let answers = shape::required(object, "answers", at, |value, at| {
            shape::record(value, at, ResolverRoutesInputAnswersValue::decode_at)
        })?;
        let failing = shape::required(object, "failing", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let kinds = shape::required(object, "kinds", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::ProviderKindName::decode_at,
            )
        })?;
        let listings = shape::required(object, "listings", at, |value, at| {
            shape::record(value, at, |value, at| {
                shape::record(value, at, ResolverRoutesInputListingsValueValue::decode_at)
            })
        })?;
        let dispose_fails = shape::optional(object, "disposeFails", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let start_ms = shape::optional(object, "startMs", at, shape::integer)?;
        let zdr_only = shape::required(object, "zdrOnly", at, shape::flag)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list_within(
                value,
                at,
                Some(1),
                None,
                ResolverRoutesInputStepsItem::decode_at,
            )
        })?;
        Ok(Self {
            entries,
            answers,
            failing,
            rows,
            kinds,
            listings,
            dispose_fails,
            start_ms,
            zdr_only,
            steps,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "entries",
            Value::Array(self.entries.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "answers",
            shape::record_value(&self.answers, |held| held.encode()),
        );
        object.insert(
            "failing",
            Value::Array(
                self.failing
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "kinds",
            Value::Array(self.kinds.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "listings",
            shape::record_value(&self.listings, |held| {
                shape::record_value(held, |held| held.encode())
            }),
        );
        if let Some(held) = &self.dispose_fails {
            object.insert(
                "disposeFails",
                Value::Array(held.iter().map(|held| Value::from(held.as_str())).collect()),
            );
        }
        if let Some(held) = &self.start_ms {
            object.insert("startMs", Value::Number(*held as f64));
        }
        object.insert("zdrOnly", Value::Bool(self.zdr_only));
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverRoutesInputAnswersValue {
    Key(ResolverRoutesInputAnswersValueKey),
    None(ResolverRoutesInputAnswersValueNone),
    Refused(ResolverRoutesInputAnswersValueRefused),
    Unready(ResolverRoutesInputAnswersValueUnready),
}

impl Shape for ResolverRoutesInputAnswersValue {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "key" => ResolverRoutesInputAnswersValueKey::decode_at(value, at).map(Self::Key),
            "none" => ResolverRoutesInputAnswersValueNone::decode_at(value, at).map(Self::None),
            "refused" => {
                ResolverRoutesInputAnswersValueRefused::decode_at(value, at).map(Self::Refused)
            }
            "unready" => {
                ResolverRoutesInputAnswersValueUnready::decode_at(value, at).map(Self::Unready)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Key(held) => held.encode(),
            Self::None(held) => held.encode(),
            Self::Refused(held) => held.encode(),
            Self::Unready(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputListingsValueValue {
    pub status: ResolverRoutesInputListingsValueValueStatus,
    pub when_zdr_off: Option<ResolverRoutesInputListingsValueValueWhenZdrOff>,
}

impl Shape for ResolverRoutesInputListingsValueValue {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let status = shape::required(
            object,
            "status",
            at,
            ResolverRoutesInputListingsValueValueStatus::decode_at,
        )?;
        let when_zdr_off = shape::optional(
            object,
            "whenZdrOff",
            at,
            ResolverRoutesInputListingsValueValueWhenZdrOff::decode_at,
        )?;
        Ok(Self {
            status,
            when_zdr_off,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("status", self.status.encode());
        if let Some(held) = &self.when_zdr_off {
            object.insert("whenZdrOff", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverRoutesInputStepsItem {
    List(ResolverRoutesInputStepsItemList),
    Answer(ResolverRoutesInputStepsItemAnswer),
    Registry(ResolverRoutesInputStepsItemRegistry),
    Zdr(ResolverRoutesInputStepsItemZdr),
    Routes(ResolverRoutesInputStepsItemRoutes),
    Resolve(ResolverRoutesInputStepsItemResolve),
    Rows(ResolverRoutesInputStepsItemRows),
    Candidates(ResolverRoutesInputStepsItemCandidates),
    Send(ResolverRoutesInputStepsItemSend),
    Refreshed(ResolverRoutesInputStepsItemRefreshed),
    Route(ResolverRoutesInputStepsItemRoute),
    Dispose(ResolverRoutesInputStepsItemDispose),
    During(ResolverRoutesInputStepsItemDuring),
}

impl Shape for ResolverRoutesInputStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "list" => ResolverRoutesInputStepsItemList::decode_at(value, at).map(Self::List),
            "answer" => ResolverRoutesInputStepsItemAnswer::decode_at(value, at).map(Self::Answer),
            "registry" => {
                ResolverRoutesInputStepsItemRegistry::decode_at(value, at).map(Self::Registry)
            }
            "zdr" => ResolverRoutesInputStepsItemZdr::decode_at(value, at).map(Self::Zdr),
            "routes" => ResolverRoutesInputStepsItemRoutes::decode_at(value, at).map(Self::Routes),
            "resolve" => {
                ResolverRoutesInputStepsItemResolve::decode_at(value, at).map(Self::Resolve)
            }
            "rows" => ResolverRoutesInputStepsItemRows::decode_at(value, at).map(Self::Rows),
            "candidates" => {
                ResolverRoutesInputStepsItemCandidates::decode_at(value, at).map(Self::Candidates)
            }
            "send" => ResolverRoutesInputStepsItemSend::decode_at(value, at).map(Self::Send),
            "refreshed" => {
                ResolverRoutesInputStepsItemRefreshed::decode_at(value, at).map(Self::Refreshed)
            }
            "route" => ResolverRoutesInputStepsItemRoute::decode_at(value, at).map(Self::Route),
            "dispose" => {
                ResolverRoutesInputStepsItemDispose::decode_at(value, at).map(Self::Dispose)
            }
            "during" => ResolverRoutesInputStepsItemDuring::decode_at(value, at).map(Self::During),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::List(held) => held.encode(),
            Self::Answer(held) => held.encode(),
            Self::Registry(held) => held.encode(),
            Self::Zdr(held) => held.encode(),
            Self::Routes(held) => held.encode(),
            Self::Resolve(held) => held.encode(),
            Self::Rows(held) => held.encode(),
            Self::Candidates(held) => held.encode(),
            Self::Send(held) => held.encode(),
            Self::Refreshed(held) => held.encode(),
            Self::Route(held) => held.encode(),
            Self::Dispose(held) => held.encode(),
            Self::During(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputAnswersValueKey {
    pub key: String,
}

impl Shape for ResolverRoutesInputAnswersValueKey {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "key")
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        Ok(Self { key })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("key"));
        object.insert("key", Value::from(self.key.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputAnswersValueNone {}

impl Shape for ResolverRoutesInputAnswersValueNone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "none")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("none"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputAnswersValueRefused {}

impl Shape for ResolverRoutesInputAnswersValueRefused {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "refused")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("refused"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputAnswersValueUnready {}

impl Shape for ResolverRoutesInputAnswersValueUnready {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "unready")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("unready"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverRoutesInputListingsValueValueStatus {
    Choice(ResolverRoutesInputListingsValueValueStatusChoice),
    Unlisted,
}

impl Shape for ResolverRoutesInputListingsValueValueStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = ResolverRoutesInputListingsValueValueStatusChoice::decode_at(value, at) {
            return Ok(Self::Choice(held));
        }
        if shape::is_text(value, at, "unlisted").is_ok() {
            return Ok(Self::Unlisted);
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Choice(held) => held.encode(),
            Self::Unlisted => Value::from("unlisted"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverRoutesInputListingsValueValueWhenZdrOff {
    Choice(ResolverRoutesInputListingsValueValueWhenZdrOffChoice),
    Unlisted,
}

impl Shape for ResolverRoutesInputListingsValueValueWhenZdrOff {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) =
            ResolverRoutesInputListingsValueValueWhenZdrOffChoice::decode_at(value, at)
        {
            return Ok(Self::Choice(held));
        }
        if shape::is_text(value, at, "unlisted").is_ok() {
            return Ok(Self::Unlisted);
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Choice(held) => held.encode(),
            Self::Unlisted => Value::from("unlisted"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemList {
    pub entries: Vec<nyaaai_contract::shapes::provider::ProviderEntry>,
}

impl Shape for ResolverRoutesInputStepsItemList {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "list")
        })?;
        let entries = shape::required(object, "entries", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
            )
        })?;
        Ok(Self { entries })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("list"));
        object.insert(
            "entries",
            Value::Array(self.entries.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemAnswer {
    pub id: String,
    pub answer: ResolverRoutesInputStepsItemAnswerAnswer,
}

impl Shape for ResolverRoutesInputStepsItemAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "answer")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let answer = shape::required(
            object,
            "answer",
            at,
            ResolverRoutesInputStepsItemAnswerAnswer::decode_at,
        )?;
        Ok(Self { id, answer })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("answer"));
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("answer", self.answer.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemRegistry {
    pub rows: Vec<String>,
}

impl Shape for ResolverRoutesInputStepsItemRegistry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "registry")
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("registry"));
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemZdr {
    pub on: bool,
}

impl Shape for ResolverRoutesInputStepsItemZdr {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "zdr")
        })?;
        let on = shape::required(object, "on", at, shape::flag)?;
        Ok(Self { on })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("zdr"));
        object.insert("on", Value::Bool(self.on));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemRoutes {
    pub model: String,
    pub use_: nyaaai_contract::shapes::model::ModelUse,
}

impl Shape for ResolverRoutesInputStepsItemRoutes {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "routes")
        })?;
        let model = shape::required(object, "model", at, shape::text)?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        Ok(Self { model, use_ })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("routes"));
        object.insert("model", Value::from(self.model.as_str()));
        object.insert("use", self.use_.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemResolve {
    pub model: String,
    pub use_: nyaaai_contract::shapes::model::ModelUse,
}

impl Shape for ResolverRoutesInputStepsItemResolve {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "resolve")
        })?;
        let model = shape::required(object, "model", at, shape::text)?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        Ok(Self { model, use_ })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("resolve"));
        object.insert("model", Value::from(self.model.as_str()));
        object.insert("use", self.use_.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemRows {
    pub use_: nyaaai_contract::shapes::model::ModelUse,
}

impl Shape for ResolverRoutesInputStepsItemRows {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "rows")
        })?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        Ok(Self { use_ })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("rows"));
        object.insert("use", self.use_.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemCandidates {
    pub use_: nyaaai_contract::shapes::model::ModelUse,
}

impl Shape for ResolverRoutesInputStepsItemCandidates {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "candidates")
        })?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        Ok(Self { use_ })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("candidates"));
        object.insert("use", self.use_.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemSend {
    pub from: u64,
    pub candidate: u64,
    pub mode: Option<ResolverRoutesInputStepsItemSendMode>,
    pub fail: Option<ResolverRoutesInputStepsItemSendFail>,
}

impl Shape for ResolverRoutesInputStepsItemSend {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "send")
        })?;
        let from = shape::required(object, "from", at, shape::natural)?;
        let candidate = shape::required(object, "candidate", at, shape::natural)?;
        let mode = shape::optional(
            object,
            "mode",
            at,
            ResolverRoutesInputStepsItemSendMode::decode_at,
        )?;
        let fail = shape::optional(
            object,
            "fail",
            at,
            ResolverRoutesInputStepsItemSendFail::decode_at,
        )?;
        Ok(Self {
            from,
            candidate,
            mode,
            fail,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("send"));
        object.insert("from", Value::Number(self.from as f64));
        object.insert("candidate", Value::Number(self.candidate as f64));
        if let Some(held) = &self.mode {
            object.insert("mode", held.encode());
        }
        if let Some(held) = &self.fail {
            object.insert("fail", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemRefreshed {
    pub entry: String,
}

impl Shape for ResolverRoutesInputStepsItemRefreshed {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "refreshed")
        })?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        Ok(Self { entry })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("refreshed"));
        object.insert("entry", Value::from(self.entry.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemRoute {
    pub entry: nyaaai_contract::shapes::provider::ProviderEntry,
}

impl Shape for ResolverRoutesInputStepsItemRoute {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "route")
        })?;
        let entry = shape::required(
            object,
            "entry",
            at,
            nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
        )?;
        Ok(Self { entry })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("route"));
        object.insert("entry", self.entry.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDispose {}

impl Shape for ResolverRoutesInputStepsItemDispose {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "dispose")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("dispose"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuring {
    pub changes: Vec<ResolverRoutesInputStepsItemDuringChangesItem>,
    pub call: ResolverRoutesInputStepsItemDuringCall,
}

impl Shape for ResolverRoutesInputStepsItemDuring {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "during")
        })?;
        let changes = shape::required(object, "changes", at, |value, at| {
            shape::list(
                value,
                at,
                ResolverRoutesInputStepsItemDuringChangesItem::decode_at,
            )
        })?;
        let call = shape::required(
            object,
            "call",
            at,
            ResolverRoutesInputStepsItemDuringCall::decode_at,
        )?;
        Ok(Self { changes, call })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("during"));
        object.insert(
            "changes",
            Value::Array(self.changes.iter().map(|held| held.encode()).collect()),
        );
        object.insert("call", self.call.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolverRoutesInputListingsValueValueStatusChoice {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl ResolverRoutesInputListingsValueValueStatusChoice {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ResolverRoutesInputListingsValueValueStatusChoice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolverRoutesInputListingsValueValueWhenZdrOffChoice {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl ResolverRoutesInputListingsValueValueWhenZdrOffChoice {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ResolverRoutesInputListingsValueValueWhenZdrOffChoice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverRoutesInputStepsItemAnswerAnswer {
    Key(ResolverRoutesInputStepsItemAnswerAnswerKey),
    None(ResolverRoutesInputStepsItemAnswerAnswerNone),
    Refused(ResolverRoutesInputStepsItemAnswerAnswerRefused),
    Unready(ResolverRoutesInputStepsItemAnswerAnswerUnready),
}

impl Shape for ResolverRoutesInputStepsItemAnswerAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "key" => {
                ResolverRoutesInputStepsItemAnswerAnswerKey::decode_at(value, at).map(Self::Key)
            }
            "none" => {
                ResolverRoutesInputStepsItemAnswerAnswerNone::decode_at(value, at).map(Self::None)
            }
            "refused" => ResolverRoutesInputStepsItemAnswerAnswerRefused::decode_at(value, at)
                .map(Self::Refused),
            "unready" => ResolverRoutesInputStepsItemAnswerAnswerUnready::decode_at(value, at)
                .map(Self::Unready),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Key(held) => held.encode(),
            Self::None(held) => held.encode(),
            Self::Refused(held) => held.encode(),
            Self::Unready(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolverRoutesInputStepsItemSendMode {
    Complete,
    Stream,
}

impl ResolverRoutesInputStepsItemSendMode {
    pub const ALL: [Self; 2] = [Self::Complete, Self::Stream];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Stream => "stream",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ResolverRoutesInputStepsItemSendMode {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemSendFail {
    pub status: i64,
    pub body: Value,
}

impl Shape for ResolverRoutesInputStepsItemSendFail {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let status = shape::required(object, "status", at, shape::integer)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { status, body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("status", Value::Number(self.status as f64));
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverRoutesInputStepsItemDuringChangesItem {
    List(ResolverRoutesInputStepsItemDuringChangesItemList),
    Answer(ResolverRoutesInputStepsItemDuringChangesItemAnswer),
    Registry(ResolverRoutesInputStepsItemDuringChangesItemRegistry),
    Zdr(ResolverRoutesInputStepsItemDuringChangesItemZdr),
}

impl Shape for ResolverRoutesInputStepsItemDuringChangesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "list" => ResolverRoutesInputStepsItemDuringChangesItemList::decode_at(value, at)
                .map(Self::List),
            "answer" => ResolverRoutesInputStepsItemDuringChangesItemAnswer::decode_at(value, at)
                .map(Self::Answer),
            "registry" => {
                ResolverRoutesInputStepsItemDuringChangesItemRegistry::decode_at(value, at)
                    .map(Self::Registry)
            }
            "zdr" => ResolverRoutesInputStepsItemDuringChangesItemZdr::decode_at(value, at)
                .map(Self::Zdr),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::List(held) => held.encode(),
            Self::Answer(held) => held.encode(),
            Self::Registry(held) => held.encode(),
            Self::Zdr(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverRoutesInputStepsItemDuringCall {
    Routes(ResolverRoutesInputStepsItemDuringCallRoutes),
    Resolve(ResolverRoutesInputStepsItemDuringCallResolve),
    Rows(ResolverRoutesInputStepsItemDuringCallRows),
    Candidates(ResolverRoutesInputStepsItemDuringCallCandidates),
    Send(ResolverRoutesInputStepsItemDuringCallSend),
}

impl Shape for ResolverRoutesInputStepsItemDuringCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "routes" => {
                ResolverRoutesInputStepsItemDuringCallRoutes::decode_at(value, at).map(Self::Routes)
            }
            "resolve" => ResolverRoutesInputStepsItemDuringCallResolve::decode_at(value, at)
                .map(Self::Resolve),
            "rows" => {
                ResolverRoutesInputStepsItemDuringCallRows::decode_at(value, at).map(Self::Rows)
            }
            "candidates" => ResolverRoutesInputStepsItemDuringCallCandidates::decode_at(value, at)
                .map(Self::Candidates),
            "send" => {
                ResolverRoutesInputStepsItemDuringCallSend::decode_at(value, at).map(Self::Send)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Routes(held) => held.encode(),
            Self::Resolve(held) => held.encode(),
            Self::Rows(held) => held.encode(),
            Self::Candidates(held) => held.encode(),
            Self::Send(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemAnswerAnswerKey {
    pub key: String,
}

impl Shape for ResolverRoutesInputStepsItemAnswerAnswerKey {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "key")
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        Ok(Self { key })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("key"));
        object.insert("key", Value::from(self.key.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemAnswerAnswerNone {}

impl Shape for ResolverRoutesInputStepsItemAnswerAnswerNone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "none")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("none"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemAnswerAnswerRefused {}

impl Shape for ResolverRoutesInputStepsItemAnswerAnswerRefused {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "refused")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("refused"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemAnswerAnswerUnready {}

impl Shape for ResolverRoutesInputStepsItemAnswerAnswerUnready {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "unready")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("unready"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringChangesItemList {
    pub entries: Vec<nyaaai_contract::shapes::provider::ProviderEntry>,
}

impl Shape for ResolverRoutesInputStepsItemDuringChangesItemList {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "list")
        })?;
        let entries = shape::required(object, "entries", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::ProviderEntry::decode_at,
            )
        })?;
        Ok(Self { entries })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("list"));
        object.insert(
            "entries",
            Value::Array(self.entries.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringChangesItemAnswer {
    pub id: String,
    pub answer: ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswer,
}

impl Shape for ResolverRoutesInputStepsItemDuringChangesItemAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "answer")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let answer = shape::required(
            object,
            "answer",
            at,
            ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswer::decode_at,
        )?;
        Ok(Self { id, answer })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("answer"));
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("answer", self.answer.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringChangesItemRegistry {
    pub rows: Vec<String>,
}

impl Shape for ResolverRoutesInputStepsItemDuringChangesItemRegistry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "registry")
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("registry"));
        object.insert(
            "rows",
            Value::Array(
                self.rows
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringChangesItemZdr {
    pub on: bool,
}

impl Shape for ResolverRoutesInputStepsItemDuringChangesItemZdr {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "zdr")
        })?;
        let on = shape::required(object, "on", at, shape::flag)?;
        Ok(Self { on })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("zdr"));
        object.insert("on", Value::Bool(self.on));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringCallRoutes {
    pub model: String,
    pub use_: nyaaai_contract::shapes::model::ModelUse,
}

impl Shape for ResolverRoutesInputStepsItemDuringCallRoutes {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "routes")
        })?;
        let model = shape::required(object, "model", at, shape::text)?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        Ok(Self { model, use_ })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("routes"));
        object.insert("model", Value::from(self.model.as_str()));
        object.insert("use", self.use_.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringCallResolve {
    pub model: String,
    pub use_: nyaaai_contract::shapes::model::ModelUse,
}

impl Shape for ResolverRoutesInputStepsItemDuringCallResolve {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "resolve")
        })?;
        let model = shape::required(object, "model", at, shape::text)?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        Ok(Self { model, use_ })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("resolve"));
        object.insert("model", Value::from(self.model.as_str()));
        object.insert("use", self.use_.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringCallRows {
    pub use_: nyaaai_contract::shapes::model::ModelUse,
}

impl Shape for ResolverRoutesInputStepsItemDuringCallRows {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "rows")
        })?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        Ok(Self { use_ })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("rows"));
        object.insert("use", self.use_.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringCallCandidates {
    pub use_: nyaaai_contract::shapes::model::ModelUse,
}

impl Shape for ResolverRoutesInputStepsItemDuringCallCandidates {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "candidates")
        })?;
        let use_ = shape::required(
            object,
            "use",
            at,
            nyaaai_contract::shapes::model::ModelUse::decode_at,
        )?;
        Ok(Self { use_ })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("candidates"));
        object.insert("use", self.use_.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringCallSend {
    pub from: u64,
    pub candidate: u64,
    pub mode: Option<ResolverRoutesInputStepsItemDuringCallSendMode>,
    pub fail: Option<ResolverRoutesInputStepsItemDuringCallSendFail>,
}

impl Shape for ResolverRoutesInputStepsItemDuringCallSend {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "send")
        })?;
        let from = shape::required(object, "from", at, shape::natural)?;
        let candidate = shape::required(object, "candidate", at, shape::natural)?;
        let mode = shape::optional(
            object,
            "mode",
            at,
            ResolverRoutesInputStepsItemDuringCallSendMode::decode_at,
        )?;
        let fail = shape::optional(
            object,
            "fail",
            at,
            ResolverRoutesInputStepsItemDuringCallSendFail::decode_at,
        )?;
        Ok(Self {
            from,
            candidate,
            mode,
            fail,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("send"));
        object.insert("from", Value::Number(self.from as f64));
        object.insert("candidate", Value::Number(self.candidate as f64));
        if let Some(held) = &self.mode {
            object.insert("mode", held.encode());
        }
        if let Some(held) = &self.fail {
            object.insert("fail", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswer {
    Key(ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerKey),
    None(ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerNone),
    Refused(ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerRefused),
    Unready(ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerUnready),
}

impl Shape for ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "key" => {
                ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerKey::decode_at(value, at)
                    .map(Self::Key)
            }
            "none" => {
                ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerNone::decode_at(value, at)
                    .map(Self::None)
            }
            "refused" => {
                ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerRefused::decode_at(
                    value, at,
                )
                .map(Self::Refused)
            }
            "unready" => {
                ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerUnready::decode_at(
                    value, at,
                )
                .map(Self::Unready)
            }
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Key(held) => held.encode(),
            Self::None(held) => held.encode(),
            Self::Refused(held) => held.encode(),
            Self::Unready(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolverRoutesInputStepsItemDuringCallSendMode {
    Complete,
    Stream,
}

impl ResolverRoutesInputStepsItemDuringCallSendMode {
    pub const ALL: [Self; 2] = [Self::Complete, Self::Stream];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Stream => "stream",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ResolverRoutesInputStepsItemDuringCallSendMode {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringCallSendFail {
    pub status: i64,
    pub body: Value,
}

impl Shape for ResolverRoutesInputStepsItemDuringCallSendFail {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let status = shape::required(object, "status", at, shape::integer)?;
        let body = shape::required(object, "body", at, shape::any)?;
        Ok(Self { status, body })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("status", Value::Number(self.status as f64));
        object.insert("body", self.body.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerKey {
    pub key: String,
}

impl Shape for ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerKey {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "key")
        })?;
        let key = shape::required(object, "key", at, shape::text)?;
        Ok(Self { key })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("key"));
        object.insert("key", Value::from(self.key.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerNone {}

impl Shape for ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerNone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "none")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("none"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerRefused {}

impl Shape for ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerRefused {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "refused")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("refused"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerUnready {}

impl Shape for ResolverRoutesInputStepsItemDuringChangesItemAnswerAnswerUnready {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "unready")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("unready"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResult {
    pub steps: Vec<ResolverRoutesResultStepsItem>,
    pub transports: Vec<String>,
    pub stopped: Vec<String>,
    pub refreshed: Vec<nyaaai_contract::shapes::model::Via>,
    pub key_reads: i64,
}

impl Shape for ResolverRoutesResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, ResolverRoutesResultStepsItem::decode_at)
        })?;
        let transports = shape::required(object, "transports", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let stopped = shape::required(object, "stopped", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let refreshed = shape::required(object, "refreshed", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::model::Via::decode_at)
        })?;
        let key_reads = shape::required(object, "keyReads", at, shape::integer)?;
        Ok(Self {
            steps,
            transports,
            stopped,
            refreshed,
            key_reads,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "transports",
            Value::Array(
                self.transports
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "stopped",
            Value::Array(
                self.stopped
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        object.insert(
            "refreshed",
            Value::Array(self.refreshed.iter().map(|held| held.encode()).collect()),
        );
        object.insert("keyReads", Value::Number(self.key_reads as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItem {
    pub answer: ResolverRoutesResultStepsItemAnswer,
    pub built: i64,
    pub disposed: i64,
}

impl Shape for ResolverRoutesResultStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let answer = shape::required(
            object,
            "answer",
            at,
            ResolverRoutesResultStepsItemAnswer::decode_at,
        )?;
        let built = shape::required(object, "built", at, shape::integer)?;
        let disposed = shape::required(object, "disposed", at, shape::integer)?;
        Ok(Self {
            answer,
            built,
            disposed,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("answer", self.answer.encode());
        object.insert("built", Value::Number(self.built as f64));
        object.insert("disposed", Value::Number(self.disposed as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResolverRoutesResultStepsItemAnswer {
    Routes(ResolverRoutesResultStepsItemAnswerRoutes),
    Resolve(ResolverRoutesResultStepsItemAnswerResolve),
    Rows(ResolverRoutesResultStepsItemAnswerRows),
    Candidates(ResolverRoutesResultStepsItemAnswerCandidates),
    Changed(ResolverRoutesResultStepsItemAnswerChanged),
    Sent(ResolverRoutesResultStepsItemAnswerSent),
    Routed(ResolverRoutesResultStepsItemAnswerRouted),
    None(ResolverRoutesResultStepsItemAnswerNone),
}

impl Shape for ResolverRoutesResultStepsItemAnswer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "routes" => {
                ResolverRoutesResultStepsItemAnswerRoutes::decode_at(value, at).map(Self::Routes)
            }
            "resolve" => {
                ResolverRoutesResultStepsItemAnswerResolve::decode_at(value, at).map(Self::Resolve)
            }
            "rows" => ResolverRoutesResultStepsItemAnswerRows::decode_at(value, at).map(Self::Rows),
            "candidates" => ResolverRoutesResultStepsItemAnswerCandidates::decode_at(value, at)
                .map(Self::Candidates),
            "changed" => {
                ResolverRoutesResultStepsItemAnswerChanged::decode_at(value, at).map(Self::Changed)
            }
            "sent" => ResolverRoutesResultStepsItemAnswerSent::decode_at(value, at).map(Self::Sent),
            "routed" => {
                ResolverRoutesResultStepsItemAnswerRouted::decode_at(value, at).map(Self::Routed)
            }
            "none" => ResolverRoutesResultStepsItemAnswerNone::decode_at(value, at).map(Self::None),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Routes(held) => held.encode(),
            Self::Resolve(held) => held.encode(),
            Self::Rows(held) => held.encode(),
            Self::Candidates(held) => held.encode(),
            Self::Changed(held) => held.encode(),
            Self::Sent(held) => held.encode(),
            Self::Routed(held) => held.encode(),
            Self::None(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerRoutes {
    pub unresolved: Option<nyaaai_contract::shapes::model::Unresolved>,
    pub candidates: Vec<ResolverRoutesResultStepsItemAnswerRoutesCandidatesItem>,
    pub passed: Vec<nyaaai_contract::shapes::provider::Passed>,
}

impl Shape for ResolverRoutesResultStepsItemAnswerRoutes {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "routes")
        })?;
        let unresolved = shape::required(object, "unresolved", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::model::Unresolved::decode_at,
            )
        })?;
        let candidates = shape::required(object, "candidates", at, |value, at| {
            shape::list(
                value,
                at,
                ResolverRoutesResultStepsItemAnswerRoutesCandidatesItem::decode_at,
            )
        })?;
        let passed = shape::required(object, "passed", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::provider::Passed::decode_at,
            )
        })?;
        Ok(Self {
            unresolved,
            candidates,
            passed,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("routes"));
        object.insert(
            "unresolved",
            self.unresolved
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "candidates",
            Value::Array(self.candidates.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "passed",
            Value::Array(self.passed.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerResolve {
    pub unresolved: Option<nyaaai_contract::shapes::model::Unresolved>,
    pub candidate: Option<ResolverRoutesResultStepsItemAnswerResolveCandidate>,
}

impl Shape for ResolverRoutesResultStepsItemAnswerResolve {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "resolve")
        })?;
        let unresolved = shape::required(object, "unresolved", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::model::Unresolved::decode_at,
            )
        })?;
        let candidate = shape::required(object, "candidate", at, |value, at| {
            shape::nullable(
                value,
                at,
                ResolverRoutesResultStepsItemAnswerResolveCandidate::decode_at,
            )
        })?;
        Ok(Self {
            unresolved,
            candidate,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("resolve"));
        object.insert(
            "unresolved",
            self.unresolved
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "candidate",
            self.candidate
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerRows {
    pub options: Vec<ResolverRoutesResultStepsItemAnswerRowsOptionsItem>,
}

impl Shape for ResolverRoutesResultStepsItemAnswerRows {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "rows")
        })?;
        let options = shape::required(object, "options", at, |value, at| {
            shape::list(
                value,
                at,
                ResolverRoutesResultStepsItemAnswerRowsOptionsItem::decode_at,
            )
        })?;
        Ok(Self { options })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("rows"));
        object.insert(
            "options",
            Value::Array(self.options.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerCandidates {
    pub served: Vec<ResolverRoutesResultStepsItemAnswerCandidatesServedItem>,
}

impl Shape for ResolverRoutesResultStepsItemAnswerCandidates {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "candidates")
        })?;
        let served = shape::required(object, "served", at, |value, at| {
            shape::list(
                value,
                at,
                ResolverRoutesResultStepsItemAnswerCandidatesServedItem::decode_at,
            )
        })?;
        Ok(Self { served })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("candidates"));
        object.insert(
            "served",
            Value::Array(self.served.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerChanged {}

impl Shape for ResolverRoutesResultStepsItemAnswerChanged {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "changed")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("changed"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerSent {
    pub zdr_only: Option<bool>,
    pub url: Option<String>,
    pub authorization: Option<String>,
    pub failure: Option<nyaaai_contract::shapes::step::Failure>,
    pub thrown: Option<String>,
}

impl Shape for ResolverRoutesResultStepsItemAnswerSent {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "sent")
        })?;
        let zdr_only = shape::required(object, "zdrOnly", at, |value, at| {
            shape::nullable(value, at, shape::flag)
        })?;
        let url = shape::required(object, "url", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let authorization = shape::required(object, "authorization", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let failure = shape::required(object, "failure", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::step::Failure::decode_at)
        })?;
        let thrown = shape::required(object, "thrown", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            zdr_only,
            url,
            authorization,
            failure,
            thrown,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("sent"));
        object.insert(
            "zdrOnly",
            self.zdr_only
                .as_ref()
                .map_or(Value::Null, |held| Value::Bool(*held)),
        );
        object.insert(
            "url",
            self.url
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "authorization",
            self.authorization
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "failure",
            self.failure
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "thrown",
            self.thrown
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerRouted {
    pub thrown: Option<String>,
}

impl Shape for ResolverRoutesResultStepsItemAnswerRouted {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "routed")
        })?;
        let thrown = shape::required(object, "thrown", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self { thrown })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("routed"));
        object.insert(
            "thrown",
            self.thrown
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerNone {}

impl Shape for ResolverRoutesResultStepsItemAnswerNone {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "none")
        })?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("none"));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerRoutesCandidatesItem {
    pub entry: String,
    pub kind: nyaaai_contract::shapes::provider::ProviderKindName,
    pub binding: String,
    pub status: ResolverRoutesResultStepsItemAnswerRoutesCandidatesItemStatus,
    pub zdr_only: bool,
    pub path: String,
}

impl Shape for ResolverRoutesResultStepsItemAnswerRoutesCandidatesItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let kind = shape::required(
            object,
            "kind",
            at,
            nyaaai_contract::shapes::provider::ProviderKindName::decode_at,
        )?;
        let binding = shape::required(object, "binding", at, shape::text)?;
        let status = shape::required(
            object,
            "status",
            at,
            ResolverRoutesResultStepsItemAnswerRoutesCandidatesItemStatus::decode_at,
        )?;
        let zdr_only = shape::required(object, "zdrOnly", at, shape::flag)?;
        let path = shape::required(object, "path", at, shape::text)?;
        Ok(Self {
            entry,
            kind,
            binding,
            status,
            zdr_only,
            path,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("kind", self.kind.encode());
        object.insert("binding", Value::from(self.binding.as_str()));
        object.insert("status", self.status.encode());
        object.insert("zdrOnly", Value::Bool(self.zdr_only));
        object.insert("path", Value::from(self.path.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerResolveCandidate {
    pub entry: String,
    pub kind: nyaaai_contract::shapes::provider::ProviderKindName,
    pub binding: String,
    pub status: ResolverRoutesResultStepsItemAnswerResolveCandidateStatus,
    pub zdr_only: bool,
    pub path: String,
}

impl Shape for ResolverRoutesResultStepsItemAnswerResolveCandidate {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let entry = shape::required(object, "entry", at, shape::text)?;
        let kind = shape::required(
            object,
            "kind",
            at,
            nyaaai_contract::shapes::provider::ProviderKindName::decode_at,
        )?;
        let binding = shape::required(object, "binding", at, shape::text)?;
        let status = shape::required(
            object,
            "status",
            at,
            ResolverRoutesResultStepsItemAnswerResolveCandidateStatus::decode_at,
        )?;
        let zdr_only = shape::required(object, "zdrOnly", at, shape::flag)?;
        let path = shape::required(object, "path", at, shape::text)?;
        Ok(Self {
            entry,
            kind,
            binding,
            status,
            zdr_only,
            path,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("entry", Value::from(self.entry.as_str()));
        object.insert("kind", self.kind.encode());
        object.insert("binding", Value::from(self.binding.as_str()));
        object.insert("status", self.status.encode());
        object.insert("zdrOnly", Value::Bool(self.zdr_only));
        object.insert("path", Value::from(self.path.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerRowsOptionsItem {
    pub key: String,
    pub status: ResolverRoutesResultStepsItemAnswerRowsOptionsItemStatus,
    pub via: Option<nyaaai_contract::shapes::model::Via>,
}

impl Shape for ResolverRoutesResultStepsItemAnswerRowsOptionsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let key = shape::required(object, "key", at, shape::text)?;
        let status = shape::required(
            object,
            "status",
            at,
            ResolverRoutesResultStepsItemAnswerRowsOptionsItemStatus::decode_at,
        )?;
        let via = shape::required(object, "via", at, |value, at| {
            shape::nullable(value, at, nyaaai_contract::shapes::model::Via::decode_at)
        })?;
        Ok(Self { key, status, via })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("key", Value::from(self.key.as_str()));
        object.insert("status", self.status.encode());
        object.insert(
            "via",
            self.via.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverRoutesResultStepsItemAnswerCandidatesServedItem {
    pub key: String,
    pub status: ResolverRoutesResultStepsItemAnswerCandidatesServedItemStatus,
    pub via: nyaaai_contract::shapes::model::Via,
}

impl Shape for ResolverRoutesResultStepsItemAnswerCandidatesServedItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let key = shape::required(object, "key", at, shape::text)?;
        let status = shape::required(
            object,
            "status",
            at,
            ResolverRoutesResultStepsItemAnswerCandidatesServedItemStatus::decode_at,
        )?;
        let via = shape::required(
            object,
            "via",
            at,
            nyaaai_contract::shapes::model::Via::decode_at,
        )?;
        Ok(Self { key, status, via })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("key", Value::from(self.key.as_str()));
        object.insert("status", self.status.encode());
        object.insert("via", self.via.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolverRoutesResultStepsItemAnswerRoutesCandidatesItemStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl ResolverRoutesResultStepsItemAnswerRoutesCandidatesItemStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ResolverRoutesResultStepsItemAnswerRoutesCandidatesItemStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolverRoutesResultStepsItemAnswerResolveCandidateStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl ResolverRoutesResultStepsItemAnswerResolveCandidateStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ResolverRoutesResultStepsItemAnswerResolveCandidateStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolverRoutesResultStepsItemAnswerRowsOptionsItemStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl ResolverRoutesResultStepsItemAnswerRowsOptionsItemStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ResolverRoutesResultStepsItemAnswerRowsOptionsItemStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResolverRoutesResultStepsItemAnswerCandidatesServedItemStatus {
    Ok,
    Retired,
    NoZdrHost,
    NoTools,
    Unknown,
}

impl ResolverRoutesResultStepsItemAnswerCandidatesServedItemStatus {
    pub const ALL: [Self; 5] = [
        Self::Ok,
        Self::Retired,
        Self::NoZdrHost,
        Self::NoTools,
        Self::Unknown,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Retired => "retired",
            Self::NoZdrHost => "noZdrHost",
            Self::NoTools => "noTools",
            Self::Unknown => "unknown",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for ResolverRoutesResultStepsItemAnswerCandidatesServedItemStatus {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetryWaitInput {
    pub options: nyaaai_contract::shapes::health::RetryOptions,
    pub draws: Vec<f64>,
    pub rows: Vec<RetryWaitInputRowsItem>,
}

impl Shape for RetryWaitInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let options = shape::required(
            object,
            "options",
            at,
            nyaaai_contract::shapes::health::RetryOptions::decode_at,
        )?;
        let draws = shape::required(object, "draws", at, |value, at| {
            shape::list_within(value, at, Some(1), None, shape::number)
        })?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, RetryWaitInputRowsItem::decode_at)
        })?;
        Ok(Self {
            options,
            draws,
            rows,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("options", self.options.encode());
        object.insert(
            "draws",
            Value::Array(self.draws.iter().map(|held| Value::Number(*held)).collect()),
        );
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetryWaitInputRowsItem {
    pub attempt: i64,
    pub waited_ms: f64,
    pub retry_after_ms: Option<i64>,
}

impl Shape for RetryWaitInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let attempt = shape::required(object, "attempt", at, shape::integer)?;
        let waited_ms = shape::required(object, "waitedMs", at, shape::number)?;
        let retry_after_ms = shape::optional(object, "retryAfterMs", at, shape::integer)?;
        Ok(Self {
            attempt,
            waited_ms,
            retry_after_ms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("attempt", Value::Number(self.attempt as f64));
        object.insert("waitedMs", Value::Number(self.waited_ms));
        if let Some(held) = &self.retry_after_ms {
            object.insert("retryAfterMs", Value::Number(*held as f64));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetryWaitResult {
    pub rows: Vec<RetryWaitResultRowsItem>,
}

impl Shape for RetryWaitResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, RetryWaitResultRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetryWaitResultRowsItem {
    pub wait_ms: Option<f64>,
    pub draws: i64,
}

impl Shape for RetryWaitResultRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let wait_ms = shape::required(object, "waitMs", at, |value, at| {
            shape::nullable(value, at, shape::number)
        })?;
        let draws = shape::required(object, "draws", at, shape::integer)?;
        Ok(Self { wait_ms, draws })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "waitMs",
            self.wait_ms
                .as_ref()
                .map_or(Value::Null, |held| Value::Number(*held)),
        );
        object.insert("draws", Value::Number(self.draws as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetryLifetimeInput {}

impl Shape for RetryLifetimeInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        shape::object(value, at)?;
        Ok(Self {})
    }

    fn encode(&self) -> Value {
        Value::Object(Object::new())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetryLifetimeResult {
    pub rows: Vec<RetryLifetimeResultRowsItem>,
}

impl Shape for RetryLifetimeResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, RetryLifetimeResultRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetryLifetimeResultRowsItem {
    pub notice: nyaaai_contract::shapes::provider::NoticeKind,
    pub lifetime: RetryLifetimeResultRowsItemLifetime,
}

impl Shape for RetryLifetimeResultRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let notice = shape::required(
            object,
            "notice",
            at,
            nyaaai_contract::shapes::provider::NoticeKind::decode_at,
        )?;
        let lifetime = shape::required(
            object,
            "lifetime",
            at,
            RetryLifetimeResultRowsItemLifetime::decode_at,
        )?;
        Ok(Self { notice, lifetime })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("notice", self.notice.encode());
        object.insert("lifetime", self.lifetime.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetryLifetimeResultRowsItemLifetime {
    Outcome,
    MidTurn,
}

impl RetryLifetimeResultRowsItemLifetime {
    pub const ALL: [Self; 2] = [Self::Outcome, Self::MidTurn];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Outcome => "outcome",
            Self::MidTurn => "midTurn",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for RetryLifetimeResultRowsItemLifetime {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolStepInput {
    pub pause_ms: f64,
    pub runs: Vec<PoolStepInputRunsItem>,
}

impl Shape for PoolStepInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let pause_ms = shape::required(object, "pauseMs", at, shape::number)?;
        let runs = shape::required(object, "runs", at, |value, at| {
            shape::list(value, at, PoolStepInputRunsItem::decode_at)
        })?;
        Ok(Self { pause_ms, runs })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("pauseMs", Value::Number(self.pause_ms));
        object.insert(
            "runs",
            Value::Array(self.runs.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolStepInputRunsItem {
    pub start: nyaaai_contract::shapes::health::PoolState,
    pub steps: Vec<PoolStepInputRunsItemStepsItem>,
}

impl Shape for PoolStepInputRunsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let start = shape::required(
            object,
            "start",
            at,
            nyaaai_contract::shapes::health::PoolState::decode_at,
        )?;
        let steps = shape::required(object, "steps", at, |value, at| {
            shape::list(value, at, PoolStepInputRunsItemStepsItem::decode_at)
        })?;
        Ok(Self { start, steps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("start", self.start.encode());
        object.insert(
            "steps",
            Value::Array(self.steps.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolStepInputRunsItemStepsItem {
    pub event: nyaaai_contract::shapes::health::PoolEvent,
    pub now_ms: f64,
}

impl Shape for PoolStepInputRunsItemStepsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let event = shape::required(
            object,
            "event",
            at,
            nyaaai_contract::shapes::health::PoolEvent::decode_at,
        )?;
        let now_ms = shape::required(object, "nowMs", at, shape::number)?;
        Ok(Self { event, now_ms })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("event", self.event.encode());
        object.insert("nowMs", Value::Number(self.now_ms));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolStepResult {
    pub runs: Vec<PoolStepResultRunsItem>,
}

impl Shape for PoolStepResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let runs = shape::required(object, "runs", at, |value, at| {
            shape::list(value, at, PoolStepResultRunsItem::decode_at)
        })?;
        Ok(Self { runs })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "runs",
            Value::Array(self.runs.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolStepResultRunsItem {
    pub states: Vec<nyaaai_contract::shapes::health::PoolState>,
}

impl Shape for PoolStepResultRunsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let states = shape::required(object, "states", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::health::PoolState::decode_at,
            )
        })?;
        Ok(Self { states })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "states",
            Value::Array(self.states.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolVerdictInput {
    pub rows: Vec<PoolVerdictInputRowsItem>,
}

impl Shape for PoolVerdictInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, PoolVerdictInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolVerdictInputRowsItem {
    pub state: nyaaai_contract::shapes::health::PoolState,
    pub exempt: bool,
    pub now_ms: f64,
}

impl Shape for PoolVerdictInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let state = shape::required(
            object,
            "state",
            at,
            nyaaai_contract::shapes::health::PoolState::decode_at,
        )?;
        let exempt = shape::required(object, "exempt", at, shape::flag)?;
        let now_ms = shape::required(object, "nowMs", at, shape::number)?;
        Ok(Self {
            state,
            exempt,
            now_ms,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("state", self.state.encode());
        object.insert("exempt", Value::Bool(self.exempt));
        object.insert("nowMs", Value::Number(self.now_ms));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolVerdictResult {
    pub verdicts: Vec<Option<nyaaai_contract::shapes::health::PoolVerdict>>,
}

impl Shape for PoolVerdictResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let verdicts = shape::required(object, "verdicts", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(
                    value,
                    at,
                    nyaaai_contract::shapes::health::PoolVerdict::decode_at,
                )
            })
        })?;
        Ok(Self { verdicts })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "verdicts",
            Value::Array(
                self.verdicts
                    .iter()
                    .map(|held| held.as_ref().map_or(Value::Null, |held| held.encode()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolProbeAtInput {
    pub rows: Vec<PoolProbeAtInputRowsItem>,
}

impl Shape for PoolProbeAtInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, PoolProbeAtInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolProbeAtInputRowsItem {
    pub now_ms: f64,
    pub eta: f64,
}

impl Shape for PoolProbeAtInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let now_ms = shape::required(object, "nowMs", at, shape::number)?;
        let eta = shape::required(object, "eta", at, shape::number)?;
        Ok(Self { now_ms, eta })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("nowMs", Value::Number(self.now_ms));
        object.insert("eta", Value::Number(self.eta));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PoolProbeAtResult {
    pub times: Vec<f64>,
    pub balance_cool_ms: f64,
    pub probe_floor_ms: f64,
    pub allowance_sources: Vec<String>,
}

impl Shape for PoolProbeAtResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let times = shape::required(object, "times", at, |value, at| {
            shape::list(value, at, shape::number)
        })?;
        let balance_cool_ms = shape::required(object, "balanceCoolMs", at, shape::number)?;
        let probe_floor_ms = shape::required(object, "probeFloorMs", at, shape::number)?;
        let allowance_sources = shape::required(object, "allowanceSources", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            times,
            balance_cool_ms,
            probe_floor_ms,
            allowance_sources,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "times",
            Value::Array(self.times.iter().map(|held| Value::Number(*held)).collect()),
        );
        object.insert("balanceCoolMs", Value::Number(self.balance_cool_ms));
        object.insert("probeFloorMs", Value::Number(self.probe_floor_ms));
        object.insert(
            "allowanceSources",
            Value::Array(
                self.allowance_sources
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostStepInput {
    pub rows: Vec<CostStepInputRowsItem>,
}

impl Shape for CostStepInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, CostStepInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostStepInputRowsItem {
    pub usage: Option<nyaaai_contract::shapes::step::NormalizedUsage>,
    pub price: Option<CostStepInputRowsItemPrice>,
    pub long_cache: bool,
}

impl Shape for CostStepInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let usage = shape::optional(
            object,
            "usage",
            at,
            nyaaai_contract::shapes::step::NormalizedUsage::decode_at,
        )?;
        let price = shape::optional(object, "price", at, CostStepInputRowsItemPrice::decode_at)?;
        let long_cache = shape::required(object, "longCache", at, shape::flag)?;
        Ok(Self {
            usage,
            price,
            long_cache,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.usage {
            object.insert("usage", held.encode());
        }
        if let Some(held) = &self.price {
            object.insert("price", held.encode());
        }
        object.insert("longCache", Value::Bool(self.long_cache));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CostStepInputRowsItemPrice {
    Price(nyaaai_contract::shapes::model::Price),
    Free,
    Unknown,
}

impl Shape for CostStepInputRowsItemPrice {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = nyaaai_contract::shapes::model::Price::decode_at(value, at) {
            return Ok(Self::Price(held));
        }
        if shape::is_text(value, at, "free").is_ok() {
            return Ok(Self::Free);
        }
        if shape::is_text(value, at, "unknown").is_ok() {
            return Ok(Self::Unknown);
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Price(held) => held.encode(),
            Self::Free => Value::from("free"),
            Self::Unknown => Value::from("unknown"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostStepResult {
    pub costs: Vec<nyaaai_contract::shapes::usage::Cost>,
}

impl Shape for CostStepResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let costs = shape::required(object, "costs", at, |value, at| {
            shape::list(value, at, nyaaai_contract::shapes::usage::Cost::decode_at)
        })?;
        Ok(Self { costs })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "costs",
            Value::Array(self.costs.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostPriceInput {
    pub values: Vec<Value>,
}

impl Shape for CostPriceInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::any)
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("values", Value::Array(self.values.clone()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostPriceResult {
    pub prices: Vec<Option<f64>>,
}

impl Shape for CostPriceResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let prices = shape::required(object, "prices", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::number)
            })
        })?;
        Ok(Self { prices })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "prices",
            Value::Array(
                self.prices
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostReportedInput {
    pub rows: Vec<CostReportedInputRowsItem>,
}

impl Shape for CostReportedInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, CostReportedInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostReportedInputRowsItem {
    pub cost: Option<f64>,
    pub byok: Option<bool>,
    pub upstream_cost: Option<f64>,
}

impl Shape for CostReportedInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let cost = shape::optional(object, "cost", at, shape::number)?;
        let byok = shape::optional(object, "byok", at, shape::flag)?;
        let upstream_cost = shape::optional(object, "upstreamCost", at, shape::number)?;
        Ok(Self {
            cost,
            byok,
            upstream_cost,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.cost {
            object.insert("cost", Value::Number(*held));
        }
        if let Some(held) = &self.byok {
            object.insert("byok", Value::Bool(*held));
        }
        if let Some(held) = &self.upstream_cost {
            object.insert("upstreamCost", Value::Number(*held));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostReportedResult {
    pub values: Vec<Option<f64>>,
}

impl Shape for CostReportedResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::number)
            })
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(
                self.values
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::Number(*held))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostLongCacheInput {
    pub rows: Vec<CostLongCacheInputRowsItem>,
}

impl Shape for CostLongCacheInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, CostLongCacheInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostLongCacheInputRowsItem {
    pub tag: nyaaai_contract::shapes::usage::CallTag,
    pub view: Option<CostLongCacheInputRowsItemView>,
}

impl Shape for CostLongCacheInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let tag = shape::required(
            object,
            "tag",
            at,
            nyaaai_contract::shapes::usage::CallTag::decode_at,
        )?;
        let view = shape::required(object, "view", at, |value, at| {
            shape::nullable(value, at, CostLongCacheInputRowsItemView::decode_at)
        })?;
        Ok(Self { tag, view })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("tag", self.tag.encode());
        object.insert(
            "view",
            self.view.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostLongCacheInputRowsItemView {
    pub cache_write1h: Option<()>,
}

impl Shape for CostLongCacheInputRowsItemView {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let cache_write1h = shape::optional(object, "cacheWrite1h", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        Ok(Self { cache_write1h })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if self.cache_write1h.is_some() {
            object.insert("cacheWrite1h", Value::Bool(true));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CostLongCacheResult {
    pub values: Vec<bool>,
}

impl Shape for CostLongCacheResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let values = shape::required(object, "values", at, |value, at| {
            shape::list(value, at, shape::flag)
        })?;
        Ok(Self { values })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "values",
            Value::Array(self.values.iter().map(|held| Value::Bool(*held)).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRecordInput {
    pub call: Value,
    pub non_finite: Vec<UsageRecordInputNonFiniteItem>,
    pub id: String,
    pub home: String,
    pub now_ms: i64,
    pub offsets: Vec<UsageRecordInputOffsetsItem>,
}

impl Shape for UsageRecordInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let call = shape::required(object, "call", at, shape::any)?;
        let non_finite = shape::required(object, "nonFinite", at, |value, at| {
            shape::list(value, at, UsageRecordInputNonFiniteItem::decode_at)
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let home = shape::required(object, "home", at, shape::text)?;
        let now_ms = shape::required(object, "nowMs", at, shape::integer)?;
        let offsets = shape::required(object, "offsets", at, |value, at| {
            shape::list(value, at, UsageRecordInputOffsetsItem::decode_at)
        })?;
        Ok(Self {
            call,
            non_finite,
            id,
            home,
            now_ms,
            offsets,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("call", self.call.clone());
        object.insert(
            "nonFinite",
            Value::Array(self.non_finite.iter().map(|held| held.encode()).collect()),
        );
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("home", Value::from(self.home.as_str()));
        object.insert("nowMs", Value::Number(self.now_ms as f64));
        object.insert(
            "offsets",
            Value::Array(self.offsets.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRecordInputNonFiniteItem {
    pub path: String,
    pub is: UsageRecordInputNonFiniteItemIs,
}

impl Shape for UsageRecordInputNonFiniteItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let path = shape::required(object, "path", at, shape::text)?;
        let is = shape::required(object, "is", at, UsageRecordInputNonFiniteItemIs::decode_at)?;
        Ok(Self { path, is })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("path", Value::from(self.path.as_str()));
        object.insert("is", self.is.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRecordInputOffsetsItem {
    pub from_ms: i64,
    pub minutes: i64,
}

impl Shape for UsageRecordInputOffsetsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let from_ms = shape::required(object, "fromMs", at, shape::integer)?;
        let minutes = shape::required(object, "minutes", at, shape::integer)?;
        Ok(Self { from_ms, minutes })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("fromMs", Value::Number(self.from_ms as f64));
        object.insert("minutes", Value::Number(self.minutes as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsageRecordInputNonFiniteItemIs {
    NotANumber,
    PlusInfinity,
    MinusInfinity,
}

impl UsageRecordInputNonFiniteItemIs {
    pub const ALL: [Self; 3] = [Self::NotANumber, Self::PlusInfinity, Self::MinusInfinity];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotANumber => "notANumber",
            Self::PlusInfinity => "plusInfinity",
            Self::MinusInfinity => "minusInfinity",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for UsageRecordInputNonFiniteItemIs {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageRecordResult {
    pub record: Option<nyaaai_contract::shapes::usage::CallRecord>,
    pub refused_at: Option<String>,
    pub line: Option<String>,
    pub day: Option<String>,
}

impl Shape for UsageRecordResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let record = shape::required(object, "record", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::usage::CallRecord::decode_at,
            )
        })?;
        let refused_at = shape::required(object, "refusedAt", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let line = shape::required(object, "line", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let day = shape::required(object, "day", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            record,
            refused_at,
            line,
            day,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "record",
            self.record
                .as_ref()
                .map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "refusedAt",
            self.refused_at
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "line",
            self.line
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "day",
            self.day
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageStatsInput {
    pub stats: Value,
    pub non_finite: Vec<UsageStatsInputNonFiniteItem>,
    pub id: String,
}

impl Shape for UsageStatsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let stats = shape::required(object, "stats", at, shape::any)?;
        let non_finite = shape::required(object, "nonFinite", at, |value, at| {
            shape::list(value, at, UsageStatsInputNonFiniteItem::decode_at)
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        Ok(Self {
            stats,
            non_finite,
            id,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("stats", self.stats.clone());
        object.insert(
            "nonFinite",
            Value::Array(self.non_finite.iter().map(|held| held.encode()).collect()),
        );
        object.insert("id", Value::from(self.id.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageStatsInputNonFiniteItem {
    pub path: String,
    pub is: UsageStatsInputNonFiniteItemIs,
}

impl Shape for UsageStatsInputNonFiniteItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let path = shape::required(object, "path", at, shape::text)?;
        let is = shape::required(object, "is", at, UsageStatsInputNonFiniteItemIs::decode_at)?;
        Ok(Self { path, is })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("path", Value::from(self.path.as_str()));
        object.insert("is", self.is.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UsageStatsInputNonFiniteItemIs {
    NotANumber,
    PlusInfinity,
    MinusInfinity,
}

impl UsageStatsInputNonFiniteItemIs {
    pub const ALL: [Self; 3] = [Self::NotANumber, Self::PlusInfinity, Self::MinusInfinity];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotANumber => "notANumber",
            Self::PlusInfinity => "plusInfinity",
            Self::MinusInfinity => "minusInfinity",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for UsageStatsInputNonFiniteItemIs {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        value
            .as_str()
            .and_then(Self::parse)
            .ok_or_else(|| shape::refused(at, "not one of the values"))
    }

    fn encode(&self) -> Value {
        Value::from(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageStatsResult {
    pub line: Option<nyaaai_contract::shapes::usage::StatsLine>,
    pub refused_at: Option<String>,
    pub text: Option<String>,
}

impl Shape for UsageStatsResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let line = shape::required(object, "line", at, |value, at| {
            shape::nullable(
                value,
                at,
                nyaaai_contract::shapes::usage::StatsLine::decode_at,
            )
        })?;
        let refused_at = shape::required(object, "refusedAt", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let text = shape::required(object, "text", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        Ok(Self {
            line,
            refused_at,
            text,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "line",
            self.line.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        object.insert(
            "refusedAt",
            self.refused_at
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "text",
            self.text
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageLinesInput {
    pub text: String,
}

impl Shape for UsageLinesInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let text = shape::required(object, "text", at, shape::text)?;
        Ok(Self { text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("text", Value::from(self.text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageLinesResult {
    pub lines: Vec<nyaaai_contract::shapes::usage::UsageLine>,
    pub dropped: i64,
}

impl Shape for UsageLinesResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let lines = shape::required(object, "lines", at, |value, at| {
            shape::list(
                value,
                at,
                nyaaai_contract::shapes::usage::UsageLine::decode_at,
            )
        })?;
        let dropped = shape::required(object, "dropped", at, shape::integer)?;
        Ok(Self { lines, dropped })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "lines",
            Value::Array(self.lines.iter().map(|held| held.encode()).collect()),
        );
        object.insert("dropped", Value::Number(self.dropped as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAppInput {
    pub rows: Vec<UsageAppInputRowsItem>,
}

impl Shape for UsageAppInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let rows = shape::required(object, "rows", at, |value, at| {
            shape::list(value, at, UsageAppInputRowsItem::decode_at)
        })?;
        Ok(Self { rows })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "rows",
            Value::Array(self.rows.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAppInputRowsItem {
    pub folder: String,
    pub home: String,
}

impl Shape for UsageAppInputRowsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let folder = shape::required(object, "folder", at, shape::text)?;
        let home = shape::required(object, "home", at, shape::text)?;
        Ok(Self { folder, home })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("folder", Value::from(self.folder.as_str()));
        object.insert("home", Value::from(self.home.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageAppResult {
    pub apps: Vec<String>,
}

impl Shape for UsageAppResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let apps = shape::required(object, "apps", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { apps })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "apps",
            Value::Array(
                self.apps
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageDayInput {
    pub offsets: Vec<UsageDayInputOffsetsItem>,
    pub instants: Vec<i64>,
}

impl Shape for UsageDayInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let offsets = shape::required(object, "offsets", at, |value, at| {
            shape::list(value, at, UsageDayInputOffsetsItem::decode_at)
        })?;
        let instants = shape::required(object, "instants", at, |value, at| {
            shape::list(value, at, shape::integer)
        })?;
        Ok(Self { offsets, instants })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "offsets",
            Value::Array(self.offsets.iter().map(|held| held.encode()).collect()),
        );
        object.insert(
            "instants",
            Value::Array(
                self.instants
                    .iter()
                    .map(|held| Value::Number(*held as f64))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageDayInputOffsetsItem {
    pub from_ms: i64,
    pub minutes: i64,
}

impl Shape for UsageDayInputOffsetsItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let from_ms = shape::required(object, "fromMs", at, shape::integer)?;
        let minutes = shape::required(object, "minutes", at, shape::integer)?;
        Ok(Self { from_ms, minutes })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("fromMs", Value::Number(self.from_ms as f64));
        object.insert("minutes", Value::Number(self.minutes as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageDayResult {
    pub days: Vec<Option<String>>,
}

impl Shape for UsageDayResult {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let days = shape::required(object, "days", at, |value, at| {
            shape::list(value, at, |value, at| {
                shape::nullable(value, at, shape::text)
            })
        })?;
        Ok(Self { days })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "days",
            Value::Array(
                self.days
                    .iter()
                    .map(|held| {
                        held.as_ref()
                            .map_or(Value::Null, |held| Value::from(held.as_str()))
                    })
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}
