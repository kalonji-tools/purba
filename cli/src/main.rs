//! PROTOTYPE for #536 Q2a: a Rust executable that finds a python and spawns it.
use std::path::PathBuf;
use std::process::{Command, exit};

/// The rule a Rust executable must own: which python runs the tests.
fn find_python() -> (PathBuf, &'static str) {
    let bin = std::env::current_exe().ok().and_then(|p| p.parent().map(PathBuf::from));
    if let Some(dir) = bin {
        for name in ["python", "python.exe"] {
            let beside = dir.join(name);
            if beside.exists() {
                return (beside, "the python beside the executable");
            }
        }
    }
    if let Some(venv) = std::env::var_os("VIRTUAL_ENV") {
        return (PathBuf::from(venv).join("bin/python"), "$VIRTUAL_ENV");
    }
    (PathBuf::from("python3"), "python3 on PATH")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    eprintln!("[proto] executable's own argv: {args:?}");
    let [_, path] = args.as_slice() else { exit(3) };
    let (python, why) = find_python();
    eprintln!("[proto] chose {} by {why}", python.display());
    let status = Command::new(&python)
        .args(["-c", "import sys, purba; sys.exit(purba._run(sys.argv[1]))", path])
        .status();
    match status {
        Ok(s) => exit(s.code().unwrap_or(3)),
        Err(e) => {
            eprintln!("[proto] could not start {}: {e}", python.display());
            exit(3)
        }
    }
}
