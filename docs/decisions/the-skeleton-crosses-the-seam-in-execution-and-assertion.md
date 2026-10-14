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

## Considered Options

- **Command reads `std::env::args()`.** Rejected. Command must guess how many leading entries belong to the interpreter.
- **Execution finds the failed assert by the line number of the traceback.** Rejected. A live traceback then reaches execution. Two asserts on one line are ambiguous.
- **Reporting writes each line through `sys.stdout`.** Rejected. Reporting then crosses the seam for every line.
- **Only execution and assertion hold a live Python object.** Chosen. Each other part takes data.

## Decision Outcome

**Reach:** `crates/**`

**The walking skeleton crosses the seam in execution and assertion.**

| part | what crosses into Rust | what stays in Python |
|---|---|---|
| assertion | the module that `compile()` and `exec()` make from the rewritten source | the module's namespace |
| execution | whether the call returned or raised `AssertionError`, and the index of the last assert that ran | the traceback |
| command, selection, parsing, dispatch, fixtures and reporting | nothing | |

- **Command's entry `_main` copies `sys.argv` into a list of strings, and keeps no Python object.** Command takes the arguments as data, as [a concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) states.
- **Fixtures builds nothing in the skeleton, so it holds no Python object.**
- **Assertion rewrites the second assert of `test_a.py` to `assert _purba_ran(1, 1 == 2)`.** The call counts that an assert ran, and records its index.
- **After `AssertionError`, the last index names the failed assert.** The failed line quotes the source text that parsing holds for that assert.
- **Reporting writes to Rust's stdout.**

**Downside:**

- **A line that a test prints can reach a pipe after purba's lines.** Python holds it in the buffer of `sys.stdout`, and Rust's stdout writes first. No skeleton test prints.
- **The rewrite puts the name `_purba_ran` in each module.** A test module that defines `_purba_ran` itself breaks the rewrite.

## Confirmation

No command checks it, because purba holds no product code.
The slices of [Wayfinder: purba's walking skeleton, running every part on main](https://github.com/kalonji-tools/purba/issues/533) add the parts.
