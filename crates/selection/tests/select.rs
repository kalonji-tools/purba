use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

fn write(name: &str, text: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::write(&path, text).expect("the test writes its module");
    path
}

#[test]
fn each_test_function_in_source_order() {
    let file = write(
        "test_order.py",
        "def test_one():\n    pass\n\ndef helper():\n    pass\n\ndef test_two():\n    pass\n",
    );
    let module = purba_parsing::parse(&file).expect("the module parses");

    let names: Vec<String> = purba_selection::select(Path::new("test_a.py"), &module)
        .0
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(
        names,
        ["test_a.py::test_one", "test_a.py::test_two"],
        "each test, in source order"
    );
}

#[test]
fn an_identity_gives_its_path_and_function() {
    let file = write("test_parts.py", "def test_one():\n    pass\n");
    let module = purba_parsing::parse(&file).expect("the module parses");

    let selected = purba_selection::select(Path::new("tests/test_a.py"), &module).0;
    let test = selected.first().expect("the module holds one test");

    assert_eq!(
        test.path(),
        Path::new("tests/test_a.py"),
        "the path as typed"
    );
    assert_eq!(test.function(), "test_one", "the function's name");
}

#[test]
fn a_name_defined_twice_is_one_test_at_its_first_definition() {
    let file = write(
        "test_twice.py",
        concat!(
            "def test_a():\n    pass\n\n",
            "def test_dup():\n    assert False\n\n",
            "def test_b():\n    pass\n\n",
            "def test_dup():\n    assert True\n",
        ),
    );
    let module = purba_parsing::parse(&file).expect("the module parses");

    let names: Vec<String> = purba_selection::select(Path::new("test_a.py"), &module)
        .0
        .iter()
        .map(ToString::to_string)
        .collect();

    assert_eq!(
        names,
        [
            "test_a.py::test_a",
            "test_a.py::test_dup",
            "test_a.py::test_b"
        ],
        "one test for each name, where import first binds it"
    );
}

/// The tests that selection hands on, and each test it refuses beside its
/// reasons, for the module that `text` holds.
fn judged(name: &str, text: &str) -> (Vec<String>, Vec<String>) {
    let file = write(name, text);
    let module = purba_parsing::parse(&file).expect("the module parses");
    let (tests, refused) = purba_selection::select(Path::new("test_m.py"), &module);
    (
        tests.iter().map(ToString::to_string).collect(),
        refused
            .iter()
            .map(|refused| format!("{} {refused}", refused.test()))
            .collect(),
    )
}

#[test]
fn a_test_that_purba_cannot_call_is_refused_with_its_reason() {
    let (tests, refused) = judged(
        "test_uncallable.py",
        concat!(
            "def test_d():\n    assert True\n\n",
            "async def test_a():\n    ...\n\n",
            "def test_b():\n    yield\n\n",
            "def test_c(x):\n    ...\n\n",
            "def test_e():\n    assert True\n    return False\n",
        ),
    );

    assert_eq!(tests, ["test_m.py::test_d"], "only the test purba can call");
    assert_eq!(
        refused,
        [
            "test_m.py::test_a purba cannot run an async test yet",
            "test_m.py::test_b the test is a generator, and purba does not iterate it",
            "test_m.py::test_c the parameter x has no default, \
             and purba cannot see which fixture builds it",
            "test_m.py::test_e the test returns a value, and purba does not read it",
        ],
        "each refused test, in source order, with its reason"
    );
}

#[test]
fn a_yield_in_the_tests_own_body_makes_it_a_generator() {
    let (tests, refused) = judged(
        "test_yield.py",
        concat!(
            "def test_never_runs():\n    assert True\n    if False:\n        yield\n\n",
            "def test_from():\n    yield from ()\n\n",
            "def test_value():\n    x = yield\n\n",
            "def test_nested_def():\n    def inner():\n        yield\n    assert True\n\n",
            "def test_lambda():\n    f = lambda: (yield)\n    assert True\n\n",
            "def test_class():\n    class Inner:\n",
            "        def method(self):\n            yield\n    assert True\n\n",
            "async def test_async_generator():\n    yield\n",
        ),
    );

    assert_eq!(
        tests,
        [
            "test_m.py::test_nested_def",
            "test_m.py::test_lambda",
            "test_m.py::test_class",
        ],
        "a yield inside a nested def, lambda or class is not the test's"
    );
    assert_eq!(
        refused,
        [
            "test_m.py::test_never_runs the test is a generator, and purba does not iterate it",
            "test_m.py::test_from the test is a generator, and purba does not iterate it",
            "test_m.py::test_value the test is a generator, and purba does not iterate it",
            "test_m.py::test_async_generator purba cannot run an async test yet",
        ],
        "a yield in a branch that never runs counts, and an async generator is async"
    );
}

#[test]
fn a_return_of_a_value_in_the_tests_own_body_is_refused() {
    let (tests, refused) = judged(
        "test_return.py",
        concat!(
            "def test_false():\n    assert True\n    return False\n\n",
            "def test_never_runs():\n    assert True\n    if False:\n        return 1\n\n",
            "def test_bare():\n    assert True\n    return\n\n",
            "def test_none():\n    assert True\n    return None\n\n",
            "def test_nested_def():\n    def inner():\n",
            "        return 1\n    assert inner() == 1\n\n",
            "def test_class():\n    class Inner:\n",
            "        def method(self):\n            return 1\n    assert True\n",
        ),
    );

    assert_eq!(
        tests,
        [
            "test_m.py::test_bare",
            "test_m.py::test_none",
            "test_m.py::test_nested_def",
            "test_m.py::test_class",
        ],
        "return, return None, and a return inside a nested def or class pass"
    );
    assert_eq!(
        refused,
        [
            "test_m.py::test_false the test returns a value, and purba does not read it",
            "test_m.py::test_never_runs the test returns a value, and purba does not read it",
        ],
        "a return of a value counts, even in a branch that never runs"
    );
}

#[test]
fn a_parameter_with_no_default_is_refused() {
    let (tests, refused) = judged(
        "test_parameters.py",
        concat!(
            "def test_default(x=1):\n    assert True\n\n",
            "def test_typed_default(x: int = 1):\n    assert True\n\n",
            "def test_typed(db: Database):\n    assert True\n\n",
            "def test_typed_two(src: Database, dst: Database):\n    assert True\n\n",
            "def test_typed_keyword(*, k: int):\n    assert True\n\n",
            "def test_star(*args, **kwargs):\n    assert True\n\n",
            "def test_positional(a, /):\n    assert True\n\n",
            "def test_keyword(*, k):\n    assert True\n\n",
            "def test_two(x, y):\n    assert True\n\n",
            "async def test_async(x):\n    ...\n",
        ),
    );

    assert_eq!(
        tests,
        [
            "test_m.py::test_default",
            "test_m.py::test_typed_default",
            "test_m.py::test_star",
        ],
        "a default, *args and **kwargs pass"
    );
    assert_eq!(
        refused,
        [
            "test_m.py::test_typed the parameter db has no default, \
             and purba cannot see which fixture builds it",
            "test_m.py::test_typed_two the parameters src, dst have no default, \
             and purba cannot see which fixtures build them",
            "test_m.py::test_typed_keyword the parameter k has no default, \
             and purba cannot see which fixture builds it",
            "test_m.py::test_positional the parameter a has no default, \
             and purba cannot see which fixture builds it",
            "test_m.py::test_keyword the parameter k has no default, \
             and purba cannot see which fixture builds it",
            "test_m.py::test_two the parameters x, y have no default, \
             and purba cannot see which fixtures build them",
            "test_m.py::test_async purba cannot run an async test yet; \
             the parameter x has no default, and purba cannot see which fixture builds it",
        ],
        "a parameter with no default is refused whatever its type, with every reason on one line"
    );
}

#[test]
fn selection_does_not_judge_a_decorated_test() {
    let (tests, refused) = judged(
        "test_decorated.py",
        concat!(
            "@sync\nasync def test_async():\n    ...\n\n",
            "@sync\ndef test_parameter(x):\n    ...\n\n",
            "@sync\ndef test_return():\n    return 1\n",
        ),
    );

    assert_eq!(
        tests,
        [
            "test_m.py::test_async",
            "test_m.py::test_parameter",
            "test_m.py::test_return",
        ],
        "a decorator can change what the call takes and returns"
    );
    assert!(refused.is_empty(), "nothing refused: {refused:?}");
}

#[test]
fn selection_judges_the_last_def_of_a_name_at_the_place_of_the_first() {
    let (tests, refused) = judged(
        "test_last.py",
        concat!(
            "def test_fixed(x):\n    ...\n\n",
            "def test_broken():\n    assert True\n\n",
            "def test_fixed():\n    assert True\n\n",
            "def test_broken(x):\n    ...\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_fixed"],
        "the last def of test_fixed takes nothing"
    );
    assert_eq!(
        refused,
        ["test_m.py::test_broken the parameter x has no default, \
         and purba cannot see which fixture builds it"],
        "the last def of test_broken takes x"
    );
}

#[test]
fn a_body_that_nests_deeper_than_the_stack_does_not_crash_selection() {
    const SMALL_STACK: usize = 256 * 1024;
    for (name, statement) in [
        ("test_deep_minus.py", format!("x = {}1", "-".repeat(20_000))),
        (
            "test_deep_pattern.py",
            format!(
                "match 1:\n        case {}1{}:\n            pass",
                "[".repeat(20_000),
                "]".repeat(20_000)
            ),
        ),
        (
            "test_deep_format_spec.py",
            format!("x = f\"{}x{}\"", "{x:".repeat(3_000), "}".repeat(3_000)),
        ),
    ] {
        let text = format!("def test_deep():\n    {statement}\n    assert True\n");
        let judged = thread::Builder::new()
            .name(name.to_owned())
            .stack_size(SMALL_STACK)
            .spawn(move || judged(name, &text))
            .expect("the thread starts")
            .join();
        let (tests, refused) = judged.expect("selection returns");
        assert_eq!(
            tests,
            ["test_m.py::test_deep"],
            "{name}: the test is handed on"
        );
        assert!(refused.is_empty(), "{name}: nothing refused: {refused:?}");
    }
}
