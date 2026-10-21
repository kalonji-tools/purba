use std::path::PathBuf;

use purba_dispatch::{Module, Test};
use purba_execution::Kind;

struct Named(&'static str);

impl Test for Named {
    fn function(&self) -> &str {
        self.0
    }
}

fn module(name: &str, text: &str, asserts: &[&str], tests: &[&'static str]) -> Module<Named> {
    Module::new(
        PathBuf::from(name),
        text.to_owned(),
        asserts.iter().map(|&assert| assert.to_owned()).collect(),
        tests.iter().map(|&test| Named(test)).collect(),
    )
}

fn seen(module: Module<Named>) -> Vec<(&'static str, Result<Kind, String>)> {
    purba_dispatch::hand(vec![module])
        .into_iter()
        .map(|(test, result)| {
            (
                test.0,
                result
                    .map(|outcome| outcome.kind().clone())
                    .map_err(|err| err.to_string()),
            )
        })
        .collect()
}

#[test]
fn each_outcome_stands_beside_its_test() {
    let test_a = module(
        "test_a.py",
        "def test_one():\n    assert _purba_ran(0, 1 == 1)\n\n\
         def test_two():\n    assert _purba_ran(1, 1 == 2)\n\n\
         def test_three():\n    pass\n",
        &["assert 1 == 1", "assert 1 == 2"],
        &["test_one", "test_two", "test_three"],
    );
    assert_eq!(
        seen(test_a),
        [
            ("test_one", Ok(Kind::Passed)),
            (
                "test_two",
                Ok(Kind::Failed {
                    assert: "assert 1 == 2".to_owned()
                })
            ),
            ("test_three", Ok(Kind::Unknown)),
        ],
        "each outcome beside its test, in order",
    );
}

#[test]
fn dispatch_calls_only_the_tests_it_is_handed() {
    let test_only = module(
        "test_only.py",
        "ran = []\n\ndef test_left_out():\n    ran.append(1)\n\n\
         def test_check():\n    assert _purba_ran(0, ran == [])\n",
        &["assert ran == []"],
        &["test_check"],
    );
    assert_eq!(
        seen(test_only),
        [("test_check", Ok(Kind::Passed))],
        "the left-out test never ran",
    );
}

#[test]
fn the_tests_of_a_module_share_one_worker() {
    let test_shared = module(
        "test_shared.py",
        "seen = []\n\n\
         def test_first():\n    seen.append(1)\n    assert _purba_ran(0, seen == [1])\n\n\
         def test_second():\n    seen.append(2)\n    assert _purba_ran(1, seen == [1, 2])\n",
        &["assert seen == [1]", "assert seen == [1, 2]"],
        &["test_first", "test_second"],
    );
    assert_eq!(
        seen(test_shared),
        [
            ("test_first", Ok(Kind::Passed)),
            ("test_second", Ok(Kind::Passed))
        ],
        "the module loaded once for both tests",
    );
}

#[test]
fn an_error_stands_beside_its_test_and_the_run_goes_on() {
    let test_err = module(
        "test_err.py",
        "def test_one():\n    raise ValueError('boom')\n\n\
         def test_two():\n    assert _purba_ran(0, True)\n",
        &["assert True"],
        &["test_one", "test_two"],
    );
    assert_eq!(
        seen(test_err),
        [
            ("test_one", Err("ValueError: boom".to_owned())),
            ("test_two", Ok(Kind::Passed)),
        ],
        "the error beside its test, and the next test runs",
    );
}
