//! `ReqwestFetch` against a loopback server; nothing leaves the machine.

use std::future::Future;
use std::time::Duration;

use nyaaai_agentic::fetch::ReqwestFetch;
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{Fetch, FetchError, FetchRequest, FetchResponse, Method};
use nyaaai_testkit::loopback::{Hold, Loopback, LoopbackExchange};

fn run<F: Future>(work: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a runtime")
        .block_on(work)
}

fn played(status: u16, chunks: &[&str]) -> LoopbackExchange {
    LoopbackExchange {
        status,
        headers: vec![
            ("Content-Type".to_owned(), "text/event-stream".to_owned()),
            ("X-Repeat".to_owned(), "a".to_owned()),
            ("x-repeat".to_owned(), "b".to_owned()),
        ],
        chunks: chunks
            .iter()
            .map(|chunk| chunk.as_bytes().to_vec())
            .collect(),
        hold: Hold::Never,
    }
}

fn post(url: String, body: &str) -> FetchRequest {
    FetchRequest {
        method: Method::Post,
        url,
        headers: vec![
            ("Content-Type".to_owned(), "application/json".to_owned()),
            ("X-Sent".to_owned(), "yes".to_owned()),
        ],
        body: Some(body.to_owned()),
    }
}

async fn drained(response: &mut FetchResponse) -> Result<Vec<u8>, FetchError> {
    let mut joined = Vec::new();
    while let Some(chunk) = response.chunks.next().await? {
        joined.extend(chunk);
    }
    Ok(joined)
}

#[test]
fn answers_status_headers_body_and_sends_its_request() {
    run(async {
        let server = Loopback::start(vec![played(201, &["data: one\n\n", "data: two\n\n"])])
            .await
            .unwrap();
        let fetch = ReqwestFetch::new().unwrap();
        let mut response = fetch
            .fetch(post(server.url("/v1/run"), "{\"a\":1}"), CancelToken::new())
            .await
            .unwrap();
        assert_eq!(response.status, 201);
        let held = |name: &str| {
            response
                .headers
                .iter()
                .find(|(each, _)| each == name)
                .map(|(_, value)| value.as_str())
        };
        assert_eq!(held("content-type"), Some("text/event-stream"));
        assert_eq!(held("x-repeat"), Some("a, b"));
        let names: Vec<&str> = response
            .headers
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        assert!(names.is_sorted());
        let body = drained(&mut response).await.unwrap();
        assert_eq!(body, b"data: one\n\ndata: two\n\n");
        let seen = server.requests();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].method, "POST");
        assert_eq!(seen[0].path, "/v1/run");
        assert_eq!(seen[0].body, b"{\"a\":1}");
        let header = |name: &str| {
            seen[0]
                .headers
                .iter()
                .find(|(held, _)| held == name)
                .map(|(_, value)| value.as_str())
        };
        assert_eq!(header("x-sent"), Some("yes"));
        assert_eq!(header("content-type"), Some("application/json"));
    });
}

#[test]
fn a_get_sends_no_body() {
    run(async {
        let server = Loopback::start(vec![played(200, &["{}"])]).await.unwrap();
        let fetch = ReqwestFetch::new().unwrap();
        let request = FetchRequest {
            method: Method::Get,
            url: server.url("/models"),
            headers: Vec::new(),
            body: Some("ignored".to_owned()),
        };
        let mut response = fetch.fetch(request, CancelToken::new()).await.unwrap();
        assert_eq!(drained(&mut response).await.unwrap(), b"{}");
        let seen = server.requests();
        assert_eq!(seen[0].method, "GET");
        assert!(seen[0].body.is_empty());
        assert!(
            seen[0]
                .headers
                .iter()
                .all(|(name, _)| name != "content-length")
        );
    });
}

#[test]
fn a_cancel_before_the_head_ends_cancelled_and_closes() {
    run(async {
        let mut held = played(200, &[]);
        held.hold = Hold::BeforeHead;
        let server = std::sync::Arc::new(Loopback::start(vec![held]).await.unwrap());
        let fetch = ReqwestFetch::new().unwrap();
        let cancel = CancelToken::new();
        let later = cancel.clone();
        let watching = std::sync::Arc::clone(&server);
        tokio::spawn(async move {
            assert!(watching.wait_requests(1).await);
            later.cancel();
        });
        let outcome = fetch.fetch(post(server.url("/"), "{}"), cancel).await;
        assert!(matches!(outcome, Err(FetchError::Cancelled)));
        assert!(server.wait_closed(1).await);
    });
}

#[test]
fn a_cancel_mid_body_ends_cancelled_and_closes() {
    run(async {
        let mut held = played(200, &["first"]);
        held.hold = Hold::AfterChunks;
        let server = Loopback::start(vec![held]).await.unwrap();
        let fetch = ReqwestFetch::new().unwrap();
        let cancel = CancelToken::new();
        let mut response = fetch
            .fetch(post(server.url("/"), "{}"), cancel.clone())
            .await
            .unwrap();
        assert_eq!(
            response.chunks.next().await.unwrap(),
            Some(b"first".to_vec())
        );
        let later = cancel.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            later.cancel();
        });
        let outcome = response.chunks.next().await;
        assert!(matches!(outcome, Err(FetchError::Cancelled)));
        assert!(matches!(
            response.chunks.next().await,
            Err(FetchError::Cancelled)
        ));
        assert!(server.wait_closed(1).await);
    });
}

#[test]
fn a_cancelled_token_makes_no_request() {
    run(async {
        let server = Loopback::start(vec![played(200, &["x"])]).await.unwrap();
        let fetch = ReqwestFetch::new().unwrap();
        let cancel = CancelToken::new();
        cancel.cancel();
        let outcome = fetch.fetch(post(server.url("/"), "{}"), cancel).await;
        assert!(matches!(outcome, Err(FetchError::Cancelled)));
        assert!(server.requests().is_empty());
    });
}

#[test]
fn a_refused_connection_fails() {
    run(async {
        let url = {
            let server = Loopback::start(Vec::new()).await.unwrap();
            server.url("/")
        };
        // The listener is gone with the server.
        tokio::time::sleep(Duration::from_millis(20)).await;
        let fetch = ReqwestFetch::new().unwrap();
        let outcome = fetch.fetch(post(url, "{}"), CancelToken::new()).await;
        assert!(matches!(outcome, Err(FetchError::Failed(_))));
    });
}

#[test]
fn a_dropped_body_closes_its_connection() {
    run(async {
        let mut held = played(200, &["first"]);
        held.hold = Hold::AfterChunks;
        let server = Loopback::start(vec![held]).await.unwrap();
        let fetch = ReqwestFetch::new().unwrap();
        let mut response = fetch
            .fetch(post(server.url("/"), "{}"), CancelToken::new())
            .await
            .unwrap();
        assert_eq!(
            response.chunks.next().await.unwrap(),
            Some(b"first".to_vec())
        );
        drop(response);
        assert!(server.wait_closed(1).await);
    });
}

#[test]
fn a_truncated_chunked_body_fails() {
    run(async {
        let mut held = played(200, &["ab"]);
        held.hold = Hold::Truncate;
        let server = Loopback::start(vec![held]).await.unwrap();
        let fetch = ReqwestFetch::new().unwrap();
        let mut response = fetch
            .fetch(post(server.url("/"), "{}"), CancelToken::new())
            .await
            .unwrap();
        let mut ended = None;
        for _ in 0..4 {
            match response.chunks.next().await {
                Ok(Some(_)) => {}
                other => {
                    ended = Some(other);
                    break;
                }
            }
        }
        assert!(matches!(ended, Some(Err(FetchError::Failed(_)))));
    });
}

#[test]
fn a_redirect_answers_its_status_and_is_not_followed() {
    run(async {
        let mut moved = played(302, &[]);
        moved
            .headers
            .push(("Location".to_owned(), "/next".to_owned()));
        let server = Loopback::start(vec![moved, played(200, &["x"])])
            .await
            .unwrap();
        let fetch = ReqwestFetch::new().unwrap();
        let mut response = fetch
            .fetch(post(server.url("/"), "{}"), CancelToken::new())
            .await
            .unwrap();
        assert_eq!(response.status, 302);
        assert!(drained(&mut response).await.unwrap().is_empty());
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(server.requests().len(), 1);
    });
}

#[test]
fn a_failed_send_holds_no_url() {
    run(async {
        let url = {
            let server = Loopback::start(Vec::new()).await.unwrap();
            server.url("/secret-path?token=abc")
        };
        tokio::time::sleep(Duration::from_millis(20)).await;
        let fetch = ReqwestFetch::new().unwrap();
        let Err(FetchError::Failed(text)) = fetch
            .fetch(post(url.clone(), "{}"), CancelToken::new())
            .await
        else {
            panic!("the send should fail");
        };
        assert!(!text.contains("secret-path"));
        assert!(!text.contains("token=abc"));
        assert!(!text.contains("http://"));
    });
}

#[test]
fn a_header_that_is_not_valid_is_refused_without_its_text() {
    run(async {
        let server = Loopback::start(vec![played(200, &["{}"])]).await.unwrap();
        let fetch = ReqwestFetch::new().unwrap();
        let mut request = post(server.url("/"), "{}");
        request
            .headers
            .push(("X-Bad".to_owned(), "line\r\nbreak-value".to_owned()));
        let Err(FetchError::Failed(text)) = fetch.fetch(request, CancelToken::new()).await else {
            panic!("the header should be refused");
        };
        assert!(!text.contains("break-value"));
        assert!(server.requests().is_empty());
    });
}

fn request_of() -> nyaaai_contract::shapes::step::CanonicalRequest {
    use nyaaai_contract::shapes::step::{
        CanonicalRequest, CanonicalRequestCache, CanonicalRequestCacheKind,
    };
    CanonicalRequest {
        model: "vendor/model-1".to_owned(),
        system: vec!["Be brief.".to_owned()],
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
        session: "session-1".to_owned(),
        zdr_only: false,
    }
}

#[test]
fn the_transport_runs_over_the_adapter() {
    use std::sync::Arc;

    use nyaaai_agentic::dialects::openrouter::OPENROUTER_CHAT;
    use nyaaai_agentic::transport::{Auth, Transport, TransportDeps, TransportError, static_key};
    use nyaaai_contract::shapes::step::StepEvent;
    use nyaaai_contract::time::SystemScheduler;

    const KEY: &str = "fake-key-one";
    run(async {
        let stream = "data: {\"id\":\"gen-7\",\"choices\":[{\"delta\":{\"content\":\"hi\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n";
        let mut failing = played(400, &[]);
        failing.chunks =
            vec![format!("{{\"error\":{{\"message\":\"bad {KEY} here\"}}}}").into_bytes()];
        failing.headers = vec![("Content-Type".to_owned(), "application/json".to_owned())];
        let server = Loopback::start(vec![played(200, &[stream]), failing])
            .await
            .unwrap();
        let logged = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let sink = Arc::clone(&logged);
        let transport = Transport::new(TransportDeps {
            fetch: Arc::new(ReqwestFetch::new().unwrap()),
            scheduler: Arc::new(SystemScheduler::new(tokio::runtime::Handle::current())),
            key: Arc::new(static_key(KEY)),
            log: Box::new(move |line| sink.lock().unwrap().push(line.to_owned())),
            base_url: server.url(""),
            label: "Loop".to_owned(),
            user_agent: "test/1".to_owned(),
            errors: Arc::new(OPENROUTER_CHAT),
            auth: Auth::Bearer,
            idle_ms: None,
            max_bytes: None,
        });
        let request = request_of();
        let cancel = CancelToken::new();
        let mut events = transport.stream(&OPENROUTER_CHAT, &request, &cancel);
        let mut seen = Vec::new();
        while let Some(event) = events.next().await.unwrap() {
            seen.push(event);
        }
        drop(events);
        assert!(
            seen.iter()
                .any(|event| matches!(event, StepEvent::Text(text) if text.text == "hi"))
        );
        assert!(
            seen.iter()
                .any(|event| matches!(event, StepEvent::Finish(_)))
        );
        let failed = transport
            .complete(&OPENROUTER_CHAT, &request, &cancel)
            .await;
        let Err(TransportError::Provider(error)) = failed else {
            panic!("the second call should fail");
        };
        assert!(error.failure.message.contains("[key]"));
        assert!(!error.failure.message.contains(KEY));
        let sent = server.requests();
        let auth = sent[0]
            .headers
            .iter()
            .find(|(name, _)| name == "authorization");
        assert_eq!(
            auth.map(|(_, value)| value.as_str()),
            Some(format!("Bearer {KEY}").as_str())
        );
        assert_eq!(logged.lock().unwrap().len(), 2);
    });
}
