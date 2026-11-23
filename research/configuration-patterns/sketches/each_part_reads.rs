//! Home 4: each part reads its own values.
//!
//! A sketch. It decides nothing. Each `mod` stands for a crate, and each `use`
//! between two mods stands for an entry in a manifest. Files arrive already
//! parsed, as `(key, value)` pairs, so the sketch needs only std.
//! It copies cargo-nextest, whose project file and user file each have their
//! own reader and their own type, and cargo's consumers that read their own key.

#![allow(dead_code)]

pub mod purba_dispatch {
    use std::path::PathBuf;

    /// Dispatch's own copy of the order of the levels.
    pub enum Level {
        CommandLine,
        Environment(String),
        Code { line: usize },
        Project(PathBuf),
        User(PathBuf),
        System(PathBuf),
    }

    pub struct Dials {
        pub timeout: Option<(u32, Level)>,
        pub workers: Option<(u16, Level)>,
    }

    /// Every key of every part. Rule 6 needs it, so each part keeps a copy,
    /// and a test that reaches every part ties the copies.
    pub const KNOWN: [&str; 3] = ["timeout", "workers", "suppression-cap"];

    pub fn read(arguments: &[String], environment: &[(String, String)], project: &[(String, String)]) -> Result<Dials, String> {
        let _ = (arguments, environment);
        if let Some((key, _)) = project.iter().find(|(key, _)| !KNOWN.contains(&key.as_str())) {
            return Err(format!("unknown key `{key}`"));
        }
        todo!("find the files, read the levels, apply the order, keep only timeout and workers")
    }
}

pub mod purba_reporting {
    /// Reporting's own copy of the same list.
    pub const KNOWN: [&str; 3] = ["timeout", "workers", "suppression-cap"];

    pub struct Dials {
        pub suppression_cap: Option<u32>,
    }

    pub fn read(arguments: &[String], environment: &[(String, String)], project: &[(String, String)]) -> Result<Dials, String> {
        let _ = (arguments, environment, project);
        todo!("the same walk and order again, keeping only suppression-cap, lower value applies")
    }
}
