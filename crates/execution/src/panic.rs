//! The wrap of Rust's panic hook. It records the part and the message of each
//! panic, for the part that catches it.

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

fn record(info: &PanicHookInfo<'_>) {
    let panic = Panic {
        part: info.location().and_then(|location| part(location.file())),
        // Rust's own hook writes this for a payload that is not text:
        // https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/panicking.rs#L760-L768
        message: info.payload_as_str().unwrap_or("Box<dyn Any>").to_owned(),
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
