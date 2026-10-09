//! Every shape the protocol carries (PROTOCOL.md §3–§6). Times are milliseconds since 1970.
//! Every object rejects a field it does not define (§2).

pub mod door;
pub mod harness;
pub mod host;
pub mod process;
pub mod sandbox;

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// The runtime's first request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct InitializeParams {
    /// The protocol versions the runtime speaks.
    pub protocol: Vec<u32>,
    /// The runtime asking.
    pub runtime: Runtime,
    /// The person's home folder.
    pub home: PathBuf,
    /// The runtime's config folder.
    pub config: PathBuf,
    /// The values the person gave for the settings the program declared, and nothing else.
    #[serde(default)]
    pub settings: BTreeMap<String, String>,
}

/// The runtime a program runs under.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    /// Its release: `6.9.6`.
    pub release: String,
    /// The operating system it runs on, as Rust names it: `linux`, `macos`.
    pub os: String,
    /// The processor architecture, as Rust names it: `x86_64`, `aarch64`.
    pub arch: String,
}

/// What a program is: the answer to `initialize`. A driver answers `harness`, an adapter
/// `machine`, never both.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Initialized {
    /// The one protocol version the program speaks, from those offered.
    pub protocol: u32,
    /// Who the program is.
    pub program: ProgramInfo,
    /// A driver's harness: its levels, identity, facts and hooks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<Box<HarnessInfo>>,
    /// An adapter's machine: what it provides, and that kind's facts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine: Option<Machine>,
    /// The settings the program needs a person to give once.
    #[serde(default)]
    pub settings: Vec<Setting>,
}

impl Initialized {
    /// What the shapes cannot say themselves (PROTOCOL.md §3): the id is kebab-case, the
    /// kind answers its own object and not the other's, a driver's levels each follow the
    /// one they need and `live` comes exactly with a door, its executables are named, and
    /// its hook events are unique. The first rule broken, in one line.
    pub fn check(&self) -> Result<(), String> {
        kebab(&self.program.id)?;
        match (self.program.kind, &self.harness, &self.machine) {
            (Kind::Harness, Some(harness), None) => harness.check(),
            (Kind::Machine, None, Some(_)) => Ok(()),
            (Kind::Harness, _, _) => Err("a driver answers `harness` and no `machine`".into()),
            (Kind::Machine, _, _) => Err("an adapter answers `machine` and no `harness`".into()),
        }
    }
}

impl HarnessInfo {
    fn check(&self) -> Result<(), String> {
        let mut named = self.levels.clone();
        named.sort();
        named.dedup();
        if named.is_empty() {
            return Err("names no level".into());
        }
        if named.len() != self.levels.len() {
            return Err("names a level twice".into());
        }
        if named[..] != Level::ALL[..named.len()] {
            return Err("names a level without the one before it".into());
        }
        if named.contains(&Level::Live) != self.facts.door.is_some() {
            return Err("names `live` without a door, or a door without `live`".into());
        }
        if self.identity.executables.iter().all(String::is_empty) {
            return Err("names no executable".into());
        }
        let mut events: Vec<&str> = self.hooks.iter().map(|hook| hook.event.as_str()).collect();
        events.sort_unstable();
        if let Some(twice) = events.windows(2).find(|pair| pair[0] == pair[1]) {
            return Err(format!("names the hook event `{}` twice", twice[0]));
        }
        Ok(())
    }
}

/// Lowercase kebab-case, at most 64 characters.
fn kebab(id: &str) -> Result<(), String> {
    let kebab = id.len() <= 64
        && id.split('-').all(|word| {
            !word.is_empty()
                && word
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        });
    if kebab {
        Ok(())
    } else {
        Err(format!(
            "`{id}` is not lowercase kebab-case of at most 64 characters"
        ))
    }
}

/// Who a program is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ProgramInfo {
    /// Lowercase kebab-case, at most 64 characters: what `Session.harness` names for a
    /// driver. A found program cannot take an official one's id.
    pub id: String,
    /// Driver or adapter.
    pub kind: Kind,
    /// The program's own release.
    pub release: String,
    /// Its name as a person reads it.
    pub label: String,
    /// A shorter name, where one is read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short: Option<String>,
}

/// What a program connects Steery to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// One harness: a driver.
    Harness,
    /// One kind of machine: an adapter.
    Machine,
}

/// How far a driver goes with its harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// It reads conversations and answers hooks (§4.1).
    Observe,
    /// It starts, stops and reads turns (§4.2).
    Drive,
    /// It reaches a running conversation through its door (§4.3).
    Live,
}

impl Level {
    /// Every level, each after the one it needs.
    pub const ALL: [Level; 3] = [Level::Observe, Level::Drive, Level::Live];
}

/// A driver's harness, as `initialize` states it (§3.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct HarnessInfo {
    /// Its levels; each needs the one before it. `live` is named exactly when
    /// `facts.door` is.
    pub levels: Vec<Level>,
    /// The oldest release the driver speaks to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<Minimum>,
    /// The action that signs a person in: `run /login`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sign_in: Option<String>,
    /// How an agent in this harness learns its conversation's id.
    pub knowing: String,
    /// One line on how its transcript file reads; absent: it keeps no transcript file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
    /// How the runtime recognizes the harness's processes.
    pub identity: Identity,
    /// What setting the harness up and serving its MCP calls read.
    pub setup: SetupFacts,
    /// Its one-shot reader; absent: the driver has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reader: Option<ReaderFacts>,
    /// What the holder rule and offers read.
    pub facts: HarnessFacts,
    /// Every moment of the harness the runtime reads.
    #[serde(default)]
    pub hooks: Vec<harness::Hook>,
}

/// The oldest release of a harness a driver speaks to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Minimum {
    /// The release.
    pub release: String,
    /// The command that updates the harness.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update: Option<String>,
}

/// How the runtime recognizes a harness's processes (§3.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Identity {
    /// The file names a harness process runs as.
    pub executables: Vec<String>,
    /// A path component naming its package, where an interpreter runs it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// The variable the harness puts its conversation's id in, in its own process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_variable: Option<String>,
}

/// What setting a harness up and serving its MCP calls read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct SetupFacts {
    /// The harness reads Steery's skill from the entry `setup/plan` lists; where false,
    /// the MCP handshake carries the whole skill.
    pub reads_skills: bool,
    /// The harness cuts an MCP tool's description before its model reads it, so the
    /// escalation policies are said whole at `session_start`.
    pub cuts_tool_text: bool,
    /// The longest the harness waits for one MCP call, in seconds, where no entry can
    /// lengthen it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_bound: Option<u32>,
}

/// A driver's one-shot reader.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ReaderFacts {
    /// What the reader shuts out.
    pub sealed: Sealed,
    /// Models the reader runs that `models` may not list.
    #[serde(default)]
    pub known: Vec<String>,
    /// The reader runs the harness's own model when given none.
    pub unnamed: bool,
}

/// What a reader shuts out (true) or still reaches (false).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Sealed {
    /// Its tools.
    pub tools: bool,
    /// Its MCP servers.
    pub mcp: bool,
    /// Rules and instructions files.
    pub instructions: bool,
    /// Extensions and plugins.
    pub extensions: bool,
    /// Its saved history.
    pub history: bool,
}

/// A driver's facts: what the runtime's holder rule and offers read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct HarnessFacts {
    /// A conversation is continued by its transcript path, not its id.
    pub by_file: bool,
    /// A continue runs only in the folder the conversation ran in.
    pub resumes_in_place: bool,
    /// A fork may run beside a live writer.
    pub forks_beside: bool,
    /// What a continue keeps without being told.
    pub keeps: Keeps,
    /// How a live conversation is reached; absent: there is no live door.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub door: Option<harness::LiveDoor>,
    /// The door checks the conversation idle itself as it takes words.
    pub checks_idle: bool,
    /// `start` makes a new worktree for a fork or a new conversation when asked.
    pub worktrees: bool,
    /// The acts the harness does at all.
    pub acts: Vec<harness::Act>,
}

/// What a continue keeps without being told.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Keeps {
    /// The model the conversation ran on.
    pub model: bool,
    /// How hard it thought.
    pub thinking: bool,
}

/// An adapter's machine: what it provides, and that kind's facts (§3.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "provides", rename_all = "snake_case")]
pub enum Machine {
    /// An operating system: its processes and its service manager (§5.1).
    Processes(ProcessFacts),
    /// A sandbox provider (§5.2).
    Sandboxes(SandboxFacts),
}

/// An operating system's facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct ProcessFacts {
    /// The operating system.
    pub os: Os,
    /// The service manager that keeps the runtime running.
    pub service: ServiceManager,
}

/// An operating system an adapter provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Os {
    Macos,
    Linux,
}

/// A service manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ServiceManager {
    Launchd,
    Systemd,
}

/// A sandbox provider's facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct SandboxFacts {
    /// What survives `sandbox/stop`.
    pub keeps: Survives,
    /// The only folder that survives, where `keeps` is `volume`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persist: Option<PathBuf>,
    /// The provider's own idle timer.
    pub idle: Idle,
    /// The optional methods it has, by their last word.
    #[serde(default)]
    pub optional: Vec<Optional>,
    /// How `sandbox/channel` reaches a sandbox.
    pub channel: Channel,
    /// The operating system inside its sandboxes.
    pub os: Os,
}

/// What survives a sandbox's stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Survives {
    /// The whole filesystem.
    Filesystem,
    /// The filesystem and the memory.
    Memory,
    /// Only the folder in `persist`.
    Volume,
}

/// A sandbox provider's own idle timer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Idle {
    /// It stops a sandbox after a time without activity.
    Activity,
    /// It stops a sandbox a fixed time after it started.
    Ttl,
    /// It has none.
    None,
}

/// An optional method of a sandbox provider, by its last word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Optional {
    Archive,
    Pause,
    Fork,
    Keepalive,
}

/// How `sandbox/channel` reaches a sandbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    /// A local command whose standard input and output reach the sandbox (§5.2).
    Argv,
    /// Bytes the adapter carries itself (§5.3).
    Stream,
}

/// A setting a program needs a person to give once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Setting {
    /// Its name, as `initialize` passes it.
    pub name: String,
    /// Kept in the runtime's private config folder and never shown again.
    #[serde(default)]
    pub secret: bool,
    /// The program does not run without it.
    #[serde(default)]
    pub required: bool,
    /// One line for a person: what it is and where to find it.
    pub about: String,
}

/// The empty object: the parameters of a method that takes none, and the result of one
/// that says nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Empty {}
