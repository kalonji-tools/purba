use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::thread;

use purba_parsing::{Error, SyntaxError};
use ruff_python_ast::Stmt;

const SMALL_STACK: usize = 256 * 1024;

fn module_at(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

fn write(name: &str, text: &str) -> PathBuf {
    let path = module_at(name);
    fs::write(&path, text).expect("the test writes its module");
    path
}

fn syntax_error(name: &str, text: &str) -> SyntaxError {
    match purba_parsing::parse(&write(name, text)) {
        Err(Error::Syntax(error)) => error,
        Err(Error::Read(error)) => panic!("{name}: the module is read, but the read gave {error}"),
        Ok(_) => panic!("{name}: a module with a syntax error gives an error"),
    }
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
    match purba_parsing::parse(&module_at("test_missing.py")) {
        Err(Error::Read(error)) => {
            assert_eq!(error.kind(), ErrorKind::NotFound, "the error is the read's");
        }
        Err(Error::Syntax(error)) => panic!("a missing file gives the read's error, not {error}"),
        Ok(_) => panic!("a missing file is an error"),
    }
}

#[test]
fn a_syntax_error_refuses_the_module_with_ruffs_first_error() {
    let error = syntax_error("test_broken.py", "def test_one(:\n    pass\nx = = 2\n");

    assert_eq!(
        error.message(),
        "Expected a parameter or the end of the parameter list",
        "ruff's words for the first of its errors"
    );
    assert_eq!(
        (error.start().line(), error.start().column()),
        (1, 14),
        "the error starts at the colon"
    );
    assert_eq!(
        (error.end().line(), error.end().column()),
        (1, 15),
        "the error ends after the colon"
    );
    assert_eq!(
        error.to_string(),
        "SyntaxError: Expected a parameter or the end of the parameter list",
        "the text holds no location"
    );
}

#[test]
fn a_column_counts_unicode_scalars_after_a_byte_order_mark() {
    for (name, text) in [
        ("test_scalar.py", "\u{e9} = = 1\n"),
        ("test_scalar_bom.py", "\u{feff}\u{e9} = = 1\n"),
    ] {
        let error = syntax_error(name, text);

        assert_eq!(
            (error.start().line(), error.start().column()),
            (1, 5),
            "{name}: the second = is the fifth scalar"
        );
    }
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

#[test]
fn a_module_that_nests_deeper_than_its_stack_and_does_not_parse_frees_its_tree() {
    let name = "test_deep_broken.py";
    let path = write(
        name,
        &format!("x = {}1\ndef test_one(:\n", "-".repeat(20_000)),
    );
    let refused = thread::Builder::new()
        .name(name.to_owned())
        .stack_size(SMALL_STACK)
        .spawn(move || matches!(purba_parsing::parse(&path), Err(Error::Syntax(_))))
        .expect("the thread starts")
        .join();
    assert_eq!(
        refused.ok(),
        Some(true),
        "{name}: the parse gives the syntax error, and frees the tree"
    );
}
