//! Home 2: a crate of the concern holds the mechanism and every dial.
//!
//! A sketch. It decides nothing. Each `mod` stands for a crate, and each `use`
//! between two mods stands for an entry in a manifest. Files arrive already
//! parsed, as `(key, value)` pairs, so the sketch needs only std.
//! It copies purba-errors (a crate of a concern that uses no part) and cargo
//! (`GlobalContext` and `context/schema.rs` declare most typed values).

#![allow(dead_code)]

/// The crate of the concern. `lint:wiring` refuses any `purba-*` entry in its
/// manifest, purba-errors included, so it holds its own refusal.
pub mod purba_configuration {
    use std::path::PathBuf;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Level {
        CommandLine,
        Environment(String),
        Code { line: usize },
        Project(PathBuf),
        User(PathBuf),
        System(PathBuf),
    }

    /// A value and the level it came from.
    pub struct Resolved<T> {
        pub value: T,
        pub level: Level,
    }

    /// Every dial of purba, typed here. A part reads its fields.
    pub struct Dials {
        pub timeout: Option<Resolved<u32>>,
        pub timeouts: Vec<(String, Resolved<u32>)>,
        pub workers: Option<Resolved<u16>>,
        pub suppression_cap: Option<Resolved<u32>>,
    }

    /// The code level, as data. The crate cannot read an AST: parsing is a part.
    pub struct CodeSetting {
        pub function: String,
        pub key: String,
        pub value: String,
        pub line: usize,
    }

    /// What the crate refuses. A part turns it into an invocation error.
    pub struct Refusal {
        pub words: String,
        pub nearest: Option<&'static str>,
    }

    pub struct Sources<'a> {
        pub command_line: &'a [(String, String)],
        pub environment: &'a [(String, String)],
        pub code: &'a [CodeSetting],
        pub project: Option<(PathBuf, Vec<(String, String)>)>,
        pub user: Option<(PathBuf, Vec<(String, String)>)>,
        pub system: Option<(PathBuf, Vec<(String, String)>)>,
    }

    /// Rules 1 to 6 live here, once, for every dial.
    pub fn resolve(sources: &Sources<'_>) -> Result<Dials, Refusal> {
        let _ = sources;
        todo!("one table of dials: name, type, rule, the levels closed to it")
    }
}

pub mod purba_errors {
    pub struct Error(pub String);

    impl Error {
        pub fn invocation(words: String) -> Self {
            Self(words)
        }
    }
}

/// Selection hands the code level to command as data of the concern's crate.
pub mod purba_selection {
    use super::purba_configuration::CodeSetting;

    pub fn settings() -> Vec<CodeSetting> {
        Vec::new()
    }
}

pub mod purba_dispatch {
    use super::purba_configuration::Dials;

    /// Dispatch reads its fields of the whole set of dials.
    pub fn timeout_of(dials: &Dials, function: &str) -> Option<u32> {
        dials
            .timeouts
            .iter()
            .find(|(name, _)| name == function)
            .map(|(_, resolved)| resolved.value)
            .or_else(|| dials.timeout.as_ref().map(|resolved| resolved.value))
    }
}

pub mod purba_command {
    use super::{purba_configuration as configuration, purba_errors, purba_selection};

    pub fn read(sources: &configuration::Sources<'_>) -> Result<configuration::Dials, purba_errors::Error> {
        let _code = purba_selection::settings();
        configuration::resolve(sources).map_err(|refusal| purba_errors::Error::invocation(refusal.words))
    }
}
