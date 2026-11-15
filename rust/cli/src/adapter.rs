//! Stdio adapter.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::{Read, Write};
use std::panic::{self, AssertUnwindSafe};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};

use nyaaai_contract::id::{CountedIds, Ids, RandomIds};
use nyaaai_contract::json::{Value, parse_json, stringify_json};
use nyaaai_contract::shape::{Path, PathPart, Shape, compatible_versions};
use nyaaai_contract::shapes::frames::{
    self, AdapterFrame, Done, Failed, Fault, FaultCode, HostFrame, Reject, Request, RunEvent,
    RunFailure, Start, Welcome,
};
use nyaaai_contract::time::{Clock, SteppedClock, SystemClock};
use nyaaai_contract::version::CONTRACT_VERSION;

/// Oversized lines stay unread.
pub const MAX_FRAME_BYTES: usize = 8 * 1024 * 1024;

pub const EXIT_REJECTED: u8 = 2;

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

pub type Commands = fn(&str, &Value, &RunContext) -> Option<Result<Value, RunError>>;

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

pub struct RunContext {
    run: String,
    pub clock: Box<dyn Clock + Send>,
    pub ids: Box<dyn Ids + Send>,
    out: Arc<Out>,
    inbox: Receiver<Inbound>,
    /// Pulled, not yet consumed.
    queued: RefCell<VecDeque<Inbound>>,
    pending: Pending,
    seq: Cell<u64>,
    asked: Cell<u64>,
    interrupted: Cell<bool>,
}

impl RunContext {
    fn next_seq(&self) -> u64 {
        self.seq.set(self.seq.get() + 1);
        self.seq.get()
    }

    #[cfg_attr(not(feature = "cases"), expect(dead_code))]
    pub fn emit(&self, event: Value) {
        self.out.frame(&AdapterFrame::Event(RunEvent {
            run: self.run.clone(),
            seq: self.next_seq(),
            event,
        }));
    }

    /// Returns host request id.
    pub fn ask(&self, kind: &str, input: Value) -> String {
        self.asked.set(self.asked.get() + 1);
        let id = format!("q{}", self.asked.get());
        locked(&self.pending).insert(id.clone());
        self.out.frame(&AdapterFrame::Request(Request {
            run: self.run.clone(),
            id: id.clone(),
            kind: kind.to_owned(),
            input,
        }));
        id
    }

    /// Nothing follows an interrupt.
    fn pull(&self) -> Option<Inbound> {
        if self.interrupted.get() {
            return None;
        }
        match self.inbox.recv() {
            Ok(Inbound::Interrupt) | Err(_) => {
                self.interrupted.set(true);
                None
            }
            Ok(held) => Some(held),
        }
    }

    /// Interrupts repeat forever.
    #[cfg_attr(not(feature = "cases"), expect(dead_code))]
    pub fn receive(&self) -> Inbound {
        let queued = self.queued.borrow_mut().pop_front();
        queued.or_else(|| self.pull()).unwrap_or(Inbound::Interrupt)
    }

    /// Other messages remain queued.
    pub fn reply(&self, asked: &str) -> Result<Value, RunError> {
        let at = self
            .queued
            .borrow()
            .iter()
            .position(|held| held.answers(asked));
        let mut found = at.and_then(|at| self.queued.borrow_mut().remove(at));
        while found.is_none() {
            let held = self.pull().ok_or(RunError::Interrupted)?;
            if held.answers(asked) {
                found = Some(held);
            } else {
                self.queued.borrow_mut().push_back(held);
            }
        }
        match found {
            Some(Inbound::Reply { value, .. }) => Ok(value),
            Some(Inbound::ReplyFailed { message, .. }) => Err(RunError::HostRequest(message)),
            _ => Err(RunError::Interrupted),
        }
    }

    /// Interrupts fail the request.
    pub fn request(&self, kind: &str, input: Value) -> Result<Value, RunError> {
        self.reply(&self.ask(kind, input))
    }

    #[cfg_attr(not(feature = "cases"), expect(dead_code))]
    pub fn steers(&self) -> Vec<String> {
        let mut queued = self.queued.borrow_mut();
        while !self.interrupted.get() {
            match self.inbox.try_recv() {
                Ok(Inbound::Interrupt) | Err(TryRecvError::Disconnected) => {
                    self.interrupted.set(true);
                }
                Ok(held) => queued.push_back(held),
                Err(TryRecvError::Empty) => break,
            }
        }
        let mut steers = Vec::new();
        queued.retain(|held| match held {
            Inbound::Steer(text) => {
                steers.push(text.clone());
                false
            }
            _ => true,
        });
        steers
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
    inbox: Sender<Inbound>,
    pending: Pending,
}

type Runs = Arc<Mutex<HashMap<String, RunHandle>>>;

fn failed(context: &RunContext, error: RunError) -> AdapterFrame {
    let (code, path, message) = match error {
        RunError::Interrupted => (RunFailure::Interrupted, None, "interrupted".to_owned()),
        RunError::BadInput(path) => (
            RunFailure::BadInput,
            Some(frame_path(path)),
            "the input does not fit the command".to_owned(),
        ),
        RunError::HostRequest(message) => (RunFailure::HostRequest, None, message),
        RunError::UnknownCommand(message) => (RunFailure::UnknownCommand, None, message),
        RunError::Failed(message) => (RunFailure::Error, None, message),
    };
    AdapterFrame::Failed(Failed {
        run: context.run.clone(),
        seq: context.next_seq(),
        code,
        path,
        message,
    })
}

fn start(frame: Start, commands: Commands, out: &Arc<Out>, runs: &Runs) -> JoinHandle<()> {
    let (inbox, receiver) = mpsc::channel();
    let pending = Pending::default();
    let (clock, ids): (Box<dyn Clock + Send>, Box<dyn Ids + Send>) = match frame.fixed {
        Some(fixed) => (
            Box::new(SteppedClock::new(fixed.now_ms, fixed.step_ms as i64)),
            Box::new(CountedIds::new(fixed.id_prefix)),
        ),
        None => (Box::new(SystemClock), Box::new(RandomIds)),
    };
    let context = RunContext {
        run: frame.run.clone(),
        clock,
        ids,
        out: Arc::clone(out),
        inbox: receiver,
        queued: RefCell::default(),
        pending: Arc::clone(&pending),
        seq: Cell::new(0),
        asked: Cell::new(0),
        interrupted: Cell::new(false),
    };
    locked(runs).insert(frame.run.clone(), RunHandle { inbox, pending });
    let runs = Arc::clone(runs);
    thread::spawn(move || {
        let worked = panic::catch_unwind(AssertUnwindSafe(|| {
            commands(&frame.command, &frame.input, &context).unwrap_or_else(|| {
                Err(RunError::UnknownCommand(format!(
                    "no command {}",
                    frame.command
                )))
            })
        }));
        let terminal = match worked {
            Ok(Ok(result)) => AdapterFrame::Done(Done {
                run: context.run.clone(),
                seq: context.next_seq(),
                result,
            }),
            Ok(Err(error)) => failed(&context, error),
            Err(_) => failed(
                &context,
                RunError::Failed("the command panicked".to_owned()),
            ),
        };
        locked(&runs).remove(&context.run);
        context.out.frame(&terminal);
    })
}

/// Serves until EOF.
pub fn serve(input: impl Read, out: Box<dyn Write + Send>, commands: Commands) -> u8 {
    let out = Arc::new(Out(Mutex::new(out)));
    let runs = Runs::default();
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    let mut reader = FrameReader::new(input);
    let mut greeted = false;

    while let Some(line) = reader.next() {
        let bytes = match line {
            Line::Oversized => {
                let message = "the frame is over the size cap";
                out.fault(FaultCode::OversizedFrame, message, None, None);
                continue;
            }
            Line::Bytes(bytes) if bytes.is_empty() => continue,
            Line::Bytes(bytes) => bytes,
        };
        let Some(value) = parse_json(&String::from_utf8_lossy(&bytes)) else {
            out.fault(FaultCode::BadFrame, "not JSON", None, None);
            continue;
        };
        let frame = match HostFrame::decode(&value) {
            Ok(frame) => frame,
            Err(refused) => {
                let message = "not a host frame";
                out.fault(FaultCode::BadFrame, message, None, Some(refused.path));
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
                        reason: "another contract version".to_owned(),
                    }));
                    return EXIT_REJECTED;
                }
                _ => out.fault(FaultCode::NotGreeted, "say hello first", None, None),
            }
            continue;
        }
        let (run, inbound) = match frame {
            HostFrame::Hello(_) => {
                out.fault(FaultCode::BadFrame, "greeted twice", None, None);
                continue;
            }
            HostFrame::Start(frame) => {
                if locked(&runs).contains_key(&frame.run) {
                    let message = "the run is open";
                    out.fault(FaultCode::DuplicateRun, message, Some(&frame.run), None);
                } else {
                    threads.retain(|thread| !thread.is_finished());
                    threads.push(start(frame, commands, &out, &runs));
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
            out.fault(FaultCode::UnknownRun, "no such run", Some(&run), None);
            continue;
        };
        let answered = match &inbound {
            Inbound::Reply { id, .. } | Inbound::ReplyFailed { id, .. } => {
                locked(&handle.pending).remove(id)
            }
            _ => true,
        };
        if answered {
            let _ = handle.inbox.send(inbound);
        } else {
            out.fault(FaultCode::UnknownReply, "no such request", Some(&run), None);
        }
    }

    for handle in locked(&runs).values() {
        let _ = handle.inbox.send(Inbound::Interrupt);
    }
    for thread in threads {
        let _ = thread.join();
    }
    0
}
