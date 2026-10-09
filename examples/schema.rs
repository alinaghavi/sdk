//! Writes one JSON Schema document holding every wire type under `$defs` to standard
//! output: `cargo run -p steery-sdk --features schema --example schema`. Its output is
//! `sdk/schema/protocol-v1.json`.

use schemars::SchemaGenerator;
use schemars::generate::SchemaSettings;
use serde_json::{Value, json};
use steery_sdk::wire::{self, harness, process, sandbox};

fn main() {
    let mut generator = SchemaGenerator::new(SchemaSettings::draft2020_12());
    macro_rules! add {
        ($($ty:ty),* $(,)?) => { $( generator.subschema_for::<$ty>(); )* };
    }
    add!(
        wire::InitializeParams,
        wire::Runtime,
        wire::Initialized,
        wire::ProgramInfo,
        wire::Kind,
        wire::Level,
        wire::Provides,
        wire::HarnessInfo,
        wire::Minimum,
        wire::Setting,
        wire::HarnessFacts,
        wire::Keeps,
        wire::ProcessFacts,
        wire::SandboxFacts,
        wire::Survives,
        wire::Idle,
        wire::Channel,
        wire::Empty,
        harness::Setup,
        harness::Planned,
        harness::Found,
        harness::Entry,
        harness::Format,
        harness::List,
        harness::Listed,
        harness::Session,
        harness::Read,
        harness::Events,
        harness::Event,
        harness::TurnState,
        harness::Failure,
        harness::FailureKind,
        harness::Voice,
        harness::HookRead,
        harness::HookMoment,
        harness::Moment,
        harness::MomentKind,
        harness::MomentCall,
        harness::HookAnswer,
        harness::HookReply,
        harness::HookAnswered,
        harness::Held,
        harness::HeldBy,
        harness::Look,
        harness::Holder,
        harness::HolderStop,
        harness::Door,
        harness::LiveDoor,
        harness::Act,
        harness::Models,
        harness::Model,
        harness::Start,
        harness::Launch,
        harness::Stdin,
        harness::TurnRead,
        harness::Stream,
        harness::TurnLine,
        harness::Ended,
        harness::Stop,
        harness::Stopped,
        harness::Reader,
        harness::ReaderTools,
        harness::ReaderPlan,
        harness::Feed,
        harness::ReadAs,
        harness::Terminal,
        harness::TerminalCommand,
        harness::Say,
        harness::SayMode,
        harness::Said,
        harness::Pick,
        harness::Picked,
        harness::Recap,
        harness::Recapped,
        harness::SessionChanged,
        harness::SessionState,
        harness::LiveCall,
        process::Start,
        process::Started,
        process::List,
        process::Listed,
        process::Entry,
        process::Environ,
        process::Values,
        process::Files,
        process::Paths,
        process::Signal,
        process::SignalName,
        process::Sent,
        process::ServicePlan,
        process::Plan,
        process::PlanCommand,
        process::PlannedFile,
        sandbox::List,
        sandbox::Listed,
        sandbox::Sandbox,
        sandbox::Ensure,
        sandbox::Source,
        sandbox::Named,
        sandbox::State,
        sandbox::Put,
        sandbox::Channel,
        sandbox::Reach,
        sandbox::StreamWrite,
        sandbox::StreamClose,
        sandbox::StreamData,
        sandbox::StreamClosed,
    );
    let mut defs: serde_json::Map<String, Value> = generator.take_definitions(true);
    for schema in defs.values_mut() {
        tidy(schema);
    }
    // The root names every definition, so a generator that emits only what it reaches
    // emits them all.
    let refs: Vec<Value> = defs
        .keys()
        .map(|name| json!({ "$ref": format!("#/$defs/{name}") }))
        .collect();
    let document = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://steery.dev/schema/protocol-v1.json",
        "title": "Protocol",
        "description": "Every shape of the Steery program protocol, version 1 (PROTOCOL.md).",
        "anyOf": refs,
        "$defs": defs,
    });
    let text = serde_json::to_string_pretty(&document).expect("a schema is JSON");
    println!("{text}");
}

/// Shapes the schema for json-schema-to-typescript, keeping what it means. A schema that
/// takes any JSON value (a `serde_json::Value`, which carries only its description) is
/// marked `tsType: unknown`, which it would otherwise read as an object; a `$ref` beside a
/// description goes into an `allOf`, which it would otherwise read as a new type.
fn tidy(schema: &mut Value) {
    match schema {
        Value::Object(map) => {
            if map.len() == 1 && map.contains_key("description") {
                map.insert("tsType".to_owned(), json!("unknown"));
                return;
            }
            if map.len() > 1
                && let Some(reference) = map.remove("$ref")
            {
                map.insert("allOf".to_owned(), json!([{ "$ref": reference }]));
                return;
            }
            for value in map.values_mut() {
                tidy(value);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(tidy),
        _ => {}
    }
}
