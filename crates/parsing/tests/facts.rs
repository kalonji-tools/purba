use purba_parsing::{Check, Facts, Module, Shape, Test};
use ruff_python_ast::Stmt;
use ruff_text_size::{Ranged, TextSize};

fn parsed(text: &str) -> Module {
    purba_parsing::parse_source(text.to_owned()).expect("the module parses")
}

fn line(module: &Module, offset: TextSize) -> usize {
    module.position(offset).line()
}

/// A test as `Class.name under=keyword defs=lines first=line`.
fn shown(module: &Module, test: &Test<'_>) -> String {
    let last = test.defs.last().expect("a test has a def");
    let mut path: Vec<&str> = test.classes.clone();
    path.push(last.name.as_str());
    let lines: Vec<String> = test
        .defs
        .iter()
        .map(|function| line(module, function.start()).to_string())
        .collect();
    format!(
        "{} under={} defs={} first={}",
        path.join("."),
        test.under.unwrap_or("-"),
        lines.join(","),
        line(module, test.first)
    )
}

#[test]
fn the_tests_of_a_module_are_its_test_names_at_the_top_under_a_statement_or_in_a_class() {
    let module = parsed(concat!(
        "def test_top():\n",
        "    pass\n",
        "\n",
        "if flag:\n",
        "    def test_under():\n",
        "        pass\n",
        "\n",
        "class TestBox:\n",
        "    def test_in(self):\n",
        "        pass\n",
        "\n",
        "    def helper(self):\n",
        "        pass\n",
        "\n",
        "for _ in []:\n",
        "    class TestDeep:\n",
        "        class Inner:\n",
        "            def test_inner(self):\n",
        "                pass\n",
        "\n",
        "def helper():\n",
        "    class TestHidden:\n",
        "        def test_hidden(self):\n",
        "            pass\n",
        "\n",
        "    def test_nested():\n",
        "        pass\n",
        "\n",
        "def test_twice():\n",
        "    pass\n",
        "\n",
        "def test_twice():\n",
        "    pass\n",
        "\n",
        "while flag:\n",
        "    def test_early():\n",
        "        pass\n",
        "\n",
        "def test_early():\n",
        "    pass\n",
        "\n",
        "def test_top():\n",
        "    pass\n",
    ));
    let facts = Facts::of(&module);

    let tests: Vec<String> = facts
        .tests()
        .iter()
        .map(|test| shown(&module, test))
        .collect();

    assert_eq!(
        tests,
        [
            "test_top under=- defs=1,42 first=1",
            "test_under under=if defs=5 first=5",
            "TestBox.test_in under=- defs=9 first=9",
            "TestDeep.Inner.test_inner under=- defs=18 first=18",
            "test_twice under=- defs=29,32 first=29",
            "test_early under=- defs=39 first=36",
        ],
        "each test name at the top, under a statement or in a class there, by its first def: \
         a def in a function is no test, and a def under a statement gives way to a def at the \
         top of its name, which keeps the first line"
    );
}

#[test]
fn the_imports_of_a_module_are_its_imports_at_the_top_and_in_its_blocks() {
    let module = parsed(concat!(
        "import os\n",
        "import os.path\n",
        "import xml.dom as dom\n",
        "from unittest import mock\n",
        "from unittest.mock import patch as swap\n",
        "from . import sibling\n",
        "from .near import far as distant\n",
        "from star import *\n",
        "import alpha as twice\n",
        "import beta as twice\n",
        "if flag:\n",
        "    import lending\n",
        "def helper():\n",
        "    import hidden\n",
        "class Box:\n",
        "    import boxed\n",
    ));
    let facts = Facts::of(&module);

    let mut imports: Vec<String> = facts
        .imports()
        .iter()
        .map(|(name, origin)| format!("{name}={origin}"))
        .collect();
    imports.sort();

    assert_eq!(
        imports,
        [
            "distant=.near.far",
            "dom=xml.dom",
            "lending=lending",
            "mock=unittest.mock",
            "os=os",
            "sibling=.sibling",
            "swap=unittest.mock.patch",
            "twice=beta",
        ],
        "each name an import binds at the top or in a block there, with where it comes from; \
         `import os.path` binds os; the last import of a name wins; no star, and nothing in a \
         def or a class"
    );
}

#[test]
fn the_bindings_of_a_module_are_each_place_that_binds_a_name_at_the_top_or_under_global() {
    let module = parsed(concat!(
        "total = 1\n",
        "def grow():\n",
        "    global total\n",
        "    total = 2\n",
        "    local = 3\n",
        "class Box:\n",
        "    inside = 4\n",
        "for item in []:\n",
        "    pass\n",
        "with open(\"f\") as handle:\n",
        "    pass\n",
        "if (found := 5):\n",
        "    pass\n",
        "import os\n",
        "del total\n",
        "class Ledger:\n",
        "    global entry\n",
        "    entry = 6\n",
        "def helper(a=(walrus := 7)):\n",
        "    pass\n",
    ));
    let facts = Facts::of(&module);
    let read = &module;

    let mut bindings: Vec<String> = facts
        .bindings()
        .iter()
        .flat_map(|(name, places)| {
            places.iter().map(move |binding| {
                format!(
                    "{name} line={} in_function={}",
                    line(read, binding.offset),
                    binding.in_function
                )
            })
        })
        .collect();
    bindings.sort();

    assert_eq!(
        bindings,
        [
            "Box line=6 in_function=false",
            "Ledger line=16 in_function=false",
            "entry line=18 in_function=false",
            "found line=12 in_function=false",
            "grow line=2 in_function=false",
            "handle line=10 in_function=false",
            "helper line=19 in_function=false",
            "item line=8 in_function=false",
            "os line=14 in_function=false",
            "total line=1 in_function=false",
            "total line=15 in_function=false",
            "total line=4 in_function=true",
            "walrus line=19 in_function=false",
        ],
        "each binding at the top, each in what runs where a def stands, and each under global \
         in a def or a class; a local of grow \
         and the name inside, which Box binds, are not the module's"
    );
}

#[test]
fn the_statements_of_a_module_are_those_at_the_top_and_in_its_blocks_in_source_order() {
    let module = parsed(concat!(
        "a = 1\n",
        "if flag:\n",
        "    b = 2\n",
        "else:\n",
        "    try:\n",
        "        c = 3\n",
        "    finally:\n",
        "        d = 4\n",
        "def helper():\n",
        "    e = 5\n",
        "class Box:\n",
        "    f = 6\n",
        "g = 7\n",
    ));
    let facts = Facts::of(&module);

    let lines: Vec<usize> = facts
        .statements()
        .iter()
        .map(|statement| line(&module, statement.start()))
        .collect();

    assert_eq!(
        lines,
        [1, 2, 3, 5, 6, 8, 9, 11, 13],
        "each statement at the top and in each block there, in source order; \
         not the body of a def or a class"
    );
    assert!(
        matches!(facts.statements()[6], Stmt::FunctionDef(_)),
        "a def at the top is one of the statements"
    );
}

#[test]
fn the_defs_of_a_module_are_each_def_at_any_depth_in_source_order() {
    let module = parsed(concat!(
        "def one():\n",
        "    def two():\n",
        "        pass\n",
        "class Box:\n",
        "    def three(self):\n",
        "        pass\n",
        "if flag:\n",
        "    def four():\n",
        "        lambda: 0\n",
    ));
    let facts = Facts::of(&module);

    let names: Vec<&str> = facts
        .defs()
        .iter()
        .map(|function| function.name.as_str())
        .collect();

    assert_eq!(
        names,
        ["one", "two", "three", "four"],
        "each def, in a def, a class or a block, in source order"
    );
    assert!(
        std::ptr::eq(facts.module(), std::ptr::from_ref(&module)),
        "the facts lend the module they read"
    );
}

#[test]
fn a_decorated_test_starts_at_its_first_decorator() {
    let module = parsed(concat!(
        "import mark\n",
        "\n",
        "@mark.slow\n",
        "def test_marked():\n",
        "    pass\n",
    ));
    let facts = Facts::of(&module);

    let tests: Vec<String> = facts
        .tests()
        .iter()
        .map(|test| shown(&module, test))
        .collect();

    assert_eq!(
        tests,
        ["test_marked under=- defs=3 first=3"],
        "a def starts at its first decorator, as ruff's range of it does"
    );
}

#[test]
fn an_empty_module_has_no_fact() {
    let module = parsed("");
    let facts = Facts::of(&module);

    assert!(
        facts.tests().is_empty()
            && facts.imports().is_empty()
            && facts.bindings().is_empty()
            && facts.statements().is_empty()
            && facts.defs().is_empty(),
        "an empty module gives no test, import, binding, statement or def"
    );
}

#[test]
fn the_facts_of_a_test_are_read_when_a_part_first_asks_and_kept() {
    let module = parsed("def test_one(a):\n    assert a == 1\n\ndef test_two():\n    pass\n");
    let facts = Facts::of(&module);
    let [one, two] = facts.tests() else {
        panic!("the module has two tests");
    };

    assert!(
        one.facts_if_read().is_none() && two.facts_if_read().is_none(),
        "reading the facts of a module reads no fact of a test"
    );
    let first = one.facts();
    assert!(
        std::ptr::eq(first, one.facts()),
        "a second ask gives the facts that the first ask read"
    );
    assert!(
        one.facts_if_read().is_some() && two.facts_if_read().is_none(),
        "asking for the facts of one test reads no fact of another"
    );
}

/// A shape as `other`, `each`, or `compare <side>` and `read`.
fn shape_of(shape: Shape<'_>) -> String {
    match shape {
        Shape::Check(Check::Compare { expected, .. }) => format!("compare {expected:?}"),
        Shape::Check(Check::Read { .. }) => "read".to_owned(),
        Shape::Each { .. } | Shape::OtherEach { .. } => "each".to_owned(),
        Shape::Other => "other".to_owned(),
    }
}

#[test]
fn the_facts_of_a_test_are_those_of_the_def_that_binds_its_name() {
    let module = parsed(concat!(
        "def test_one(a):\n",
        "    pass\n",
        "\n",
        "def test_one(x, /, y=1, *rest, z, **more):\n",
        "    import os\n",
        "    total = x\n",
        "    def inner(q):\n",
        "        w = 1\n",
        "        try:\n",
        "            pass\n",
        "        except E:\n",
        "            pass\n",
        "    try:\n",
        "        f(total)\n",
        "    except ValueError:\n",
        "        pass\n",
        "    with ctx:\n",
        "        for item in rest:\n",
        "            assert item\n",
        "    assert (total, 1)\n",
        "    assert total == 1\n",
    ));
    let facts = Facts::of(&module);
    let [test] = facts.tests() else {
        panic!("the module has one test");
    };
    let own = test.facts();

    let mut parameters: Vec<&str> = own.parameters.iter().copied().collect();
    parameters.sort_unstable();
    assert_eq!(
        parameters,
        ["more", "rest", "x", "y", "z"],
        "each parameter of the def that binds the name, of each kind, and none of the first def"
    );
    let mut locals: Vec<&str> = own.locals.iter().copied().collect();
    locals.sort_unstable();
    assert_eq!(
        locals,
        [
            "inner", "item", "more", "os", "rest", "total", "x", "y", "z"
        ],
        "the parameters, and each name that the body binds, but no name of a nested def's body"
    );
    let ending: Vec<(usize, String)> = own
        .ending
        .iter()
        .map(|(assert, shape)| (line(&module, assert.start()), shape_of(*shape)))
        .collect();
    assert_eq!(
        ending,
        [
            (20, "other".to_owned()),
            (21, "compare Some(Right)".to_owned()),
        ],
        "the asserts at the end, a tuple assert among them, each with its shape and so its \
         expected side"
    );
    let statements: Vec<usize> = own
        .statements
        .iter()
        .map(|statement| line(&module, statement.start()))
        .collect();
    assert_eq!(
        statements,
        [5, 6, 7, 13, 14, 16, 17, 18, 19, 20, 21],
        "each statement at any depth in source order, the body of an except among them, and \
         none of a nested def's body"
    );
    let tries: Vec<usize> = own
        .tries()
        .map(|attempt| line(&module, attempt.start()))
        .collect();
    assert_eq!(
        tries,
        [13],
        "the try of the test, and not the try of its nested def"
    );
}

#[test]
fn a_class_in_a_test_binds_its_name_and_its_body_is_no_part_of_the_test() {
    let module = parsed(concat!(
        "def test_one():\n",
        "    class Inner:\n",
        "        x = 1\n",
        "        def grow(self):\n",
        "            pass\n",
        "    assert Inner.x == 1\n",
    ));
    let facts = Facts::of(&module);
    let [test] = facts.tests() else {
        panic!("the module has one test");
    };
    let own = test.facts();

    let locals: Vec<&str> = own.locals.iter().copied().collect();
    assert_eq!(
        locals,
        ["Inner"],
        "the class's name is a local, and no name of its body is"
    );
    let statements: Vec<usize> = own
        .statements
        .iter()
        .map(|statement| line(&module, statement.start()))
        .collect();
    assert_eq!(
        statements,
        [2, 6],
        "the class is a statement of the test, and no statement of its body is"
    );
}
