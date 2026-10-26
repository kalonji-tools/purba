use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::thread;

use ruff_python_ast::Stmt;

fn module_at(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

fn write(name: &str, text: &str) -> PathBuf {
    let path = module_at(name);
    fs::write(&path, text).expect("the test writes its module");
    path
}

#[test]
fn a_module_gives_its_text_and_its_tree() {
    let text = "def test_one():\n    assert 1 == 1\n";
    let module = purba_parsing::parse(&write("test_tree.py", text)).expect("the module parses");

    assert_eq!(module.source(), text, "the text is the file's text");
    let functions: Vec<&str> = module
        .syntax()
        .body
        .iter()
        .filter_map(|statement| match statement {
            Stmt::FunctionDef(function) => Some(function.name.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(functions, ["test_one"], "the tree holds the one function");
}

#[test]
fn a_missing_file_is_not_found() {
    let error = purba_parsing::parse(&module_at("test_missing.py"))
        .err()
        .expect("a missing file is an error");

    assert_eq!(error.kind(), ErrorKind::NotFound, "the error is the read's");
}

#[test]
fn a_syntax_error_is_not_an_error_of_parse() {
    let parsed = purba_parsing::parse(&write("test_broken.py", "def test_one(:\n"));

    assert!(parsed.is_ok(), "a syntax error gives a module");
}

#[test]
fn a_byte_order_mark_is_not_part_of_the_text() {
    let text = "def test_one():\n    assert 1 == 1\n";
    let module = purba_parsing::parse(&write("test_bom.py", &format!("\u{feff}{text}")))
        .expect("the module parses");

    assert_eq!(module.source(), text, "the text starts after the mark");
}

#[test]
fn a_module_that_nests_deeper_than_its_stack_frees_its_tree() {
    const SMALL_STACK: usize = 256 * 1024;
    for (name, text) in [
        (
            "test_deep_minus.py",
            format!("x = {}1\n", "-".repeat(20_000)),
        ),
        (
            "test_deep_pattern.py",
            format!(
                "match 1:\n    case {}1{}:\n        pass\n",
                "[".repeat(20_000),
                "]".repeat(20_000)
            ),
        ),
        (
            "test_deep_format_spec.py",
            format!("x = f\"{}x{}\"\n", "{x:".repeat(3_000), "}".repeat(3_000)),
        ),
    ] {
        let path = write(name, &text);
        let freed = thread::Builder::new()
            .name(name.to_owned())
            .stack_size(SMALL_STACK)
            .spawn(move || {
                let module = purba_parsing::parse(&path).expect("the module parses");
                drop(module);
            })
            .expect("the thread starts")
            .join();
        assert!(freed.is_ok(), "{name}: the tree is freed");
    }
}
