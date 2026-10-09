//! The host protocol (PROTOCOL.md §6): control's channel to `steery sandbox serve`, a remote
//! host. Control sends requests; `serve` answers them and sends notifications.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Initialized, Runtime};

/// The most bytes of one connection sent and not yet credited.
pub const WINDOW: u64 = 256 << 10;
/// The most bytes one `host/data` carries.
pub const CHUNK: usize = 64 << 10;
/// The most bytes one `host/fs/read` answers.
pub const READ_MAX: u64 = 4 << 20;

/// `host/hello`: the channel's first request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "HostHello")
)]
#[serde(deny_unknown_fields)]
pub struct Hello {
    /// The protocol versions control speaks.
    pub protocol: Vec<u32>,
    /// Control's runtime.
    pub runtime: Runtime,
}

/// What `serve` is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "HostWelcome")
)]
#[serde(deny_unknown_fields)]
pub struct Welcome {
    /// The one protocol version it speaks, from those offered.
    pub protocol: u32,
    /// Its release.
    pub release: String,
    /// The operating system it runs on, as Rust names it.
    pub os: String,
    /// The processor architecture, as Rust names it.
    pub arch: String,
    /// The host's home folder.
    pub home: PathBuf,
    /// The host's Steery folder.
    pub steery: PathBuf,
    /// The port of its loopback listener.
    pub port: u16,
}

/// `host/fs/stat`: what is at each path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FsStat {
    pub paths: Vec<PathBuf>,
}

/// What `host/fs/stat` found, in order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FsStats {
    /// `null` where nothing is.
    pub stats: Vec<Option<Stat>>,
}

/// What is at a path. A link is not followed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Stat {
    pub kind: FileKind,
    pub size: u64,
    pub mtime: u64,
    /// Its Unix mode bits.
    pub mode: u32,
    pub dev: u64,
    pub ino: u64,
}

/// What a path is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    File,
    Dir,
    Link,
    /// Anything else: a fifo, a socket, a device.
    Other,
}

/// `host/fs/read`: bytes of a file, within its size as read once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FsRead {
    pub path: PathBuf,
    /// From here; absent: 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset: Option<u64>,
    /// At most this many bytes; absent: to the end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length: Option<u64>,
    /// Only the last this many bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tail: Option<u64>,
}

/// The bytes `host/fs/read` read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FsBytes {
    /// Base64.
    pub bytes: String,
    /// The file's size as read.
    pub size: u64,
    /// The file's modification time as read.
    pub mtime: u64,
}

/// `host/fs/list`: what a folder holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FsList {
    pub path: PathBuf,
}

/// What `host/fs/list` found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FsEntries {
    pub entries: Vec<FsEntry>,
}

/// One thing in a folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FsEntry {
    pub name: String,
    pub kind: FileKind,
}

/// `host/fs/write` (replace atomically) and `host/fs/append`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FsWrite {
    pub path: PathBuf,
    /// Base64.
    pub bytes: String,
    /// The mode of a file made.
    pub mode: u32,
}

/// `host/fs/mkdir`: a folder and its parents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FsMkdir {
    pub path: PathBuf,
    pub mode: u32,
}

/// `host/fs/remove`: a file, or a folder with what it holds where `recursive`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct FsRemove {
    pub path: PathBuf,
    pub recursive: bool,
}

/// What `host/programs` answers: the host's programs, as each answered `initialize`, and
/// why each that does not run cannot be started.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Programs {
    /// Its operating-system adapter.
    pub os: Initialized,
    /// Its drivers that run.
    pub drivers: Vec<Initialized>,
    /// Its drivers that cannot be started, each with why.
    pub missing: Vec<Missing>,
}

/// A program of the host that cannot be started.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Missing {
    /// `os`, or a driver's id.
    pub program: String,
    /// Why, in one line.
    pub line: String,
}

/// `host/call`: a program's method, on the host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "HostCall")
)]
#[serde(deny_unknown_fields)]
pub struct Call {
    /// `os`, or a driver's id.
    pub program: String,
    pub method: String,
    pub params: Value,
}

/// The program `os` in `host/call`.
pub const OS: &str = "os";

/// `host/notified`: a program's notification, from the host.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Notified {
    pub program: String,
    pub method: String,
    pub params: Value,
}

/// `host/exited`: a program of the host exited; it starts again on its next call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Exited {
    /// `os`, or a driver's id.
    pub program: String,
}

/// `host/opened`: the host's listener took a connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Opened {
    /// `h<n>`.
    pub conn: String,
}

/// `host/dial`: a connection on the host, to a loopback port or a unix socket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Dial {
    /// `c<n>`.
    pub conn: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub socket: Option<PathBuf>,
}

/// `host/data`: a connection's bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "HostData")
)]
#[serde(deny_unknown_fields)]
pub struct Data {
    pub conn: String,
    /// Base64, at most [`CHUNK`] bytes.
    pub bytes: String,
}

/// `host/credit`: bytes of a connection passed on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Credit {
    pub conn: String,
    pub credit: u64,
}

/// `host/close`: a connection's side ends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "HostClose")
)]
#[serde(deny_unknown_fields)]
pub struct Close {
    pub conn: String,
    pub how: CloseHow,
}

/// How a connection's side ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CloseHow {
    /// That side's writing ends; the other may still send.
    Write,
    /// The connection ends at once both ways.
    Reset,
}
