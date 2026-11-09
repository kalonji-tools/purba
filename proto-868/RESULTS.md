# PROTOTYPE 868: where a test puts an interrupt in the rewrite

Throwaway. Never merge. Built on `main` at `340def6`.

**Question.** Command's interrupt check moves from after the rewrite to before it, so an interrupt in the rewrite prints the count. Which seam lets a test put SIGINT inside the rewrite, and pin that order?

## The fix

`crates/command/src/lib.rs`: selection runs in `run_with`, then `not_interrupted()?`, then `rewritten(...)`. `modules` is gone.

## Seams

| seam | code | test | record |
|---|---|---|---|
| A: a command test hands in the rewrite | `run` calls a `#[doc(hidden)] pub fn run_with(arguments, out, rewrite)`; `rewritten` takes the rewrite | `crates/command/tests/proto_868_interrupt_in_rewrite.rs`, own binary: the rewrite it hands in calls `signal.raise_signal(SIGINT)`, then the real rewrite | none |
| C: a fault setting raises SIGINT at each point | `PURBA_FAULT=<point>:interrupt` → `libc::raise(SIGINT)` at command, parse, selection, rewrite, assert; `libc` added to 4 crates | `tests/proto_868_interrupt_points.bats` on the fault wheel: a table of every point, and an exact check of the rewrite row | fault record :31 allows only a panic or an unexpected error; title says "internal error" |
| B: C at the rewrite alone | one arm in assertion; `libc` in assertion | the rewrite row of C | as C |

## Measured

Order of `main` (check after the rewrite):

- A: `Err(Interrupted)` → red (`A-main-order.txt`).
- C: rewrite row prints no line, no count, dies by SIGINT → red (`C-main-order.txt`).

Check moved before the rewrite:

- A: green (`A-moved-order.txt`).
- C rewrite row: `test_refused project error …` then `3 tests: 1 blocked, 2 unreached; 1 project error`, dies by SIGINT → green (`C-moved-order.txt`).
- `mise run test`: exit 0, 222 Rust ok, 548 shell, 69 wheel (`suite-moved-order.txt`). The existing `an_interrupt_before_selection_ends_gives_no_count` still passes.
- `mise run lint:rust`: exit 0 after `cargo fmt`.

Mutants:

- check moved back after the rewrite → A red, C red (the two baseline runs above).
- check deleted → the existing `an_interrupt_before_selection_ends_gives_no_count` red: `Ok(2)` (`mutant-no-check.txt`).

## What C's table adds, and what it does not

- The command, parse and selection rows each print nothing and die by SIGINT, on both orders. `interrupted()` runs each waiting signal handler first, so a signal anywhere before the check reads as one before it. These rows check what the existing command test already checks.
- The assert row matches the existing wheel test "a Ctrl-C in a test's call makes the test interrupted".
- New coverage in C is the rewrite row alone, plus the death by SIGINT through the console script.

## Not varied

- macOS. Linux, CPython 3.12.14 only.
- A rewrite that panics after the interrupt (`NotRewritten`) — not built.
- `libc` is a plain dependency in the prototype; a real C would make it optional under `fault`.
