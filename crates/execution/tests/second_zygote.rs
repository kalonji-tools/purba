//! A zygote that loads nothing and ends before it is ready stops its module,
//! as a zygote that ends before its load ends does.

mod common;

use std::ffi::CString;

use purba_execution::Worker;
use pyo3::Python;

/// A module whose load leaves a thread alive, so a zygote that loads nothing
/// forks its tests.
const TEST_THREADED: &str = "import threading\n\n\
    threading.Thread(target=threading.Event().wait, daemon=True).start()\n\n\
    def test_one():\n    assert 1 == 1\n\n\
    def test_two():\n    assert 2 == 2\n";

// Each zygote calls `atexit._clear` before it is ready. This test puts a
// function in its place for the life of the process, so it lives alone in its
// binary.
#[test]
fn a_zygote_that_loads_nothing_and_ends_before_it_is_ready_stops_its_module() {
    let first = std::env::temp_dir().join(format!("purba-second-{}", std::process::id()));
    let module = common::module("test_threaded_load.py", TEST_THREADED);
    let stopped = purba_execution::attach(|worker: &mut Worker<'_, &str>| {
        Python::attach(|py| {
            let code = CString::new(format!(
                "import atexit\nimport os\n\nclear = atexit._clear\n\n\
                 def clear_or_exit():\n    if os.path.exists({first:?}):\n        \
                 os._exit(7)\n    open({first:?}, 'w').close()\n    clear()\n\n\
                 atexit._clear = clear_or_exit\n"
            ))
            .expect("the code holds no NUL");
            py.run(&code, None, None)
                .expect("the second zygote exits 7 before it is ready");
        });
        match worker.zygote(&module) {
            Ok((zygote, _)) => {
                zygote.end();
                None
            }
            Err(stop) => Some(stop.error.line()),
        }
    });
    std::fs::remove_file(&first).expect("the first zygote marked that it cleared");
    assert_eq!(
        stopped.as_deref(),
        Some("project error: its process exited 7 before its load ended"),
        "a zygote that loads nothing and exits before it is ready stops its module with \
         its exit code, as the first zygote does"
    );
}
