//! Home 1: command reads and resolves, and hands each part its typed values.
//!
//! A sketch. It decides nothing. Each `mod` stands for a crate, and each `use`
//! between two mods stands for an entry in a manifest. Files arrive already
//! parsed, as `(key, value)` pairs, so the sketch needs only std.
//! It copies ruff (`ruff_workspace` builds `LinterSettings`, which `ruff_linter`
//! declares) and uv (`uv-cli` resolves; the crates that do the work take typed
//! arguments). It is the shape of the prototype's `config.rs`, with the types
//! moved into the parts.

#![allow(dead_code)]

/// Selection reads the code level, because only selection reads the comments of a test.
pub mod purba_selection {
    /// One `# purba: set[<key>] <value>` above the `def` of `function`.
    pub struct Setting {
        pub function: String,
        pub key: String,
        pub value: String,
        pub line: usize,
    }
}

/// Dispatch declares what it reads. It owns what each value means.
pub mod purba_dispatch {
    use std::collections::HashMap;
    use std::num::NonZeroU32;

    /// A timeout in seconds. 0 is no value, so a zero cannot turn the check off.
    #[derive(Clone, Copy)]
    pub struct Seconds(pub NonZeroU32);

    pub struct Dials {
        pub timeout: Option<Seconds>,
        pub timeouts: HashMap<String, Seconds>,
        pub workers: Option<u16>,
    }
}

/// Reporting declares the cap, because it counts the suppressed verdicts.
pub mod purba_reporting {
    pub struct Dials {
        pub suppression_cap: Option<u32>,
    }
}

pub mod purba_command {
    use super::{purba_dispatch, purba_reporting, purba_selection};
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

    /// How two levels of one dial resolve.
    pub enum Rule {
        /// Rule 2: the higher level overwrites the lower one.
        Higher,
        /// Rule 2, the cap: the lower value applies.
        Lower,
    }

    /// The one table of dials. Command holds every name.
    pub struct Dial {
        pub name: &'static str,
        pub rule: Rule,
        /// Rule 3: the user file and the system file may not set it.
        pub changes_an_outcome: bool,
    }

    pub const DIALS: [Dial; 3] = [
        Dial { name: "timeout", rule: Rule::Higher, changes_an_outcome: true },
        Dial { name: "workers", rule: Rule::Higher, changes_an_outcome: false },
        Dial { name: "suppression-cap", rule: Rule::Lower, changes_an_outcome: true },
    ];

    /// One value that one level sets.
    pub struct Set {
        pub dial: &'static str,
        pub value: String,
        pub level: Level,
    }

    /// Command's own invocation error. Command may use purba-errors.
    pub struct Refusal(pub String);

    /// Every value of every level, before any rule applies.
    pub struct Levels {
        pub sets: Vec<Set>,
        pub code: Vec<purba_selection::Setting>,
    }

    impl Levels {
        /// Rule 6 and rule 3, at the read: a key no dial owns, or a closed level.
        pub fn read(level: Level, pairs: &[(String, String)]) -> Result<Vec<Set>, Refusal> {
            let closed = matches!(level, Level::User(_) | Level::System(_));
            let mut sets = Vec::new();
            for (key, value) in pairs {
                let Some(dial) = DIALS.iter().find(|dial| dial.name == key) else {
                    return Err(Refusal(format!("unknown key `{key}`")));
                };
                if closed && dial.changes_an_outcome {
                    return Err(Refusal(format!("{level:?} may not set {key}")));
                }
                sets.push(Set { dial: dial.name, value: value.clone(), level: level.clone() });
            }
            Ok(sets)
        }

        /// The typed values of dispatch. Command turns each string into dispatch's type.
        pub fn dispatch(&self) -> Result<purba_dispatch::Dials, Refusal> {
            todo!("resolve timeout and workers by DIALS, and each code timeout by rule 4")
        }

        pub fn reporting(&self) -> Result<purba_reporting::Dials, Refusal> {
            todo!("resolve suppression-cap by Rule::Lower")
        }
    }
}
