//! Home 3: a crate of the concern holds the mechanism, and each part declares
//! its own dials to it.
//!
//! A sketch. It decides nothing. Each `mod` stands for a crate, and each `use`
//! between two mods stands for an entry in a manifest. Files arrive already
//! parsed, as `(key, value)` pairs, so the sketch needs only std.
//! It copies figment's guide "For Library Authors" (each library declares its
//! `Config` and extracts it from a shared `Figment`), schematic's per-field
//! `merge`, and cargo's consumers that declare a struct and read it by key.

#![allow(dead_code)]

/// The crate of the concern: names and mechanism, no dial.
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

    pub enum Rule {
        /// Rule 2: the higher level overwrites the lower one.
        Higher,
        /// Rule 2, the cap: the lower value applies.
        Lower,
    }

    /// What a part declares about one dial. The part owns its meaning.
    pub struct Dial {
        pub name: &'static str,
        pub rule: Rule,
        /// Rule 3: the user file and the system file may not set it.
        pub changes_an_outcome: bool,
        /// Rule 4: code may set it on one test.
        pub in_code: bool,
    }

    /// A part implements this for the struct that it reads.
    pub trait Dials: Sized {
        const DIALS: &'static [Dial];
        fn from_resolved(resolved: &Resolved) -> Result<Self, Refusal>;
    }

    pub struct Set {
        pub dial: &'static str,
        pub value: String,
        pub level: Level,
    }

    /// Each resolved value, by the name of its dial, with its level.
    pub struct Resolved {
        pub sets: Vec<Set>,
    }

    impl Resolved {
        pub fn get(&self, dial: &str) -> Option<&Set> {
            self.sets.iter().find(|set| set.dial == dial)
        }
    }

    pub struct Refusal {
        pub words: String,
        pub nearest: Option<&'static str>,
    }

    /// Rule 6 needs every declared dial before it reads a key.
    pub fn read(declared: &[&'static [Dial]], levels: &[(Level, Vec<(String, String)>)]) -> Result<Resolved, Refusal> {
        let _ = (declared, levels);
        todo!("refuse a key no declared dial owns, apply each Dial's rule and closed levels")
    }
}

pub mod purba_dispatch {
    use super::purba_configuration::{Dial, Dials as Declared, Refusal, Resolved, Rule};

    pub struct Dials {
        pub timeout: Option<u32>,
        pub workers: Option<u16>,
    }

    impl Declared for Dials {
        const DIALS: &'static [Dial] = &[
            Dial { name: "timeout", rule: Rule::Higher, changes_an_outcome: true, in_code: true },
            Dial { name: "workers", rule: Rule::Higher, changes_an_outcome: false, in_code: false },
        ];

        fn from_resolved(resolved: &Resolved) -> Result<Self, Refusal> {
            let number = |dial: &str| resolved.get(dial).and_then(|set| set.value.parse().ok());
            Ok(Self { timeout: number("timeout"), workers: number("workers").and_then(|n: u32| u16::try_from(n).ok()) })
        }
    }
}

pub mod purba_reporting {
    use super::purba_configuration::{Dial, Dials as Declared, Refusal, Resolved, Rule};

    pub struct Dials {
        pub suppression_cap: Option<u32>,
    }

    impl Declared for Dials {
        const DIALS: &'static [Dial] = &[Dial { name: "suppression-cap", rule: Rule::Lower, changes_an_outcome: true, in_code: false }];

        fn from_resolved(resolved: &Resolved) -> Result<Self, Refusal> {
            Ok(Self { suppression_cap: resolved.get("suppression-cap").and_then(|set| set.value.parse().ok()) })
        }
    }
}

/// Command gathers each part's declaration, because it uses every part.
pub mod purba_command {
    use super::purba_configuration::{self as configuration, Dials as _, Level, Refusal};
    use super::{purba_dispatch, purba_reporting};

    pub fn read(levels: &[(Level, Vec<(String, String)>)]) -> Result<(purba_dispatch::Dials, purba_reporting::Dials), Refusal> {
        let resolved = configuration::read(&[purba_dispatch::Dials::DIALS, purba_reporting::Dials::DIALS], levels)?;
        Ok((purba_dispatch::Dials::from_resolved(&resolved)?, purba_reporting::Dials::from_resolved(&resolved)?))
    }
}
