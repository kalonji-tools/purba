# A command test checks only command's rules

## Context and Problem Statement

Command starts a run, and every other part takes part in it.
A test of command that compares the whole stdout also checks what reporting prints and what each other part finds.
A change to another part then edits the tests of command.

| fact | source |
|---|---|
| the count line is pinned 8 times in the tests of command | [`crates/command/tests/run.rs`](https://github.com/kalonji-tools/purba/blob/bc83b7a/crates/command/tests/run.rs) |
| 6 of 16 tests of command check a rule of another part: CPython's errors, ruff's messages and the hook's name | [`crates/command/tests/run.rs`](https://github.com/kalonji-tools/purba/blob/bc83b7a/crates/command/tests/run.rs) |
| no record requires a test of command to compare the whole stdout | [Execution counts each error by its kind, and command's tests check only command's rules](https://github.com/kalonji-tools/purba/issues/748) |

## Considered Options

- **A test of command compares the whole stdout.** Rejected. A change to reporting's count line fails each test of command that pins it.
- **A test of command checks only command's rules.** Chosen. Each rule fails the tests of the part that owns it.

## Decision Outcome

**Reach:** `crates/command/tests/**`

**Topic:** One section of the product design; 

**A command test checks only command's rules.**

- **A test of command checks a rule that command owns.** Command owns these rules:
  - It reads the arguments.
  - It sets the order of the lines, and reporting writes them.
  - It stops the run at an invocation error.
  - It runs no test after a stop.
  - It stops the run, with no line and no count, at an interrupt before selection ends.
  - It returns the exit code.
- **A test of command checks the test and the kind of each line, and the exit code.** It pins no reason and no count line.
- **The tests of each other part check that part's rules.** `crates/assertion/tests/load.rs` checks CPython's errors at the load, and `crates/parsing/tests/parse.rs` checks ruff's messages.

**Downside:**

- **A test of command reads the kind of a line in reporting's words.** A change to those words still edits the tests of command.
- **No test of command sees a wrong reason or a wrong count.** Reporting's own tests, and the tests of each part that finds an error, must see it. `tests/*.bats` sees it on the installed wheel.

## Confirmation

`mise run test:rust` checks, through command, the order of the lines, what stops the run, what runs after a stop, and the exit code.
No check refuses a test of command that pins a reason or a count line.
A change of the word `tests` in reporting's count line fails no test of command.
