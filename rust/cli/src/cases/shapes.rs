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
}

impl Shape for SseItemsInput {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let chunks = shape::required(object, "chunks", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { chunks })
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
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SseItemsResult {
    pub pushed: Vec<Vec<nyaaai_contract::shapes::step::SseItem>>,
    pub ended: Vec<nyaaai_contract::shapes::step::SseItem>,
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
        Ok(Self { pushed, ended })
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
    Value(FailureOfInputThrownItemValue),
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
            "value" => FailureOfInputThrownItemValue::decode_at(value, at).map(Self::Value),
            _ => Err(shape::no_branch(at, "make")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Failed(held) => held.encode(),
            Self::Provider(held) => held.encode(),
            Self::Error(held) => held.encode(),
            Self::Value(held) => held.encode(),
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
pub struct FailureOfInputThrownItemValue {
    pub value: Value,
}

impl Shape for FailureOfInputThrownItemValue {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "make", at, |value, at| {
            shape::is_text(value, at, "value")
        })?;
        let value_field = shape::required(object, "value", at, shape::any)?;
        Ok(Self { value: value_field })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("make", Value::from("value"));
        object.insert("value", self.value.clone());
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
