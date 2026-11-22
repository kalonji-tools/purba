# Each fact about a run is found once, by the part that sees it, and crosses as plain data

## Context and Problem Statement

More than one part finds some facts about a run.
A test's process writes part of a case's line before purba's process applies each suppression.
Two copies of one fact can disagree.

| fact | source |
|---|---|
| selection walks a module for its tests in three places | [`found`](https://github.com/kalonji-tools/purba/blob/0029170f/crates/selection/src/lib.rs#L649), and the calls of `methods_of` at [L707](https://github.com/kalonji-tools/purba/blob/0029170f/crates/selection/src/lib.rs#L707) and [L829](https://github.com/kalonji-tools/purba/blob/0029170f/crates/selection/src/lib.rs#L829) |
| assertion finds the same tests again, by their names | [`tests_of`](https://github.com/kalonji-tools/purba/blob/0029170f/crates/assertion/src/lib.rs#L264) |
| two copies of `qualified` give the dotted name of an expression, and only one gives a builtin a name | [patch](https://github.com/kalonji-tools/purba/blob/0029170f/crates/selection/src/patch.rs#L80-L103), [framework](https://github.com/kalonji-tools/purba/blob/0029170f/crates/selection/src/framework.rs#L111-L133) |
| a test's process and a replay start by different steps, and only the test's process sends purba each interrupt that it keeps | [`live_as_test`](https://github.com/kalonji-tools/purba/blob/0029170f/crates/execution/src/zygote.rs#L985-L1030), [`live_as_replay`](https://github.com/kalonji-tools/purba/blob/0029170f/crates/execution/src/zygote.rs#L1077-L1113) |
| the test's process writes the text of each fault of its paths before any suppression, and sends a flag that `order.rs` reads to add the words about a `try` of one path | [the call of `path_faults`](https://github.com/kalonji-tools/purba/blob/0029170f/crates/execution/src/lib.rs#L475), [`path_faults`](https://github.com/kalonji-tools/purba/blob/0029170f/crates/execution/src/lib.rs#L610-L657), [`order.rs`](https://github.com/kalonji-tools/purba/blob/0029170f/crates/execution/src/order.rs#L204-L216) |
| "only facts cross a part, as plain data" stands as a note inside a figure | [command lends one parse, L54](https://github.com/kalonji-tools/purba/blob/0029170f/docs/decisions/command-lends-one-parse-to-selection-and-assertion.md?plain=1#L54) |
| purba decides the outcome of each case after its last replay, and the test's process reads no suppression | [each test runs in a process of its own, L42](https://github.com/kalonji-tools/purba/blob/0029170f/docs/decisions/each-test-runs-in-a-process-of-its-own.md?plain=1#L42) |

## Considered Options

- **One rule for the parse of a module and one for the children of the zygote, with no rule that they share.** No record refuses the next fact that a part finds a second time.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**Area:** design

**Each fact about a run is found once, by the part that sees it, and crosses to another part or process as plain data.**
A fact with one finder has no second copy that can disagree with it.

- **Before the run, parsing finds the facts of each module once, and selection and assertion read them.** Neither part walks a tree for a fact that parsing finds.
- **Parsing finds the facts of a module when it parses the module, and the facts of a test when a part first asks for them.** The facts of a module are its tests, its imports and the names that it binds. So parsing finds no fact of a test in a module that selection refuses. [Command lends one parse](command-lends-one-parse-to-selection-and-assertion.md#considered-options) rejects a walk that reads each module before selection decides.
- **Parsing holds one walk that does not overflow the stack, and each part runs it with only the nodes that it matches.**
- **During the run, each child of the zygote gathers facts, and purba's process alone decides the outcome of each case from them.** Each child starts by the same steps, and sends each message through its pipe. A replay sends its verdict that way too.
- **purba's process applies each suppression, then writes each fault of a case's paths, and decides the words about a `try` of one path.** A child writes no fault.

[Command lends one parse to selection and assertion](command-lends-one-parse-to-selection-and-assertion.md#decision-outcome) holds how command lends the tree to each part.
Until the code follows this record, [purba parses Python with ruff](purba-parses-python-with-ruff.md#decision-outcome) holds the seam over ruff.

**Downside:**

- **Parsing holds the facts of each module beside its thin seam over ruff.** A change to a fact that selection or assertion wants edits parsing.
- **The facts of a module borrow its tree.** They drop before command drops the tree.
- **When assertion rewrites a source that selection did not read, the rewrite finds its own facts.** [`Rewrite::of_source`](https://github.com/kalonji-tools/purba/blob/0029170f/crates/assertion/src/lib.rs#L218-L223) parses a source without selection.
- **A replay sends its verdict through its pipe, and its exit code no longer carries the verdict.** purba reads a replay through the reader of a test's process.
- **The facts of a case carry each fault of its paths, each `with` that swallowed a raise, and each `try` of its test.** The test's process sends each of them to purba's process.
- **A process that the test's process forks to read one expectation alone still ends with what it read as its exit code.** The zygote does not fork it, and it sends no message through a pipe ([`zygote.rs` L1113-L1118](https://github.com/kalonji-tools/purba/blob/0029170f/crates/execution/src/zygote.rs#L1113-L1118), [L1146-L1153](https://github.com/kalonji-tools/purba/blob/0029170f/crates/execution/src/zygote.rs#L1146-L1153)). purba never sees a SIGINT that reaches only that process while purba's mask holds the signal, and the run goes on ([the probe](https://github.com/kalonji-tools/purba/issues/1171#issuecomment-7077713723)).

## Confirmation

No command refuses a part that finds a fact a second time, so a reviewer judges it.
The code does not follow this record yet.
The tasks of [Wayfinder: each fact is found once, by the part that sees it](https://github.com/kalonji-tools/purba/issues/1174) change the code to follow it.
