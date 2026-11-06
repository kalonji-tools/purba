# Every part of the skeleton is Rust, inside the extension module

## Context and Problem Statement

The walking skeleton gives each [part](../../CONTEXT.md#part) a body, and a part may hold Rust, Python or both.
A part needs its language before anyone writes it.
[purba runs inside the tester's interpreter](purba-runs-inside-the-testers-interpreter.md#decision-outcome) makes purba an extension module that the tester's CPython loads.

| fact | source |
|---|---|
| the runners of purba's tests are `bats` and `cargo test`, and neither runs a Python file inside purba | [an implementation follows test-driven development](an-implementation-follows-test-driven-development.md#decision-outcome) |
| a Rust function loaded a test module with `compile()` and `exec()`, called each test, and told `AssertionError` apart, under `cargo test` | [the prototype's run and its test](https://github.com/kalonji-tools/purba/blob/4182dafb603f7c415309eee0f84d235a25825f7d/src/lib.rs#L58-L162) |
| maturin 1.15.0 leaves a `[[bin]]` out of a PyO3 wheel, and warns of nothing | [Where does each part of the walking skeleton live, Rust or Python?](https://github.com/kalonji-tools/purba/issues/536#issuecomment-6161936709) |
| with no venv active, a Rust executable spawned a `python3` without purba, which exited 1 on `ModuleNotFoundError` | [Where does each part of the walking skeleton live, Rust or Python?](https://github.com/kalonji-tools/purba/issues/536#issuecomment-6161936709) |

## Considered Options

- **Execution's import step in a Python file.** Rejected. Only the end-to-end test runs it.
- **Command in a `__main__.py`.** Rejected. Only the end-to-end test runs it.
- **Command as a Rust executable that spawns python.** Rejected. It ships in a second wheel. purba then chooses the interpreter, so a wrong choice exits 1, which reads as a failed test.
- **Every part in Rust, inside the extension module.** Chosen. `cargo test` runs every part.

## Decision Outcome

**Reach:** `crates/**` `/pyproject.toml` `/Cargo.toml`

**Every part of the walking skeleton is Rust, inside the one extension module.**

| part | what forced Rust |
|---|---|
| parsing | [purba parses Python with ruff](purba-parses-python-with-ruff.md#decision-outcome) chooses four Rust crates |
| selection | it reads parsing's AST, and ruff's AST exists only in Rust |
| assertion | it rewrites ruff's AST, and only its load reaches Python |
| execution | PyO3 holds the module, the call and the exception, and `cargo test` runs it |
| command, dispatch, fixtures and reporting | none of them holds Python state in the skeleton |

- **No part holds a Python file.** The wheel's only Python file is the `__init__.py` that maturin writes, as [a tester runs purba on one module](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#context-and-problem-statement) states.
- **Command runs inside the extension module, and never as a Rust executable.** The console script stays as [a tester runs purba on one module](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#decision-outcome) declares it.
- **The design in depth of a part may move that part to Python.**

**Downside:**

- **A Rust test that calls Python must find `libpython` when it runs.** On Linux and macOS, `test:rust` must give the test binary the library's path, or the binary does not load.
- **A part that moves to Python needs a second implementation.** Fixtures is the part most likely to move, because no oxitest code that builds a fixture ever lived in Rust, as [purba divides by responsibility and not by language](purba-divides-by-responsibility-and-not-by-language.md#context-and-problem-statement) shows.

## Confirmation

`mise run test:wheel` checks that the installed `purba` runs a module through `_main`, inside the extension module.
No command checks that each part is Rust.
[The skeleton crosses the seam in execution and assertion](the-skeleton-crosses-the-seam-in-execution-and-assertion.md#decision-outcome) holds what crosses between Rust and Python.
[Each part is a crate](each-part-is-a-crate.md#decision-outcome) says which crate holds each part's code.
