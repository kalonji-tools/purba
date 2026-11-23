//! Assertion: rewrites the asserts of a module, and loads the rewritten module.
//! An assert at the end of a test's body, or at the end of the body of a `with`
//! just before such asserts, is an expectation. A tuple assert is never an
//! expectation. An expectation becomes a reading that execution compares.
//! Every other assert, except a tuple assert, keeps a mark that records that it
//! ran, and whether it held. A step follows each statement of a test before its first expectation,
//! and each item of each `with` of a test is wrapped, so that the case sees its
//! hooks and its `with`s.

mod load;
mod rewrite;

pub use load::{
    Loaded, LoadedTest, Readers, Readings, Replaced, Swallowed, Verdict, Watched, load,
};
pub use rewrite::{Rewrite, Try, rewrite};

/// The name that each marked assert calls.
const RAN: &str = "_purba_ran";
/// The name that each expectation calls with its reading.
const READ: &str = "_purba_read";
/// The name that each expectation calls first, to learn whether it reads.
const READS: &str = "_purba_reads";
/// The name that a test calls after each statement before its first
/// expectation, with the line of the statement.
const STEP: &str = "_purba_step";
/// The name that wraps each item of each `with` of a test, with the line of
/// the `with`.
const WITH: &str = "_purba_with";
/// The name that the one statement of a `try` of one path calls when it did
/// not raise, with the index of the `try`.
const MISSED: &str = "_purba_missed";

/// Panics in a build with the feature `fault`, when `PURBA_FAULT` is
/// `setting`. The faults `rewrite` and `assert` panic here.
fn fault(setting: &str) {
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|value| value == setting)
    {
        panic!("PURBA_FAULT={setting}");
    }
}
