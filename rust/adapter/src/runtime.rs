//! Stdio adapter.

use std::collections::{HashMap, HashSet, VecDeque};
use std::future::{Future, poll_fn};
use std::io::{Read, Write};
use std::panic::{self, AssertUnwindSafe};
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::task::{Context, Poll, Waker};

use tokio::task::JoinHandle;

pub use nyaaai_contract::cancel::CancelToken;
pub use nyaaai_contract::future::BoxFuture;
use nyaaai_contract::id::{CountedIds, Ids, RandomIds};
use nyaaai_contract::json::{Value, parse_json, stringify_json};
use nyaaai_contract::shape::{Path, PathPart, Shape, compatible_versions};
use nyaaai_contract::shapes::frames::{
    self, AdapterFrame, Done, Failed, Fault, FaultCode, HostFrame, Reject, Request, RunEvent,
    RunFailure, Start, Welcome,
};
use nyaaai_contract::time::{Clock, SteppedClock, SystemClock};
use nyaaai_contract::version::CONTRACT_VERSION;

use nyaaai_contract::adapter_texts::{
    BAD_INPUT, CRASHED, GREETED_TWICE, INTERRUPTED, MISFIT, NO_COMMAND, NO_SUCH_REQUEST,
    NO_SUCH_RUN, NOT_HOST_FRAME, NOT_JSON, OVERSIZED_FRAME, REJECT_REASON, RUN_OPEN, SAY_HELLO,
};
pub use nyaaai_contract::adapter_texts::{EXIT_REJECTED, MAX_FRAME_BYTES};

/// The runtime would not start.
pub const EXIT_FAILED: u8 = 1;

const READ_CHUNK_BYTES: usize = 1 << 16;

pub enum Inbound {
    Steer(String),
    Interrupt,
    Reply { id: String, value: Value },
    ReplyFailed { id: String, message: String },
}

impl Inbound {
    fn answers(&self, asked: &str) -> bool {
        matches!(self, Inbound::Reply { id, .. } | Inbound::ReplyFailed { id, .. } if id == asked)
    }
}

pub enum RunError {
    Interrupted,
    BadInput(Path),
    HostRequest(String),
    UnknownCommand(String),
    Failed(String),
}

impl From<std::io::Error> for RunError {
    fn from(error: std::io::Error) -> Self {
        RunError::Failed(error.to_string())
    }
}

/// What a run answers.
pub type Command = BoxFuture<'static, Result<Value, RunError>>;

/// `None` is an unknown command.
pub type Commands = fn(&str, Value, RunContext) -> Option<Command>;

/// A command of blocking work.
pub type BlockingCommands = fn(&str, &Value, &BlockingContext) -> Option<Result<Value, RunError>>;

/// Decodes input, and holds the output to its shape.
pub fn typed<Input: Shape, Output: Shape>(
    input: &Value,
    run: impl FnOnce(Input) -> Result<Output, RunError>,
) -> Result<Value, RunError> {
    let input = Input::decode(input).map_err(|refused| RunError::BadInput(refused.path))?;
    held_to(run(input)?)
}

/// `typed`, awaited.
pub async fn typed_async<Input, Output, Work>(
    input: &Value,
    run: impl FnOnce(Input) -> Work,
) -> Result<Value, RunError>
where
    Input: Shape,
    Output: Shape,
    Work: Future<Output = Result<Output, RunError>>,
{
    let input = Input::decode(input).map_err(|refused| RunError::BadInput(refused.path))?;
    held_to(run(input).await?)
}

fn held_to<Output: Shape>(made: Output) -> Result<Value, RunError> {
    Output::decode(&made.encode())
        .map(|held| held.encode())
        .map_err(|_| RunError::Failed(MISFIT.to_owned()))
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

pub fn frame_path(path: Path) -> Vec<frames::PathPart> {
    path.into_iter()
        .map(|part| match part {
            PathPart::Key(key) => frames::PathPart::Text(key),
            PathPart::Index(index) => frames::PathPart::Integer(index as u64),
        })
        .collect()
}

struct Out(Mutex<Box<dyn Write + Send>>);

impl Out {
    fn frame(&self, frame: &AdapterFrame) {
        let mut line = stringify_json(&frame.encode());
        line.push('\n');
        let mut out = locked(&self.0);
        let _ = out.write_all(line.as_bytes()).and_then(|()| out.flush());
    }

    fn fault(&self, code: FaultCode, message: &str, run: Option<&str>, path: Option<Path>) {
        self.frame(&AdapterFrame::Fault(Fault {
            code,
            run: run.map(str::to_owned),
            path: path.map(frame_path),
            message: message.to_owned(),
        }));
    }
}

type Pending = Arc<Mutex<HashSet<String>>>;

/// Messages for one run. Any number of tasks may wait on it.
#[derive(Default)]
struct Mailbox {
    state: Mutex<MailState>,
}

#[derive(Default)]
struct MailState {
    items: VecDeque<Inbound>,
    interrupted: bool,
    waiting: Vec<Waker>,
}

impl Mailbox {
    /// Nothing follows an interrupt.
    fn push(&self, inbound: Inbound) {
        let woken = {
            let mut state = locked(&self.state);
            if state.interrupted {
                return;
            }
            if matches!(inbound, Inbound::Interrupt) {
                state.interrupted = true;
            } else {
                state.items.push_back(inbound);
            }
            std::mem::take(&mut state.waiting)
        };
        for waker in woken {
            waker.wake();
        }
    }

    /// The first held message `wanted` takes; `None` once interrupted with none held.
    fn take(
        &self,
        waker: &Context<'_>,
        wanted: impl Fn(&Inbound) -> bool,
    ) -> Poll<Option<Inbound>> {
        let mut state = locked(&self.state);
        if let Some(at) = state.items.iter().position(wanted) {
            return Poll::Ready(state.items.remove(at));
        }
        if state.interrupted {
            return Poll::Ready(None);
        }
        if !state
            .waiting
            .iter()
            .any(|held| held.will_wake(waker.waker()))
        {
            state.waiting.push(waker.waker().clone());
        }
        Poll::Pending
    }

    fn steers(&self) -> Vec<String> {
        let mut steers = Vec::new();
        locked(&self.state).items.retain(|held| match held {
            Inbound::Steer(text) => {
                steers.push(text.clone());
                false
            }
            _ => true,
        });
        steers
    }
}

struct Shared {
    run: String,
    out: Arc<Out>,
    mailbox: Arc<Mailbox>,
    pending: Pending,
    seq: AtomicU64,
    asked: AtomicU64,
}

/// One run's end of the host. Clones share the run.
#[derive(Clone)]
pub struct RunContext {
    pub clock: Arc<dyn Clock + Send + Sync>,
    pub ids: Arc<dyn Ids + Send + Sync>,
    cancel: CancelToken,
    shared: Arc<Shared>,
}

impl RunContext {
    fn next_seq(&self) -> u64 {
        self.shared.seq.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Ends on an interrupt or the input closing.
    pub fn token(&self) -> &CancelToken {
        &self.cancel
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }

    pub fn emit(&self, event: Value) {
        self.shared.out.frame(&AdapterFrame::Event(RunEvent {
            run: self.shared.run.clone(),
            seq: self.next_seq(),
            event,
        }));
    }

    /// Returns host request id.
    pub fn ask(&self, kind: &str, input: Value) -> String {
        let asked = self.shared.asked.fetch_add(1, Ordering::Relaxed) + 1;
        let id = format!("q{asked}");
        locked(&self.shared.pending).insert(id.clone());
        self.shared.out.frame(&AdapterFrame::Request(Request {
            run: self.shared.run.clone(),
            id: id.clone(),
            kind: kind.to_owned(),
            input,
        }));
        id
    }

    /// Interrupts repeat forever.
    pub async fn receive(&self) -> Inbound {
        let mailbox = &self.shared.mailbox;
        poll_fn(|waker| mailbox.take(waker, |_| true))
            .await
            .unwrap_or(Inbound::Interrupt)
    }

    /// Other messages remain queued.
    pub async fn reply(&self, asked: &str) -> Result<Value, RunError> {
        let mailbox = &self.shared.mailbox;
        match poll_fn(|waker| mailbox.take(waker, |held| held.answers(asked))).await {
            Some(Inbound::Reply { value, .. }) => Ok(value),
            Some(Inbound::ReplyFailed { message, .. }) => Err(RunError::HostRequest(message)),
            _ => Err(RunError::Interrupted),
        }
    }

    /// Interrupts fail the request.
    pub async fn request(&self, kind: &str, input: Value) -> Result<Value, RunError> {
        self.reply(&self.ask(kind, input)).await
    }

    pub fn steers(&self) -> Vec<String> {
        self.shared.mailbox.steers()
    }
}

/// What a blocking command sees of its run.
pub struct BlockingContext {
    pub clock: Arc<dyn Clock + Send + Sync>,
    pub ids: Arc<dyn Ids + Send + Sync>,
    cancel: CancelToken,
}

impl BlockingContext {
    pub fn is_cancelled(&self) -> bool {
        self.cancel.is_cancelled()
    }
}

/// Runs blocking work on the blocking pool.
pub fn blocking(
    context: &RunContext,
    work: impl FnOnce(&BlockingContext) -> Result<Value, RunError> + Send + 'static,
) -> Command {
    let context = BlockingContext {
        clock: Arc::clone(&context.clock),
        ids: Arc::clone(&context.ids),
        cancel: context.cancel.clone(),
    };
    Box::pin(async move {
        tokio::task::spawn_blocking(move || work(&context))
            .await
            .unwrap_or_else(|_| Err(RunError::Failed(CRASHED.to_owned())))
    })
}

/// Turns a panic into a value.
struct Guarded(Command);

impl Future for Guarded {
    type Output = Option<Result<Value, RunError>>;

    fn poll(self: Pin<&mut Self>, waker: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = &mut self.get_mut().0;
        match panic::catch_unwind(AssertUnwindSafe(|| inner.as_mut().poll(waker))) {
            Ok(Poll::Pending) => Poll::Pending,
            Ok(Poll::Ready(result)) => Poll::Ready(Some(result)),
            Err(_) => Poll::Ready(None),
        }
    }
}

enum Line {
    Bytes(Vec<u8>),
    /// Oversized lines notify once.
    Oversized,
}

/// LF only. Tails drop.
struct FrameReader<R> {
    input: R,
    chunk: Vec<u8>,
    from: usize,
    held: Vec<u8>,
    skipping: bool,
}

impl<R: Read> FrameReader<R> {
    fn new(input: R) -> Self {
        Self {
            input,
            chunk: Vec::new(),
            from: 0,
            held: Vec::new(),
            skipping: false,
        }
    }

    fn next(&mut self) -> Option<Line> {
        loop {
            if self.from >= self.chunk.len() {
                self.chunk.resize(READ_CHUNK_BYTES, 0);
                let read = self.input.read(&mut self.chunk).unwrap_or(0);
                self.chunk.truncate(read);
                self.from = 0;
                if read == 0 {
                    return None;
                }
            }
            let rest = &self.chunk[self.from..];
            let at = rest.iter().position(|byte| *byte == b'\n');
            let end = at.unwrap_or(rest.len());
            let mut told = false;
            if !self.skipping {
                if self.held.len() + end > MAX_FRAME_BYTES {
                    self.skipping = true;
                    self.held = Vec::new();
                    told = true;
                } else {
                    self.held.extend_from_slice(&rest[..end]);
                }
            }
            self.from += end + usize::from(at.is_some());
            let mut line = None;
            if at.is_some() {
                let skipped = std::mem::replace(&mut self.skipping, false);
                let held = std::mem::take(&mut self.held);
                if !skipped {
                    line = Some(Line::Bytes(held));
                }
            }
            if told {
                return Some(Line::Oversized);
            }
            if line.is_some() {
                return line;
            }
        }
    }
}

struct RunHandle {
    inbox: Arc<Mailbox>,
    pending: Pending,
    cancel: CancelToken,
}

type Runs = Arc<Mutex<HashMap<String, RunHandle>>>;

type Launch = dyn Fn(&str, Value, RunContext) -> Option<Command> + Send + Sync;

fn failed(context: &RunContext, error: RunError) -> AdapterFrame {
    let (code, path, message) = match error {
        RunError::Interrupted => (RunFailure::Interrupted, None, INTERRUPTED.to_owned()),
        RunError::BadInput(path) => (
            RunFailure::BadInput,
            Some(frame_path(path)),
            BAD_INPUT.to_owned(),
        ),
        RunError::HostRequest(message) => (RunFailure::HostRequest, None, message),
        RunError::UnknownCommand(message) => (RunFailure::UnknownCommand, None, message),
        RunError::Failed(message) => (RunFailure::Error, None, message),
    };
    AdapterFrame::Failed(Failed {
        run: context.shared.run.clone(),
        seq: context.next_seq(),
        code,
        path,
        message,
    })
}

fn start(
    frame: Start,
    launch: &Arc<Launch>,
    out: &Arc<Out>,
    runs: &Runs,
    pool: &tokio::runtime::Handle,
) -> JoinHandle<()> {
    let inbox = Arc::new(Mailbox::default());
    let pending = Pending::default();
    let cancel = CancelToken::new();
    let (clock, ids): (Arc<dyn Clock + Send + Sync>, Arc<dyn Ids + Send + Sync>) = match frame.fixed
    {
        Some(fixed) => (
            Arc::new(SteppedClock::new(fixed.now_ms, fixed.step_ms as i64)),
            Arc::new(CountedIds::new(fixed.id_prefix)),
        ),
        None => (Arc::new(SystemClock), Arc::new(RandomIds)),
    };
    let context = RunContext {
        clock,
        ids,
        cancel: cancel.clone(),
        shared: Arc::new(Shared {
            run: frame.run.clone(),
            out: Arc::clone(out),
            mailbox: Arc::clone(&inbox),
            pending: Arc::clone(&pending),
            seq: AtomicU64::new(0),
            asked: AtomicU64::new(0),
        }),
    };
    locked(runs).insert(
        frame.run.clone(),
        RunHandle {
            inbox,
            pending,
            cancel,
        },
    );
    let runs = Arc::clone(runs);
    let launch = Arc::clone(launch);
    pool.spawn(async move {
        let launched = panic::catch_unwind(AssertUnwindSafe(|| {
            launch(&frame.command, frame.input, context.clone())
        }));
        let worked = match launched {
            Ok(Some(command)) => Guarded(command).await,
            Ok(None) => Some(Err(RunError::UnknownCommand(format!(
                "{NO_COMMAND}{}",
                frame.command
            )))),
            Err(_) => None,
        };
        let terminal = match worked {
            Some(Ok(result)) => AdapterFrame::Done(Done {
                run: frame.run.clone(),
                seq: context.next_seq(),
                result,
            }),
            Some(Err(error)) => failed(&context, error),
            None => failed(&context, RunError::Failed(CRASHED.to_owned())),
        };
        locked(&runs).remove(&frame.run);
        context.shared.out.frame(&terminal);
    })
}

/// Serves until EOF. Commands are awaited.
pub fn serve(input: impl Read, out: Box<dyn Write + Send>, commands: Commands) -> u8 {
    serve_with(input, out, Arc::new(commands))
}

/// Serves until EOF. Commands are blocking work on the blocking pool.
pub fn serve_blocking(
    input: impl Read,
    out: Box<dyn Write + Send>,
    commands: BlockingCommands,
) -> u8 {
    serve_with(
        input,
        out,
        Arc::new(move |name, input, context| {
            let name = name.to_owned();
            Some(blocking(&context, move |blocked| {
                commands(&name, &input, blocked)
                    .unwrap_or_else(|| Err(RunError::UnknownCommand(format!("{NO_COMMAND}{name}"))))
            }))
        }),
    )
}

fn serve_with(input: impl Read, out: Box<dyn Write + Send>, launch: Arc<Launch>) -> u8 {
    let Ok(runtime) = tokio::runtime::Builder::new_multi_thread()
        .enable_time()
        .build()
    else {
        return EXIT_FAILED;
    };
    let pool = runtime.handle().clone();
    let out = Arc::new(Out(Mutex::new(out)));
    let runs = Runs::default();
    let mut tasks: Vec<JoinHandle<()>> = Vec::new();
    let mut reader = FrameReader::new(input);
    let mut greeted = false;

    while let Some(line) = reader.next() {
        let bytes = match line {
            Line::Oversized => {
                out.fault(FaultCode::OversizedFrame, OVERSIZED_FRAME, None, None);
                continue;
            }
            Line::Bytes(bytes) if bytes.is_empty() => continue,
            Line::Bytes(bytes) => bytes,
        };
        let Some(value) = parse_json(&String::from_utf8_lossy(&bytes)) else {
            out.fault(FaultCode::BadFrame, NOT_JSON, None, None);
            continue;
        };
        let frame = match HostFrame::decode(&value) {
            Ok(frame) => frame,
            Err(refused) => {
                out.fault(
                    FaultCode::BadFrame,
                    NOT_HOST_FRAME,
                    None,
                    Some(refused.path),
                );
                continue;
            }
        };
        if !greeted {
            match frame {
                HostFrame::Hello(hello)
                    if compatible_versions(&hello.version, CONTRACT_VERSION) =>
                {
                    greeted = true;
                    out.frame(&AdapterFrame::Welcome(Welcome {
                        version: CONTRACT_VERSION.to_owned(),
                    }));
                }
                HostFrame::Hello(_) => {
                    out.frame(&AdapterFrame::Reject(Reject {
                        version: CONTRACT_VERSION.to_owned(),
                        reason: REJECT_REASON.to_owned(),
                    }));
                    return EXIT_REJECTED;
                }
                _ => out.fault(FaultCode::NotGreeted, SAY_HELLO, None, None),
            }
            continue;
        }
        let (run, inbound) = match frame {
            HostFrame::Hello(_) => {
                out.fault(FaultCode::BadFrame, GREETED_TWICE, None, None);
                continue;
            }
            HostFrame::Start(frame) => {
                if locked(&runs).contains_key(&frame.run) {
                    out.fault(FaultCode::DuplicateRun, RUN_OPEN, Some(&frame.run), None);
                } else {
                    tasks.retain(|task| !task.is_finished());
                    tasks.push(start(frame, &launch, &out, &runs, &pool));
                }
                continue;
            }
            HostFrame::Steer(frame) => (frame.run, Inbound::Steer(frame.text)),
            HostFrame::Interrupt(frame) => (frame.run, Inbound::Interrupt),
            HostFrame::Reply(frame) => (
                frame.run,
                Inbound::Reply {
                    id: frame.id,
                    value: frame.value,
                },
            ),
            HostFrame::ReplyFailed(frame) => (
                frame.run,
                Inbound::ReplyFailed {
                    id: frame.id,
                    message: frame.message,
                },
            ),
        };
        let held = locked(&runs);
        let Some(handle) = held.get(&run) else {
            out.fault(FaultCode::UnknownRun, NO_SUCH_RUN, Some(&run), None);
            continue;
        };
        let answered = match &inbound {
            Inbound::Reply { id, .. } | Inbound::ReplyFailed { id, .. } => {
                locked(&handle.pending).remove(id)
            }
            _ => true,
        };
        if answered {
            let interrupt = matches!(inbound, Inbound::Interrupt);
            handle.inbox.push(inbound);
            if interrupt {
                handle.cancel.cancel();
            }
        } else {
            out.fault(FaultCode::UnknownReply, NO_SUCH_REQUEST, Some(&run), None);
        }
    }

    for handle in locked(&runs).values() {
        handle.inbox.push(Inbound::Interrupt);
        handle.cancel.cancel();
    }
    runtime.block_on(async {
        for task in tasks {
            let _ = task.await;
        }
    });
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Sink(Arc<Mutex<Vec<u8>>>);

    impl Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            locked(&self.0).extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn script(commands: &[&str]) -> String {
        let mut lines = vec![format!(
            r#"{{"type":"hello","version":"{CONTRACT_VERSION}"}}"#
        )];
        for command in commands {
            lines.push(format!(
                r#"{{"type":"start","run":"{command}","command":"{command}","input":null}}"#
            ));
        }
        lines.join("\n") + "\n"
    }

    fn served(input: &str, run: impl FnOnce(&[u8], Box<dyn Write + Send>) -> u8) -> String {
        let held = Arc::new(Mutex::new(Vec::new()));
        assert_eq!(run(input.as_bytes(), Box::new(Sink(Arc::clone(&held)))), 0);
        String::from_utf8(locked(&held).clone()).unwrap()
    }

    fn crashed(text: &str, run: &str) -> bool {
        text.split('\n').any(|line| {
            line.contains(&format!(r#""run":"{run}""#))
                && line.contains(r#""type":"failed""#)
                && line.contains("the command crashed")
        })
    }

    #[test]
    fn a_panic_in_an_awaited_future_fails_the_run_and_serving_goes_on() {
        fn commands(name: &str, _: Value, _: RunContext) -> Option<Command> {
            match name {
                "later" => Some(Box::pin(async {
                    tokio::task::yield_now().await;
                    panic::resume_unwind(Box::new("crash"))
                })),
                "fine" => Some(Box::pin(async { Ok(Value::Null) })),
                _ => None,
            }
        }
        let text = served(&script(&["later", "fine"]), |input, out| {
            serve(input, out, commands)
        });
        assert!(crashed(&text, "later"), "{text}");
        assert!(text.contains(r#""type":"done""#), "{text}");
    }

    #[test]
    fn a_panic_in_a_blocking_arm_fails_the_run_and_serving_goes_on() {
        fn commands(name: &str, _: &Value, _: &BlockingContext) -> Option<Result<Value, RunError>> {
            match name {
                "boom" => panic::resume_unwind(Box::new("crash")),
                "fine" => Some(Ok(Value::Null)),
                _ => None,
            }
        }
        let text = served(&script(&["boom", "fine", "none"]), |input, out| {
            serve_blocking(input, out, commands)
        });
        assert!(crashed(&text, "boom"), "{text}");
        assert!(text.contains(r#""type":"done""#), "{text}");
        assert!(text.contains("no command none"), "{text}");
    }

    #[test]
    fn the_input_closing_cancels_a_run_that_never_reads_its_inbox() {
        fn commands(_: &str, _: Value, context: RunContext) -> Option<Command> {
            Some(Box::pin(async move {
                context.token().cancelled().await;
                Err(RunError::Interrupted)
            }))
        }
        let text = served(&script(&["wait"]), |input, out| serve(input, out, commands));
        assert!(text.contains(r#""code":"interrupted""#), "{text}");
    }

    fn context() -> (RunContext, Arc<Mailbox>) {
        let mailbox = Arc::new(Mailbox::default());
        let context = RunContext {
            clock: Arc::new(SystemClock),
            ids: Arc::new(RandomIds),
            cancel: CancelToken::new(),
            shared: Arc::new(Shared {
                run: "t".to_owned(),
                out: Arc::new(Out(Mutex::new(Box::new(std::io::sink())))),
                mailbox: Arc::clone(&mailbox),
                pending: Pending::default(),
                seq: AtomicU64::new(0),
                asked: AtomicU64::new(0),
            }),
        };
        (context, mailbox)
    }

    fn reply_of(id: &str, value: &str) -> Inbound {
        Inbound::Reply {
            id: id.to_owned(),
            value: Value::from(value.to_owned()),
        }
    }

    /// Lets spawned waiters reach their await.
    async fn settle() {
        for _ in 0..5 {
            tokio::task::yield_now().await;
        }
    }

    fn pool() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
    }

    #[test]
    fn two_replies_awaited_at_once_each_get_their_own_in_reverse_order() {
        let (context, mailbox) = context();
        pool().block_on(async {
            let (first, second) = (context.clone(), context.clone());
            let first = tokio::spawn(async move { first.reply("q1").await });
            let second = tokio::spawn(async move { second.reply("q2").await });
            settle().await;
            mailbox.push(reply_of("q2", "two"));
            mailbox.push(reply_of("q1", "one"));
            assert!(matches!(first.await.unwrap(), Ok(Value::String(text)) if text == "one"));
            assert!(matches!(second.await.unwrap(), Ok(Value::String(text)) if text == "two"));
        });
    }

    #[test]
    fn a_receive_and_a_reply_pending_together_each_get_theirs() {
        let (context, mailbox) = context();
        pool().block_on(async {
            let (heard, asked) = (context.clone(), context.clone());
            let heard = tokio::spawn(async move { heard.receive().await });
            let asked = tokio::spawn(async move { asked.reply("q1").await });
            settle().await;
            mailbox.push(Inbound::Steer("hey".to_owned()));
            mailbox.push(reply_of("q1", "one"));
            assert!(matches!(heard.await.unwrap(), Inbound::Steer(text) if text == "hey"));
            assert!(matches!(asked.await.unwrap(), Ok(Value::String(text)) if text == "one"));
        });
    }

    #[test]
    fn an_interrupt_ends_every_pending_wait() {
        let (context, mailbox) = context();
        pool().block_on(async {
            let (heard, first, second) = (context.clone(), context.clone(), context.clone());
            let heard = tokio::spawn(async move { heard.receive().await });
            let first = tokio::spawn(async move { first.reply("q1").await });
            let second = tokio::spawn(async move { second.reply("q2").await });
            settle().await;
            mailbox.push(Inbound::Interrupt);
            assert!(matches!(heard.await.unwrap(), Inbound::Interrupt));
            assert!(matches!(first.await.unwrap(), Err(RunError::Interrupted)));
            assert!(matches!(second.await.unwrap(), Err(RunError::Interrupted)));
        });
    }
}
