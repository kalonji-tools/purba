//! A zygote that loads nothing and ends before it is ready stops its module,
//! as a zygote that ends before its load ends does.

mod common;

use std::ffi::CString;

use purba_errors::Error;
use purba_execution::Purba;
use pyo3::Python;

/// A module whose load leaves a thread alive. A zygote that loads nothing then
/// forks its tests. A thread that the load joins raises first.
const TEST_THREADED: &str = "import threading\n\n\
    loader = threading.Thread(target=lambda: 1 / 0, name=\"L\")\n\
    loader.start()\nloader.join()\n\n\
    threading.Thread(target=threading.Event().wait, daemon=True).start()\n\n\
    def test_one():\n    assert 1 == 1\n\n\
    def test_two():\n    assert 2 == 2\n";

// Each zygote calls `atexit._clear` before it is ready. This test puts a
// function in its place for the life of the process, so it lives alone in its
// binary. The first process to call it writes its id in the marker. Any other
// process exits 7 there, so only the second zygote exits 7.
#[test]
fn a_zygote_that_loads_nothing_and_ends_before_it_is_ready_stops_its_module() {
    let first = std::env::temp_dir().join(format!("purba-second-{}", std::process::id()));
    let _ = std::fs::remove_file(&first);
    let module = common::module("test_threaded_load.py", TEST_THREADED);
    let stopped = purba_execution::attach(|purba: &Purba<'_>| {
        Python::attach(|py| {
            let code = CString::new(format!(
                "import atexit\nimport os\n\nclear = atexit._clear\n\n\
                 def clear_or_exit():\n    if not os.path.exists({first:?}):\n        \
                 with open({first:?}, 'w') as mark:\n            \
                 mark.write(str(os.getpid()))\n    \
                 elif open({first:?}).read() != str(os.getpid()):\n        \
                 os._exit(7)\n    clear()\n\n\
                 atexit._clear = clear_or_exit\n"
            ))
            .expect("the code holds no NUL");
            py.run(&code, None, None)
                .expect("the second zygote exits 7 before it is ready");
        });
        match purba.zygote(&module) {
            Ok((zygote, _)) => {
                zygote.end();
                None
            }
            Err(stop) => Some((
                stop.error.line(),
                stop.threads.iter().map(Error::line).collect::<Vec<_>>(),
            )),
        }
    });
    std::fs::remove_file(&first).expect("the first zygote marked that it cleared");
    assert_eq!(
        stopped,
        Some((
            "project error: its process exited 7 before its load ended".to_owned(),
            vec!["project error: thread L raised ZeroDivisionError: division by zero".to_owned()],
        )),
        "a zygote that loads nothing and exits before it is ready stops its module with \
         its exit code, as the first zygote does, and keeps the error of each thread of \
         the load"
    );
}
