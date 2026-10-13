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
| the commit that first names a ruff crate carries the declaration and its check together | [purba parses Python with ruff](purba-parses-python-with-ruff.md#confirmation) |
| clippy's `disallowed-methods` refused a call to `ruff_python_parser::parse_module` outside the parsing module, which carried the one `#[expect]` | [Which of purba's invariants does a gate refuse?](https://github.com/kalonji-tools/purba/issues/512#issuecomment-6127311542) |
| the compiler refused a `Py<PyAny>` inside an outcome that derives `Serialize` and `Deserialize` | [Which of purba's invariants does a gate refuse?](https://github.com/kalonji-tools/purba/issues/512#issuecomment-6127311542) |
| with pyo3's `serde` feature on, a `Py<T>` serialises when `T` is a `#[pyclass]` that serialises | [pyo3 v0.29.2](https://github.com/PyO3/pyo3/blob/v0.29.2/src/conversions/serde.rs#L18-L20) |
| ruff exposes more than one function that parses | [Which of purba's invariants does a gate refuse?](https://github.com/kalonji-tools/purba/issues/512#issuecomment-6127311542) |
| `.config/mise.toml` pinned no Python linter | [`.config/mise.toml`](https://github.com/kalonji-tools/purba/blob/4c57d5049d9782f114243868a3be8774ffb1810b/.config/mise.toml#L9-L64) |
| a private `Cell` field that changes through `&self`, and an `Option<Py<PyAny>>` field that `#[serde(skip)]` leaves out, each compiled in a struct that derives `Serialize` and `Deserialize`, on purba's pins and serde `1.0.229`, with pyo3's `serde` feature off | [Which of purba's invariants does a gate refuse?](https://github.com/kalonji-tools/purba/issues/512#issuecomment-6141818801) |
| `#[allow(C)]` overrides the check for C, so that its violations go unreported, and `#[expect(C)]` suppresses C too | [the Rust Reference](https://doc.rust-lang.org/reference/attributes/diagnostics.html#lint-check-attributes) |

## Considered Options

- **A gate refuses every breach, and only breaches.** Rejected. Every invariant fails that bar, so the reviewer would keep each of them. A tool sees most breaches of each invariant in the table below.
- **A gate refuses only breaches, and names what it misses.** Chosen. For each invariant that a gate refuses, the reviewer keeps what the table names as missed.
- **Every gate at the walking skeleton.** Rejected. The skeleton may hold no second part and no outcome type.
- **Each gate at the design in depth of its part.** Rejected. Code would stand without its gate until that design merges.
- **Each gate in the pull request that adds the first code it reads.** Chosen. The gate runs red on real code, and not on an empty tree.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `/Cargo.toml` `clippy.toml` `src/**` `*.py`

**A gate refuses each invariant that a tool can see, from the first code the gate reads.**

- **Each refusal of a gate is a breach, and a gate may miss a breach.** The table names each miss that a probe or a source shows, and the reviewer checks for others too.
- **The pull request that adds the first code a gate reads also adds the gate.** It runs the gate green and red.
- **The probes of clippy and the compiler show that a gate can decide a breach.** They are evidence, and they are not a design.

| invariant | a gate can refuse it when | the code that first builds it | what it misses |
|---|---|---|---|
| one parser | (no condition: clippy sees it in one crate) | the first ruff crate in `Cargo.toml`, and the first Python file of a part for the Python half | a function that parses and that `disallowed-methods` omits, a call under its own `#[allow]` or `#[expect]`, and Python's `ast.parse` until the Python half is built |
| an outcome is data | the outcome must encode to bytes | the outcome type | a field that `#[serde(skip)]` leaves out, and a `Py<T>` of a `#[pyclass]` that serialises, if pyo3's `serde` feature is on |
| an outcome is final | the outcome has private fields and no method that takes `&mut self` | the outcome type | a field that changes through `&self`, such as a `Cell`, and a part that builds a replacement outcome |
| one identity | only selection constructs a test's identity | the identity type | a part that formats its own name for a test |
| lazy work | selection and execution both exist | the walking skeleton, as a test that runs one of two modules and refuses any import of the other | a module that only a query leaves out, a rewrite that reads the other module and never imports it, and a path that the test does not take |
| the diagram is the only wiring | each part has a boundary that a tool reads | the code of the second part | an import between parts in Python, until the Python half is built |

**The async invariants have no gate.** Only a run shows which loop and which task hold a value.
The design of fixtures reduces the [risk](../../LANDSCAPE.md#the-risks) that an async fixture leaves its loop or its task.

**Downside:**

- **A gate that needs a part's design waits for that design.** If the design does not meet the condition in the table, the invariant stays with the reviewer. The pull request that adds that design rewrites the row.
- **The table names a miss for each gate, so the reviewer still checks each invariant.**
- **No gate refuses a breach of an async invariant.** A test of the design of fixtures sees one breach, and not the class.

## Confirmation

No gate refuses a breach of an invariant yet.
Each pull request that builds a gate adds its green and red runs to this section.
