use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;

const MAX_DEPTH: usize = 32;
const MAX_CAPTURES: usize = 99;

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
    /// Captures are one-based.
    Group {
        item: Box<Expr>,
        capture: Option<usize>,
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
                captures: item.captures || capture.is_some(),
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
    captures: usize,
    insensitive: bool,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, insensitive: bool) -> Self {
        Self {
            source,
            at: 0,
            depth: 0,
            captures: 0,
            insensitive,
        }
    }

    fn parse(mut self) -> Result<(Expr, usize), PatternError> {
        let expression = self.expression()?;
        if self.at != self.source.len() {
            return Err(PatternError::new("unbalanced group"));
        }
        Ok((expression, self.captures))
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
            Some(']') | Some('{') | Some('}') => Err(PatternError::new("unexpected delimiter")),
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
                    None
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
            self.captures += 1;
            Some(self.captures)
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
        if "\\/.*+?()[]{}|^$".contains(code) || (in_class && code == '-') {
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

#[derive(Clone, Copy, Debug)]
enum AssertKind {
    Start,
    End,
    LineStart,
    LineEnd,
    Boundary,
    NotBoundary,
}

#[derive(Clone, Debug)]
enum Instruction {
    Char(Ranges),
    Split { first: usize, second: usize },
    Jump(usize),
    Save(usize),
    Assert(AssertKind),
    Match,
}

struct Scalar {
    value: u32,
    start: usize,
    byte_start: usize,
    byte_end: usize,
}

struct TextInput<'a> {
    text: &'a str,
    scalars: Vec<Scalar>,
    length: usize,
}

impl<'a> TextInput<'a> {
    fn new(text: &'a str) -> Self {
        let mut units = 0;
        let scalars = text
            .char_indices()
            .map(|(byte_start, value)| {
                let held = Scalar {
                    value: value as u32,
                    start: units,
                    byte_start,
                    byte_end: byte_start + value.len_utf8(),
                };
                units += value.len_utf16();
                held
            })
            .collect();
        Self {
            text,
            scalars,
            length: units,
        }
    }

    fn index_at(&self, offset: usize) -> usize {
        self.scalars.partition_point(|scalar| scalar.start < offset)
    }

    fn offset_at(&self, index: usize) -> usize {
        self.scalars
            .get(index)
            .map_or(self.length, |scalar| scalar.start)
    }

    fn byte_at(&self, offset: usize) -> usize {
        let index = self.index_at(offset);
        match self.scalars.get(index) {
            Some(scalar) if scalar.start == offset => scalar.byte_start,
            Some(scalar) => scalar.byte_end,
            None => self.text.len(),
        }
    }
}

#[derive(Clone)]
struct Thread {
    pc: usize,
    captures: Rc<Vec<Option<usize>>>,
    start_index: usize,
}

struct VmMatch {
    start_index: usize,
    end_index: usize,
    captures: Rc<Vec<Option<usize>>>,
}

fn compile_expression(
    expression: &Expr,
    program: &mut Vec<Instruction>,
    insensitive: bool,
    multiline: bool,
    dot_all: bool,
) {
    match expression {
        Expr::Empty => {}
        Expr::Literal(value) => {
            let ranges = if insensitive {
                fold_ascii(&vec![(*value, *value)])
            } else {
                vec![(*value, *value)]
            };
            program.push(Instruction::Char(ranges));
        }
        Expr::Class(ranges) => program.push(Instruction::Char(ranges.clone())),
        Expr::Dot => program.push(Instruction::Char(if dot_all {
            vec![(0, 0xd7ff), (0xe000, 0x10ffff)]
        } else {
            complement_ranges(&vec![(0x0a, 0x0a)])
        })),
        Expr::Anchor(start) => program.push(Instruction::Assert(match (*start, multiline) {
            (true, true) => AssertKind::LineStart,
            (false, true) => AssertKind::LineEnd,
            (true, false) => AssertKind::Start,
            (false, false) => AssertKind::End,
        })),
        Expr::Boundary(negated) => program.push(Instruction::Assert(if *negated {
            AssertKind::NotBoundary
        } else {
            AssertKind::Boundary
        })),
        Expr::Sequence(items) => {
            for item in items {
                compile_expression(item, program, insensitive, multiline, dot_all);
            }
        }
        Expr::Alternative(items) => {
            let mut jumps = Vec::new();
            let Some((last, earlier)) = items.split_last() else {
                return;
            };
            for item in earlier {
                let split = program.len();
                program.push(Instruction::Split {
                    first: split + 1,
                    second: usize::MAX,
                });
                compile_expression(item, program, insensitive, multiline, dot_all);
                jumps.push(program.len());
                program.push(Instruction::Jump(usize::MAX));
                let next = program.len();
                if let Instruction::Split { second, .. } = &mut program[split] {
                    *second = next;
                }
            }
            compile_expression(last, program, insensitive, multiline, dot_all);
            let end = program.len();
            for jump in jumps {
                if let Instruction::Jump(target) = &mut program[jump] {
                    *target = end;
                }
            }
        }
        Expr::Group { item, capture } => {
            if let Some(capture) = capture {
                program.push(Instruction::Save(capture * 2));
            }
            compile_expression(item, program, insensitive, multiline, dot_all);
            if let Some(capture) = capture {
                program.push(Instruction::Save(capture * 2 + 1));
            }
        }
        Expr::Repeat {
            item,
            min,
            max,
            lazy,
        } => {
            for _ in 0..*min {
                compile_expression(item, program, insensitive, multiline, dot_all);
            }
            if max.is_none() {
                let split = program.len();
                program.push(Instruction::Split {
                    first: usize::MAX,
                    second: usize::MAX,
                });
                let body = program.len();
                compile_expression(item, program, insensitive, multiline, dot_all);
                program.push(Instruction::Jump(split));
                let out = program.len();
                if let Instruction::Split { first, second } = &mut program[split] {
                    *first = if *lazy { out } else { body };
                    *second = if *lazy { body } else { out };
                }
            } else if let Some(max) = max {
                for _ in *min..*max {
                    let split = program.len();
                    program.push(Instruction::Split {
                        first: usize::MAX,
                        second: usize::MAX,
                    });
                    let body = program.len();
                    compile_expression(item, program, insensitive, multiline, dot_all);
                    let out = program.len();
                    if let Instruction::Split { first, second } = &mut program[split] {
                        *first = if *lazy { out } else { body };
                        *second = if *lazy { body } else { out };
                    }
                }
            }
        }
    }
}

fn compile_program(
    tree: &Expr,
    insensitive: bool,
    multiline: bool,
    dot_all: bool,
) -> Vec<Instruction> {
    let mut program = vec![Instruction::Save(0)];
    compile_expression(tree, &mut program, insensitive, multiline, dot_all);
    program.extend([Instruction::Save(1), Instruction::Match]);
    program
}

fn is_word(value: Option<u32>) -> bool {
    value.is_some_and(|value| matches!(value, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a))
}

fn holds_assertion(assertion: AssertKind, input: &TextInput<'_>, index: usize) -> bool {
    let left = index.checked_sub(1).and_then(|at| input.scalars.get(at));
    let right = input.scalars.get(index);
    match assertion {
        AssertKind::Start => index == 0,
        AssertKind::End => index == input.scalars.len(),
        AssertKind::LineStart => index == 0 || left.is_some_and(|scalar| scalar.value == 0x0a),
        AssertKind::LineEnd => {
            index == input.scalars.len() || right.is_some_and(|scalar| scalar.value == 0x0a)
        }
        AssertKind::Boundary => {
            is_word(left.map(|scalar| scalar.value)) != is_word(right.map(|scalar| scalar.value))
        }
        AssertKind::NotBoundary => {
            is_word(left.map(|scalar| scalar.value)) == is_word(right.map(|scalar| scalar.value))
        }
    }
}

fn add_thread(
    program: &[Instruction],
    threads: &mut Vec<Thread>,
    seen: &mut [u64],
    step: u64,
    thread: Thread,
    input: &TextInput<'_>,
    index: usize,
) {
    let mut stack = vec![thread];
    while let Some(held) = stack.pop() {
        if seen[held.pc] == step {
            continue;
        }
        seen[held.pc] = step;
        match &program[held.pc] {
            Instruction::Jump(target) => stack.push(Thread {
                pc: *target,
                captures: held.captures,
                start_index: held.start_index,
            }),
            Instruction::Split { first, second } => {
                stack.push(Thread {
                    pc: *second,
                    captures: held.captures.clone(),
                    start_index: held.start_index,
                });
                stack.push(Thread {
                    pc: *first,
                    captures: held.captures,
                    start_index: held.start_index,
                });
            }
            Instruction::Save(slot) => {
                let mut captures = (*held.captures).clone();
                captures[*slot] = Some(input.offset_at(index));
                stack.push(Thread {
                    pc: held.pc + 1,
                    captures: Rc::new(captures),
                    start_index: held.start_index,
                });
            }
            Instruction::Assert(assertion) => {
                if holds_assertion(*assertion, input, index) {
                    stack.push(Thread {
                        pc: held.pc + 1,
                        captures: held.captures,
                        start_index: held.start_index,
                    });
                }
            }
            Instruction::Char(_) | Instruction::Match => threads.push(held),
        }
    }
}

fn has_scalar(ranges: &[(u32, u32)], value: u32) -> bool {
    ranges
        .iter()
        .any(|(start, end)| value >= *start && value <= *end)
}

fn run_program(
    program: &[Instruction],
    input: &TextInput<'_>,
    start_index: usize,
    capture_count: usize,
) -> Option<VmMatch> {
    let mut current = Vec::new();
    let mut current_seen = vec![0_u64; program.len()];
    let mut next_seen = vec![0_u64; program.len()];
    let empty_captures = Rc::new(vec![None; (capture_count + 1) * 2]);
    let mut candidate = None;
    for (step, index) in (start_index..=input.scalars.len()).enumerate() {
        let current_step = step as u64 + 1;
        let next_step = current_step + 1;
        if candidate.is_none() {
            add_thread(
                program,
                &mut current,
                &mut current_seen,
                current_step,
                Thread {
                    pc: 0,
                    captures: empty_captures.clone(),
                    start_index: index,
                },
                input,
                index,
            );
        }
        let mut next = Vec::new();
        for thread in current {
            match &program[thread.pc] {
                Instruction::Match => {
                    candidate = Some(VmMatch {
                        start_index: thread.start_index,
                        end_index: index,
                        captures: thread.captures,
                    });
                    break;
                }
                Instruction::Char(ranges)
                    if index < input.scalars.len()
                        && has_scalar(ranges, input.scalars[index].value) =>
                {
                    add_thread(
                        program,
                        &mut next,
                        &mut next_seen,
                        next_step,
                        Thread {
                            pc: thread.pc + 1,
                            captures: thread.captures,
                            start_index: thread.start_index,
                        },
                        input,
                        index + 1,
                    );
                }
                // Epsilon instructions are expanded.
                _ => {}
            }
        }
        if candidate.is_some() && next.is_empty() {
            break;
        }
        current = next;
        std::mem::swap(&mut current_seen, &mut next_seen);
    }
    candidate
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
    program: Vec<Instruction>,
    captures: usize,
}

impl Pattern {
    pub fn compile(source: &str, flags: &str) -> Result<Self, PatternError> {
        let (insensitive, multiline, dot_all) = flag_set(flags)?;
        let (tree, captures) = Parser::new(source, insensitive).parse()?;
        if traits_of(&tree)?.cost > MAX_COST {
            return Err(PatternError::new("the pattern is too large"));
        }
        if captures > MAX_CAPTURES {
            return Err(PatternError::new("too many capture groups"));
        }
        let program = compile_program(&tree, insensitive, multiline, dot_all);
        Ok(Self {
            source: source.to_owned(),
            flags: flags.to_owned(),
            program,
            captures,
        })
    }

    pub fn test(&self, text: &str) -> bool {
        self.find(text, 0).is_some()
    }

    /// Find accepts UTF-16 offsets.
    pub fn find(&self, text: &str, from: usize) -> Option<PatternMatch> {
        let input = TextInput::new(text);
        self.find_input(&input, from)
    }

    /// Empty matches advance one scalar.
    fn each(&self, input: &TextInput<'_>) -> Vec<PatternMatch> {
        let mut found = Vec::new();
        let mut from = 0;
        while from <= input.scalars.len() {
            let Some(held) = run_program(&self.program, input, from, self.captures) else {
                break;
            };
            from = if held.start_index == held.end_index {
                held.end_index + 1
            } else {
                held.end_index
            };
            found.push(self.materialize(input, held));
        }
        found
    }

    pub fn find_all(&self, text: &str) -> Vec<PatternMatch> {
        let input = TextInput::new(text);
        self.each(&input)
    }

    pub fn replace_all(&self, text: &str, template: &str) -> String {
        self.replace_with(text, |matched| substitute(matched, template))
    }

    pub fn replace_with<F>(&self, text: &str, mut each: F) -> String
    where
        F: FnMut(&PatternMatch) -> String,
    {
        let input = TextInput::new(text);
        let mut result = String::new();
        let mut tail = 0;
        for held in self.each(&input) {
            let start = input.byte_at(held.start);
            let end = input.byte_at(held.end);
            result.push_str(&text[tail..start]);
            result.push_str(&each(&held));
            tail = end;
        }
        result.push_str(&text[tail..]);
        result
    }

    /// Unmatched captures become empty.
    pub fn split(&self, text: &str) -> Vec<String> {
        let input = TextInput::new(text);
        if text.is_empty() {
            return if self.find_input(&input, 0).is_some() {
                Vec::new()
            } else {
                vec![String::new()]
            };
        }
        let mut pieces = Vec::new();
        let mut piece_units = 0;
        let mut piece_byte = 0;
        for held in self.each(&input) {
            if held.start >= input.length {
                break;
            }
            if held.end == piece_units {
                continue;
            }
            let start = input.byte_at(held.start);
            let end = input.byte_at(held.end);
            pieces.push(text[piece_byte..start].to_owned());
            pieces.extend(
                held.captures
                    .iter()
                    .map(|capture| capture.clone().unwrap_or_default()),
            );
            piece_units = held.end;
            piece_byte = end;
        }
        pieces.push(text[piece_byte..].to_owned());
        pieces
    }

    fn find_input(&self, input: &TextInput<'_>, from: usize) -> Option<PatternMatch> {
        let start_index = input.index_at(from.min(input.length));
        let found = run_program(&self.program, input, start_index, self.captures)?;
        Some(self.materialize(input, found))
    }

    fn materialize(&self, input: &TextInput<'_>, found: VmMatch) -> PatternMatch {
        let slots = &found.captures;
        let start = slots[0].expect("the whole match has a start");
        let end = slots[1].expect("the whole match has an end");
        let start_byte = input.byte_at(start);
        let end_byte = input.byte_at(end);
        let captures = (1..=self.captures)
            .map(|group| match (slots[group * 2], slots[group * 2 + 1]) {
                (Some(start), Some(end)) => {
                    Some(input.text[input.byte_at(start)..input.byte_at(end)].to_owned())
                }
                _ => None,
            })
            .collect();
        PatternMatch {
            start,
            end,
            text: input.text[start_byte..end_byte].to_owned(),
            captures,
        }
    }
}

pub fn pattern_problem(source: &str, flags: &str) -> Option<String> {
    Pattern::compile(source, flags)
        .err()
        .map(|error| error.to_string())
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
