//! How programs are published (PROTOCOL.md §7): a release is a manifest,
//! [`FILE`], beside one archive per program and target. The runtime reads only this to add
//! a program, so whoever publishes one writes exactly this shape.
//!
//! ```json
//! {"manifest": 1, "release": "v1.2.0",
//!  "programs": [{"id": "omp", "kind": "harness",
//!                "files": {"aarch64-apple-darwin": {
//!                  "archive": "steery-driver-omp-v1.2.0-aarch64-apple-darwin.tar.gz",
//!                  "sha256": "…64 lowercase hex…"}}}]}
//! ```

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::wire::{Kind, ProgramId};

/// The manifest's file name: a release's asset, or any URL's last segment.
pub const FILE: &str = "steery-programs.json";

/// The manifest format this crate reads and writes.
pub const MANIFEST: u32 = 1;

/// The longest `release`, in characters.
pub const RELEASE_MAX: usize = 64;

/// A release of one or more programs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// [`MANIFEST`].
    pub manifest: u32,
    /// The release's name, as its tag reads: letters, digits, `.`, `-`, `_` and `+`, at
    /// most [`RELEASE_MAX`] characters. Every archive's name carries it.
    pub release: String,
    /// Each program of the release, each id once.
    pub programs: Vec<Program>,
}

/// One program of a release.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct Program {
    /// The id its `initialize` answers.
    pub id: ProgramId,
    /// The kind its `initialize` answers: `harness` for a driver, `machine` for an adapter.
    pub kind: Kind,
    /// Its build for each target, by Rust target triple: `aarch64-apple-darwin`,
    /// `x86_64-unknown-linux-gnu`. At least one.
    pub files: BTreeMap<String, File>,
}

/// One build of a program: an archive holding exactly one regular file, the program's
/// executable ([`executable`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
pub struct File {
    /// The archive's file name, exactly [`archive`]'s: it is fetched from the manifest's
    /// own folder.
    pub archive: String,
    /// The archive's SHA-256, 64 lowercase hex digits.
    pub sha256: String,
}

impl Manifest {
    /// What the shapes cannot say themselves: the format, a release name that is safe in a
    /// file name, each id once, at least one build each, every target a triple, every
    /// archive named for its program, release and target, and every digest 64 lowercase
    /// hex. The first rule broken, in one line.
    pub fn check(&self) -> Result<(), String> {
        if self.manifest != MANIFEST {
            return Err(format!(
                "is manifest format {}, and this reads {MANIFEST}",
                self.manifest
            ));
        }
        if !release_name(&self.release) {
            return Err(format!(
                "names the release {:?}: one of letters, digits, `.`, `-`, `_` and `+`, at most {RELEASE_MAX}, is wanted",
                self.release
            ));
        }
        if self.programs.is_empty() {
            return Err("names no program".into());
        }
        let mut ids = BTreeSet::new();
        for program in &self.programs {
            if !ids.insert(program.id.as_str()) {
                return Err(format!("names the program {} twice", program.id));
            }
            if program.files.is_empty() {
                return Err(format!("has no build of {}", program.id));
            }
            for (target, file) in &program.files {
                if !triple(target) {
                    return Err(format!("names {target:?}, which is no target triple"));
                }
                let named = archive(program.kind, &program.id, &self.release, target);
                if file.archive != named {
                    return Err(format!(
                        "names {:?} for {} on {target}, and {named:?} is its name",
                        file.archive, program.id
                    ));
                }
                if file.sha256.len() != 64
                    || !file
                        .sha256
                        .bytes()
                        .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
                {
                    return Err(format!(
                        "gives {} for {target} a sha256 that is not 64 lowercase hex digits",
                        program.id
                    ));
                }
            }
        }
        Ok(())
    }

    /// The program `id`, where the release has it.
    pub fn program(&self, id: &str) -> Option<&Program> {
        self.programs.iter().find(|program| program.id == id)
    }
}

/// The executable a program's archive holds: `steery-driver-<id>` for a driver,
/// `steery-adapter-<id>` for an adapter.
pub fn executable(kind: Kind, id: &ProgramId) -> String {
    format!("steery-{}-{id}", noun(kind))
}

/// What a person calls a program of `kind`: `driver` or `adapter`.
pub fn noun(kind: Kind) -> &'static str {
    match kind {
        Kind::Harness => "driver",
        Kind::Machine => "adapter",
    }
}

/// The name of the archive of `id`'s build for `target` in `release`:
/// `steery-driver-omp-v1.2.0-aarch64-apple-darwin.tar.gz`.
pub fn archive(kind: Kind, id: &ProgramId, release: &str, target: &str) -> String {
    format!("{}-{release}-{target}.tar.gz", executable(kind, id))
}

/// The target triple of a system `os` on `arch`, as Rust names both, where programs are
/// published for it: `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`.
pub fn target(os: &str, arch: &str) -> Option<String> {
    match (os, arch) {
        ("macos", "aarch64" | "x86_64") => Some(format!("{arch}-apple-darwin")),
        ("linux", "aarch64" | "x86_64") => Some(format!("{arch}-unknown-linux-gnu")),
        _ => None,
    }
}

/// Whether `release` is safe to put in a file name and a URL.
fn release_name(release: &str) -> bool {
    !release.is_empty()
        && release.chars().count() <= RELEASE_MAX
        && !release.starts_with(['.', '-'])
        && release
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".-_+".contains(&b))
}

/// Whether `target` reads as a target triple: three or four words of lowercase letters,
/// digits and `_`, joined by `-`.
fn triple(target: &str) -> bool {
    let words: Vec<&str> = target.split('-').collect();
    (3..=4).contains(&words.len())
        && words.iter().all(|word| {
            !word.is_empty()
                && word
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Manifest {
        let id: ProgramId = "omp".parse().unwrap();
        let target = "aarch64-apple-darwin";
        Manifest {
            manifest: MANIFEST,
            release: "v1.2.0".into(),
            programs: vec![Program {
                files: BTreeMap::from([(
                    target.to_owned(),
                    File {
                        archive: archive(Kind::Harness, &id, "v1.2.0", target),
                        sha256: "0".repeat(64),
                    },
                )]),
                id,
                kind: Kind::Harness,
            }],
        }
    }

    /// An archive's name is fixed by its program, release and target, so a manifest can
    /// point at no other file: a path, another release's archive, or another program's.
    #[test]
    fn an_archive_is_named_for_its_program_release_and_target() {
        assert_eq!(manifest().check(), Ok(()));
        for (named, why) in [
            (
                "../steery-driver-omp-v1.2.0-aarch64-apple-darwin.tar.gz",
                "a path",
            ),
            (
                "steery-driver-omp-v1.1.0-aarch64-apple-darwin.tar.gz",
                "another release",
            ),
            (
                "steery-driver-pi-v1.2.0-aarch64-apple-darwin.tar.gz",
                "another program",
            ),
            (
                "steery-adapter-omp-v1.2.0-aarch64-apple-darwin.tar.gz",
                "another kind",
            ),
        ] {
            let mut m = manifest();
            m.programs[0].files.values_mut().next().unwrap().archive = named.into();
            assert!(m.check().is_err(), "{why} is refused");
        }
    }

    #[test]
    fn a_release_name_is_safe_in_a_path() {
        for release in ["../x", "a/b", "", ".hidden", "-x", "v1 2"] {
            let mut m = manifest();
            m.release = release.into();
            assert!(m.check().is_err(), "{release:?} is refused");
        }
        let mut m = manifest();
        m.release = "v1.2.0+build_3".into();
        let named = archive(
            Kind::Harness,
            &m.programs[0].id,
            "v1.2.0+build_3",
            "aarch64-apple-darwin",
        );
        m.programs[0].files.values_mut().next().unwrap().archive = named;
        assert_eq!(m.check(), Ok(()));
    }

    #[test]
    fn an_id_twice_and_a_bad_digest_are_refused() {
        let mut twice = manifest();
        twice.programs.push(twice.programs[0].clone());
        assert!(twice.check().unwrap_err().contains("twice"));
        let mut digest = manifest();
        digest.programs[0].files.values_mut().next().unwrap().sha256 = "A".repeat(64);
        assert!(digest.check().unwrap_err().contains("sha256"));
    }

    #[test]
    fn unknown_fields_are_refused() {
        let text = r#"{"manifest": 1, "release": "v1", "programs": [], "extra": true}"#;
        assert!(serde_json::from_str::<Manifest>(text).is_err());
    }
}
