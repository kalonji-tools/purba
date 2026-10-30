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
fn a_test_that_carries_a_decorator_is_refused() {
    let (tests, refused) = judged(
        "test_decorated.py",
        concat!(
            "import purba\nimport purba as p\n\n",
            "def test_plain():\n    assert True\n\n",
            "@keep\ndef test_name():\n    assert True\n\n",
            "@functools.cache\ndef test_attribute():\n    assert True\n\n",
            "@pytest.mark.skipif(sys.platform == \"win32\", reason=\"fork\")\n",
            "def test_call():\n    assert True\n\n",
            "@table[0]\ndef test_subscript():\n    assert True\n\n",
            "@(lambda function:\n        function)\ndef test_lambda():\n    assert True\n\n",
            "@keep\n@mark(reason=\"slow\")\ndef test_two():\n    assert True\n\n",
            "@p.skip\ndef test_alias():\n    assert True\n\n",
            "@purba.skip\ndef test_purba():\n    assert True\n\n",
            "@sync\nasync def test_async():\n    return 1\n\n",
            "@keep\ndef test_every(x):\n    yield\n    return 1\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_plain"],
        "only the test with no decorator"
    );
    let not_purbas = |name: &str| {
        format!("the decorator {name} is not purba's, and purba cannot see what it does")
    };
    assert_eq!(
        refused,
        [
            format!("test_m.py::test_name {}", not_purbas("keep")),
            format!(
                "test_m.py::test_attribute {}",
                not_purbas("functools.cache")
            ),
            format!("test_m.py::test_call {}", not_purbas("pytest.mark.skipif")),
            format!("test_m.py::test_subscript {}", not_purbas("table[0]")),
            format!(
                "test_m.py::test_lambda {}",
                not_purbas("lambda function: function")
            ),
            "test_m.py::test_two the decorators keep, mark are not purba's, \
             and purba cannot see what they do"
                .to_owned(),
            format!("test_m.py::test_alias {}", not_purbas("p.skip")),
            format!("test_m.py::test_purba {}", not_purbas("purba.skip")),
            format!(
                "test_m.py::test_async {}; purba cannot run an async test yet; \
                 the test returns a value, and purba does not read it",
                not_purbas("sync")
            ),
            format!(
                "test_m.py::test_every {}; \
                 the test is a generator, and purba does not iterate it; \
                 the test returns a value, and purba does not read it; \
                 the parameter x has no default, and purba cannot see which fixture builds it",
                not_purbas("keep")
            ),
        ],
        "each decorator, with no call's arguments, then each reason in the body"
    );
}

/// A module that binds the names of its tests again, after the `def` or under
/// `global` in a function. Five tests keep the name that their `def` binds.
const REBOUND: &str = r"def rebind():
    global test_global
    test_global = None

class Holder:
    def method(self):
        global test_method
        test_method = None

test_before = None

def test_before(): assert True
def test_global(): assert True
def test_method(): assert True
def test_assign(): assert True
def test_augmented(): assert True
def test_annotated(): assert True
def test_deleted(): assert True
def test_imported(): assert True
def test_dotted(): assert True
def test_from(): assert True
def test_for(): assert True
def test_with(): assert True
def test_except(): assert True
def test_walrus(): assert True
def test_walrus_in_comprehension(): assert True
def test_case(): assert True
def test_type(): assert True
def test_class(): assert True
def test_def_in_if(): assert True
def test_def_in_try(): assert True
def test_def_in_for(): assert True
def test_def_in_while(): assert True
def test_def_in_with(): assert True
def test_def_in_match(): assert True
def test_never(): assert True
def test_twice(): assert True
def test_bare(): assert True
def test_in_class(): assert True
def test_comprehension(): assert True
def test_local(): assert True

test_assign = None
test_augmented += 1
test_annotated: int = 1
del test_deleted
import os as test_imported
import test_dotted.path
from os.path import join as test_from
for test_for in []: pass
with open(__file__) as test_with: pass
try: pass
except Exception as test_except: pass
(test_walrus := 1)
[(test_walrus_in_comprehension := 1) for _ in []]
match 1:
    case test_case: pass
type test_type = int
class test_class: pass
if True:
    def test_def_in_if(): assert True
try:
    def test_def_in_try(): assert True
finally: pass
for _ in []:
    def test_def_in_for(): assert True
while False:
    def test_def_in_while(): assert True
with open(__file__):
    def test_def_in_with(): assert True
match 1:
    case _:
        def test_def_in_match(): assert True
if False:
    test_never = None
for test_twice in []: pass
del test_twice
test_bare: int
class Box:
    test_in_class = None
[test_comprehension for test_comprehension in []]
def other():
    test_local = None
";

#[test]
fn a_test_whose_name_the_module_binds_again_is_refused() {
    let (tests, refused) = judged("test_rebound.py", REBOUND);

    assert_eq!(
        tests,
        [
            "test_m.py::test_before",
            "test_m.py::test_bare",
            "test_m.py::test_in_class",
            "test_m.py::test_comprehension",
            "test_m.py::test_local",
        ],
        "a binding before the def, a bare annotation, a class's body, \
         a comprehension's target and a name local to a function pass"
    );
    let rebound = |name: &str, line: u32| {
        format!(
            "test_m.py::{name} line {line} binds {name} again, \
             and purba cannot see what the name then holds"
        )
    };
    assert_eq!(
        refused,
        [
            rebound("test_global", 3),
            rebound("test_method", 8),
            rebound("test_assign", 43),
            rebound("test_augmented", 44),
            rebound("test_annotated", 45),
            rebound("test_deleted", 46),
            rebound("test_imported", 47),
            rebound("test_dotted", 48),
            rebound("test_from", 49),
            rebound("test_for", 50),
            rebound("test_with", 51),
            rebound("test_except", 53),
            rebound("test_walrus", 54),
            rebound("test_walrus_in_comprehension", 55),
            rebound("test_case", 57),
            rebound("test_type", 58),
            rebound("test_class", 59),
            rebound("test_def_in_if", 61),
            rebound("test_def_in_try", 63),
            rebound("test_def_in_for", 66),
            rebound("test_def_in_while", 68),
            rebound("test_def_in_with", 70),
            rebound("test_def_in_match", 73),
            rebound("test_never", 75),
            "test_m.py::test_twice lines 76, 77 bind test_twice again, \
             and purba cannot see what the name then holds"
                .to_owned(),
        ],
        "each binding after the last def, or under global in a function anywhere, with its line"
    );
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
        let text = format!("def test_deep():\n    {statement}\n    assert True\n\n{statement}\n");
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
