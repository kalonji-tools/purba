use std::path::PathBuf;

use purba_execution::{Kind, Module};

// The guard on `os._exit` keeps one record for the process, and cargo runs the
// tests of one binary on several threads. The `run` of another test could take
// the record of a call made here. So each call to `os._exit` in this process
// lives in this one test, alone in its binary.
#[test]
fn a_call_to_os_exit_gives_an_error_however_the_test_makes_it() {
    let test_exit = Module::new(
        PathBuf::from("test_exit.py"),
        "import os\nimport threading\n\n\
         def test_fails():\n    assert _purba_ran(0, 1 == 2)\n\n\
         def test_catches_exception():\n    try:\n        os._exit(3)\n\
         \x20   except Exception:\n        pass\n\n\
         def test_starts_a_thread():\n    thread = threading.Thread(target=os._exit, args=(1,))\n\
         \x20   thread.start()\n    thread.join()\n    assert _purba_ran(1, True)\n\n\
         def test_catches_base_exception():\n    try:\n        os._exit(2)\n\
         \x20   except BaseException:\n        pass\n    assert _purba_ran(2, True)\n\n\
         def test_passes():\n    assert _purba_ran(3, 1 == 1)\n"
            .to_owned(),
        [
            "assert 1 == 2",
            "assert True",
            "assert True",
            "assert 1 == 1",
        ]
        .map(str::to_owned)
        .to_vec(),
    );
    let results: Vec<Result<Kind, String>> = purba_execution::attach(|worker| {
        [
            "test_fails",
            "test_catches_exception",
            "test_starts_a_thread",
            "test_catches_base_exception",
            "test_passes",
        ]
        .iter()
        .map(|function| {
            worker
                .run(&test_exit, function)
                .map(|outcome| outcome.kind().clone())
                .map_err(|error| error.to_string())
        })
        .collect()
    });
    assert_eq!(
        results,
        [
            Ok(Kind::Failed {
                assert: "assert 1 == 2".to_owned()
            }),
            Err("ExitInsideATest: os._exit(3)".to_owned()),
            Err("ExitInsideATest: os._exit(1)".to_owned()),
            Err("ExitInsideATest: os._exit(2)".to_owned()),
            Ok(Kind::Passed),
        ],
        "each call is an error, whoever sees it, and the next test runs",
    );
}
