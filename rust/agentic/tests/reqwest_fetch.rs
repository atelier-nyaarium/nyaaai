//! `ReqwestFetch` against a loopback server; nothing leaves the machine.

use std::future::Future;
use std::time::Duration;

use nyaaai_agentic::fetch::ReqwestFetch;
use nyaaai_contract::cancel::CancelToken;
use nyaaai_contract::fetch::{Fetch, FetchError, FetchRequest, FetchResponse, Method};
use nyaaai_testkit::loopback::{Hold, Loopback, LoopbackExchange};
use nyaaai_testkit::trace::read_trace;

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
fn plays_a_recorded_trace() {
    run(async {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../cases/traces/openrouter.title.json");
        let trace = read_trace(&file).unwrap();
        let exchanges = LoopbackExchange::from_trace(&trace).unwrap();
        let server = Loopback::start(exchanges).await.unwrap();
        let fetch = ReqwestFetch::new().unwrap();
        for exchange in &trace.exchanges {
            let mut response = fetch
                .fetch(post(server.url("/"), "{}"), CancelToken::new())
                .await
                .unwrap();
            assert_eq!(u64::from(response.status), exchange.status);
            let joined: Vec<u8> = exchange
                .chunks
                .iter()
                .flat_map(|chunk| nyaaai_contract::base64::base64_decode(chunk).unwrap())
                .collect();
            assert_eq!(drained(&mut response).await.unwrap(), joined);
        }
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
