# An outcome and an error live in a crate that links no Python

## Context and Problem Statement

Execution held the names of an outcome and an error.
Execution links Python.
So each part that read an outcome linked Python too.
But [a concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) says that such a part passes data.
Only a test held that an outcome carries no live Python object.

| fact | source |
|---|---|
| reporting named only `Error`, `Kind`, `Outcome`, `Shown`, `Tally` and `Warning` of execution | [`crates/reporting/src/lib.rs` L11 at `0f560edd`](https://github.com/kalonji-tools/purba/blob/0f560edd/crates/reporting/src/lib.rs#L11) |
| `cargo tree -p purba-reporting -e normal` listed 88 crates, with `pyo3` and `libc` | [The names of an outcome and an error move into purba-errors](https://github.com/kalonji-tools/purba/issues/1194) |
| a field `#[serde(skip)] probe: Option<Py<PyAny>>` on `Outcome` passed `an_outcome_is_data`, and the same field in a crate with no pyo3 stopped at `E0433` | [the probe](https://github.com/kalonji-tools/purba/issues/1191#issuecomment-7095856444) |
| execution built an internal error at five places besides `catch`, and two of them hold no panic: a frame from the child, and a frame that holds no message | [the spec](https://github.com/kalonji-tools/purba/issues/1194#issuecomment-7098505488) |
| clippy refused a call of a listed method of `purba-execution` from reporting, and passed the call from dispatch under `#[expect(clippy::disallowed_methods)]` | [the probe](https://github.com/kalonji-tools/purba/issues/1194#issuecomment-7098505488) |

## Considered Options

- **The names stay in execution.** The invariant rests on a test that a skipped field passes.
- **A new part holds the names.** The part has no job, and it adds six arrows to the parts diagram.
- **Only `catch` builds an internal error.** Execution builds one from a frame of the child, and that frame holds no panic to catch.
- **`Error::internal` and `Outcome::new` are public, and no check limits who calls them.** Any part could build an internal error or an outcome.

## Decision Outcome

**Reach:** `crates/**` `/.cargo/clippy.toml` `scripts/check-wiring.sh` `/LANDSCAPE.md`

**Area:** errors

**The names of an outcome and an error live in `purba-errors`, the crate of the errors concern, and that crate links no Python.**
The compiler then refuses a live Python object in an outcome, an error, a warning or a tally.

- **The crate holds `Outcome`, `Kind`, `Quote`, `Shown`, `Error`, `ErrorKind`, `catch`, `Warning`, `Suppressed`, `Tally` and the module `panic`.** `Kind` holds `Quote`, and `Quote` holds `Shown`, so `Shown` left assertion.
- **It is the crate of a concern, and not a part.** It has no job of its own. The line of errors in [the landscape](../../LANDSCAPE.md#the-cross-cutting-concerns) names it. Each part that the line names may use it. It uses no part.
- **Only execution makes an outcome.** `Outcome::new` is public, and clippy refuses a call of it outside execution.
- **Only execution and `catch` build an internal error.** `Error::internal` is public, and clippy refuses a call of it outside execution. `catch` builds one for a panic that crosses it.
- **Execution calls each constructor from one function, under its one `#[expect]`.**
- **Selection hands out `Warning`.** Command copies no warning into another shape.
- **A panic in `crates/errors` names no part.** The part that caught the panic shows in its place.
- **The names of the strict checks stay in selection, with a copy in execution.** [A concern's shared names may live in a part](a-concerns-shared-names-may-live-in-a-part.md#decision-outcome) holds them.

**Downside:**

- **The clippy list misses an `#[expect]` outside execution, a `[lints]` table that allows `clippy::disallowed_methods`, and a `Deserialize` of an outcome or an error.** The reviewer looks for each.
- **The gate finds the crate by the words of one sentence.** It reads "Its crate is `purba-errors`." in the line of errors. A rewrite of that sentence changes what the gate allows.
- **A use of `purba-errors` draws no arrow.** The parts diagram does not show which parts use it.

## Confirmation

`mise run lint:wiring` refuses a part that uses `purba-errors` when the line of errors does not name the part.
It also refuses a part that `purba-errors` uses.
It refuses pyo3 outside the dev-dependencies of `purba-errors`.
`mise run test:shell` runs `scripts/test/check-wiring.bats`, with one case for each of these shapes.
`mise run test:rust` runs `an_outcome_is_data` in `crates/errors/tests/outcome.rs`, and the two `compile_fail` doc tests on `Outcome` in `crates/errors/src/lib.rs`.

`mise run lint:rust` refuses a call of `Error::internal` or `Outcome::new` outside execution.
No test holds that refusal, so [the probe](https://github.com/kalonji-tools/purba/pull/1200#issuecomment-7099754797) holds it:

- Reporting calls `Outcome::new` and `Error::internal`. `cargo clippy -p purba-reporting` exits 101: "use of a disallowed method `purba_errors::Outcome::new`". It prints the same line for `purba_errors::Error::internal`.
- `cargo tree -p purba-reporting -e normal` lists no `pyo3` and no `libc`.

[Should the names of an outcome and an error live in a crate that links no Python?](https://github.com/kalonji-tools/purba/issues/1191) asks the question.
[An error says its kind, and reporting takes six ways in](an-error-says-its-kind-and-reporting-takes-four-ways-in.md#decision-outcome) holds what an error says.
