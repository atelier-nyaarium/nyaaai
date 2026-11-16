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
