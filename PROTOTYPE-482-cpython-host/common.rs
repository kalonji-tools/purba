// PROTOTYPE #482, thrown away. Included by both crates.
use pyo3::prelude::*;

/// Select tests by reading the file as text, without importing it.
pub fn select(path: &str) -> Vec<String> {
    std::fs::read_to_string(path)
        .expect("test file reads")
        .lines()
        .filter_map(|line| line.strip_prefix("def "))
        .filter_map(|rest| rest.split('(').next())
        .filter(|name| name.starts_with("test_"))
        .map(str::to_owned)
        .collect()
}

/// Print which interpreter, prefix and ABI the test runs on.
pub fn report(py: Python<'_>) -> PyResult<()> {
    let sys = py.import("sys")?;
    let sysconfig = py.import("sysconfig")?;
    let version: String = sys.getattr("version")?.extract()?;
    let executable: String = sys.getattr("executable")?.extract()?;
    let prefix: String = sys.getattr("prefix")?.extract()?;
    let base_prefix: String = sys.getattr("base_prefix")?.extract()?;
    let gil_disabled = sysconfig.call_method1("get_config_var", ("Py_GIL_DISABLED",))?;
    let soabi = sysconfig.call_method1("get_config_var", ("SOABI",))?;
    println!("  interpreter : {}", version.split_whitespace().next().unwrap_or("?"));
    println!("  SOABI       : {soabi}   free-threaded: {gil_disabled}");
    println!("  executable  : {executable}");
    println!("  prefix      : {prefix}");
    println!("  base_prefix : {base_prefix}");
    Ok(())
}

/// Import the selected file and call each selected test.
pub fn run_tests(py: Python<'_>, path: &str, tests: &[String]) -> PyResult<()> {
    let file = std::path::Path::new(path);
    let dir = file.parent().and_then(|p| p.to_str()).unwrap_or(".");
    let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
    py.import("sys")?.getattr("path")?.call_method1("insert", (0, dir))?;
    let module = match py.import(stem) {
        Ok(module) => module,
        Err(error) => {
            println!("  IMPORT FAILED: {error}");
            return Ok(());
        }
    };
    for test in tests {
        match module.getattr(test.as_str())?.call0() {
            Ok(_) => println!("  PASS {test}"),
            Err(error) => println!("  FAIL {test}: {error}"),
        }
    }
    Ok(())
}
