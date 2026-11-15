//! The readers of a case: a process that the case's process forks for each
//! expectation after the first, before any reading, to read that expectation
//! alone. A reader ends with its verdict as its exit code.

use std::io;

use purba_assertion::{Readers, Verdict};
use pyo3::Python;

use crate::fork::{Forked, fork};
use crate::zygote;

/// The readers that execution forks.
pub struct Forks;

pub static READERS: Forks = Forks;

const HOLDS: libc::c_int = 0;
const FAILS: libc::c_int = 1;
const RAISES: libc::c_int = 2;

impl Readers for Forks {
    fn threaded(&self, py: Python<'_>) -> bool {
        zygote::threaded(py)
    }

    fn fork(&self, py: Python<'_>) -> io::Result<Option<i32>> {
        zygote::flush(py);
        Ok(match fork(py)? {
            Forked::Child => None,
            Forked::Parent(pid) => Some(pid),
        })
    }

    /// A reader that a signal ends gives `Raises`, as does any exit code but
    /// the three of a verdict.
    fn wait(&self, pid: i32) -> Verdict {
        let status = zygote::wait(pid);
        if !libc::WIFEXITED(status) {
            return Verdict::Raises;
        }
        match libc::WEXITSTATUS(status) {
            HOLDS => Verdict::Holds,
            FAILS => Verdict::Fails,
            _ => Verdict::Raises,
        }
    }

    fn end(&self, verdict: Verdict) -> ! {
        let code = match verdict {
            Verdict::Holds => HOLDS,
            Verdict::Fails => FAILS,
            Verdict::Raises => RAISES,
        };
        // SAFETY: `_exit` takes any code, and ends the process with no
        // `finally` block, `atexit` function or wait for threads.
        unsafe { libc::_exit(code) }
    }
}
