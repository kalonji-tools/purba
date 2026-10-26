//! PROTOTYPE (#657), never merge: a part's test hands the next part a broken
//! value, in-process, and reads the line and the exit code reporting renders.
#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]
use std::fmt;
use std::io::Write as _;

use purba_dispatch::{Module, Test};

struct T(&'static str);
impl Test for T {
    fn function(&self) -> &str {
        self.0
    }
}
impl fmt::Display for T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "test_x.py::{}", self.0)
    }
}

fn run(text: &str, asserts: &[&str]) -> (String, u8) {
    let dir = std::env::temp_dir().join(format!("proto657-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("test_x.py");
    std::fs::File::create(&path).unwrap().write_all(text.as_bytes()).unwrap();
    let module = Module::new(
        path,
        text.to_owned(),
        asserts.iter().map(|&a| a.to_owned()).collect(),
        vec![T("test_holds")],
    );
    let results = purba_dispatch::hand(vec![module]);
    let mut out = Vec::new();
    let code = purba_reporting::print(&mut out, &results).unwrap();
    (String::from_utf8(out).unwrap(), code)
}

#[test]
fn broken_values_from_assertion() {
    let cases = [
        ("right rewrite", "def test_holds():\n    assert _purba_ran(0, 2 == 2)\n", &["assert 2 == 2"][..]),
        ("asserts list lost", "def test_holds():\n    assert _purba_ran(0, 2 == 2)\n", &[][..]),
        ("condition lost", "def test_holds():\n    assert _purba_ran(0, False)\n", &["assert 2 == 2"][..]),
        ("mark lost", "def test_holds():\n    assert 2 == 2\n", &["assert 2 == 2"][..]),
    ];
    for (name, text, asserts) in cases {
        let (out, code) = run(text, asserts);
        eprintln!("### {name}\n{out}exit={code}");
    }
}
