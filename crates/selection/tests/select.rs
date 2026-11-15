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
            "def test_dup():\n    assert 1 == 2\n\n",
            "def test_b():\n    pass\n\n",
            "def test_dup():\n    assert 1 == 1\n",
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
            "def test_d():\n    assert 1 == 1\n\n",
            "async def test_a():\n    ...\n\n",
            "def test_b():\n    yield\n\n",
            "def test_c(x):\n    ...\n\n",
            "def test_e():\n    assert 1 == 1\n    return False\n",
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
            "def test_never_runs():\n    assert 1 == 1\n    if False:\n        yield\n\n",
            "def test_from():\n    yield from ()\n\n",
            "def test_value():\n    x = yield\n\n",
            "def test_nested_def():\n    def inner():\n        yield\n    assert 1 == 1\n\n",
            "def test_lambda():\n    f = lambda: (yield)\n    assert 1 == 1\n\n",
            "def test_class():\n    class Inner:\n",
            "        def method(self):\n            yield\n    assert 1 == 1\n\n",
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
            "def test_false():\n    assert 1 == 1\n    return False\n\n",
            "def test_never_runs():\n    assert 1 == 1\n    if False:\n        return 1\n\n",
            "def test_bare():\n    assert 1 == 1\n    return\n\n",
            "def test_none():\n    assert 1 == 1\n    return None\n\n",
            "def test_nested_def():\n    def inner():\n",
            "        return 1\n    assert inner() == 1\n\n",
            "def test_class():\n    class Inner:\n",
            "        def method(self):\n            return 1\n    assert 1 == 1\n",
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
            "def test_default(x=1):\n    assert 1 == 1\n\n",
            "def test_typed_default(x: int = 1):\n    assert 1 == 1\n\n",
            "def test_typed(db: Database):\n    assert 1 == 1\n\n",
            "def test_typed_two(src: Database, dst: Database):\n    assert 1 == 1\n\n",
            "def test_typed_keyword(*, k: int):\n    assert 1 == 1\n\n",
            "def test_star(*args, **kwargs):\n    assert 1 == 1\n\n",
            "def test_positional(a, /):\n    assert 1 == 1\n\n",
            "def test_keyword(*, k):\n    assert 1 == 1\n\n",
            "def test_two(x, y):\n    assert 1 == 1\n\n",
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
            "def test_plain():\n    assert 1 == 1\n\n",
            "@keep\ndef test_name():\n    assert 1 == 1\n\n",
            "@functools.cache\ndef test_attribute():\n    assert 1 == 1\n\n",
            "@pytest.mark.skipif(sys.platform == \"win32\", reason=\"fork\")\n",
            "def test_call():\n    assert 1 == 1\n\n",
            "@table[0]\ndef test_subscript():\n    assert 1 == 1\n\n",
            "@(lambda function:\n        function)\ndef test_lambda():\n    assert 1 == 1\n\n",
            "@keep\n@mark(reason=\"slow\")\ndef test_two():\n    assert 1 == 1\n\n",
            "@p.skip\ndef test_alias():\n    assert 1 == 1\n\n",
            "@purba.skip\ndef test_purba():\n    assert 1 == 1\n\n",
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

def test_before(): assert 1 == 1
def test_global(): assert 1 == 1
def test_method(): assert 1 == 1
def test_assign(): assert 1 == 1
def test_augmented(): assert 1 == 1
def test_annotated(): assert 1 == 1
def test_deleted(): assert 1 == 1
def test_imported(): assert 1 == 1
def test_dotted(): assert 1 == 1
def test_from(): assert 1 == 1
def test_for(): assert 1 == 1
def test_with(): assert 1 == 1
def test_except(): assert 1 == 1
def test_walrus(): assert 1 == 1
def test_walrus_in_comprehension(): assert 1 == 1
def test_case(): assert 1 == 1
def test_type(): assert 1 == 1
def test_class(): assert 1 == 1
def test_def_in_if(): assert 1 == 1
def test_def_in_try(): assert 1 == 1
def test_def_in_for(): assert 1 == 1
def test_def_in_while(): assert 1 == 1
def test_def_in_with(): assert 1 == 1
def test_def_in_match(): assert 1 == 1
def test_never(): assert 1 == 1
def test_twice(): assert 1 == 1
def test_bare(): assert 1 == 1
def test_in_class(): assert 1 == 1
def test_comprehension(): assert 1 == 1
def test_local(): assert 1 == 1

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
    def test_def_in_if(): assert 1 == 1
try:
    def test_def_in_try(): assert 1 == 1
finally: pass
for _ in []:
    def test_def_in_for(): assert 1 == 1
while False:
    def test_def_in_while(): assert 1 == 1
with open(__file__):
    def test_def_in_with(): assert 1 == 1
match 1:
    case _:
        def test_def_in_match(): assert 1 == 1
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
fn a_test_name_bound_with_no_def_is_refused() {
    let (tests, refused) = judged(
        "test_nodef.py",
        concat!(
            "def keep(f):\n    return f\n\n",
            "def helper():\n    assert 1 == 2\n\n",
            "test_call = keep(lambda: \"x\")\n",
            "test_name = helper\n",
            "test_lambda = lambda: None\n",
            "from os.path import join as test_from\n",
            "for test_for in [helper]:\n    pass\n",
            "if True:\n    test_under_if = helper\n",
            "def setup():\n    global test_global\n    test_global = helper\n",
            "test_mixed = []\n",
            "test_mixed = helper\n",
            "test_twice = helper\n",
            "test_twice = keep(helper)\n",
            "@keep\nclass test_decorated:\n    pass\n",
            "def test_real():\n    assert 1 == 1\n",
        ),
    );

    assert_eq!(tests, ["test_m.py::test_real"], "only the def is a test");
    let no_def = |name: &str, line: u32| {
        format!(
            "test_m.py::{name} line {line} binds {name} with no def, \
             and purba cannot see what the name holds"
        )
    };
    assert_eq!(
        refused,
        [
            no_def("test_call", 7),
            no_def("test_name", 8),
            no_def("test_lambda", 9),
            no_def("test_from", 10),
            no_def("test_for", 11),
            no_def("test_under_if", 14),
            no_def("test_global", 17),
            no_def("test_mixed", 19),
            "test_m.py::test_twice lines 20, 21 bind test_twice with no def, \
             and purba cannot see what the name holds"
                .to_owned(),
            no_def("test_decorated", 23),
        ],
        "each name bound with no def, in source order, with each line that may bind a function"
    );
}

#[test]
fn a_test_name_that_never_holds_a_function_is_not_a_test() {
    let (tests, refused) = judged(
        "test_never.py",
        concat!(
            "import test_plain\n",
            "import test_package.module\n",
            "import os.path as test_module\n",
            "test_list = [(1, 2)]\n",
            "test_tuple = (1,)\n",
            "test_dict = {}\n",
            "test_set = {1}\n",
            "test_string = \"x\"\n",
            "test_bytes = b\"x\"\n",
            "test_fstring = f\"{1}\"\n",
            "test_number = 1\n",
            "test_bool = True\n",
            "test_none = None\n",
            "test_ellipsis = ...\n",
            "test_annotated: int = 3\n",
            "test_list_comprehension = [v for v in ()]\n",
            "test_set_comprehension = {v for v in ()}\n",
            "test_dict_comprehension = {v: v for v in ()}\n",
            "test_generator = (v for v in ())\n",
            "test_chained = test_other = []\n",
            "class test_class:\n    pass\n",
            "test_deleted = 1\n",
            "del test_deleted\n",
            "def test_real():\n    assert 1 == 1\n",
        ),
    );

    assert_eq!(tests, ["test_m.py::test_real"], "only the def is a test");
    assert!(
        refused.is_empty(),
        "a binding that never holds a function is no test: {refused:?}"
    );
}

#[test]
fn a_name_bound_with_no_def_keeps_its_place_among_refused_tests() {
    let (tests, refused) = judged(
        "test_place.py",
        concat!(
            "async def test_a():\n    ...\n\n",
            "test_b = len\n",
            "def test_c(x):\n    ...\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            "test_m.py::test_a purba cannot run an async test yet",
            "test_m.py::test_b line 4 binds test_b with no def, \
             and purba cannot see what the name holds",
            "test_m.py::test_c the parameter x has no default, \
             and purba cannot see which fixture builds it",
        ],
        "the refused lines keep source order"
    );
}

#[test]
fn selection_judges_the_last_def_of_a_name_at_the_place_of_the_first() {
    let (tests, refused) = judged(
        "test_last.py",
        concat!(
            "def test_fixed(x):\n    ...\n\n",
            "def test_broken():\n    assert 1 == 1\n\n",
            "def test_fixed():\n    assert 1 == 1\n\n",
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
        let text = format!("def test_deep():\n    {statement}\n    assert 1 == 1\n\n{statement}\n");
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

/// The words of the reason for a test whose `def` sits under `keyword`.
fn under(name: &str, keyword: &str) -> String {
    format!(
        "test_m.py::{name} the def sits under `{keyword}`, \
         and purba cannot see whether import binds it"
    )
}

#[test]
fn a_test_under_a_statement_at_the_modules_level_is_refused() {
    let (tests, refused) = judged(
        "test_under.py",
        concat!(
            "async def test_first():\n    ...\n\n",
            "if c:\n    def test_if(): assert 1 == 1\n",
            "elif d:\n    def test_elif(): assert 1 == 1\n",
            "else:\n    def test_else(): assert 1 == 1\n",
            "try:\n    def test_try(): assert 1 == 1\n",
            "except E:\n    def test_except(): assert 1 == 1\n",
            "else:\n    def test_try_else(): assert 1 == 1\n",
            "finally:\n    def test_finally(): assert 1 == 1\n",
            "with m:\n    def test_with(): assert 1 == 1\n",
            "async with m:\n    def test_async_with(): assert 1 == 1\n",
            "for _ in []:\n    def test_for(): assert 1 == 1\n",
            "else:\n    def test_for_else(): assert 1 == 1\n",
            "async for _ in []:\n    def test_async_for(): assert 1 == 1\n",
            "while w:\n    def test_while(): assert 1 == 1\n",
            "else:\n    def test_while_else(): assert 1 == 1\n",
            "match v:\n    case 1:\n        def test_case(): assert 1 == 1\n",
            "if c:\n    for _ in []:\n        with m:\n",
            "            def test_deeper(): assert 1 == 1\n",
            "if c:\n    def helper(): pass\n",
            "    async def test_body(x):\n        return 1\n\n",
            "def test_last():\n    assert 1 == 1\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_last"],
        "no def under a statement is handed on"
    );
    assert_eq!(
        refused,
        [
            "test_m.py::test_first purba cannot run an async test yet".to_owned(),
            under("test_if", "if"),
            under("test_elif", "if"),
            under("test_else", "if"),
            under("test_try", "try"),
            under("test_except", "try"),
            under("test_try_else", "try"),
            under("test_finally", "try"),
            under("test_with", "with"),
            under("test_async_with", "with"),
            under("test_for", "for"),
            under("test_for_else", "for"),
            under("test_async_for", "for"),
            under("test_while", "while"),
            under("test_while_else", "while"),
            under("test_case", "match"),
            under("test_deeper", "if"),
            format!(
                "{}; purba cannot run an async test yet; \
                 the test returns a value, and purba does not read it; \
                 the parameter x has no default, and purba cannot see which fixture builds it",
                under("test_body", "if")
            ),
        ],
        "each def named test under a statement, in source order with the other refused tests, \
         naming the statement at the module's level, then each reason in the body"
    );
}

#[test]
fn a_def_in_a_body_of_its_own_under_a_statement_is_not_a_test() {
    let (tests, refused) = judged(
        "test_own_body.py",
        concat!(
            "if c:\n    class C:\n        def test_method(self): assert 1 == 1\n",
            "    def outer():\n        def test_inner(): assert 1 == 1\n",
            "    f = lambda: 1\n",
            "try:\n    class D:\n        def test_other(self): assert 1 == 1\n",
            "finally:\n    pass\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert!(
        refused.is_empty(),
        "the body of a class or a def is not the module's: {refused:?}"
    );
}

#[test]
fn a_def_at_the_top_decides_the_name_of_a_def_under_a_statement() {
    let (tests, refused) = judged(
        "test_top_decides.py",
        concat!(
            "if c:\n    def test_later(): assert 1 == 2\n\n",
            "def test_later():\n    assert 1 == 1\n\n",
            "def test_earlier():\n    assert 1 == 1\n\n",
            "if c:\n    def test_earlier(): assert 1 == 2\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_later"],
        "the later def at the top binds the name last"
    );
    assert_eq!(
        refused,
        ["test_m.py::test_earlier line 11 binds test_earlier again, \
         and purba cannot see what the name then holds"],
        "the def under the statement binds the earlier test's name again, \
         and gives no line of its own"
    );
}

#[test]
fn two_defs_of_one_name_under_statements_are_one_refused_test_at_the_first() {
    let (tests, refused) = judged(
        "test_under_twice.py",
        concat!(
            "if c:\n    def test_twice(): assert 1 == 1\n",
            "if c:\n    def test_between(): assert 1 == 1\n",
            "try:\n    def test_twice(): assert 1 == 1\n",
            "finally:\n    pass\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [under("test_twice", "if"), under("test_between", "if")],
        "one refused test for each name, where import first binds it"
    );
}

#[test]
fn a_def_under_statements_that_nest_deeper_than_the_stack_does_not_crash_selection() {
    const SMALL_STACK: usize = 256 * 1024;
    const DEPTH: usize = 3_000;
    let mut text = String::new();
    for level in 0..DEPTH {
        text.push_str(&" ".repeat(level));
        text.push_str("if c:\n");
    }
    text.push_str(&" ".repeat(DEPTH));
    text.push_str("def test_deep(): assert 1 == 1\n");
    let name = "test_deep_statements.py";
    let judged = thread::Builder::new()
        .name(name.to_owned())
        .stack_size(SMALL_STACK)
        .spawn(move || judged(name, &text))
        .expect("the thread starts")
        .join();
    let (tests, refused) = judged.expect("selection returns");

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [under("test_deep", "if")],
        "the def under the deepest statement is refused"
    );
}

/// The words of a fault of an expectation that is not one comparison.
fn not_one(line: usize) -> String {
    format!(
        "line {line} is not one comparison, \
         and an expectation compares a reading by one operator"
    )
}

/// The words of a fault of an expected value that is not a literal.
fn computes(line: usize, text: &str) -> String {
    format!("line {line} computes {text}, and an expected value is a literal")
}

/// The words of a fault of a private name that a test reads.
fn private(line: usize, name: &str) -> String {
    format!("line {line} reads the private name {name}, and a test reads no private name")
}

#[test]
fn an_expectation_that_is_not_one_comparison_is_refused() {
    let (tests, refused) = judged(
        "test_not_one.py",
        concat!(
            "def test_bare():\n    assert 1 == 1\n    assert x\n\n",
            "def test_not():\n    assert not x\n\n",
            "def test_call():\n    assert isinstance(x, int)\n\n",
            "def test_chain():\n    assert 0 < x < 9\n\n",
            "def test_and():\n    assert x == 1 and y == 2\n\n",
            "def test_tuple():\n    assert x == 1\n    assert (x == 2, \"msg\")\n\n",
            "def test_empty():\n    assert ()\n\n",
            "def test_act():\n    assert x\n    y = 1\n    assert y == 1\n\n",
            "def test_ops():\n",
            "    assert x == 1\n    assert x != 1\n    assert x < 1\n    assert x <= 1\n",
            "    assert x > 1\n    assert x >= 1\n    assert x in [1]\n    assert x not in [1]\n",
            "    assert x is None\n    assert x is not None, \"with a message\"\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_act", "test_m.py::test_ops"],
        "an assert in the act is no expectation, and each of the ten operators is one comparison"
    );
    assert_eq!(
        refused,
        [
            format!("test_m.py::test_bare {}", not_one(3)),
            format!("test_m.py::test_not {}", not_one(6)),
            format!("test_m.py::test_call {}", not_one(9)),
            format!("test_m.py::test_chain {}", not_one(12)),
            format!("test_m.py::test_and {}", not_one(15)),
            format!("test_m.py::test_tuple {}", not_one(19)),
            format!("test_m.py::test_empty {}", not_one(22)),
        ],
        "a bare value, a not, a call, a chain, an and, and a tuple at the end are each refused"
    );
}

#[test]
fn an_expected_value_that_is_not_a_literal_is_refused() {
    let (tests, refused) = judged(
        "test_computed.py",
        concat!(
            "N = 3\n\n",
            "def test_constant():\n    assert x == N\n\n",
            "def test_call():\n    assert x == f()\n\n",
            "def test_format():\n    assert x == f\"{y}\"\n\n",
            "def test_input(n=3):\n    assert x == n\n\n",
            "def test_both():\n    assert f() == g(\n        1)\n\n",
            "def test_literals():\n",
            "    assert x == \"7\" * 20 + \"\\n\"\n",
            "    assert x == -1\n    assert x == ~1 + 1.5j\n    assert x == b\"x\"\n",
            "    assert x is None\n    assert x is True\n    assert x == ...\n",
            "    assert x == [1, (2, \"a\"), {3}, {\"k\": 4.5}]\n",
            "    assert x == f\"plain\" \"joined\"\n    assert \"a\" in x\n    assert 1 == 1\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_literals"],
        "a literal on either side, operators on literals, and a container of them pass"
    );
    assert_eq!(
        refused,
        [
            format!("test_m.py::test_constant {}", computes(4, "N")),
            format!("test_m.py::test_call {}", computes(7, "f()")),
            format!("test_m.py::test_format {}", computes(10, "f\"{y}\"")),
            format!("test_m.py::test_input {}", computes(13, "n")),
            format!("test_m.py::test_both {}", computes(16, "g( 1)")),
        ],
        "a constant, a call, an f-string with a placeholder, an input, and two readings \
         are each refused, by the text on the right, on one line"
    );
}

#[test]
fn a_test_that_reads_a_private_name_is_refused() {
    let (tests, refused) = judged(
        "test_private.py",
        concat!(
            "import os\n",
            "from lib import _hidden, shown\n",
            "from lib._impl import Thing\n",
            "import _thread\n",
            "from gettext import gettext as _\n",
            "from . import _sibling\n",
            "\n",
            "def _make():\n    return 1\n\n",
            "def test_attribute():\n    w = os.getcwd()\n    w._flush()\n",
            "    assert w.__len__() == 1\n\n",
            "def test_imported():\n    _hidden()\n    assert Thing() == 1\n\n",
            "def test_module():\n    assert _thread.get_ident() == 1\n\n",
            "def test_relative():\n    _sibling()\n    assert 1 == 1\n\n",
            "def test_inside():\n    from lib import _x\n    import lib._inner as inner\n",
            "    assert 1 == 1\n\n",
            "def test_store():\n    obj = shown()\n    obj._x = 1\n",
            "    assert f\"{obj._y}\" == \"1\"\n\n",
            "def test_free():\n    _local = _make()\n    _(\"text\")\n    shown()\n",
            "    assert _local == 1\n\n",
            "def test_once():\n    assert w._a + w._a == 2\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_free"],
        "a name of the test's own module, a local, a dunder and a public import are free"
    );
    assert_eq!(
        refused,
        [
            format!("test_m.py::test_attribute {}", private(13, "_flush")),
            format!(
                "test_m.py::test_imported {}; {}",
                private(17, "lib._hidden"),
                private(18, "lib._impl.Thing")
            ),
            format!("test_m.py::test_module {}", private(21, "_thread")),
            format!("test_m.py::test_relative {}", private(24, "._sibling")),
            format!(
                "test_m.py::test_inside {}; {}",
                private(28, "lib._x"),
                private(29, "lib._inner")
            ),
            format!(
                "test_m.py::test_store {}; {}",
                private(34, "_x"),
                private(35, "_y")
            ),
            format!("test_m.py::test_once {}", private(44, "_a")),
        ],
        "an attribute, an import from another module, and a private part of a module's path \
         are each refused, once a line, in source order"
    );
}

#[test]
fn a_refused_line_names_the_reasons_of_the_def_then_each_fault_of_the_body_in_source_order() {
    let (tests, refused) = judged(
        "test_mixed.py",
        concat!(
            "@dec\n",
            "def test_mixed():\n    w._a()\n    assert w == N\n    assert w\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [format!(
            "test_m.py::test_mixed the decorator dec is not purba's, \
             and purba cannot see what it does; {}; {}; {}",
            private(3, "_a"),
            computes(4, "N"),
            not_one(5)
        )],
        "the decorator first, then each fault of the body by its line"
    );
}

#[test]
fn an_expectation_that_nests_deeper_than_the_stack_does_not_crash_selection() {
    const SMALL_STACK: usize = 256 * 1024;
    let deep = "-".repeat(20_000);
    let nested = format!("{}1{}", "[".repeat(3_000), "]".repeat(3_000));
    let text = format!(
        "def test_deep():\n    assert x == {deep}1\n    assert {deep}x == 1\n    \
         assert x == {nested}\n"
    );
    let name = "test_deep_expectation.py";
    let judged = thread::Builder::new()
        .name(name.to_owned())
        .stack_size(SMALL_STACK)
        .spawn(move || judged(name, &text))
        .expect("the thread starts")
        .join();
    let (tests, refused) = judged.expect("selection returns");

    assert_eq!(tests, ["test_m.py::test_deep"], "the test is handed on");
    assert!(refused.is_empty(), "nothing refused: {refused:?}");
}
