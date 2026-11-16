//! A loopback HTTP/1.1 server that plays recorded exchanges.

use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use nyaaai_contract::base64::base64_decode;
use nyaaai_contract::text::ascii_lower;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tokio::task::JoinHandle;

use crate::trace_shapes::Trace;

/// Where the server stops answering.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Hold {
    /// The body ends and the connection closes.
    #[default]
    Never,
    /// Nothing is answered after the request.
    BeforeHead,
    /// The chunks go out and the body stays open.
    AfterChunks,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopbackExchange {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    /// Each flushed apart, as a chunk of its own.
    pub chunks: Vec<Vec<u8>>,
    pub hold: Hold,
}

impl LoopbackExchange {
    /// The exchanges of a trace, none held.
    pub fn from_trace(trace: &Trace) -> io::Result<Vec<Self>> {
        trace
            .exchanges
            .iter()
            .map(|exchange| {
                let chunks = exchange
                    .chunks
                    .iter()
                    .map(|chunk| {
                        base64_decode(chunk)
                            .ok_or_else(|| io::Error::other("a trace chunk is not base64"))
                    })
                    .collect::<io::Result<Vec<_>>>()?;
                Ok(Self {
                    status: exchange.status as u16,
                    headers: exchange
                        .headers
                        .iter()
                        .map(|(name, value)| (name.to_owned(), value.clone()))
                        .collect(),
                    chunks,
                    hold: Hold::Never,
                })
            })
            .collect()
    }
}

/// A request as the server read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeenRequest {
    pub method: String,
    pub path: String,
    /// Lowercase names, in arrival order.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

struct State {
    requests: Vec<SeenRequest>,
}

/// Serves the exchanges in order, one per connection.
pub struct Loopback {
    addr: SocketAddr,
    state: Arc<Mutex<State>>,
    closed: Arc<AtomicUsize>,
    wake: Arc<Notify>,
    task: JoinHandle<()>,
}

fn locked(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Loopback {
    pub async fn start(exchanges: Vec<LoopbackExchange>) -> io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let state = Arc::new(Mutex::new(State {
            requests: Vec::new(),
        }));
        let closed = Arc::new(AtomicUsize::new(0));
        let wake = Arc::new(Notify::new());
        let task = tokio::spawn(serve(
            listener,
            exchanges,
            Arc::clone(&state),
            Arc::clone(&closed),
            Arc::clone(&wake),
        ));
        Ok(Self {
            addr,
            state,
            closed,
            wake,
            task,
        })
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.addr)
    }

    pub fn requests(&self) -> Vec<SeenRequest> {
        locked(&self.state).requests.clone()
    }

    /// Connections the client closed while the server held them.
    pub fn closed(&self) -> usize {
        self.closed.load(Ordering::SeqCst)
    }

    /// Waits for `count` held connections to be closed by the client.
    pub async fn wait_closed(&self, count: usize) -> bool {
        let waiting = async {
            loop {
                let woken = self.wake.notified();
                if self.closed() >= count {
                    return;
                }
                woken.await;
            }
        };
        tokio::time::timeout(Duration::from_secs(10), waiting)
            .await
            .is_ok()
    }

    /// Waits until `count` requests have been read.
    pub async fn wait_requests(&self, count: usize) -> bool {
        let waiting = async {
            loop {
                let woken = self.wake.notified();
                if self.requests().len() >= count {
                    return;
                }
                woken.await;
            }
        };
        tokio::time::timeout(Duration::from_secs(10), waiting)
            .await
            .is_ok()
    }
}

impl Drop for Loopback {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn serve(
    listener: TcpListener,
    exchanges: Vec<LoopbackExchange>,
    state: Arc<Mutex<State>>,
    closed: Arc<AtomicUsize>,
    wake: Arc<Notify>,
) {
    for exchange in exchanges {
        let Ok((stream, _)) = listener.accept().await else {
            return;
        };
        let state = Arc::clone(&state);
        let closed = Arc::clone(&closed);
        let wake = Arc::clone(&wake);
        // One connection after another keeps the script in order.
        let _ = tokio::spawn(async move {
            let _ = answer(stream, exchange, &state, &closed, &wake).await;
        })
        .await;
    }
}

fn find(bytes: &[u8], needle: &[u8]) -> Option<usize> {
    bytes
        .windows(needle.len())
        .position(|window| window == needle)
}

async fn read_request(stream: &mut TcpStream) -> io::Result<SeenRequest> {
    let mut held: Vec<u8> = Vec::new();
    let end = loop {
        if let Some(at) = find(&held, b"\r\n\r\n") {
            break at;
        }
        let mut buffer = [0u8; 4096];
        let read = stream.read(&mut buffer).await?;
        if read == 0 {
            return Err(io::Error::other("the client closed before its request"));
        }
        held.extend_from_slice(&buffer[..read]);
    };
    let head = String::from_utf8_lossy(&held[..end]).into_owned();
    let mut lines = head.split("\r\n");
    let first = lines.next().unwrap_or_default();
    let mut parts = first.split(' ');
    let method = parts.next().unwrap_or_default().to_owned();
    let path = parts.next().unwrap_or_default().to_owned();
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (ascii_lower(name), value.trim_matches(' ').to_owned()))
        .collect();
    let length = headers
        .iter()
        .find(|(name, _)| name == "content-length")
        .and_then(|(_, value)| value.parse::<usize>().ok())
        .unwrap_or(0);
    let mut body = held[end + 4..].to_vec();
    while body.len() < length {
        let mut buffer = [0u8; 4096];
        let read = stream.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        body.extend_from_slice(&buffer[..read]);
    }
    Ok(SeenRequest {
        method,
        path,
        headers,
        body,
    })
}

/// Reads to the end: the client closed.
async fn until_closed(stream: &mut TcpStream) {
    let mut buffer = [0u8; 256];
    while matches!(stream.read(&mut buffer).await, Ok(read) if read > 0) {}
}

async fn answer(
    mut stream: TcpStream,
    exchange: LoopbackExchange,
    state: &Mutex<State>,
    closed: &AtomicUsize,
    wake: &Notify,
) -> io::Result<()> {
    let request = read_request(&mut stream).await?;
    locked(state).requests.push(request);
    wake.notify_waiters();
    if exchange.hold == Hold::BeforeHead {
        until_closed(&mut stream).await;
        closed.fetch_add(1, Ordering::SeqCst);
        wake.notify_waiters();
        return Ok(());
    }
    let mut head = format!("HTTP/1.1 {} Played\r\n", exchange.status);
    for (name, value) in &exchange.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    head.push_str("Transfer-Encoding: chunked\r\nConnection: close\r\n\r\n");
    stream.write_all(head.as_bytes()).await?;
    stream.flush().await?;
    for chunk in &exchange.chunks {
        let mut framed = format!("{:x}\r\n", chunk.len()).into_bytes();
        framed.extend_from_slice(chunk);
        framed.extend_from_slice(b"\r\n");
        stream.write_all(&framed).await?;
        stream.flush().await?;
        // Apart on the wire.
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    if exchange.hold == Hold::AfterChunks {
        until_closed(&mut stream).await;
        closed.fetch_add(1, Ordering::SeqCst);
        wake.notify_waiters();
        return Ok(());
    }
    stream.write_all(b"0\r\n\r\n").await?;
    stream.flush().await?;
    stream.shutdown().await
}
