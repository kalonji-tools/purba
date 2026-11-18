# A concern's shared names may live in a part

## Context and Problem Statement

[A concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) says that no part owns a concern.
It does not say which part keeps the names that a concern's parts share.
The parts of strictness wrote the names of the strict checks in several places.
Nothing tied those places together.

| fact | source |
|---|---|
| the record of concerns merged before the first commit under `crates/` | [`cf4a787c`](https://github.com/kalonji-tools/purba/commit/cf4a787c) on 2026-10-12; `git log --reverse --format=%h 6a8ff676 -- crates` prints [`8a79a02c`](https://github.com/kalonji-tools/purba/commit/8a79a02c), of 2026-10-14, first |
| execution holds `Error` and `ErrorKind`, the names of the errors concern | [`crates/execution/src/lib.rs:132`](https://github.com/kalonji-tools/purba/blob/6a8ff676856dc02786ac626207cef4cdccb86192/crates/execution/src/lib.rs#L132), [`:143`](https://github.com/kalonji-tools/purba/blob/6a8ff676856dc02786ac626207cef4cdccb86192/crates/execution/src/lib.rs#L143) |
| command and dispatch use `Error`, and reporting also uses `ErrorKind` | [`crates/command/src/lib.rs:123`](https://github.com/kalonji-tools/purba/blob/6a8ff676856dc02786ac626207cef4cdccb86192/crates/command/src/lib.rs#L123), [`crates/dispatch/src/lib.rs:9`](https://github.com/kalonji-tools/purba/blob/6a8ff676856dc02786ac626207cef4cdccb86192/crates/dispatch/src/lib.rs#L9), [`crates/reporting/src/lib.rs:11`](https://github.com/kalonji-tools/purba/blob/6a8ff676856dc02786ac626207cef4cdccb86192/crates/reporting/src/lib.rs#L11) |
| selection and execution each wrote the four names of the checks that execution sees | [`crates/selection/src/suppression.rs:28`](https://github.com/kalonji-tools/purba/blob/6a8ff676856dc02786ac626207cef4cdccb86192/crates/selection/src/suppression.rs#L28), [`crates/execution/src/lib.rs:614`](https://github.com/kalonji-tools/purba/blob/6a8ff676856dc02786ac626207cef4cdccb86192/crates/execution/src/lib.rs#L614) |
| no test named either list | `git grep -n -e SUPPRESSIBLE -e OF_EXECUTION 6a8ff676` prints four lines, each in `crates/*/src/` |
| execution cannot reach selection, and dispatch reaches both | [the parts diagram of each part is a crate](https://github.com/kalonji-tools/purba/blob/6a8ff676856dc02786ac626207cef4cdccb86192/docs/decisions/each-part-is-a-crate.md?plain=1#L56-L59) |

## Considered Options

- **Each part writes the names it needs, and nothing ties them.** A name added to one list and not to the other goes unseen. No test compares the lists.
- **A part of their own holds the names of the strict checks.** It checks each name at compile time. It costs new arrows and a new part in the order of design, as [the issue of this record](https://github.com/kalonji-tools/purba/issues/1090) says under Out of scope.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**Area:** design, strictness

**A concern's shared names may live in one part.**
The part keeps the names.
The concern's duty stays with each part that follows it.

- **A part that cannot reach the names keeps a copy.** A test in a part that reaches the copy and the original fails when the two differ.
- **The part that holds the names does not own the concern.** Each part still refuses what it sees.

| concern | the names | the part that holds them | a copy, and the test that ties it |
|---|---|---|---|
| errors | `Error`, `ErrorKind` | execution | none |
| strictness | each strict check: its name, its tier, and where its breach shows | selection | execution's `SUPPRESSIBLE`, tied by `crates/dispatch/tests/checks.rs` |

**Downside:**

- **A copy can drift where a test does not read it.** The test of dispatch reads `SUPPRESSIBLE` alone. Assertion and execution write the name of each fault that they see as a literal.
- **A reader can take the part that holds the names for the owner of the concern.** This record and the record of concerns link each other to say otherwise.

## Confirmation

`mise run test:rust` runs `crates/dispatch/tests/checks.rs`, which fails when execution's `SUPPRESSIBLE` and selection's table differ.

[Selection keeps its strict checks in one table, resolves each name in one place, and keeps the order of its suppressions](https://github.com/kalonji-tools/purba/issues/1090)
