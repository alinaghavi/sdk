//! An operating system's methods (PROTOCOL.md §5.1). A process is named by its pid and its
//! `process_start`: the start time the kernel gives it.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Two readings of one process's start time this close are the same start.
pub const SAME_START_MS: u64 = 10_000;

/// `process/start`: start a process detached, in a session and process group of its own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "ProcessStart")
)]
#[serde(deny_unknown_fields)]
pub struct Start {
    /// The program and its arguments.
    pub argv: Vec<String>,
    /// Its whole environment.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// The folder it runs in.
    pub cwd: PathBuf,
    /// The file its standard input reads; absent: none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdin: Option<PathBuf>,
    /// The file its standard output is appended to.
    pub stdout: PathBuf,
    /// The file its standard error is appended to.
    pub stderr: PathBuf,
    /// The service (its label) whose share of the CPU it runs in, where there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// The process `process/start` started.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Started {
    /// Its pid.
    pub pid: u32,
    /// Its start time, in milliseconds since 1970.
    pub process_start: u64,
}

/// `process/list`: the processes running.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "ProcessList")
)]
#[serde(deny_unknown_fields)]
pub struct List {
    /// Only these; absent: every process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pids: Option<Vec<u32>>,
}

/// The processes `process/list` found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "ProcessListed")
)]
#[serde(deny_unknown_fields)]
pub struct Listed {
    /// Each one running, no zombie; a pid asked for that runs nothing is absent.
    pub processes: Vec<Entry>,
}

/// One running process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "ProcessEntry")
)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// Its pid.
    pub pid: u32,
    /// Its parent's pid.
    pub ppid: u32,
    /// Its start time, in milliseconds since 1970.
    pub process_start: u64,
    /// Its executable's name.
    pub command: String,
    /// Its arguments, its own name first; absent where they cannot be read, and it may
    /// then be any program.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// The user it runs as.
    pub user: String,
}

/// `process/environ`: some of a process's environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Environ {
    /// Its pid.
    pub pid: u32,
    /// Its start time, which the pid must still have.
    pub process_start: u64,
    /// The variables read.
    pub names: Vec<String>,
}

/// The variables `process/environ` read.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Values {
    /// Each one set, by name; one not set is absent.
    pub values: BTreeMap<String, String>,
}

/// `process/files`: the files a process holds open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Files {
    /// Its pid.
    pub pid: u32,
    /// Its start time, which the pid must still have.
    pub process_start: u64,
}

/// The files `process/files` found open.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Paths {
    /// Each one's absolute path.
    pub paths: Vec<PathBuf>,
}

/// `process/listening`: the processes listening on a TCP port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Listening {
    pub port: u16,
}

/// The processes `process/listening` found.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Listeners {
    pub processes: Vec<Listener>,
}

/// A process listening on a port.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Listener {
    pub pid: u32,
    pub process_start: u64,
    /// Its executable's name.
    pub command: String,
}

/// `process/signal`: signal a process, only where the pid still has that start time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Signal {
    /// Its pid.
    pub pid: u32,
    /// Its start time, which the pid must still have.
    pub process_start: u64,
    /// The signal.
    pub signal: SignalName,
    /// Signal its whole process group.
    pub group: bool,
}

/// A signal the runtime sends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SignalName {
    Int,
    Term,
    Kill,
    Cont,
}

/// Whether `process/signal` sent the signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Sent {
    /// False where the pid no longer has that start time.
    pub sent: bool,
}

/// `service/plan`: what installs the runtime as a service. It is pure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ServicePlan {
    /// The service's name.
    pub label: String,
    /// The runtime's binary.
    pub binary: PathBuf,
    /// Its arguments.
    #[serde(default)]
    pub args: Vec<String>,
    /// Its environment.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// The loopback port whose listener the service manager hands it.
    pub port: u16,
    /// Its share, 1 to 100 percent, of what the computer grants its user.
    pub cpu: u8,
    /// The file its own output is appended to.
    pub log: PathBuf,
}

/// What installs the runtime as a service; the runtime writes and runs it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Plan {
    /// The files to write.
    pub files: Vec<PlannedFile>,
    /// The commands to run after, in order; the first failure of one that may
    /// not fail stops the rest.
    pub commands: Vec<PlanCommand>,
}

/// One command of a [`Plan`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct PlanCommand {
    /// The command and its arguments.
    pub argv: Vec<String>,
    /// Its failure does not stop the plan.
    #[serde(default)]
    pub may_fail: bool,
}

/// One file of a [`Plan`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct PlannedFile {
    /// Its absolute path.
    pub path: PathBuf,
    /// Its whole content.
    pub content: String,
    /// Its Unix mode bits.
    pub mode: u32,
}

/// `service/status`, `service/start`, `service/restart`, `service/remove`: one service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Service {
    /// The service's name.
    pub label: String,
}

/// Where a service stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Status {
    /// Its unit is on disk.
    pub installed: bool,
    /// Its manager runs it.
    pub running: bool,
    /// The process it runs as.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_start: Option<u64>,
    /// The percent of one core the computer grants the runtime's user.
    pub granted: u32,
}
