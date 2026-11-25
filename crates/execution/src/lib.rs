//! Execution: loads a module, runs a test, and makes its outcome. It keeps the
//! first interrupt of the process.

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("purba runs on Linux and macOS. On Windows, run it under WSL2.");

mod fork;
mod frame;
mod interrupt;
mod order;
mod paths;
mod step;
mod thread;
mod whence;
mod zygote;

use std::any::Any;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;

use order::{Called, Facts};
/// Assertion's rewrite of a module, which a [`Module`] holds.
pub use purba_assertion::Rewrite;
use purba_assertion::{Loaded, Readings, Watched};
use purba_errors::{Error, ErrorKind, Kind, Outcome, Quote, Suppressed, Warning, panic};
use pyo3::exceptions::PyAssertionError;
use pyo3::types::PyAnyMethods;
use pyo3::{Bound, Py, PyAny, PyErr, Python};
use step::{Gives, Met};
pub use zygote::{End, Stop, Zygote};

/// A module that dispatch hands [`Purba::zygote`].
pub struct Module {
    path: PathBuf,
    rewrite: Rewrite,
}

impl Module {
    /// Makes a module from the `path` that its file is read from, and its
    /// rewrite.
    #[must_use]
    pub const fn new(path: PathBuf, rewrite: Rewrite) -> Self {
        Self { path, rewrite }
    }
}

/// The internal error of `caught`. Execution is the one part that builds one.
#[expect(
    clippy::disallowed_methods,
    reason = "execution is the one part that builds an internal error"
)]
const fn internal(caught: panic::Caught) -> Error {
    Error::internal(caught)
}

/// The internal error of a panic's `payload`, which execution caught.
fn caught(payload: &(dyn Any + Send)) -> Error {
    internal(panic::caught(payload, "execution"))
}

/// The outcome of `kind`. Execution is the one part that makes one.
#[expect(
    clippy::disallowed_methods,
    reason = "execution is the one part that makes an outcome"
)]
const fn outcome(kind: Kind) -> Outcome {
    Outcome::new(kind)
}

fn error(err: &PyErr) -> Error {
    Error::project(err.to_string())
}

/// Names what a case replaced since the filters were last held: a hook of the
/// process or the warning filters, as the hook names them, or else the events
/// of the watch of the paths. Gives `None` when it replaced none.
fn replaced(py: Python<'_>) -> Option<purba_assertion::Replaced> {
    thread::replaced(py).or_else(paths::replaced)
}

/// The step that a thread's exception charges: the load of the module, or the
/// test that a test's process ran. It is the step that left the thread alive,
/// or else the step that ran last.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Load,
    Test,
}

/// purba's own process, which forks a zygote for each module.
pub struct Purba<'py> {
    py: Python<'py>,
    hook: &'static Py<thread::Hook>,
}

/// Keeps `root` as the folder that the path in the line of each raise starts
/// from, and `folder` as the folder that the run started in.
///
/// The root takes the place of `folder` as the start of each path. Only the
/// first call of the process counts.
pub fn start_paths_at(root: PathBuf, folder: PathBuf) {
    whence::keep(root, folder);
}

/// Gives `work` purba's process for as long as it runs.
///
/// Before `work`, it keeps the working folder as the folder that the run
/// started in, which a relative file name in the line of a raise resolves
/// against, unless [`start_paths_at`] kept one, and hooks
/// `threading.excepthook` and `sys.unraisablehook`, for the life of the
/// process. So each run imports `threading`. Each zygote inherits both.
///
/// # Panics
///
/// The hook could not take the place of `threading.excepthook` or
/// `sys.unraisablehook`.
pub fn attach<R>(work: impl for<'py> FnOnce(&Purba<'py>) -> R) -> R {
    Python::initialize();
    Python::attach(|py| {
        whence::keep_the_folder();
        let hook = thread::hook(py)
            .expect("the hook takes the place of threading.excepthook and sys.unraisablehook");
        work(&Purba { py, hook })
    })
}

/// Loads a module in a zygote, and runs a test in each child of the zygote.
///
/// It loads at most one module, the one of its process, and keeps it and its
/// path, or keeps only its path when its load failed. It keeps the step that
/// left each thread alive.
pub(crate) struct Worker<'py> {
    py: Python<'py>,
    loaded: Option<Loaded<'py>>,
    path: Option<PathBuf>,
    hook: &'static Py<thread::Hook>,
    owners: Vec<(Py<PyAny>, Step)>,
    last: Option<Step>,
}

/// What a load or a call meets, from before its first line of Python: the
/// window of the first interrupt, and the threads alive then.
struct Window {
    interrupt: interrupt::Window,
    alive: Vec<Py<PyAny>>,
}

impl<'py> Worker<'py> {
    /// A worker that has loaded nothing, and charges each thread through
    /// `hook`, which purba took.
    pub(crate) const fn new(py: Python<'py>, hook: &'static Py<thread::Hook>) -> Self {
        Self {
            py,
            loaded: None,
            path: None,
            hook,
            owners: Vec::new(),
            last: None,
        }
    }

    /// Calls `function` of the module that `load` loaded, and gathers the
    /// facts of its case, when the call ran. Also gives what the step gives
    /// besides them: `Interrupted` when the first interrupt landed in the
    /// step, and when it landed before the test's call, the call does not
    /// run. Each thread that the step leaves alive gets the test as its step,
    /// unless it has one.
    fn gather(&mut self, function: &str) -> (Option<Facts>, Result<Gives<()>, Error>) {
        let window = self.open();
        let (own, interrupted) = self.call(&window, function);
        let met = self.close(&window, Step::Test, interrupted);
        let (facts, own) = match own {
            Ok(facts) => (facts, Ok(())),
            Err(error) => (None, Err(error)),
        };
        (facts, step::decide(own, met))
    }

    /// Loads `module`, the one module of its process, and keeps it, or keeps
    /// no module when its load fails. The load takes the note of a panic, so
    /// no later test takes it. Each thread that the load leaves alive gets the
    /// load as its step. When the first interrupt landed in the load, it gives
    /// no project error, and keeps no module.
    ///
    /// # Errors
    ///
    /// The load raised. An internal error when the module caught a panic and the
    /// panic hook noted it. A later call names the module that did not load.
    pub(crate) fn load(&mut self, module: &Module) -> Result<(), Error> {
        self.path = Some(module.path.clone());
        let window = self.open();
        let py = self.py;
        let (own, interrupted) = if window.interrupt.kept() {
            (Ok(None), true)
        } else {
            thread::hold_filters(py);
            let (own, landed) = interrupt::calling(py, || {
                paths::load(py, || {
                    let shown = whence::shown(&module.path);
                    let loaded =
                        purba_assertion::load_shown(py, &module.path, shown, &module.rewrite)
                            .map_err(|err| error(&err))?;
                    if let Some(replaced) = thread::replaced(py) {
                        return Err(Error::project(format!(
                            "[hook-replaced] the module replaced {} as it loaded, and {}",
                            replaced.what(),
                            replaced.rule()
                        )));
                    }
                    Ok(loaded)
                })
            });
            (own.map(Some), landed)
        };
        let met = self.close(&window, Step::Load, interrupted);
        let (kept, result) = match step::decide(own, met) {
            Ok(Gives::Own(loaded)) => (loaded, Ok(())),
            Ok(Gives::Interrupted) => (None, Ok(())),
            Err(error) => (None, Err(error)),
        };
        self.loaded = kept;
        result
    }

    /// The module that `load` loaded.
    fn module(&self) -> Result<&Loaded<'py>, Error> {
        self.loaded.as_ref().ok_or_else(|| {
            let path = self
                .path
                .as_deref()
                .unwrap_or_else(|| "the module".as_ref());
            Error::project(format!("{} did not load", path.display()))
        })
    }

    /// Opens the window of a step, before its first line of Python.
    fn open(&self) -> Window {
        let interrupt = interrupt::Window::open();
        Window {
            interrupt,
            alive: self.hook.get().alive(self.py),
        }
    }

    /// Closes the window of `step`. Gives `step` each thread that the step left
    /// alive, and takes the note of a panic.
    fn close(&mut self, window: &Window, step: Step, interrupted: bool) -> Met {
        self.own(&window.alive, step);
        Met {
            panic: panic::take(),
            interrupted,
        }
    }

    /// Takes each exception that a thread raised since the last take, in the
    /// order threads raised them, as a project error with its step. A thread
    /// with no step charges the load or the test that ran last. Before any
    /// load or test, it takes nothing.
    pub(crate) fn threads(&self) -> Vec<(Step, Error)> {
        let Some(last) = self.last else {
            return Vec::new();
        };
        self.hook
            .get()
            .take()
            .into_iter()
            .map(|raised| {
                let step = raised
                    .thread
                    .and_then(|thread| {
                        self.owners
                            .iter()
                            .find(|(owned, _)| owned.is(&thread))
                            .map(|&(_, step)| step)
                    })
                    .unwrap_or(last);
                (step, Error::project(raised.text))
            })
            .collect()
    }

    /// Gives `step` each thread that is alive now and was not in `before`, and
    /// has no step yet. `step` then ran last.
    fn own(&mut self, before: &[Py<PyAny>], step: Step) {
        let new: Vec<Py<PyAny>> = self
            .hook
            .get()
            .alive(self.py)
            .into_iter()
            .filter(|thread| {
                !before
                    .iter()
                    .chain(self.owners.iter().map(|(owned, _)| owned))
                    .any(|known| known.is(thread))
            })
            .collect();
        self.owners
            .extend(new.into_iter().map(|thread| (thread, step)));
        self.last = Some(step);
    }

    /// Looks `function` up in the module that `load` loaded. Calls it, unless
    /// the first interrupt landed since `window` opened, and watches the paths
    /// of the call. Also says whether the first interrupt landed in the step.
    /// Gives the facts of the case, or `None` when the first interrupt landed
    /// before the call. When a thread was alive at the first expectation, the
    /// facts hold each later expectation for a replay.
    fn call(&self, window: &Window, function: &str) -> (Result<Option<Facts>, Error>, bool) {
        let py = self.py;
        let loaded = match self.module() {
            Ok(loaded) => loaded,
            Err(error) => return (Err(error), window.interrupt.kept()),
        };
        let test = match loaded.module().getattr(function) {
            Ok(test) => test,
            Err(err) => return (Err(error(&err)), window.interrupt.kept()),
        };
        // A call of an object that is not callable runs no code, and gives
        // CPython's own `TypeError`.
        if !test.is_callable()
            && let Err(err) = test.call0()
        {
            return (Err(error(&err)), window.interrupt.kept());
        }
        let purba_fixtures::Inputs {} = purba_fixtures::inputs(function);
        if window.interrupt.kept() {
            return (Ok(None), true);
        }
        let tested = loaded.test(function);
        let before = tested.first_expectation().unwrap_or(usize::MAX);
        thread::hold_filters(py);
        let watch = paths::Watch::start(py, test.getattr("__code__").ok().as_ref(), before);
        let mut landed = false;
        let watched = loaded.watch(&zygote::READERS, || {
            let (called, inside) = interrupt::calling(py, || {
                std::panic::catch_unwind(AssertUnwindSafe(|| test.call0()))
            });
            landed = inside;
            Ok(called)
        });
        let path = watch.finish(py);
        let Watched {
            result,
            mark,
            readings,
            form,
            raised,
            swallowed,
            missed,
        } = watched;
        let result = match result {
            Ok(Ok(called)) => called.map(drop),
            Ok(Err(payload)) => {
                return (Err(caught(payload.as_ref())), landed);
            }
            Err(err) => Err(err),
        };
        let (called, with) = match result {
            Ok(()) => (Called::Returned, None),
            Err(err) => raised_by(py, &err, &raised, &test),
        };
        let Readings {
            in_order,
            shown,
            alone,
            refused,
            replay,
        } = readings;
        let texts = tested.texts(
            in_order
                .iter()
                .map(|(index, _)| *index)
                .chain(alone.iter().map(|(index, _)| *index))
                .chain(replay.iter().copied()),
        );
        let owned = |faults: Vec<(&'static str, String)>| {
            faults
                .into_iter()
                .map(|(check, words)| (check.to_owned(), words))
                .collect()
        };
        let facts = Facts {
            form: owned(form),
            called,
            with,
            mark: mark.map(|(assert, held)| (assert.to_owned(), held)),
            path,
            swallowed,
            tries: tested.tries().to_vec(),
            refused,
            in_order,
            shown,
            alone,
            later: replay,
            texts,
            missed,
            stated: tested.states_expectation(),
        };
        (Ok(Some(facts)), landed)
    }

    /// Runs `function` of the module that `load` loaded again, as a replay
    /// that reads only the expectation at `index`. At that expectation,
    /// `REPLAYS` sends the verdict and ends the process. It returns when the
    /// call returned before it reached that expectation.
    ///
    /// # Errors
    ///
    /// The module did not load, or has no `function`.
    pub(crate) fn replay(&self, function: &str, index: usize) -> Result<(), Error> {
        let py = self.py;
        let loaded = self.module()?;
        let test = loaded
            .module()
            .getattr(function)
            .map_err(|err| error(&err))?;
        let _ = interrupt::calling(py, || {
            loaded.replay(&zygote::REPLAYS, index, || test.call0())
        });
        Ok(())
    }
}

/// What the call of `test` gave when it raised `err`, and the text of the
/// `with` whose entry or cleanup raised `err`, when one of `raised` did.
fn raised_by(
    py: Python<'_>,
    err: &PyErr,
    raised: &[(Py<PyAny>, String)],
    test: &Bound<'_, PyAny>,
) -> (Called, Option<String>) {
    let value = err.value(py);
    let with = raised
        .iter()
        .find(|(exception, _)| exception.bind(py).is(value))
        .map(|(_, text)| text.clone());
    let called = Called::Raised {
        exception: err.to_string(),
        whence: whence::of(py, err, test),
        assertion: err.is_instance_of::<PyAssertionError>(py),
    };
    (called, with)
}

/// The checks that execution sees, of those that a suppression turns into a
/// warning. Selection's table names the same checks, and a test of dispatch
/// compares the two.
pub const SUPPRESSIBLE: [&str; 4] = [
    "second-path",
    "assert-before-expectation",
    "swallowed-raise",
    "iterator-read",
];

/// Sets each hook of the process that a run needs.
///
/// It wraps Rust's panic hook first, then gives each interrupt signal purba's
/// signal handler. A test of the signal handler makes this call, so it runs
/// the process that a run has.
///
/// # Errors
///
/// `signal` refused the signal handler, or `os.register_at_fork` refused the
/// function that gives a forked child back the signal handlers.
pub fn set_hooks() -> pyo3::PyResult<()> {
    panic::wrap_hook();
    interrupt::install()
}

/// Whether purba's signal handler kept an interrupt.
///
/// A signal that waits runs the signal handler before this check. Before Python
/// starts, no signal handler of purba's exists, so none kept one.
#[must_use]
pub fn interrupted() -> bool {
    Python::try_attach(interrupt::kept).unwrap_or(false)
}
