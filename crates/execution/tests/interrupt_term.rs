//! A SIGTERM does what a SIGINT does, and the first interrupt gives each
//! interrupt signal its default action.

use std::path::PathBuf;

use purba_execution::{Kind, Module, Rewrite};

const TEST_TERM: &str = "import signal\n\n\
    def test_term():\n    try:\n        signal.raise_signal(signal.SIGTERM)\n    \
    except KeyboardInterrupt:\n        \
    signal.signal(signal.SIGTERM, lambda signum, frame: None)\n    assert True\n";

const SIGNALS: [libc::c_int; 3] = [libc::SIGINT, libc::SIGTERM, libc::SIGHUP];

/// Whether the operating system gives `signum` its default action, or ignores it.
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
fn a_sigterm_in_a_call_makes_the_test_interrupted_and_gives_each_signal_its_default_action() {
    let module = Module::new(
        PathBuf::from("test_term.py"),
        Rewrite::of_source(TEST_TERM).expect("the source parses"),
    );
    // A signal that purba's parent ignores stays ignored.
    let before = SIGNALS.map(disposition);
    let kind = purba_execution::attach(|worker| {
        purba_execution::interrupt::install().expect("purba's signal handler takes each signal");
        worker.load(&module).expect("the module loads");
        worker
            .run(&module, "test_term", "test_term")
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
            "default"
        }
    });
    assert_eq!(
        SIGNALS.map(disposition),
        expected,
        "after the first interrupt, SIGINT, SIGTERM and SIGHUP each have their default \
         action, unless purba's parent ignores it. SIGTERM has it though the test gave \
         SIGTERM its own signal handler after the first interrupt."
    );
}
