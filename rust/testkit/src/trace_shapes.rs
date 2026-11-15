//! Generated from schema/trace.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use nyaaai_contract::json::{Object, Record, Value};
use nyaaai_contract::shape::{self, Path, Refused, Shape};

#[derive(Debug, Clone, PartialEq)]
pub struct TraceExchange {
    pub request: Value,
    pub status: u64,
    pub headers: Record<String>,
    pub chunks: Vec<String>,
}

impl Shape for TraceExchange {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let request = shape::required(object, "request", at, shape::any)?;
        let status = shape::required(object, "status", at, |value, at| {
            shape::natural(value, at)
                .and_then(|held| shape::at_least(held, 100, at))
                .and_then(|held| shape::at_most(held, 599, at))
        })?;
        let headers = shape::required(object, "headers", at, |value, at| {
            shape::record(value, at, shape::text)
        })?;
        let chunks = shape::required(object, "chunks", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self {
            request,
            status,
            headers,
            chunks,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("request", self.request.clone());
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
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trace {
    pub exchanges: Vec<TraceExchange>,
}

impl Shape for Trace {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "format", at, |value, at| {
            shape::is_number(value, at, 1.0)
        })?;
        let exchanges = shape::required(object, "exchanges", at, |value, at| {
            shape::list(value, at, TraceExchange::decode_at)
        })?;
        Ok(Self { exchanges })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("format", Value::Number(1.0));
        object.insert(
            "exchanges",
            Value::Array(self.exchanges.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

/// Decodes then encodes a named shape.
pub fn normalize(shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
    let at = &mut Path::new();
    Some(match shape {
        "TraceExchange" => TraceExchange::decode_at(value, at).map(|held| held.encode()),
        "Trace" => Trace::decode_at(value, at).map(|held| held.encode()),
        _ => return None,
    })
}
