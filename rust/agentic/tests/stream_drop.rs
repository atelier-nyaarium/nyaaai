//! A dropped `next` never sends a request twice; debug prints hide the key.

use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use nyaaai_agentic::dialects::openrouter::OPENROUTER_CHAT;
use nyaaai_agentic::transport::{Auth, KeyAuth, Transport, TransportDeps, static_key};
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{Fetch, FetchError, FetchRequest, FetchResponse};
use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::shapes::step::{
    CanonicalRequest, CanonicalRequestCache, CanonicalRequestCacheKind,
};
use nyaaai_testkit::scheduler::{FakeScheduler, TimerWheel};

/// Counts requests, then never answers.
struct Silent(Arc<AtomicUsize>);

impl Fetch for Silent {
    fn fetch(
        &self,
        _request: FetchRequest,
        _cancel: CancelToken,
    ) -> BoxFuture<'_, Result<FetchResponse, FetchError>> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(std::future::pending())
    }
}

fn request() -> CanonicalRequest {
    CanonicalRequest {
        model: "vendor/model-1".to_owned(),
        system: Vec::new(),
        summary: None,
        history: Vec::new(),
        state: None,
        tools: None,
        tool_choice: None,
        schema: None,
        effort: None,
        cache: CanonicalRequestCache {
            kind: CanonicalRequestCacheKind::None,
            ttl: None,
        },
        max_tokens: None,
        session: "s".to_owned(),
        zdr_only: false,
    }
}

#[test]
fn a_dropped_next_does_not_send_again() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime.block_on(async {
        let sent = Arc::new(AtomicUsize::new(0));
        let logged = Arc::new(Mutex::new(Vec::<String>::new()));
        let sink = Arc::clone(&logged);
        let told = Arc::new(AtomicUsize::new(0));
        let transport = Transport::new(TransportDeps {
            fetch: Arc::new(Silent(Arc::clone(&sent))),
            scheduler: Arc::new(FakeScheduler::new(Arc::new(TimerWheel::new(0)))),
            key: Arc::new(static_key("fake-1")),
            log: Box::new(move |line| sink.lock().unwrap().push(line.to_owned())),
            base_url: "http://silent.invalid".to_owned(),
            label: "L".to_owned(),
            user_agent: "t/1".to_owned(),
            errors: Arc::new(OPENROUTER_CHAT),
            auth: Auth::Bearer,
            budgets: None,
        });
        let counted = Arc::clone(&told);
        transport.on_settled(move |_| {
            counted.fetch_add(1, Ordering::SeqCst);
        });
        let request = request();
        let cancel = CancelToken::new();
        let mut events = transport.stream(&OPENROUTER_CHAT, &request, &cancel);
        {
            let mut first = std::pin::pin!(events.next());
            let mut context = Context::from_waker(Waker::noop());
            assert!(matches!(first.as_mut().poll(&mut context), Poll::Pending));
        }
        assert_eq!(sent.load(Ordering::SeqCst), 1);
        assert!(matches!(events.next().await, Ok(None)));
        assert!(matches!(events.next().await, Ok(None)));
        drop(events);
        assert_eq!(sent.load(Ordering::SeqCst), 1);
        assert_eq!(logged.lock().unwrap().as_slice(), ["L - - -"]);
        assert_eq!(told.load(Ordering::SeqCst), 0);
    });
}

#[test]
fn a_key_auth_prints_no_key() {
    let shown = format!("{:?}", KeyAuth::Key("fake-secret-1".to_owned()));
    assert!(!shown.contains("fake-secret-1"));
}
