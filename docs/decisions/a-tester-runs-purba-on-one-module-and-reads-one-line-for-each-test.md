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
| dispatch hands command each test beside its outcome, or beside the error it gave in place of one | [the landscape](../../LANDSCAPE.md#the-parts-and-their-dependencies) |

## Considered Options

- **`python -m purba`, alone or beside the console script.** It needs a `__main__.py` in purba's package, so command holds a Python file. [Every part of the skeleton is Rust, inside the extension module](every-part-of-the-skeleton-is-rust-inside-the-extension-module.md#decision-outcome) keeps Python files out.
- **A directory and a query as the input.** The query language belongs to the design in depth of selection.
- **A data format for an [integrator](../../CONTEXT.md#integrator) now, such as JSON Lines.** Reporting decides a format in its design in depth, and a format chosen now joins the public surface first.
- **A test that raised prints as failed.** The tester cannot tell a defect that an assert found from an exception that stopped the test.
- **A reason over several lines prints each line break as `\n`.** An escaped reason is hard to read.
- **A reason over several lines prints only its first line.** A cut reason hides the rest of the assert or the error.

## Decision Outcome

**Reach:** `/pyproject.toml` `crates/**`

**Area:** command

**A [tester](../../CONTEXT.md#tester) runs `purba <path>` on one module, and reads one line for each test and a count.**
A console script adds one command and one private name, and the wheel keeps its layout.

- **`purba` is a console script.** `pyproject.toml` declares `purba = "purba:_main"` under `[project.scripts]`.
- **`_main` stays outside the public surface.** The command name `purba` joins it.
- **A run takes exactly one path to one `.py` file, and no flag.**
- **For a usage error, a module that purba cannot read, or a refused write, purba writes one line to stderr, which says why.** The line is `purba: invocation error: `, then `usage: purba <path to one .py file>`, `cannot read <path>: <error>` or `cannot write to stdout: <error>`. A module that declares an encoding other than UTF-8 takes the `cannot read` line.
- **A run prints to stdout the line of each test that selection refuses, then each other test's line, and the count.**

  ```mermaid
  flowchart TD
    parse{"does the module parse, and does selection meet no internal error?"}
    parse -- no --> stopped{"an interrupt before selection ends?"}
    parse -- yes --> selected{"an interrupt before selection ends?"}
    stopped -- yes --> silent["no line and no count"]
    selected -- yes --> silent
    stopped -- no --> module["the module's line, and no line for a test"]
    selected -- no --> refused["the line of each test that selection refuses, in source order"]
    refused -- "the rewrite met an internal error" --> blocked["the module's line, then each other test's line as blocked"]
    refused -- "the rewrite met none" --> others["each other test's line, in source order, as dispatch hands the test on"]
    module --> count["the count"]
    blocked --> count
    others --> count
  ```

  A module that does not load prints its line, then the line of each of its tests. A load that the first interrupt lands in prints no line, unless it meets an internal error. The line of a thread's exception prints when purba takes the exception. It follows the line of the test that ran then, and the error of a load prints before the first test of its module. An [unreached test](../../CONTEXT.md#unreached-test) prints no line, as [An interrupted run prints its count, and dies by the signal](an-interrupted-run-prints-its-count-and-dies-by-the-signal.md#decision-outcome) says. [An interrupted run prints its count, and dies by the signal](an-interrupted-run-prints-its-count-and-dies-by-the-signal.md#decision-outcome) holds why a run that an interrupt stops before selection ends prints no count.

```
tests/test_a.py::test_one passed
tests/test_a.py::test_two failed: assert 1 == 2
tests/test_a.py::test_three unknown: no assert ran
3 tests: 1 passed, 1 failed, 1 unknown
```

- **Each line holds the path as the tester typed it, then `::` and the name of the test function.** Then it names the test's outcome, `blocked`, or the kind of its error. The line of a module, or of a thread that its load left alive, holds its path, then the kind of its error.
- **A line that is not `passed` or `interrupted` gives a reason.** A failed line quotes the text of its assert, with no values. [A failed line quotes the last assert that did not hold, without its comments](a-failed-line-quotes-the-last-assert-that-did-not-hold-without-its-comments.md#decision-outcome) says which assert a failed line quotes, and which part of its text. An `unknown` line says `no assert ran`. [A tuple assert keeps CPython's warning and counts as no assert](a-tuple-assert-keeps-cpythons-warning-and-counts-as-no-assert.md#decision-outcome) says which assert counts as none. A `raised` line gives the type and message of the exception, such as `ValueError: boom`. The line of an error gives its text. A `blocked` line names the module that did not load.
- **A reason prints on one line.** Its lines join with one space, each without the whitespace at its ends. An empty line drops out.
- **The count names the tests, then each outcome, `blocked`, `unreached` and each kind of error above 0.** `test` and each kind of error take an `s`, except at 1. [purba is always strict](purba-is-always-strict.md#decision-outcome) shows and counts `unknown`. [A line names its kind in the glossary's words](a-line-names-its-kind-in-the-glossarys-words.md#decision-outcome) gives the order of the columns.
- **A run writes no data for an integrator yet.** The integrator reads the exit code.

**Downside:**

- **`python -m purba` fails.** CPython prints `No module named purba.__main__; 'purba' is a package and cannot be directly executed`.
- **A failed line shows no values.** The tester reads them only after assertion shows them in its design in depth.
- **A reason changes a string literal that spans several lines.** Each run of whitespace in it that holds a line break prints as one space.
- **`from purba import *` imports `_main`.** PyO3 lists `_main` in `__all__`, and maturin's `__init__.py` imports each name on that list.
- **The end-to-end test of the walking skeleton reads text.** Reporting rewrites the text in its design in depth, and that test with it.
- **An integrator gets only the exit code until reporting decides a format.**
- **Two spellings of one path print two identities.** `tests/test_a.py` and `./tests/test_a.py` name one module. The design in depth of selection decides how an identity starts.

## Confirmation

`mise run test:rust` checks each line and the count that reporting writes.
It also checks the text of each error that command returns.
`mise run test:wheel` runs the installed `purba` on `test_a.py`, and checks the lines it prints.
It checks that `_main` writes the line of a usage error to stderr.
It checks that a test that aborts purba leaves the line of each test before it.
It checks the lines of a module that does not parse, and of one that does not load.

[bats tests the installed wheel from outside purba](bats-tests-the-installed-wheel-from-outside-purba.md#decision-outcome) decides what `mise run test:wheel` runs.
[A test that selection refuses prints before any test runs](a-test-that-selection-refuses-prints-before-any-test-runs.md#decision-outcome) says when a refused test's line prints.

No command checks the rest.
