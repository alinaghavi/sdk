//! The methods of an adapter that makes hosts (PROTOCOL.md §5.2) and its `stream` channel
//! (§5.3). A host is named by the adapter's own id for it.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// `hosts/list`: the adapter's hosts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "HostsList")
)]
#[serde(deny_unknown_fields)]
pub struct List {
    /// Only those carrying every one of these labels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<BTreeMap<String, String>>,
}

/// The hosts `hosts/list` found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "HostsListed")
)]
#[serde(deny_unknown_fields)]
pub struct Listed {
    /// Each one.
    pub hosts: Vec<Host>,
}

/// One host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Host {
    /// The adapter's id for it.
    pub host: String,
    /// Its state, in the vendor's own word.
    pub state: String,
    /// Its labels.
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    /// When it was made.
    pub created_at: u64,
}

/// `hosts/ensure`: the host for `key`, made where none is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Ensure {
    /// What finds it again.
    pub key: String,
    /// What it is made from; absent: the adapter's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// Its labels.
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
}

/// What a host is made from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Source {
    /// A vendor's snapshot, by name.
    Snapshot(String),
    /// A container image, by reference.
    Image(String),
    /// A Dockerfile's whole text, which the vendor builds in its owner's account: the same
    /// bytes, the same build. Only where the adapter's facts say `builds`.
    Dockerfile(String),
}

/// A host, as `hosts/ensure` and `hosts/fork` answer it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Named {
    /// The adapter's id for it.
    pub host: String,
}

/// The parameters of `hosts/start`, `stop`, `delete`, `archive`, `pause`, `fork` and
/// `keepalive`.
pub type Target = Named;

/// A host's state after `hosts/start`, `stop`, `delete`, `archive` or `pause`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct State {
    /// In the vendor's own word.
    pub state: String,
}

/// `hosts/expose`: a way for every other host of the adapter to one port of a host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Expose {
    /// The adapter's id for the host.
    pub host: String,
    /// The port inside it.
    pub port: u16,
}

/// The way `hosts/expose` answers: the same while the host lives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Exposed {
    /// The URL that reaches the port.
    pub url: String,
    /// The headers a request to `url` carries.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
}

/// `hosts/put`: write a file into a host. Its result is [`super::Empty`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Put {
    /// The adapter's id for the host.
    pub host: String,
    /// The absolute file inside it.
    pub path: PathBuf,
    /// Its whole content, base64.
    pub content: String,
    /// Its Unix mode bits.
    pub mode: u32,
}

/// `hosts/channel`: a local way to `command`'s standard input and output in a host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema),
    schemars(rename = "HostsChannel")
)]
#[serde(deny_unknown_fields)]
pub struct Channel {
    /// The adapter's id for the host.
    pub host: String,
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
