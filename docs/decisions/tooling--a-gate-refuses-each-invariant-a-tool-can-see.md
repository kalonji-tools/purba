# A gate refuses each invariant a tool can see, from the first code it reads

## Context and Problem Statement

[An invariant needs no design of its own](an-invariant-needs-no-design-of-its-own.md#decision-outcome) names the invariants between purba's parts.
No check refuses a breach of one yet.
[A gate owns the mechanical standard](a-gate-owns-the-mechanical.md#decision-outcome) applies to them.
[A standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md#decision-outcome) keeps out each one that no command decides.
purba holds no product code, so no gate over an invariant has anything to read yet.

| fact | source |
|---|---|
| `src/` held one file, `src/lib.rs`, and it held no product code | [`src/`](https://github.com/kalonji-tools/purba/tree/4c57d5049d9782f114243868a3be8774ffb1810b/src), [`src/lib.rs`](https://github.com/kalonji-tools/purba/blob/4c57d5049d9782f114243868a3be8774ffb1810b/src/lib.rs#L3-L5) |
| a check that passes on a clean tree and refuses nothing is a suggestion | [a gate owns the mechanical standard](a-gate-owns-the-mechanical.md#confirmation) |
| the commit that first names a ruff crate carries the declaration and its check together | [purba parses Python with ruff](https://github.com/kalonji-tools/purba/blob/b6fd3a39b19578e9c4f2a3b7cc9dd75c675d71d5/docs/decisions/purba-parses-python-with-ruff.md#confirmation) |
| clippy's `disallowed-methods` refused a call to `ruff_python_parser::parse_module` outside the parsing module, which carried the one `#[expect]` | [Which of purba's invariants does a gate refuse?](https://github.com/kalonji-tools/purba/issues/512#issuecomment-6127311542) |
| the compiler refused a `Py<PyAny>` inside an outcome that derives `Serialize` and `Deserialize` | [Which of purba's invariants does a gate refuse?](https://github.com/kalonji-tools/purba/issues/512#issuecomment-6127311542) |
| with pyo3's `serde` feature on, a `Py<T>` serialises when `T` is a `#[pyclass]` that serialises | [pyo3 v0.29.2](https://github.com/PyO3/pyo3/blob/v0.29.2/src/conversions/serde.rs#L18-L20) |
| a `clippy.toml` at the root of a workspace refused a disallowed call inside a member crate that takes the workspace's lints, and only warned in one that does not | [the probes' output](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6170716506) |
| ruff exposes more than one function that parses | [Which of purba's invariants does a gate refuse?](https://github.com/kalonji-tools/purba/issues/512#issuecomment-6127311542) |
| `.config/mise.toml` pinned no Python linter | [`.config/mise.toml`](https://github.com/kalonji-tools/purba/blob/4c57d5049d9782f114243868a3be8774ffb1810b/.config/mise.toml#L9-L64) |
| a private `Cell` field that changes through `&self`, and an `Option<Py<PyAny>>` field that `#[serde(skip)]` leaves out, each compiled in a struct that derives `Serialize` and `Deserialize`, on purba's pins and serde `1.0.229`, with pyo3's `serde` feature off | [Which of purba's invariants does a gate refuse?](https://github.com/kalonji-tools/purba/issues/512#issuecomment-6141818801) |
| `#[allow(C)]` overrides the check for C, so that its violations go unreported, and `#[expect(C)]` suppresses C too | [the Rust Reference](https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-check-attributes) |

## Considered Options

- **A gate refuses every breach, and only breaches.** Rejected. Every invariant fails that bar, so the reviewer would keep each of them. A tool sees most breaches of each invariant in the table below.
- **A gate refuses only breaches, and names what it misses.** Chosen. For each invariant that a gate refuses, the reviewer keeps what the table names as missed.
- **Every gate at the walking skeleton.** Rejected. The skeleton adds one part in each slice, so the code of an early slice would stand without its gate until the last slice merges.
- **Each gate at the design in depth of its part.** Rejected. Code would stand without its gate until that design merges.
- **Each gate in the pull request that adds the first code it reads.** Chosen. The gate runs red on real code, and not on an empty tree.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `/Cargo.toml` `/.cargo/config.toml` `/.config/tasks.toml` `clippy.toml` `crates/**` `scripts/check-wiring.sh` `*.py`

**A gate refuses each invariant that a tool can see, from the first code the gate reads.**

- **Each refusal of a gate is a breach, and a gate may miss a breach.** The table names each miss that a probe or a source shows, and the reviewer checks for others too.
- **The pull request that adds the first code a gate reads also adds the gate.** It runs the gate green and red.
- **The probes of clippy and the compiler show that a gate can decide a breach.** They are evidence, and they are not a design.

| invariant | a gate can refuse it when | the code that first builds it | what it misses |
|---|---|---|---|
| one parser | cargo denies each warning, and `.cargo/config.toml` points clippy at its list | the first ruff crate in `Cargo.toml`, and the first Python file of a part for the Python half | a function that parses and that `disallowed-methods` omits, a call under its own `#[allow]` or `#[expect]`, a `[lints]` table that allows `clippy::disallowed_methods`, and Python's `ast.parse` until the Python half is built |
| an outcome is data | the outcome must encode to bytes | the outcome type | a field that `#[serde(skip)]` leaves out, a `Serialize` written by hand that does not encode, and a `Py<T>` of a `#[pyclass]` that serialises, if pyo3's `serde` feature is on |
| an outcome is final | the outcome has private fields and no method that takes `&mut self` | the outcome type | a `Cell` or another field that changes through `&self`, code of execution that writes the field of an outcome, and a part that builds a replacement outcome, such as through `Deserialize` |
| one identity | only selection constructs a test's identity, and a `compile_fail` doc test refuses a public field | the identity type | a part that formats its own name for a test, a public constructor that selection adds such as a derived `Default`, and a type of another part that implements dispatch's `Test` |
| lazy work | selection and execution both exist | the walking skeleton, as a test that runs one of two modules and refuses any import of the other | a module that only a query leaves out, a rewrite that reads the other module and never imports it, and a path that the test does not take |
| the diagram is the only wiring | each part has a boundary that a tool reads | the code of the second part | a `pub use` of another part's crate, a use by the crate `purba`, a message of the sequence drawn with an arrow other than `->>` or `-->>` or with no space around its arrow, and an import between parts in Python until the Python half is built |

**The async invariants have no gate.** Only a run shows which loop and which task hold a value.
The design of fixtures reduces the [risk](../../LANDSCAPE.md#the-risks) that an async fixture leaves its loop or its task.

**Downside:**

- **A gate that needs a part's design waits for that design.** If the design does not meet the condition in the table, the invariant stays with the reviewer. The pull request that adds that design rewrites the row.
- **The table names a miss for each gate, so the reviewer still checks each invariant.**
- **No gate refuses a breach of an async invariant.** A test of the design of fixtures sees one breach, and not the class.

## Confirmation

Six gates refuse a breach of an invariant, one for each row of the table below.
Each pull request that builds a gate adds its row to the table.
It adds a probe to the list below only when no test holds that probe.

| invariant | command | passed on | the test that holds its probes |
|---|---|---|---|
| one parser | `mise run lint:rust` | the pull request that added `purba-parsing` | none, so the list below holds them |
| an outcome is data | `mise run test:rust` | the pull request that added `purba-execution` | `an_outcome_is_data`, in `crates/execution/tests/run.rs`. The list below holds one probe |
| an outcome is final | `mise run test:rust` | the pull request that added `purba-execution` | the two `compile_fail` doc tests on `Outcome`, in `crates/execution/src/lib.rs` |
| one identity | `mise run test:rust` | the pull request that added `purba-selection` | the three `compile_fail` doc tests on `TestId`, in `crates/selection/src/lib.rs` |
| lazy work | `mise run test:wheel` | the pull request that added `_main` | `tests/skeleton.bats`, which runs `test_a.py` and refuses an import of `test_b.py` |
| the diagram is the only wiring | `mise run lint:wiring` | the pull request that added `purba-selection` | `scripts/test/check-wiring.bats`, with one case for each shape it refuses |

The probes that no test holds:

- `purba` calls `parse_module`, and `mise run lint:rust` exits 101: "use of a disallowed method `ruff_python_parser::parse_module`".
- `purba-parsing` calls `parse_unchecked_source` with no `#[expect]`, and `mise run lint:rust` exits 101: "use of a disallowed method `ruff_python_parser::parse_unchecked_source`".
- `purba-parsing` names its own `[lints]` table and calls `parse_unchecked_source` with no `#[expect]`. `mise run lint:rust` exits 101: "warnings are denied by `build.warnings` configuration".
- `Outcome` gains a field `error: Option<Py<PyAny>>`, and `run` sets it to `None`. The file imports `Py` and `PyAny`. `mise run test:rust` exits 101: "the trait bound `Py<PyAny>: serde::Serialize` is not satisfied".

Clippy kept its old result after `clippy.toml` changed, until a source file changed.
