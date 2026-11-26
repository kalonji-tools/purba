//! The wrap of Rust's panic hook. It records the part and the message of each
//! panic, for the part that catches it.

use std::any::Any;
use std::panic::{self, PanicHookInfo};
use std::sync::{Mutex, Once, PoisonError};

/// The part and the message of a panic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Panic {
    /// The part whose source holds the panic. None when the panic is outside
    /// the source of a part.
    pub part: Option<String>,
    /// The text of the panic.
    pub message: String,
}

impl Panic {
    /// This panic, caught by `catcher`, which it names when it names no part.
    #[must_use]
    pub fn caught_by(self, catcher: &str) -> Caught {
        Caught {
            part: self.part.unwrap_or_else(|| catcher.to_owned()),
            message: self.message,
        }
    }
}

/// The part that found a panic, and its message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Caught {
    /// The part whose source holds the panic, or the part that caught it.
    pub part: String,
    /// The text of the panic.
    pub message: String,
}

/// What Rust's own hook writes for a payload that is not text:
/// <https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/panicking.rs#L760-L768>
const NOT_TEXT: &str = "Box<dyn Any>";

/// The last panic of the process, on any thread.
static LAST: Mutex<Option<Panic>> = Mutex::new(None);

/// Wraps Rust's panic hook, once in a process. The wrap records each panic,
/// then calls the hook it replaced, which writes its text to stderr.
pub fn wrap_hook() {
    static WRAP: Once = Once::new();
    WRAP.call_once(|| {
        let replaced = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            record(info);
            replaced(info);
        }));
    });
}

/// Hands back the last panic that the wrap recorded, and clears it.
#[must_use]
pub fn take() -> Option<Panic> {
    LAST.lock().unwrap_or_else(PoisonError::into_inner).take()
}

/// The panic whose payload `catcher` caught. The part is that of the note
/// when the note holds the same message, and `catcher` otherwise. Clears the
/// note either way.
#[must_use]
pub fn caught(payload: &(dyn Any + Send), catcher: &str) -> Caught {
    let message = text(payload);
    // A panic that `resume_unwind` starts never reaches the hook, so the note
    // can be that of an earlier panic, which a test caught:
    // https://doc.rust-lang.org/1.99.1/std/panic/fn.resume_unwind.html
    match take() {
        Some(panic) if panic.message == message => panic.caught_by(catcher),
        _ => Caught {
            part: catcher.to_owned(),
            message,
        },
    }
}

/// The text of a panic's payload, or what Rust's hook writes for one that is
/// not text.
fn text(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| NOT_TEXT.to_owned())
}

fn record(info: &PanicHookInfo<'_>) {
    let panic = Panic {
        part: info.location().and_then(|location| part(location.file())),
        message: info.payload_as_str().unwrap_or(NOT_TEXT).to_owned(),
    };
    *LAST.lock().unwrap_or_else(PoisonError::into_inner) = Some(panic);
}

/// The parts, as "The parts and their dependencies" in `LANDSCAPE.md` names
/// them.
const PARTS: [&str; 8] = [
    "command",
    "selection",
    "parsing",
    "dispatch",
    "execution",
    "fixtures",
    "assertion",
    "reporting",
];

/// The part whose source holds `file`. Cargo names a file of this workspace
/// from its root, and a file of a dependency by its full path:
/// <https://github.com/rust-lang/cargo/blob/5f94df4/src/util/workspace.rs#L128-L146>
fn part(file: &str) -> Option<String> {
    let (part, _) = file.strip_prefix("crates/")?.split_once('/')?;
    PARTS.contains(&part).then(|| part.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_location_outside_the_source_of_a_part_names_no_part() {
        assert_eq!(
            part("crates/purba/src/lib.rs"),
            None,
            "the crate purba holds no part"
        );
        assert_eq!(
            part(
                "/home/u/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/\
                 ruff_source_file-0.0.17/src/line_index.rs"
            ),
            None,
            "a dependency"
        );
        assert_eq!(
            part(
                "/rustc/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/\
                 library/alloc/src/raw_vec/mod.rs"
            ),
            None,
            "the standard library"
        );
    }

    #[test]
    fn a_location_in_the_crate_of_a_concern_names_no_part() {
        assert_eq!(
            part("crates/errors/src/lib.rs"),
            None,
            "the crate purba-errors holds no part"
        );
        assert_eq!(
            part("crates/config/src/lib.rs"),
            None,
            "the crate purba-config holds no part"
        );
    }

    #[test]
    fn each_part_names_itself() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../LANDSCAPE.md");
        let landscape =
            std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{path}: {error}"));
        let mut parts: Vec<&str> = landscape
            .lines()
            .skip_while(|line| *line != "## The parts and their dependencies")
            .skip(1)
            .take_while(|line| !line.starts_with("## "))
            .filter_map(|line| line.strip_prefix("- **")?.split_once(".**"))
            .map(|(name, _)| name)
            .collect();
        let mut held = PARTS;
        parts.sort_unstable();
        held.sort_unstable();

        assert_eq!(parts, held, "{path} names the parts that PARTS holds");
        for name in PARTS {
            assert_eq!(
                part(&format!("crates/{name}/src/lib.rs")),
                Some(name.to_owned()),
                "{name} is a part"
            );
        }
    }

    #[test]
    fn a_location_in_a_crate_that_is_not_a_part_names_no_part() {
        assert_eq!(
            part("crates/newcrate/src/lib.rs"),
            None,
            "a crate that the list of the parts leaves out"
        );
        for name in ["", "select", "assertions"] {
            assert_eq!(
                part(&format!("crates/{name}/src/lib.rs")),
                None,
                "the name {name:?} is near a part or empty"
            );
        }
        // A list of the crates that are not parts can hold each name that a
        // test writes out, so these names are generated.
        let letters = || ('a'..='z').map(String::from);
        let names = letters().chain(
            letters().flat_map(|first| letters().map(move |second| format!("{first}{second}"))),
        );
        for name in names {
            assert_eq!(
                part(&format!("crates/{name}/src/lib.rs")),
                None,
                "the name {name:?} is no part"
            );
        }
    }

    #[test]
    fn a_panic_outside_the_parts_names_its_catcher() {
        let panic = Panic {
            part: part("crates/config/src/lib.rs"),
            message: "boom".to_owned(),
        };
        assert_eq!(
            panic.caught_by("dispatch"),
            Caught {
                part: "dispatch".to_owned(),
                message: "boom".to_owned(),
            },
            "a panic in purba-config names the part that caught it"
        );
    }
}
