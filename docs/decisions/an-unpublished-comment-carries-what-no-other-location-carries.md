# An unpublished comment carries what no other location carries

## Context and Problem Statement

Five locations record why a change is the way it is, and they repeat each other.

| location | what states its register |
|---|---|
| commit message | [a commit message outlives its review](a-commit-outlives-its-review.md) |
| decision record | [a decision record is rewritten, not amended](a-record-is-rewritten-not-amended.md) |
| pull request | [a register belongs to one location](a-register-belongs-to-one-location.md), until `.github/PULL_REQUEST_TEMPLATE.md` is written |
| published comment | the language's convention for voice, and [a register belongs to one location](a-register-belongs-to-one-location.md) for which artifact earns one |
| unpublished comment | nothing, and this record ends that |

The fifth is the only one with nothing saying what belongs there, so a writer with something to say puts it in all five.

Volume is not the defect.

| corpus | non-blank | unpublished comment | share |
|---|---:|---:|---:|
| prototype Rust and Python production | 71,382 | 3,959 | 5.5% |
| this repository | 343 | 176 | 51.3% |
| this repository, `src/lib.rs` | 17 | 0 | 0% |

Every unpublished comment in this repository sits in configuration.

Duplication is the defect.
`Cargo.toml` holds nine comment blocks over twenty-seven lines.

| block | lines | restates |
|---|---:|---|
| `crate-type` | 6 | [the crate carries an rlib](the-crate-carries-an-rlib.md) |
| the `ruff` pins | 4 | [purba parses Python with ruff's parser](purba-parses-python-with-ruff.md) |
| `extension-module` | 3 | [the crate carries an rlib](the-crate-carries-an-rlib.md) |

Each of the three cites the closed issue that produced the record, so the pointer reaches what happened rather than what is true.

## Considered Options

- **Locality: a comment explains the line it sits on.** Rejected. All three duplicated blocks pass it, because each is local to the line it sits on.
- **A comment-to-code ratio.** Rejected. [A register belongs to one location](a-register-belongs-to-one-location.md) refused it by name. Removing the duplicated lines improves `Cargo.toml` and raises every ratio computable over it.
- **A separate register for configuration.** Rejected. The split between configuration and code counts violations rather than a difference in kind, and the admission test below names configuration.
- **Trap-only: a comment exists to stop a wrong edit.** Rejected as the whole rule. Whether an edit is plausible is a judgement. It survives inside the admission test.
- **Subtraction: a comment carries what the other locations do not.** Chosen. It removes the duplicated lines by construction.

## Decision Outcome

An unpublished comment is written only when the information belongs in no other location.

**The order of claim.**
The pull request, then the commit and the issue its subject names, then the decision record, then the published comment.
What remains may be an unpublished comment.
Each of those four states in its own register what it takes.
This record restates none of them.

**Information on the issue is already in the commit, so it is not written again.**
A decision record subtracts more strongly, because it sits in the same tree and states what is true.

**Admission.**
A comment is written when complex configuration, logic or an algorithm needs elaboration that no other location holds.

**A link replaces the elaboration wherever a primary source exists.**
The research paper, the published algorithm, the upstream reference, the framework and the design pattern each stay correct without maintenance.

**A comment cites that source and never an issue number.**
The issue number belongs to the commit subject.

**Fewer unpublished comments is a goal this project works towards.**
It is not a threshold, and no check counts them.

**This record is the exception to [a register belongs to one location](a-register-belongs-to-one-location.md).**
That record states that nothing about an unpublished comment is gated.
Link liveness is decidable, so that one property is gated and the rest is convention.

**Downside:**

- **The rule that decides is unenforceable.** Whether information belongs in another location is a judgement, so nothing gates the sentence this record exists to state.
- **A reader who reaches the general record first is told this location is never gated.**
- **The recommended citation is the form most likely to fail the gate.** A digital object identifier resolves to a publisher, and a publisher blocks the checker. The accepted codes below absorb it, and a resource withdrawn behind a hard block then passes.
- **A checked link is not a read link.** The gate proves a page answers, never that it still says what the comment claims.
- **The gate depends on a network.** A change is refused for the state of somebody else's server, which is why only the changed-file leg blocks.

## Confirmation

**`lychee` over tracked files, blocking on the changed ones and reporting on the rest.**

| setting | value |
|---|---|
| inputs | tracked files, from `git ls-files` |
| extensions | `md`, `toml`, `rs`, `yml`, `yaml`, `py`, `nix` |
| excluded | `*.lock` |
| accepted | `200..=299`, `403`, `429` |
| timeouts | a failure |
| fragments | off on the blocking leg, on for the scheduled leg |

A pull request is refused for a dead link in a file it changed.
A scheduled run over the whole tree reports rot and refuses nothing.

The checker opens markdown and HTML only, so a comment in a source file is never read unless the extensions are named.
It extracts a bare address, a titled link and an autolink alike, and it resolves a relative link between records to a path it then checks.
That resolution is the `cited paths exist` row graded weak in [a decision record is rewritten, not amended](a-record-is-rewritten-not-amended.md).

**Blind spots.**

| blind spot | effect |
|---|---|
| reserved example domains are excluded by default | a placeholder address is not a checked address |
| a redirect is followed and reported valid | an address that has moved permanently is never corrected |
| `403` and `429` are accepted | a withdrawn page passes |

The gate is not wired.
An extension list cannot name a tracked file that has no extension, and `CODEOWNERS` is one.
So the input set above cannot reach every tracked file that holds an address.
Naming further extensions does not repair it, because the gap is the absence of an extension.
[Which tools does mise.toml name, and how does it record a tool that is refused?](https://github.com/kalonji-tools/purba/issues/113) owns whether a tool file names this checker.
Its roster records the checker as living in this record alone.

Nothing decides whether a comment that survives the subtraction was worth writing.
That fails as friction.
