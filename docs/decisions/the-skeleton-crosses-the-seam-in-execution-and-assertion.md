# The skeleton crosses the seam in execution and assertion

## Context and Problem Statement

[Every part of the skeleton is Rust, inside the extension module](every-part-of-the-skeleton-is-rust-inside-the-extension-module.md#decision-outcome), so the walking skeleton reaches Python only through PyO3.
[The landscape](../../LANDSCAPE.md#the-cross-cutting-concerns) names the Rust/Python seam a concern of execution, fixtures and assertion.
Each part that reaches Python must say what crosses, and what stays in Python.

| fact | source |
|---|---|
| in a console script, `std::env::args()` starts with the interpreter's path, then the script's path | [Where does each part of the walking skeleton live, Rust or Python?](https://github.com/kalonji-tools/purba/issues/536#issuecomment-6161936709) |
| `sys.argv` starts with the script's name | [the `sys` module](https://docs.python.org/3.14/library/sys.html#sys.argv) |
| `sys.stdout` is block-buffered when it is not interactive | [the `sys` module](https://docs.python.org/3.14/library/sys.html#sys.stdout) |
| pytest 9.1.1 reports as failed a test that raises `AssertionError` from `unittest`'s `assertEqual` | [What a failed line quotes](https://github.com/kalonji-tools/purba/issues/627#issuecomment-6398220855) |

## Considered Options

- **Command reads `std::env::args()`.** Rejected. Command must guess how many leading entries belong to the interpreter.
- **Execution finds the failed assert by the line number of the traceback.** Rejected. A live traceback then reaches execution. Two asserts on one line are ambiguous.
- **Reporting writes each line through `sys.stdout`.** Rejected. Reporting then crosses the seam for every line.
- **An `AssertionError` that no assert raised fails, with the error's text.** Rejected. A failed line then names no assert. purba then counts a failure that no assert found.
- **Only execution and assertion hold a live Python object.** Chosen. Each other part takes data.

## Decision Outcome

**Reach:** `crates/**`

**The walking skeleton crosses the seam in execution and assertion.**

| part | what crosses into Rust | what stays in Python |
|---|---|---|
| assertion | the module that assertion's load makes from the rewritten source, the index of the last assert that ran, and whether it held | the module's namespace |
| execution | whether the call returned or raised `AssertionError`, and the type and message of each error that gives no outcome, as text | the traceback |
| command, selection, parsing, dispatch, fixtures and reporting | nothing | |

- **Command's entry `_main` copies `sys.argv` into a list of strings, and keeps no Python object.** Command takes the arguments as data, as [a concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) states.
- **Fixtures builds nothing in the skeleton, so it holds no Python object.**
- **Assertion rewrites the second assert of `test_a.py` to `assert _purba_ran(1, 1 == 2)`.** The call tests the value once, records the index of the assert and whether it held, and gives back that `bool`.
- **A test fails only when the last assert that ran did not hold, and the test raised `AssertionError`.** The failed line quotes the source text of that assert. Assertion's rewrite copies that text from the module that command lends it. An `AssertionError` gives an error in place of an outcome when no marked assert ran or the last one held.
- **Reporting writes to Rust's stdout.**

[A loaded module joins sys.modules under its file's name, without .py](a-loaded-module-joins-sys-modules-under-its-files-name.md#decision-outcome) holds where assertion's load puts the module.

**Downside:**

- **A line that a test prints can reach a pipe after purba's lines.** Python holds it in the buffer of `sys.stdout`, and Rust's stdout writes first. No skeleton test prints.
- **The load puts the name `_purba_ran` in each module.** A test module that defines `_purba_ran` itself breaks the rewrite.
- **A call that raises `AssertionError` with no marked assert gives an error, where pytest reports a failure.** `unittest`'s `assertEqual` is one. A run of only such a test exits 4.
- **A test that catches the `AssertionError` of an assert that did not hold, and then raises `AssertionError` with no assert, fails on the caught assert.** The call records the last assert that ran, and not the error that the test raised.

## Confirmation

`mise run test:rust` checks that assertion rewrites the second assert of `test_a.py` to `assert _purba_ran(1, 1 == 2)`.
It also checks that the rewrite gives the source text of each assert at its index.
It checks that assertion's load runs the rewritten text as a module that holds `_purba_ran`.
It checks that assertion's loaded module gives the text of the last assert that ran in one call, if any ran, and whether it held.
It checks that an assert tests its value once.
It checks that a call to `_purba_ran` with an index that names no assert raises `IndexError`.

`mise run test:rust` also checks that execution makes a passed, failed or `unknown` outcome from the call and the last assert that ran.
It checks that `AssertionError` gives an error when no assert ran or the last one held.

`mise run test:wheel` checks that `_main` hands command the arguments of the console script.

No command checks the rest.
