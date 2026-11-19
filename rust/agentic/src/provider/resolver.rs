//! Routes a model key across the provider list.

use std::collections::HashMap;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::task::{Context, Poll, Waker};

use nyaaai_contract::cancel::{CancelToken, Unsubscribe};
use nyaaai_contract::entry_keys::{EntryKeys, KeyAnswer};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::json::{Record, Value};
use nyaaai_contract::model_registry::ModelRegistry;
use nyaaai_contract::shapes::model::{
    Binding, ModelRow, ModelRowCache, ModelRowVerifiedPrice, ModelUse, ModelView, ModelViewStatus,
    Refusal, RowIdentity, RowIdentityHosted, Unresolved, Via,
};
use nyaaai_contract::shapes::provider::{
    EntryFields, KeyReach, Passed, ProviderEntry, ProviderKindName, SkipReason,
};
use nyaaai_contract::shapes::step::{CanonicalRequest, StepResult};
use tokio::sync::OnceCell;

use super::incarnation::{EntryRegistry, Incarnation, RegistryEvent, identity_of};
use super::kind::{EntryCatalog, KindDeps, KindField, ProviderKind, Route};
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

    /// A stream over the request, stamped with the current switch.
    pub fn stream<'a>(
        &'a self,
        codec: &'a dyn Codec,
        mut request: CanonicalRequest,
        cancel: &CancelToken,
    ) -> EventStream<'a> {
        request.zdr_only = (self.zdr_only)();
        self.transport.stream_owned(codec, request, cancel)
    }

    /// The route's own transport, for what a request through the stamping does not cover.
    pub fn transport(&self) -> &Arc<Transport> {
        &self.transport
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
    /// The row when resolved.
    pub entry: ProviderEntry,
    /// Sends nothing once it ended.
    pub incarnation: Arc<Incarnation>,
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
    /// Incarnations in priority order.
    pub entries: Arc<EntryRegistry>,
    /// Key reads go through it.
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

/// One route's key: given only while its incarnation lives.
pub struct EntryKeySource {
    keys: Arc<dyn EntryKeys>,
    /// Weak: the route is held by the incarnation, and a strong one would keep it alive.
    incarnation: Weak<Incarnation>,
    id: String,
}

pub fn key_source_of(keys: Arc<dyn EntryKeys>, incarnation: &Arc<Incarnation>) -> EntryKeySource {
    EntryKeySource {
        keys,
        incarnation: Arc::downgrade(incarnation),
        id: incarnation.id.clone(),
    }
}

impl KeySource for EntryKeySource {
    fn get(&self) -> BoxFuture<'_, Option<String>> {
        Box::pin(async move {
            let answer = self.keys.get(&self.id).await;
            if !self.incarnation.upgrade().is_some_and(|held| held.live()) {
                return None;
            }
            match answer {
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
    pub incarnation: Arc<Incarnation>,
    pub kind: Arc<dyn ProviderKind>,
    pub transport: Arc<Transport>,
    pub catalog: Arc<dyn Catalog>,
    /// The deps the route was made with.
    pub deps: KindDeps,
    pub zdr_only: Live,
}

/// Resolves through one entry.
pub async fn served_by(d: &ServedByDeps, key: &str, use_: &ModelUse) -> Routes {
    let view = match d.catalog.view(key, use_.clone()).await {
        Some(view) => view,
        None => unknown_view(&adhoc_row(key, d.kind.kind())),
    };
    let entry = d.incarnation.entry();
    let codec = d.kind.codec(Some(&view), &fields_of(&entry), &d.deps);
    let binding = view
        .row
        .bindings
        .get(d.kind.kind().as_str())
        .cloned()
        .unwrap_or_else(|| Binding { id: key.to_owned() });
    let route = Resolved {
        key: key.to_owned(),
        entry,
        incarnation: Arc::clone(&d.incarnation),
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

/// A later entry's view is waited for this long.
const LATER_VIEW_MS: i64 = 2000;

/// A later entry's wait: one timer over its key read and its view.
struct Bound {
    runtime: tokio::runtime::Handle,
    expired: Arc<AtomicBool>,
    waker: Arc<Mutex<Option<Waker>>>,
    timer: nyaaai_contract::time::Scheduled,
}

impl Bound {
    fn new(deps: &KindDeps) -> Self {
        let expired = Arc::new(AtomicBool::new(false));
        let waker: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));
        let (flag, wake) = (Arc::clone(&expired), Arc::clone(&waker));
        let timer = deps.scheduler.schedule(
            LATER_VIEW_MS,
            Box::new(move || {
                flag.store(true, Ordering::SeqCst);
                if let Some(waker) = locked(&wake).take() {
                    waker.wake();
                }
            }),
        );
        Self {
            runtime: deps.runtime.clone(),
            expired,
            waker,
            timer,
        }
    }

    /// Starts the read at the call; one that outlives the bound goes on, off the caller.
    async fn go<T: Send + 'static>(&self, mut read: BoxFuture<'static, T>) -> Option<T> {
        match started(read.as_mut()) {
            Poll::Ready(value) => (!self.expired.load(Ordering::SeqCst)).then_some(value),
            Poll::Pending => {
                let running = self.runtime.spawn(read);
                self.race(async move { running.await.ok() }).await.flatten()
            }
        }
    }

    /// The read's answer, or none once the bound ended, even when the read ended with it.
    async fn race<T>(&self, read: impl Future<Output = T>) -> Option<T> {
        let mut read = std::pin::pin!(read);
        std::future::poll_fn(|context| {
            *locked(&self.waker) = Some(context.waker().clone());
            if self.expired.load(Ordering::SeqCst) {
                return Poll::Ready(None);
            }
            match read.as_mut().poll(context) {
                Poll::Ready(value) => {
                    Poll::Ready((!self.expired.load(Ordering::SeqCst)).then_some(value))
                }
                Poll::Pending => Poll::Pending,
            }
        })
        .await
    }
}

/// The first poll, taken where no context lives across an await.
fn started<T>(read: Pin<&mut (dyn Future<Output = T> + Send)>) -> Poll<T> {
    read.poll(&mut Context::from_waker(Waker::noop()))
}

impl Drop for Bound {
    fn drop(&mut self) {
        self.timer.cancel();
    }
}

fn locked<T>(held: &Mutex<T>) -> MutexGuard<'_, T> {
    held.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Stops a catalog that listens; a failure stops nothing else.
fn disposed(catalog: &dyn EntryCatalog) {
    let _ = catch_unwind(AssertUnwindSafe(|| catalog.dispose()));
}

/// A route with the deps it was made with.
#[derive(Clone)]
struct Made {
    route: Route,
    deps: KindDeps,
}

/// The routes a resolver made, by incarnation number, in the order made.
type MadeRoutes = Arc<Mutex<Vec<(u64, Made)>>>;

/// Stops the route made for an incarnation, when there is one.
fn release(made: &Mutex<Vec<(u64, Made)>>, number: u64) {
    let gone = {
        let mut held = locked(made);
        held.iter()
            .position(|(each, _)| *each == number)
            .map(|at| held.remove(at).1)
    };
    if let Some(gone) = gone {
        disposed(&*gone.route.catalog);
    }
}

/// One key check per entry in one read, shared by the rows reading at once.
type Reaches = Arc<Mutex<HashMap<u64, Arc<OnceCell<KeyReach>>>>>;

/// Routes keys across entries; each incarnation holds its route.
pub struct ModelResolver {
    d: ResolverDeps,
    /// Owned here, not on the incarnation: a registry may serve several resolvers.
    made: MadeRoutes,
    unsubscribe: Mutex<Option<Unsubscribe>>,
    closed: AtomicBool,
}

impl ModelResolver {
    pub fn new(d: ResolverDeps) -> Self {
        let made: MadeRoutes = Arc::new(Mutex::new(Vec::new()));
        let unsubscribe = d.entries.watch(Arc::new({
            let made = Arc::clone(&made);
            move |events| {
                for event in events {
                    if let RegistryEvent::Retired(incarnation) = event {
                        release(&made, incarnation.number);
                    }
                }
            }
        }));
        Self {
            d,
            made,
            unsubscribe: Mutex::new(Some(unsubscribe)),
            closed: AtomicBool::new(false),
        }
    }

    fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    /// Stops listening; the routes go; nothing is built after.
    pub fn dispose(&self) {
        if self.closed.swap(true, Ordering::SeqCst) {
            return;
        }
        if let Some(unsubscribe) = locked(&self.unsubscribe).take() {
            unsubscribe();
        }
        let routes = std::mem::take(&mut *locked(&self.made));
        for (_, made) in routes {
            disposed(&*made.route.catalog);
        }
    }

    fn kind_of(&self, name: ProviderKindName) -> Option<&Arc<dyn ProviderKind>> {
        self.d.kinds.iter().find(|held| held.kind() == name)
    }

    /// The incarnation's route, made once.
    pub fn route(&self, entry: &ProviderEntry) -> Result<Route, ResolveError> {
        if self.is_closed() {
            return Err(ResolveError("The resolver is disposed.".to_owned()));
        }
        let incarnation = self
            .d
            .entries
            .incarnation(&entry.id)
            .filter(|held| held.identity == identity_of(entry))
            .ok_or_else(|| ResolveError("The entry is not listed.".to_owned()))?;
        Ok(self.held(&incarnation)?.route)
    }

    fn held(&self, incarnation: &Arc<Incarnation>) -> Result<Made, ResolveError> {
        if self.is_closed() {
            return Err(ResolveError("The resolver is disposed.".to_owned()));
        }
        if let Some((_, known)) = locked(&self.made)
            .iter()
            .find(|(each, _)| *each == incarnation.number)
        {
            return Ok(known.clone());
        }
        let entry = incarnation.entry();
        let kind = self
            .kind_of(entry.kind)
            .ok_or_else(|| ResolveError(format!("No {} provider kind.", entry.kind.as_str())))?;
        let deps = (self.d.kind_deps)(entry.kind);
        let key = key_source_of(Arc::clone(&self.d.keys), incarnation);
        let transport = kind
            .transport(&fields_of(&entry), Arc::new(key), &deps)
            .map_err(ResolveError)?;
        let told = self.d.refreshed.clone();
        // Weak: the route is held by the incarnation.
        let named = Arc::downgrade(incarnation);
        let catalog = kind.catalog(
            &entry,
            Arc::clone(&transport),
            super::kind::CatalogDeps {
                runtime: deps.runtime.clone(),
                scheduler: Arc::clone(&deps.scheduler),
                rows: Arc::clone(&self.d.registry),
                refreshed: Arc::new(move || {
                    if let (Some(told), Some(incarnation)) = (&told, named.upgrade()) {
                        told(&incarnation.entry());
                    }
                }),
            },
        );
        let made = Made {
            route: Route { transport, catalog },
            deps,
        };
        // Another task may have made it first; the first one stays.
        let extra = {
            let mut held = locked(&self.made);
            if self.is_closed() {
                Some(true)
            } else if let Some((_, first)) =
                held.iter().find(|(each, _)| *each == incarnation.number)
            {
                let first = first.clone();
                drop(held);
                disposed(&*made.route.catalog);
                return Ok(first);
            } else {
                held.push((incarnation.number, made.clone()));
                None
            }
        };
        if extra.is_some() {
            disposed(&*made.route.catalog);
            return Err(ResolveError("The resolver is disposed.".to_owned()));
        }
        // Its `retired` event may have passed already; take back what was stored.
        if !incarnation.live() {
            release(&self.made, incarnation.number);
        }
        Ok(made)
    }

    /// The route, or none when it cannot be made.
    fn tried(&self, incarnation: &Arc<Incarnation>) -> Option<Made> {
        self.held(incarnation).ok()
    }

    /// What a key says of one entry.
    pub async fn reach(&self, entry: &ProviderEntry) -> KeyReach {
        self.reach_owned(entry).await
    }

    fn reach_owned(&self, entry: &ProviderEntry) -> BoxFuture<'static, KeyReach> {
        let field = self
            .kind_of(entry.kind)
            .map_or(KindField::Key, |held| held.field());
        let (keys, id) = (Arc::clone(&self.d.keys), entry.id.clone());
        Box::pin(async move { reach_of(field, &keys.get(&id).await) })
    }

    pub async fn resolve(&self, key: &str, use_: &ModelUse) -> Resolution {
        match self.routes(key, use_).await {
            Planned::Routes(mut routes) => {
                Resolution::Resolved(Box::new(routes.candidates.swap_remove(0)))
            }
            Planned::Unresolved(unresolved) => Resolution::Unresolved(unresolved),
        }
    }

    /// Eligible entries, priority order.
    pub async fn routes(&self, key: &str, use_: &ModelUse) -> Planned {
        let rows = self.d.registry.rows();
        let Some(row) = rows.iter().find(|each| each.key == key) else {
            return Planned::Unresolved(Unresolved {
                key: key.to_owned(),
                refusal: Refusal::NoModel,
                entry: None,
            });
        };
        let routed = self
            .routed(row, use_, (self.d.zdr_only)(), &self.reaches(), false, true)
            .await;
        match routed {
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
        }
    }

    /// Settings rows in order.
    pub async fn rows(&self, use_: &ModelUse) -> Vec<ResolvedOption> {
        let zdr_only = (self.d.zdr_only)();
        let reach = self.reaches();
        let rows = self.d.registry.rows();
        crate::kinds::join::join_all(
            rows.iter()
                .map(|row| self.option_of(row, use_, zdr_only, &reach)),
        )
        .await
    }

    /// One row's option; it asks the entries as a Settings row does.
    async fn option_of(
        &self,
        row: &ModelRow,
        use_: &ModelUse,
        zdr_only: bool,
        reach: &Reaches,
    ) -> ResolvedOption {
        match self.routed(row, use_, zdr_only, reach, true, false).await {
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
        }
    }

    /// Resolvable keys with their serving entries.
    pub async fn candidates(&self, use_: &ModelUse) -> Vec<ResolvedOption> {
        self.rows(use_)
            .await
            .into_iter()
            .filter(|option| option.via.is_some())
            .collect()
    }

    /// Key check per read.
    fn reaches(&self) -> Reaches {
        Arc::new(Mutex::new(HashMap::new()))
    }

    async fn reach_once(&self, reaches: &Reaches, incarnation: &Incarnation) -> KeyReach {
        let cell = Arc::clone(locked(reaches).entry(incarnation.number).or_default());
        let entry = incarnation.entry();
        *cell.get_or_init(|| self.reach(&entry)).await
    }

    async fn routed(
        &self,
        row: &ModelRow,
        use_: &ModelUse,
        zdr_only: bool,
        reaches: &Reaches,
        standing: bool,
        all: bool,
    ) -> Routed {
        let mut worst = Unserved {
            refusal: Refusal::NoModel,
            view: None,
            entry: None,
        };
        let mut matched = false;
        let mut found: Vec<Resolved> = Vec::new();
        let mut passed: Vec<Passed> = Vec::new();
        let listed: Vec<Arc<Incarnation>> = if self.is_closed() {
            Vec::new()
        } else {
            self.d.entries.incarnations()
        };
        let owner = match &row.identity {
            RowIdentity::Local(local) => Some(local.entry.clone()),
            _ => None,
        };
        if let Some(owner) = &owner
            && !listed.iter().any(|each| &each.id == owner)
        {
            return Routed::Refused(Box::new(Unserved {
                refusal: Refusal::NoModel,
                view: None,
                entry: None,
            }));
        }
        for incarnation in listed {
            let entry = incarnation.entry();
            if !found.is_empty() && !all {
                break;
            }
            let kind = self.kind_of(entry.kind);
            let binding = row.bindings.get(entry.kind.as_str());
            let elsewhere = owner.as_ref().is_some_and(|owner| *owner != entry.id);
            let (Some(kind), Some(binding), false) = (kind, binding, elsewhere) else {
                continue;
            };
            // A later entry's cold key store or catalog does not hold the answer.
            let bound =
                (!found.is_empty() && all).then(|| Bound::new(&(self.d.kind_deps)(entry.kind)));
            let reached = match &bound {
                Some(bound) => {
                    let cell = Arc::clone(locked(reaches).entry(incarnation.number).or_default());
                    let read = self.reach_owned(&entry);
                    let Some(reached) = bound
                        .go(Box::pin(async move { *cell.get_or_init(|| read).await }))
                        .await
                    else {
                        continue;
                    };
                    reached
                }
                None => self.reach_once(reaches, &incarnation).await,
            };
            if self.is_closed() || !incarnation.live() {
                continue;
            }
            matched = true;
            if reached != KeyReach::Ok {
                passed.push(Passed {
                    entry: entry.id.clone(),
                    label: entry.label.clone(),
                    reason: skip_of(reached),
                    before: found.len() as f64,
                });
                let held = if standing && worst.view.is_none() {
                    self.tried(&incarnation)
                } else {
                    None
                };
                let behind = match held {
                    Some(held) => self.view_of(&held.route, row, use_, zdr_only).await,
                    None => None,
                };
                worst = raised(worst, reach_refusal(reached), behind, &entry.label);
                continue;
            }
            let Some(held) = self.tried(&incarnation) else {
                passed.push(Passed {
                    entry: entry.id.clone(),
                    label: entry.label.clone(),
                    reason: SkipReason::Network,
                    before: found.len() as f64,
                });
                worst = raised(worst, Refusal::Unreachable, None, &entry.label);
                continue;
            };
            let view = match &bound {
                Some(bound) => {
                    let Some(view) = self.within(bound, &held, row, use_, zdr_only).await else {
                        continue;
                    };
                    view
                }
                None => self.view_of(&held.route, row, use_, zdr_only).await,
            };
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
                incarnation: Arc::clone(&incarnation),
                kind: Arc::clone(kind),
                binding: binding.clone(),
                codec: kind.codec(Some(&view), &fields_of(&entry), &held.deps),
                view,
                transport: RouteTransport::new(held.route.transport, Arc::clone(&live)),
                zdr_only: live,
            });
        }
        if !found.is_empty() {
            return Routed::Ok {
                candidates: found,
                passed,
            };
        }
        Routed::Refused(Box::new(if matched {
            worst
        } else {
            Unserved {
                refusal: Refusal::KeyMissing,
                view: None,
                entry: None,
            }
        }))
    }

    /// The view, or none once the bound ends; a read past it goes on and is shared.
    async fn within(
        &self,
        bound: &Bound,
        held: &Made,
        row: &ModelRow,
        use_: &ModelUse,
        zdr_only: bool,
    ) -> Option<Option<ModelView>> {
        let catalog = Arc::clone(&held.route.catalog);
        let (use_, key) = (use_.clone(), row.key.clone());
        bound
            .go(Box::pin(async move {
                let views = catalog
                    .views(use_, zdr_only, None)
                    .await
                    .unwrap_or_default();
                views.into_iter().find(|each| each.row.key == key)
            }))
            .await
    }

    async fn view_of(
        &self,
        route: &Route,
        row: &ModelRow,
        use_: &ModelUse,
        zdr_only: bool,
    ) -> Option<ModelView> {
        let views = route
            .catalog
            .views(use_.clone(), zdr_only, None)
            .await
            .unwrap_or_default();
        views.into_iter().find(|each| each.row.key == row.key)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Barrier;
    use std::sync::atomic::AtomicUsize;

    use nyaaai_contract::shapes::provider::{KeyState, KeyStateNone};
    use nyaaai_contract::shapes::step::Failure;
    use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};
    use nyaaai_testkit::scripted_fetch::ScriptedFetch;
    use nyaaai_testkit::scripted_ports::{
        ScriptedEntryKeys, ScriptedModelRegistry, ScriptedProviderList,
    };

    use super::*;
    use crate::dialects::openrouter::OPENROUTER_CHAT;
    use crate::provider::disposition::disposition_of;
    use crate::provider::kind::{CatalogDeps, KindError};
    use crate::transport::{Auth, TransportDeps};

    struct Stopping(Arc<AtomicUsize>);

    impl EntryCatalog for Stopping {
        fn views(
            &self,
            _use: ModelUse,
            _zdr_only: bool,
            _cancel: Option<&CancelToken>,
        ) -> BoxFuture<'_, Result<Vec<ModelView>, crate::kinds::fetcher::Aborted>> {
            Box::pin(async { Ok(Vec::new()) })
        }

        fn dispose(&self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }

        fn arc_any(self: Arc<Self>) -> Arc<dyn std::any::Any + Send + Sync> {
            self
        }
    }

    /// Catalogs wait for each other, so two routes build before either is kept.
    struct Meeting {
        barrier: Barrier,
        built: AtomicUsize,
        stopped: Arc<AtomicUsize>,
    }

    impl ProviderKind for Meeting {
        fn kind(&self) -> ProviderKindName {
            ProviderKindName::Openrouter
        }

        fn label(&self) -> &str {
            "Stub"
        }

        fn field(&self) -> KindField {
            KindField::Key
        }

        fn check<'a>(
            &'a self,
            _entry: &'a EntryFields,
            _secret: Option<&'a str>,
            _deps: &'a KindDeps,
            _cancel: &'a CancelToken,
        ) -> BoxFuture<'a, Result<KeyState, KindError>> {
            Box::pin(async { Ok(KeyState::None(KeyStateNone {})) })
        }

        fn transport(
            &self,
            _entry: &EntryFields,
            key: Arc<dyn KeySource>,
            deps: &KindDeps,
        ) -> Result<Arc<Transport>, String> {
            Ok(Arc::new(Transport::new(TransportDeps {
                fetch: Arc::clone(&deps.fetch),
                scheduler: Arc::clone(&deps.scheduler),
                key,
                log: Box::new(|_| {}),
                base_url: "https://stub.test".to_owned(),
                label: "Stub".to_owned(),
                user_agent: "tests/1.0".to_owned(),
                errors: Arc::new(OPENROUTER_CHAT),
                auth: Auth::Optional,
                idle_ms: None,
                max_bytes: None,
            })))
        }

        fn catalog(
            &self,
            _entry: &ProviderEntry,
            _transport: Arc<Transport>,
            _deps: CatalogDeps,
        ) -> Arc<dyn EntryCatalog> {
            self.built.fetch_add(1, Ordering::SeqCst);
            self.barrier.wait();
            Arc::new(Stopping(Arc::clone(&self.stopped)))
        }

        fn codec(
            &self,
            _view: Option<&ModelView>,
            _entry: &EntryFields,
            _deps: &KindDeps,
        ) -> Box<dyn Codec> {
            Box::new(OPENROUTER_CHAT)
        }

        fn failure(
            &self,
            failure: &Failure,
            view: Option<&ModelView>,
        ) -> nyaaai_contract::shapes::provider::Disposition {
            disposition_of(failure, view)
        }
    }

    #[test]
    fn two_tasks_making_one_route_keep_the_first_and_stop_the_extra() {
        let wheel = Arc::new(TimerWheel::new(0));
        let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
        let fetch = Arc::new(ScriptedFetch::new(Vec::new(), wheel));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let stopped = Arc::new(AtomicUsize::new(0));
        let kind = Arc::new(Meeting {
            barrier: Barrier::new(2),
            built: AtomicUsize::new(0),
            stopped: Arc::clone(&stopped),
        });
        let entry = ProviderEntry {
            id: "a".to_owned(),
            kind: ProviderKindName::Openrouter,
            label: "First".to_owned(),
            plan: None,
            url: None,
            extra_usage: None,
        };
        let resolver = ModelResolver::new(ResolverDeps {
            entries: Arc::new(EntryRegistry::new(
                Arc::new(ScriptedProviderList::new(vec![entry.clone()])),
                &ScriptedEntryKeys::new(),
            )),
            keys: Arc::new(ScriptedEntryKeys::new()),
            registry: Arc::new(ScriptedModelRegistry::empty()),
            kinds: vec![kind.clone() as Arc<dyn ProviderKind>],
            kind_deps: Arc::new(move |_| KindDeps {
                runtime: runtime.handle().clone(),
                fetch: fetch.clone(),
                scheduler: scheduler.clone(),
                log: Arc::new(|_| {}),
                user_agent: "tests/1.0".to_owned(),
                base_url: None,
            }),
            zdr_only: Arc::new(|| true),
            refreshed: None,
        });
        let routes: Vec<Route> = std::thread::scope(|scope| {
            let first = scope.spawn(|| resolver.route(&entry));
            let second = scope.spawn(|| resolver.route(&entry));
            vec![
                first.join().expect("joined").expect("a route"),
                second.join().expect("joined").expect("a route"),
            ]
        });
        assert_eq!(kind.built.load(Ordering::SeqCst), 2);
        assert!(Arc::ptr_eq(&routes[0].catalog, &routes[1].catalog));
        assert_eq!(stopped.load(Ordering::SeqCst), 1);
        resolver.dispose();
        assert_eq!(stopped.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn an_incarnation_that_left_is_freed_with_its_route() {
        let wheel = Arc::new(TimerWheel::new(0));
        let scheduler = Arc::new(FakeScheduler::new(Arc::clone(&wheel)));
        let fetch = Arc::new(ScriptedFetch::new(Vec::new(), wheel));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let stopped = Arc::new(AtomicUsize::new(0));
        let kind = Arc::new(Meeting {
            barrier: Barrier::new(1),
            built: AtomicUsize::new(0),
            stopped: Arc::clone(&stopped),
        });
        let entry = ProviderEntry {
            id: "a".to_owned(),
            kind: ProviderKindName::Openrouter,
            label: "First".to_owned(),
            plan: None,
            url: None,
            extra_usage: None,
        };
        let list = Arc::new(ScriptedProviderList::new(vec![entry.clone()]));
        let entries = Arc::new(EntryRegistry::new(list.clone(), &ScriptedEntryKeys::new()));
        let resolver = ModelResolver::new(ResolverDeps {
            entries: Arc::clone(&entries),
            keys: Arc::new(ScriptedEntryKeys::new()),
            registry: Arc::new(ScriptedModelRegistry::empty()),
            kinds: vec![kind as Arc<dyn ProviderKind>],
            kind_deps: Arc::new(move |_| KindDeps {
                runtime: runtime.handle().clone(),
                fetch: fetch.clone(),
                scheduler: scheduler.clone(),
                log: Arc::new(|_| {}),
                user_agent: "tests/1.0".to_owned(),
                base_url: None,
            }),
            zdr_only: Arc::new(|| true),
            refreshed: None,
        });
        let route = resolver.route(&entry).expect("a route");
        let held = Arc::downgrade(&entries.incarnation("a").expect("listed"));
        list.set(Vec::new());
        assert_eq!(stopped.load(Ordering::SeqCst), 1);
        drop(route);
        assert!(held.upgrade().is_none());
    }
}
