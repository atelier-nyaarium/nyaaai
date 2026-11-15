//! Generated from the issue table by scripts/contract.ts. Do not edit.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueCode {
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

impl IssueCode {
    pub fn as_str(&self) -> &'static str {
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

    pub fn template(&self) -> &'static str {
        match self {
            Self::Type => "expected {types}",
            Self::Enum => "not in enum",
            Self::Const => "not equal to const",
            Self::Minimum => "below {bound}",
            Self::ExclusiveMinimum => "not above {bound}",
            Self::Maximum => "above {bound}",
            Self::ExclusiveMaximum => "not below {bound}",
            Self::MinLength => "shorter than {bound}",
            Self::MaxLength => "longer than {bound}",
            Self::MinItems => "fewer than {bound}",
            Self::MaxItems => "more than {bound}",
            Self::Required => "missing {name}",
            Self::Additional => "additional {name}",
            Self::PropertyName => "invalid property name",
            Self::NoBranch => "no branch matches",
            Self::ManyBranches => "multiple branches match",
            Self::UnknownKeyword => "unsupported keyword {name}",
            Self::BadKeyword => "invalid keyword {name}",
            Self::BadReference => "invalid reference",
            Self::ReferenceLoop => "reference loop",
            Self::Limit => "past a limit",
        }
    }
}
