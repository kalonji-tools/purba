//! The frame that a raise came from, as the line below a raised case names it.

use pyo3::types::{PyAnyMethods, PyStringMethods};
use pyo3::{Bound, IntoPyObject, PyAny, PyErr, PyResult, Python};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The folder that the path of each raise starts from.
static FOLDER: OnceLock<PathBuf> = OnceLock::new();

/// Keeps `folder` as the folder that the path of each raise starts from. Only
/// the first call of the process keeps one.
pub fn keep(folder: PathBuf) {
    let _ = FOLDER.set(folder);
}

/// Keeps the folder that the run started in, before any case can change it,
/// unless a folder is already kept. A folder that cannot be read leaves the
/// folder of each raise in its place.
pub fn keep_the_folder() {
    if let Ok(folder) = std::env::current_dir() {
        keep(folder);
    }
}

/// `path` from the kept folder when it lies under that folder, and `path`
/// itself otherwise.
pub fn shown(path: &Path) -> &Path {
    FOLDER
        .get()
        .and_then(|folder| path.strip_prefix(folder).ok())
        .unwrap_or(path)
}

/// The flag of the code of a generator function.
/// <https://docs.python.org/3.12/library/inspect.html#inspect.CO_GENERATOR>
const GENERATOR: u32 = 0x20;

/// The line that names the frame that `err` came from: the innermost frame of
/// its traceback outside Python's standard library. A file named in angle
/// brackets, such as `<string>` or `<frozen os>`, counts as the standard
/// library's, since the standard library generates such code and the tester
/// cannot open it.
///
/// It is `None` when every frame lies in the standard library, and when that
/// frame runs the code of `test`, or of a comprehension or a `lambda` inside
/// it, since a raise in the test's own code needs no line of its own.
///
/// The path is relative to the kept folder, after each symbolic link resolves,
/// and absolute outside it.
pub fn of(py: Python<'_>, err: &PyErr, test: &Bound<'_, PyAny>) -> Option<String> {
    line(py, err, test).ok().flatten()
}

fn line(py: Python<'_>, err: &PyErr, test: &Bound<'_, PyAny>) -> PyResult<Option<String>> {
    let path = py.import("os.path")?;
    let start = match FOLDER.get() {
        Some(folder) => folder.clone().into_pyobject(py)?.into_any(),
        None => py.import("os")?.getattr("getcwd")?.call0()?,
    };
    // A relative file name resolves against the folder of the run, and not
    // against the folder that the case may have changed to.
    let real = |name: &Bound<'_, PyAny>| -> PyResult<String> {
        let joined = path.getattr("join")?.call1((&start, name))?;
        path.getattr("realpath")?.call1((joined,))?.extract()
    };
    let paths = py.import("sysconfig")?.getattr("get_paths")?.call0()?;
    let mut library = Vec::new();
    for key in ["stdlib", "platstdlib"] {
        library.push(real(&paths.get_item(key)?)?);
    }
    let mut innermost = None;
    let mut traceback = err.traceback(py).map(Bound::into_any);
    while let Some(entry) = traceback {
        let code = entry.getattr("tb_frame")?.getattr("f_code")?;
        let file = code.getattr("co_filename")?;
        let name = file.str()?.to_cow()?.into_owned();
        let in_library = name.starts_with('<') || {
            let file = real(&file)?;
            !file.contains("-packages/")
                && library
                    .iter()
                    .any(|root| file.starts_with(&format!("{root}/")))
        };
        if !in_library {
            innermost = Some((code, entry.getattr("tb_lineno")?.extract::<usize>()?));
        }
        traceback = Some(entry.getattr("tb_next")?).filter(|next| !next.is_none());
    }
    let Some((code, lineno)) = innermost else {
        return Ok(None);
    };
    if test
        .getattr("__code__")
        .is_ok_and(|own| is_own(&own, &code).unwrap_or(false))
    {
        return Ok(None);
    }
    let file = real(&code.getattr("co_filename")?)?;
    let folder = real(&start)?;
    let shown = file
        .strip_prefix(&format!("{folder}/"))
        .map_or_else(|| file.clone(), str::to_owned);
    let function: String = code.getattr("co_name")?.extract()?;
    let flags: u32 = code.getattr("co_flags")?.extract()?;
    Ok(Some(if flags & GENERATOR == 0 {
        format!("raised in {function}() at {shown}:{lineno}")
    } else {
        format!("raised inside the generator {function}() at {shown}:{lineno}")
    }))
}

/// Whether `code` is `own`, the code of a test, or the code of a comprehension
/// or a `lambda` that `own` holds, at any depth.
fn is_own(own: &Bound<'_, PyAny>, code: &Bound<'_, PyAny>) -> PyResult<bool> {
    let kind = own.get_type();
    let mut pending = vec![own.clone()];
    while let Some(current) = pending.pop() {
        if current.is(code) {
            return Ok(true);
        }
        for constant in current.getattr("co_consts")?.try_iter()? {
            let constant = constant?;
            if constant.is_instance(&kind)?
                && constant
                    .getattr("co_name")?
                    .extract::<String>()?
                    .starts_with('<')
            {
                pending.push(constant);
            }
        }
    }
    Ok(false)
}
