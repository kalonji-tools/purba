//! A SIGTERM does what a SIGINT does, and after the first interrupt each
//! interrupt signal keeps a handler of the operating system that ends the
//! process at the second.

mod common;

use purba_execution::{Kind, Worker};

/// A module whose test takes a SIGTERM, then gives SIGTERM its own signal
/// handler. As its process ends, the test marks one of three answers for each
/// of SIGINT, SIGTERM and SIGHUP. The operating system gives it its default
/// action, ignores it, or runs a handler.
fn test_term(marker: &str) -> String {
    format!(
        "import atexit\nimport ctypes\nimport signal\n\n\
         def disposition(signum):\n    action = ctypes.create_string_buffer(256)\n    \
         assert ctypes.CDLL(None).sigaction(signum, None, action) == 0\n    \
         handler = ctypes.c_void_p.from_buffer(action).value or 0\n    \
         return {{0: 'default', 1: 'ignored'}}.get(handler, 'handled')\n\n\
         def mark():\n    with open({marker:?}, 'w') as marker:\n        \
         marker.write(' '.join(disposition(s) \
         for s in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)))\n\n\
         def test_term():\n    atexit.register(mark)\n    try:\n        \
         signal.raise_signal(signal.SIGTERM)\n    \
         except KeyboardInterrupt:\n        \
         signal.signal(signal.SIGTERM, lambda signum, frame: None)\n    assert True\n"
    )
}

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
    let marker = std::env::temp_dir().join(format!("purba-term-{}", std::process::id()));
    let module = common::module(
        "test_term.py",
        &test_term(marker.to_str().expect("the marker's path is text")),
    );
    // A signal that purba's parent ignores stays ignored.
    let before = SIGNALS.map(disposition);
    let kind = purba_execution::attach(|worker: &mut Worker<'_, &str>| {
        purba_execution::set_hooks().expect("purba's signal handler takes each signal");
        let (mut zygote, _) = worker.zygote(&module).expect("the zygote loads the module");
        let (result, _, _) = zygote.run("test_term", &[], "test_term");
        zygote.end();
        result
            .map(|outcome| outcome.kind().clone())
            .map_err(|error| error.to_string())
    });
    let after = std::fs::read_to_string(&marker).expect("test_term marked the dispositions");
    std::fs::remove_file(&marker).expect("the marker goes");
    assert_eq!(
        kind,
        Ok(Kind::Interrupted),
        "a SIGTERM in a call makes the test interrupted, as a SIGINT does"
    );
    let expected = before
        .map(|was| {
            if was == "ignored" {
                "ignored"
            } else {
                "handled"
            }
        })
        .join(" ");
    assert_eq!(
        after, expected,
        "after the first interrupt, SIGINT, SIGTERM and SIGHUP each keep purba's native \
         handler, unless purba's parent ignores it. SIGTERM keeps it though the test gave \
         SIGTERM its own signal handler after the first interrupt."
    );
}
