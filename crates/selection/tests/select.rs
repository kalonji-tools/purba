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
        .tests()
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

    let selected = purba_selection::select(Path::new("tests/test_a.py"), &module);
    let test = selected.tests().first().expect("the module holds one test");

    assert_eq!(
        test.path(),
        Path::new("tests/test_a.py"),
        "the path as typed"
    );
    assert_eq!(test.function(), "test_one", "the function's name");
}

#[test]
fn a_name_defined_twice_is_one_refused_test() {
    let (tests, refused) = judged(
        "test_twice.py",
        concat!(
            "def test_a():\n    assert 1 == 1\n\n",
            "def test_dup():\n    assert 1 == 2\n\n",
            "def test_b():\n    assert 1 == 1\n\n",
            "def test_dup():\n    assert 1 == 1\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_a", "test_m.py::test_b"],
        "each name defined once is a test"
    );
    assert_eq!(
        refused,
        [
            "test_m.py::test_dup [name-bound-again] line 10 binds test_dup again, \
             and purba cannot see what the name then holds"
        ],
        "the second def binds the name again, and the first def never runs"
    );
}

/// The tests that selection hands on, and each test it refuses beside its
/// reasons, for the module that `text` holds.
fn judged(name: &str, text: &str) -> (Vec<String>, Vec<String>) {
    let file = write(name, text);
    let module = purba_parsing::parse(&file).expect("the module parses");
    let selected = purba_selection::select(Path::new("test_m.py"), &module);
    let (tests, refused) = (selected.tests(), selected.refused());
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
            "test_m.py::test_e [returned-value] the test returns a value, \
             and purba does not read it",
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
            "test_m.py::test_false [returned-value] the test returns a value, \
             and purba does not read it",
            "test_m.py::test_never_runs [returned-value] the test returns a value, \
             and purba does not read it",
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
        format!(
            "[foreign-decorator] the decorator {name} is not purba's, \
             and purba cannot see what it does"
        )
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
            "test_m.py::test_two [foreign-decorator] the decorators keep, mark are not purba's, \
             and purba cannot see what they do"
                .to_owned(),
            format!("test_m.py::test_alias {}", not_purbas("p.skip")),
            format!("test_m.py::test_purba {}", not_purbas("purba.skip")),
            format!(
                "test_m.py::test_async {}; purba cannot run an async test yet; \
                 [returned-value] the test returns a value, and purba does not read it",
                not_purbas("sync")
            ),
            format!(
                "test_m.py::test_every {}; \
                 the test is a generator, and purba does not iterate it; \
                 [returned-value] the test returns a value, and purba does not read it; \
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
            "test_m.py::{name} [name-bound-again] line {line} binds {name} again, \
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
            "test_m.py::test_twice [name-bound-again] lines 76, 77 bind test_twice again, \
             and purba cannot see what the name then holds"
                .to_owned(),
        ],
        "each binding after the first def, or under global in a function anywhere, with its line"
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
            "test_m.py::{name} [name-without-def] line {line} binds {name} with no def, \
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
            "test_m.py::test_twice [name-without-def] lines 20, 21 bind test_twice with no def, \
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
            "test_m.py::test_b [name-without-def] line 4 binds test_b with no def, \
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

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            "test_m.py::test_fixed [name-bound-again] line 7 binds test_fixed again, \
             and purba cannot see what the name then holds",
            "test_m.py::test_broken the parameter x has no default, \
             and purba cannot see which fixture builds it; \
             [name-bound-again] line 10 binds test_broken again, \
             and purba cannot see what the name then holds",
        ],
        "the last def of test_fixed takes nothing, and the last def of test_broken takes x"
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
        "test_m.py::{name} [def-under-block] the def sits under `{keyword}`, \
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
                 [returned-value] the test returns a value, and purba does not read it; \
                 the parameter x has no default, and purba cannot see which fixture builds it",
                under("test_body", "if")
            ),
        ],
        "each def named test under a statement, in source order with the other refused tests, \
         naming the statement at the module's level, then each reason in the body"
    );
}

#[test]
fn a_def_in_a_def_under_a_statement_is_not_a_test_and_a_def_in_a_class_is_refused() {
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
    assert_eq!(
        refused,
        [method("C::test_method", ""), method("D::test_other", "")],
        "a class under a statement is read, and the body of a def is not the module's"
    );
}

/// The words of the reason for a test `name` in a class, then `more`.
fn method(name: &str, more: &str) -> String {
    format!("test_m.py::{name} purba cannot call a method of a class{more}")
}

#[test]
fn a_test_in_a_class_is_refused_and_named_by_its_class() {
    let (tests, refused) = judged(
        "test_class.py",
        concat!(
            "def test_plain():\n    assert 1 == 1\n\n",
            "class TestGroup:\n",
            "    def test_in_class(self):\n        assert 1 == 2\n\n",
            "    def test_fixture(self, tmp_path):\n        assert tmp_path.name == \"x\"\n\n",
            "class FakeClient:\n",
            "    def test_connection(self):\n        self.ok = True\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_plain"],
        "only the test at the module's level"
    );
    assert_eq!(
        refused,
        [
            method("TestGroup::test_in_class", ""),
            method(
                "TestGroup::test_fixture",
                "; the parameter tmp_path has no default, \
                 and purba cannot see which fixture builds it"
            ),
            method("FakeClient::test_connection", ""),
        ],
        "each test of every class, whatever its name, in source order, with no reason for self"
    );
}

#[test]
fn a_class_in_a_class_adds_its_name_and_a_class_in_a_def_is_not_read() {
    let (tests, refused) = judged(
        "test_nested_class.py",
        concat!(
            "class Outer:\n",
            "    class Inner:\n",
            "        def test_deep(self):\n            assert 1 == 1\n\n",
            "    def test_outer(self):\n        assert 1 == 1\n\n",
            "def helper():\n",
            "    class InDef:\n",
            "        def test_hidden(self):\n            assert 1 == 1\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            method("Outer::Inner::test_deep", ""),
            method("Outer::test_outer", "")
        ],
        "each class on the way names the test, in source order"
    );
}

#[test]
fn a_def_under_a_statement_in_a_class_is_refused_as_a_test_in_a_class() {
    let (tests, refused) = judged(
        "test_class_block.py",
        concat!(
            "class Guarded:\n",
            "    if c:\n        def test_if(self):\n            assert 1 == 1\n",
            "    try:\n        class Inner:\n",
            "            def test_try(self):\n                assert 1 == 1\n",
            "    finally:\n        pass\n",
            "    def helper(self):\n",
            "        def test_in_def():\n            assert 1 == 1\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            method("Guarded::test_if", ""),
            method("Guarded::Inner::test_try", "")
        ],
        "a def or a class under a statement in a class is read, and a def in a method is not"
    );
}

#[test]
fn a_test_in_a_class_gives_every_reason_of_its_def_and_is_one_test_for_each_name() {
    let (tests, refused) = judged(
        "test_class_reasons.py",
        concat!(
            "def test_shared():\n    assert 1 == 1\n\n",
            "class Case(TestCase):\n",
            "    def test_shared(self):\n        assert 1 == 1\n\n",
            "    def test_twice(self):\n        assert 1 == 2\n\n",
            "    @staticmethod\n",
            "    def test_static(x):\n        assert x == 1\n\n",
            "    async def test_async(self):\n        assert 1 == 1\n\n",
            "    def test_twice(self):\n        assert 1 == 1\n\n",
            "class Other:\n",
            "    def test_shared(self):\n        assert 1 == 1\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_shared"],
        "a test at the module's level keeps its name beside a method of that name"
    );
    assert_eq!(
        refused,
        [
            method("Case::test_shared", ""),
            method("Case::test_twice", ""),
            method(
                "Case::test_static",
                "; [foreign-decorator] the decorator staticmethod is not purba's, \
                 and purba cannot see what it does; \
                 the parameter x has no default, and purba cannot see which fixture builds it"
            ),
            method("Case::test_async", "; purba cannot run an async test yet"),
            method("Other::test_shared", ""),
        ],
        "one refused test for each name in each class, at its first def, \
         and no reason that the module's own names give"
    );
}

#[test]
fn a_def_at_the_top_and_a_def_under_a_statement_bind_one_name_again() {
    let (tests, refused) = judged(
        "test_top_decides.py",
        concat!(
            "if c:\n    def test_later(): assert 1 == 2\n\n",
            "def test_later():\n    assert 1 == 1\n\n",
            "def test_earlier():\n    assert 1 == 1\n\n",
            "if c:\n    def test_earlier(): assert 1 == 2\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            "test_m.py::test_later [name-bound-again] line 4 binds test_later again, \
             and purba cannot see what the name then holds",
            "test_m.py::test_earlier [name-bound-again] line 11 binds test_earlier again, \
             and purba cannot see what the name then holds",
        ],
        "the later def binds the name again, at the top or under the statement, \
         and the def under the statement gives no line of its own"
    );
}

#[test]
fn a_refused_line_sits_at_the_first_def_when_that_def_is_under_a_statement() {
    let (tests, refused) = judged(
        "test_first_under.py",
        concat!(
            "if True:\n    def test_x(): assert 1 == 1\n\n",
            "async def test_y():\n    ...\n\n",
            "def test_x():\n    assert 1 == 1\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            "test_m.py::test_x [name-bound-again] line 7 binds test_x again, \
             and purba cannot see what the name then holds",
            "test_m.py::test_y purba cannot run an async test yet",
        ],
        "the line of test_x sits at its def under the statement, before test_y"
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
        [
            format!(
                "{}; [name-bound-again] line 6 binds test_twice again, \
                 and purba cannot see what the name then holds",
                under("test_twice", "if")
            ),
            under("test_between", "if"),
        ],
        "one refused test for each name, where import first binds it, \
         and the second def binds the name again"
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
        "[not-one-comparison] line {line} is not one comparison, \
         and an expectation compares a reading by one operator, \
         or reads not x, isinstance(x, T), x.startswith(y) or x.endswith(y)"
    )
}

/// The words of a fault of an expected value that is not a literal.
fn computes(line: usize, text: &str) -> String {
    format!("[computed-expected] line {line} computes {text}, and an expected value is a literal")
}

/// The words of a fault of an input on the expected side.
fn input(line: usize, name: &str) -> String {
    format!(
        "[computed-expected] line {line} compares to the input {name}, \
         and an expected value is a literal until purba has cases"
    )
}

/// The words of a notice of a private name that a test reads.
fn private(line: usize, name: &str) -> String {
    format!("line {line} reads the private name {name}")
}

/// Each selected test of the module that `text` holds, with its notices.
fn noticed(name: &str, text: &str) -> Vec<(String, Vec<String>)> {
    let file = write(name, text);
    let module = purba_parsing::parse(&file).expect("the module parses");
    purba_selection::select(Path::new("test_m.py"), &module)
        .tests()
        .iter()
        .map(|test| (test.to_string(), test.notices().to_vec()))
        .collect()
}

#[test]
fn an_expectation_that_is_not_one_comparison_or_one_of_four_reads_is_refused() {
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
            "    assert x is None\n    assert x is not None, \"with a message\"\n\n",
            "def test_reads():\n    assert x.startswith(\"a\")\n    assert x.endswith(\"a\")\n\n",
            "def test_keyword():\n    assert isinstance(x, T=int)\n\n",
            "def test_three():\n    assert isinstance(x, int, y)\n\n",
            "def test_two_args():\n    assert x.startswith(\"a\", 1)\n\n",
            "def test_method():\n    assert x.upper()\n",
        ),
    );

    assert_eq!(
        tests,
        [
            "test_m.py::test_not",
            "test_m.py::test_call",
            "test_m.py::test_act",
            "test_m.py::test_ops",
            "test_m.py::test_reads",
        ],
        "an assert in the act is no expectation, each of the ten operators is one comparison, \
         and not x, isinstance(x, T), x.startswith(y) and x.endswith(y) each read"
    );
    assert_eq!(
        refused,
        [
            format!("test_m.py::test_bare {}", not_one(3)),
            format!("test_m.py::test_chain {}", not_one(12)),
            format!("test_m.py::test_and {}", not_one(15)),
            format!("test_m.py::test_tuple {}", not_one(19)),
            format!("test_m.py::test_empty {}", not_one(22)),
            format!("test_m.py::test_keyword {}", not_one(46)),
            format!("test_m.py::test_three {}", not_one(49)),
            format!("test_m.py::test_two_args {}", not_one(52)),
            format!("test_m.py::test_method {}", not_one(55)),
        ],
        "a bare value, a chain, an and, a tuple, a keyword or a third argument of isinstance, \
         a second argument of startswith, and another method are each refused"
    );
}

#[test]
fn an_all_or_any_of_one_check_is_an_expectation_whose_expected_side_is_a_literal() {
    let (tests, refused) = judged(
        "test_each.py",
        concat!(
            "def test_compare():\n    assert all(len(c) == 2 for c in parts)\n\n",
            "def test_any():\n    assert any(p == \"/opt\" for p in paths)\n\n",
            "def test_instance():\n    assert all(isinstance(x, int) for x in xs)\n\n",
            "def test_not():\n    assert all(not x for x in xs)\n\n",
            "def test_prefix():\n    assert any(s.startswith(\"a\") for s in names)\n\n",
            "def test_pairs():\n    assert all(a < 3 for a, b in pairs)\n\n",
            "def test_suffix():\n    assert all(s.endswith(\".py\") for s in files)\n\n",
            "def test_computed():\n    assert all(len(c) == size for c in parts)\n",
        ),
    );

    assert_eq!(
        tests,
        [
            "test_m.py::test_compare",
            "test_m.py::test_any",
            "test_m.py::test_instance",
            "test_m.py::test_not",
            "test_m.py::test_prefix",
            "test_m.py::test_pairs",
            "test_m.py::test_suffix",
        ],
        "an all() or an any() of one comparison or one of the four reads, with one for and \
         no if, is an expectation"
    );
    assert_eq!(
        refused,
        [format!("test_m.py::test_computed {}", computes(23, "size"))],
        "the check of each item keeps the rule of an expected value"
    );
}

/// The words of a fault of an `all()` or an `any()` of another shape.
fn each_shape(line: usize, function: &str, shape: &str) -> String {
    format!(
        "[not-one-comparison] line {line} is an {function}() {shape}, \
         and an {function}() takes one for, no if"
    )
}

#[test]
fn an_all_or_any_of_another_shape_is_refused_with_words_that_name_its_shape() {
    let (tests, refused) = judged(
        "test_each_shapes.py",
        concat!(
            "def test_if():\n    assert all(x > 0 for x in xs if x)\n\n",
            "def test_two_for():\n    assert all(x > 0 for row in rows for x in row)\n\n",
            "def test_list():\n    assert all([x > 0 for x in xs])\n\n",
            "def test_any_if():\n    assert any(x > 0 for x in xs if x)\n\n",
            "def test_name():\n    assert all(xs)\n\n",
            "def test_bare_item():\n    assert all(x for x in xs)\n\n",
            "def test_three_for():\n    assert all(x > 0 for a in b for c in a for x in c)\n\n",
            "def test_four_for():\n",
            "    assert any(x > 0 for a in b for c in a for d in c for x in d)\n\n",
            "def test_walrus():\n    assert all((y := x) < 3 for x in xs)\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            format!("test_m.py::test_if {}", each_shape(2, "all", "with an if")),
            format!(
                "test_m.py::test_two_for {}",
                each_shape(5, "all", "with two for")
            ),
            format!(
                "test_m.py::test_list {}",
                each_shape(8, "all", "of a list, not a generator")
            ),
            format!(
                "test_m.py::test_any_if {}",
                each_shape(11, "any", "with an if")
            ),
            format!("test_m.py::test_name {}", not_one(14)),
            format!("test_m.py::test_bare_item {}", not_one(17)),
            format!(
                "test_m.py::test_three_for {}",
                each_shape(20, "all", "with three for")
            ),
            format!(
                "test_m.py::test_four_for {}",
                each_shape(23, "any", "with more than three for")
            ),
            format!(
                "test_m.py::test_walrus {}",
                each_shape(26, "all", "with a :=")
            ),
        ],
        "an if, a second for, a list and a := each name the shape; an all() of a name, or of an \
         item that is no check, keeps the words of one comparison"
    );
}

#[test]
fn an_expectation_at_the_end_of_the_body_of_a_with_keeps_the_rules_of_an_expectation() {
    let (tests, refused) = judged(
        "test_with_rules.py",
        concat!(
            "def test_inside():\n    with x:\n        y = 1\n        assert y\n\n",
            "def test_nested():\n    with x:\n        with z:\n            assert y == f()\n",
            "    assert y == 1\n\n",
            "def test_held():\n    with x:\n        assert y == 1\n    assert y == 2\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_held"],
        "an expectation inside a with and one after it, each one comparison of a literal"
    );
    assert_eq!(
        refused,
        [
            format!("test_m.py::test_inside {}", not_one(4)),
            format!("test_m.py::test_nested {}", computes(9, "f()")),
        ],
        "an assert at the end of a with's body, at any depth, is an expectation"
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
            format!("test_m.py::test_input {}", input(13, "n")),
            format!("test_m.py::test_both {}", computes(16, "g( 1)")),
        ],
        "a constant, a call, an f-string with a placeholder, and two readings are each \
         refused, by the text on the right, on one line, and an input by its own words"
    );
}

#[test]
fn a_test_that_reads_a_private_name_runs_with_a_notice_for_each_line() {
    let text = concat!(
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
    );

    let (_, refused) = judged("test_private.py", text);

    assert!(refused.is_empty(), "nothing refused: {refused:?}");
    let each = |test: &str, notices: &[String]| (format!("test_m.py::{test}"), notices.to_vec());
    assert_eq!(
        noticed("test_private_notices.py", text),
        [
            each("test_attribute", &[private(13, "_flush")]),
            each(
                "test_imported",
                &[private(17, "lib._hidden"), private(18, "lib._impl.Thing")]
            ),
            each("test_module", &[private(21, "_thread")]),
            each("test_relative", &[private(24, "._sibling")]),
            each(
                "test_inside",
                &[private(28, "lib._x"), private(29, "lib._inner")]
            ),
            each("test_store", &[private(34, "_x"), private(35, "_y")]),
            each("test_free", &[]),
            each("test_once", &[private(44, "_a")]),
        ],
        "an attribute, an import from another module, and a private part of a module's path \
         each give a notice, once a line, in source order; a name of the test's own module, \
         a local, a dunder and a public import give none"
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
            "test_m.py::test_mixed [foreign-decorator] the decorator dec is not purba's, \
             and purba cannot see what it does; {}; {}",
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

/// The suppressed verdicts of each selected test and of each refused one, for
/// the module that `text` holds: the test, the check, its words, and the reason.
fn suppressed(name: &str, text: &str) -> Vec<(String, String, String, String)> {
    let file = write(name, text);
    let module = purba_parsing::parse(&file).expect("the module parses");
    let selected = purba_selection::select(Path::new("test_m.py"), &module);
    let (tests, refused) = (selected.tests(), selected.refused());
    tests
        .iter()
        .chain(refused.iter().map(purba_selection::Refused::test))
        .flat_map(|test| {
            test.suppressed().iter().map(move |verdict| {
                (
                    test.to_string(),
                    verdict.check.to_owned(),
                    verdict.words.clone(),
                    verdict.reason.clone(),
                )
            })
        })
        .collect()
}

/// The fault of the module's suppressions, for the module that `text` holds.
fn module_fault(name: &str, text: &str) -> Option<String> {
    let file = write(name, text);
    let module = purba_parsing::parse(&file).expect("the module parses");
    purba_selection::select(Path::new("test_m.py"), &module)
        .fault()
        .map(ToOwned::to_owned)
}

/// One suppressed verdict of `test_m.py::<test>`.
fn verdict(test: &str, check: &str, words: &str, reason: &str) -> (String, String, String, String) {
    (
        format!("test_m.py::{test}"),
        check.to_owned(),
        words.to_owned(),
        reason.to_owned(),
    )
}

const SUPPRESSED: &str = concat!(
    "import json\n\n\n",
    "# purba: suppress[computed-expected] the payload is its own expected value\n",
    "def test_round_trip():\n",
    "    obj = {\"a\": 1}\n",
    "    assert json.loads(json.dumps(obj)) == obj\n\n\n",
    "# purba: suppress[foreign-decorator] it only wraps the test\n",
    "@keep\n",
    "def test_decorated():\n",
    "    assert 1 == 1\n\n\n",
    "# purba: suppress[computed-expected] the parse is its own expected value\n",
    "# an ordinary comment between them\n",
    "# purba: suppress[not-one-comparison] a subset reads best as one call\n",
    "def test_two():\n",
    "    got = json.loads(\"[1]\")\n",
    "    assert got == json.loads(\"[1]\")\n",
    "    assert got.issubset({1})\n",
);

#[test]
fn a_suppression_above_a_def_or_its_decorator_turns_a_practice_check_into_a_warning() {
    let (tests, refused) = judged("test_suppressed.py", SUPPRESSED);

    assert_eq!(
        tests,
        [
            "test_m.py::test_round_trip",
            "test_m.py::test_decorated",
            "test_m.py::test_two",
        ],
        "each suppressed test runs"
    );
    assert!(refused.is_empty(), "nothing refused: {refused:?}");
    assert_eq!(
        suppressed("test_suppressed_verdicts.py", SUPPRESSED),
        [
            verdict(
                "test_round_trip",
                "computed-expected",
                "line 7 computes obj, and an expected value is a literal",
                "the payload is its own expected value",
            ),
            verdict(
                "test_decorated",
                "foreign-decorator",
                "the decorator keep is not purba's, and purba cannot see what it does",
                "it only wraps the test",
            ),
            verdict(
                "test_two",
                "computed-expected",
                "line 21 computes json.loads(\"[1]\"), and an expected value is a literal",
                "the parse is its own expected value",
            ),
            verdict(
                "test_two",
                "not-one-comparison",
                "line 22 is not one comparison, \
                 and an expectation compares a reading by one operator, \
                 or reads not x, isinstance(x, T), x.startswith(y) or x.endswith(y)",
                "a subset reads best as one call",
            ),
        ],
        "each verdict with its check, its words and its reason; \
         suppressions stack, and an ordinary comment between them keeps them attached"
    );
}

#[test]
fn a_suppressed_verdict_of_two_lines_is_one_verdict_with_both_clauses() {
    let text = concat!(
        "# purba: suppress[computed-expected] both sides are digests\n",
        "def test_two_lines():\n",
        "    assert a == b\n",
        "    assert c == d\n",
    );

    assert_eq!(
        suppressed("test_two_lines.py", text),
        [verdict(
            "test_two_lines",
            "computed-expected",
            "line 3 computes b, and an expected value is a literal; \
             line 4 computes d, and an expected value is a literal",
            "both sides are digests",
        )],
        "one verdict for one check of one test"
    );
}

#[test]
fn a_suppression_of_the_module_reaches_each_test_of_it() {
    let text = concat!(
        "\"\"\"The module's docstring.\"\"\"\n",
        "# purba: suppress-module[computed-expected] each side is a digest\n",
        "import hashlib\n\n\n",
        "def test_one():\n",
        "    assert a == b\n\n\n",
        "def test_two():\n",
        "    assert c == d\n\n\n",
        "def test_clean():\n",
        "    assert c == 1\n",
    );

    let (tests, refused) = judged("test_module_suppression.py", text);

    assert_eq!(
        tests,
        [
            "test_m.py::test_one",
            "test_m.py::test_two",
            "test_m.py::test_clean"
        ],
        "each test runs"
    );
    assert!(refused.is_empty(), "nothing refused: {refused:?}");
    assert_eq!(
        suppressed("test_module_suppression_verdicts.py", text),
        [
            verdict(
                "test_one",
                "computed-expected",
                "line 7 computes b, and an expected value is a literal",
                "each side is a digest",
            ),
            verdict(
                "test_two",
                "computed-expected",
                "line 11 computes d, and an expected value is a literal",
                "each side is a digest",
            ),
        ],
        "a verdict for each test that breaks the check, and none for a clean test"
    );
    assert_eq!(
        module_fault("test_module_suppression_fault.py", text),
        None,
        "a suppression above the first statement after the docstring is in its place"
    );
}

#[test]
fn a_suppression_that_does_not_reach_every_reason_leaves_the_test_refused_with_its_verdict() {
    let text = concat!(
        "# purba: suppress[computed-expected] the cache is its own expected value\n",
        "def test_computed_and_bare():\n",
        "    got = cache.slots\n",
        "    assert got == expected\n",
        "    assert got\n",
    );

    let (tests, refused) = judged("test_partial.py", text);

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [format!("test_m.py::test_computed_and_bare {}", not_one(5))],
        "the reason that no suppression reaches still refuses the test"
    );
    assert_eq!(
        suppressed("test_partial_verdicts.py", text),
        [verdict(
            "test_computed_and_bare",
            "computed-expected",
            "line 4 computes expected, and an expected value is a literal",
            "the cache is its own expected value",
        )],
        "the refused test keeps its suppressed verdict"
    );
}

#[test]
fn no_suppression_reaches_a_test_that_purba_cannot_call() {
    let text = concat!(
        "# purba: suppress[computed-expected] the payload is its own expected value\n",
        "def test_fixture(tmp_path):\n",
        "    assert got == len(\"xy\")\n",
    );

    let (_, refused) = judged("test_cannot_call.py", text);

    assert_eq!(
        refused,
        [format!(
            "test_m.py::test_fixture the parameter tmp_path has no default, \
             and purba cannot see which fixture builds it; {}",
            computes(3, "len(\"xy\")")
        )],
        "the reason with no name keeps no bracket, and the suppressed check still refuses"
    );
    assert!(
        suppressed("test_cannot_call_verdicts.py", text).is_empty(),
        "no verdict is suppressed"
    );
}

#[test]
fn a_test_carries_each_suppression_that_reaches_it_for_the_checks_of_the_run() {
    let text = concat!(
        "# purba: suppress-module[second-path] the act branches on the platform\n",
        "import sys\n\n\n",
        "# purba: suppress[assert-before-expectation] the act checks its own input\n",
        "def test_act():\n",
        "    data = b\"abc\"\n",
        "    assert data == b\"abc\"\n",
        "    assert data.upper() == b\"ABC\"\n",
    );
    let file = write("test_run_checks.py", text);
    let module = purba_parsing::parse(&file).expect("the module parses");

    let selected = purba_selection::select(Path::new("test_m.py"), &module);
    let (tests, refused) = (selected.tests(), selected.refused());

    assert!(
        refused.is_empty(),
        "nothing refused: {:?}",
        refused.iter().map(ToString::to_string).collect::<Vec<_>>()
    );
    let selected = tests.first().expect("the module holds one test");
    let suppressions: Vec<(&str, &str)> = selected
        .suppressions()
        .iter()
        .map(|suppression| (suppression.check, suppression.reason.as_str()))
        .collect();
    assert_eq!(
        suppressions,
        [
            ("assert-before-expectation", "the act checks its own input"),
            ("second-path", "the act branches on the platform"),
        ],
        "the test's own suppressions, then the module's"
    );
    assert!(
        selected.suppressed().is_empty(),
        "selection suppresses no verdict of the run"
    );
}

#[test]
fn each_fault_of_a_suppression_directly_above_a_test_refuses_that_test() {
    let (tests, refused) = judged(
        "test_broken.py",
        concat!(
            "\"\"\"Suppressions that purba refuses.\"\"\"\n",
            "import json\n\n\n",
            "# purba: suppress[returned-value] the caller reads it\n",
            "def test_returns():\n",
            "    got = len(\"ab\")\n",
            "    assert got == 2\n",
            "    return got\n\n\n",
            "# purba: suppress[no-expectation] it only logs\n",
            "def test_logs():\n",
            "    print(\"hello\")\n\n\n",
            "# purba: suppress[computed-expected]\n",
            "def test_no_reason():\n",
            "    obj = {\"a\": 1}\n",
            "    assert json.loads(json.dumps(obj)) == obj\n\n\n",
            "# purba: suppress[slow-test] it takes a second\n",
            "def test_unknown():\n",
            "    got = len(\"ab\")\n",
            "    assert got == 2\n\n\n",
            "# purba: suppress computed-expected the payload is its own expected value\n",
            "def test_unread():\n",
            "    got = len(\"ab\")\n",
            "    assert got == 2\n\n\n",
            "# purba: suppress[computed-expected] the cache is its own expected value\n",
            "# purba: suppress[computed-expected] the slots are their own expected value\n",
            "def test_twice():\n",
            "    got = cache.slots\n",
            "    assert got == 1\n\n\n",
            "# purba: suppress-module[computed-expected] the cache is its own expected value\n",
            "def test_module_below():\n",
            "    got = len(\"ab\")\n",
            "    assert got == 2\n\n\n",
            "# purba: suppress[private-name] the cache has no public view\n",
            "def test_private():\n",
            "    got = cache._slots\n",
            "    assert got == 1\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            "test_m.py::test_returns line 5 suppresses returned-value, \
             and a suppression cannot help: purba reads no expectation of such a test; \
             [returned-value] the test returns a value, and purba does not read it",
            "test_m.py::test_logs line 12 suppresses no-expectation, \
             and an outcome cannot stand without that check",
            "test_m.py::test_no_reason line 17 suppresses computed-expected with no reason, \
             and a suppression gives one; \
             [computed-expected] line 20 computes obj, and an expected value is a literal",
            "test_m.py::test_unknown line 23 suppresses slow-test, which is no strict check",
            "test_m.py::test_unread line 29 is a purba comment that reads as no suppression, \
             and a suppression reads # purba: suppress[<check>] <reason>",
            "test_m.py::test_twice line 36 suppresses computed-expected again, \
             and a site names each check once",
            "test_m.py::test_module_below line 42 suppresses the module below its first statement",
            "test_m.py::test_private line 48 suppresses private-name, which is no strict check",
        ],
        "each fault of a suppression, then each check that no suppression reaches"
    );
}

#[test]
fn a_suppression_of_a_check_of_the_never_suppressed_tier_is_refused_by_its_tier() {
    for (check, words) in [
        (
            "no-expectation",
            "and an outcome cannot stand without that check",
        ),
        (
            "readings-differ",
            "and an outcome cannot stand without that check",
        ),
        (
            "assert-in-reading",
            "and an outcome cannot stand without that check",
        ),
        (
            "hook-replaced",
            "and an outcome cannot stand without that check",
        ),
        ("no-item", "and an outcome cannot stand without that check"),
        (
            "def-under-block",
            "and a suppression cannot help: purba reads no expectation of such a test",
        ),
        (
            "name-without-def",
            "and a suppression cannot help: purba reads no expectation of such a test",
        ),
        (
            "returned-value",
            "and a suppression cannot help: purba reads no expectation of such a test",
        ),
    ] {
        let text =
            format!("# purba: suppress[{check}] a reason\ndef test_x():\n    assert 1 == 1\n");

        let (tests, refused) = judged(&format!("test_never_{}.py", check.replace('-', "_")), &text);

        assert!(tests.is_empty(), "{check}: no test: {tests:?}");
        assert_eq!(
            refused,
            [format!(
                "test_m.py::test_x line 1 suppresses {check}, {words}"
            )],
            "{check}: refused by its tier"
        );
    }
}

#[test]
fn a_suppression_of_each_check_of_the_practice_tier_is_in_its_place() {
    for check in [
        "foreign-decorator",
        "foreign-framework",
        "name-bound-again",
        "not-one-comparison",
        "computed-expected",
        "second-path",
        "assert-before-expectation",
        "swallowed-raise",
        "iterator-read",
        "type-alone",
    ] {
        let text =
            format!("# purba: suppress[{check}] a reason\ndef test_x():\n    assert 1 == 1\n");

        let (tests, refused) = judged(
            &format!("test_practice_{}.py", check.replace('-', "_")),
            &text,
        );

        assert_eq!(tests, ["test_m.py::test_x"], "{check}: the test runs");
        assert!(refused.is_empty(), "{check}: nothing refused: {refused:?}");
    }
}

#[test]
fn a_fault_of_a_suppression_that_sits_above_no_test_is_the_modules() {
    let cases = [
        (
            "test_above_helper.py",
            concat!(
                "\"\"\"A suppression above a helper.\"\"\"\n",
                "import hashlib\n\n\n",
                "# purba: suppress[computed-expected] digests match\n",
                "def check_digest(data):\n",
                "    return hashlib.sha256(data).hexdigest()\n\n\n",
                "def test_digest():\n",
                "    assert check_digest(b\"abc\") == \"digest\"\n",
            ),
            "line 5 suppresses above check_digest, which is no test",
        ),
        (
            "test_above_blank.py",
            concat!(
                "# purba: suppress[computed-expected] digests match\n\n",
                "def test_digest():\n",
                "    assert got == 1\n",
            ),
            "line 1 suppresses above no test",
        ),
        (
            "test_in_body.py",
            concat!(
                "def test_digest():\n",
                "    # purba: suppress[computed-expected] digests match\n",
                "    assert got == 1\n",
            ),
            "line 2 suppresses above no test",
        ),
        (
            "test_trailing.py",
            concat!(
                "x = 1  # purba: suppress[computed-expected] digests match\n",
                "def test_digest():\n",
                "    assert got == 1\n",
            ),
            "line 1 suppresses above no test",
        ),
        (
            "test_module_low.py",
            concat!(
                "import hashlib\n",
                "# purba: suppress-module[computed-expected] digests match\n",
                "import json\n",
            ),
            "line 2 suppresses the module below its first statement",
        ),
        (
            "test_module_broken.py",
            concat!("# purba: skip this module\n", "import json\n",),
            "line 1 is a purba comment that reads as no suppression, \
             and a suppression reads # purba: suppress[<check>] <reason>",
        ),
        (
            "test_module_faults.py",
            concat!(
                "# purba: suppress-module[slow-test] it takes a second\n",
                "# purba: suppress-module[computed-expected]\n",
                "# purba: suppress-module[no-expectation] it only logs\n",
                "# purba: suppress-module[foreign-decorator] one\n",
                "# purba: suppress-module[foreign-decorator] two\n",
                "import json\n",
            ),
            "line 1 suppresses slow-test, which is no strict check; \
             line 2 suppresses computed-expected with no reason, and a suppression gives one; \
             line 3 suppresses no-expectation, and an outcome cannot stand without that check; \
             line 5 suppresses foreign-decorator again, and a site names each check once",
        ),
    ];
    for (name, text, fault) in cases {
        assert_eq!(
            module_fault(name, text).as_deref(),
            Some(fault),
            "{name}: the module's fault"
        );
    }
}

#[test]
fn a_comment_that_is_not_purbas_and_a_string_are_no_suppression() {
    let text = concat!(
        "# see purba: the docs\n",
        "import json\n\n\n",
        "def test_text():\n",
        "    got = \"# purba: suppress[computed-expected] no\"\n",
        "    assert got == \"x\"\n",
    );

    assert_eq!(
        module_fault("test_not_purbas.py", text),
        None,
        "neither reads as a purba comment"
    );
    assert!(
        suppressed("test_not_purbas_verdicts.py", text).is_empty(),
        "nothing is suppressed"
    );
}

/// The words of a fault of a patch of `target`.
fn patched(line: usize, target: &str) -> String {
    format!(
        "[patched-name] line {line} patches {target}, and a stand-in enters a test as an argument"
    )
}

/// The words of a fault of a mock with no spec.
fn without_spec(line: usize) -> String {
    format!(
        "[mock-without-spec] line {line} makes a mock with no spec, \
         and a mock has the spec of what it stands for"
    )
}

#[test]
fn a_patch_in_a_test_body_is_refused_by_its_target() {
    let (tests, refused) = judged(
        "test_patched.py",
        concat!(
            "import os\n",
            "import unittest.mock\n",
            "from os import environ\n",
            "from unittest import mock\n",
            "from unittest.mock import patch\n",
            "\n",
            "import lending\n",
            "import lending as books\n",
            "from lending import Library\n",
            "\n",
            "\n",
            "def test_object():\n",
            "    with mock.patch.object(lending, \"Library\", object):\n",
            "        got = 1\n",
            "    assert got == 1\n",
            "\n",
            "\n",
            "def test_string():\n",
            "    patcher = patch(\"lending.borrow\", print)\n",
            "    patcher.start()\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_dict_and_multiple():\n",
            "    with unittest.mock.patch.dict(os.environ, {\"EDITOR\": \"vi\"}):\n",
            "        with mock.patch.multiple(books, Library=object, borrow=print):\n",
            "            got = 1\n",
            "    assert got == 1\n",
            "\n",
            "\n",
            "def test_assign():\n",
            "    lending.Library = object\n",
            "    environ[\"EDITOR\"] = \"vi\"\n",
            "    del books.borrow\n",
            "    Library.lend += 1\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_builtins():\n",
            "    setattr(lending, \"Library\", object)\n",
            "    delattr(os, \"sep\")\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_nested():\n",
            "    def swap():\n",
            "        lending.Library = object\n",
            "\n",
            "    swap()\n",
            "    assert 1 == 1\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            format!("test_m.py::test_object {}", patched(13, "lending.Library")),
            format!("test_m.py::test_string {}", patched(19, "lending.borrow")),
            format!(
                "test_m.py::test_dict_and_multiple {}; {}; {}",
                patched(25, "os.environ"),
                patched(26, "lending.Library"),
                patched(26, "lending.borrow")
            ),
            format!(
                "test_m.py::test_assign {}; {}; {}; {}",
                patched(32, "lending.Library"),
                patched(33, "os.environ[\"EDITOR\"]"),
                patched(34, "lending.borrow"),
                patched(35, "lending.Library.lend")
            ),
            format!(
                "test_m.py::test_builtins {}; {}",
                patched(40, "lending.Library"),
                patched(41, "os.sep")
            ),
            format!("test_m.py::test_nested {}", patched(47, "lending.Library")),
        ],
        "each patch of unittest.mock, in a with or not, and each assignment, del, setattr() \
         and delattr() of a name or an item of an import, by its target through the imports"
    );
}

#[test]
fn a_change_to_a_local_a_patch_that_is_not_unittest_mocks_and_a_decorator_are_no_patch() {
    let (tests, refused) = judged(
        "test_not_patched.py",
        concat!(
            "from unittest import mock\n",
            "\n",
            "import lending\n",
            "from mylib import patch\n",
            "\n",
            "\n",
            "class Box:\n",
            "    pass\n",
            "\n",
            "\n",
            "def test_local():\n",
            "    box = Box()\n",
            "    box.item = 1\n",
            "    setattr(box, \"item\", 2)\n",
            "    table = {}\n",
            "    table[\"k\"] = 1\n",
            "    assert box.item == 2\n",
            "\n",
            "\n",
            "def test_shadow():\n",
            "    lending = Box()\n",
            "    lending.Library = object\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_other_patch():\n",
            "    patch(\"lending.borrow\")\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "@mock.patch.object(lending, \"Library\", object)\n",
            "def test_decorated():\n",
            "    assert 1 == 1\n",
        ),
    );

    assert_eq!(
        tests,
        [
            "test_m.py::test_local",
            "test_m.py::test_shadow",
            "test_m.py::test_other_patch",
        ],
        "a local's attribute and item, a local that hides an import, and another patch are free"
    );
    assert_eq!(
        refused,
        [
            "test_m.py::test_decorated [foreign-decorator] the decorator mock.patch.object \
          is not purba's, and purba cannot see what it does"
        ],
        "a patch as a decorator stays a foreign decorator"
    );
}

#[test]
fn a_call_of_a_setattr_that_the_module_defines_is_no_patch() {
    let (tests, refused) = judged(
        "test_own_setattr.py",
        concat!(
            "import lending\n",
            "\n",
            "\n",
            "class Box:\n",
            "    pass\n",
            "\n",
            "\n",
            "def setattr(target, name, value):\n",
            "    return (target, name, value)\n",
            "\n",
            "\n",
            "def delattr(target, name):\n",
            "    return (target, name)\n",
            "\n",
            "\n",
            "def test_record():\n",
            "    got = setattr(lending, \"shelf\", 1)\n",
            "    assert got[1] == \"shelf\"\n",
            "\n",
            "\n",
            "def test_forget():\n",
            "    got = delattr(lending, \"shelf\")\n",
            "    assert got[1] == \"shelf\"\n",
            "\n",
            "\n",
            "def test_class():\n",
            "    Box.item = 1\n",
            "    assert Box.item == 1\n",
        ),
    );

    assert!(refused.is_empty(), "no test is refused: {refused:?}");
    assert_eq!(
        tests,
        [
            "test_m.py::test_record",
            "test_m.py::test_forget",
            "test_m.py::test_class",
        ],
        "a call of the module's own setattr or delattr, and a store to its own class, patch nothing"
    );
}

#[test]
fn a_call_of_a_setattr_that_the_module_assigns_is_no_patch() {
    let (tests, refused) = judged(
        "test_assigned_setattr.py",
        concat!(
            "import lending\n",
            "\n",
            "setattr = lambda target, name, value: (target, name, value)\n",
            "\n",
            "\n",
            "def test_record():\n",
            "    got = setattr(lending, \"shelf\", 1)\n",
            "    assert got[1] == \"shelf\"\n",
        ),
    );

    assert!(refused.is_empty(), "no test is refused: {refused:?}");
    assert_eq!(
        tests,
        ["test_m.py::test_record"],
        "a name that the module binds with no def is the module's own, not the builtin"
    );
}

#[test]
fn a_read_of_a_private_name_that_the_test_imports_gives_one_notice_at_its_import() {
    let text = concat!(
        "def test_inside():\n",
        "    from lib import _x\n",
        "    _x()\n",
        "    assert 1 == 1\n",
    );

    assert_eq!(
        noticed("test_private_read.py", text),
        [(
            "test_m.py::test_inside".to_owned(),
            vec![private(2, "lib._x")]
        )],
        "the import gives the notice, and a read of the name that it binds gives none"
    );
}

#[test]
fn a_mock_with_no_spec_is_refused_once_a_line() {
    let (tests, refused) = judged(
        "test_mocks.py",
        concat!(
            "import unittest.mock\n",
            "from unittest import mock\n",
            "from unittest.mock import AsyncMock, MagicMock, Mock, ",
            "NonCallableMagicMock, NonCallableMock\n",
            "\n",
            "import lending\n",
            "\n",
            "\n",
            "def test_bare():\n",
            "    a = Mock()\n",
            "    b = mock.MagicMock(return_value=1)\n",
            "    c = NonCallableMock(spec=None)\n",
            "    d = NonCallableMagicMock(name=\"d\")\n",
            "    e = unittest.mock.AsyncMock()\n",
            "    assert a == 1\n",
            "\n",
            "\n",
            "def test_once_a_line():\n",
            "    pair = (Mock(), MagicMock())\n",
            "    assert pair == 1\n",
            "\n",
            "\n",
            "def test_specced(options={}):\n",
            "    a = Mock(spec=lending.Library)\n",
            "    b = MagicMock(lending.Library)\n",
            "    c = AsyncMock(spec_set=lending.borrow)\n",
            "    d = mock.create_autospec(lending.Library)\n",
            "    e = Mock(**options)\n",
            "    assert a == 1\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_specced"],
        "a spec by keyword or by position, a spec_set, create_autospec() and **kwargs pass"
    );
    assert_eq!(
        refused,
        [
            format!(
                "test_m.py::test_bare {}; {}; {}; {}; {}",
                without_spec(9),
                without_spec(10),
                without_spec(11),
                without_spec(12),
                without_spec(13)
            ),
            format!("test_m.py::test_once_a_line {}", without_spec(18)),
        ],
        "each of the five mock classes with no spec, or a spec of None, once a line"
    );
}

#[test]
fn a_patch_that_gives_no_value_is_refused_by_each_name() {
    let (tests, refused) = judged(
        "test_no_value.py",
        concat!(
            "from unittest import mock\n",
            "\n",
            "import lending\n",
            "\n",
            "\n",
            "def test_no_value():\n",
            "    with mock.patch.object(lending, \"Library\"):\n",
            "        got = 1\n",
            "    assert got == 1\n",
            "\n",
            "\n",
            "def test_default():\n",
            "    with mock.patch(\"lending.borrow\", mock.DEFAULT):\n",
            "        with mock.patch.multiple(lending, Library=mock.DEFAULT):\n",
            "            got = 1\n",
            "    assert got == 1\n",
            "\n",
            "\n",
            "def test_spec_given():\n",
            "    with mock.patch.object(lending, \"Library\", autospec=True):\n",
            "        with mock.patch(\"lending.borrow\", spec=True):\n",
            "            with mock.patch(\"lending.today\", new_callable=list):\n",
            "                got = 1\n",
            "    assert got == 1\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            format!(
                "test_m.py::test_no_value {}; {}",
                patched(7, "lending.Library"),
                without_spec(7)
            ),
            format!(
                "test_m.py::test_default {}; {}; {}; {}",
                patched(13, "lending.borrow"),
                without_spec(13),
                patched(14, "lending.Library"),
                without_spec(14)
            ),
            format!(
                "test_m.py::test_spec_given {}; {}; {}",
                patched(20, "lending.Library"),
                patched(21, "lending.borrow"),
                patched(22, "lending.today")
            ),
        ],
        "a patch with no value or DEFAULT makes a mock with no spec; \
         autospec, spec and new_callable leave one refusal"
    );
}

const PATCH_SUPPRESSED: &str = concat!(
    "from unittest import mock\n",
    "\n",
    "import lending\n",
    "\n",
    "\n",
    "# purba: suppress[patched-name] the library has no seam yet\n",
    "def test_half():\n",
    "    with mock.patch.object(lending, \"Library\"):\n",
    "        got = 1\n",
    "    assert got == 1\n",
    "\n",
    "\n",
    "# purba: suppress[patched-name] the library has no seam yet\n",
    "# purba: suppress[mock-without-spec] the stub answers one call\n",
    "def test_both():\n",
    "    with mock.patch.object(lending, \"Library\"):\n",
    "        got = 1\n",
    "    assert got == 1\n",
);

#[test]
fn a_suppression_of_patched_name_alone_leaves_the_mock_without_spec() {
    let (tests, refused) = judged("test_patch_suppressed.py", PATCH_SUPPRESSED);

    assert_eq!(
        tests,
        ["test_m.py::test_both"],
        "both checks suppressed, the test runs"
    );
    assert_eq!(
        refused,
        [format!("test_m.py::test_half {}", without_spec(8))],
        "the mock with no spec still refuses the test"
    );
    let patch = "the library has no seam yet";
    assert_eq!(
        suppressed("test_patch_suppressed_verdicts.py", PATCH_SUPPRESSED),
        [
            verdict(
                "test_both",
                "patched-name",
                "line 16 patches lending.Library, and a stand-in enters a test as an argument",
                patch,
            ),
            verdict(
                "test_both",
                "mock-without-spec",
                "line 16 makes a mock with no spec, and a mock has the spec of what it stands for",
                "the stub answers one call",
            ),
            verdict(
                "test_half",
                "patched-name",
                "line 8 patches lending.Library, and a stand-in enters a test as an argument",
                patch,
            ),
        ],
        "both checks are of the practice tier"
    );
}

#[test]
fn a_star_argument_may_give_a_spec_so_it_gives_no_mock_without_spec() {
    let (tests, refused) = judged(
        "test_star.py",
        concat!(
            "from unittest import mock\n",
            "\n",
            "import lending\n",
            "\n",
            "\n",
            "def test_multiple_star(extra={}):\n",
            "    with mock.patch.multiple(lending, borrow=mock.DEFAULT, **extra):\n",
            "        got = 1\n",
            "    assert got == 1\n",
            "\n",
            "\n",
            "def test_object_star(extra={}):\n",
            "    with mock.patch.object(lending, \"Library\", **extra):\n",
            "        got = 1\n",
            "    assert got == 1\n",
            "\n",
            "\n",
            "def test_mock_star(args=()):\n",
            "    stub = mock.Mock(*args)\n",
            "    assert stub == 1\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_mock_star"],
        "a mock class with a star argument may get its spec from it"
    );
    assert_eq!(
        refused,
        [
            format!(
                "test_m.py::test_multiple_star {}",
                patched(7, "lending.borrow")
            ),
            format!(
                "test_m.py::test_object_star {}",
                patched(13, "lending.Library")
            ),
        ],
        "a patch with a star argument is refused as a patch alone"
    );
}

/// The words of each unused suppression of the module `text`. Selection judges
/// each refused test, and execution judges each selected test. Execution read
/// the paths of a test when `judged` gives true for its function, and its
/// checks gave the warnings that `warned` names.
fn unused(
    name: &str,
    text: &str,
    judged: impl Fn(&str) -> bool,
    warned: impl Fn(&str) -> Vec<&'static str>,
) -> Vec<String> {
    let file = write(name, text);
    let module = purba_parsing::parse(&file).expect("the module parses");
    let mut selected = purba_selection::select(Path::new(name), &module);
    for test in selected.tests().to_vec() {
        selected.judge(&test, &warned(test.function()), judged(test.function()));
    }
    selected.unused()
}

#[test]
fn a_suppression_that_selection_judged_and_that_suppressed_nothing_is_unused() {
    let text = concat!(
        "# purba: suppress-module[foreign-decorator] one test is wrapped\n",
        "# purba: suppress-module[computed-expected] no test computes\n",
        "import os\n\n\n",
        "# purba: suppress[not-one-comparison] the test once compared twice\n",
        "def test_plain():\n",
        "    value = 1\n",
        "    assert value == 1\n\n\n",
        "@keep\n",
        "def test_wrapped():\n",
        "    value = os.sep\n",
        "    assert value == \"/\"\n",
    );

    assert_eq!(
        unused("test_unused_selection.py", text, |_| true, |_| Vec::new()),
        [
            "line 2 suppresses computed-expected, which no test of the module breaks",
            "line 6 suppresses not-one-comparison, which test_plain does not break",
        ],
        "the module's suppression that one test used passes; the others are unused, by line"
    );
}

#[test]
fn a_module_suppression_that_each_test_names_itself_reaches_no_test_and_is_unused() {
    let text = concat!(
        "# purba: suppress-module[foreign-decorator] the module wraps its tests\n",
        "import os\n\n\n",
        "# purba: suppress[foreign-decorator] the test is wrapped\n",
        "@keep\n",
        "def test_wrapped():\n",
        "    value = os.sep\n",
        "    assert value == \"/\"\n",
    );

    assert_eq!(
        unused("test_unused_shadowed.py", text, |_| true, |_| Vec::new()),
        ["line 1 suppresses foreign-decorator, which no test of the module breaks"],
        "the test's own suppression is used, so the module's reaches no test"
    );
}

#[test]
fn a_refused_test_judges_its_suppressions_unless_purba_cannot_call_it() {
    let text = concat!(
        "# purba: suppress[foreign-decorator] purba cannot call the test\n",
        "async def test_async():\n",
        "    value = 1\n",
        "    assert value == 1\n\n\n",
        "# purba: suppress[foreign-decorator] the test was once wrapped\n",
        "def test_computes():\n",
        "    value = 1\n",
        "    assert value == int(\"1\")\n",
    );

    assert_eq!(
        unused("test_unused_refused.py", text, |_| true, |_| Vec::new()),
        ["line 7 suppresses foreign-decorator, which test_computes does not break"],
        "the test refused for a computed value judges its suppression; the async test does not"
    );
}

#[test]
fn a_suppression_of_a_check_of_execution_is_judged_only_where_execution_read_it() {
    let text = concat!(
        "# purba: suppress-module[swallowed-raise] one act may swallow a raise\n\n\n",
        "# purba: suppress[second-path] the act once branched\n",
        "def test_branch():\n",
        "    value = 1\n",
        "    assert value == 1\n\n\n",
        "def test_raises():\n",
        "    value = int(\"x\")\n",
        "    assert value == 1\n",
    );
    let read = |function: &str| function == "test_branch";

    assert_eq!(
        unused("test_unused_execution.py", text, read, |_| Vec::new()),
        ["line 4 suppresses second-path, which test_branch does not break"],
        "the module's suppression waits for test_raises, which execution did not read"
    );
    assert_eq!(
        unused(
            "test_unused_execution.py",
            text,
            |_| true,
            |_| vec!["second-path"]
        ),
        ["line 1 suppresses swallowed-raise, which no test of the module breaks"],
        "a warning uses the test's suppression; once each test is read, the module's is unused"
    );
}

#[test]
fn a_second_suppression_above_a_test_that_the_module_defines_twice_is_unused() {
    let text = concat!(
        "import os\n\n\n",
        "# purba: suppress[foreign-decorator] the first def is wrapped\n",
        "@keep\n",
        "def test_wrapped():\n",
        "    value = os.sep\n",
        "    assert value == \"/\"\n\n\n",
        "# purba: suppress[foreign-decorator] the second def is wrapped\n",
        "@keep\n",
        "def test_wrapped():\n",
        "    value = os.sep\n",
        "    assert value == \"/\"\n",
    );

    assert_eq!(
        unused("test_unused_twice.py", text, |_| true, |_| Vec::new()),
        ["line 11 suppresses foreign-decorator, which test_wrapped does not break"],
        "only the first suppression of a check reaches the test"
    );
}

/// The words of a fault of an import of another test framework.
fn imported(line: usize, module: &str) -> String {
    format!(
        "[foreign-framework] line {line} imports {module}, \
         and a test module imports no other test framework"
    )
}

#[test]
fn an_import_of_pytest_outside_a_test_reaches_every_test_of_the_module() {
    let (tests, refused) = judged(
        "test_framework_module.py",
        concat!(
            "import os\n",
            "import pytest\n",
            "import pytest as pt\n",
            "from _pytest.monkeypatch import MonkeyPatch\n",
            "from pytest import *\n",
            "import unittest.mock\n",
            "from typing import TYPE_CHECKING\n",
            "\n",
            "try:\n",
            "    import _pytest.python_api\n",
            "except ImportError:\n",
            "    pass\n",
            "\n",
            "if TYPE_CHECKING:\n",
            "    from pytest import fixture\n",
            "\n",
            "\n",
            "def helper():\n",
            "    import pytest.mark\n",
            "\n",
            "\n",
            "class Shop:\n",
            "    import _pytest\n",
            "\n",
            "\n",
            "def test_a():\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_b():\n",
            "    assert 2 == 2\n",
        ),
    );
    let reasons = [
        imported(2, "pytest"),
        imported(3, "pytest"),
        imported(4, "_pytest.monkeypatch"),
        imported(5, "pytest"),
        imported(10, "_pytest.python_api"),
        imported(15, "pytest"),
        imported(19, "pytest.mark"),
        imported(23, "_pytest"),
    ]
    .join("; ");

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            format!("test_m.py::test_a {reasons}"),
            format!("test_m.py::test_b {reasons}"),
        ],
        "each import line of pytest or _pytest outside a test, in a block, under TYPE_CHECKING, \
         in a helper or in a class, reaches every test, in line order, and unittest gives none"
    );
}

#[test]
fn an_import_or_a_known_call_in_a_test_reaches_that_test_only() {
    let (tests, refused) = judged(
        "test_framework_calls.py",
        concat!(
            "import importlib\n",
            "import importlib.util\n",
            "import pkgutil\n",
            "import runpy\n",
            "from importlib import import_module as load\n",
            "from importlib.resources import files\n",
            "from unittest import mock\n",
            "\n",
            "\n",
            "def test_statement():\n",
            "    import pytest\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_calls():\n",
            "    importlib.import_module(\"pytest\")\n",
            "    load(\"_pytest.fixtures\")\n",
            "    __import__(\"pytest\")\n",
            "    importlib.__import__(\"pytest\")\n",
            "    importlib.util.find_spec(\"pytest.mark\")\n",
            "    pkgutil.resolve_name(\"pytest:approx\")\n",
            "    runpy.run_module(\"pytest\")\n",
            "    files(\"pytest\")\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_patch():\n",
            "    with mock.patch(\"pytest.approx\", print):\n",
            "        got = 1\n",
            "    assert got == 1\n",
            "\n",
            "\n",
            "def test_no_import():\n",
            "    importlib.util.find_spec(\"pytest\")\n",
            "    importlib.import_module(\"unittest\")\n",
            "    name = \"pytest\"\n",
            "    importlib.import_module(name)\n",
            "    assert 1 == 1\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_no_import"],
        "find_spec of pytest alone imports nothing, and a name that is no literal is not read"
    );
    assert_eq!(
        refused,
        [
            format!("test_m.py::test_statement {}", imported(11, "pytest")),
            format!(
                "test_m.py::test_calls {}",
                [
                    imported(16, "pytest"),
                    imported(17, "_pytest.fixtures"),
                    imported(18, "pytest"),
                    imported(19, "pytest"),
                    imported(20, "pytest.mark"),
                    imported(21, "pytest:approx"),
                    imported(22, "pytest"),
                    imported(23, "pytest"),
                ]
                .join("; ")
            ),
            format!(
                "test_m.py::test_patch {}; {}",
                patched(28, "pytest.approx"),
                imported(28, "pytest.approx")
            ),
        ],
        "each import line and each known call through the imports reaches its own test only, \
         and names the module as its literal writes it"
    );
}

#[test]
fn a_call_that_no_import_makes_known_and_a_module_that_is_not_pytest_give_no_reason() {
    let (tests, refused) = judged(
        "test_framework_lookalike.py",
        concat!(
            "import pytest_mock\n",
            "from shop import files\n",
            "\n",
            "\n",
            "def test_lookalike():\n",
            "    files(\"pytest\")\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_local():\n",
            "    def __import__(name):\n",
            "        return name\n",
            "\n",
            "    __import__(\"pytest\")\n",
            "    assert 1 == 1\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_lookalike", "test_m.py::test_local"],
        "a files() of another module, a local __import__, and pytest_mock import no pytest"
    );
    assert!(refused.is_empty(), "nothing refused: {refused:?}");
}

#[test]
fn a_suppression_of_foreign_framework_lets_the_test_run_with_each_import_in_its_warning() {
    let text = concat!(
        "# purba: suppress-module[foreign-framework] the suite still runs under pytest\n",
        "import pytest\n",
        "\n",
        "\n",
        "def test_a():\n",
        "    import _pytest\n",
        "    assert 1 == 1\n",
    );

    assert_eq!(
        suppressed("test_framework_suppressed.py", text),
        [verdict(
            "test_a",
            "foreign-framework",
            "line 2 imports pytest, and a test module imports no other test framework; \
             line 6 imports _pytest, and a test module imports no other test framework",
            "the suite still runs under pytest",
        )],
        "the module's suppression lifts each import, and the warning names each"
    );
}

#[test]
fn what_runs_when_a_def_is_made_and_a_call_outside_a_test_reach_every_test() {
    let (tests, refused) = judged(
        "test_framework_made.py",
        concat!(
            "import importlib\n",
            "\n",
            "import pytest, _pytest\n",
            "import pytest; import pytest\n",
            "\n",
            "\n",
            "def helper():\n",
            "    importlib.import_module(\"pytest\")\n",
            "\n",
            "\n",
            "def made(module=__import__(\"_pytest\")):\n",
            "    return module\n",
            "\n",
            "\n",
            "@importlib.import_module(\"pytest.mark\").skip\n",
            "def test_a():\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_b():\n",
            "    assert 2 == 2\n",
        ),
    );
    let reasons = [
        imported(3, "pytest"),
        imported(3, "_pytest"),
        imported(4, "pytest"),
        imported(8, "pytest"),
        imported(11, "_pytest"),
        imported(15, "pytest.mark"),
    ]
    .join("; ");

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [
            format!(
                "test_m.py::test_a [foreign-decorator] the decorator \
                 importlib.import_module(\"pytest.mark\").skip is not purba's, \
                 and purba cannot see what it does; {reasons}"
            ),
            format!("test_m.py::test_b {reasons}"),
        ],
        "a call in a helper, and a call in a decorator or a default of a def, reach every test; \
         each module of a line gives one reason"
    );
}

#[test]
fn a_def_of_the_module_and_a_patch_of_pytest_alone_import_no_pytest() {
    let (tests, refused) = judged(
        "test_framework_no_import.py",
        concat!(
            "from unittest import mock\n",
            "\n",
            "\n",
            "def __import__(name):\n",
            "    return name\n",
            "\n",
            "\n",
            "def test_def():\n",
            "    __import__(\"pytest\")\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_patch():\n",
            "    with mock.patch(\"pytest\", print):\n",
            "        got = 1\n",
            "    assert got == 1\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_def"],
        "a name that the module binds with a def is no builtin"
    );
    assert_eq!(
        refused,
        [format!("test_m.py::test_patch {}", patched(14, "pytest"))],
        "a patch imports only the parent of a dotted target"
    );
}

/// The words of a fault of an `except` that only passes, of the type `kind`.
fn alone(line: usize, kind: &str) -> String {
    format!(
        "[type-alone] line {line} expects {kind} by its type alone; \
         bind except {kind} as e: and check str(e)"
    )
}

#[test]
fn each_except_that_only_passes_refuses_its_test_as_type_alone() {
    let (tests, refused) = judged(
        "test_type_alone.py",
        concat!(
            "class Closed(Exception):\n",
            "    pass\n",
            "\n",
            "\n",
            "def lend():\n",
            "    raise Closed('closed')\n",
            "\n",
            "\n",
            "def test_pass():\n",
            "    try:\n",
            "        lend()\n",
            "    except Closed:\n",
            "        pass\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_noops():\n",
            "    try:\n",
            "        lend()\n",
            "    except Closed:\n",
            "        ...\n",
            "    try:\n",
            "        lend()\n",
            "    except Closed:\n",
            "        'expected'\n",
            "        # a comment\n",
            "        pass\n",
            "    assert 1 == 1\n",
            "\n",
            "\n",
            "def test_does_more():\n",
            "    seen = []\n",
            "    try:\n",
            "        lend()\n",
            "    except Closed:\n",
            "        seen.append(1)\n",
            "    def inner():\n",
            "        try:\n",
            "            lend()\n",
            "        except Closed:\n",
            "            pass\n",
            "    assert seen == [1]\n",
        ),
    );

    assert_eq!(
        tests,
        ["test_m.py::test_does_more"],
        "an except that does more than pass, and a try in a nested def, give no reason"
    );
    assert_eq!(
        refused,
        [
            format!("test_m.py::test_pass {}", alone(12, "Closed")),
            format!(
                "test_m.py::test_noops {}; {}",
                alone(20, "Closed"),
                alone(24, "Closed")
            ),
        ],
        "each except whose body holds only pass, ... or a string gives a reason at its own \
         line, in line order, and a comment is no statement"
    );
}

#[test]
fn an_except_that_only_passes_gives_a_reason_in_a_try_of_each_shape() {
    let (tests, refused) = judged(
        "test_type_alone_shapes.py",
        concat!(
            "class Closed(Exception):\n",
            "    pass\n",
            "\n",
            "\n",
            "def lend():\n",
            "    raise Closed('closed')\n",
            "\n",
            "\n",
            "def test_shapes():\n",
            "    try:\n",
            "        lend()\n",
            "    except (Closed, KeyError):\n",
            "        pass\n",
            "    try:\n",
            "        lend()\n",
            "    except:\n",
            "        pass\n",
            "    try:\n",
            "        lend()\n",
            "    except Closed as e:\n",
            "        pass\n",
            "    try:\n",
            "        x = 1\n",
            "        lend()\n",
            "    except KeyError:\n",
            "        pass\n",
            "    except shop.Closed:\n",
            "        pass\n",
            "    else:\n",
            "        pass\n",
            "    finally:\n",
            "        pass\n",
            "    if x:\n",
            "        try:\n",
            "            lend()\n",
            "        except* Closed:\n",
            "            pass\n",
            "    assert 1 == 1\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [format!(
            "test_m.py::test_shapes {}",
            [
                alone(12, "(Closed, KeyError)"),
                "[type-alone] line 16 expects any raise; \
                 bind except SomeError as e: and check str(e)"
                    .to_owned(),
                alone(20, "Closed"),
                alone(25, "KeyError"),
                alone(27, "shop.Closed"),
                "[type-alone] line 36 expects Closed by its type alone; \
                 bind except* Closed as e: and check str(e)"
                    .to_owned(),
            ]
            .join("; ")
        )],
        "a tuple, a bare except, an as, a try of two statements, a second except, an \
         except* and a try in a block each give a reason, but an else and a finally do not"
    );
}

#[test]
fn a_suppression_of_type_alone_lets_the_test_run_with_the_except_in_its_warning() {
    let text = concat!(
        "# purba: suppress[type-alone] the type is the whole contract\n",
        "def test_a():\n",
        "    try:\n",
        "        int('x')\n",
        "    except ValueError:\n",
        "        pass\n",
        "    assert 1 == 1\n",
    );

    assert_eq!(
        suppressed("test_type_alone_suppressed.py", text),
        [verdict(
            "test_a",
            "type-alone",
            "line 5 expects ValueError by its type alone; \
             bind except ValueError as e: and check str(e)",
            "the type is the whole contract",
        )],
        "the suppression lifts the reason, and the warning names the except"
    );
}

#[test]
fn a_tuple_with_no_parentheses_gets_them_in_the_fix_that_its_line_names() {
    let (tests, refused) = judged(
        "test_type_alone_bare_tuple.py",
        concat!(
            "def test_a():\n",
            "    try:\n",
            "        int('x')\n",
            "    except ValueError, KeyError:\n",
            "        pass\n",
            "    assert 1 == 1\n",
        ),
    );

    assert!(tests.is_empty(), "no test: {tests:?}");
    assert_eq!(
        refused,
        [format!(
            "test_m.py::test_a {}",
            alone(4, "(ValueError, KeyError)")
        )],
        "Python 3.14 refuses an as after a tuple with no parentheses, so the fix adds them"
    );
}
