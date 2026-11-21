//! This test binary never starts Python, so a `parse` that calls Python for a
//! module that it reads as UTF-8 panics here.

use std::fs;
use std::path::PathBuf;

#[test]
fn a_module_read_as_utf_8_parses_with_no_interpreter() {
    for (name, text) in [
        ("test_no_declaration.py", "x = 'é'\n"),
        ("test_declares_utf_8.py", "# coding: utf-8\nx = 'é'\n"),
        ("test_declares_utf8.py", "# coding: utf8\nx = 'é'\n"),
    ] {
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
        fs::write(&path, text).expect("the test writes its module");

        match purba_parsing::parse(&path) {
            Ok(module) => assert_eq!(module.source(), text, "{name}: the text is the file's"),
            Err(error) => panic!("{name}: the module parses, not {error:?}"),
        }
    }
}
