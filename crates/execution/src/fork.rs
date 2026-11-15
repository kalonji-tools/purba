//! purba's own fork, which forks a zygote or a test's process. Python's hooks
//! run around it, as they run around `os.fork`, but purba's signal handlers stay
//! in the child.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

use pyo3::Python;

/// Whether purba's own fork runs now. The child inherits it until the hooks
/// that run after the fork return.
static OWN: AtomicBool = AtomicBool::new(false);

/// The side of a fork that a process is on.
pub enum Forked {
    Child,
    Parent(libc::pid_t),
}

/// Forks the process. The fork runs the hooks of `os.register_at_fork`, and the
/// child reinitializes Python's threads and locks, as `os.fork` does:
/// <https://github.com/python/cpython/blob/v3.15.2/Modules/posixmodule.c#L8707-L8727>
///
/// # Errors
///
/// The operating system refused the fork.
pub fn fork(_py: Python<'_>) -> io::Result<Forked> {
    OWN.store(true, Ordering::SeqCst);
    // SAFETY: the caller holds the GIL, which `PyOS_BeforeFork` needs.
    unsafe { pyo3::ffi::PyOS_BeforeFork() };
    // SAFETY: the child runs only Python and purba's own code, which Python's
    // hooks after the fork made safe to run.
    let pid = unsafe { libc::fork() };
    if pid == 0 {
        // SAFETY: this is the child of the fork that `PyOS_BeforeFork` opened.
        unsafe { pyo3::ffi::PyOS_AfterFork_Child() };
        OWN.store(false, Ordering::SeqCst);
        return Ok(Forked::Child);
    }
    let error = io::Error::last_os_error();
    // SAFETY: this is the parent of the fork that `PyOS_BeforeFork` opened.
    unsafe { pyo3::ffi::PyOS_AfterFork_Parent() };
    OWN.store(false, Ordering::SeqCst);
    if pid < 0 {
        Err(error)
    } else {
        Ok(Forked::Parent(pid))
    }
}

/// Whether purba's own fork runs now.
pub fn own() -> bool {
    OWN.load(Ordering::SeqCst)
}
