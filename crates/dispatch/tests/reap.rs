//! Dispatch waits for the zygote of a module on each way out of it: after its
//! tests run, after a break at a test, and after a break at the error of a
//! load's thread.
//!
//! The check asks for any child of this process, so this file holds one test.
//! Cargo runs the tests of one file on threads of one process.

mod common;

use std::io;
use std::ops::ControlFlow;

use common::{Named, module};
use purba_dispatch::Handed;

/// A module whose load joins a thread that raises, so the load gives the
/// error of one thread.
const TEST_THREAD: &str = "import threading\n\n\
    loader = threading.Thread(target=lambda: 1 / 0, name=\"L\")\n\
    loader.start()\nloader.join()\n\n\
    def test_one():\n    assert True\n";

/// What `waitpid` gives when this process has no child: `-1`, and `ECHILD`.
const NO_CHILD: (i32, Option<i32>) = (-1, Some(libc::ECHILD));

/// Hands the module, and breaks at the first thing that `stop` picks. Gives
/// the flow that `hand` returns, and what `waitpid` gives right after it.
fn handed(stop: fn(&Handed<Named>) -> bool) -> (ControlFlow<()>, (i32, Option<i32>)) {
    let flow = purba_dispatch::hand(
        vec![module("test_thread.py", TEST_THREAD, &["test_one"])],
        |handed| {
            if stop(&handed) {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        },
    );
    // SAFETY: a null status pointer asks for no status, and `WNOHANG` does
    // not block.
    let child = unsafe { libc::waitpid(-1, std::ptr::null_mut(), libc::WNOHANG) };
    (flow, (child, io::Error::last_os_error().raw_os_error()))
}

#[test]
fn hand_leaves_no_child_on_each_way_out_of_a_module() {
    assert_eq!(
        handed(|_| false),
        (ControlFlow::Continue(()), NO_CHILD),
        "a module whose test runs leaves no child, alive or unreaped",
    );
    assert_eq!(
        handed(|handed| matches!(handed, Handed::Test(..))),
        (ControlFlow::Break(()), NO_CHILD),
        "a break at a test leaves no child, alive or unreaped",
    );
    assert_eq!(
        handed(|handed| matches!(handed, Handed::LoadThread(..))),
        (ControlFlow::Break(()), NO_CHILD),
        "a break at the error of a load's thread leaves no child, alive or unreaped",
    );
}
