# An invariant needs no design of its own

## Context and Problem Statement

The landscape names the invariants between purba's parts.
Each part is designed in depth on its own.
Each part must still keep the rules it shares with its neighbours.
One design in depth can break a rule that no file states.
No reader sees the breach.
The landscape needs a test that keeps out what a concern already states.

| fact | source |
|---|---|
| lazy work has no design of its own, so it is an invariant | [a concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) |
| dispatch and reporting pass data, so neither crosses the Rust/Python seam | [a concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) |
| the timeout of a test belongs inside dispatch, and selection owns what a selected test is | [purba divides by responsibility and not by language](purba-divides-by-responsibility-and-not-by-language.md#decision-outcome) |
| one module owns parsing, and everything else takes its AST | [purba parses Python with ruff's parser](purba-parses-python-with-ruff.md#decision-outcome) |
| oxitest ran two parsers, and its assert rewriter built CPython AST nodes from Rust | [How did oxitest divide into parts, and what did each part cost?](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730) |
| oxitest's assert rewriter made 56 of oxitest's 202 calls from Rust into Python | [How did oxitest divide into parts, and what did each part cost?](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730) |
| oxitest's reporter held Python plugin objects, and called them on every test | [How did oxitest divide into parts, and what did each part cost?](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730) |
| oxitest found that `asyncio.run()` for each fixture call finalizes the fixture before the test body runs, so its teardown never runs | [Async organizational strategy for oxitest](https://github.com/kalonji-tools/oxitest/blob/731c2b3f76c708125eb4984d9725652208f5ae81/docs/adr/0006-async-organizational-strategy.md?plain=1#L14) |
| oxitest moved a test that uses an async fixture that serves many tests onto that fixture's loop, "because a value cannot move between loops" | [oxitest's fixture system redesign](https://github.com/kalonji-tools/oxitest/blob/731c2b3f76c708125eb4984d9725652208f5ae81/docs/adr/0009-fixture-system-redesign.md?plain=1#L383) |
| oxitest clamped a teardown "so it can never be scheduled after its loop closes" | [oxitest's fixture system redesign](https://github.com/kalonji-tools/oxitest/blob/731c2b3f76c708125eb4984d9725652208f5ae81/docs/adr/0009-fixture-system-redesign.md?plain=1#L383) |
| oxitest found that "one loop is necessary and not sufficient", because a fixture's setup, the test and its teardown must run in one task | [Async organizational strategy for oxitest, its third amendment](https://github.com/kalonji-tools/oxitest/blob/731c2b3f76c708125eb4984d9725652208f5ae81/docs/adr/0006-async-organizational-strategy.md?plain=1#L176-L184) |
| oxitest exempted a fixture that serves many tests from that rule, because no task can span them | [Async organizational strategy for oxitest, its third amendment](https://github.com/kalonji-tools/oxitest/blob/731c2b3f76c708125eb4984d9725652208f5ae81/docs/adr/0006-async-organizational-strategy.md?plain=1#L198) |
| lazy work is a mechanism of fast feedback, and not a quality | [purba ranks an honest outcome first](purba-ranks-an-honest-outcome-first.md#decision-outcome) |
| the fast-feedback scenario counts imports, and the honest-outcome scenario compares outcomes by test name | [the landscape](../../LANDSCAPE.md#the-driver-scenarios) |

## Considered Options

- **An invariant is any rule that two or more parts follow.** Rejected. The rules of strictness and configuration would join the list, though each concern already states its own rule.
- **An invariant is a rule that two or more parts follow, written as an absence, that needs no design of its own.** Chosen. One instance breaks an absence, so a reviewer can point at the breach.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `/CONTEXT.md`

**An [invariant](../../CONTEXT.md#invariant) needs no design of its own.**
[The landscape](../../LANDSCAPE.md#the-invariants) names each invariant and the parts it binds.

| invariant | why it holds |
|---|---|
| lazy work | the fast-feedback scenario counts the imports outside the selection |
| an outcome is data | the seam stays in execution, fixtures and assertion, and a worker can be another process |
| an outcome is final | no outcome depends on the worker that ran the test, or on a later part |
| one identity | the honest-outcome scenario compares outcomes by test name, so a second name can hide a test |
| one parser | oxitest's second parser drew a large share of its calls into Python |
| an async fixture stays on its loop | a value built on one loop fails on another, and a teardown after its loop closes never runs |
| an async fixture shares its test's task | a value that a fixture sets in its own task does not reach the test |
| the diagram is the only wiring | a change inside one part can be reviewed without reading another part |

**What is not an invariant:**

- **A rule that a concern states.** Configuration states that no value turns a check off.
  Strictness states that nothing unchecked shows as passed.
- **Only execution makes an outcome.** Dispatch makes the outcome of a test whose worker stopped, because the timeout of a test belongs to dispatch.
- **How the async invariants hold.** Which loop runs which test belongs to the design of execution.

**Downside:**

- **Execution sets the detail of a report before it returns the outcome.** Reporting cannot ask a live object for more, so execution renders each value in the test's own process.
- **A test that uses an async fixture that serves many tests must run on that fixture's loop.** Tests that share a loop can share what each one leaves on it. The design of execution carries that cost.
- **A new dependency between parts needs a landscape change first.** The diagram changes before the code does.

## Confirmation

No check refuses a breach yet.
[A gate owns the mechanical standard](a-gate-owns-the-mechanical.md) applies to the invariants that the landscape names.
[A gate refuses each invariant a tool can see, from the first code it reads](a-gate-refuses-each-invariant-a-tool-can-see.md#decision-outcome) says which of them a gate refuses, and when.
The walking skeleton is the first to run code that each invariant binds.
