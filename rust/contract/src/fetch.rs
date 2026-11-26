//! The HTTP port, and the budgets every exchange runs on.

use std::fmt;
use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::budgets::{MAX_BYTES, MAX_IDLE_MS, MAX_TOTAL_MS, PRESETS};
use crate::cancel::CancelToken;
use crate::future::BoxFuture;
pub use crate::shapes::step::Budget;
use crate::shapes::step::HeaderList;
use crate::text::ascii_lower;
use crate::time::{Deadline, Ended, Scheduled, Scheduler, deadline};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

/// A `Get` carries no body.
#[derive(Clone, PartialEq)]
pub struct FetchRequest {
    pub method: Method,
    pub url: String,
    pub headers: HeaderList,
    pub body: Option<String>,
    pub budget: Budget,
}

impl fmt::Debug for FetchRequest {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The key never prints.
        let headers: Vec<(&str, &str)> = self
            .headers
            .iter()
            .map(|(name, value)| {
                let secret = ascii_lower(name) == "authorization";
                (name.as_str(), if secret { "[key]" } else { value.as_str() })
            })
            .collect();
        out.debug_struct("FetchRequest")
            .field("method", &self.method)
            .field("url", &self.url)
            .field("headers", &headers)
            .field("body", &self.body)
            .field("budget", &self.budget)
            .finish()
    }
}

/// Which bound of a budget ran out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetBound {
    Total,
    Idle,
    Bytes,
}

impl BudgetBound {
    pub fn text(self) -> &'static str {
        match self {
            Self::Total => "total",
            Self::Idle => "idle",
            Self::Bytes => "bytes",
        }
    }
}

/// A bound of an exchange ran out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetExceeded {
    pub bound: BudgetBound,
    pub limit: u64,
}

impl fmt::Display for BudgetExceeded {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            out,
            "The {} budget of {} ran out.",
            self.bound.text(),
            self.limit
        )
    }
}

impl std::error::Error for BudgetExceeded {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    Cancelled,
    Failed(String),
    Budget(BudgetExceeded),
}

/// A read after the cancel ends `Cancelled`; the connection frees at end, drop or cancel.
pub trait Chunks: Send {
    /// `None` ends the body.
    fn next(&mut self) -> BoxFuture<'_, Result<Option<Vec<u8>>, FetchError>>;
}

pub struct FetchResponse {
    pub status: u16,
    /// In the `response_headers` form.
    pub headers: HeaderList,
    pub chunks: Box<dyn Chunks>,
}

/// Redirects are not followed; the cancel covers body reads.
pub trait Fetch: Send + Sync {
    fn fetch(
        &self,
        request: FetchRequest,
        cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>>;
}

/// Enforces no budget; only `budgeted` takes one.
pub trait RawFetch: Send + Sync {
    fn fetch(
        &self,
        request: FetchRequest,
        cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>>;
}

/// The presets by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetName {
    KeyCheck,
    Limits,
    Stats,
    Catalog,
    Detail,
    Discover,
    Warm,
    Reply,
    Stream,
}

/// Table order.
pub const BUDGET_NAMES: [BudgetName; 9] = [
    BudgetName::KeyCheck,
    BudgetName::Limits,
    BudgetName::Stats,
    BudgetName::Catalog,
    BudgetName::Detail,
    BudgetName::Discover,
    BudgetName::Warm,
    BudgetName::Reply,
    BudgetName::Stream,
];

impl BudgetName {
    pub fn text(self) -> &'static str {
        match self {
            Self::KeyCheck => "keyCheck",
            Self::Limits => "limits",
            Self::Stats => "stats",
            Self::Catalog => "catalog",
            Self::Detail => "detail",
            Self::Discover => "discover",
            Self::Warm => "warm",
            Self::Reply => "reply",
            Self::Stream => "stream",
        }
    }

    pub fn of(text: &str) -> Option<Self> {
        BUDGET_NAMES.into_iter().find(|name| name.text() == text)
    }
}

/// A budget for every preset.
#[derive(Debug, Clone, PartialEq)]
pub struct Budgets {
    held: [Budget; 9],
}

impl Budgets {
    pub fn get(&self, name: BudgetName) -> &Budget {
        &self.held[name as usize]
    }
}

impl Default for Budgets {
    /// The presets.
    fn default() -> Self {
        Self {
            held: BUDGET_NAMES.map(|name| {
                let preset = PRESETS.iter().find(|(text, _)| *text == name.text());
                // The generated table names every preset.
                let [total_ms, idle_ms, max_bytes] = preset
                    .map_or([MAX_TOTAL_MS, MAX_IDLE_MS, MAX_BYTES], |(_, numbers)| {
                        *numbers
                    });
                Budget {
                    total_ms,
                    idle_ms,
                    max_bytes,
                }
            }),
        }
    }
}

/// Every preset the engine uses.
pub fn budgets() -> Budgets {
    Budgets::default()
}

pub const BUDGET_CEILINGS: Budget = Budget {
    total_ms: MAX_TOTAL_MS,
    idle_ms: MAX_IDLE_MS,
    max_bytes: MAX_BYTES,
};

/// One preset's changed fields; any number, so a refusal can be told.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BudgetChange {
    pub total_ms: Option<f64>,
    pub idle_ms: Option<f64>,
    pub max_bytes: Option<f64>,
}

/// The presets with a host's changes, by preset name; a refusal is a fixed text.
pub fn budget_table(changes: &[(&str, BudgetChange)]) -> Result<Budgets, String> {
    if let Some((name, _)) = changes
        .iter()
        .find(|(name, _)| BudgetName::of(name).is_none())
    {
        return Err(format!("There is no {name} budget."));
    }
    let mut table = budgets();
    for name in BUDGET_NAMES {
        let mut change = BudgetChange::default();
        for (_, held) in changes.iter().filter(|(each, _)| *each == name.text()) {
            change.total_ms = held.total_ms.or(change.total_ms);
            change.idle_ms = held.idle_ms.or(change.idle_ms);
            change.max_bytes = held.max_bytes.or(change.max_bytes);
        }
        let preset = table.get(name).clone();
        let value = |field: &str, held: Option<f64>, preset: u64, ceiling: u64| {
            let Some(held) = held else {
                return Ok(preset);
            };
            let whole = held.is_finite() && held.fract() == 0.0;
            if !whole || held < 1.0 || held > ceiling as f64 {
                return Err(format!(
                    "The {} budget's {field} is not a whole number from 1 to {ceiling}.",
                    name.text()
                ));
            }
            Ok(held as u64)
        };
        table.held[name as usize] = Budget {
            total_ms: value("totalMs", change.total_ms, preset.total_ms, MAX_TOTAL_MS)?,
            idle_ms: value("idleMs", change.idle_ms, preset.idle_ms, MAX_IDLE_MS)?,
            max_bytes: value("maxBytes", change.max_bytes, preset.max_bytes, MAX_BYTES)?,
        };
    }
    Ok(table)
}

/// What one exchange's clocks know.
#[derive(Default)]
struct ExchangeState {
    /// The first end; it stands.
    end: Option<FetchError>,
    /// The network wait in progress, by number.
    waiting: Option<u64>,
    waits: u64,
    /// The total passed while no wait ran.
    overdue: bool,
    /// The clocks have stopped.
    over: bool,
    finished: bool,
    bytes: u64,
    total: Option<Scheduled>,
    idle: Option<Scheduled>,
}

/// One exchange's clocks and count; timers hold it weakly.
struct Exchange {
    scheduler: Arc<dyn Scheduler>,
    budget: Budget,
    caller: CancelToken,
    /// What the raw fetch gets: the caller's child, cancelled at an end.
    raw: CancelToken,
    state: Mutex<ExchangeState>,
}

/// How a wait begins.
enum Begin {
    Refused(FetchError),
    /// The clocks have stopped.
    Free,
    Clocked(WaitGuard),
}

/// Ends its wait and idle clock when dropped.
struct WaitGuard {
    exchange: Arc<Exchange>,
    number: u64,
}

impl Drop for WaitGuard {
    fn drop(&mut self) {
        let idle = {
            let mut state = self.exchange.locked();
            if state.waiting == Some(self.number) {
                state.waiting = None;
                state.idle.take()
            } else {
                None
            }
        };
        stop(idle);
    }
}

fn stop(timers: impl IntoIterator<Item = Scheduled>) {
    for timer in timers {
        timer.cancel();
    }
}

impl Exchange {
    fn locked(&self) -> MutexGuard<'_, ExchangeState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Stops the clocks; their timers go out.
    fn settle(state: &mut ExchangeState) -> [Option<Scheduled>; 2] {
        state.over = true;
        [state.total.take(), state.idle.take()]
    }

    fn settled(&self) {
        let timers = Self::settle(&mut self.locked());
        stop(timers.into_iter().flatten());
    }

    /// The first end stands; a caller's cancel already made comes first.
    fn finish(&self, error: FetchError) -> FetchError {
        let (stood, timers) = {
            let mut state = self.locked();
            if let Some(end) = &state.end {
                return end.clone();
            }
            let error = if self.caller.is_cancelled() {
                FetchError::Cancelled
            } else {
                error
            };
            state.end = Some(error.clone());
            (error, Self::settle(&mut state))
        };
        stop(timers.into_iter().flatten());
        self.raw.cancel();
        stood
    }

    fn breach(&self, bound: BudgetBound) -> FetchError {
        let limit = match bound {
            BudgetBound::Total => self.budget.total_ms,
            BudgetBound::Idle => self.budget.idle_ms,
            BudgetBound::Bytes => self.budget.max_bytes,
        };
        self.finish(FetchError::Budget(BudgetExceeded { bound, limit }))
    }

    /// Arms the total clock.
    fn start(self: &Arc<Self>) {
        let weak = Arc::downgrade(self);
        let timer = self.scheduler.schedule(
            delay(self.budget.total_ms),
            Box::new(move || {
                if let Some(exchange) = weak.upgrade() {
                    exchange.on_total();
                }
            }),
        );
        let late = {
            let mut state = self.locked();
            if state.over {
                Some(timer)
            } else {
                state.total = Some(timer);
                None
            }
        };
        stop(late);
    }

    /// Whether a clock due now has nothing left to end.
    fn ended_before(&self, number: Option<u64>) -> bool {
        {
            let state = self.locked();
            let stale = number.is_some_and(|number| state.waiting != Some(number));
            if state.end.is_some() || state.over || stale {
                return true;
            }
        }
        if self.caller.is_cancelled() {
            self.finish(FetchError::Cancelled);
            return true;
        }
        false
    }

    /// A clock yields to all else due at its instant; its timer becomes the clock's.
    fn at_end(self: &Arc<Self>, number: Option<u64>, act: fn(&Self, Option<u64>)) {
        if self.ended_before(number) {
            return;
        }
        let weak = Arc::downgrade(self);
        let timer = self.scheduler.schedule(
            0,
            Box::new(move || {
                if let Some(exchange) = weak.upgrade() {
                    act(&exchange, number);
                }
            }),
        );
        let late = {
            let mut state = self.locked();
            let stale = number.is_some_and(|number| state.waiting != Some(number));
            if state.over || stale {
                Some(timer)
            } else {
                let slot = if number.is_some() {
                    &mut state.idle
                } else {
                    &mut state.total
                };
                *slot = Some(timer);
                None
            }
        };
        stop(late);
    }

    fn on_total(self: &Arc<Self>) {
        self.at_end(None, |exchange, _| exchange.total_ends());
    }

    fn total_ends(&self) {
        {
            let mut state = self.locked();
            if state.end.is_some() || state.over {
                return;
            }
            if state.waiting.is_none() && !self.caller.is_cancelled() {
                state.overdue = true;
                return;
            }
        }
        self.breach(BudgetBound::Total);
    }

    fn on_idle(self: &Arc<Self>, number: u64) {
        self.at_end(Some(number), |exchange, number| {
            if let Some(number) = number {
                exchange.idle_ends(number);
            }
        });
    }

    fn idle_ends(&self, number: u64) {
        {
            let state = self.locked();
            if state.end.is_some() || state.waiting != Some(number) {
                return;
            }
        }
        self.breach(BudgetBound::Idle);
    }

    /// Begins a wait on the network; arms its idle clock.
    fn begin(self: &Arc<Self>) -> Begin {
        let number = {
            let mut state = self.locked();
            if let Some(end) = &state.end {
                return Begin::Refused(end.clone());
            }
            if state.over {
                return Begin::Free;
            }
            if self.caller.is_cancelled() {
                drop(state);
                return Begin::Refused(self.finish(FetchError::Cancelled));
            }
            if state.overdue {
                drop(state);
                return Begin::Refused(self.breach(BudgetBound::Total));
            }
            state.waits += 1;
            state.waiting = Some(state.waits);
            state.waits
        };
        let weak: Weak<Self> = Arc::downgrade(self);
        // A chunk at the bound counts.
        let timer = self.scheduler.schedule(
            delay(self.budget.idle_ms.saturating_add(1)),
            Box::new(move || {
                if let Some(exchange) = weak.upgrade() {
                    exchange.on_idle(number);
                }
            }),
        );
        let late = {
            let mut state = self.locked();
            if state.waiting == Some(number) {
                state.idle = Some(timer);
                None
            } else {
                Some(timer)
            }
        };
        stop(late);
        Begin::Clocked(WaitGuard {
            exchange: Arc::clone(self),
            number,
        })
    }

    /// The read, under the clocks its wait began with.
    async fn waited<T, F>(&self, begun: Begin, read: impl FnOnce() -> F) -> Result<T, FetchError>
    where
        F: Future<Output = Result<T, FetchError>>,
    {
        let guard = match begun {
            Begin::Refused(error) => return Err(error),
            Begin::Free => return read().await,
            Begin::Clocked(guard) => guard,
        };
        let outcome = self.raw.run_until(read()).await;
        drop(guard);
        if let Some(end) = self.locked().end.clone() {
            return Err(end);
        }
        if self.caller.is_cancelled() {
            return Err(self.finish(FetchError::Cancelled));
        }
        match outcome {
            Some(Ok(value)) => Ok(value),
            Some(Err(error)) => {
                self.settled();
                Err(error)
            }
            None => Err(self.finish(FetchError::Cancelled)),
        }
    }
}

/// A delay the scheduler takes.
fn delay(ms: u64) -> i64 {
    i64::try_from(ms).unwrap_or(i64::MAX)
}

/// The exchange's one owner: the request future, then the body.
struct Owner(Arc<Exchange>);

impl Drop for Owner {
    fn drop(&mut self) {
        let (timers, finished) = {
            let mut state = self.0.locked();
            (Exchange::settle(&mut state), state.finished)
        };
        stop(timers.into_iter().flatten());
        if !finished {
            self.0.raw.cancel();
        }
    }
}

/// What an ended exchange keeps of its raw body: nothing.
struct Released;

impl Chunks for Released {
    fn next(&mut self) -> BoxFuture<'_, Result<Option<Vec<u8>>, FetchError>> {
        Box::pin(async { Ok(None) })
    }
}

/// The body, counted and clocked.
struct Body {
    owner: Owner,
    chunks: Box<dyn Chunks>,
}

impl Chunks for Body {
    fn next(&mut self) -> BoxFuture<'_, Result<Option<Vec<u8>>, FetchError>> {
        let Self { owner, chunks } = self;
        let begun = owner.0.begin();
        Box::pin(async move {
            let exchange = &owner.0;
            let step = match exchange.waited(begun, || chunks.next()).await {
                Ok(step) => step,
                Err(error) => {
                    // An ended exchange lets its connection go.
                    if exchange.locked().end.is_some() {
                        *chunks = Box::new(Released);
                    }
                    return Err(error);
                }
            };
            let Some(chunk) = step else {
                let timers = {
                    let mut state = exchange.locked();
                    state.finished = true;
                    Exchange::settle(&mut state)
                };
                stop(timers.into_iter().flatten());
                return Ok(None);
            };
            let past = {
                let mut state = exchange.locked();
                state.bytes = state.bytes.saturating_add(chunk.len() as u64);
                state.bytes > exchange.budget.max_bytes
            };
            if past {
                let error = exchange.breach(BudgetBound::Bytes);
                *chunks = Box::new(Released);
                return Err(error);
            }
            Ok(Some(chunk))
        })
    }
}

/// The one enforcer of a budget.
pub struct Budgeted<R> {
    raw: R,
    scheduler: Arc<dyn Scheduler>,
}

/// A raw fetch under each request's budget, on the scheduler's clocks.
pub fn budgeted<R: RawFetch>(raw: R, scheduler: Arc<dyn Scheduler>) -> Budgeted<R> {
    Budgeted { raw, scheduler }
}

impl<R> Budgeted<R> {
    pub fn raw(&self) -> &R {
        &self.raw
    }
}

impl<R: RawFetch> Fetch for Budgeted<R> {
    fn fetch(
        &self,
        request: FetchRequest,
        cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>> {
        if cancel.is_cancelled() {
            // The raw fetch refuses it.
            let answer = self.raw.fetch(request, cancel);
            return Box::pin(async move {
                drop(answer.await);
                Err(FetchError::Cancelled)
            });
        }
        let exchange = Arc::new(Exchange {
            scheduler: Arc::clone(&self.scheduler),
            budget: request.budget.clone(),
            raw: cancel.child(),
            caller: cancel,
            state: Mutex::new(ExchangeState::default()),
        });
        exchange.start();
        let begun = exchange.begin();
        let head = self.raw.fetch(request, exchange.raw.clone());
        let owner = Owner(exchange);
        Box::pin(async move {
            let response = owner.0.waited(begun, || head).await?;
            Ok(FetchResponse {
                status: response.status,
                headers: response.headers,
                chunks: Box::new(Body {
                    owner,
                    chunks: response.chunks,
                }),
            })
        })
    }
}

/// An operation's clock; its owner clears it.
pub struct Span {
    deadline: Deadline,
    caller: CancelToken,
    budget: Budget,
    start_ms: i64,
    scheduler: Arc<dyn Scheduler>,
}

impl Span {
    /// Cancelled at the caller's cancel or the span's end.
    pub fn token(&self) -> &CancelToken {
        self.deadline.token()
    }

    /// The caller's token alone; what a fetch gets.
    pub fn caller(&self) -> &CancelToken {
        &self.caller
    }

    pub fn ended(&self) -> Option<Ended> {
        self.deadline.ended()
    }

    pub fn clear(&self) {
        self.deadline.clear();
    }

    /// What is left; refuses once ended.
    pub fn budget(&self) -> Result<Budget, BudgetExceeded> {
        let spent = self.scheduler.now_ms().saturating_sub(self.start_ms);
        let preset = i128::from(self.budget.total_ms);
        let left = preset.min(preset - i128::from(spent));
        if self.ended() == Some(Ended::Deadline) || left < 1 {
            return Err(BudgetExceeded {
                bound: BudgetBound::Total,
                limit: self.budget.total_ms,
            });
        }
        let total_ms = left as u64;
        Ok(Budget {
            total_ms,
            idle_ms: self.budget.idle_ms.min(total_ms),
            max_bytes: self.budget.max_bytes,
        })
    }
}

/// An operation's span: a deadline over `total_ms`; no caller never ends.
pub fn span(scheduler: &Arc<dyn Scheduler>, budget: &Budget, caller: Option<&CancelToken>) -> Span {
    let caller = caller.cloned().unwrap_or_default();
    let start_ms = scheduler.now_ms();
    Span {
        deadline: deadline(&**scheduler, delay(budget.total_ms), &caller),
        caller,
        budget: budget.clone(),
        start_ms,
        scheduler: Arc::clone(scheduler),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_debug_print_hides_the_key() {
        let request = FetchRequest {
            method: Method::Post,
            url: "http://host/".to_owned(),
            headers: vec![
                ("Accept".to_owned(), "x".to_owned()),
                (
                    "authorization".to_owned(),
                    "Bearer fake-secret-value".to_owned(),
                ),
            ],
            body: None,
            budget: BUDGET_CEILINGS,
        };
        let shown = format!("{request:?}");
        assert!(!shown.contains("fake-secret-value"));
        assert!(shown.contains("[key]"));
        assert!(shown.contains("Accept"));
    }
}
