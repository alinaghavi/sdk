//! The plugin door, version 1 (PROTOCOL.md §4.4): WebSocket text frames between Steery's
//! plugin in a harness and the runtime's loopback listener, each tagged by `t`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::harness::{LiveState, Moment, SessionRef};

/// The door version this crate speaks.
pub const DOOR: u32 = 1;
/// The path a plugin dials on the runtime's loopback listener.
pub const PATH: &str = "/door";
/// The longest frame either side reads.
pub const FRAME_MAX: usize = 16 << 20;

/// Close codes the runtime sends.
pub mod close {
    /// The first frame was not a readable hello, or names no plugin driver.
    pub const HELLO: u16 = 4400;
    /// The runtime does not speak the hello's door version.
    pub const VERSION: u16 = 4409;
    /// A newer connection from the same process and conversation replaced this one.
    pub const REPLACED: u16 = 4410;
}

/// A frame the plugin sends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "t", rename_all = "snake_case", deny_unknown_fields)]
pub enum PluginFrame {
    /// The first frame, within 5 s.
    Hello {
        /// The door version the plugin speaks.
        door: u32,
        /// The id of a driver whose door is `plugin`.
        harness: String,
        /// The harness process.
        pid: u32,
        /// The conversation this copy is in; a sub-agent's own id where it is one.
        session_id: String,
        mode: DoorMode,
        /// The conversation a sub-agent's copy works in.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        lead: Option<SessionRef>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        state: Option<LiveState>,
    },
    /// The whole state, when it changes.
    State { state: LiveState },
    /// A moment of a hook `via: door`.
    Moment { id: String, moment: Moment },
    /// A turn of the copy's own ended.
    Settle {
        id: String,
        /// The turn asked its person something.
        asked: bool,
        /// The conversation was parked.
        parked: bool,
        /// The turn waits on a Steery request.
        waiting: bool,
        /// The turn was already kept going once.
        continued: bool,
    },
    /// The answer to an ask: `ok` the op's answer, or `refused` with the harness's line.
    Answer {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ok: Option<Value>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        refused: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
}

/// Who is at a plugin's harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum DoorMode {
    /// A person at a terminal.
    Tui,
    /// A program drawing for a person.
    Rpc,
    /// A one-shot run.
    Json,
    /// A one-shot run.
    Print,
}

impl DoorMode {
    /// A person, or a program drawing for one, is at it.
    pub fn interactive(self) -> bool {
        matches!(self, Self::Tui | Self::Rpc)
    }
}

/// A frame the runtime sends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "t", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeFrame {
    /// The hello was taken.
    Welcome { door: u32 },
    /// The answer to a moment or a settle.
    Answer { id: String, ok: Spoken },
    /// Something the runtime asks of the copy.
    Ask(Ask),
    /// The conversation's request cards, for a copy that draws.
    Cards {
        /// Every open request, newest first, at most 20.
        open: Vec<Card>,
        /// Cards settled since the last frame.
        settled: Vec<SettledCard>,
    },
}

/// The runtime's answer to a moment or a settle.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Spoken {
    /// A paragraph for the agent; after a settle, the plugin keeps the agent going with it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub say: Option<String>,
}

/// What the runtime asks of a copy, by `op`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Ask {
    /// Words into the conversation; answered `{}` once the harness took them.
    Say {
        id: String,
        words: String,
        mode: DoorSayMode,
        /// Only where the conversation is idle; else `harness-busy`, nothing handed over.
        idle: bool,
        /// One throw's words go in once.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        throw: Option<String>,
    },
    /// A model and thinking; answered with [`super::harness::Picked`].
    Pick {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        thinking: Option<String>,
    },
    /// Stop the turn running; answered with [`DoorStopped`]. Without `say`, the person's
    /// pause.
    Stop {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        say: Option<String>,
    },
    /// Where the conversation stands, in its own model's words; answered with
    /// [`super::harness::Text`].
    Recap { id: String },
}

/// When door words go in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum DoorSayMode {
    /// Into the turn running.
    Steer,
    /// After it.
    Queue,
}

/// What a door's stop did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct DoorStopped {
    /// A turn was stopped.
    pub stopped: bool,
    /// Background jobs still running.
    #[serde(default)]
    pub running: Vec<String>,
}

/// Refusal words of the door's own.
pub mod refusal {
    /// The copy is letting the conversation go and took nothing.
    pub const ENDING: &str = "ending";
    /// The turn running is not one the given `say` began.
    pub const OTHER_TURN: &str = "other-turn";
}

/// An open request card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Card {
    /// The request, as Steery's server gives it.
    pub request: Value,
    /// Its text form.
    pub text: String,
}

/// A card settled since the last frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct SettledCard {
    pub id: String,
    pub title: String,
    pub status: CardStatus,
    /// Who answered it, where known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    /// Why it was withdrawn.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// How a card settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CardStatus {
    Answered,
    Withdrawn,
}
