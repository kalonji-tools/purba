//! When the zygote that loads nothing ends before it is ready, dispatch hands
//! the module once, with its error and each of its tests.

mod common;

use std::path::PathBuf;
use std::process::Command;

use common::{Named, module};
use purba_dispatch::{Handed, Module};
use purba_execution::Error;

/// A module whose load leaves a thread alive. A zygote that loads nothing then
/// forks its tests. A thread that the load joins raises first, so only a stop
/// after the load carries its error.
const TEST_THREADED: &str = "import threading\n\n\
    loader = threading.Thread(target=lambda: 1 / 0, name=\"L\")\n\
    loader.start()\nloader.join()\n\n\
    threading.Thread(target=threading.Event().wait, daemon=True).start()\n\n\
    def test_one():\n    assert 1 == 1\n\n\
    def test_two():\n    assert 2 == 2\n";

/// The `sitecustomize` that the interpreter of the inner run imports as it
/// starts. Each zygote calls `atexit._clear` before it is ready. The first
/// process to call it writes its id in the marker. Any other process exits 7
/// there, so only the second zygote exits 7.
const SITE: &str = "import atexit\nimport os\n\nclear = atexit._clear\nmarker = {marker:?}\n\n\
    def clear_or_exit():\n    if not os.path.exists(marker):\n\
    \x20       with open(marker, 'w') as mark:\n            mark.write(str(os.getpid()))\n\
    \x20   elif open(marker).read() != str(os.getpid()):\n        os._exit(7)\n    clear()\n\n\
    atexit._clear = clear_or_exit\n";

/// What dispatch hands on, by the name of each test and the line of each error.
fn seen(module: Module<Named>) -> Vec<String> {
    let mut seen = Vec::new();
    let _ = purba_dispatch::hand(vec![module], |handed| {
        seen.push(match handed {
            Handed::Test(test, result, _) => {
                format!("{} {:?}", test.0, result.map_err(|err| err.line()).err())
            }
            Handed::Stopped {
                path,
                error,
                threads,
                tests,
            } => format!(
                "stopped {} {} {:?} {:?}",
                path.display(),
                error.line(),
                threads.iter().map(Error::line).collect::<Vec<_>>(),
                tests.iter().map(|test| test.0).collect::<Vec<_>>(),
            ),
            Handed::Ended {
                path,
                error,
                named,
                tests,
            } => format!(
                "ended {} {} {named} {:?}",
                path.display(),
                error.line(),
                tests.iter().map(|test| test.0).collect::<Vec<_>>(),
            ),
            Handed::TestThread(_, err) | Handed::LoadThread(_, err) => {
                format!("thread {}", err.line())
            }
            Handed::Unreached(test) => format!("unreached {}", test.0),
        });
        std::ops::ControlFlow::<()>::Continue(())
    });
    seen
}

// The interpreter reads `PYTHONPATH` once, as it starts. So the test runs its
// own binary again, with the `sitecustomize` on that path. The inner run checks
// what dispatch hands on.
#[test]
fn a_module_whose_zygote_that_loads_nothing_ends_before_it_is_ready_is_handed_once() {
    if std::env::var_os("PURBA_SECOND_ZYGOTE").is_some() {
        let handed = seen(module(
            "test_threaded_load.py",
            TEST_THREADED,
            &["test_one", "test_two"],
        ));
        assert_eq!(
            handed,
            [
                "stopped test_threaded_load.py project error: its process exited 7 before its \
                 load ended [\"project error: thread L raised ZeroDivisionError: division by \
                 zero\"] [\"test_one\", \"test_two\"]"
            ],
            "the module is handed once, with the second zygote's error, the error of the \
             load's thread and each of its tests",
        );
        return;
    }
    let site = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("second-zygote-{}", std::process::id()));
    let marker = site.join("cleared");
    let _ = std::fs::remove_file(&marker);
    std::fs::create_dir_all(&site).expect("the directory of the site is made");
    std::fs::write(
        site.join("sitecustomize.py"),
        SITE.replace(
            "{marker:?}",
            &format!("{:?}", marker.to_str().expect("UTF-8")),
        ),
    )
    .expect("the site is written");
    let inner = Command::new(std::env::current_exe().expect("the test knows its binary"))
        .args([
            "--exact",
            "a_module_whose_zygote_that_loads_nothing_ends_before_it_is_ready_is_handed_once",
            "--nocapture",
        ])
        .env("PURBA_SECOND_ZYGOTE", "1")
        .env("PYTHONPATH", &site)
        .output()
        .expect("the inner run starts");
    assert!(
        inner.status.success() && marker.exists(),
        "the inner run passes, and its first zygote cleared: {}",
        String::from_utf8_lossy(&inner.stderr),
    );
}
