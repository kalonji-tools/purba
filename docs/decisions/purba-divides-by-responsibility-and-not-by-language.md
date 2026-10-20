# purba divides by responsibility and not by language

## Context and Problem Statement

The landscape names purba's parts.
Each part is designed in depth on its own.
The division decides which part waits on another.
oxitest divided by language, into Rust modules and one Python package with a bridge between them.

| fact | source |
|---|---|
| oxitest's `bridge.rs` drew 17.8 `fix` commits per 1,000 production lines | [How did oxitest divide into parts, and what did each part cost?](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730) |
| two oxitest files held 89% of its 202 calls from Rust back into Python | [How did oxitest divide into parts, and what did each part cost?](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730) |
| no oxitest code that builds a fixture ever lived in Rust | [How did oxitest divide into parts, and what did each part cost?](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730) |
| oxitest spread its worker protocol over six modules, where `worker_session` drew 30 and `parallel` 26.8 `fix` commits per 1,000 production lines | [How did oxitest divide into parts, and what did each part cost?](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730) |
| pytest, cargo-nextest and unittest share eight parts, and none of them gives parallel execution a part of its own | [Which parts and cross-cutting concerns do comparable test runners have?](https://github.com/kalonji-tools/purba/issues/465#issuecomment-6100161159) |
| cargo-nextest spreads process-per-test over five places | [Which parts and cross-cutting concerns do comparable test runners have?](https://github.com/kalonji-tools/purba/issues/465#issuecomment-6100161159) |
| the query language and static prescan are one design | [Decide the distilled feature set](https://github.com/kalonji-tools/purba/issues/10#issuecomment-5593350290) |
| one module owns parsing, and every other module takes its AST | [purba parses Python with ruff's parser](purba-parses-python-with-ruff.md#decision-outcome) |

## Considered Options

- **Divide by language, into a Rust core and a Python package with a bridge between them.** Rejected. The bridge becomes a dependency of every part. oxitest's bridge drew the cost that the Context shows.
- **Divide into the parts that the three comparable runners share.** Rejected. None of them gives parallel execution a part of its own. Each makes the result model a part, where purba makes it an interface.
- **Divide by responsibility, each part with one interface.** Chosen. A part owns what it decides, whichever language holds its code.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `/CONTEXT.md` `crates/**`

**purba divides into parts by responsibility, and not by language.**
Each [part](../../CONTEXT.md#part) may hold Rust, Python or both.
[The landscape](../../LANDSCAPE.md#the-parts-and-their-dependencies) names each part and draws which part depends on which.

| part | why it is a part of its own |
|---|---|
| dispatch | the worker protocol needs one owner, and the timeout of a test is designed inside it |
| selection | discovery and the query are one design, and it owns what a selected test is |
| parsing | selection and assertion both depend on its AST |
| assertion | it hooks CPython's import, and no other part does |
| execution | it owns the outcome that reporting reads, so the text and the data come from one outcome |

**What is not a part:**

- **The Rust/Python seam, strictness, configuration and interrupt handling.** Each one is a [concern](../../CONTEXT.md#concern).
- **Async.** It is a property of execution. The async invariants tie it to fixtures. [An invariant needs no design of its own](an-invariant-needs-no-design-of-its-own.md#decision-outcome) names them.

**What stays off the diagram:**

- **The `inspect` TUI and doctest.** [Decide the distilled feature set](https://github.com/kalonji-tools/purba/issues/10#issuecomment-5593350290) refused them.
- **A plugin API.** [purba's public surface starts empty and has no plugin API](purbas-public-surface-starts-empty-and-has-no-plugin-api.md#decision-outcome) refuses it.
- **The cache, affected tests, retry, the debugger and coverage.** No decision admits them yet.
- **Built-in fixtures and JUnit output.** Built-in fixtures belong inside fixtures, and JUnit output belongs inside reporting.

**Downside:**

- **The cost of the seam spreads over several parts.** Each part whose job needs a live Python object crosses the seam, and no single part holds that cost.
- **A test that hangs has no owner until dispatch is designed.** A hung test threatens an honest outcome.
- **The two research findings disagree about parallel execution.** This record follows oxitest's cost. cargo-nextest, the one comparable runner that runs tests in parallel, keeps it inside its executor.

## Confirmation

No check judges whether the parts divide by responsibility, because [a standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md).
The reviewable-core scenario in [the landscape](../../LANDSCAPE.md#the-driver-scenarios) checks the division once code exists.

[The landscape](../../LANDSCAPE.md#the-cross-cutting-concerns) names the parts each concern crosses.
[A concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) holds why each one is a concern.
[The landscape](../../LANDSCAPE.md#the-order-of-design) ranks these parts.
[A design in depth waits for what it assumes](a-design-in-depth-waits-for-what-it-assumes.md#decision-outcome) holds why.
