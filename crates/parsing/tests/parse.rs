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

fn write_bytes(name: &str, bytes: &[u8]) -> PathBuf {
    let path = module_at(name);
    fs::write(&path, bytes).expect("the test writes its module");
    path
}

fn assert_refused(name: &str, bytes: &[u8], encoding: &str) {
    let error = match purba_parsing::parse(&write_bytes(name, bytes)) {
        Err(Error::Read(error)) => error,
        Err(Error::Syntax(error)) => {
            panic!("{name}: the read refuses the module, not ruff: {error}")
        }
        Ok(_) => panic!("{name}: a module that declares an encoding other than UTF-8 is refused"),
    };

    assert_eq!(
        error.kind(),
        ErrorKind::InvalidData,
        "{name}: the read refuses the data"
    );
    assert_eq!(
        error.to_string(),
        format!("it declares the encoding {encoding}, and purba reads only UTF-8"),
        "{name}: the error names the declared encoding"
    );
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
fn a_declaration_other_than_utf_8_refuses_the_module() {
    for (name, bytes, encoding) in [
        (
            "test_latin_first.py",
            &b"# -*- coding: latin-1 -*-\nx = '\xc3\xa9'\n"[..],
            "latin-1",
        ),
        (
            "test_latin_after_comment.py",
            b"#!/usr/bin/env python\n# coding: latin-1\nx = '\xc3\xa9'\n",
            "latin-1",
        ),
        (
            "test_latin_after_blank.py",
            b"\n# coding: latin-1\nx = '\xc3\xa9'\n",
            "latin-1",
        ),
        (
            "test_latin_vim.py",
            b"# vim: set fileencoding=latin-1 :\nx = '\xc3\xa9'\n",
            "latin-1",
        ),
        (
            "test_latin_bom.py",
            b"\xef\xbb\xbf# coding: latin-1\nx = '\xc3\xa9'\n",
            "latin-1",
        ),
        ("test_ascii.py", b"# coding: ascii\nx = 'e'\n", "ascii"),
        ("test_utf_dot_8.py", b"# coding: utf.8\nx = 'e'\n", "utf.8"),
        (
            "test_utf_dot_8_upper.py",
            b"# coding: UTF.8\nx = 'e'\n",
            "UTF.8",
        ),
        ("test_u_dot_8.py", b"# coding: u.8\nx = 'e'\n", "u.8"),
        (
            "test_utf8_sig.py",
            b"# coding: utf8-sig\nx = 'e'\n",
            "utf8-sig",
        ),
        (
            "test_cp_dot_65001.py",
            b"# coding: cp.65001\nx = 'e'\n",
            "cp.65001",
        ),
        (
            "test_bom_utf8.py",
            b"\xef\xbb\xbf# coding: utf8\nx = 'e'\n",
            "utf8",
        ),
        (
            "test_bom_u8.py",
            b"\xef\xbb\xbf# coding: u8\nx = 'e'\n",
            "u8",
        ),
        (
            "test_bom_cp65001.py",
            b"\xef\xbb\xbf# coding: cp65001\nx = 'e'\n",
            "cp65001",
        ),
        (
            "test_cr_blank.py",
            b"\r# coding: latin-1\rx = 'e'\r",
            "latin-1",
        ),
        (
            "test_cr_comment.py",
            b"#!\r# coding: latin-1\rx = 'e'\r",
            "latin-1",
        ),
        (
            "test_crlf_comment.py",
            b"#!\r\n# coding: latin-1\r\nx = 'e'\r\n",
            "latin-1",
        ),
        (
            "test_tab_before.py",
            b"\t# coding: latin-1\nx = 'e'\n",
            "latin-1",
        ),
        (
            "test_form_feed_before.py",
            b"\x0c# coding: latin-1\nx = 'e'\n",
            "latin-1",
        ),
        (
            "test_space_tab_form_feed_before.py",
            b" \t\x0c# coding: latin-1\nx = 'e'\n",
            "latin-1",
        ),
    ] {
        assert_refused(name, bytes, encoding);
    }
}

#[test]
fn a_utf_8_declaration_or_a_comment_that_declares_nothing_keeps_the_module() {
    for (name, text) in [
        ("test_utf_8.py", "# coding: utf-8\nx = 'é'\n"),
        ("test_utf8_upper.py", "# coding: UTF8\nx = 'é'\n"),
        ("test_utf_8_underscore.py", "# coding=utf_8\nx = 'é'\n"),
        (
            "test_utf_8_unix.py",
            "# -*- coding: utf-8-unix -*-\nx = 'é'\n",
        ),
        ("test_u8.py", "# coding: u8\nx = 'é'\n"),
        ("test_cp65001.py", "# coding: cp65001\nx = 'é'\n"),
        (
            "test_utf_8_sig_doubled.py",
            "# coding: utf__8__sig\nx = 'é'\n",
        ),
        ("test_utf_8_sig.py", "# coding: utf_8_sig\nx = 'é'\n"),
        ("test_utf8_dot_ucs2.py", "# coding: utf8.ucs2\nx = 'é'\n"),
        ("test_utf_8_doubled.py", "# coding: utf__8\nx = 'é'\n"),
        ("test_bom_utf_8.py", "\u{feff}# coding: utf-8\nx = 'é'\n"),
        (
            "test_bom_utf_8_sig.py",
            "\u{feff}# coding: utf_8_sig\nx = 'é'\n",
        ),
        (
            "test_cr_third_line.py",
            "#!\r\r# coding: latin-1\rx = 'é'\r",
        ),
        (
            "test_cr_code_first.py",
            "#a\rx = 1\n# coding: latin-1\nx = 'é'\n",
        ),
        ("test_form_feed_after.py", "# coding:\x0clatin-1\nx = 'é'\n"),
        (
            "test_after_code.py",
            "import os\n# coding: latin-1\nx = 'é'\n",
        ),
        ("test_third_line.py", "#a\n#b\n# coding: latin-1\nx = 'é'\n"),
        ("test_beside_code.py", "y = 1  # coding: latin-1\nx = 'é'\n"),
    ] {
        match purba_parsing::parse(&write(name, text)) {
            Ok(module) => assert_eq!(
                module.source(),
                text.trim_start_matches('\u{feff}'),
                "{name}: the text is the file's, after a byte order mark"
            ),
            Err(error) => panic!("{name}: the module parses, not {error:?}"),
        }
    }
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

#[test]
fn an_assert_with_no_value_or_a_starred_value_gives_ruffs_message() {
    for (name, text, message) in [
        (
            "test_bare.py",
            "def test_a():\n    assert\n",
            "Expected an expression",
        ),
        (
            "test_star.py",
            "def test_a():\n    x = [1]\n    assert *x\n",
            "Starred expression cannot be used here",
        ),
        (
            "test_dstar.py",
            "def test_a():\n    x = {\"value\": 1}\n    assert **x\n",
            "Expected an expression",
        ),
        ("test_eof.py", "x = (\n", "unexpected EOF while parsing"),
    ] {
        assert_eq!(
            syntax_error(name, text).message(),
            message,
            "{name}: ruff's words for its first error"
        );
    }
}
