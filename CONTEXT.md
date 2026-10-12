# purba

purba is a test framework for Python, written in Rust. It is a standing scaffold today and carries no product code.

This file is the source of truth for what a word means here. A decision record states what purba decided and links to this file for the words it uses.

## Language

### Acceptance

The statement that purba has taken a commit into `main`. A workflow derives it from the code owner's approval, so it cannot disagree with the approval it reports.

### Actor

A role that reads purba. It names nobody, and one entity occupies several.

### Architect

The [actor](#actor) that chooses purba's direction and is answerable for what that direction costs.

### Artifact

Anything purba writes for a [reader](#reader), and never a file that a workflow uploads from a job. It is rewritten in place until its direction is agreed, and frozen after.

### Candidate

A third-party Python project, pinned to a commit, whose tests decide when a [milestone](#milestone) is done.

### Coder

The [actor](#actor) that implements a settled decision in code.

### Commit message

The [location](#location) that carries what must outlive the review that produced it. Its subject is the changelog entry.

### Concern

A rule or mechanism that two or more [parts](#part) follow, and that no part owns. Like a part, each concern is designed in depth on its own.

### Debrief

A comment on a merged [pull request](#pull-request) that says where the change differs from its [implementation plan](#implementation-plan). It is owed only where the two differ.

### Design spec

A comment on an [issue](#issue) that states what a change must do and what was rejected. An issue is specified once its type is set, which declares that the person who decides agrees to the spec.

### Desired-state suite

A [candidate's](#candidate) tests, restated for purba before purba can run them. The person approves it before the build starts.

### Gate

A check that refuses a merge. A rule nothing refuses is a convention rather than a gate.

### Handler

The [actor](#actor) that writes the instructions an agent acts on.

### Implementation plan

What the author of a [pull request](#pull-request) writes before the implementation, to say how the [design spec](#design-spec) is built. The [reviewer](#reviewer) reads it.

### Integrator

The [actor](#actor) that wires purba's machine-readable output into another system.

### Invariant

A rule that two or more [parts](#part) follow, written as an absence. Unlike a [concern](#concern), it needs no design of its own.

### Issue

A unit of work, holding a question to settle or a task to do.

_Avoid_: ticket

### Landscape

purba seen whole, in one file, at the level of its parts. It states purba's purpose, surroundings, division, [invariants](#invariant), cross-cutting [concerns](#concern) and [risks](#risk), and the order in which each part and concern is designed in depth.

### Liability

A statement about who is answerable for a change. Each one is written from the act that makes it true.

### Library author

The [actor](#actor) that builds a library against purba's public interface, which a test imports by name.

### Location

A place purba writes into. A [commit message](#commit-message), an [issue](#issue) and `docs/decisions/` are three of them.

### Maintainer

A person who holds write access to this repository. It is not an [actor](#actor), because it names who somebody is rather than what they read.

### Milestone

A set of [issues](#issue) that a pinned [candidate](#candidate) decides is done. GitHub's milestone object carries it, so purba never uses a milestone for a release.

### Origin

The statement that a contributor has the right to submit a contribution. Only a person can make it, so no workflow writes one.

### Outside contribution

Work offered from a fork. It is applied to a branch in this repository rather than merged from the fork.

### Packager

The [actor](#actor) that builds purba's distributions and ships them into environments it does not control.

### Part

A division of purba with one responsibility and one interface. Each part is designed in depth on its own, and its code may be Rust, Python or both.

### Post pass

A read of a branch, after the implementation, for what can be removed or improved, and for each breach of a [record](#record) whose [reach](#reach) the change touches. It leaves its findings as a comment on the [pull request](#pull-request).

### Prose

The sentences of a [record](#record) outside its headings, its [reach](#reach), tables, fenced blocks and quotations.

### Published comment

A comment that purba renders to a [reader](#reader) outside the tree, through a documentation site, a type stub or a language's own help.

### Pull request

The [location](#location) that holds the discussion of how a change was implemented. That discussion stays here and never reaches the [commit message](#commit-message), because the review consumes it.

### Reach

The [locations](#location) a [record](#record) applies to. It names every location where a change makes the record apply, and a location where the change goes to a person counts.

### Reader

An [actor](#actor) that a [location](#location) serves. A location inherits its readers from the path tree.

### Record

A Markdown file under `docs/decisions/`, titled with a proposition, that reads as current state. It is replaced rather than annotated.

### Refusal

The [location](#location) a check writes into when it exits 1 on a contributor's work. What a check writes when it cannot run is not a refusal.

### Register

How information is distributed within one [location](#location). A register belongs to one location and is never compared across two.

### Repair

A change that restores what a [record](#record), a [design spec](#design-spec) or a [gate](#gate) already requires. It carries no decision.

### Reviewer

The [actor](#actor) that reads and approves work it did not write.

### Risk

A failure scenario a test can check. It lands on one [part](#part) or [concern](#concern), and is weighed by how likely it is and by the quality it breaks.

### Significance

The property that routes a change to the code owner. A change is architecturally significant if and only if it adds or changes a [record](#record).

### Stem

The [actor](#actor) that has arrived at a part of purba it does not know yet. An actor returns to the stem at every part it does not know.

### Strict check

A rule that judges one of a [tester's](#tester) tests and gives a verdict of violation, clean or unknown. A [gate](#gate) judges a contributor's change instead.

### Technical writer

The [actor](#actor) that writes purba's documentation for a [reader](#reader) outside the work.

### Tester

The [actor](#actor) that runs purba against its own code.

### Toolsmith

The [actor](#actor) that builds and configures purba's tooling.

### Triage

What an [issue](#issue) gets when it is filed: one label that says how it is resolved, and the issues that block it.

### Unpublished comment

A comment that stays in the tree, read only by somebody who opens the file. It is written only when the information belongs in no other [location](#location).
