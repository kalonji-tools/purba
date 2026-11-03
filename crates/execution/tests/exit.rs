use std::path::PathBuf;

use purba_execution::{Kind, Module, Rewrite};

fn module(name: &str, source: &str) -> Module {
    Module::new(
        PathBuf::from(name),
        Rewrite::of_source(source).expect("the source parses"),
    )
}

// The guard on `os._exit` keeps one record for the process, and cargo runs the
// tests of one binary on several threads. The `run` of another test could take
// the record of a call made here. So each call to `os._exit` in this process
// lives in this one test, alone in its binary.
#[test]
fn a_call_to_os_exit_gives_raised_however_the_test_makes_it() {
    let test_exit = module(
        "test_exit.py",
        "import os\nimport threading\n\n\
         def test_fails():\n    assert 1 == 2\n\n\
         def test_catches_exception():\n    try:\n        os._exit(3)\n\
         \x20   except Exception:\n        pass\n\n\
         def test_starts_a_thread():\n    thread = threading.Thread(target=os._exit, args=(1,))\n\
         \x20   thread.start()\n    thread.join()\n    assert True\n\n\
         def test_catches_base_exception():\n    try:\n        os._exit(2)\n\
         \x20   except BaseException:\n        pass\n    assert True\n\n\
         def test_passes():\n    assert 1 == 1\n",
    );
    let test_load_exit = module(
        "test_load_exit.py",
        "import os\nos._exit(0)\n\ndef test_one():\n    pass\n",
    );
    let raised = |call: &str| {
        Ok(Kind::Raised {
            exception: format!("ExitInsideATest: {call}"),
        })
    };
    let results: Vec<Result<Kind, String>> = purba_execution::attach(|worker| {
        [
            (&test_exit, "test_fails"),
            (&test_exit, "test_catches_exception"),
            (&test_exit, "test_starts_a_thread"),
            (&test_exit, "test_catches_base_exception"),
            (&test_exit, "test_passes"),
            (&test_load_exit, "test_one"),
            (&test_exit, "test_passes"),
        ]
        .iter()
        .map(|(module, function)| {
            worker
                .run(module, function)
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
            raised("os._exit(3)"),
            raised("os._exit(1)"),
            raised("os._exit(2)"),
            Ok(Kind::Passed),
            Err("ExitInsideATest: os._exit(0)".to_owned()),
            Ok(Kind::Passed),
        ],
        "each call in a test gives raised, whoever sees it, and the next test runs. \
         A call while the module loads fails the load, so no test's call runs and it gives \
         an error, and the next test does not get the call",
    );
}
