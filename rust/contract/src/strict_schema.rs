//! JSON Schema subset implementation.

use std::collections::{HashMap, HashSet};

use crate::issues::IssueCode;
use crate::json::{Object, Value};
use crate::number::number_text;
use crate::shape::PathPart;
use crate::strict_policy::{ANNOTATIONS, DROPPED, LISTS, MAPS, SINGLES, TYPES, VALIDATED};
pub use crate::strict_policy::{MAX_ISSUES, MAX_SCHEMA_DEPTH, MAX_STEPS, MAX_VALIDATE_DEPTH};

const ROOT_REF: &str = "#";

type Bound = (&'static str, IssueCode, fn(f64, f64) -> bool);

const BOUNDS: [Bound; 4] = [
    ("minimum", IssueCode::Minimum, |value, bound| value < bound),
    (
        "exclusiveMinimum",
        IssueCode::ExclusiveMinimum,
        |value, bound| value <= bound,
    ),
    ("maximum", IssueCode::Maximum, |value, bound| value > bound),
    (
        "exclusiveMaximum",
        IssueCode::ExclusiveMaximum,
        |value, bound| value >= bound,
    ),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub path: Vec<PathPart>,
    pub code: IssueCode,
    pub message: String,
}

fn issue(path: &[PathPart], code: IssueCode, values: &[(&str, &str)]) -> Issue {
    let mut message = code.template().to_owned();
    for (key, value) in values {
        message = message.replace(&format!("{{{key}}}"), value);
    }
    Issue {
        path: path.to_vec(),
        code,
        message,
    }
}

fn ref_name(value: Option<&Value>) -> Option<&str> {
    let name = value?.as_str()?.strip_prefix("#/$defs/")?;
    (!name.is_empty() && !name.contains('/')).then_some(name)
}

fn root_defs(root: &Value) -> Option<&Object> {
    root.get("$defs")?.as_object()
}

fn annotation(key: &str) -> bool {
    ANNOTATIONS.contains(&key)
}

fn validated(key: &str) -> bool {
    VALIDATED.contains(&key)
}

fn known_type(kind: &str) -> bool {
    TYPES.contains(&kind)
}

fn types_of(schema: &Object) -> Option<Vec<&str>> {
    match schema.get("type")? {
        Value::String(kind) => Some(vec![kind]),
        Value::Array(kinds) => Some(kinds.iter().filter_map(Value::as_str).collect()),
        _ => None,
    }
}

fn distinct(texts: &[&str]) -> bool {
    texts.iter().collect::<HashSet<_>>().len() == texts.len()
}

/// Transforms before subschema traversal.
fn each_schema(value: &Value, depth: usize, change: &mut dyn FnMut(&Object) -> Object) -> Value {
    let Some(node) = value.as_object().filter(|_| depth <= MAX_SCHEMA_DEPTH) else {
        return value.clone();
    };
    let mut held = Object::new();
    for (key, child) in change(node).iter() {
        let rebuilt = match (key, child) {
            (_, Value::Object(named)) if MAPS.contains(&key) => {
                let mut map = Object::new();
                for (name, item) in named.iter() {
                    map.insert(name, each_schema(item, depth + 1, change));
                }
                Value::Object(map)
            }
            _ if SINGLES.contains(&key) => each_schema(child, depth + 1, change),
            (_, Value::Array(branches)) if LISTS.contains(&key) => Value::Array(
                branches
                    .iter()
                    .map(|branch| each_schema(branch, depth + 1, change))
                    .collect(),
            ),
            _ => child.clone(),
        };
        held.insert(key, rebuilt);
    }
    Value::Object(held)
}

struct Checker<'a> {
    defs: Option<&'a Object>,
    path: Vec<PathPart>,
    problems: Vec<Issue>,
}

impl Checker<'_> {
    fn tell(&mut self, code: IssueCode, name: &str) {
        self.problems
            .push(issue(&self.path, code, &[("name", name)]));
    }

    fn named(&mut self, schemas: &Object, depth: usize) {
        for (name, schema) in schemas.iter() {
            self.path.push(PathPart::Key(name.to_owned()));
            self.node(schema, depth + 1);
            self.path.pop();
        }
    }

    fn node(&mut self, value: &Value, depth: usize) {
        let Some(object) = value.as_object() else {
            self.tell(IssueCode::BadKeyword, "schema");
            return;
        };
        if depth > MAX_SCHEMA_DEPTH {
            self.tell(IssueCode::Limit, "");
            return;
        }
        let root = self.path.is_empty();
        let ref_sibling = object.get("$ref").is_some()
            && object
                .keys()
                .any(|key| validated(key) && key != "$ref" && key != "$defs");
        for (key, child) in object.iter() {
            if annotation(key) {
                continue;
            }
            self.path.push(PathPart::Key(key.to_owned()));
            if validated(key) {
                self.keyword(key, child, root, ref_sibling, depth);
            } else {
                self.tell(IssueCode::UnknownKeyword, key);
            }
            self.path.pop();
        }
    }

    fn keyword(&mut self, key: &str, child: &Value, root: bool, ref_sibling: bool, depth: usize) {
        let number = child.as_f64();
        let good = match key {
            "$defs" => match child.as_object().filter(|_| root) {
                Some(definitions) => {
                    self.named(definitions, depth);
                    true
                }
                None => false,
            },
            "$ref" => {
                let known = ref_name(Some(child))
                    .is_some_and(|name| self.defs.is_some_and(|defs| defs.get(name).is_some()));
                if ref_sibling || !known {
                    self.tell(IssueCode::BadReference, "");
                }
                true
            }
            "type" => {
                let kinds = match child {
                    Value::String(kind) => vec![Some(kind.as_str())],
                    Value::Array(kinds) => kinds.iter().map(Value::as_str).collect(),
                    _ => Vec::new(),
                };
                let texts: Vec<&str> = kinds.iter().flatten().copied().collect();
                !kinds.is_empty()
                    && texts.len() == kinds.len()
                    && texts.iter().all(|kind| known_type(kind))
                    && distinct(&texts)
            }
            "enum" => child.as_array().is_some_and(|members| !members.is_empty()),
            "minimum" | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" => {
                number.is_some_and(f64::is_finite)
            }
            "minLength" | "maxLength" | "minItems" | "maxItems" => number
                .is_some_and(|bound| bound.is_finite() && bound >= 0.0 && bound.fract() == 0.0),
            "required" => child.as_array().is_some_and(|names| {
                let texts: Vec<&str> = names.iter().filter_map(Value::as_str).collect();
                texts.len() == names.len() && distinct(&texts)
            }),
            "properties" => match child.as_object() {
                Some(properties) => {
                    self.named(properties, depth);
                    true
                }
                None => false,
            },
            "items" | "propertyNames" => {
                self.node(child, depth + 1);
                true
            }
            "additionalProperties" => {
                if child.as_bool().is_none() {
                    self.node(child, depth + 1);
                }
                true
            }
            "anyOf" | "oneOf" => match child.as_array().filter(|branches| !branches.is_empty()) {
                Some(branches) => {
                    for (index, branch) in branches.iter().enumerate() {
                        self.path.push(PathPart::Index(index));
                        self.node(branch, depth + 1);
                        self.path.pop();
                    }
                    true
                }
                None => false,
            },
            _ => true,
        };
        if !good {
            self.tell(IssueCode::BadKeyword, key);
        }
    }
}

/// Finds inputless refs.
fn inputless_refs<'a>(value: &'a Value, defs: &Object, found: &mut HashSet<&'a str>, depth: usize) {
    let Some(object) = value.as_object().filter(|_| depth <= MAX_SCHEMA_DEPTH) else {
        return;
    };
    if let Some(name) = ref_name(object.get("$ref"))
        && defs.get(name).is_some()
    {
        found.insert(name);
    }
    for keyword in LISTS {
        for branch in object.get(keyword).and_then(Value::as_array).unwrap_or(&[]) {
            inputless_refs(branch, defs, found, depth + 1);
        }
    }
}

fn has_loop(start: &str, graph: &HashMap<&str, HashSet<&str>>) -> bool {
    let mut pending: Vec<&str> = graph.get(start).into_iter().flatten().copied().collect();
    let mut seen = HashSet::new();
    while let Some(next) = pending.pop() {
        if next == start {
            return true;
        }
        if seen.insert(next) {
            pending.extend(graph.get(next).into_iter().flatten().copied());
        }
    }
    false
}

/// Key order aside. An overflowed number equals no null.
fn same_json(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Null, Value::Null) => true,
        (Value::Bool(left), Value::Bool(right)) => left == right,
        (Value::Number(left), Value::Number(right)) => left == right,
        (Value::String(left), Value::String(right)) => left == right,
        (Value::Array(left), Value::Array(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .zip(right)
                    .all(|(left, right)| same_json(left, right))
        }
        (Value::Object(left), Value::Object(right)) => {
            left.len() == right.len()
                && left
                    .iter()
                    .all(|(key, item)| right.get(key).is_some_and(|other| same_json(item, other)))
        }
        _ => false,
    }
}

fn valid_type(kind: &str, value: &Value) -> bool {
    match kind {
        "null" => value.is_null(),
        "boolean" => matches!(value, Value::Bool(_)),
        "number" => value.as_f64().is_some_and(f64::is_finite),
        "integer" => value
            .as_f64()
            .is_some_and(|number| number.is_finite() && number.fract() == 0.0),
        "string" => matches!(value, Value::String(_)),
        "array" => matches!(value, Value::Array(_)),
        "object" => matches!(value, Value::Object(_)),
        _ => false,
    }
}

struct Run<'a> {
    defs: Option<&'a Object>,
    issues: Vec<Issue>,
    path: Vec<PathPart>,
    steps: usize,
    halted: bool,
    /// Limit issue path.
    limit: Option<Vec<PathPart>>,
}

impl<'a> Run<'a> {
    /// Halts checks, including probes.
    fn halt(&mut self, path: Vec<PathPart>) -> bool {
        self.halted = true;
        self.limit.get_or_insert(path);
        false
    }

    fn fails(&mut self, report: bool, code: IssueCode, values: &[(&str, &str)]) -> bool {
        if !report {
            return true;
        }
        self.issues.push(issue(&self.path, code, values));
        if self.issues.len() >= MAX_ISSUES {
            self.halted = true;
        }
        self.halted
    }

    fn bound(&mut self, report: bool, code: IssueCode, bound: f64) -> bool {
        self.fails(report, code, &[("bound", &number_text(bound))])
    }

    /// Probes suppress issue reporting.
    fn check(&mut self, schema: &'a Value, value: &Value, depth: usize, report: bool) -> bool {
        if self.halted {
            return false;
        }
        self.steps += 1;
        if self.steps > MAX_STEPS {
            return self.halt(Vec::new());
        }
        let Some(node) = schema.as_object() else {
            return true;
        };
        if depth > MAX_VALIDATE_DEPTH {
            return self.halt(self.path.clone());
        }
        if node.get("$ref").is_some() {
            let definition = ref_name(node.get("$ref"))
                .and_then(|name| self.defs.and_then(|defs| defs.get(name)));
            return definition.is_none_or(|held| self.check(held, value, depth + 1, report));
        }
        if let Some(types) = types_of(node)
            && !types.iter().any(|kind| valid_type(kind, value))
        {
            self.fails(report, IssueCode::Type, &[("types", &types.join(" or "))]);
            return false;
        }
        let mut clean = true;
        let members = node.get("enum").and_then(Value::as_array);
        let fixed = node.get("const");
        if members.is_some_and(|held| !held.iter().any(|member| same_json(member, value))) {
            clean = false;
            if self.fails(report, IssueCode::Enum, &[]) {
                return false;
            }
        }
        if fixed.is_some_and(|held| !same_json(held, value)) {
            clean = false;
            if self.fails(report, IssueCode::Const, &[]) {
                return false;
            }
        }
        if let Value::Number(number) = value {
            for (keyword, code, fails) in BOUNDS {
                let Some(bound) = node.get(keyword).and_then(Value::as_f64) else {
                    continue;
                };
                if fails(*number, bound) {
                    clean = false;
                    if self.bound(report, code, bound) {
                        return false;
                    }
                }
            }
        }
        if let Value::String(text) = value {
            let length = text.chars().count() as f64;
            let least = node.get("minLength").and_then(Value::as_f64);
            let most = node.get("maxLength").and_then(Value::as_f64);
            if let Some(bound) = least.filter(|bound| length < *bound) {
                clean = false;
                if self.bound(report, IssueCode::MinLength, bound) {
                    return false;
                }
            }
            if let Some(bound) = most.filter(|bound| length > *bound) {
                clean = false;
                if self.bound(report, IssueCode::MaxLength, bound) {
                    return false;
                }
            }
        }
        if let Value::Array(items) = value
            && !self.list(node, items, depth, report)
        {
            clean = false;
            if !report || self.halted {
                return false;
            }
        }
        if let Value::Object(members) = value
            && !self.object(node, members, depth, report)
        {
            clean = false;
            if !report || self.halted {
                return false;
            }
        }
        self.unions(node, value, depth, report) && clean
    }

    fn list(&mut self, schema: &'a Object, items: &[Value], depth: usize, report: bool) -> bool {
        let mut clean = true;
        let count = items.len() as f64;
        let least = schema.get("minItems").and_then(Value::as_f64);
        let most = schema.get("maxItems").and_then(Value::as_f64);
        if let Some(bound) = least.filter(|bound| count < *bound) {
            clean = false;
            if self.bound(report, IssueCode::MinItems, bound) {
                return false;
            }
        }
        if let Some(bound) = most.filter(|bound| count > *bound) {
            clean = false;
            if self.bound(report, IssueCode::MaxItems, bound) {
                return false;
            }
        }
        let Some(each) = schema.get("items") else {
            return clean;
        };
        for (index, item) in items.iter().enumerate() {
            self.path.push(PathPart::Index(index));
            let held = self.check(each, item, depth + 1, report);
            self.path.pop();
            if held {
                continue;
            }
            clean = false;
            if !report || self.halted {
                return false;
            }
        }
        clean
    }

    fn object(&mut self, schema: &'a Object, value: &Object, depth: usize, report: bool) -> bool {
        let mut clean = true;
        let required = schema.get("required").and_then(Value::as_array);
        for name in required.unwrap_or(&[]).iter().filter_map(Value::as_str) {
            if value.get(name).is_some() {
                continue;
            }
            clean = false;
            self.path.push(PathPart::Key(name.to_owned()));
            let stops = self.fails(report, IssueCode::Required, &[("name", name)]);
            self.path.pop();
            if stops {
                return false;
            }
        }
        for (name, item) in value.iter() {
            self.path.push(PathPart::Key(name.to_owned()));
            let held = self.member(schema, name, item, depth, report);
            self.path.pop();
            if held {
                continue;
            }
            clean = false;
            if !report || self.halted {
                return false;
            }
        }
        clean
    }

    fn member(
        &mut self,
        schema: &'a Object,
        name: &str,
        item: &Value,
        depth: usize,
        report: bool,
    ) -> bool {
        let mut clean = true;
        if let Some(names) = schema.get("propertyNames") {
            let named = self.check(names, &Value::from(name), depth + 1, false);
            if self.halted {
                return false;
            }
            if !named {
                clean = false;
                if self.fails(report, IssueCode::PropertyName, &[]) {
                    return false;
                }
            }
        }
        let property = schema
            .get("properties")
            .and_then(Value::as_object)
            .and_then(|properties| properties.get(name));
        if let Some(held) = property {
            return self.check(held, item, depth + 1, report) && clean;
        }
        match schema.get("additionalProperties") {
            Some(Value::Bool(false)) => {
                self.fails(report, IssueCode::Additional, &[("name", name)]);
                false
            }
            Some(additional @ Value::Object(_)) => {
                self.check(additional, item, depth + 1, report) && clean
            }
            _ => clean,
        }
    }

    fn unions(&mut self, schema: &'a Object, value: &Value, depth: usize, report: bool) -> bool {
        let mut clean = true;
        if let Some(branches) = schema.get("anyOf").and_then(Value::as_array) {
            let mut matched = false;
            for branch in branches {
                matched = self.check(branch, value, depth + 1, false);
                if self.halted {
                    return false;
                }
                if matched {
                    break;
                }
            }
            if !matched {
                clean = false;
                if self.fails(report, IssueCode::NoBranch, &[]) {
                    return false;
                }
            }
        }
        if let Some(branches) = schema.get("oneOf").and_then(Value::as_array) {
            let mut matches = 0;
            for branch in branches {
                if self.check(branch, value, depth + 1, false) {
                    matches += 1;
                }
                if self.halted {
                    return false;
                }
                if matches > 1 {
                    break;
                }
            }
            if matches != 1 {
                clean = false;
                let code = if matches == 0 {
                    IssueCode::NoBranch
                } else {
                    IssueCode::ManyBranches
                };
                if self.fails(report, code, &[]) {
                    return false;
                }
            }
        }
        clean
    }
}

/// Drops keywords and rewrites root refs.
pub fn normalize_schema(schema: &Value) -> Value {
    let mut rooted = false;
    let dropped = each_schema(schema, 0, &mut |node| {
        let mut held = Object::new();
        for (key, value) in node.iter() {
            if !DROPPED.contains(&key) {
                held.insert(key, value.clone());
            }
        }
        rooted |= held.get("$ref").and_then(Value::as_str) == Some(ROOT_REF);
        held
    });
    let Some(root) = dropped.as_object().filter(|_| rooted) else {
        return dropped;
    };
    let empty = Object::new();
    let defs = match root.get("$defs") {
        None => &empty,
        Some(Value::Object(defs)) => defs,
        Some(_) => return dropped,
    };
    let mut name = "Root".to_owned();
    while defs.get(&name).is_some() {
        name.push('_');
    }
    let target = format!("#/$defs/{name}");
    let named = each_schema(&dropped, 0, &mut |node| {
        let mut held = Object::new();
        for (key, value) in node.iter() {
            let rooted = key == "$ref" && value.as_str() == Some(ROOT_REF);
            held.insert(
                key,
                if rooted {
                    Value::from(target.as_str())
                } else {
                    value.clone()
                },
            );
        }
        held
    });
    let mut definition = Object::new();
    let mut all_defs = Object::new();
    for (key, value) in named.as_object().into_iter().flat_map(Object::iter) {
        match (key, value) {
            ("$defs", Value::Object(held)) => {
                for (held_name, held_definition) in held.iter() {
                    all_defs.insert(held_name, held_definition.clone());
                }
            }
            ("$defs", _) => {}
            _ => definition.insert(key, value.clone()),
        }
    }
    all_defs.insert(name, Value::Object(definition));
    let mut moved = Object::new();
    moved.insert("$ref", Value::from(target));
    moved.insert("$defs", Value::Object(all_defs));
    Value::Object(moved)
}

/// Reports issues by path.
pub fn schema_problems(schema: &Value) -> Vec<Issue> {
    let defs = root_defs(schema);
    let mut checker = Checker {
        defs,
        path: Vec::new(),
        problems: Vec::new(),
    };
    checker.node(schema, 0);
    let mut problems = checker.problems;
    let Some(defs) = defs else {
        return problems;
    };
    let mut graph = HashMap::new();
    for (name, definition) in defs.iter() {
        let mut refs = HashSet::new();
        inputless_refs(definition, defs, &mut refs, 0);
        graph.insert(name, refs);
    }
    for name in defs.keys() {
        if has_loop(name, &graph) {
            let path = [
                PathPart::Key("$defs".to_owned()),
                PathPart::Key(name.to_owned()),
            ];
            problems.push(issue(&path, IssueCode::ReferenceLoop, &[]));
        }
    }
    problems
}

/// Preserves issue order.
pub fn validate(schema: &Value, value: &Value) -> Vec<Issue> {
    let mut run = Run {
        defs: root_defs(schema),
        issues: Vec::new(),
        path: Vec::new(),
        steps: 0,
        halted: false,
        limit: None,
    };
    run.check(schema, value, 0, true);
    if let Some(path) = run.limit.take().filter(|_| run.issues.len() < MAX_ISSUES) {
        run.issues.push(issue(&path, IssueCode::Limit, &[]));
    }
    run.issues
}
