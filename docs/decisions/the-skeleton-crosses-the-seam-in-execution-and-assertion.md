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

- **Command reads `std::env::args()`.** Command must guess how many leading entries belong to the interpreter.
- **Execution finds the failed assert by the line number of the traceback.** A live traceback then reaches execution. Two asserts on one line are ambiguous.
- **Reporting writes each line through `sys.stdout`.** Reporting then crosses the seam for every line.
- **An `AssertionError` that no assert raised fails, with the error's text.** A failed line then names no assert. purba then counts a failure that no assert found.
- **A Python file watches the lines and branches of a case.** It would be the one part that is not Rust.

## Decision Outcome

**Reach:** `crates/**`

**Area:** the Rust/Python seam

**The walking skeleton crosses the seam in execution and assertion.**
Each other part takes data.

| part | what crosses into Rust | what stays in Python |
|---|---|---|
| assertion | the module that assertion's load makes from the rewritten source, the index of the last marked assert that did not hold, or else of the last one that ran, and whether it held, whether each [expectation](../../CONTEXT.md#expectation) held, read in order and alone, the `repr` of each side of an expectation that did not hold, as text, the first assert before the first expectation or inside a reading, or the first hook replaced before it, and each exception that a `with` of a test raised or swallowed | the module's namespace |
| execution | whether the call returned or raised `AssertionError`, the type and message of each other exception, as text, and each line and branch that `sys.monitoring` hands execution | the traceback, and the frames that execution and assertion walk to find who called a function and the line of the test |
| command, selection, parsing, dispatch, fixtures and reporting | nothing | |

- **Command's entry `_main` copies `sys.argv` into a list of strings, and keeps no Python object.** Command takes the arguments as data, as [a concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) states.
- **Fixtures builds nothing in the skeleton, so it holds no Python object.**
- **Assertion rewrites the expectation of `test_two` in `test_a.py` to `_purba_reads(1) and _purba_read(1, (1), (2))`.** The first call says whether this process reads the expectation. The second compares the two sides once by `==`, and records the index of the expectation and whether it held. When they did not, it records the `repr` of each side, as text.
- **Assertion marks each other assert: `assert x` becomes `assert _purba_ran(i, x)`.** The call tests the value once, records the index of the assert and whether it held, and gives back that `bool`.
- **The mark records an assert that runs before the first expectation outside the cleanup of a `with`, or inside a reading.** Execution refuses the case.
- **Assertion writes `; _purba_step(n)` after each simple statement of a test before its first expectation.** It wraps each item `m` of each `with` of a test as `_purba_with(n, m)`. `n` is the line of the statement. The step asks execution whether the case replaced a hook. The wrap enters and leaves the manager. It keeps what the entry or the cleanup raised, and each raise that the cleanup swallowed.
- **Execution watches the lines and branches of a case through a callable of the extension module, which `sys.monitoring` calls.** No part needs a Python file, so [every part of the skeleton is Rust inside the extension module](every-part-of-the-skeleton-is-rust-inside-the-extension-module.md#decision-outcome) stays true.
- **A Python thread alive at the first expectation gives a [replay](../../CONTEXT.md#replay) for each later expectation.** [Each test runs in a process of its own](each-test-runs-in-a-process-of-its-own.md#decision-outcome) says which process forks it.
- **Assertion's rewrite copies the text of each assert that it rewrites from the module that command lends it.** [A failed line quotes each expectation that did not hold, or else the last assert that did not hold, without its comments](a-failed-line-quotes-the-last-assert-that-did-not-hold-without-its-comments.md#decision-outcome) says which asserts a failed case quotes.
- **Reporting writes to Rust's stdout.**

[A loaded module joins sys.modules under its file's name, without .py](a-loaded-module-joins-sys-modules-under-its-files-name.md#decision-outcome) holds where assertion's load puts the module.

**Downside:**

- **A line that a test prints can reach a pipe after purba's lines.** Python holds it in the buffer of `sys.stdout`, and Rust's stdout writes first. No skeleton test prints.
- **A test whose call raises `AssertionError` with no marked assert has the outcome `raised`, where pytest reports a failure.** `unittest`'s `assertEqual` is one. A run of only such a test exits 4.

## Confirmation

`mise run test:rust` checks that assertion rewrites the assert of `test_two` in `test_a.py` to `_purba_reads(1) and _purba_read(1, (1), (2))`.
It checks that a module whose tests the rewrite is not given keeps a mark on each assert, `assert _purba_ran(1, 1 == 2)` for that one.
It also checks that the rewrite gives the text of each assert at its index.

It checks that assertion's load runs the rewritten text as a module that holds `_purba_ran`.
It checks that assertion's loaded module gives the text of the last assert that ran in one call, when each one held.
It checks that an assert tests its value once.
It checks that a call to `_purba_ran` with an index that names no assert raises `IndexError`.

It checks that the asserts at a test's end become its expectations, and that each later line keeps its number.
It checks that an expectation after a `;` still parses.
It checks that the first expectation of a case reads each later one alone, and each one in order.
It checks that a Python thread alive at the first expectation leaves the case with no fork.
It checks that a fork that the operating system refuses is kept, and that each expectation is still read in order.

`mise run test:rust` also checks the outcome that execution makes: passed, failed, or the project error of a case with no expectation.
It checks that `AssertionError` gives the outcome `raised` when no assert ran.
It checks that execution refuses an assert before the first expectation, whatever the case does after it.
It checks that the rewrite writes a step after each simple statement of a test before its first expectation.
It checks that the rewrite wraps each item of each `with` of a test.

`mise run test:wheel` checks that `_main` hands command the arguments of the console script.
It checks that a thread alive at the first expectation gives a replay for each later expectation.

No command checks the rest.
