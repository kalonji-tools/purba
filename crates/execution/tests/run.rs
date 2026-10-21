use std::path::PathBuf;

use purba_execution::{Kind, Module, Outcome};

fn module(name: &str, text: &str, asserts: &[&str]) -> Module {
    Module::new(
        PathBuf::from(name),
        text.to_owned(),
        asserts.iter().map(|&assert| assert.to_owned()).collect(),
    )
}

fn kinds(module: &Module, functions: &[&str]) -> Vec<Kind> {
    purba_execution::attach(|worker| {
        functions
            .iter()
            .map(|function| {
                worker
                    .run(module, function)
                    .expect("the test runs")
                    .kind()
                    .clone()
            })
            .collect()
    })
}

#[test]
fn the_three_tests_of_test_a_pass_fail_and_are_unknown() {
    let test_a = module(
        "test_a.py",
        "def test_one():\n    assert _purba_ran(0, 1 == 1)\n\n\
         def test_two():\n    assert _purba_ran(1, 1 == 2)\n\n\
         def test_three():\n    pass\n",
        &["assert 1 == 1", "assert 1 == 2"],
    );
    assert_eq!(
        kinds(&test_a, &["test_one", "test_two", "test_three"]),
        [
            Kind::Passed,
            Kind::Failed {
                assert: "assert 1 == 2".to_owned()
            },
            Kind::Unknown,
        ],
        "one outcome for each test, in order",
    );
}

#[test]
fn a_module_level_assert_leaves_a_test_unknown() {
    let test_level = module(
        "test_level.py",
        "assert _purba_ran(0, True)\n\ndef test_none():\n    pass\n",
        &["assert True"],
    );
    assert_eq!(
        kinds(&test_level, &["test_none"]),
        [Kind::Unknown],
        "execution clears the index of the module's assert before the test",
    );
}

#[test]
fn a_worker_loads_each_module_once() {
    let test_once = module(
        "test_once.py",
        "calls = []\n\n\
         def test_first():\n    calls.append(1)\n    assert _purba_ran(0, calls == [1])\n\n\
         def test_second():\n    calls.append(2)\n    assert _purba_ran(1, calls == [1, 2])\n",
        &["assert calls == [1]", "assert calls == [1, 2]"],
    );
    assert_eq!(
        kinds(&test_once, &["test_first", "test_second"]),
        [Kind::Passed, Kind::Passed],
        "the second test sees the list that the first test changed",
    );
}

#[test]
fn another_exception_is_an_error() {
    let test_raises = module(
        "test_raises.py",
        "def test_raises():\n    raise ValueError('no')\n",
        &[],
    );
    let error = purba_execution::attach(|worker| worker.run(&test_raises, "test_raises"))
        .expect_err("ValueError is no outcome");
    assert_eq!(
        error.to_string(),
        "ValueError: no",
        "Python's error as text"
    );
}

#[test]
fn an_outcome_is_data() {
    fn data<T: serde::Serialize + serde::de::DeserializeOwned>() {}
    data::<Outcome>();
}

#[test]
fn a_module_that_fails_to_load_runs_its_body_once() {
    let loads = module(
        "test_fails.py",
        "import builtins\nbuiltins.purba_loads = getattr(builtins, 'purba_loads', 0) + 1\n\
         raise ValueError(f'load {builtins.purba_loads}')\n\n\
         def test_one():\n    pass\n\ndef test_two():\n    pass\n",
        &[],
    );
    let errors: Vec<String> = purba_execution::attach(|worker| {
        ["test_one", "test_two"]
            .iter()
            .map(|function| {
                worker
                    .run(&loads, function)
                    .expect_err("the module does not load")
                    .to_string()
            })
            .collect()
    });
    assert_eq!(
        errors,
        ["ValueError: load 1", "test_fails.py did not load"],
        "the second test names the module, and the body runs once",
    );
}
