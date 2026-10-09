//! Every shape the protocol carries (PROTOCOL.md §3–§5). Times are milliseconds since 1970.

pub mod harness;
pub mod process;
pub mod sandbox;

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The runtime's first request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
pub struct Runtime {
    /// Its release: `6.9.6`.
    pub release: String,
    /// The operating system it runs on, as Rust names it: `linux`, `macos`.
    pub os: String,
    /// The processor architecture, as Rust names it: `x86_64`, `aarch64`.
    pub arch: String,
}

/// What a program is: the answer to `initialize`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Initialized {
    /// The one protocol version the program speaks, from those offered.
    pub protocol: u32,
    /// Who the program is.
    pub program: ProgramInfo,
    /// A driver's levels; each needs the one before it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub levels: Vec<Level>,
    /// A driver's harness, as a person meets it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub harness: Option<HarnessInfo>,
    /// What an adapter provides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provides: Option<Provides>,
    /// The facts the runtime reads: [`HarnessFacts`] for a driver, [`ProcessFacts`] or
    /// [`SandboxFacts`] for an adapter, by what it provides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub facts: Option<Value>,
    /// The settings the program needs a person to give once.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub settings: Vec<Setting>,
}

impl Initialized {
    /// `facts`, read as a driver's.
    pub fn harness_facts(&self) -> Result<HarnessFacts, serde_json::Error> {
        HarnessFacts::deserialize(self.facts.as_ref().unwrap_or(&Value::Null))
    }

    /// `facts`, read as an operating system's.
    pub fn process_facts(&self) -> Result<ProcessFacts, serde_json::Error> {
        ProcessFacts::deserialize(self.facts.as_ref().unwrap_or(&Value::Null))
    }

    /// `facts`, read as a sandbox provider's.
    pub fn sandbox_facts(&self) -> Result<SandboxFacts, serde_json::Error> {
        SandboxFacts::deserialize(self.facts.as_ref().unwrap_or(&Value::Null))
    }
}

/// Who a program is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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
    /// It speaks to a running conversation (§4.3).
    Live,
}

impl Level {
    /// Every level, each after the one it needs.
    pub const ALL: [Level; 3] = [Level::Observe, Level::Drive, Level::Live];
}

/// What an adapter provides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Provides {
    /// An operating system: its processes and its service manager (§5.1).
    Processes,
    /// A sandbox provider (§5.2).
    Sandboxes,
}

/// A driver's harness, as a person meets it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HarnessInfo {
    /// The oldest release the driver speaks to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<Minimum>,
    /// The action that signs a person in: `run /login`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sign_in: Option<String>,
    /// How an agent in this harness learns its conversation's id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub knowing: Option<String>,
    /// One line on how its transcript file reads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<String>,
}

/// The oldest release of a harness a driver speaks to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Minimum {
    /// The release.
    pub release: String,
    /// The command that updates the harness.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub update: Option<String>,
}

/// A setting a program needs a person to give once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

/// A driver's facts: what the runtime's holder rule and offers read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HarnessFacts {
    /// A conversation is continued by its transcript path, not its id.
    pub by_file: bool,
    /// A resume refuses where the conversation's folder is gone.
    pub resumes_in_place: bool,
    /// A fork may run beside a live writer.
    pub forks_beside: bool,
    /// What a resume keeps without being told.
    pub keeps: Keeps,
    /// How a live conversation is reached; absent: there is no live door.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub door: Option<harness::LiveDoor>,
    /// The door can say the conversation is idle.
    pub checks_idle: bool,
    /// The acts the harness does at all.
    pub acts: Vec<harness::Act>,
}

/// What a resume keeps without being told.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Keeps {
    /// The model the conversation ran on.
    pub model: bool,
    /// How hard it thought.
    pub thinking: bool,
}

/// An operating system's facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ProcessFacts {
    /// `macos` or `linux`.
    pub os: String,
    /// The service manager that keeps the runtime running: `launchd` or `systemd`.
    pub service: String,
}

/// A sandbox provider's facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SandboxFacts {
    /// What survives `sandbox/stop`.
    pub keeps: Survives,
    /// The only folder that survives, where `keeps` is `volume`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persist: Option<PathBuf>,
    /// The provider's own idle timer.
    pub idle: Idle,
    /// The optional methods it has, by their last word: `archive`, `pause`, `fork`,
    /// `keepalive`.
    #[serde(default)]
    pub optional: Vec<String>,
    /// How `sandbox/channel` reaches a sandbox.
    pub channel: Channel,
    /// The operating system inside its sandboxes.
    pub os: String,
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

/// The empty object: the parameters of a method that takes none, and the result of one
/// that says nothing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Empty {}
