# A tester runs purba on one module, and reads one line for each test

## Context and Problem Statement

The walking skeleton needs a way to start a [run](../../CONTEXT.md#run).
A run must show each [outcome](../../CONTEXT.md#outcome).
How a run starts joins the [public surface](../../CONTEXT.md#public-surface).
[purba's public surface starts empty and has no plugin API](purbas-public-surface-starts-empty-and-has-no-plugin-api.md#decision-outcome) lets a name join only with the issue that needs it.
[A run exits 0 only when it checked something and every test passed](a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md) holds what a run returns.

| fact | source |
|---|---|
| a console script calls its function with no arguments, and passes the integer it returns to `sys.exit` | [the entry points specification](https://github.com/pypa/packaging.python.org/blob/d1ad4ad3d6d6f3a3582e4c6b8a1b6f8284f4cb3b/source/specifications/entry-points.rst?plain=1#L125-L142) |
| maturin wraps an extension module that has no Python source in a package, whose `__init__.py` imports every name in the module's `__all__` | [maturin's PyO3 bindings](https://github.com/PyO3/maturin/blob/v1.15.0/src/binding_generator/pyo3_binding.rs#L147-L158) |
| PyO3 appends each name that a module adds to its `__all__` | [PyO3's `PyModule::add`](https://github.com/PyO3/pyo3/blob/v0.29.2/src/types/module.rs#L487-L501) |
| `python -m` on a package runs its `__main__` submodule, and refuses a package that has none | [CPython's `runpy`](https://github.com/python/cpython/blob/v3.15.0/Lib/runpy.py#L145-L156) |
| command hands the outcomes to reporting, and the diagram draws no stream | [the landscape](../../LANDSCAPE.md#the-parts-and-their-dependencies) |

## Considered Options

- **`python -m purba`, alone or beside the console script.** Rejected. It needs a `__main__.py` in purba's package, so command holds a Python file. [Every part of the skeleton is Rust, inside the extension module](every-part-of-the-skeleton-is-rust-inside-the-extension-module.md#decision-outcome) keeps Python files out.
- **A directory and a query as the input.** Rejected. The query language belongs to the design in depth of selection.
- **A data format for an [integrator](../../CONTEXT.md#integrator) now, such as JSON Lines.** Rejected. Reporting decides a format in its design in depth, and a format chosen now joins the public surface first.
- **A console script on one path to one module, which prints one line for each test.** Chosen. It adds one command and one private name, and the wheel keeps its layout.

## Decision Outcome

**Reach:** `/pyproject.toml` `crates/**`

**A [tester](../../CONTEXT.md#tester) runs `purba <path>` on one module, and reads one line for each test and a count.**

- **`purba` is a console script.** `pyproject.toml` declares `purba = "purba:_main"` under `[project.scripts]`.
- **`_main` stays outside the public surface.** The command name `purba` joins it.
- **A run takes exactly one path to one `.py` file, and no flag.**
- **A run prints to stdout, in source order, once the run ends.**

```
tests/test_a.py::test_one passed
tests/test_a.py::test_two failed: assert 1 == 2
tests/test_a.py::test_three unknown: no assert ran
3 tests: 1 passed, 1 failed, 1 unknown
```

- **Each line holds the path as the tester typed it, then `::`, the name of the test function, and its outcome.**
- **A line that is not `passed` gives a reason.** A failed line quotes the source text of its assert, with no values.
- **The count names `unknown`.** [purba is always strict](purba-is-always-strict.md#decision-outcome) shows and counts it.
- **A run writes no data for an integrator yet.** The integrator reads the exit code.

**Downside:**

- **`python -m purba` fails.** CPython prints `No module named purba.__main__; 'purba' is a package and cannot be directly executed`.
- **A failed line shows no values.** The tester reads them only after assertion shows them in its design in depth.
- **`from purba import *` imports `_main`.** PyO3 lists `_main` in `__all__`, and maturin's `__init__.py` imports each name on that list.
- **The end-to-end test of the walking skeleton reads text.** Reporting rewrites the text in its design in depth, and that test with it.
- **An integrator gets only the exit code until reporting decides a format.**
- **Two spellings of one path print two identities.** `tests/test_a.py` and `./tests/test_a.py` name one module. The design in depth of selection decides how an identity starts.

## Confirmation

No command checks it, because purba holds no product code.
The end-to-end test of [Wayfinder: purba's walking skeleton, running every part on main](https://github.com/kalonji-tools/purba/issues/533) runs `purba` on one module and reads its lines.
[bats tests the installed wheel from outside purba](bats-tests-the-installed-wheel-from-outside-purba.md#decision-outcome) decides what runs that test.
