//! PROTOTYPE lib: the only crate that uses PyO3.
use purba_core::execution::{Call, Interpreter};
use pyo3::exceptions::PyAssertionError;
use pyo3::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static RAN: AtomicUsize = AtomicUsize::new(0);
static LAST: AtomicUsize = AtomicUsize::new(0);

#[pyfunction]
fn _purba_ran(index: usize, value: Bound<'_, PyAny>) -> Bound<'_, PyAny> {
    RAN.fetch_add(1, Ordering::SeqCst);
    LAST.store(index, Ordering::SeqCst);
    value
}

struct Python3<'py> {
    py: Python<'py>,
    module: Option<Bound<'py, PyAny>>,
}

impl Interpreter for Python3<'_> {
    fn load(&mut self, path: &str, rewritten: &str) -> Result<(), String> {
        let py = self.py;
        let module = (|| -> PyResult<_> {
            let module = py
                .import("types")?
                .getattr("ModuleType")?
                .call1(("test_module",))?;
            let globals = module.getattr("__dict__")?;
            globals.set_item("__file__", path)?;
            py.import("sys")?
                .getattr("modules")?
                .set_item("test_module", &module)?;
            globals.set_item("_purba_ran", wrap_pyfunction!(_purba_ran, py)?)?;
            let builtins = py.import("builtins")?;
            let code = builtins
                .getattr("compile")?
                .call1((rewritten, path, "exec"))?;
            builtins.getattr("exec")?.call1((code, &globals))?;
            Ok(module)
        })()
        .map_err(|e| e.to_string())?;
        self.module = Some(module);
        Ok(())
    }

    fn call(&mut self, name: &str) -> Call {
        let before = RAN.load(Ordering::SeqCst);
        let Some(module) = &self.module else {
            return Call::Raised("no module".into());
        };
        match module.getattr(name).and_then(|f| f.call0()) {
            Err(e) if e.is_instance_of::<PyAssertionError>(self.py) => Call::AssertionFailed {
                assert_index: LAST.load(Ordering::SeqCst),
                message: e
                    .value(self.py)
                    .str()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
            },
            Err(e) => Call::Raised(e.to_string()),
            Ok(_) => Call::Returned {
                asserts_ran: RAN.load(Ordering::SeqCst) - before,
            },
        }
    }
}

#[pyfunction]
fn _main(py: Python<'_>) -> i32 {
    let argv: PyResult<Vec<String>> = py.import("sys").and_then(|s| s.getattr("argv")?.extract());
    argv.map_or(3, |argv| {
        purba_cli::run(&argv[1..], &mut Python3 { py, module: None })
    })
}

#[pymodule]
fn purba(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_main, module)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rewritten_module_runs() {
        Python::initialize();
        Python::attach(|py| {
            let mut python = Python3 { py, module: None };
            let (rewritten, _) =
                purba_core::assertion::rewrite("def test_two():\n    assert 1 == 2\n");
            python.load("test_a.py", &rewritten).unwrap();
            assert_eq!(
                python.call("test_two"),
                Call::AssertionFailed {
                    assert_index: 0,
                    message: String::new(),
                },
                "1 == 2 fails"
            );
        });
    }
}
