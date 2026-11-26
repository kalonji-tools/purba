//! A signal that waits as a load starts stops that load.

mod common;

use std::ffi::CString;

use purba_execution::{Purba, Started};
use pyo3::Python;

/// Makes the zygote's last call of Python before its load send SIGINT, by C's
/// `raise`, in place of `atexit._clear`. The call runs no line of Python.
/// Python's own C handler only marks the signal for later. So purba's signal
/// handler runs at the next line of Python, which is inside the load's window.
const SIGINT_BEFORE_LOAD: &str = "import atexit\nimport ctypes\nimport functools\n\n\
    atexit._clear = functools.partial(ctypes.CDLL(None)['raise'], 2)\n";

// purba's signal handler keeps the first signal for the life of the process.
// So the one signal of this binary lives in this one test.
#[test]
fn a_sigint_that_waits_as_a_load_starts_stops_the_load_and_no_line_of_the_module_runs() {
    let marker = std::env::temp_dir().join(format!("purba-waiting-{}", std::process::id()));
    let module = common::module(
        "test_waiting.py",
        &format!("open({marker:?}, 'w').close()\n\ndef test_one():\n    assert True\n"),
    );
    let (loaded, kept) = purba_execution::attach(|purba: &Purba<'_>| {
        purba_execution::set_hooks().expect("purba's signal handler takes SIGINT");
        Python::attach(|py| {
            let code = CString::new(SIGINT_BEFORE_LOAD).expect("the code holds no NUL");
            py.run(&code, None, None)
                .expect("the zygote's last call before its load sends SIGINT");
        });
        let loaded = match purba.zygote(&module) {
            Started::Loaded(..) => Ok(()),
            Started::Stopped(stop) => Err(stop.error.to_string()),
            Started::Unreached => Err("the module is unreached".to_owned()),
        };
        (loaded, purba_execution::interrupted())
    });
    let ran = marker.exists();
    if ran {
        std::fs::remove_file(&marker).expect("the marker goes");
    }
    assert_eq!(
        (loaded, ran, kept),
        (Ok(()), false, true),
        "a SIGINT that waits as a load starts stops the load before the module's first line, \
         gives no error, and purba keeps it"
    );
}
