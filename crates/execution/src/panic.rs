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

/// The part whose source holds `file`. Cargo names a file of this workspace
/// from its root, and a file of a dependency by its full path:
/// <https://github.com/rust-lang/cargo/blob/5f94df4/src/util/workspace.rs#L128-L146>
fn part(file: &str) -> Option<String> {
    const SEPARATORS: [char; 2] = ['/', '\\'];
    let (part, _) = file
        .strip_prefix("crates")?
        .strip_prefix(SEPARATORS)?
        .split_once(SEPARATORS)?;
    // The crate `purba` builds the extension module, and is not a part.
    (part != "purba").then(|| part.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_location_under_crates_names_its_part() {
        assert_eq!(
            part("crates/assertion/src/lib.rs"),
            Some("assertion".to_owned()),
            "a file of assertion"
        );
        assert_eq!(
            part("crates\\assertion\\src\\lib.rs"),
            Some("assertion".to_owned()),
            "a file of assertion, as Windows separates it"
        );
    }

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
}
