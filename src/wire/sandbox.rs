//! A sandbox provider's methods (PROTOCOL.md §5.2) and its `stream` channel (§5.3). A
//! sandbox is named by the provider's own id for it.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// `sandboxes/list`: the provider's sandboxes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "SandboxList")
)]
#[serde(deny_unknown_fields)]
pub struct List {
    /// Only those carrying every one of these labels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<BTreeMap<String, String>>,
}

/// The sandboxes `sandboxes/list` found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "SandboxListed")
)]
#[serde(deny_unknown_fields)]
pub struct Listed {
    /// Each one.
    pub sandboxes: Vec<Sandbox>,
}

/// One sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Sandbox {
    /// The provider's id for it.
    pub sandbox: String,
    /// Its state, in the provider's own word.
    pub state: String,
    /// Its labels.
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    /// When it was made.
    pub created_at: u64,
}

/// `sandbox/ensure`: the sandbox for `key`, made where none is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Ensure {
    /// What finds it again.
    pub key: String,
    /// What it is made from; absent: the provider's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// Its labels.
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
}

/// What a sandbox is made from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
    /// A provider's snapshot, by name.
    Snapshot(String),
    /// A container image, by reference.
    Image(String),
}

/// A sandbox, as `sandbox/ensure` and `sandbox/fork` answer it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Named {
    /// The provider's id for it.
    pub sandbox: String,
}

/// The parameters of `sandbox/start`, `stop`, `delete`, `archive`, `pause`, `fork` and
/// `keepalive`.
pub type Target = Named;

/// A sandbox's state after `sandbox/start`, `stop`, `delete`, `archive` or `pause`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct State {
    /// In the provider's own word.
    pub state: String,
}

/// `sandbox/put`: write a file into a sandbox. Its result is [`super::Empty`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Put {
    /// The provider's id for the sandbox.
    pub sandbox: String,
    /// The absolute file inside it.
    pub path: PathBuf,
    /// Its whole content, base64.
    pub content: String,
    /// Its Unix mode bits.
    pub mode: u32,
}

/// `sandbox/channel`: a local way to `command`'s standard input and output in a sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "SandboxChannel")
)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    /// The provider's id for the sandbox.
    pub sandbox: String,
    /// The command inside it, its program first.
    pub command: Vec<String>,
}

/// How a channel reaches its command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
pub enum Reach {
    /// A local command whose standard input and output are the channel.
    Argv {
        argv: Vec<String>,
        /// The variables it adds to the environment.
        #[serde(default)]
        env: BTreeMap<String, String>,
    },
    /// Bytes the adapter carries itself, as `stream/*` messages under this id.
    Stream { stream: String },
}

/// `stream/write`: the runtime's bytes for a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct StreamWrite {
    /// The channel.
    pub stream: String,
    /// The bytes, base64.
    pub bytes: String,
}

/// `stream/close`: the runtime ends a channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct StreamClose {
    /// The channel.
    pub stream: String,
}

/// `stream/data`: the command's bytes, as the adapter sends them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct StreamData {
    /// The channel.
    pub stream: String,
    /// The bytes, base64.
    pub bytes: String,
}

/// `stream/closed`: a channel ended, as the adapter says.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct StreamClosed {
    /// The channel.
    pub stream: String,
    /// The command's exit code, where it has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit: Option<i32>,
}
