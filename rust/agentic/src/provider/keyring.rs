//! The one reader of a host's keys, and the guard every key a kind would send passes.

use std::sync::Arc;

use nyaaai_contract::cancel::Unsubscribe;
use nyaaai_contract::entry_keys::{EntryKeys, KeyAnswer};
use nyaaai_contract::fetch::Span;
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::provider_list::ProviderList;
use nyaaai_contract::shapes::provider::{
    EntryFields, KeyState, KeyStateInsecure, KeyStateNone, ProviderEntry, ProviderKindName,
};
use nyaaai_contract::url::parse_url;

use crate::provider::entry::{key_binding_of, takes_key};
use crate::provider::host::key_may_travel;
use crate::provider::incarnation::EntryRegistry;
use crate::provider::kind::{KindDeps, KindError, ProviderKind};

/// The port's answer, or a stored key that may not be sent: given for another binding, or over plain http.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyRead {
    Key { key: String, bound_to: String },
    None,
    Refused,
    Unready,
    Foreign { bound_to: String },
    Insecure,
}

impl From<KeyAnswer> for KeyRead {
    fn from(answer: KeyAnswer) -> Self {
        match answer {
            KeyAnswer::Key { key, bound_to } => Self::Key { key, bound_to },
            KeyAnswer::None => Self::None,
            KeyAnswer::Refused => Self::Refused,
            KeyAnswer::Unready => Self::Unready,
        }
    }
}

/// Keys by entry id, as the keyring reads them.
pub trait KeyReads: Send + Sync {
    fn get(&self, id: &str) -> BoxFuture<'_, KeyRead>;

    /// Told the id of an entry whose key changed; the result unsubscribes.
    fn on_change(&self, listener: Box<dyn Fn(&str) + Send + Sync>) -> Unsubscribe;
}

pub struct KeyringDeps {
    /// The host's store; nothing else calls its `get`.
    pub keys: Arc<dyn EntryKeys>,
    pub entries: Arc<EntryRegistry>,
    pub kinds: Vec<Arc<dyn ProviderKind>>,
    /// Where each kind's requests go.
    pub kind_deps: Arc<dyn Fn(ProviderKindName) -> KindDeps + Send + Sync>,
}

/// A host's own reader of configured keys.
pub trait VouchedRead: Send + Sync {
    fn get(&self, id: &str) -> BoxFuture<'_, Option<String>>;

    fn on_change(&self, listener: Box<dyn Fn(&str) + Send + Sync>) -> Unsubscribe;
}

fn fields_of(entry: &ProviderEntry) -> EntryFields {
    EntryFields {
        kind: entry.kind,
        label: entry.label.clone(),
        plan: entry.plan,
        url: entry.url.clone(),
        extra_usage: entry.extra_usage,
    }
}

fn kind_in(
    kinds: &[Arc<dyn ProviderKind>],
    name: ProviderKindName,
) -> Option<&Arc<dyn ProviderKind>> {
    kinds.iter().find(|held| held.kind() == name)
}

/// The guard: a key only for its binding, only where it may travel.
fn guarded(
    kinds: &[Arc<dyn ProviderKind>],
    fields: &EntryFields,
    key: String,
    bound_to: String,
    deps: &KindDeps,
) -> KeyRead {
    let Some(kind) = kind_in(kinds, fields.kind).filter(|held| takes_key(&***held)) else {
        return KeyRead::None;
    };
    if key_binding_of(fields, kinds).as_ref() != Some(&bound_to) {
        return KeyRead::Foreign { bound_to };
    }
    // An unreadable address sends nothing.
    let address = deps.base_url.clone().or_else(|| kind.address(fields));
    if address.is_some_and(|held| parse_url(&held).is_some() && !key_may_travel(&held)) {
        return KeyRead::Insecure;
    }
    KeyRead::Key { key, bound_to }
}

/// A typed key's check: the guard, then the kind's own.
pub fn check_typed<'a>(
    kinds: &'a [Arc<dyn ProviderKind>],
    fields: &'a EntryFields,
    secret: Option<&'a str>,
    deps: &'a KindDeps,
    span: &'a Span,
) -> BoxFuture<'a, Result<KeyState, KindError>> {
    Box::pin(async move {
        let Some(kind) = kind_in(kinds, fields.kind) else {
            return Ok(KeyState::None(KeyStateNone {}));
        };
        let (Some(secret), Some(binding)) = (secret, key_binding_of(fields, kinds)) else {
            let given = secret.filter(|_| takes_key(&**kind));
            return kind.check(fields, given, deps, span).await;
        };
        match guarded(kinds, fields, secret.to_owned(), binding, deps) {
            KeyRead::Insecure => Ok(KeyState::Insecure(KeyStateInsecure {})),
            _ => kind.check(fields, Some(secret), deps, span).await,
        }
    })
}

/// Keys from a host's configuration, each bound to its entry as listed when the read began.
pub fn vouched_keys(
    read: Arc<dyn VouchedRead>,
    list: Arc<dyn ProviderList>,
    kinds: Vec<Arc<dyn ProviderKind>>,
) -> Arc<dyn EntryKeys> {
    Arc::new(Vouched { read, list, kinds })
}

struct Vouched {
    read: Arc<dyn VouchedRead>,
    list: Arc<dyn ProviderList>,
    kinds: Vec<Arc<dyn ProviderKind>>,
}

impl Vouched {
    fn binding_of(&self, id: &str) -> Option<String> {
        let entry = self.list.entries().into_iter().find(|each| each.id == id)?;
        key_binding_of(&fields_of(&entry), &self.kinds)
    }
}

impl EntryKeys for Vouched {
    fn get(&self, id: &str) -> BoxFuture<'_, KeyAnswer> {
        let id = id.to_owned();
        let Some(binding) = self.binding_of(&id) else {
            return Box::pin(async { KeyAnswer::None });
        };
        let read = self.read.get(&id);
        Box::pin(async move {
            let key = read.await;
            // The entry changed while the read ran.
            match key {
                Some(key) if self.binding_of(&id).as_ref() == Some(&binding) => KeyAnswer::Key {
                    key,
                    bound_to: binding,
                },
                _ => KeyAnswer::None,
            }
        })
    }

    fn on_change(&self, listener: Box<dyn Fn(&str) + Send + Sync>) -> Unsubscribe {
        self.read.on_change(listener)
    }
}

/// The only caller of the store's `get`.
pub struct Keyring {
    d: KeyringDeps,
}

impl Keyring {
    pub fn new(deps: KeyringDeps) -> Self {
        Self { d: deps }
    }

    /// The entry's key as its binding and address allow; the store is asked at the call.
    pub fn read(&self, entry: &ProviderEntry) -> BoxFuture<'_, KeyRead> {
        let answer = self.d.keys.get(&entry.id);
        let fields = fields_of(entry);
        Box::pin(async move {
            match answer.await {
                KeyAnswer::Key { key, bound_to } => guarded(
                    &self.d.kinds,
                    &fields,
                    key,
                    bound_to,
                    &(self.d.kind_deps)(fields.kind),
                ),
                other => other.into(),
            }
        })
    }
}

impl KeyReads for Keyring {
    /// The listed entry's; none when the registry holds no such id.
    fn get(&self, id: &str) -> BoxFuture<'_, KeyRead> {
        match self.d.entries.incarnation(id) {
            None => Box::pin(async { KeyRead::None }),
            Some(incarnation) => self.read(&incarnation.entry()),
        }
    }

    fn on_change(&self, listener: Box<dyn Fn(&str) + Send + Sync>) -> Unsubscribe {
        self.d.entries.on_key_change(Arc::from(listener))
    }
}
