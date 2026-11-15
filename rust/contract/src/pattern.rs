use std::error::Error;
use std::fmt::{self, Display, Formatter};

use regex::{Captures, Regex, RegexBuilder};

/// Caps compiled regex size.
const SIZE_LIMIT: usize = 32 * 1024 * 1024;

const MAX_DEPTH: usize = 32;

/// Expanded atoms and ranges.
const MAX_COST: usize = 10_000;

type Range = (u32, u32);
type Ranges = Vec<Range>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternMatch {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub captures: Vec<Option<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PatternError {
    message: String,
}

impl PatternError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for PatternError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for PatternError {}

#[derive(Clone, Debug)]
enum Expr {
    Empty,
    Literal(u32),
    Class(Ranges),
    Dot,
    Anchor(bool),
    Boundary(bool),
    Sequence(Vec<Expr>),
    Alternative(Vec<Expr>),
    Group {
        item: Box<Expr>,
        capture: bool,
    },
    Repeat {
        item: Box<Expr>,
        min: usize,
        max: Option<usize>,
        lazy: bool,
    },
}

#[derive(Clone, Debug)]
enum Escaped {
    Literal(u32),
    Ranges(Ranges),
    Boundary(bool),
}

#[derive(Clone, Debug)]
enum ClassAtom {
    Literal(u32),
    Ranges(Ranges),
    Hyphen,
}

#[derive(Clone, Copy)]
struct Traits {
    /// Matches empty text.
    nullable: bool,
    captures: bool,
    /// Captures always participate.
    certain: bool,
    cost: usize,
}

const ATOM: Traits = Traits {
    nullable: false,
    captures: false,
    certain: true,
    cost: 1,
};

const ASSERTION: Traits = Traits {
    nullable: true,
    ..ATOM
};

fn capped(cost: usize) -> usize {
    cost.min(MAX_COST + 1)
}

/// Rejects cross-engine differences.
fn traits_of(expression: &Expr) -> Result<Traits, PatternError> {
    Ok(match expression {
        Expr::Empty | Expr::Anchor(_) | Expr::Boundary(_) => ASSERTION,
        Expr::Literal(_) | Expr::Dot => ATOM,
        Expr::Class(ranges) => Traits {
            cost: ranges.len().max(1),
            ..ATOM
        },
        Expr::Sequence(items) => {
            let mut held = Traits {
                nullable: true,
                cost: 0,
                ..ATOM
            };
            for item in items {
                let item = traits_of(item)?;
                held.nullable &= item.nullable;
                held.captures |= item.captures;
                held.certain &= item.certain;
                held.cost = capped(held.cost + item.cost);
            }
            held
        }
        Expr::Alternative(items) => {
            let mut held = Traits { cost: 0, ..ATOM };
            for item in items {
                let item = traits_of(item)?;
                held.nullable |= item.nullable;
                held.captures |= item.captures;
                held.cost = capped(held.cost + item.cost);
            }
            // Alternatives can skip captures.
            held.certain = !held.captures;
            held
        }
        Expr::Group { item, capture } => {
            let item = traits_of(item)?;
            Traits {
                captures: item.captures || *capture,
                ..item
            }
        }
        Expr::Repeat { item, min, max, .. } => {
            let item = traits_of(item)?;
            if item.nullable {
                return Err(PatternError::new("a repeated item must take a character"));
            }
            if max.is_none_or(|runs| runs > 1) && !item.certain {
                return Err(PatternError::new(
                    "a capture in a repeat must take part in every round",
                ));
            }
            Traits {
                nullable: *min == 0,
                captures: item.captures,
                certain: if *min == 0 {
                    !item.captures
                } else {
                    item.certain
                },
                cost: capped(item.cost.saturating_mul(max.unwrap_or(min + 1).max(1))),
            }
        }
    })
}

struct Parser<'a> {
    source: &'a str,
    at: usize,
    depth: usize,
    insensitive: bool,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, insensitive: bool) -> Self {
        Self {
            source,
            at: 0,
            depth: 0,
            insensitive,
        }
    }

    fn parse(mut self) -> Result<Expr, PatternError> {
        let expression = self.expression()?;
        if self.at != self.source.len() {
            return Err(PatternError::new("unbalanced group"));
        }
        Ok(expression)
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.at..)?.chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let value = self.peek()?;
        self.at += value.len_utf8();
        Some(value)
    }

    fn expression(&mut self) -> Result<Expr, PatternError> {
        let mut branches = vec![self.sequence()?];
        while self.peek() == Some('|') {
            self.bump();
            branches.push(self.sequence()?);
        }
        Ok(if branches.len() == 1 {
            branches.remove(0)
        } else {
            Expr::Alternative(branches)
        })
    }

    fn sequence(&mut self) -> Result<Expr, PatternError> {
        let mut items = Vec::new();
        while self.at < self.source.len() && !matches!(self.peek(), Some(')') | Some('|')) {
            if self.quantifier_start() {
                return Err(PatternError::new("quantifier has nothing to repeat"));
            }
            let mut item = self.atom()?;
            if self.quantifier_start() {
                item = self.repeat(item)?;
            }
            items.push(item);
        }
        Ok(match items.len() {
            0 => Expr::Empty,
            1 => items.remove(0),
            _ => Expr::Sequence(items),
        })
    }

    fn quantifier_start(&self) -> bool {
        matches!(self.peek(), Some('*') | Some('+') | Some('?') | Some('{'))
    }

    fn atom(&mut self) -> Result<Expr, PatternError> {
        match self.peek() {
            Some('(') => self.group(),
            Some('[') => self.character_class(),
            Some('.') => {
                self.bump();
                Ok(Expr::Dot)
            }
            Some('^') => {
                self.bump();
                Ok(Expr::Anchor(true))
            }
            Some('$') => {
                self.bump();
                Ok(Expr::Anchor(false))
            }
            Some('\\') => match self.escape(false)? {
                Escaped::Literal(value) => Ok(Expr::Literal(value)),
                Escaped::Ranges(ranges) => Ok(Expr::Class(ranges)),
                Escaped::Boundary(negated) => Ok(Expr::Boundary(negated)),
            },
            Some(')') => Err(PatternError::new("unbalanced group")),
            Some(']') | Some('{') => Err(PatternError::new("unexpected delimiter")),
            Some(value) => {
                self.bump();
                Ok(Expr::Literal(value as u32))
            }
            None => Err(PatternError::new("expected an expression")),
        }
    }

    fn group(&mut self) -> Result<Expr, PatternError> {
        self.bump();
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(PatternError::new("groups nest too deep"));
        }
        let capture = if self.peek() == Some('?') {
            self.bump();
            match self.peek() {
                Some(':') => {
                    self.bump();
                    false
                }
                Some('=') | Some('!') => {
                    return Err(PatternError::new("lookaround is outside the dialect"));
                }
                Some('<') => {
                    self.bump();
                    if matches!(self.peek(), Some('=') | Some('!')) {
                        return Err(PatternError::new("lookaround is outside the dialect"));
                    }
                    return Err(PatternError::new("named groups are outside the dialect"));
                }
                _ => return Err(PatternError::new("unsupported group syntax")),
            }
        } else {
            true
        };
        let item = self.expression()?;
        if self.peek() != Some(')') {
            return Err(PatternError::new("unbalanced group"));
        }
        self.bump();
        self.depth -= 1;
        Ok(Expr::Group {
            item: Box::new(item),
            capture,
        })
    }

    fn repeat(&mut self, item: Expr) -> Result<Expr, PatternError> {
        let (min, max) = match self.bump() {
            Some('*') => (0, None),
            Some('+') => (1, None),
            Some('?') => (0, Some(1)),
            Some('{') => {
                let min = self.decimal()?;
                let max = if self.peek() == Some(',') {
                    self.bump();
                    if self.peek() == Some('}') {
                        None
                    } else {
                        Some(self.decimal()?)
                    }
                } else {
                    Some(min)
                };
                if self.peek() != Some('}') {
                    return Err(PatternError::new("invalid quantifier"));
                }
                self.bump();
                if min > 1000 || max.is_some_and(|value| value > 1000) {
                    return Err(PatternError::new("quantifier bound exceeds 1000"));
                }
                if max.is_some_and(|value| min > value) {
                    return Err(PatternError::new("inverted quantifier bounds"));
                }
                (min, max)
            }
            _ => return Err(PatternError::new("invalid quantifier")),
        };
        let lazy = self.peek() == Some('?');
        if lazy {
            self.bump();
        }
        Ok(Expr::Repeat {
            item: Box::new(item),
            min,
            max,
            lazy,
        })
    }

    fn decimal(&mut self) -> Result<usize, PatternError> {
        let start = self.at;
        while self.peek().is_some_and(|value| value.is_ascii_digit()) {
            self.bump();
        }
        if start == self.at {
            return Err(PatternError::new("invalid quantifier"));
        }
        self.source[start..self.at]
            .parse()
            .map_err(|_| PatternError::new("invalid quantifier"))
    }

    fn character_class(&mut self) -> Result<Expr, PatternError> {
        self.bump();
        let negated = self.peek() == Some('^');
        if negated {
            self.bump();
        }
        if self.peek() == Some(']') {
            return Err(PatternError::new("empty class"));
        }
        let mut atoms = Vec::new();
        while self.at < self.source.len() && self.peek() != Some(']') {
            match self.peek() {
                Some('\\') => match self.escape(true)? {
                    Escaped::Literal(value) => atoms.push(ClassAtom::Literal(value)),
                    Escaped::Ranges(ranges) => atoms.push(ClassAtom::Ranges(ranges)),
                    Escaped::Boundary(_) => {
                        return Err(PatternError::new("word boundary is outside a class"));
                    }
                },
                Some('-') => {
                    self.bump();
                    atoms.push(ClassAtom::Hyphen);
                }
                Some(value) => {
                    self.bump();
                    atoms.push(ClassAtom::Literal(value as u32));
                }
                None => break,
            }
        }
        if self.peek() != Some(']') {
            return Err(PatternError::new("unbalanced class"));
        }
        self.bump();
        if atoms.is_empty() {
            return Err(PatternError::new("empty class"));
        }
        let mut ranges = Vec::new();
        let mut index = 0;
        while index < atoms.len() {
            let first = &atoms[index];
            let hyphen = atoms.get(index + 1);
            let last = atoms.get(index + 2);
            if let (
                ClassAtom::Literal(start),
                Some(ClassAtom::Hyphen),
                Some(ClassAtom::Literal(end)),
            ) = (first, hyphen, last)
            {
                if start > end {
                    return Err(PatternError::new("inverted range"));
                }
                ranges.push((*start, *end));
                index += 3;
            } else if matches!(hyphen, Some(ClassAtom::Hyphen)) && last.is_some() {
                return Err(PatternError::new("range endpoints must be characters"));
            } else {
                match first {
                    ClassAtom::Literal(value) => ranges.push((*value, *value)),
                    ClassAtom::Ranges(held) => ranges.extend(held.iter().copied()),
                    ClassAtom::Hyphen => ranges.push((0x2d, 0x2d)),
                }
                index += 1;
            }
        }
        let mut ranges = normalize_ranges(ranges);
        if self.insensitive {
            ranges = fold_ascii(&ranges);
        }
        if negated {
            ranges = complement_ranges(&ranges);
        }
        Ok(Expr::Class(ranges))
    }

    fn escape(&mut self, in_class: bool) -> Result<Escaped, PatternError> {
        self.bump();
        let code = self
            .bump()
            .ok_or_else(|| PatternError::new("trailing escape"))?;
        let simple = match code {
            'n' => Some(0x0a),
            'r' => Some(0x0d),
            't' => Some(0x09),
            'f' => Some(0x0c),
            'v' => Some(0x0b),
            '0' => Some(0),
            _ => None,
        };
        if let Some(value) = simple {
            if code == '0' && self.peek().is_some_and(|held| held.is_ascii_digit()) {
                return Err(PatternError::new("decimal escapes are outside the dialect"));
            }
            return Ok(Escaped::Literal(value));
        }
        if matches!(code, 'd' | 'w' | 's' | 'D' | 'W' | 'S') {
            return Ok(Escaped::Ranges(shorthand(code)));
        }
        if matches!(code, 'b' | 'B') {
            if in_class {
                return Err(PatternError::new("unsupported class escape"));
            }
            return Ok(Escaped::Boundary(code == 'B'));
        }
        if code == 'x' {
            return self.hex_escape(2).map(Escaped::Literal);
        }
        if code == 'u' {
            if self.peek() == Some('{') {
                self.bump();
                let start = self.at;
                while self.peek().is_some_and(|value| value.is_ascii_hexdigit()) {
                    self.bump();
                }
                let digits = &self.source[start..self.at];
                if digits.is_empty() || digits.len() > 6 || self.peek() != Some('}') {
                    return Err(PatternError::new("invalid Unicode escape"));
                }
                self.bump();
                let value = u32::from_str_radix(digits, 16)
                    .map_err(|_| PatternError::new("invalid Unicode escape"))?;
                if value > 0x10ffff || (0xd800..=0xdfff).contains(&value) {
                    return Err(PatternError::new("invalid Unicode scalar"));
                }
                return Ok(Escaped::Literal(value));
            }
            return self.hex_escape(4).map(Escaped::Literal);
        }
        if matches!(code, 'p' | 'P') {
            return Err(PatternError::new(
                "Unicode properties are outside the dialect",
            ));
        }
        if code == 'c' {
            return Err(PatternError::new("control escapes are outside the dialect"));
        }
        if code == 'k' || code.is_ascii_digit() && code != '0' {
            return Err(PatternError::new("backreferences are outside the dialect"));
        }
        if "\\/.*+?()[]{}|^$-".contains(code) {
            return Ok(Escaped::Literal(code as u32));
        }
        Err(PatternError::new("unsupported escape"))
    }

    fn hex_escape(&mut self, count: usize) -> Result<u32, PatternError> {
        let end = self.at.saturating_add(count);
        let digits = self
            .source
            .get(self.at..end)
            .ok_or_else(|| PatternError::new("invalid hexadecimal escape"))?;
        if !digits.chars().all(|value| value.is_ascii_hexdigit()) {
            return Err(PatternError::new("invalid hexadecimal escape"));
        }
        self.at = end;
        let value = u32::from_str_radix(digits, 16)
            .map_err(|_| PatternError::new("invalid hexadecimal escape"))?;
        if (0xd800..=0xdfff).contains(&value) {
            return Err(PatternError::new("surrogate code points are refused"));
        }
        Ok(value)
    }
}

fn shorthand(code: char) -> Ranges {
    let base = match code.to_ascii_lowercase() {
        'd' => vec![(0x30, 0x39)],
        'w' => vec![(0x30, 0x39), (0x41, 0x5a), (0x5f, 0x5f), (0x61, 0x7a)],
        _ => vec![
            (0x09, 0x0d),
            (0x20, 0x20),
            (0xa0, 0xa0),
            (0x1680, 0x1680),
            (0x2000, 0x200a),
            (0x2028, 0x2029),
            (0x202f, 0x202f),
            (0x205f, 0x205f),
            (0x3000, 0x3000),
            (0xfeff, 0xfeff),
        ],
    };
    if code.is_ascii_lowercase() {
        base
    } else {
        complement_ranges(&base)
    }
}

fn normalize_ranges(ranges: Ranges) -> Ranges {
    let mut ranges = ranges
        .into_iter()
        .flat_map(|(start, end)| {
            let mut parts = Vec::new();
            if start <= 0xd7ff {
                parts.push((start, end.min(0xd7ff)));
            }
            if end >= 0xe000 {
                parts.push((start.max(0xe000), end));
            }
            parts
        })
        .filter(|(start, end)| start <= end)
        .collect::<Vec<_>>();
    ranges.sort_by_key(|range| range.0);
    let mut result: Ranges = Vec::new();
    for (start, end) in ranges {
        if let Some(last) = result.last_mut()
            && start <= last.1.saturating_add(1)
        {
            last.1 = last.1.max(end);
        } else {
            result.push((start, end));
        }
    }
    result
}

fn complement_ranges(ranges: &Ranges) -> Ranges {
    let present = normalize_ranges(ranges.clone());
    let mut result = Vec::new();
    for (floor, ceiling) in [(0, 0xd7ff), (0xe000, 0x10ffff)] {
        let mut next = floor;
        for (start, end) in &present {
            if *end < floor || *start > ceiling {
                continue;
            }
            if *start > next {
                result.push((next, start.saturating_sub(1).min(ceiling)));
            }
            next = next.max(end.saturating_add(1));
            if next > ceiling {
                break;
            }
        }
        if next <= ceiling {
            result.push((next, ceiling));
        }
    }
    result
}

fn fold_ascii(ranges: &Ranges) -> Ranges {
    let mut result = ranges.clone();
    for (start, end) in ranges {
        let upper_start = (*start).max(0x41);
        let upper_end = (*end).min(0x5a);
        if upper_start <= upper_end {
            result.push((upper_start + 0x20, upper_end + 0x20));
        }
        let lower_start = (*start).max(0x61);
        let lower_end = (*end).min(0x7a);
        if lower_start <= lower_end {
            result.push((lower_start - 0x20, lower_end - 0x20));
        }
    }
    normalize_ranges(result)
}

fn escaped(value: u32) -> String {
    if matches!(value, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a) {
        char::from_u32(value).unwrap().to_string()
    } else {
        format!(r"\u{{{value:x}}}")
    }
}

fn emit_ranges(ranges: &Ranges) -> String {
    let body = ranges
        .iter()
        .map(|(start, end)| {
            if start == end {
                escaped(*start)
            } else {
                format!("{}-{}", escaped(*start), escaped(*end))
            }
        })
        .collect::<String>();
    format!("[{body}]")
}

fn emit(expression: &Expr, insensitive: bool, multiline: bool, dot_all: bool) -> String {
    match expression {
        Expr::Empty => "(?:)".to_owned(),
        Expr::Literal(value) => {
            if insensitive && (0x41..=0x5a).contains(value) {
                emit_ranges(&normalize_ranges(vec![
                    (*value, *value),
                    (*value + 0x20, *value + 0x20),
                ]))
            } else if insensitive && (0x61..=0x7a).contains(value) {
                emit_ranges(&normalize_ranges(vec![
                    (*value, *value),
                    (*value - 0x20, *value - 0x20),
                ]))
            } else {
                escaped(*value)
            }
        }
        Expr::Class(ranges) => emit_ranges(ranges),
        Expr::Dot => {
            if dot_all {
                "(?s:.)".to_owned()
            } else {
                r"[^\n]".to_owned()
            }
        }
        Expr::Anchor(start) => match (*start, multiline) {
            (true, true) => "(?m:^)".to_owned(),
            (false, true) => "(?m:$)".to_owned(),
            (true, false) => r"\A".to_owned(),
            (false, false) => r"\z".to_owned(),
        },
        Expr::Boundary(negated) => {
            if *negated {
                r"(?-u:\B)".to_owned()
            } else {
                r"(?-u:\b)".to_owned()
            }
        }
        Expr::Sequence(items) => items
            .iter()
            .map(|item| emit(item, insensitive, multiline, dot_all))
            .collect(),
        Expr::Alternative(items) => format!(
            "(?:{})",
            items
                .iter()
                .map(|item| emit(item, insensitive, multiline, dot_all))
                .collect::<Vec<_>>()
                .join("|")
        ),
        Expr::Group { item, capture } => format!(
            "{}{})",
            if *capture { "(" } else { "(?:" },
            emit(item, insensitive, multiline, dot_all)
        ),
        Expr::Repeat {
            item,
            min,
            max,
            lazy,
        } => {
            let quantifier = match max {
                None if *min == 0 => "*".to_owned(),
                None if *min == 1 => "+".to_owned(),
                None => format!("{{{min},}}"),
                Some(1) if *min == 0 => "?".to_owned(),
                Some(end) if end == min => format!("{{{min}}}"),
                Some(end) => format!("{{{min},{end}}}"),
            };
            format!(
                "(?:{}){}{}",
                emit(item, insensitive, multiline, dot_all),
                quantifier,
                if *lazy { "?" } else { "" }
            )
        }
    }
}

fn flag_set(flags: &str) -> Result<(bool, bool, bool), PatternError> {
    let mut insensitive = false;
    let mut multiline = false;
    let mut dot_all = false;
    for flag in flags.chars() {
        let held = match flag {
            'i' => &mut insensitive,
            'm' => &mut multiline,
            's' => &mut dot_all,
            _ => return Err(PatternError::new("unsupported flag")),
        };
        if *held {
            return Err(PatternError::new("repeated flag"));
        }
        *held = true;
    }
    Ok((insensitive, multiline, dot_all))
}

#[derive(Debug)]
pub struct Pattern {
    pub source: String,
    pub flags: String,
    regex: Regex,
}

impl Pattern {
    pub fn compile(source: &str, flags: &str) -> Result<Self, PatternError> {
        let (insensitive, multiline, dot_all) = flag_set(flags)?;
        let tree = Parser::new(source, insensitive).parse()?;
        if traits_of(&tree)?.cost > MAX_COST {
            return Err(PatternError::new("the pattern is too large"));
        }
        let emitted = emit(&tree, insensitive, multiline, dot_all);
        let regex = RegexBuilder::new(&emitted)
            .size_limit(SIZE_LIMIT)
            .build()
            .map_err(|error| PatternError::new(error.to_string()))?;
        Ok(Self {
            source: source.to_owned(),
            flags: flags.to_owned(),
            regex,
        })
    }

    pub fn test(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    /// Finds from UTF-16 offsets.
    pub fn find(&self, text: &str, from: usize) -> Option<PatternMatch> {
        let found = self.regex.captures_at(text, byte_at_utf16(text, from))?;
        Some(Units::new(text).matched(&found))
    }

    /// Empty matches advance one scalar.
    fn each<'t>(&self, text: &'t str) -> Vec<Captures<'t>> {
        let mut found = Vec::new();
        let mut from = 0;
        while from <= text.len() {
            let Some(held) = self.regex.captures_at(text, from) else {
                break;
            };
            let (start, end) = span(&held);
            from = if start == end {
                next_boundary(text, end)
            } else {
                end
            };
            found.push(held);
        }
        found
    }

    pub fn find_all(&self, text: &str) -> Vec<PatternMatch> {
        let mut units = Units::new(text);
        self.each(text)
            .iter()
            .map(|held| units.matched(held))
            .collect()
    }

    pub fn replace_all(&self, text: &str, template: &str) -> String {
        self.replace_with(text, |matched| substitute(matched, template))
    }

    pub fn replace_with<F>(&self, text: &str, mut each: F) -> String
    where
        F: FnMut(&PatternMatch) -> String,
    {
        let mut units = Units::new(text);
        let mut result = String::new();
        let mut tail = 0;
        for held in self.each(text) {
            let (start, end) = span(&held);
            result.push_str(&text[tail..start]);
            result.push_str(&each(&units.matched(&held)));
            tail = end;
        }
        result.push_str(&text[tail..]);
        result
    }

    /// Unmatched captures become empty.
    pub fn split(&self, text: &str) -> Vec<String> {
        if text.is_empty() {
            return if self.regex.is_match(text) {
                Vec::new()
            } else {
                vec![String::new()]
            };
        }
        let mut pieces = Vec::new();
        let mut piece = 0;
        let mut from = 0;
        while from < text.len() {
            let Some(held) = self.regex.captures_at(text, from) else {
                break;
            };
            let (start, end) = span(&held);
            if start >= text.len() {
                break;
            }
            // Skip piece-start empty matches.
            if end == piece {
                from = next_boundary(text, from);
                continue;
            }
            pieces.push(text[piece..start].to_owned());
            pieces.extend(
                held.iter().skip(1).map(|capture| {
                    capture.map_or_else(String::new, |part| part.as_str().to_owned())
                }),
            );
            piece = end;
            from = end;
        }
        pieces.push(text[piece..].to_owned());
        pieces
    }
}

pub fn pattern_problem(source: &str, flags: &str) -> Option<String> {
    Pattern::compile(source, flags)
        .err()
        .map(|error| error.to_string())
}

fn byte_at_utf16(text: &str, offset: usize) -> usize {
    let mut units = 0;
    for (byte, value) in text.char_indices() {
        if units >= offset {
            return byte;
        }
        let next = units + value.len_utf16();
        if offset < next {
            return byte + value.len_utf8();
        }
        units = next;
    }
    text.len()
}

fn span(found: &Captures<'_>) -> (usize, usize) {
    found
        .get(0)
        .map_or((0, 0), |whole| (whole.start(), whole.end()))
}

/// Next scalar, else sentinel.
fn next_boundary(text: &str, at: usize) -> usize {
    match text.get(at..).and_then(|tail| tail.chars().next()) {
        Some(next) => at + next.len_utf8(),
        None => text.len() + 1,
    }
}

/// Maps byte offsets to UTF-16.
struct Units<'t> {
    text: &'t str,
    byte: usize,
    units: usize,
}

impl<'t> Units<'t> {
    fn new(text: &'t str) -> Self {
        Self {
            text,
            byte: 0,
            units: 0,
        }
    }

    fn at(&mut self, byte: usize) -> usize {
        if byte < self.byte {
            self.byte = 0;
            self.units = 0;
        }
        let passed = &self.text[self.byte..byte];
        self.units += passed.chars().map(char::len_utf16).sum::<usize>();
        self.byte = byte;
        self.units
    }

    fn matched(&mut self, found: &Captures<'_>) -> PatternMatch {
        let (start, end) = span(found);
        PatternMatch {
            start: self.at(start),
            end: self.at(end),
            text: self.text[start..end].to_owned(),
            captures: found
                .iter()
                .skip(1)
                .map(|capture| capture.map(|part| part.as_str().to_owned()))
                .collect(),
        }
    }
}

fn substitute(matched: &PatternMatch, template: &str) -> String {
    let chars = template.chars().collect::<Vec<_>>();
    let mut result = String::new();
    let mut at = 0;
    while at < chars.len() {
        if chars[at] != '$' || at + 1 >= chars.len() {
            result.push(chars[at]);
            at += 1;
            continue;
        }
        match chars[at + 1] {
            '$' => {
                result.push('$');
                at += 2;
            }
            '&' => {
                result.push_str(&matched.text);
                at += 2;
            }
            value @ '1'..='9' => {
                let capture = value.to_digit(10).unwrap() as usize - 1;
                if let Some(Some(text)) = matched.captures.get(capture) {
                    result.push_str(text);
                }
                at += 2;
            }
            _ => {
                result.push('$');
                at += 1;
            }
        }
    }
    result
}
