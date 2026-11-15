//! The readers of a case: a process that the case's process forks for each
//! expectation after the first, before any reading, to read that expectation
//! alone. A reader ends with its verdict as its exit code. The steps of a case
//! also ask here whether it replaced a hook, and the first expectation stops the
//! watch of its paths.

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

    fn wait(&self, pid: i32) -> Verdict {
        verdict(zygote::wait(pid))
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

    fn replaced(&self, py: Python<'_>) -> Option<&'static str> {
        crate::thread::replaced(py)
    }

    fn expecting(&self, _py: Python<'_>) {
        crate::paths::stop();
    }
}

/// The verdict of a reader or a replay that ended with the wait status
/// `status`. One that a signal ends gives `Raises`, as does any exit code but
/// the three of a verdict.
pub const fn verdict(status: libc::c_int) -> Verdict {
    if !libc::WIFEXITED(status) {
        return Verdict::Raises;
    }
    match libc::WEXITSTATUS(status) {
        HOLDS => Verdict::Holds,
        FAILS => Verdict::Fails,
        _ => Verdict::Raises,
    }
}
