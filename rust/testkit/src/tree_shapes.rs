//! Generated from schema/tree.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use nyaaai_contract::json::{Object, Value};
use nyaaai_contract::shape::{self, Path, Refused, Shape};

#[derive(Debug, Clone, PartialEq)]
pub struct TreeEntry {
    pub path: String,
    pub text: Option<String>,
    pub base64: Option<String>,
    pub link: Option<String>,
}

impl Shape for TreeEntry {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let path = shape::required(object, "path", at, shape::text)?;
        let text = shape::optional(object, "text", at, shape::text)?;
        let base64 = shape::optional(object, "base64", at, shape::text)?;
        let link = shape::optional(object, "link", at, shape::text)?;
        Ok(Self {
            path,
            text,
            base64,
            link,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("path", Value::from(self.path.as_str()));
        if let Some(held) = &self.text {
            object.insert("text", Value::from(held.as_str()));
        }
        if let Some(held) = &self.base64 {
            object.insert("base64", Value::from(held.as_str()));
        }
        if let Some(held) = &self.link {
            object.insert("link", Value::from(held.as_str()));
        }
        Value::Object(object)
    }
}

/// Decodes then encodes a named shape.
pub fn normalize(shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
    let at = &mut Path::new();
    Some(match shape {
        "TreeEntry" => TreeEntry::decode_at(value, at).map(|held| held.encode()),
        _ => return None,
    })
}
