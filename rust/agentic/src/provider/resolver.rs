//! Routes a model key across the provider list.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use nyaaai_contract::cancel::{CancelToken, Unsubscribe};
use nyaaai_contract::entry_keys::{EntryKeys, KeyAnswer};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::{Record, Value};
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::provider_list::ProviderList;
use nyaaai_contract::shapes::model::{
    Binding, ModelRow, ModelRowCache, ModelRowVerifiedPrice, ModelUse, ModelView, ModelViewStatus,
    Refusal, RowIdentity, RowIdentityHosted, Unresolved, Via,
};
use nyaaai_contract::shapes::provider::{
    EntryFields, KeyReach, Passed, ProviderEntry, ProviderKindName, SkipReason,
};
use nyaaai_contract::shapes::step::{CanonicalRequest, StepResult};
use tokio::sync::OnceCell;

use super::kind::{KindDeps, KindField, ProviderKind, Route};
use crate::codec::Codec;
use crate::models::row_key::{parse_row_key_text, served_id_of};
use crate::models::view::unknown_view;
use crate::transport::{EventStream, KeyAuth, KeySource, Transport, TransportError};

/// One model's view by key.
pub trait Catalog: Send + Sync {
    fn view<'a>(&'a self, key: &'a str, use_: ModelUse) -> BoxFuture<'a, Option<ModelView>>;
}

type Live = Arc<dyn Fn() -> bool + Send + Sync>;

/// Told which entry's catalog refreshed.
pub type Refreshed = Arc<dyn Fn(&ProviderEntry) + Send + Sync>;

/// A route's transport: each request goes with the ZDR switch as it stands then.
#[derive(Clone)]
pub struct RouteTransport {
    transport: Arc<Transport>,
    zdr_only: Live,
}

impl RouteTransport {
    pub fn new(transport: Arc<Transport>, zdr_only: Live) -> Self {
        Self {
            transport,
            zdr_only,
        }
    }

    pub async fn complete(
        &self,
        codec: &dyn Codec,
        request: &CanonicalRequest,
        cancel: &CancelToken,
    ) -> Result<StepResult, TransportError> {
        let mut stamped = request.clone();
        stamped.zdr_only = (self.zdr_only)();
        self.transport.complete(codec, &stamped, cancel).await
    }

    /// The request is stamped with the switch as it stands, in place.
    pub fn stream<'a>(
        &'a self,
        codec: &'a dyn Codec,
        request: &'a mut CanonicalRequest,
        cancel: &CancelToken,
    ) -> EventStream<'a> {
        request.zdr_only = (self.zdr_only)();
        self.transport.stream(codec, request, cancel)
    }

    pub async fn get_json(
        &self,
        path: &str,
        cancel: &CancelToken,
        auth: &KeyAuth,
    ) -> Result<Value, TransportError> {
        self.transport.get_json(path, cancel, auth).await
    }
}

pub struct Resolved {
    pub key: String,
    pub entry: ProviderEntry,
    pub kind: Arc<dyn ProviderKind>,
    pub binding: Binding,
    /// The serving entry's price, context and efforts.
    pub view: ModelView,
    pub transport: RouteTransport,
    pub codec: Box<dyn Codec>,
    zdr_only: Live,
}

impl Resolved {
    /// Zero-retention routes only; read live.
    pub fn zdr_only(&self) -> bool {
        (self.zdr_only)()
    }
}

pub enum Resolution {
    Resolved(Box<Resolved>),
    Unresolved(Unresolved),
}

/// Eligible routes, priority order; at least one.
pub struct Routes {
    pub key: String,
    pub candidates: Vec<Resolved>,
    /// Dropped entries and reasons.
    pub passed: Vec<Passed>,
}

pub enum Planned {
    Routes(Routes),
    Unresolved(Unresolved),
}

/// A row under one use.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedOption {
    pub key: String,
    /// The serving entry's view; unserved, the view behind the refusal, else unknown.
    pub view: ModelView,
    /// Absent when no entry serves it.
    pub via: Option<Via>,
}

/// A refusal to route: the text of why.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolveError(pub String);

impl fmt::Display for ResolveError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(&self.0)
    }
}

impl std::error::Error for ResolveError {}

/// What a resolver stands on.
pub struct ResolverDeps {
    /// List order is priority order.
    pub list: Arc<dyn ProviderList>,
    pub keys: Arc<dyn EntryKeys>,
    pub registry: Arc<dyn ModelRegistry>,
    pub kinds: Vec<Arc<dyn ProviderKind>>,
    /// What a kind's transport stands on.
    pub kind_deps: Arc<dyn Fn(ProviderKindName) -> KindDeps + Send + Sync>,
    pub zdr_only: Live,
    /// Told after a catalog's refresh.
    pub refreshed: Option<Refreshed>,
}

/// Top refusal and view.
struct Unserved {
    refusal: Refusal,
    view: Option<ModelView>,
    entry: Option<String>,
}

enum Routed {
    Ok {
        candidates: Vec<Resolved>,
        passed: Vec<Passed>,
    },
    Refused(Box<Unserved>),
}

fn skip_of(reach: KeyReach) -> SkipReason {
    match reach {
        KeyReach::KeyRejected => SkipReason::KeyRejected,
        KeyReach::Unreachable => SkipReason::Network,
        _ => SkipReason::KeyMissing,
    }
}

fn reach_refusal(reach: KeyReach) -> Refusal {
    match reach {
        KeyReach::KeyRejected => Refusal::KeyRejected,
        KeyReach::Unreachable => Refusal::Unreachable,
        _ => Refusal::KeyMissing,
    }
}

/// Higher rank wins.
fn rank(refusal: Refusal) -> u8 {
    match refusal {
        Refusal::NoModel => 1,
        Refusal::ModelRetired => 2,
        Refusal::NoZdrHost => 3,
        Refusal::Unreachable => 4,
        Refusal::KeyRejected => 5,
        Refusal::KeyMissing => 6,
    }
}

pub fn via_of(entry: &ProviderEntry) -> Via {
    Via {
        entry: entry.id.clone(),
        label: entry.label.clone(),
    }
}

/// What identifies a route: id, plan and URL.
pub fn route_key(entry: &ProviderEntry) -> String {
    [
        entry.id.as_str(),
        entry.plan.map_or("", |plan| plan.as_str()),
        entry.url.as_deref().unwrap_or(""),
    ]
    .join("\n")
}

/// A URL kind needs no key; a key kind needs one.
pub fn reach_of(field: KindField, answer: &KeyAnswer) -> KeyReach {
    match answer {
        KeyAnswer::Key(_) => KeyReach::Ok,
        KeyAnswer::None => {
            if field == KindField::Url {
                KeyReach::Ok
            } else {
                KeyReach::KeyMissing
            }
        }
        KeyAnswer::Refused => KeyReach::KeyRejected,
        KeyAnswer::Unready => KeyReach::Unreachable,
    }
}

/// One route's key: a key stays with its URL.
pub struct EntryKeySource {
    keys: Arc<dyn EntryKeys>,
    list: Arc<dyn ProviderList>,
    entry: ProviderEntry,
}

pub fn key_source_of(
    keys: Arc<dyn EntryKeys>,
    list: Arc<dyn ProviderList>,
    entry: ProviderEntry,
) -> EntryKeySource {
    EntryKeySource { keys, list, entry }
}

impl KeySource for EntryKeySource {
    fn get(&self) -> BoxFuture<'_, Option<String>> {
        Box::pin(async move {
            let listed = self
                .list
                .entries()
                .into_iter()
                .find(|each| each.id == self.entry.id);
            if listed.and_then(|held| held.url) != self.entry.url {
                return None;
            }
            match self.keys.get(&self.entry.id).await {
                KeyAnswer::Key(key) => Some(key),
                _ => None,
            }
        })
    }
}

/// Unknown views remain eligible.
fn usable(view: &ModelView) -> bool {
    matches!(view.status, ModelViewStatus::Ok | ModelViewStatus::Unknown)
}

fn raised(worst: Unserved, next: Refusal, behind: Option<ModelView>, label: &str) -> Unserved {
    let higher = rank(next) > rank(worst.refusal);
    let view = if higher {
        behind.or(worst.view)
    } else {
        worst.view.or(behind)
    };
    Unserved {
        refusal: if higher { next } else { worst.refusal },
        view,
        entry: if higher {
            Some(label.to_owned())
        } else {
            worst.entry.or_else(|| Some(label.to_owned()))
        },
    }
}

fn refusal_of(view: Option<&ModelView>) -> Refusal {
    match view.map(|held| held.status) {
        Some(ModelViewStatus::Retired) => Refusal::ModelRetired,
        Some(ModelViewStatus::NoZdrHost) => Refusal::NoZdrHost,
        _ => Refusal::NoModel,
    }
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

/// Synthesizes an unknown row; a bare key is the kind's own id.
fn adhoc_row(key: &str, kind: ProviderKindName) -> ModelRow {
    let identity = parse_row_key_text(key).unwrap_or_else(|| {
        RowIdentity::Hosted(RowIdentityHosted {
            provider: kind,
            id: key.to_owned(),
        })
    });
    let id = match &identity {
        RowIdentity::Curated(_) => key.to_owned(),
        other => served_id_of(other).unwrap_or(key).to_owned(),
    };
    let mut bindings = Record::new();
    bindings.insert(kind.as_str(), Binding { id });
    ModelRow {
        key: key.to_owned(),
        identity,
        bindings,
        family: key.to_owned(),
        label: key.to_owned(),
        reasoning: false,
        schema: false,
        cache: ModelRowCache::Implicit,
        verified: String::new(),
        verified_price: ModelRowVerifiedPrice {
            prompt: 0.0,
            completion: 0.0,
        },
    }
}

/// What `served_by` stands on.
pub struct ServedByDeps {
    pub entry: ProviderEntry,
    pub kind: Arc<dyn ProviderKind>,
    pub transport: Arc<Transport>,
    pub catalog: Arc<dyn Catalog>,
    pub zdr_only: Live,
}

/// Resolves through one entry.
pub async fn served_by(d: &ServedByDeps, key: &str, use_: ModelUse) -> Routes {
    let view = match d.catalog.view(key, use_).await {
        Some(view) => view,
        None => unknown_view(&adhoc_row(key, d.kind.kind())),
    };
    let codec = d.kind.codec(Some(&view), &fields_of(&d.entry));
    let binding = view
        .row
        .bindings
        .get(d.kind.kind().as_str())
        .cloned()
        .unwrap_or_else(|| Binding { id: key.to_owned() });
    let route = Resolved {
        key: key.to_owned(),
        entry: d.entry.clone(),
        kind: Arc::clone(&d.kind),
        binding,
        view,
        transport: RouteTransport::new(Arc::clone(&d.transport), Arc::clone(&d.zdr_only)),
        codec,
        zdr_only: Arc::clone(&d.zdr_only),
    };
    Routes {
        key: key.to_owned(),
        candidates: vec![route],
        passed: Vec::new(),
    }
}

type Made = Arc<Mutex<Vec<(String, Route)>>>;

fn locked<T>(held: &Mutex<T>) -> MutexGuard<'_, T> {
    held.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// One key check per entry in one read, shared by the rows reading at once.
type Reaches = Arc<Mutex<HashMap<String, Arc<OnceCell<KeyReach>>>>>;

/// Routes keys across entries; holds one route per entry as it stands.
pub struct ModelResolver {
    d: ResolverDeps,
    made: Made,
    unsubscribe: Mutex<Option<Unsubscribe>>,
}

impl ModelResolver {
    pub fn new(d: ResolverDeps) -> Self {
        let made: Made = Arc::new(Mutex::new(Vec::new()));
        let held = Arc::clone(&made);
        let list = Arc::clone(&d.list);
        let unsubscribe = d.list.on_change(Box::new(move || {
            let listed: HashSet<String> = list.entries().iter().map(route_key).collect();
            let gone: Vec<Route> = {
                let mut held = locked(&held);
                let mut gone = Vec::new();
                let mut kept = Vec::new();
                for (at, route) in held.drain(..) {
                    if listed.contains(&at) {
                        kept.push((at, route));
                    } else {
                        gone.push(route);
                    }
                }
                *held = kept;
                gone
            };
            for route in gone {
                route.catalog.dispose();
            }
        }));
        Self {
            d,
            made,
            unsubscribe: Mutex::new(Some(unsubscribe)),
        }
    }

    /// Stops listening; the routes go.
    pub fn dispose(&self) {
        if let Some(unsubscribe) = locked(&self.unsubscribe).take() {
            unsubscribe();
        }
        let routes: Vec<(String, Route)> = std::mem::take(&mut *locked(&self.made));
        for (_, route) in routes {
            route.catalog.dispose();
        }
    }

    fn kind_of(&self, name: ProviderKindName) -> Option<&Arc<dyn ProviderKind>> {
        self.d.kinds.iter().find(|held| held.kind() == name)
    }

    /// The entry's route, made once per id, plan and URL.
    pub fn route(&self, entry: &ProviderEntry) -> Result<Route, ResolveError> {
        let at = route_key(entry);
        if let Some((_, known)) = locked(&self.made).iter().find(|(held, _)| *held == at) {
            return Ok(known.clone());
        }
        let kind = self
            .kind_of(entry.kind)
            .ok_or_else(|| ResolveError(format!("No {} provider kind.", entry.kind.as_str())))?;
        let deps = (self.d.kind_deps)(entry.kind);
        let key = key_source_of(
            Arc::clone(&self.d.keys),
            Arc::clone(&self.d.list),
            entry.clone(),
        );
        let transport = kind
            .transport(&fields_of(entry), Arc::new(key), &deps)
            .map_err(ResolveError)?;
        let told = self.d.refreshed.clone();
        let named = entry.clone();
        let catalog = kind.catalog(
            entry,
            Arc::clone(&transport),
            super::kind::CatalogDeps {
                scheduler: Arc::clone(&deps.scheduler),
                rows: Arc::clone(&self.d.registry),
                refreshed: Arc::new(move || {
                    if let Some(told) = &told {
                        told(&named);
                    }
                }),
            },
        );
        let made = Route { transport, catalog };
        locked(&self.made).push((at, made.clone()));
        Ok(made)
    }

    /// What a key says of one entry.
    pub async fn reach(&self, entry: &ProviderEntry) -> KeyReach {
        let field = self
            .kind_of(entry.kind)
            .map_or(KindField::Key, |held| held.field());
        reach_of(field, &self.d.keys.get(&entry.id).await)
    }

    pub async fn resolve(&self, key: &str, use_: ModelUse) -> Result<Resolution, ResolveError> {
        Ok(match self.routes(key, use_).await? {
            Planned::Routes(mut routes) => {
                Resolution::Resolved(Box::new(routes.candidates.swap_remove(0)))
            }
            Planned::Unresolved(unresolved) => Resolution::Unresolved(unresolved),
        })
    }

    /// Eligible entries, priority order.
    pub async fn routes(&self, key: &str, use_: ModelUse) -> Result<Planned, ResolveError> {
        let rows = self.d.registry.rows();
        let Some(row) = rows.iter().find(|each| each.key == key) else {
            return Ok(Planned::Unresolved(Unresolved {
                key: key.to_owned(),
                refusal: Refusal::NoModel,
                entry: None,
            }));
        };
        let routed = self
            .routed(row, use_, (self.d.zdr_only)(), &self.reaches(), false, true)
            .await?;
        Ok(match routed {
            Routed::Ok { candidates, passed } => Planned::Routes(Routes {
                key: key.to_owned(),
                candidates,
                passed,
            }),
            Routed::Refused(unserved) => Planned::Unresolved(Unresolved {
                key: key.to_owned(),
                refusal: unserved.refusal,
                entry: unserved.entry,
            }),
        })
    }

    /// Settings rows in order.
    pub async fn rows(&self, use_: ModelUse) -> Result<Vec<ResolvedOption>, ResolveError> {
        let zdr_only = (self.d.zdr_only)();
        let reach = self.reaches();
        let rows = self.d.registry.rows();
        let answers = crate::kinds::join::join_all(
            rows.iter()
                .map(|row| {
                    let reach = &reach;
                    Box::pin(async move {
                        let routed = self.routed(row, use_, zdr_only, reach, true, false).await?;
                        Ok::<ResolvedOption, ResolveError>(match routed {
                            Routed::Refused(unserved) => ResolvedOption {
                                key: row.key.clone(),
                                view: unserved.view.unwrap_or_else(|| unknown_view(row)),
                                via: None,
                            },
                            Routed::Ok { candidates, .. } => {
                                let first = &candidates[0];
                                ResolvedOption {
                                    key: row.key.clone(),
                                    view: first.view.clone(),
                                    via: Some(via_of(&first.entry)),
                                }
                            }
                        })
                    })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = _> + Send + '_>>
                })
                .collect(),
        )
        .await;
        answers.into_iter().collect()
    }

    /// Resolvable keys with their serving entries.
    pub async fn candidates(&self, use_: ModelUse) -> Result<Vec<ResolvedOption>, ResolveError> {
        Ok(self
            .rows(use_)
            .await?
            .into_iter()
            .filter(|option| option.via.is_some())
            .collect())
    }

    /// Key check per read.
    fn reaches(&self) -> Reaches {
        Arc::new(Mutex::new(HashMap::new()))
    }

    async fn reach_once(&self, reaches: &Reaches, entry: &ProviderEntry) -> KeyReach {
        let cell = Arc::clone(locked(reaches).entry(entry.id.clone()).or_default());
        *cell.get_or_init(|| self.reach(entry)).await
    }

    async fn routed(
        &self,
        row: &ModelRow,
        use_: ModelUse,
        zdr_only: bool,
        reaches: &Reaches,
        standing: bool,
        all: bool,
    ) -> Result<Routed, ResolveError> {
        let mut worst = Unserved {
            refusal: Refusal::NoModel,
            view: None,
            entry: None,
        };
        let mut matched = false;
        let mut found: Vec<Resolved> = Vec::new();
        let mut passed: Vec<Passed> = Vec::new();
        for entry in self.d.list.entries() {
            if !found.is_empty() && !all {
                break;
            }
            let kind = self.kind_of(entry.kind);
            let binding = row.bindings.get(entry.kind.as_str());
            let elsewhere =
                matches!(&row.identity, RowIdentity::Local(local) if local.entry != entry.id);
            let (Some(kind), Some(binding), false) = (kind, binding, elsewhere) else {
                continue;
            };
            matched = true;
            let reached = self.reach_once(reaches, &entry).await;
            if reached != KeyReach::Ok {
                passed.push(Passed {
                    entry: entry.id.clone(),
                    label: entry.label.clone(),
                    reason: skip_of(reached),
                    before: found.len() as f64,
                });
                let behind = if standing && worst.view.is_none() {
                    self.view_of(&entry, row, use_, zdr_only).await?
                } else {
                    None
                };
                worst = raised(worst, reach_refusal(reached), behind, &entry.label);
                continue;
            }
            let view = self.view_of(&entry, row, use_, zdr_only).await?;
            let view = match view {
                Some(held) if usable(&held) => held,
                other => {
                    worst = raised(worst, refusal_of(other.as_ref()), other, &entry.label);
                    continue;
                }
            };
            let live = Arc::clone(&self.d.zdr_only);
            found.push(Resolved {
                key: row.key.clone(),
                entry: entry.clone(),
                kind: Arc::clone(kind),
                binding: binding.clone(),
                codec: kind.codec(Some(&view), &fields_of(&entry)),
                view,
                transport: RouteTransport::new(self.route(&entry)?.transport, Arc::clone(&live)),
                zdr_only: live,
            });
        }
        if !found.is_empty() {
            return Ok(Routed::Ok {
                candidates: found,
                passed,
            });
        }
        Ok(Routed::Refused(Box::new(if matched {
            worst
        } else {
            Unserved {
                refusal: Refusal::KeyMissing,
                view: None,
                entry: None,
            }
        })))
    }

    async fn view_of(
        &self,
        entry: &ProviderEntry,
        row: &ModelRow,
        use_: ModelUse,
        zdr_only: bool,
    ) -> Result<Option<ModelView>, ResolveError> {
        let route = self.route(entry)?;
        let views = route
            .catalog
            .views(use_, zdr_only, None)
            .await
            .unwrap_or_default();
        Ok(views.into_iter().find(|each| each.row.key == row.key))
    }
}
