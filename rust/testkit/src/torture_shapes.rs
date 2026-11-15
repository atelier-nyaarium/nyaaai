//! Generated from schema/torture.output.json by scripts/contract.ts. Do not edit.

#![allow(
    clippy::enum_variant_names,
    clippy::large_enum_variant,
    clippy::ptr_arg
)]

use nyaaai_contract::json::{Object, Record, Value};
use nyaaai_contract::shape::{self, Path, Refused, Shape};

use super::torture_shapes_hooks as hooks;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Red,
    Green,
    LightBlue,
}

impl Color {
    pub const ALL: [Self; 3] = [Self::Red, Self::Green, Self::LightBlue];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Green => "green",
            Self::LightBlue => "light-blue",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for Color {
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

pub type Name = String;

pub fn decode_name(value: &Value, at: &mut Path) -> Result<Name, Refused> {
    shape::text_within(value, at, Some(1), Some(3))
}

pub fn encode_name(held: &Name) -> Value {
    Value::from(held.as_str())
}

pub type Names = Vec<Name>;

pub fn decode_names(value: &Value, at: &mut Path) -> Result<Names, Refused> {
    shape::list_within(value, at, None, Some(2), decode_name)
}

pub fn encode_names(held: &Names) -> Value {
    Value::Array(held.iter().map(encode_name).collect())
}

#[derive(Debug, Clone, PartialEq)]
pub struct Leaf {
    pub value: i64,
    pub color: Option<Color>,
}

impl Shape for Leaf {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "leaf")
        })?;
        let value_field = shape::required(object, "value", at, shape::integer)?;
        let color = shape::optional(object, "color", at, Color::decode_at)?;
        Ok(Self {
            value: value_field,
            color,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("leaf"));
        object.insert("value", Value::Number(self.value as f64));
        if let Some(held) = &self.color {
            object.insert("color", held.encode());
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Branch {
    pub kids: Vec<Tree>,
}

impl Shape for Branch {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "kind", at, |value, at| {
            shape::is_text(value, at, "branch")
        })?;
        let kids = shape::required(object, "kids", at, |value, at| {
            shape::list(value, at, Tree::decode_at)
        })?;
        Ok(Self { kids })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("kind", Value::from("branch"));
        object.insert(
            "kids",
            Value::Array(self.kids.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tree {
    Leaf(Leaf),
    Branch(Branch),
}

impl Shape for Tree {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "kind", at)? {
            "leaf" => Leaf::decode_at(value, at).map(Self::Leaf),
            "branch" => Branch::decode_at(value, at).map(Self::Branch),
            _ => Err(shape::no_branch(at, "kind")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Leaf(held) => held.encode(),
            Self::Branch(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chain {
    pub value: f64,
    pub next: Option<Box<Chain>>,
}

impl Shape for Chain {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let value_field = shape::required(object, "value", at, shape::number)?;
        let next = shape::required(object, "next", at, |value, at| {
            shape::nullable(value, at, |value, at| {
                Chain::decode_at(value, at).map(Box::new)
            })
        })?;
        Ok(Self {
            value: value_field,
            next,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("value", Value::Number(self.value));
        object.insert(
            "next",
            self.next.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Presence {
    pub optional: Option<String>,
    pub nullable: Option<String>,
    pub both: Option<Option<String>>,
    pub defaulted: i64,
    pub defaulted_null: Option<String>,
    pub list: Vec<String>,
    pub any: Option<Value>,
    pub must: Value,
}

impl Shape for Presence {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let optional = shape::optional(object, "optional", at, shape::text)?;
        let nullable = shape::required(object, "nullable", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let both = shape::optional(object, "both", at, |value, at| {
            shape::nullable(value, at, shape::text)
        })?;
        let defaulted = shape::defaulted(object, "defaulted", at, r#"3"#, shape::integer)?;
        let defaulted_null =
            shape::defaulted(object, "defaultedNull", at, r#"null"#, |value, at| {
                shape::nullable(value, at, shape::text)
            })?;
        let list = shape::defaulted(object, "list", at, r#"["a"]"#, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        let any = shape::optional(object, "any", at, shape::any)?;
        let must = shape::required(object, "must", at, shape::any)?;
        Ok(Self {
            optional,
            nullable,
            both,
            defaulted,
            defaulted_null,
            list,
            any,
            must,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        if let Some(held) = &self.optional {
            object.insert("optional", Value::from(held.as_str()));
        }
        object.insert(
            "nullable",
            self.nullable
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        if let Some(held) = &self.both {
            object.insert(
                "both",
                held.as_ref()
                    .map_or(Value::Null, |held| Value::from(held.as_str())),
            );
        }
        object.insert("defaulted", Value::Number(self.defaulted as f64));
        object.insert(
            "defaultedNull",
            self.defaulted_null
                .as_ref()
                .map_or(Value::Null, |held| Value::from(held.as_str())),
        );
        object.insert(
            "list",
            Value::Array(
                self.list
                    .iter()
                    .map(|held| Value::from(held.as_str()))
                    .collect(),
            ),
        );
        if let Some(held) = &self.any {
            object.insert("any", held.clone());
        }
        object.insert("must", self.must.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Overlap {
    Object1(OverlapObject1),
    Object2(OverlapObject2),
    Leaf(Leaf),
    Auto,
    Text(String),
    Integer(i64),
    Number(f64),
    Null,
    List(Vec<bool>),
}

impl Shape for Overlap {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        if let Ok(held) = OverlapObject1::decode_at(value, at) {
            return Ok(Self::Object1(held));
        }
        if let Ok(held) = OverlapObject2::decode_at(value, at) {
            return Ok(Self::Object2(held));
        }
        if let Ok(held) = Leaf::decode_at(value, at) {
            return Ok(Self::Leaf(held));
        }
        if shape::is_text(value, at, "auto").is_ok() {
            return Ok(Self::Auto);
        }
        if let Ok(held) = shape::text(value, at) {
            return Ok(Self::Text(held));
        }
        if let Ok(held) = shape::integer(value, at) {
            return Ok(Self::Integer(held));
        }
        if let Ok(held) = shape::number(value, at) {
            return Ok(Self::Number(held));
        }
        if shape::null(value, at).is_ok() {
            return Ok(Self::Null);
        }
        if let Ok(held) = shape::list(value, at, shape::flag) {
            return Ok(Self::List(held));
        }
        Err(shape::refused(at, "no branch matches"))
    }

    fn encode(&self) -> Value {
        match self {
            Self::Object1(held) => held.encode(),
            Self::Object2(held) => held.encode(),
            Self::Leaf(held) => held.encode(),
            Self::Auto => Value::from("auto"),
            Self::Text(held) => Value::from(held.as_str()),
            Self::Integer(held) => Value::Number(*held as f64),
            Self::Number(held) => Value::Number(*held),
            Self::Null => Value::Null,
            Self::List(held) => Value::Array(held.iter().map(|held| Value::Bool(*held)).collect()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlapObject1 {
    pub a: String,
}

impl Shape for OverlapObject1 {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let a = shape::required(object, "a", at, shape::text)?;
        Ok(Self { a })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("a", Value::from(self.a.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverlapObject2 {
    pub a: String,
    pub b: f64,
}

impl Shape for OverlapObject2 {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let a = shape::required(object, "a", at, shape::text)?;
        let b = shape::required(object, "b", at, shape::number)?;
        Ok(Self { a, b })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("a", Value::from(self.a.as_str()));
        object.insert("b", Value::Number(self.b));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Open {
    pub counts: Record<u64>,
    pub nested: Record<Record<Leaf>>,
}

impl Shape for Open {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let counts = shape::required(object, "counts", at, |value, at| {
            shape::record(value, at, shape::natural)
        })?;
        let nested = shape::required(object, "nested", at, |value, at| {
            shape::record(value, at, |value, at| {
                shape::record(value, at, Leaf::decode_at)
            })
        })?;
        Ok(Self { counts, nested })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "counts",
            shape::record_value(&self.counts, |held| Value::Number(*held as f64)),
        );
        object.insert(
            "nested",
            shape::record_value(&self.nested, |held| {
                shape::record_value(held, |held| held.encode())
            }),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bounded {
    pub name: Name,
    pub ratio: f64,
    pub few: Vec<bool>,
    pub level: i64,
    pub count: u64,
}

impl Shape for Bounded {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let name = shape::required(object, "name", at, decode_name)?;
        let ratio = shape::required(object, "ratio", at, |value, at| {
            shape::number(value, at)
                .and_then(|held| shape::above(held, 0.0, at))
                .and_then(|held| shape::at_most(held, 1.0, at))
        })?;
        let few = shape::required(object, "few", at, |value, at| {
            shape::list_within(value, at, Some(1), Some(2), shape::flag)
        })?;
        let level = shape::required(object, "level", at, |value, at| {
            shape::integer(value, at)
                .and_then(|held| shape::at_least(held, -2, at))
                .and_then(|held| shape::at_most(held, 2, at))
        })?;
        let count = shape::required(object, "count", at, shape::natural)?;
        Ok(Self {
            name,
            ratio,
            few,
            level,
            count,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("name", encode_name(&self.name));
        object.insert("ratio", Value::Number(self.ratio));
        object.insert(
            "few",
            Value::Array(self.few.iter().map(|held| Value::Bool(*held)).collect()),
        );
        object.insert("level", Value::Number(self.level as f64));
        object.insert("count", Value::Number(self.count as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Literals {
    pub maybe: Option<()>,
}

impl Shape for Literals {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "yes", at, |value, at| {
            shape::is_flag(value, at, true)
        })?;
        shape::required(object, "five", at, |value, at| {
            shape::is_number(value, at, 5.0)
        })?;
        shape::required(object, "nothing", at, shape::null)?;
        shape::required(object, "word", at, |value, at| {
            shape::is_text(value, at, "word")
        })?;
        let maybe = shape::optional(object, "maybe", at, |value, at| {
            shape::is_text(value, at, "x")
        })?;
        Ok(Self { maybe })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("yes", Value::Bool(true));
        object.insert("five", Value::Number(5.0));
        object.insert("nothing", Value::Null);
        object.insert("word", Value::from("word"));
        if self.maybe.is_some() {
            object.insert("maybe", Value::from("x"));
        }
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Outer {
    pub inner: OuterInner,
    pub list: Vec<OuterListItem>,
    pub type_: OuterType,
}

impl Shape for Outer {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let inner = shape::required(object, "inner", at, OuterInner::decode_at)?;
        let list = shape::required(object, "list", at, |value, at| {
            shape::list(value, at, OuterListItem::decode_at)
        })?;
        let type_ = shape::required(object, "type", at, OuterType::decode_at)?;
        Ok(Self { inner, list, type_ })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("inner", self.inner.encode());
        object.insert(
            "list",
            Value::Array(self.list.iter().map(|held| held.encode()).collect()),
        );
        object.insert("type", self.type_.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OuterInner {
    pub deep: Option<OuterInnerDeep>,
}

impl Shape for OuterInner {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let deep = shape::required(object, "deep", at, |value, at| {
            shape::nullable(value, at, OuterInnerDeep::decode_at)
        })?;
        Ok(Self { deep })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert(
            "deep",
            self.deep.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OuterListItem {
    pub color: Color,
    pub names: Names,
}

impl Shape for OuterListItem {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let color = shape::required(object, "color", at, Color::decode_at)?;
        let names = shape::required(object, "names", at, decode_names)?;
        Ok(Self { color, names })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("color", self.color.encode());
        object.insert("names", encode_names(&self.names));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OuterType {
    Struct,
    Enum,
}

impl OuterType {
    pub const ALL: [Self; 2] = [Self::Struct, Self::Enum];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Enum => "enum",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|held| held.as_str() == text)
    }
}

impl Shape for OuterType {
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
pub struct OuterInnerDeep {
    pub x: i64,
}

impl Shape for OuterInnerDeep {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let x = shape::required(object, "x", at, shape::integer)?;
        Ok(Self { x })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("x", Value::Number(self.x as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Text(EventText),
    ToolCall(EventToolCall),
    Tree(EventTree),
}

impl Shape for Event {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        match shape::tag(object, "type", at)? {
            "text" => EventText::decode_at(value, at).map(Self::Text),
            "tool-call" => EventToolCall::decode_at(value, at).map(Self::ToolCall),
            "tree" => EventTree::decode_at(value, at).map(Self::Tree),
            _ => Err(shape::no_branch(at, "type")),
        }
    }

    fn encode(&self) -> Value {
        match self {
            Self::Text(held) => held.encode(),
            Self::ToolCall(held) => held.encode(),
            Self::Tree(held) => held.encode(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventText {
    pub text: String,
}

impl Shape for EventText {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "text")
        })?;
        let text = shape::required(object, "text", at, shape::text)?;
        Ok(Self { text })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("text"));
        object.insert("text", Value::from(self.text.as_str()));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventToolCall {
    pub id: String,
    pub input: Value,
}

impl Shape for EventToolCall {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "tool-call")
        })?;
        let id = shape::required(object, "id", at, shape::text)?;
        let input = shape::required(object, "input", at, shape::any)?;
        Ok(Self { id, input })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("tool-call"));
        object.insert("id", Value::from(self.id.as_str()));
        object.insert("input", self.input.clone());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventTree {
    pub tree: Tree,
}

impl Shape for EventTree {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        shape::required(object, "type", at, |value, at| {
            shape::is_text(value, at, "tree")
        })?;
        let tree = shape::required(object, "tree", at, Tree::decode_at)?;
        Ok(Self { tree })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("type", Value::from("tree"));
        object.insert("tree", self.tree.encode());
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Legacy {
    pub id: String,
    pub tags: Vec<String>,
}

impl Legacy {
    fn decode_plain(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let id = shape::required(object, "id", at, shape::text)?;
        let tags = shape::required(object, "tags", at, |value, at| {
            shape::list(value, at, shape::text)
        })?;
        Ok(Self { id, tags })
    }
}

impl Shape for Legacy {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        {
            let value = &hooks::rename_legacy(value);
            Self::decode_plain(value, at)
        }
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("id", Value::from(self.id.as_str()));
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
pub struct Span {
    pub from: i64,
    pub to: i64,
}

impl Span {
    fn decode_plain(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let from = shape::required(object, "from", at, shape::integer)?;
        let to = shape::required(object, "to", at, shape::integer)?;
        Ok(Self { from, to })
    }
}

impl Shape for Span {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        Self::decode_plain(value, at).and_then(|held| shape::holds(hooks::ordered(&held), held, at))
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("from", Value::Number(self.from as f64));
        object.insert("to", Value::Number(self.to as f64));
        Value::Object(object)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Holder {
    pub legacy: Legacy,
    pub span: Option<Span>,
    pub label: String,
    pub spans: Vec<Span>,
}

impl Shape for Holder {
    fn decode_at(value: &Value, at: &mut Path) -> Result<Self, Refused> {
        let object = shape::object(value, at)?;
        let legacy = shape::required(object, "legacy", at, Legacy::decode_at)?;
        let span = shape::required(object, "span", at, |value, at| {
            shape::nullable(value, at, Span::decode_at)
        })?;
        let label = shape::required(object, "label", at, |value, at| {
            let value = &hooks::trimmed(value);
            shape::text_within(value, at, Some(1), None)
        })?;
        let spans = shape::required(object, "spans", at, |value, at| {
            shape::list(value, at, Span::decode_at)
        })?;
        Ok(Self {
            legacy,
            span,
            label,
            spans,
        })
    }

    fn encode(&self) -> Value {
        let mut object = Object::new();
        object.insert("legacy", self.legacy.encode());
        object.insert(
            "span",
            self.span.as_ref().map_or(Value::Null, |held| held.encode()),
        );
        object.insert("label", Value::from(self.label.as_str()));
        object.insert(
            "spans",
            Value::Array(self.spans.iter().map(|held| held.encode()).collect()),
        );
        Value::Object(object)
    }
}

/// Decodes then encodes a named shape.
pub fn normalize(shape: &str, value: &Value) -> Option<Result<Value, Refused>> {
    let at = &mut Path::new();
    Some(match shape {
        "Color" => Color::decode_at(value, at).map(|held| held.encode()),
        "Name" => decode_name(value, at).map(|held| encode_name(&held)),
        "Names" => decode_names(value, at).map(|held| encode_names(&held)),
        "Leaf" => Leaf::decode_at(value, at).map(|held| held.encode()),
        "Branch" => Branch::decode_at(value, at).map(|held| held.encode()),
        "Tree" => Tree::decode_at(value, at).map(|held| held.encode()),
        "Chain" => Chain::decode_at(value, at).map(|held| held.encode()),
        "Presence" => Presence::decode_at(value, at).map(|held| held.encode()),
        "Overlap" => Overlap::decode_at(value, at).map(|held| held.encode()),
        "Open" => Open::decode_at(value, at).map(|held| held.encode()),
        "Bounded" => Bounded::decode_at(value, at).map(|held| held.encode()),
        "Literals" => Literals::decode_at(value, at).map(|held| held.encode()),
        "Outer" => Outer::decode_at(value, at).map(|held| held.encode()),
        "Event" => Event::decode_at(value, at).map(|held| held.encode()),
        "Legacy" => Legacy::decode_at(value, at).map(|held| held.encode()),
        "Span" => Span::decode_at(value, at).map(|held| held.encode()),
        "Holder" => Holder::decode_at(value, at).map(|held| held.encode()),
        _ => return None,
    })
}
