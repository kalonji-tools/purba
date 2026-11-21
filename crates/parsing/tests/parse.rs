use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::thread;

use purba_parsing::{Error, SyntaxError};
use pyo3::Python;
use ruff_python_ast::Stmt;
use ruff_text_size::Ranged;

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

/// The text of the module that `bytes` holds, as parsing decodes it.
fn decoded(name: &str, bytes: &[u8]) -> String {
    match purba_parsing::parse(&write_bytes(name, bytes)) {
        Ok(module) => module.source().to_owned(),
        Err(error) => panic!("{name}: the module decodes and parses, not {error:?}"),
    }
}

/// The line of the error that refuses to decode the module that `bytes` holds.
fn undecodable(name: &str, bytes: &[u8]) -> String {
    match purba_parsing::parse(&write_bytes(name, bytes)) {
        Err(Error::Decode(error)) => error.to_string(),
        Err(error) => panic!("{name}: the decode refuses the module, not {error:?}"),
        Ok(_) => panic!("{name}: a module that its encoding cannot decode is refused"),
    }
}

fn syntax_error(name: &str, text: &str) -> SyntaxError {
    match purba_parsing::parse(&write(name, text)) {
        Err(Error::Syntax(error)) => error,
        Err(error) => panic!("{name}: ruff refuses the module, not {error:?}"),
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
        Err(error) => panic!("a missing file gives the read's error, not {error:?}"),
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
fn a_declaration_other_than_utf_8_decodes_the_module_with_its_codec() {
    Python::initialize();
    for (name, bytes, text) in [
        (
            "test_latin_first.py",
            &b"# -*- coding: latin-1 -*-\nx = '\xc3\xa9'\n"[..],
            "# -*- coding: latin-1 -*-\nx = '\u{c3}\u{a9}'\n",
        ),
        (
            "test_latin_after_comment.py",
            b"#!/usr/bin/env python\n# coding: latin-1\nx = '\xe9'\n",
            "#!/usr/bin/env python\n# coding: latin-1\nx = 'é'\n",
        ),
        (
            "test_latin_after_blank.py",
            b"\n# coding: latin-1\nx = '\xe9'\n",
            "\n# coding: latin-1\nx = 'é'\n",
        ),
        (
            "test_latin_vim.py",
            b"# vim: set fileencoding=latin-1 :\nx = '\xe9'\n",
            "# vim: set fileencoding=latin-1 :\nx = 'é'\n",
        ),
        (
            "test_ascii.py",
            b"# coding: ascii\nx = 'e'\n",
            "# coding: ascii\nx = 'e'\n",
        ),
        (
            "test_iso_8859_1.py",
            b"# coding: iso-8859-1\nx = '\x80'\n",
            "# coding: iso-8859-1\nx = '\u{80}'\n",
        ),
        (
            "test_cp1252.py",
            b"# coding: cp1252\nx = '\x80'\n",
            "# coding: cp1252\nx = '€'\n",
        ),
        (
            "test_euc_jp.py",
            b"# coding: euc_jp\nx = '\xc6\xfc\xcb\xdc'\n",
            "# coding: euc_jp\nx = '日本'\n",
        ),
        (
            "test_cr_blank.py",
            b"\r# coding: latin-1\rx = '\xe9'\r",
            "\r# coding: latin-1\rx = 'é'\r",
        ),
        (
            "test_cr_comment.py",
            b"#!\r# coding: latin-1\rx = '\xe9'\r",
            "#!\r# coding: latin-1\rx = 'é'\r",
        ),
        (
            "test_crlf_comment.py",
            b"#!\r\n# coding: latin-1\r\nx = '\xe9'\r\n",
            "#!\r\n# coding: latin-1\r\nx = 'é'\r\n",
        ),
        (
            "test_tab_before.py",
            b"\t# coding: latin-1\nx = '\xe9'\n",
            "\t# coding: latin-1\nx = 'é'\n",
        ),
        (
            "test_form_feed_before.py",
            b"\x0c# coding: latin-1\nx = '\xe9'\n",
            "\x0c# coding: latin-1\nx = 'é'\n",
        ),
        (
            "test_space_tab_form_feed_before.py",
            b" \t\x0c# coding: latin-1\nx = '\xe9'\n",
            " \t\x0c# coding: latin-1\nx = 'é'\n",
        ),
        (
            "test_declaration_only.py",
            b"# coding: latin-1",
            "# coding: latin-1",
        ),
    ] {
        assert_eq!(
            decoded(name, bytes),
            text,
            "{name}: the codec of the declared encoding gives the text"
        );
    }
}

#[test]
fn a_syntax_error_after_a_decode_counts_the_columns_of_the_decoded_text() {
    Python::initialize();
    let path = write_bytes("test_latin_broken.py", b"# coding: latin-1\n\xe9 = = 1\n");
    let error = match purba_parsing::parse(&path) {
        Err(Error::Syntax(error)) => error,
        Err(error) => panic!("ruff refuses the decoded module, not {error:?}"),
        Ok(_) => panic!("a decoded module with a syntax error gives an error"),
    };

    assert_eq!(
        (error.start().line(), error.start().column()),
        (2, 5),
        "the second = is the fifth scalar of line 2, as CPython counts it"
    );
}

#[test]
fn a_module_that_its_encoding_cannot_decode_is_refused_with_cpythons_words() {
    Python::initialize();
    for (name, bytes, line) in [
        (
            "test_latin_bom.py",
            &b"\xef\xbb\xbf# coding: latin-1\nx = 'e'\n"[..],
            "SyntaxError: encoding problem: latin-1 with BOM",
        ),
        (
            "test_latin_bom_line_two.py",
            b"\xef\xbb\xbf#!\n# coding: latin-1\nx = 'e'\n",
            "SyntaxError: encoding problem: latin-1 with BOM",
        ),
        (
            "test_bom_utf8.py",
            b"\xef\xbb\xbf# coding: utf8\nx = 'e'\n",
            "SyntaxError: encoding problem: utf8 with BOM",
        ),
        (
            "test_bom_u8.py",
            b"\xef\xbb\xbf# coding: u8\nx = 'e'\n",
            "SyntaxError: encoding problem: u8 with BOM",
        ),
        (
            "test_bom_cp65001.py",
            b"\xef\xbb\xbf# coding: cp65001\nx = 'e'\n",
            "SyntaxError: encoding problem: cp65001 with BOM",
        ),
        (
            "test_utf_dot_8.py",
            b"# coding: utf.8\nx = 'e'\n",
            "SyntaxError: unknown encoding: utf.8",
        ),
        (
            "test_utf_dot_8_upper.py",
            b"# coding: UTF.8\nx = 'e'\n",
            "SyntaxError: unknown encoding: UTF.8",
        ),
        (
            "test_u_dot_8.py",
            b"# coding: u.8\nx = 'e'\n",
            "SyntaxError: unknown encoding: u.8",
        ),
        (
            "test_utf8_sig.py",
            b"# coding: utf8-sig\nx = 'e'\n",
            "SyntaxError: unknown encoding: utf8-sig",
        ),
        (
            "test_cp_dot_65001.py",
            b"# coding: cp.65001\nx = 'e'\n",
            "SyntaxError: unknown encoding: cp.65001",
        ),
        (
            "test_unknown.py",
            b"# coding: foobar\nx = 'e'\n",
            "SyntaxError: unknown encoding: foobar",
        ),
        (
            "test_rot13.py",
            b"# coding: rot13\nx = 'e'\n",
            concat!(
                "SyntaxError: 'rot13' is not a text encoding; ",
                "use codecs.decode() to handle arbitrary codecs"
            ),
        ),
        (
            "test_cp1252_0x81.py",
            b"# coding: cp1252\nx = '\x81'\n",
            concat!(
                "SyntaxError: 'charmap' codec can't decode byte 0x81 in position 22: ",
                "character maps to <undefined>"
            ),
        ),
        (
            "test_ascii_0xe9.py",
            b"# coding: ascii\nx = '\xe9'\n",
            concat!(
                "SyntaxError: 'ascii' codec can't decode byte 0xe9 in position 21: ",
                "ordinal not in range(128)"
            ),
        ),
        (
            "test_no_declaration_0xff.py",
            b"x = '\xff'\n",
            "SyntaxError: stream did not contain valid UTF-8",
        ),
        (
            "test_utf_8_0xff.py",
            b"# coding: utf-8\nx = '\xff'\n",
            "SyntaxError: stream did not contain valid UTF-8",
        ),
    ] {
        assert_eq!(
            undecodable(name, bytes),
            line,
            "{name}: the error gives the words of CPython's codec, or of purba's UTF-8"
        );
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
fn a_module_in_the_syntax_of_a_newer_or_older_python_parses() {
    for (name, text) in [
        ("test_type_param_default.py", "def f[T = int](): pass\n"),
        (
            "test_unparenthesized_except.py",
            "try:\n    pass\nexcept ValueError, TypeError:\n    pass\n",
        ),
        ("test_template_string.py", "x = t'{1}'\n"),
        ("test_lazy_import.py", "lazy import os\n"),
        (
            "test_unpacking_in_comprehension.py",
            "x = [*a for a in b]\n",
        ),
        ("test_unpacking_in_set.py", "x = {*a for a in b}\n"),
        ("test_unpacking_in_generator.py", "x = (*a for a in b)\n"),
        ("test_unpacking_in_dict.py", "x = {**a for a in b}\n"),
        (
            "test_unary_plus_pattern.py",
            "match x:\n    case +1:\n        pass\n",
        ),
        ("test_parenthesized_keyword.py", "f((a)=1)\n"),
    ] {
        if let Err(error) = purba_parsing::parse(&write(name, text)) {
            panic!("{name}: the CPython that loads the module refuses it, not parsing: {error:?}");
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

/// The text of each assert at the end of the module's first `def`, as `ending`
/// gives them.
fn ending_of(name: &str, text: &str) -> Vec<String> {
    let module = purba_parsing::parse(&write(name, text)).expect("the module parses");
    let Some(Stmt::FunctionDef(function)) = module.syntax().body.first() else {
        panic!("{name}: the module starts with a def");
    };
    purba_parsing::ending(&function.body)
        .into_iter()
        .map(|assert| module.source()[assert.range()].to_owned())
        .collect()
}

#[test]
fn the_ending_is_the_asserts_at_the_end_of_a_body() {
    let text = concat!(
        "def test_a():\n",
        "    x = 1\n",
        "    assert x == 2\n",
        "    y = 3\n",
        "    assert x == 1\n",
        "    assert y == 3\n",
    );

    assert_eq!(
        ending_of("test_end.py", text),
        ["assert x == 1", "assert y == 3"],
        "the asserts after the last other statement, in source order"
    );
}

#[test]
fn the_asserts_at_the_end_of_a_with_before_them_come_first() {
    let nested = concat!(
        "def test_a():\n",
        "    with open('a') as a:\n",
        "        with open('b') as b:\n",
        "            assert a == 1\n",
        "        assert b == 2\n",
        "    assert a == 3\n",
    );
    let awaited =
        "async def test_b():\n    async with c as d:\n        assert d == 1\n    assert d == 2\n";

    assert_eq!(
        ending_of("test_with.py", nested),
        ["assert a == 1", "assert b == 2", "assert a == 3"],
        "the walk enters each with just before the asserts, at two depths"
    );
    assert_eq!(
        ending_of("test_async_with.py", awaited),
        ["assert d == 2"],
        "an async with ends the walk"
    );
}

#[test]
fn a_with_that_a_statement_keeps_from_the_asserts_is_not_entered() {
    let apart = concat!(
        "def test_c():\n",
        "    with open('c') as c:\n",
        "        assert c == 1\n",
        "    x = 2\n",
        "    assert x == 2\n",
    );

    assert_eq!(
        ending_of("test_with_apart.py", apart),
        ["assert x == 2"],
        "the walk enters only a with just before the asserts"
    );
}

#[test]
fn a_tuple_assert_at_the_end_counts_like_any_other_assert() {
    let text = "def test_a():\n    x = 1\n    assert x == 1\n    assert (x, 1)\n";

    assert_eq!(
        ending_of("test_tuple_end.py", text),
        ["assert x == 1", "assert (x, 1)"],
        "the walk takes a tuple assert, and goes on above it"
    );
}

#[test]
fn the_blocks_of_a_statement_are_the_bodies_it_holds_in_source_order() {
    let text = concat!(
        "if a:\n    x = 1\nelif b:\n    x = 2\nelse:\n    x = 3\n",
        "for i in c:\n    x = 4\nelse:\n    x = 5\n",
        "while d:\n    x = 6\nelse:\n    x = 7\n",
        "with e:\n    x = 8\n",
        "try:\n    x = 9\nexcept E:\n    x = 10\nexcept F:\n    x = 11\n",
        "else:\n    x = 12\nfinally:\n    x = 13\n",
        "match f:\n    case 1:\n        x = 14\n    case _:\n        x = 15\n",
        "def g():\n    x = 16\n",
        "class H:\n    x = 17\n",
        "x = 18\n",
    );
    let module = purba_parsing::parse(&write("test_blocks.py", text)).expect("the module parses");

    let firsts: Vec<Vec<&str>> = module
        .syntax()
        .body
        .iter()
        .map(|statement| {
            purba_parsing::blocks(statement)
                .into_iter()
                .map(|block| &module.source()[block[0].range()])
                .collect()
        })
        .collect();

    assert_eq!(
        firsts,
        [
            vec!["x = 1", "x = 2", "x = 3"],
            vec!["x = 4", "x = 5"],
            vec!["x = 6", "x = 7"],
            vec!["x = 8"],
            vec!["x = 9", "x = 10", "x = 11", "x = 12", "x = 13"],
            vec!["x = 14", "x = 15"],
            vec![],
            vec![],
            vec![],
        ],
        "each block of if, for, while, with, try with two handlers, and match, in source order; \
         a def, a class and a simple statement hold none"
    );
}

/// What `shape` names in the test of each assert of `text`, as text: the kind
/// of shape, and the text of each part that it hands back.
fn shapes_of(name: &str, text: &str) -> Vec<String> {
    use purba_parsing::{Check, Shape};
    let module = purba_parsing::parse(&write(name, text)).expect("the module parses");
    let part = |expr: &dyn Ranged| module.source()[expr.range()].to_owned();
    let check = |check: Check<'_>| match check {
        Check::Compare { op, left, right } => {
            format!("compare {} {} {}", part(left), op.as_str(), part(right))
        }
        Check::Read {
            read,
            reading,
            other,
        } => format!(
            "read {read:?} {} {}",
            part(reading),
            other.map_or_else(|| "-".to_owned(), |other| part(other))
        ),
    };
    module
        .syntax()
        .body
        .iter()
        .map(|statement| {
            let Stmt::Assert(assert) = statement else {
                panic!("{name}: each statement is an assert");
            };
            match purba_parsing::shape(&assert.test) {
                Shape::Check(one) => check(one),
                Shape::Each {
                    any,
                    check: one,
                    item,
                    target,
                    reading,
                } => format!(
                    "each any={any} [{}] item {} for {} in {}",
                    check(one),
                    part(item),
                    part(target),
                    part(reading)
                ),
                Shape::OtherEach { function, shape } => format!("other {function} {shape}"),
                Shape::Other => "no check".to_owned(),
            }
        })
        .collect()
}

#[test]
fn the_shape_of_a_test_names_its_check_or_why_it_has_none() {
    let text = concat!(
        "assert x == 1\n",
        "assert not x\n",
        "assert isinstance(x, int)\n",
        "assert x.startswith('a')\n",
        "assert x.endswith('z')\n",
        "assert all(v > 0 for v in xs)\n",
        "assert any(v.startswith('a') for v in xs)\n",
        "assert all(v for v in xs if v)\n",
        "assert any([v == 1 for v in xs])\n",
        "assert all(f(v) for v in xs)\n",
        "assert 0 < x < 2\n",
        "assert isinstance(x, int, y)\n",
        "assert x\n",
        "assert all(v > 0 async for v in xs)\n",
        "assert isinstance(*xs)\n",
        "assert isinstance(*xs, int)\n",
    );

    assert_eq!(
        shapes_of("test_shapes.py", text),
        [
            "compare x == 1",
            "read Not x -",
            "read Instance x int",
            "read Start x 'a'",
            "read End x 'z'",
            "each any=false [compare v > 0] item v > 0 for v in xs",
            "each any=true [read Start v 'a'] item v.startswith('a') for v in xs",
            "other all with an if",
            "other any of a list, not a generator",
            "no check",
            "no check",
            "no check",
            "no check",
            "no check",
            "no check",
            "no check",
        ],
        "a comparison by one operator and each of the four reads are one check; an all() or an \
         any() of one check hands it back; another all() or any() names its shape; an all() of \
         an async generator, a read with a starred argument and any other test is no check"
    );
}
