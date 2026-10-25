# How tools with several stages carry an error from the stage that finds it to the person

This file answers [#655](https://github.com/kalonji-tools/purba/issues/655).
It is never merged.

## Answer

Every tool in this survey turns an error into data at the stage that finds it, and the stage goes on.
A tool stops only at a few fixed points.
It stops before any work when the request is wrong, and at a gate where later work would build on a broken result.
rustc stops between its phases. cargo stops starting new units after the first failed one. pytest stops after collection, before any test.
ruff, rust-analyzer, `cargo --keep-going` and `pytest --continue-on-collection-errors` stop only the work that depends on the error.
A parser built to recover names every error it finds: rustc's, ruff's, rust-analyzer's and nextest's filterset parser.
A reader through serde or TOML stops at its first error, in cargo, nextest and uv, and ruff's rule selectors stop at the first too.
rustc, cargo and nextest print each error when the stage finds it. ruff prints every diagnostic at the end.
A tool marks an error as its own in one of three ways.
The stage that notices the defect marks it, as rustc's `bug!` and cargo's `internal()` do.
A Rust panic, or an exception that escapes a hook, marks it.
Or the entry point gives a class to every error that no stage marked.
Only rustc and nextest give an error inside the tool an exit code that no error of the user shares.
Every Rust tool mixes at least two ways to type an error. The common shape is a typed error inside each crate, a shared diagnostic or a sink between crates, and `anyhow` or `eyre` only at `main`.
pydantic-core and polars keep a Python exception as a `PyErr`, and test its class before any text is made. purba makes the text first.
For purba, an error type that each part returns, and a sink trait that each part declares in its own interface, cross no arrow that the diagram does not draw. Both keep lazy work.
A crate of errors that every part shares is refused by `mise run lint:wiring`, because errors is a concern and not a part.

| tool | inside a stage | where it stops | names every error it finds | an error inside the tool shows as | exit: input error / error inside the tool |
|---|---|---|---|---|---|
| rustc | records, goes on | at fixed checkpoints between phases | yes, then "aborting due to N previous errors" | "the compiler unexpectedly panicked. This is a bug" | 1 / 101 |
| rust-analyzer | records, never stops | never; it caps syntax errors at 128 a file | yes, for each file | an LSP `InternalError` reply, "request handler panicked" | CLI: 1 / not established |
| ruff | records for each file | configuration: at the first unknown rule selector | lint: yes; configuration: no | a `Fatal` diagnostic, "This indicates a bug in Ruff." | 1 / 2, shared with a configuration error |
| cargo | the first failed unit stops new units | `--keep-going` lifts the stop | units: yes; a manifest: no | a note, "this is an unexpected cargo internal error" | 101 / 101 |
| cargo-nextest | its own parsers record, go on | after the first failed test by default; a setup script always | its own parsers: yes; its serde config: no | "Unknown/unexpected failures" exit 1 | 100, 94, 96 and others / 1 |
| pytest | collection records each error | after collection, before any test | collection: yes; missing paths: no | `INTERNALERROR>` lines | 1, 2, 4 / 3, shared with a raising `conftest.py` hook |
| uv | the resolver builds one error from the whole conflict | install: at the first error | the conflict chain: yes; TOML: no | the same lines as a user error | 1 / 2, shared with an argument error |

Each row is [A], read at a tag or measured in this session, unless a section marks it otherwise.

## 1. Stop, or record and go on

### Where each tool decides to stop

| tool | claim | tier | source |
|---|---|---|---|
| rustc | `DiagCtxt` is one sink for every stage: "Certain errors (fatal, bug, unimpl) may cause immediate exit, others log errors for later reporting." | [A] | [rustc_errors/src/lib.rs#L269-L272](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/lib.rs#L269-L272) |
| rustc | The sink hands a diagnostic to the emitter as the stage emits it, not at the end | [A] | [rustc_errors/src/lib.rs#L1362-L1368](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/lib.rs#L1362-L1368) |
| rustc | Parsing stops only on an error it cannot recover from: `parse_error.emit()`, then `guar.raise_fatal()` | [A] | [rustc_interface/src/passes.rs#L70-L75](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_interface/src/passes.rs#L70-L75) |
| rustc | After macro expansion it stops only if a macro gave an error: `if ecx.nb_macro_errors > 0` | [A] | [passes.rs#L218-L220](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_interface/src/passes.rs#L218-L220) |
| rustc | After type check and borrow check run over the whole crate, it stops, but not for lint errors: "Avoid overwhelming user with errors if borrow checking failed." | [A] | [passes.rs#L1205-L1215](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_interface/src/passes.rs#L1205-L1215) |
| rustc | Before codegen it stops on any error or delayed bug, "because codegen will likely cause more ICEs, obscuring the original problem" | [A] | [passes.rs#L1305-L1310](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_interface/src/passes.rs#L1305-L1310) |
| rustc | `passes.rs` holds two more stops: a recursion limit after expansion (L235), and before a run with no codegen (L1329). A stop is an unwind with a marker, `resume_unwind(Box::new(FatalErrorMarker))` | [A] | [passes.rs#L232-L236](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_interface/src/passes.rs#L232-L236); [rustc_span/src/fatal_error.rs#L15-L17](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_span/src/fatal_error.rs#L15-L17) |
| rustc | The count line: "aborting due to {count} previous errors", from a count without duplicates | [A] | [rustc_errors/src/lib.rs#L778-L782](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/lib.rs#L778-L782) |
| rust-analyzer | "we always produce a syntax tree, even for completely invalid files." | [A] | [crates/syntax/src/lib.rs#L69-L73](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/syntax/src/lib.rs#L69-L73) |
| rust-analyzer | Syntax diagnostics: "Only take first 128 errors to prevent slowing down editor/ide" | [A] | [ide-diagnostics/src/lib.rs#L340-L362](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/ide-diagnostics/src/lib.rs#L340-L362) |
| rust-analyzer | With 16 parse errors or more in a file, it skips the semantic diagnostics of that file, "so we can show these faster" | [A] | [ide-diagnostics/src/lib.rs#L428-L433](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/ide-diagnostics/src/lib.rs#L428-L433) |
| rust-analyzer | It sends the syntax diagnostics of a file first, and the semantic ones in a second message | [A] | [main_loop.rs#L735-L745](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/rust-analyzer/src/main_loop.rs#L735-L745) |
| ruff | Each file is linted on its own thread: `paths.par_iter()` | [A] | [crates/ruff/src/commands/check.rs#L83](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff/src/commands/check.rs#L83) |
| ruff | A file it cannot read becomes an `E902` diagnostic, and `lint_path` returns `Ok` | [A] | [crates/ruff/src/diagnostics.rs#L62-L76](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff/src/diagnostics.rs#L62-L76), [L283](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff/src/diagnostics.rs#L283) |
| ruff | With a syntax error, the rules on tokens still run, and only the rules on the AST stop: "Run the AST-based rules only if there are no syntax errors." | [A] | [ruff_linter/src/linter.rs#L196-L197](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff_linter/src/linter.rs#L196-L197) |
| ruff | It folds every file's diagnostics into one list, sorts it, and prints it after the last file | [A] | [check.rs#L166-L182](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff/src/commands/check.rs#L166-L182) |
| ruff | An unknown rule selector stops the configuration at the first one, unless preview makes it a warning | [A] | [ruff_workspace/src/configuration.rs#L104-L116](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff_workspace/src/configuration.rs#L104-L116); P-ruff-3 |
| cargo | After the first error, the job queue starts no new unit unless `keep_going`: `if errors.count == 0 \|\| build_runner.bcx.build_config.keep_going {` | [A] | [src/compiler/job_queue/mod.rs#L818-L823](https://github.com/rust-lang/cargo/blob/0.100.0/src/compiler/job_queue/mod.rs#L818-L823) |
| cargo | Units already running drain: "Do not use `?` or break out of the loop early." | [A] | [job_queue/mod.rs#L815-L817](https://github.com/rust-lang/cargo/blob/0.100.0/src/compiler/job_queue/mod.rs#L815-L817) |
| cargo | `handle_error` prints each error as it arrives, and then counts it | [A] | [job_queue/mod.rs#L916-L933](https://github.com/rust-lang/cargo/blob/0.100.0/src/compiler/job_queue/mod.rs#L916-L933) |
| cargo | The caller gets only the count, wrapped as `AlreadyPrintedError` so `main` does not print it again | [A] | [job_queue/mod.rs#L891-L894](https://github.com/rust-lang/cargo/blob/0.100.0/src/compiler/job_queue/mod.rs#L891-L894) |
| cargo-nextest | Fail-fast is `max_fail: 1` with `TerminateMode::Wait` | [A] | [nextest-runner/src/config/elements/max_fail.rs#L25-L36](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-runner/src/config/elements/max_fail.rs#L25-L36) |
| cargo-nextest | "Setup scripts failing always cause the entire test run to be cancelled (--no-fail-fast is ignored)." | [A] | [nextest-runner/src/runner/dispatcher.rs#L640-L642](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-runner/src/runner/dispatcher.rs#L640-L642) |
| cargo-nextest | The filterset parser "should: always return Ok(_)", and on an error "push an error in the parsing state" | [A] | [nextest-filtering/src/parsing.rs#L4-L14](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-filtering/src/parsing.rs#L4-L14) |
| cargo-nextest | It parses every `-E` argument and partitions all errors before it fails | [A] | [cargo-nextest/src/dispatch/helpers.rs#L165-L180](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/cargo-nextest/src/dispatch/helpers.rs#L165-L180) |
| cargo-nextest | Its config reader through serde holds one error: `DeserializeError(Box<serde_path_to_error::Error<ConfigError>>)` | [A] | [nextest-runner/src/errors.rs#L246-L248](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-runner/src/errors.rs#L246-L248); P-nextest-4 |
| pytest | Collection of each node runs in `CallInfo.from_call`; an exception becomes a `CollectReport` with `outcome = "failed"` | [A] | [src/_pytest/runner.py#L408-L434](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/runner.py#L408-L434) |
| pytest | After collection, any collection error stops the run before a test: `raise session.Interrupted(` | [A] | [src/_pytest/main.py#L397-L401](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/main.py#L397-L401) |
| pytest | A path that does not exist raises `UsageError` at once, for the first such path; a test id that matches nothing is appended to a list, and every one is named | [A] | [main.py#L1143-L1149](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/main.py#L1143-L1149), [L852-L862](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/main.py#L852-L862); P-pytest-6 |
| pytest | A collection error and a failed test add to one counter: `pytest_collectreport = pytest_runtest_logreport` | [A] | [main.py#L705](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/main.py#L705) |
| pytest | It writes each result's letter at once, and the detail of every error after the run | [A] | [src/_pytest/terminal.py#L654-L655](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/terminal.py#L654-L655), [L997-L1007](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/terminal.py#L997-L1007) |
| uv | A failed resolution is one `NoSolutionError` that holds the whole derivation tree | [A] | [crates/uv-resolver/src/error.rs#L426-L428](https://github.com/astral-sh/uv/blob/0.13.5/crates/uv-resolver/src/error.rs#L426-L428) |
| uv | Preparing an install collects with `.try_collect()`, which stops at the first `Err` | [A] source; [C] not measured | [crates/uv-installer/src/preparer.rs#L98-L101](https://github.com/astral-sh/uv/blob/0.13.5/crates/uv-installer/src/preparer.rs#L98-L101) |
| uv | A warning is printed at once by `warn_user!`, through `eprintln!` | [A] | [crates/uv-warnings/src/lib.rs#L78-L91](https://github.com/astral-sh/uv/blob/0.13.5/crates/uv-warnings/src/lib.rs#L78-L91) |

### What the person sees

| probe | what it shows | tier |
|---|---|---|
| P-rustc-1: two type errors in two functions | both, then `error: aborting due to 2 previous errors`, exit 1 | [A] |
| P-rustc-2: a syntax error the parser recovers from, and a type error | both errors; the type check ran | [A] |
| P-rustc-3: a mismatched delimiter, and a type error | only the delimiter error; the type check never ran | [A] |
| P-ra-2: P-rustc-2's file through `rust-analyzer diagnostics` | four errors: two syntax, E0308, and a follow-on E0282 that rustc does not print | [A] |
| P-ruff-1: a file with a syntax error, and a file with F401 and F821 | two syntax diagnostics for one bad line, both lint errors, `Found 4 errors.`, exit 1 | [A] |
| P-ruff-3: `--select NOPE,ALSONOPE` | `ruff failed`, then a `Cause:` line for the unknown selector `NOPE`; `ALSONOPE` is never named; exit 2 | [A] |
| P-cargo-1: two crates of one workspace, each with a type error, `-j1` | `cargo build` names one crate; `cargo build --keep-going` names both; exit 101 | [A] |
| P-cargo-1, `-j4`, no `--keep-going` | both crates are named, because both were running when the first failed | [A] |
| P-nextest-2: `-E 'bogus('` | two errors drawn against the input, `error: failed to parse filterset`, exit 94, and no build | [A] |
| P-nextest-3: a config with two bad filtersets | both sections named, exit 96 | [A] |
| P-nextest-4: a config with two values of the wrong type | only the first, `profile.default.retries`; exit 96 | [A] |
| P-pytest-1: two test modules that fail to import | both named, then `Interrupted: 2 errors during collection`, exit 2 | [A] |
| P-pytest-2: the same, `--continue-on-collection-errors` | both named, the third module's test passes, exit 1 | [A] |
| P-pytest-6: two paths that do not exist | `ERROR: file or directory not found: missing_one.py`; the second is never named; exit 4 | [A] |
| P-pytest-6: two test ids that match nothing | both named, exit 4 | [A] |
| P-uv-1: `pyproject.toml` with an unclosed array | one TOML error, exit 2 | [A] |

- **A gate stops the work that would build on a broken result, and nothing else.** rustc stops before codegen, ruff stops only the AST rules of one file, and cargo stops only units not yet started.
- **Two tools stop the whole run for one error in the project.** pytest stops every module when one fails to collect. nextest cancels the run when a setup script fails, even with `--no-fail-fast`.
- **A parser built to recover names every error it finds.** A reader through serde or TOML stops at the first one (P-nextest-4, P-uv-1, P-cargo-2 in the appendix).
- **Showing early and naming every error do not conflict.** rustc, cargo and nextest print each error when found, and print a count at the end.
- **rustc keeps one error from causing a second.** It printed 2 errors where rust-analyzer printed 4 on the same file (P-rustc-2, P-ra-2). Item 3 shows the mechanism.

## 2. A diagnostic about the input, and an error inside the tool

| tool | how the tool knows the error is its own | what the person sees | exit | tier | source |
|---|---|---|---|---|---|
| rustc | a level, not a type: `Bug` is "For bugs in the compiler. Manifests as an ICE (internal compiler error) panic." | `error: internal compiler error`, then "the compiler unexpectedly panicked. This is a bug", and a link to file a report | 101 | [A] | [rustc_errors/src/lib.rs#L1586-L1587](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/lib.rs#L1586-L1587); [rustc_driver_impl/src/diagnostics.rs#L53-L55](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_driver_impl/src/diagnostics.rs#L53-L55); P-rustc-4 |
| rustc | `DelayedBug` is kept until the end, and "silently dropped" if a real error was emitted | nothing, when the input had an error; an ICE otherwise | 101 | [A] | [rustc_errors/src/lib.rs#L1597-L1601](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/lib.rs#L1597-L1601) |
| rustc | a user error | the diagnostic and the count line | 1, `EXIT_FAILURE`, "used for compilation failures and invalid flags" | [A] | [rustc_driver_impl/src/lib.rs#L114-L115](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_driver_impl/src/lib.rs#L114-L115) |
| rustc | 101 is not in rustc. It is std's "error code for panicking programs" | — | 101 | [A] | [library/std/src/rt.rs#L182-L183](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/rt.rs#L182-L183) |
| rust-analyzer | a panic in a read-only request: "they don't modify the state, so it's OK to recover from their failures" | an LSP error reply with code `InternalError`, "request handler panicked"; the server goes on | — | [A] | [handlers/dispatch.rs#L34-L35](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/rust-analyzer/src/handlers/dispatch.rs#L34-L35), [L349-L363](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/rust-analyzer/src/handlers/dispatch.rs#L349-L363) |
| rust-analyzer | a panic while it computes diagnostics | an empty list of diagnostics for those files, and no message | — | [A] | [main_loop.rs#L730-L740](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/rust-analyzer/src/main_loop.rs#L730-L740) |
| rust-analyzer | a panic in a request that changes state: "this one isn't guarded by `catch_unwind`, so, please, don't make bugs :-)" | not established | — | [A] source; [C] the result | [handlers/dispatch.rs#L42-L44](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/rust-analyzer/src/handlers/dispatch.rs#L42-L44) |
| ruff | a panic while it lints one file, caught by `catch_unwind` for that file | a `Fatal` diagnostic with "This indicates a bug in Ruff." and an issue link; once a run, "Panic during linting indicates a bug in Ruff." on stderr; the other files still lint | 2 | [A] source; [C] not triggered | [check.rs#L193-L214](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff/src/commands/check.rs#L193-L214); [ruff_linter/src/message/mod.rs#L29-L48](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff_linter/src/message/mod.rs#L29-L48); [crates/ruff/src/lib.rs#L474-L495](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff/src/lib.rs#L474-L495) |
| ruff | an `anyhow::Error` that reaches `main`, such as a bad selector: "This communicates that this isn't a linter error but ruff itself hard-errored" | `ruff failed`, then each `Cause:` | 2 | [A] | [crates/ruff/src/main.rs#L51-L78](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff/src/main.rs#L51-L78); P-ruff-3 |
| ruff | `DiagnosticId::InternalError`: "This indicates a bug in the program rather than a user error." | — | — | [A] | [ruff_db/src/diagnostic/mod.rs#L1115-L1118](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff_db/src/diagnostic/mod.rs#L1115-L1118) |
| cargo | a marker type, `InternalError`, anywhere in the `anyhow` chain | "this is an unexpected cargo internal error", and "we would appreciate a bug report" | 101 | [A] | [src/util/errors.rs#L175-L181](https://github.com/rust-lang/cargo/blob/0.100.0/src/util/errors.rs#L175-L181); [src/lib.rs#L195-L212](https://github.com/rust-lang/cargo/blob/0.100.0/src/lib.rs#L195-L212) |
| cargo | any other library error: `CliError::new(err, 101)` | the error and each `Caused by:` | 101; a malformed `Cargo.toml` exits 101 too (P-cargo-2) | [A] | [src/util/errors.rs#L343-L347](https://github.com/rust-lang/cargo/blob/0.100.0/src/util/errors.rs#L343-L347) |
| cargo | a panic: no panic hook in `src/` | not established | std's 101 | [A] the absence; [C] not triggered | `grep -rn "set_hook\|catch_unwind"` in Appendix A |
| cargo-nextest | `ExpectedError` is "An error occurred in a program that nextest ran, not in nextest itself." Each variant has its own code | the error, and each `Caused by:` | 100, 101, 105, 96, 94 and others | [A] | [cargo-nextest/src/errors.rs#L75-L81](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/cargo-nextest/src/errors.rs#L75-L81), [L599-L605](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/cargo-nextest/src/errors.rs#L599-L605) |
| cargo-nextest | anything else: "Unknown/unexpected failures will always result in exit code 1." | not established | 1 | [A] source; [C] not triggered | [nextest-metadata/src/exit_codes.rs#L4-L10](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-metadata/src/exit_codes.rs#L4-L10) |
| pytest | an exception that escapes the session, other than `UsageError`, `Failed` and an interrupt: `except BaseException:` | `INTERNALERROR>` before each line of the traceback, and no summary | 3, `INTERNAL_ERROR`, "An internal error got in the way." | [A] | [config/__init__.py#L101-L122](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/config/__init__.py#L101-L122); [main.py#L346-L350](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/main.py#L346-L350); [config/__init__.py#L1267-L1270](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/config/__init__.py#L1267-L1270); [terminal.py#L963-L974](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/terminal.py#L963-L974) |
| pytest | a `conftest.py` hook of the user that raises takes the same path | about 30 `INTERNALERROR>` lines, mostly pluggy frames, then the user's frame | 3 | [A] | P-pytest-3 |
| pytest | inside a test, the phase decides the word: a raise in `setup` or `teardown` is `error`, in `call` it is `failed` | `ERROR at setup of …` beside `FAILED …` | 1 for both | [A] | [terminal.py#L342-L344](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/terminal.py#L342-L344); P-pytest-4 |
| pytest | text that names a bug in pytest, or a link to report one, on the `INTERNALERROR` path | none found in `wrap_session`, `notify_exception` or `pytest_internalerror` | — | [C] | — |
| uv | the entry point sorts an error in three: `User`, `Argument`, `Unexpected`; an error it cannot downcast is `Unexpected` | the same `error:` and `cause:` lines for all three | 1, 2, 2 | [A] | [crates/uv-command-support/src/lib.rs#L35-L49](https://github.com/astral-sh/uv/blob/0.13.5/crates/uv-command-support/src/lib.rs#L35-L49); [crates/uv/src/lib.rs#L3138-L3171](https://github.com/astral-sh/uv/blob/0.13.5/crates/uv/src/lib.rs#L3138-L3171) |
| uv | each operation crate decides `is_user_failure()`; a malformed `pyproject.toml` is `Unexpected` | — | 2 | [A] | [crates/uv-resolve-operations/src/error.rs#L131-L160](https://github.com/astral-sh/uv/blob/0.13.5/crates/uv-resolve-operations/src/error.rs#L131-L160); [crates/uv-project-commands/src/error.rs#L48-L64](https://github.com/astral-sh/uv/blob/0.13.5/crates/uv-project-commands/src/error.rs#L48-L64); P-uv-1 |
| uv | a panic on the work thread: `.expect("Tokio executor failed, was there a panic?")` | not established | not established | [A] source; [C] not triggered | [crates/uv/src/lib.rs#L3130-L3136](https://github.com/astral-sh/uv/blob/0.13.5/crates/uv/src/lib.rs#L3130-L3136) |

- **A tool marks its own defect in one of three ways.** The stage that notices it says so: rustc's `Bug` level, cargo's `InternalError`. A panic says so: rustc, ruff, rust-analyzer. An exception that escapes a hook says so: pytest.
- **Each such class has a default for an error nobody classified.** uv defaults to `Unexpected`. nextest defaults to "unexpected", exit 1. cargo defaults to an ordinary error, 101, with no note of a bug.
- **A boundary draws the line, and a defect of the user's code on the tool's side of it is a defect of the tool.** pytest's raising `conftest.py` hook exits 3, as a bug in pytest does. A raise inside a test is the test's, whoever wrote the line.
- **Two tools give an error inside the tool a code of its own.** rustc gives 101 and nextest gives 1. ruff's 2 also names a bad selector. uv's 2 also names a bad argument. pytest's 3 also names a raising hook. cargo's 101 names every error.
- **Two tools say "bug" to the person, and both find it by a panic.** rustc prints "This is a bug" and a link. ruff prints "This indicates a bug in Ruff." and a link. cargo prints a note only for an `InternalError` marker.
- **[#616](https://github.com/kalonji-tools/purba/issues/616) measured the same codes from outside** ([research/exit-codes.md](https://github.com/kalonji-tools/purba/blob/9b3fcc44897ffff09a876f00034adf6fe88a81dd/research/exit-codes.md), item 2) [B].

## 3. How the Rust tools type their errors across crates

| tool | an enum for each crate | one shared type | `anyhow` or `eyre` at the boundary | a diagnostic type | a sink that stages write to |
|---|---|---|---|---|---|
| rustc | no `thiserror` in any `compiler/rustc_*` manifest; each crate holds `#[derive(Diagnostic)]` structs, in at least 34 of 77 crates | `Diag`, and the token `ErrorGuaranteed` | no `anyhow` | `Diag<'a, G>`, built from the `Diagnostic` trait | `DiagCtxt` |
| rust-analyzer | no `thiserror` anywhere | `ide_diagnostics::Diagnostic`, and `hir`'s `AnyDiagnostic` enum | `anyhow` in `project-model`, `load-cargo` and `rust-analyzer`; `main` returns `anyhow::Result<ExitCode>` | the same data type | a caller's `acc: &mut Vec<AnyDiagnostic<'db>>` |
| ruff | `thiserror` in 12 of 55 crates, for errors that are not yet diagnostics, such as `SourceError` | `ruff_db::diagnostic::Diagnostic`, with `DiagnosticId` and `Severity` | `anyhow` at the CLI, in `lint_path` and `main` | `Diagnostic`, rendered by `ruff_annotate_snippets`, a fork of annotate-snippets | `LintContext`, a `RefCell<Vec<Diagnostic>>` for each file |
| cargo | `thiserror` in 5 of 25 crates | none | everywhere: `pub type CargoResult<T> = anyhow::Result<T>;`; a kind is a marker type in the chain | `annotate-snippets`, through `Shell::print_report` | `Shell`, through `GlobalContext::shell()` |
| cargo-nextest | `thiserror` in 3 of 8 crates; `nextest-runner/src/errors.rs` alone derives 65 | `ExpectedError` at the boundary only | `color-eyre` in `main` | `miette` for a filterset error, drawn against its input | a reporter callback, fed one `ReporterEvent` at a time |
| uv | `thiserror` in 61 of 91 crates | `UvError`, three classes, each holding an `anyhow::Error` | `anyhow` in 30 crates, at the operation boundary | none; no `miette` in any manifest | a global `warn_user!` for warnings only |

All rows are [A]. Counts are of manifests that name the dependency, from the commands in Appendix A. Links:
rustc [Diag](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/diagnostic.rs#L447-L449), [Diagnostic](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/diagnostic.rs#L104-L109), [AmbiguousPlus](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_parse/src/diagnostics.rs#L17-L21);
rust-analyzer [Diagnostic](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/ide-diagnostics/src/lib.rs#L182-L186), [AnyDiagnostic](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/hir/src/diagnostics.rs#L117-L119), [acc](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/hir/src/lib.rs#L701-L705), [main](https://github.com/rust-lang/rust-analyzer/blob/5ec20d044dbf04895877a55921ddc0ae6e73e127/crates/rust-analyzer/src/bin/main.rs#L28);
ruff [Diagnostic](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff_db/src/diagnostic/mod.rs#L21-L34), [LintContext](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff_linter/src/checkers/ast/mod.rs#L3639-L3648), [SourceError](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff_linter/src/source_kind.rs#L354-L360), [fork](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff_annotate_snippets/README.md#L1-L4);
cargo [CargoResult](https://github.com/rust-lang/cargo/blob/0.100.0/src/util/errors.rs#L11), [Shell](https://github.com/rust-lang/cargo/blob/0.100.0/crates/cargo-util-terminal/src/shell.rs#L15-L26), [print_report](https://github.com/rust-lang/cargo/blob/0.100.0/crates/cargo-util-terminal/src/shell.rs#L428-L450);
nextest [ExpectedError](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/cargo-nextest/src/errors.rs#L81), [ParseSingleError](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-filtering/src/errors.rs#L34-L36), [main](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/cargo-nextest/src/main.rs#L6-L11), [callback](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-runner/src/runner/dispatcher.rs#L500-L512);
uv [UvError](https://github.com/astral-sh/uv/blob/0.13.5/crates/uv-command-support/src/lib.rs#L36-L37), [warn_user](https://github.com/astral-sh/uv/blob/0.13.5/crates/uv-warnings/src/lib.rs#L78-L91).

### How a kind reaches the boundary

| tool | how | tier | source |
|---|---|---|---|
| rustc | `ErrorGuaranteed` proves the person was told. Its field is private, so "values of this type cannot be constructed outside of this crate", and only `emit_diagnostic` mints one | [A] | [rustc_span/src/lib.rs#L2839-L2846](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_span/src/lib.rs#L2839-L2846); [rustc_errors/src/lib.rs#L1382-L1385](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/lib.rs#L1382-L1385) |
| rustc | A stage returns it: `Result<…, ErrorGuaranteed>` | [A] | [rustc_middle/src/queries.rs#L599](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_middle/src/queries.rs#L599) |
| rustc | Data holds it too. `TyKind::Error` "is propagated to avoid useless error messages" | [A] | [rustc_type_ir/src/ty_kind.rs#L343-L346](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_type_ir/src/ty_kind.rs#L343-L346) |
| rustc | A built `Diag` that nobody emits panics on drop, "because a diagnostic was built but not used" | [A] | [rustc_errors/src/diagnostic.rs#L453-L454](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_errors/src/diagnostic.rs#L453-L454) |
| cargo | `main` searches the whole chain: `.any(\|e\| e.downcast_ref::<InternalError>().is_some())` | [A] | [src/lib.rs#L198-L201](https://github.com/rust-lang/cargo/blob/0.100.0/src/lib.rs#L198-L201) |
| cargo | It cuts the chain at `AlreadyPrintedError`, "An error that does not need to be printed because it does not add any new information" | [A] | [src/lib.rs#L222-L233](https://github.com/rust-lang/cargo/blob/0.100.0/src/lib.rs#L222-L233); [src/util/errors.rs#L210-L214](https://github.com/rust-lang/cargo/blob/0.100.0/src/util/errors.rs#L210-L214) |
| cargo-nextest | A library error becomes a variant at the call site, `.map_err(ExpectedError::config_parse_error)`, and an exhaustive `match` gives its code | [A] | [cargo-nextest/src/dispatch/common.rs#L88-L93](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/cargo-nextest/src/dispatch/common.rs#L88-L93); [errors.rs#L599-L605](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/cargo-nextest/src/errors.rs#L599-L605) |
| uv | An operation crate decides the class in its `From<…> for UvError`, and the entry point downcasts: `err.downcast::<UvError>()` | [A] | [crates/uv/src/lib.rs#L3141](https://github.com/astral-sh/uv/blob/0.13.5/crates/uv/src/lib.rs#L3141) |
| ruff | The kind is a field of the one type: `DiagnosticId::Panic`, `Severity::Fatal`; `main` reads the highest severity | [A] | [crates/ruff/src/lib.rs#L474-L495](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff/src/lib.rs#L474-L495) |

- **Every tool mixes at least two of the five ways.** No tool uses one alone.
- **`anyhow` or `eyre` sits only where no later code needs the kind, or the code downcasts to get it back.** cargo and uv downcast to a type they name. A downcast names the type, so the boundary depends on the crate that defines it.
- **The type that crosses crates is data, or a sink that holds data.** rustc, ruff and rust-analyzer share a diagnostic type. cargo, nextest and uv keep one enum or marker at the boundary.

## 4. A Python exception crossing into Rust under PyO3

| claim | tier | source |
|---|---|---|
| A `PyErr` is lazy until read. Reading it by `value`, `get_type` or `is_instance` "will create the full exception object if it was not already created." | [A] | [pyo3 src/err/mod.rs#L41-L43](https://github.com/PyO3/pyo3/blob/v0.30.1/src/err/mod.rs#L41-L43) |
| Normalized, it holds `pvalue: Py<PyBaseException>`; from Python 3.12 the type and traceback come from the value | [A] | [src/err/err_state.rs#L138-L144](https://github.com/PyO3/pyo3/blob/v0.30.1/src/err/err_state.rs#L138-L144) |
| `PyErr` is `Send` and `Sync`, by PyO3's own test | [A] | [src/err/mod.rs#L918-L920](https://github.com/PyO3/pyo3/blob/v0.30.1/src/err/mod.rs#L918-L920) |
| Rust tests the class with `is_instance_of::<T>`, which calls `PyErr_GivenExceptionMatches`, so a subclass matches | [A] | [src/err/mod.rs#L436-L446](https://github.com/PyO3/pyo3/blob/v0.30.1/src/err/mod.rs#L436-L446) |
| `Display` writes the type's `qualname`, then `": "` and `str(value)`; a `str()` that fails gives `<exception str() failed>` | [A] | [src/err/mod.rs#L697-L707](https://github.com/PyO3/pyo3/blob/v0.30.1/src/err/mod.rs#L697-L707) |
| A panic in a `#[pyfunction]` is caught at the trampoline and raised as `PanicException`, which "is derived from BaseException" | [A] | [src/impl_/trampoline.rs#L315-L338](https://github.com/PyO3/pyo3/blob/v0.30.1/src/impl_/trampoline.rs#L315-L338); [src/panic.rs#L11-L16](https://github.com/PyO3/pyo3/blob/v0.30.1/src/panic.rs#L11-L16) |
| `PyErr::take`, which `PyErr::fetch` calls: "If the error is a `PanicException` … then this function will resume the panic." The same holds at v0.29.2, the version purba locks | [A] | [src/err/mod.rs#L281-L293](https://github.com/PyO3/pyo3/blob/v0.30.1/src/err/mod.rs#L281-L293); [v0.29.2 src/err/mod.rs#L285-L293](https://github.com/PyO3/pyo3/blob/v0.29.2/src/err/mod.rs#L285-L293), [L330-L332](https://github.com/PyO3/pyo3/blob/v0.29.2/src/err/mod.rs#L330-L332) |
| pydantic-core's `ValError` has four variants: `LineErrors(Vec<ValLineError>)`, `InternalErr(PyErr)`, `Omit` and `UseDefault`; `From<PyErr>` gives `InternalErr` | [A] | [pydantic-core/src/errors/line_error.rs#L35-L46](https://github.com/pydantic/pydantic/blob/v2.14.1/pydantic-core/src/errors/line_error.rs#L35-L46) |
| `convert_err` tests the class of the user's exception: `ValueError` and its subclasses, then `AssertionError`, become line errors; any other class is `ValError::InternalErr(err)` | [A] | [pydantic-core/src/validators/function.rs#L503-L524](https://github.com/pydantic/pydantic/blob/v2.14.1/pydantic-core/src/validators/function.rs#L503-L524) |
| The reason, in its doc comment: "`TypeError` is now considered as a runtime error to catch errors in function signatures" | [A] | [function.rs#L501-L502](https://github.com/pydantic/pydantic/blob/v2.14.1/pydantic-core/src/validators/function.rs#L501-L502) |
| A line error keeps the original exception object as `error` in its context | [A] | [function.rs#L487-L491](https://github.com/pydantic/pydantic/blob/v2.14.1/pydantic-core/src/validators/function.rs#L487-L491); P-py-1 |
| A model collects every field's line errors in one `Vec`, and returns at once on any other error: `err => return Err(err)` | [A] | [pydantic-core/src/validators/model_fields.rs#L366-L375](https://github.com/pydantic/pydantic/blob/v2.14.1/pydantic-core/src/validators/model_fields.rs#L366-L375) |
| At the boundary back to Python, `ValError::InternalErr(err) => err`: the same exception goes out | [A] | [pydantic-core/src/errors/validation_exception.rs#L61-L91](https://github.com/pydantic/pydantic/blob/v2.14.1/pydantic-core/src/errors/validation_exception.rs#L61-L91) |
| polars' `PolarsError` has a variant for each kind, most holding text, and `Python { error: python::PyErrWrap }`; `From<PyErr>` builds that variant | [A] | [crates/polars-error/src/lib.rs#L83-L114](https://github.com/pola-rs/polars/blob/py-2.1.0/crates/polars-error/src/lib.rs#L83-L114); [crates/polars-error/src/python.rs#L6-L14](https://github.com/pola-rs/polars/blob/py-2.1.0/crates/polars-error/src/python.rs#L6-L14) |
| At the boundary: `PolarsError::Python { error } => error.0`; each other variant maps to its own Python class | [A] | [crates/polars-python/src/error.rs#L73-L154](https://github.com/pola-rs/polars/blob/py-2.1.0/crates/polars-python/src/error.rs#L73-L154) |
| To add the expression to a Python error, polars builds a new instance of the same class with the longer message, and copies the traceback | [A] | [crates/polars-error/src/lib.rs#L349-L358](https://github.com/pola-rs/polars/blob/py-2.1.0/crates/polars-error/src/lib.rs#L349-L358) |
| polars has no variant for its own bug; a grep for "this is a bug" and "please report" in its error and Python crates found nothing | [A] the grep; [C] the absence beyond it | Appendix A |

| probe | result | tier |
|---|---|---|
| P-py-1: pydantic, a model with two invalid `int` fields | one `ValidationError`, `error_count: 2` | [A] |
| P-py-1: a validator on field `a` raises `ValueError`, and `b` is invalid | two line errors: `value_error` and `int_parsing` | [A] |
| P-py-1: a validator on `a` raises `KeyError` or `TypeError`, and `b` is invalid | the raw `KeyError` or `TypeError`; the line error for `b` is lost | [A] |
| P-py-2: polars, `map_elements(lambda x: 1 / 0)` | `builtins.ZeroDivisionError`, `'division by zero'` | [A] |
| P-py-2: polars, `map_batches(lambda s: 1 / 0)` | `ZeroDivisionError`, and the message gains "This error occurred in the following expression" | [A] |
| P-py-2: polars, a missing column | `polars.exceptions.ColumnNotFoundError` | [A] |

- **Both projects decide the kind at the seam, by the class, before any text exists.** pydantic-core calls `is_instance_of`. polars keeps the `PyErr` itself until it returns to Python.
- **pydantic-core names a class as the kind of a defect.** `TypeError` from a validator escapes as itself, to catch a wrong signature.
- **Both return the `PyErr` to Python.** purba returns an outcome to reporting instead, and "an outcome is data" refuses a live Python object there. So purba can keep the class only as data: its module, its `qualname` and its message.
- **PyO3 already keeps a panic a panic across a Python frame.** A panic raised as `PanicException` resumes when Rust fetches it. Section 5 reads what that means for `_purba_ran`.

## 5. What each pattern means for purba

purba today, at [`efb4c9c`](https://github.com/kalonji-tools/purba/tree/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c):

| fact | tier | source |
|---|---|---|
| Execution makes the text of a `PyErr` the error: `Error(err.to_string())` | [A] | [crates/execution/src/lib.rs#L72-L87](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/crates/execution/src/lib.rs#L72-L87) |
| Reporting prints any error as `{test} error: …`, and the tally makes it 4 | [A] | [crates/reporting/src/lib.rs#L40-L42](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/crates/reporting/src/lib.rs#L40-L42), [L63-L73](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/crates/reporting/src/lib.rs#L63-L73) |
| Command hands dispatch's results to reporting, and never names execution's `Error`. Its manifest names no `purba-execution` | [A] | [crates/command/src/lib.rs#L40-L44](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/crates/command/src/lib.rs#L40-L44); [crates/command/Cargo.toml#L11-L15](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/crates/command/Cargo.toml#L11-L15) |
| Dispatch already declares a trait in its own interface, `Test`, and selection's `TestId` implements it | [A] | [crates/dispatch/src/lib.rs#L9-L19](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/crates/dispatch/src/lib.rs#L9-L19) |
| Parsing calls `parse_unchecked_source` and keeps the `Parsed`, which holds every syntax error: `pub fn errors(&self) -> &[ParseError]` in `ruff_python_parser` 0.0.17. No part reads it | [A] | [crates/parsing/src/lib.rs#L44-L52](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/crates/parsing/src/lib.rs#L44-L52); [ruff_python_parser 0.0.17 lib.rs#L405-L408](https://docs.rs/ruff_python_parser/0.0.17/src/ruff_python_parser/lib.rs.html#405-408) |
| `_main` catches a panic and returns 3; the panic hook prints the message | [A] | [crates/purba/src/lib.rs#L33-L38](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/crates/purba/src/lib.rs#L33-L38) |
| `lint:wiring` reads only crates whose name starts with `purba-`, and refuses one whose part the diagram does not draw | [A] | [scripts/check-wiring.sh#L70-L78](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/scripts/check-wiring.sh#L70-L78), [L104-L107](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/scripts/check-wiring.sh#L104-L107) |
| A bare `assert`, a defect of purba's rewrite, prints `NameError` as the test's error, exit 4 (row 7c); a syntax error prints one test error, and each later test of the module `did not load` (row 7) | [B] | [the inventory of failure shapes](https://github.com/kalonji-tools/purba/issues/652#issuecomment-6403261116) |

The parts diagram draws these arrows: command to selection, dispatch, reporting, parsing and assertion; selection and assertion to parsing; dispatch to selection and execution; execution to fixtures and assertion; reporting to execution.
An error that goes back to its caller goes back along the arrow of the call, which the diagram already draws.

### The patterns, measured against purba

| pattern | seen in | interfaces that change | crosses an undrawn arrow? | keeps lazy work? |
|---|---|---|---|---|
| each part returns its own error type | nextest's libraries, uv, ruff before a diagnostic | parsing, execution, assertion, reporting; command maps parsing's errors for reporting | no | yes |
| one error type shared by every part | rustc, ruff, rust-analyzer, uv's `UvError` | every part | yes: refused as `no-part-crate`, unless the diagram gains a part | yes |
| `anyhow` at the boundary | cargo, ruff, rust-analyzer, uv | command, `_main` | a downcast in `_main` adds a use the check skips | yes |
| a diagnostic type that renders itself | rustc, cargo, ruff, nextest's filterset | each part that finds an error, and reporting | no `purba-*` arrow; the check cannot see it | yes |
| a sink that reporting defines | rustc, ruff, cargo | parsing, execution, dispatch | yes: parsing, execution and dispatch to reporting | yes |
| a sink trait that each part declares | rust-analyzer's `acc`, nextest's callback; dispatch's `Test` already has the shape | parsing, execution, dispatch, command | no | yes |

#### Each part returns its own error type

```rust
// purba-parsing: the errors ruff already returns, as data
pub struct SyntaxError { pub line: usize, pub column: usize, pub message: String }
impl Module { pub fn syntax_errors(&self) -> Vec<SyntaxError>; }

// purba-execution: the class kept as data, before any text
pub enum Error {
    Raised { module: String, qualname: String, message: String },
    DidNotLoad { path: PathBuf },
    ExitInsideATest { call: String },
}
```

- **Interfaces:** parsing gains its errors. Execution's `Error` becomes an enum. Reporting matches each variant to a line and a code. Command maps parsing's errors into a type that reporting defines, because reporting draws no arrow to parsing.
- **Arrows:** none new. Execution's `Error` already reaches reporting through dispatch and command. Command holds it without naming it.
- **Lazy work:** holds. Each part reports only what it did.
- **What it gives:** the kind that [#609](https://github.com/kalonji-tools/purba/issues/609) needs reaches reporting, which renders the exit code.

#### One error type shared by every part

```rust
// crates/errors, published as purba-errors
pub enum Kind { Outcome, Diagnostic, Defect }
pub struct Error { kind: Kind, text: String }
// each part's Cargo.toml: purba-errors = { path = "../errors" }
```

- **Interfaces:** every part names `purba_errors::Error`.
- **Arrows:** `lint:wiring` refuses `purba-errors`, because no part of the diagram is called errors. Errors is a concern in the landscape, and [a concern is what no part can own](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/docs/decisions/a-concern-is-what-no-part-can-own.md#decision-outcome).
- **A loophole, not an option:** a crate whose name does not start with `purba-` passes the check unseen.
- **Lazy work:** holds.

#### `anyhow` at the boundary

```rust
// purba-command
pub fn run(arguments: &[String], out: &mut impl Write) -> anyhow::Result<u8>;
// purba, in _main
if error.chain().any(|cause| cause.is::<purba_parsing::Unreadable>()) { /* … */ }
```

- **Interfaces:** command's `Error` enum goes. `_main` downcasts to get a kind back.
- **Arrows:** the downcast makes `purba` use a part other than command. [Each part is a crate](https://github.com/kalonji-tools/purba/blob/efb4c9c2cee14629e70ac7e9ca02442c47d38e6c/docs/decisions/each-part-is-a-crate.md#confirmation) says no command checks that, because the check skips the crate `purba`.
- **What it costs:** today's `command::Error` already plays cargo's `CliError`, with three named cases. `anyhow` names none. Reporting, not `_main`, renders the code of a run that finished.
- **Lazy work:** holds.

#### A diagnostic type that renders itself

```rust
// purba-parsing, with miette
#[derive(thiserror::Error, miette::Diagnostic, Debug)]
#[error("invalid syntax")]
pub struct SyntaxError { #[source_code] source: String, #[label] at: miette::SourceSpan }
```

- **Interfaces:** each part that finds an error returns a type that can render itself. Reporting prints it.
- **Arrows:** a third-party crate is not a `purba-*` edge, so the check sees none.
- **What it costs:** rendering moves out of reporting. The landscape says reporting "Renders each outcome as text for a person, and as data for an agent". `miette::Report` boxes a trait object, so it must become data before it crosses a process, as "an outcome is data" asks.
- **Lazy work:** holds.

#### A sink that reporting defines

```rust
// purba-reporting
pub struct Sink { /* the writer, the tally */ }
impl Sink { pub fn diagnostic(&mut self, line: Line); }
// purba-parsing
pub fn parse(path: &Path, sink: &mut purba_reporting::Sink) -> io::Result<Module>;
```

- **Arrows:** parsing, dispatch and execution would each use reporting. The diagram draws none of those arrows, so `lint:wiring` refuses them.
- **What it gives:** each error prints when found, as in rustc and cargo.
- **Lazy work:** holds.

#### A sink trait that each part declares

```rust
// purba-parsing
pub trait Diagnostics { fn syntax(&mut self, path: &Path, error: SyntaxError); }
pub fn parse(path: &Path, diagnostics: &mut impl Diagnostics) -> io::Result<Module>;

// purba-command
struct ToReporting<'out, W: Write> { out: &'out mut W }
impl<W: Write> purba_parsing::Diagnostics for ToReporting<'_, W> {
    fn syntax(&mut self, path: &Path, error: SyntaxError) { /* a line, through reporting */ }
}
```

- **Interfaces:** each part that finds an error gains a generic parameter. Dispatch implements execution's trait, and command implements dispatch's and parsing's.
- **Arrows:** none new. Each part names only its own trait, as dispatch names `Test` and selection's `TestId` implements it.
- **What it gives:** command can hand reporting a line as soon as a part finds the error.
- **What it costs:** a worker in another process must send each call as data. nextest's events and pytest's serializable reports already do ([reports.py#L221-L230](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/reports.py#L221-L230)).
- **Lazy work:** holds. A sink sees only the work that ran.

### Where purba can stop, for [Does purba refuse each error at the earliest part that can see it?](https://github.com/kalonji-tools/purba/issues/608)

| error | the earliest part that sees it | what the survey does | it stops | work added to see it earlier |
|---|---|---|---|---|
| an argument | command | every tool stops before work | the run | none |
| a path purba cannot read | parsing, through command | pytest names only the first missing path (P-pytest-6) | the run | none |
| a syntax error | parsing, before any test runs | rustc and ruff name each one; ruff stops only that file's AST rules | that module's tests | none: `Parsed` already holds every error |
| a module that raises on import | execution, at its first selected test | pytest imports every module before the first test | that module's tests | importing early breaks lazy work |
| a panic inside purba | any part | ruff catches it for each file and goes on; rustc stops | not decided here | — |

## What this means for purba

- **The kind is lost in one place.** Execution turns a `PyErr` into text at L85-L87. Every tool that keeps a kind decides it before text: by `is_instance_of` (pydantic-core), by a variant (nextest), or by a field (ruff).
- **Each part returning its own type, and a sink trait each part declares, fit the diagram.** Neither needs a new arrow. The shared crate and a sink in reporting each need arrows the diagram does not draw.
- **A syntax error can show before any test, at no cost.** `ruff_python_parser` 0.0.17 already returns every error in the `Parsed` that parsing keeps. Today the first one shows only when CPython's `compile()` raises at load.
- **A follow-on line repeats an error the run already showed.** Each later test of a module that did not load prints `did not load`. rustc's `ErrorGuaranteed` and cargo's `AlreadyPrintedError` exist so that a later stage stays silent.
- **An error in the project that "stops only the tests that depend on it" matches ruff, rust-analyzer and `cargo --keep-going`.** pytest's default, which stops every module for one collection error, does not.
- **A request error can name every error when command parses it.** Owned parsers do: nextest's filterset parser and pytest's test ids. A reader through serde stops at the first. Command reads its arguments without clap.
- **No tool in the survey tells its own defect apart from the user's code when both raise in the user's frame.** pytest's `INTERNALERROR` covers only its hooks. A purba defect that raises in the test, such as row 7c's `NameError`, needs a mechanism no surveyed tool shows [C].
- **A Rust panic inside `_purba_ran` during a test should stop the run with 3.** PyO3 raises it as `PanicException`, and resumes the panic when execution fetches it, so `_main`'s `catch_unwind` takes it. That reading is from PyO3's source; I did not run it [C]. A test that catches `BaseException` would hide it.
- **An error inside purba gets its own code in rustc and nextest only.** In ruff, uv and pytest, the code an internal error gets is shared with an input error. #609 chooses purba's codes.

## Appendix A. Commands

`$R` is `/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/16dca681-fc06-4d1d-a235-dc886367a8a3/scratchpad/research-r3`.
Every source was read on 2026-10-25 from a shallow clone at the tag, with `sed -n` and `grep -n`.

| key | command |
|---|---|
| S1 | `git clone -q --depth 1 --branch <tag> --filter=blob:none https://github.com/<owner>/<repo> $R/<dir>` for rust-lang/rust 1.99.1, astral-sh/ruff 0.17.2, rust-lang/rust-analyzer 2026-10-19, rust-lang/cargo 0.100.0, nextest-rs/nextest cargo-nextest-0.9.149, pytest-dev/pytest 9.1.1, astral-sh/uv 0.13.5, pydantic/pydantic v2.14.1, pola-rs/polars py-2.1.0, PyO3/pyo3 v0.30.1 |
| S2 | `curl -sfL https://raw.githubusercontent.com/PyO3/pyo3/v0.29.2/src/err/mod.rs`; `grep -A1 '^name = "pyo3"$' Cargo.lock` in purba gives `0.29.2` |
| S3 | `sed -n 400,433p ~/.cargo/registry/src/index.crates.io-*/ruff_python_parser-0.0.17/src/lib.rs`; `grep -A2 '^name = "ruff_python_parser"' Cargo.lock` gives `0.0.17` |
| S4 | in purba at `efb4c9c`: `cat -n crates/{command,purba,execution,dispatch,reporting,parsing}/src/lib.rs`; `sed -n 60,115p scripts/check-wiring.sh` |
| C1 | `grep -l '^thiserror' crates/*/Cargo.toml \| wc -l` and the same for `^anyhow`, in ruff, where a dev dependency also counts; in rust-analyzer, where `lib/lsp-server`'s dev dependency on `anyhow` is left out; cargo, nextest and uv counted from `[dependencies]` and `[target.*.dependencies]` only, by `$R/probe-cargo/count.sh` |
| C2 | rustc: `grep -rl 'derive(Diagnostic)' --include=*.rs compiler/`; `grep -l 'thiserror\|anyhow' compiler/rustc_*/Cargo.toml` (empty) |
| C3 | cargo: `grep -rn "set_hook\|catch_unwind" src crates/cargo-util-terminal` (empty) |
| C4 | polars: `grep -rn "this is a bug\|is a bug\|please report" crates/polars-error crates/polars-python/src` (empty) |
| C5 | rustc: `grep -n "abort_if_errors\|raise_fatal\|FatalError.raise" compiler/rustc_interface/src/passes.rs` |
| P-rustc | in `$R/probe-rustc/`: `rustc two_type_errors.rs` (1), `rustc syntax_and_type.rs` (2), `rustc unclosed_and_type.rs` (3), `RUSTC_BOOTSTRAP=1 rustc -Z treat-err-as-bug=1 two_type_errors.rs` (4) |
| P-ra | in `$R/probe-ra/`, a cargo package with no dependencies and `CARGO_NET_OFFLINE=true`: `rust-analyzer diagnostics .` on P-rustc-1's file (1) and on P-rustc-2's file (2) |
| P-ruff | in `$R/probe-ruff-pytest/`, `NO_COLOR=1 uvx --no-config ruff@0.17.2`: `check --no-cache --isolated .` on a syntax-error file and an F401+F821 file (1); with a file of mode 000 (2); `--select NOPE`, `--select NOPE,ALSONOPE`, and `--preview --select NOPE,F` (3) |
| P-pytest | `uvx --no-config --from pytest==9.1.1 python -I -m pytest -q -p no:cacheprovider --color=no`: two modules that import a missing module, and a passing one (1); the same with `--continue-on-collection-errors` (2); a `conftest.py` whose `pytest_collection_modifyitems` raises `RuntimeError` (3); a fixture that raises beside an assert that fails (4); `--bogus-flag` (5); in `$R/probe-mine/py/`, `missing_one.py missing_two.py`, and `test_ok.py::test_nope test_ok.py::test_gone` (6) |
| P-cargo | in `$R/probe-cargo/`, with `CARGO_HOME`, `CARGO_TARGET_DIR` there, `CARGO_TERM_COLOR=never` and `--offline`: a workspace of `a` and `b`, each `pub fn f() -> u32 { "not a number" }`, through `cargo build`, `cargo build --keep-going` and `cargo check`, at `-j1` and `-j4` (1); `[dependencies` with no `]` (2) |
| P-nextest | `nix-shell -p cargo-nextest`: a failing test with and without `--no-fail-fast` (1); `cargo nextest run --offline -E 'bogus('`, and with a second `-E 'test(=a) and and'` (2); `.config/nextest.toml` with `default-filter = "bogus("` and an override `filter = "test(=a) and and"` (3); `retries = "lots"` with `slow-timeout = 7` (4) |
| P-uv | `nix-shell -p uv`, `UV_CACHE_DIR` in scratch, `UV_NO_CONFIG=1 UV_PYTHON_DOWNLOADS=never`: `uv lock --offline` on `dependencies = ["foo"` (1) |
| P-py | `nix-shell -p 'python3.withPackages(ps: [ps.pydantic ps.polars])' --run 'python -I <script>'`: a pydantic model with two `int` fields and a `field_validator("a")` that raises `ValueError`, `AssertionError`, `KeyError`, `TypeError` or `PydanticCustomError` (1); polars `map_elements`, `map_batches` and a missing column (2) |
| L | `gh api repos/O/R/commits/<default branch> --jq .commit.committer.date`; `gh api repos/O/R/releases/latest --jq '.tag_name+" "+.published_at'`; for cargo, `git ls-remote --tags --refs` and `gh api repos/rust-lang/cargo/commits/0.100.0 --jq .commit.committer.date` |

## Appendix B. Versions measured

| tool | source read | version measured |
|---|---|---|
| rustc | 1.99.1 | 1.99.0 (b940084d7 2026-09-28) |
| rust-analyzer | tag 2026-10-19, `5ec20d0` | 1.99.0 (b940084 2026-09-28) |
| ruff | 0.17.2 | 0.17.2, from PyPI by `uvx` |
| cargo | 0.100.0 | 1.99.0 (5f94df478 2026-08-27), the same commit as the source |
| cargo-nextest | 0.9.149 | 0.9.136, from nixpkgs |
| pytest | 9.1.1 | 9.1.1, from PyPI by `uvx`, Python 3.14.4 |
| uv | 0.13.5 | 0.11.21, from nixpkgs; it prints `Caused by:` where 0.13.5 prints `cause:` |
| PyO3 | v0.30.1; v0.29.2 for `take` and `fetch` | not run |
| pydantic-core | pydantic v2.14.1, `pydantic-core/` | pydantic 2.12.5, pydantic-core 2.41.5, Python 3.13.15, from nixpkgs |
| polars | py-2.1.0 | 1.40.1, Python 3.13.15, from nixpkgs |
| `ruff_python_parser` | 0.0.17, the version purba locks | not run |

## Appendix C. Liveness

Measured 2026-10-25.

| project | default branch, last commit | latest release or tag | date |
|---|---|---|---|
| rust-lang/rust | `main`, 2026-10-25 | release 1.99.1 | 2026-10-16 |
| astral-sh/ruff | `main`, 2026-10-25 | release 0.17.2 | 2026-10-22 |
| rust-lang/rust-analyzer | `master`, 2026-10-22 | release 2026-10-19 | 2026-10-19 |
| rust-lang/cargo | `master`, 2026-10-25 | tag 0.100.0 (no GitHub releases) | 2026-08-28 (tag commit) |
| nextest-rs/nextest | `main`, 2026-10-25 | release cargo-nextest-0.9.149 | 2026-10-20 |
| pytest-dev/pytest | `main`, 2026-10-25 | release 9.1.1 | 2026-06-19 |
| astral-sh/uv | `main`, 2026-10-25 | release 0.13.5 | 2026-10-22 |
| pydantic/pydantic, which holds pydantic-core | `main`, 2026-10-23 | release v2.14.1 | 2026-10-11 |
| pydantic/pydantic-core | archived; `main`, 2026-01-07 | release v2.41.5 | 2025-11-04 |
| pola-rs/polars | `main`, 2026-10-25 | release py-2.1.0 | 2026-10-25 |
| PyO3/pyo3 | `main`, 2026-10-25 | release v0.30.1 | 2026-10-14 |
