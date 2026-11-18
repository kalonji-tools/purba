//! A SIGTERM does what a SIGINT does, and after the first interrupt each
//! interrupt signal keeps a handler of the operating system that ends the
//! process at the second.

use std::path::PathBuf;

use purba_execution::{Kind, Module, Rewrite};

const TEST_TERM: &str = "import signal\n\n\
    def test_term():\n    try:\n        signal.raise_signal(signal.SIGTERM)\n    \
    except KeyboardInterrupt:\n        \
    signal.signal(signal.SIGTERM, lambda signum, frame: None)\n    assert True\n";

const SIGNALS: [libc::c_int; 3] = [libc::SIGINT, libc::SIGTERM, libc::SIGHUP];

/// Whether the operating system gives `signum` its default action, ignores it,
/// or runs a handler.
fn disposition(signum: libc::c_int) -> &'static str {
    // SAFETY: a null new action only reads the current one.
    let current = unsafe {
        let mut current: libc::sigaction = std::mem::zeroed();
        assert_eq!(
            libc::sigaction(signum, std::ptr::null(), &raw mut current),
            0,
            "sigaction reads the action of signal {signum}"
        );
        current.sa_sigaction
    };
    match current {
        libc::SIG_DFL => "default",
        libc::SIG_IGN => "ignored",
        _ => "handled",
    }
}

// purba's signal handler keeps the first signal for the life of the process,
// and a second signal ends the process. So this test lives alone in its
// binary.
#[test]
fn a_sigterm_in_a_call_makes_the_test_interrupted_and_each_signal_keeps_a_handler() {
    let module = Module::new(
        PathBuf::from("test_term.py"),
        Rewrite::of_source(TEST_TERM, &tests_in(TEST_TERM)).expect("the source parses"),
    );
    // A signal that purba's parent ignores stays ignored.
    let before = SIGNALS.map(disposition);
    let kind = purba_execution::attach(|worker| {
        purba_execution::set_hooks().expect("purba's signal handler takes each signal");
        worker.load(&module).expect("the module loads");
        worker
            .run(&module, "test_term", &[], "test_term")
            .map(|outcome| outcome.kind().clone())
            .map_err(|error| error.to_string())
    });
    assert_eq!(
        kind,
        Ok(Kind::Interrupted),
        "a SIGTERM in a call makes the test interrupted, as a SIGINT does"
    );
    let expected = before.map(|was| {
        if was == "ignored" {
            "ignored"
        } else {
            "handled"
        }
    });
    assert_eq!(
        SIGNALS.map(disposition),
        expected,
        "after the first interrupt, SIGINT, SIGTERM and SIGHUP each keep purba's native \
         handler, unless purba's parent ignores it. SIGTERM keeps it though the test gave \
         SIGTERM its own signal handler after the first interrupt."
    );
}

/// The name of each `def test_…` at the top of `source`, as selection names
/// the tests of a module.
fn tests_in(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| line.strip_prefix("def test_").map(|_| &line[4..]))
        .filter_map(|line| line.split_once('(').map(|(name, _)| name))
        .collect()
}
