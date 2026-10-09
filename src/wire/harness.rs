//! A driver's methods (PROTOCOL.md §4): observe, drive and live.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `setup/plan`: what setting the harness up for Steery writes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Setup {
    /// The person's home folder.
    pub home: PathBuf,
    /// Steery's MCP URL.
    pub mcp_url: String,
    /// The skill's Markdown.
    pub skill: String,
    /// The command line a hook runs.
    pub hook: String,
    /// The runtime's loopback door URL, which a `plugin` door dials.
    pub door: String,
    /// The runtime's release.
    pub release: String,
}

/// The harness found here, and the entries that set it up.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Planned {
    /// The harness, where it is installed here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub found: Option<Found>,
    /// One line for a person, where the release found is older than the minimum.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outdated: Option<String>,
    /// What the runtime writes into the harness, and nothing else.
    pub entries: Vec<Entry>,
    /// Lines for a person about the setup.
    #[serde(default)]
    pub notes: Vec<String>,
}

/// An installed harness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Found {
    /// Its binary.
    pub binary: PathBuf,
    /// Its release.
    pub release: String,
    /// The folder it keeps its configuration and conversations in.
    pub root: PathBuf,
}

/// One place in one file the runtime writes, merges byte-exactly, records, and gives back
/// when the harness is removed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Entry {
    /// The absolute file.
    pub path: PathBuf,
    /// How the file reads.
    pub format: Format,
    /// The place in it: a JSON Pointer or a TOML key path; absent: the whole file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<String>,
    /// What goes there; a `text` file's is a string.
    pub value: Value,
}

/// How a setup file reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Format {
    Json,
    Toml,
    Text,
}

/// `sessions/list`: the harness's conversations.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct List {
    /// Only those updated since.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seen_since: Option<u64>,
}

/// The conversations `sessions/list` found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Listed {
    /// Each one.
    pub sessions: Vec<Session>,
}

/// One conversation in the harness's store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Session {
    /// The harness's id for it.
    pub external_id: String,
    /// The folder it runs in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    /// Its title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// When it began.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
    /// When it last changed.
    pub updated_at: u64,
    /// Its transcript file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript: Option<PathBuf>,
    /// The conversation it is a sub-agent of.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lead: Option<String>,
}

/// `sessions/read`: a conversation's events after a cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Read {
    /// The conversation.
    pub external_id: String,
    /// Where the last read ended; absent: from the beginning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// The events `sessions/read` read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Events {
    /// Where this read ended, for the next.
    pub cursor: String,
    /// Each one, in order.
    pub events: Vec<Event>,
}

/// One thing a conversation did.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    TurnBegan {
        at: u64,
    },
    TurnEnded {
        at: u64,
        state: TurnState,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        failure: Option<Failure>,
    },
    Words {
        at: u64,
        voice: Voice,
        text: String,
    },
    CallBegan {
        at: u64,
        /// The harness's id for the call.
        id: String,
        tool: String,
        /// The MCP server or extension the tool belongs to.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        group: Option<String>,
    },
    CallEnded {
        at: u64,
        id: String,
        ok: bool,
    },
    Tokens {
        at: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        model: Option<String>,
        input: u64,
        output: u64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cache_read: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cache_write: Option<u64>,
    },
    Model {
        at: u64,
        model: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        thinking: Option<String>,
    },
    SubAgent {
        /// The sub-agent's own conversation.
        external_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        task: Option<String>,
    },
    Link {
        at: u64,
        url: String,
    },
}

/// How a turn ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum TurnState {
    Finished,
    Failed,
    Interrupted,
}

/// Why a turn failed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Failure {
    /// The kind.
    pub kind: FailureKind,
    /// The harness's own line.
    pub line: String,
}

/// The kind of a turn's failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    SignIn,
    RateLimit,
    ContextFull,
    ModelMissing,
    Other,
}

/// Who said words in a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Voice {
    Person,
    Agent,
    Steery,
}

/// `hook/read`: what the harness handed its hook.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HookRead {
    /// The harness's own name for the moment.
    pub event: String,
    /// The document the harness handed the hook, as it came.
    pub document: String,
}

/// The moment `hook/read` read, where it is one Steery carries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HookMoment {
    /// The moment; absent: none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moment: Option<Moment>,
}

/// A moment the harness stopped at.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Moment {
    /// The kind.
    pub kind: MomentKind,
    /// The conversation it stopped in.
    pub external_id: String,
    /// The tool call it stopped at.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call: Option<MomentCall>,
}

/// The kind of a [`Moment`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum MomentKind {
    Prompt,
    BeforeCall,
    AfterCall,
    TurnEnd,
    SessionStart,
    SessionEnd,
}

/// The tool call a moment is at.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct MomentCall {
    /// The tool.
    pub tool: String,
    /// Its input.
    pub input: Value,
}

/// `hook/answer`: what the hook prints back for an answer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HookAnswer {
    /// The harness's own name for the moment.
    pub event: String,
    /// The document the harness handed the hook, as it came.
    pub document: String,
    /// What the person answered.
    pub answer: HookReply,
}

/// What a person answered a moment with.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HookReply {
    /// Their words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<String>,
    /// Their decision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
}

/// What the hook prints back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct HookAnswered {
    /// Its standard output.
    pub stdout: String,
    /// Its exit code.
    pub exit_code: i32,
}

/// `held`: who holds a conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Held {
    /// The conversation.
    pub external_id: String,
}

/// What `held` answers: who holds the conversation, or where the runtime looks to find out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(untagged)]
pub enum HeldBy {
    /// The driver knows.
    Holder(Holder),
    /// The driver knows only where to look.
    Look {
        /// Where to look.
        look: Look,
    },
}

/// Where the runtime looks for a conversation's holder: the operating-system adapter says
/// which of these executables hold the transcript open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Look {
    /// The conversation's transcript file.
    pub transcript: PathBuf,
    /// The harness's executable names.
    pub executables: Vec<String>,
}

/// Who holds a conversation. A driver that cannot prove absence answers a person whose
/// stop is a process with `certain: false`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Holder {
    /// Nobody.
    Nobody {
        /// What proves it: the harness's own registry, or no process writing the transcript.
        proof: String,
    },
    /// A person's process.
    Person {
        /// How its live door treats a model pick; absent: it has none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        door: Option<Door>,
        /// How it is stopped.
        stop: HolderStop,
    },
}

/// How a holding process is stopped.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum HolderStop {
    /// Only its owner stops it.
    Owner,
    /// This process.
    Process {
        pid: u32,
        process_start: u64,
        /// The driver proved this is the holder.
        certain: bool,
    },
    /// Somewhere the runtime cannot reach, said in one line.
    Beyond(String),
}

/// How a harness's own live door treats a model pick.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Door {
    /// It applies a pick itself.
    Picks,
    /// It keeps its own.
    Keeps,
}

/// How a driver reaches a live conversation (PROTOCOL.md §4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LiveDoor {
    /// Steery's plugin door: the runtime speaks it directly and applies a pick itself; the
    /// driver answers no live method.
    Plugin,
    /// The harness's own door, which applies a pick itself.
    Picks,
    /// The harness's own door, which keeps its own.
    Keeps,
}

/// What a turn does with a conversation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Act {
    Continue,
    Fork,
    New,
    Hand,
}

/// `models`: the models the harness runs here. Its parameters are [`super::Empty`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Models {
    /// Each one listed.
    pub models: Vec<Model>,
    /// The harness also runs models it does not list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unlisted: Option<bool>,
}

/// One model a harness runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Model {
    /// Its provider, as the harness spells it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// Its id, as the harness spells it.
    pub model: String,
    /// Its name as a person reads it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The thinking levels it takes, in the harness's own words.
    #[serde(default)]
    pub thinking: Vec<String>,
}

/// `start`: the launch plan of a turn. It starts nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Start {
    /// What the turn does.
    pub act: Act,
    /// The conversation continued or forked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    /// The folder it runs in.
    pub cwd: PathBuf,
    /// Run a fork or a new conversation in a new worktree.
    pub worktree: bool,
    /// The model, as the harness spells it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// How hard it thinks, in the harness's own word.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    /// Folders outside `cwd` the turn reads.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reads: Vec<PathBuf>,
}

/// A command the runtime starts, through the operating-system adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Launch {
    /// The program and its arguments.
    pub argv: Vec<String>,
    /// The variables it adds to the environment.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// The folder it runs in.
    pub cwd: PathBuf,
    /// What its standard input carries.
    pub stdin: Stdin,
}

/// What a launch's standard input carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Stdin {
    /// The turn's words.
    Words,
    /// Nothing.
    None,
}

/// `turn/read`: one line a turn's process wrote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TurnRead {
    /// The stream it came on.
    pub stream: Stream,
    /// The line.
    pub line: String,
}

/// One of a process's output streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Stream {
    Stdout,
    Stderr,
}

/// What a turn's line says.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TurnLine {
    /// The conversation the harness named.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub named: Option<String>,
    /// The harness took the words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub took: Option<bool>,
    /// The turn ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended: Option<Ended>,
    /// A refusal word the harness gave.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refused: Option<String>,
}

/// How a turn ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Ended {
    /// The state.
    pub state: TurnState,
    /// The harness's own line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// `stop`: the harness's own way to end a turn; the runtime's signals come after.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Stop {
    /// The conversation.
    pub external_id: String,
    /// Who holds it, as `held` said.
    pub holder: Holder,
}

/// Whether `stop` ended the turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Stopped {
    /// It ended.
    pub stopped: bool,
}

/// `reader`: a launch plan for a one-shot model read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Reader {
    /// The system prompt.
    pub system: String,
    /// The model, as the harness spells it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The tools it may use.
    pub tools: ReaderTools,
}

/// The tools a reader may use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ReaderTools {
    None,
    Read,
}

/// A reader's launch plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ReaderPlan {
    /// The command.
    #[serde(flatten)]
    pub launch: Launch,
    /// How the input reaches it.
    pub feed: Feed,
    /// How its output reads.
    pub read: ReadAs,
}

/// How a reader's input reaches it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Feed {
    Stdin,
    File,
}

/// How a reader's output reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ReadAs {
    /// The whole output is the answer.
    Whole,
    /// The output is events, each read with `turn/read`.
    Events,
}

/// `terminal`: the command that opens a conversation in a terminal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Terminal {
    /// The conversation.
    pub external_id: String,
    /// The folder it opens in.
    pub cwd: PathBuf,
}

/// The command `terminal` gave.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct TerminalCommand {
    /// The command line.
    pub command: String,
}

/// `say`: words into a running conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Say {
    /// The conversation.
    pub external_id: String,
    /// The words.
    pub words: String,
    /// When they go in.
    pub mode: SayMode,
}

/// When said words go in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SayMode {
    /// Into the turn running.
    Steer,
    /// After the turn running.
    Queue,
    /// At once.
    Immediate,
}

/// When the harness took said words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Said {
    /// When.
    pub at: u64,
}

/// `pick`: a model and thinking for a running conversation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Pick {
    /// The conversation.
    pub external_id: String,
    /// The model, as the harness spells it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// How hard it thinks, in the harness's own word.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
}

/// The model and thinking a conversation runs on, as read back after `pick`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Picked {
    /// The model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// How hard it thinks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
}

/// `recap`: a conversation's own model says where it stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Recap {
    /// The conversation.
    pub external_id: String,
    /// What is asked of it.
    pub asked: String,
}

/// The recap's words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Recapped {
    /// Written by the conversation's own model.
    pub text: String,
}

/// `session/changed`: a notification a driver at `live` sends unasked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SessionChanged {
    /// The conversation.
    pub external_id: String,
    /// Where it stands now.
    pub state: SessionState,
}

/// Where a running conversation stands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct SessionState {
    /// A turn runs.
    pub busy: bool,
    /// The door checked it idle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idle: Option<bool>,
    /// The model it runs on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// How hard it thinks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    /// The share of its context window left, 0 to 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_left: Option<f64>,
    /// The tool calls running.
    #[serde(default)]
    pub calls: Vec<LiveCall>,
}

/// A tool call running.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct LiveCall {
    /// The harness's id for it.
    pub id: String,
    /// The tool.
    pub tool: String,
    /// When it began.
    pub since: u64,
}
