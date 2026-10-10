//! The smallest sandbox adapter: a "provider" whose sandboxes are folders under
//! `~/.example-sandboxes/<id>`, reached by running a command there with `sh`. It shows
//! the shape of PROTOCOL.md §5.2 — list, ensure, start, stop, delete and a channel — with
//! nothing between the adapter and its provider. An adapter for a real provider calls the
//! provider's published API in each method, and refuses with `offline` while it cannot be
//! reached.
//!
//! It declares one setting, `EXAMPLE_TOKEN`, to show how a person gives one: at `steery
//! machine add example --setting EXAMPLE_TOKEN`.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde_json::Value;
use steery_sdk::wire::{self, sandbox};
use steery_sdk::{Error, PROTOCOL, Program, method, params, refusal, result};

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    steery_sdk::serve(|_notifier| Example::default()).await
}

#[derive(Default)]
struct Example {
    /// Where the sandboxes are: `~/.example-sandboxes`, from the home `initialize` gave.
    root: OnceLock<PathBuf>,
}

impl Program for Example {
    fn initialize(&self, params: &wire::InitializeParams) -> Result<wire::Initialized, Error> {
        if !params.protocol.contains(&PROTOCOL) {
            return Err(Error::invalid(format!(
                "this adapter speaks protocol {PROTOCOL}"
            )));
        }
        let _ = self.root.set(params.home.join(".example-sandboxes"));
        Ok(wire::Initialized {
            protocol: PROTOCOL,
            program: wire::ProgramInfo {
                id: "example-sandboxes".parse().map_err(Error::invalid)?,
                kind: wire::Kind::Machine,
                release: env!("CARGO_PKG_VERSION").into(),
                label: "Example sandboxes".into(),
                short: "Ex".into(),
                tint: None,
                mark: None,
            },
            harness: None,
            machine: Some(wire::Machine::Sandboxes(wire::SandboxFacts {
                keeps: wire::Survives::Filesystem,
                persist: None,
                idle: wire::Idle::None,
                optional: Vec::new(),
                channel: wire::Channel::Argv,
                os: wire::Os::Linux,
                unavailable: None,
            })),
            settings: vec![wire::Setting {
                name: "EXAMPLE_TOKEN".into(),
                secret: true,
                required: false,
                about: "A token for the provider; this example reads none".into(),
            }],
        })
    }

    async fn call(&self, method: &str, given: Value) -> Result<Value, Error> {
        match method {
            method::SANDBOXES_LIST => {
                let asked: sandbox::List = params(given)?;
                result(sandbox::Listed {
                    sandboxes: self.list(asked.labels.as_ref()),
                })
            }
            method::SANDBOX_ENSURE => {
                let asked: sandbox::Ensure = params(given)?;
                let folder = self.folder(&asked.key)?;
                std::fs::create_dir_all(&folder).map_err(failed)?;
                let labels = serde_json::to_vec(&asked.labels).map_err(failed)?;
                std::fs::write(folder.join(".labels.json"), labels).map_err(failed)?;
                result(sandbox::Named { sandbox: asked.key })
            }
            method::SANDBOX_START | method::SANDBOX_STOP => {
                let asked: sandbox::Target = params(given)?;
                self.existing(&asked.sandbox)?;
                // A folder is always running; a provider says its own word here.
                result(sandbox::State {
                    state: "started".into(),
                })
            }
            method::SANDBOX_DELETE => {
                let asked: sandbox::Target = params(given)?;
                std::fs::remove_dir_all(self.existing(&asked.sandbox)?).map_err(failed)?;
                result(sandbox::State {
                    state: "deleted".into(),
                })
            }
            method::SANDBOX_CHANNEL => {
                let asked: sandbox::Channel = params(given)?;
                let folder = self.existing(&asked.sandbox)?;
                let mut argv = vec![
                    "sh".to_owned(),
                    "-c".to_owned(),
                    "cd \"$1\" && shift && exec \"$@\"".to_owned(),
                    "sh".to_owned(),
                    folder.display().to_string(),
                ];
                argv.extend(asked.command);
                result(sandbox::Reach::Argv {
                    argv,
                    env: BTreeMap::new(),
                })
            }
            _ => Err(Error::Unknown),
        }
    }
}

impl Example {
    /// The folder of the sandbox `id`; an id that is no plain name is refused.
    fn folder(&self, id: &str) -> Result<PathBuf, Error> {
        let plain = !id.is_empty()
            && id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
        if !plain {
            return Err(Error::invalid(format!("{id:?} is no sandbox id")));
        }
        let root = self
            .root
            .get()
            .ok_or_else(|| Error::invalid("not initialized"))?;
        Ok(root.join(id))
    }

    /// The folder of the sandbox `id`, which must exist.
    fn existing(&self, id: &str) -> Result<PathBuf, Error> {
        let folder = self.folder(id)?;
        if folder.is_dir() {
            Ok(folder)
        } else {
            Err(Error::refused(
                refusal::NOT_FOUND,
                format!("no sandbox {id}"),
            ))
        }
    }

    /// Every sandbox carrying each of `labels`.
    fn list(&self, labels: Option<&BTreeMap<String, String>>) -> Vec<sandbox::Sandbox> {
        let Some(Ok(entries)) = self.root.get().map(std::fs::read_dir) else {
            return Vec::new();
        };
        entries
            .flatten()
            .filter_map(|entry| {
                let id = entry.file_name().to_str()?.to_owned();
                let kept: BTreeMap<String, String> =
                    std::fs::read(entry.path().join(".labels.json"))
                        .ok()
                        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                        .unwrap_or_default();
                let wanted =
                    labels.is_none_or(|labels| labels.iter().all(|(k, v)| kept.get(k) == Some(v)));
                let created_at = entry
                    .metadata()
                    .ok()?
                    .modified()
                    .ok()?
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_millis() as u64;
                wanted.then_some(sandbox::Sandbox {
                    sandbox: id,
                    state: "started".into(),
                    labels: kept,
                    created_at,
                })
            })
            .collect()
    }
}

/// A file operation that failed, as the refusal a provider's failure is.
fn failed(error: impl std::fmt::Display) -> Error {
    Error::refused(refusal::REFUSED, error.to_string())
}
