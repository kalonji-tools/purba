use purba_parsing::{Facts, Module, Test};
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
