//! The runtime's side: start a program, speak to it, and end it (PROTOCOL.md §2).

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::{mpsc, oneshot};

use crate::{INVALID_CODE, REFUSED_CODE, UNKNOWN_CODE, frame, method, wire};

/// How long a program has after `shutdown` and its closed standard input before SIGTERM,
/// and after SIGTERM before it is killed.
const GRACE: Duration = Duration::from_secs(5);

/// A notification a program sent.
#[derive(Debug, Clone, PartialEq)]
pub struct Notification {
    pub method: String,
    pub params: Value,
}

/// Why a call has no result.
#[derive(Debug, thiserror::Error)]
pub enum CallError {
    /// The program refused: `word` as it gave it, `line` its one line for a person.
    #[error("refused: {word}")]
    Refused { word: String, line: Option<String> },
    /// The program has no such method.
    #[error("unknown method")]
    Unknown,
    /// No answer within the method's bound.
    #[error("no answer in time")]
    Timeout,
    /// The program's standard input or output closed.
    #[error("the program closed")]
    Closed,
    /// The program read the parameters as invalid, or answered something that does not
    /// read as the method's result.
    #[error("invalid: {0}")]
    Invalid(String),
    /// Any other JSON-RPC error.
    #[error("error {code}: {message}")]
    Remote { code: i64, message: String },
}

/// One caller waiting for an id's answer.
struct Waiter {
    serial: u64,
    answer: oneshot::Sender<Result<Value, CallError>>,
}

/// Every id still owed, by id; `None` once the program's output has ended.
type Pending = Arc<Mutex<Option<HashMap<String, Vec<Waiter>>>>>;

/// A running program.
pub struct Client {
    child: Child,
    stdin: tokio::sync::Mutex<Option<ChildStdin>>,
    pending: Pending,
    notifications: Mutex<Option<mpsc::UnboundedReceiver<Notification>>>,
    next: AtomicU64,
    reader: tokio::task::JoinHandle<()>,
}

impl Client {
    /// Start `command` with `args`. Its standard error is appended to `stderr_log`, or
    /// is this process's own where there is none.
    pub async fn spawn(
        command: &Path,
        args: &[String],
        stderr_log: Option<&Path>,
    ) -> std::io::Result<Self> {
        let stderr = match stderr_log {
            Some(path) => Stdio::from(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?,
            ),
            None => Stdio::inherit(),
        };
        let mut child = tokio::process::Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr)
            .kill_on_drop(true)
            .spawn()?;
        let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
            return Err(std::io::Error::other("the program's stdio is not piped"));
        };
        let pending: Pending = Arc::new(Mutex::new(Some(HashMap::new())));
        let (notify, notifications) = mpsc::unbounded_channel();
        let reader = tokio::spawn(read(stdout, pending.clone(), notify));
        Ok(Self {
            child,
            stdin: tokio::sync::Mutex::new(Some(stdin)),
            pending,
            notifications: Mutex::new(Some(notifications)),
            next: AtomicU64::new(1),
            reader,
        })
    }

    /// Send `initialize`, and read what the program is. A program that names a protocol
    /// version `params` did not offer, or breaks a rule [`wire::Initialized::check`]
    /// reads, is [`CallError::Invalid`].
    pub async fn initialize(
        &self,
        params: &wire::InitializeParams,
    ) -> Result<wire::Initialized, CallError> {
        let initialized = self.ask::<method::Initialize>(params).await?;
        if !params.protocol.contains(&initialized.protocol) {
            return Err(CallError::Invalid(format!(
                "protocol {} was not offered",
                initialized.protocol
            )));
        }
        initialized.check().map_err(CallError::Invalid)?;
        Ok(initialized)
    }

    /// Ask the request `M` with `params`, waiting at most its bound (PROTOCOL.md §9), and
    /// read its answer as its result.
    pub async fn ask<M: method::Method>(&self, params: &M::Params) -> Result<M::Result, CallError> {
        let params = serde_json::to_value(params).map_err(|e| CallError::Invalid(e.to_string()))?;
        let answer = self.call(M::NAME, params, M::BOUND).await?;
        serde_json::from_value(answer).map_err(|e| CallError::Invalid(e.to_string()))
    }

    /// Call `method` under a new id, waiting at most `within`.
    pub async fn call(
        &self,
        method: &str,
        params: Value,
        within: Duration,
    ) -> Result<Value, CallError> {
        let id = self.next.fetch_add(1, Ordering::Relaxed).to_string();
        self.call_as(&id, method, params, within).await
    }

    /// Call `method` under `id`, waiting at most `within`. The same id sent again with the
    /// same method and parameters is answered from its first result.
    pub async fn call_as(
        &self,
        id: &str,
        method: &str,
        params: Value,
        within: Duration,
    ) -> Result<Value, CallError> {
        let serial = self.next.fetch_add(1, Ordering::Relaxed);
        let (answer, answered) = oneshot::channel();
        {
            let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
            let Some(pending) = pending.as_mut() else {
                return Err(CallError::Closed);
            };
            pending
                .entry(id.to_owned())
                .or_default()
                .push(Waiter { serial, answer });
        }
        let line = frame::encode(
            &json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }),
        );
        if self.write(&line).await.is_err() {
            self.forget(id, serial);
            return Err(CallError::Closed);
        }
        match tokio::time::timeout(within, answered).await {
            Ok(Ok(answer)) => answer,
            Ok(Err(_)) => Err(CallError::Closed),
            Err(_) => {
                // A late answer is dropped.
                self.forget(id, serial);
                Err(CallError::Timeout)
            }
        }
    }

    /// The program's notifications, in the order it sent them. `Some` the first time only.
    pub fn notifications(&self) -> Option<mpsc::UnboundedReceiver<Notification>> {
        self.notifications
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
    }

    /// Send `shutdown`, wait for its answer, close standard input, and send SIGTERM to a
    /// program still running 5 s later; one still running 5 s after that is killed.
    pub async fn shutdown(mut self) -> std::io::Result<std::process::ExitStatus> {
        let _ = self.ask::<method::Shutdown>(&wire::Empty {}).await;
        drop(self.stdin.lock().await.take());
        if let Ok(status) = tokio::time::timeout(GRACE, self.child.wait()).await {
            self.reader.abort();
            return status;
        }
        #[cfg(unix)]
        if let Some(pid) = self.child.id().and_then(|pid| i32::try_from(pid).ok()) {
            // SAFETY: a signal to the child this client started and has not reaped.
            unsafe { libc::kill(pid, libc::SIGTERM) };
            if let Ok(status) = tokio::time::timeout(GRACE, self.child.wait()).await {
                self.reader.abort();
                return status;
            }
        }
        self.child.start_kill()?;
        let status = self.child.wait().await;
        self.reader.abort();
        status
    }

    async fn write(&self, line: &[u8]) -> std::io::Result<()> {
        let mut stdin = self.stdin.lock().await;
        let Some(stdin) = stdin.as_mut() else {
            return Err(std::io::ErrorKind::BrokenPipe.into());
        };
        stdin.write_all(line).await?;
        stdin.flush().await
    }

    fn forget(&self, id: &str, serial: u64) {
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(pending) = pending.as_mut()
            && let Some(waiters) = pending.get_mut(id)
        {
            waiters.retain(|waiter| waiter.serial != serial);
            if waiters.is_empty() {
                pending.remove(id);
            }
        }
    }
}

/// Read the program's output until it ends: answers to their waiters, notifications to
/// their channel. A line that is no JSON, or over 16 MiB, is skipped.
async fn read(stdout: ChildStdout, pending: Pending, notify: mpsc::UnboundedSender<Notification>) {
    let mut stdout = BufReader::new(stdout);
    let mut line = Vec::new();
    while let Ok(Some(read)) = frame::next(&mut stdout, &mut line).await {
        if !matches!(read, frame::Read::Line) {
            continue;
        }
        let Ok(message) = serde_json::from_slice::<Value>(&line) else {
            continue;
        };
        let id = message.get("id").filter(|id| !id.is_null());
        if let Some(method) = message.get("method").and_then(Value::as_str) {
            // A program sends no requests in version 1.
            if id.is_none() {
                let _ = notify.send(Notification {
                    method: method.to_owned(),
                    params: message.get("params").cloned().unwrap_or(Value::Null),
                });
            }
            continue;
        }
        let Some(id) = id else {
            continue;
        };
        let key = match id {
            Value::String(id) => id.clone(),
            other => other.to_string(),
        };
        let waiters = {
            let mut pending = pending.lock().unwrap_or_else(|e| e.into_inner());
            pending.as_mut().and_then(|pending| pending.remove(&key))
        };
        for waiter in waiters.into_iter().flatten() {
            let _ = waiter.answer.send(answer(&message));
        }
    }
    // Every waiter's sender is dropped with the map: each reads as closed.
    pending.lock().unwrap_or_else(|e| e.into_inner()).take();
}

/// What one answer says.
fn answer(message: &Value) -> Result<Value, CallError> {
    let Some(error) = message.get("error") else {
        return Ok(message.get("result").cloned().unwrap_or(Value::Null));
    };
    let code = error.get("code").and_then(Value::as_i64).unwrap_or(0);
    let text = |value: Option<&Value>| value.and_then(Value::as_str).map(str::to_owned);
    let message = text(error.get("message")).unwrap_or_default();
    Err(match code {
        REFUSED_CODE => {
            let data = error.get("data");
            CallError::Refused {
                word: text(data.and_then(|data| data.get("refused")))
                    .unwrap_or_else(|| crate::refusal::REFUSED.to_owned()),
                line: text(data.and_then(|data| data.get("line"))),
            }
        }
        UNKNOWN_CODE => CallError::Unknown,
        INVALID_CODE => CallError::Invalid(message),
        code => CallError::Remote { code, message },
    })
}
