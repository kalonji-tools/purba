# The crate carries an rlib

## Context and Problem Statement

A Rust example that nothing compiles is prose that looks like code.
The failure here is worse than that, because it reports success.

With `crate-type = ["cdylib"]` alone:

| command | result |
|---|---|
| `cargo test` | exits 0, and omits the Doc-tests section entirely |
| `cargo doc` | renders the example |

The output is a rendered, wrong example under a green CI.
Cargo's `Target::doctestable()` admits only `Rlib`, `Lib` and `ProcMacro`, so a bare cdylib has nothing to collect.
PyO3's guide never mentions this.

The same shape appears one level up in the ecosystem.
A comparable project carries 745 lines of Rust doc fences behind a test target that no CI workflow runs.
Its Rust workflow states in a comment that it excludes doc tests on purpose.

## Considered Options

- **A bare cdylib.** Rejected. It is the silent failure above.
- **A separate core library crate.** Deferred, not refused. It is a permanent architectural commitment and belongs to the placement rule. Its evidence cuts both ways, because the project that has this split documents its seam least of any measured.
- **One crate carrying both crate types.** Chosen. The extra rlib is inert to the wheel builder, which produces a byte-identical abi3 wheel. Verified end to end.

## Decision Outcome

The crate declares `crate-type = ["cdylib", "rlib"]`, and `extension-module` is a named feature rather than a default one.

That feature omits libpython.
As a default it would break both documentation lints.

The Rust-example policy is the gate rather than a written rule.
Fences are allowed anywhere and the documentation tests run in CI.
A fence on a bridge item fails to link, so it becomes prose or it moves to the core.
Nothing is written down, because the compiler decides.
That matters, because the measured ecosystem failure is a policy nobody enforced.

**Downside:** bounded by physics. A verified Rust example can cover the Rust core and never the bridge, because `extension-module` omits libpython and any example touching the Python C API needs that feature off. The seam is the part a reader most wants an example of, and it is the part that cannot carry one. The manifest line is also expensive only if forgotten: it costs nothing to keep and it is silent to lose, which is the asymmetry that produced the problem it solves.

## Confirmation

`cargo test --doc`.

It does more than run examples.
On a bare cdylib it exits 101 with "no library targets found", so the crate-type line is guarded by the same command that verifies the examples.

It runs locally today and is not yet wired into CI.
The quality workflow carries it, together with `cargo doc -D warnings` as a lint that is never published.
That workflow does not exist yet.

It reports zero tests until an example exists, and zero is honest.
A comparable project's documentation job is green in CI while collecting zero tests, because its fence syntax is never collected, and the difference is invisible in both the rendered page and the CI log.
A zero here means zero exist.
