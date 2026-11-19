//! Model row identity and its key codec.

use std::sync::LazyLock;

use nyaaai_contract::json::Value;
use nyaaai_contract::pattern::Pattern;
use nyaaai_contract::shapes::model::{
    RowIdentity, RowIdentityCurated, RowIdentityHosted, RowIdentityLocal,
};
use nyaaai_contract::shapes::provider::ProviderKindName;
use nyaaai_contract::text::{has_control, text_length, trim_text};

use super::curated::MODELS;
use crate::provider::entry::is_entry_id;

const LOCAL: &str = "local";

/// A provider's own id or tag; `~` starts an alias.
static SERVED_ID: LazyLock<Pattern> = LazyLock::new(|| {
    Pattern::compile("^~?\\w[\\w.:+-]*(?:\\/\\w[\\w.:+-]*)*$", "").expect("a fixed pattern")
});

const ID_CHARS: usize = 200;

const TAG_CHARS: usize = 1024;

fn served_id(value: &str) -> bool {
    text_length(value) <= ID_CHARS && SERVED_ID.test(value)
}

/// Servers name these: paths, spaces.
fn local_tag(value: &str) -> bool {
    !trim_text(value).is_empty() && text_length(value) <= TAG_CHARS && !has_control(value)
}

fn is_curated(key: &str) -> bool {
    MODELS.iter().any(|row| row.key == key)
}

/// Settings and pick form.
pub fn row_key(identity: &RowIdentity) -> String {
    match identity {
        RowIdentity::Curated(held) => held.key.clone(),
        RowIdentity::Hosted(held) => format!("{}:{}", held.provider.as_str(), held.id),
        RowIdentity::Local(held) => format!("{LOCAL}:{}/{}", held.entry, held.tag),
    }
}

/// Scheme before any slash; none means a curated key.
pub fn parse_row_key(value: &Value) -> Option<RowIdentity> {
    parse_row_key_text(value.as_str()?)
}

pub fn parse_row_key_text(value: &str) -> Option<RowIdentity> {
    let colon = value.find(':');
    let slash = value.find('/');
    let Some(colon) = colon.filter(|colon| slash.is_none_or(|slash| slash > *colon)) else {
        return is_curated(value).then(|| {
            RowIdentity::Curated(RowIdentityCurated {
                key: value.to_owned(),
            })
        });
    };
    let scheme = &value[..colon];
    let rest = &value[colon + 1..];
    if scheme == LOCAL {
        let at = rest.find('/').filter(|at| *at > 0)?;
        let (entry, tag) = (&rest[..at], &rest[at + 1..]);
        return (is_entry_id(entry) && local_tag(tag)).then(|| {
            RowIdentity::Local(RowIdentityLocal {
                entry: entry.to_owned(),
                tag: tag.to_owned(),
            })
        });
    }
    let provider = ProviderKindName::parse(scheme)?;
    served_id(rest).then(|| {
        RowIdentity::Hosted(RowIdentityHosted {
            provider,
            id: rest.to_owned(),
        })
    })
}

/// Browse id; slashless hosted ids show their kind.
pub fn shown_id(identity: &RowIdentity) -> String {
    match identity {
        RowIdentity::Curated(held) => held.key.clone(),
        RowIdentity::Hosted(held) => {
            if held.id.contains('/') {
                held.id.clone()
            } else {
                format!("{}/{}", held.provider.as_str(), held.id)
            }
        }
        RowIdentity::Local(held) => format!("{}/{}", held.entry, held.tag),
    }
}

/// A stored key's Browse id; unreadable keys as stored.
pub fn shown_key(key: &str) -> String {
    parse_row_key_text(key).map_or_else(|| key.to_owned(), |identity| shown_id(&identity))
}

/// The serving kind's model id; none for a curated identity.
pub fn served_id_of(identity: &RowIdentity) -> Option<&str> {
    match identity {
        RowIdentity::Curated(_) => None,
        RowIdentity::Hosted(held) => Some(&held.id),
        RowIdentity::Local(held) => Some(&held.tag),
    }
}
