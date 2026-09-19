# purba

purba is a test framework for Python, written in Rust. It is a standing scaffold today and carries no product code.

This file is the source of truth for what a word means here. A decision record states what purba decided and links to this file for the words it uses.

## Language

### Acceptance

The statement that purba has taken a commit into `main`. A workflow derives it from the code owner's approval, so it cannot disagree with the approval it reports.

### Actor

A role that reads purba, stated as the mindset it reads from. It says nothing about who occupies it, and one entity occupies several.

### Architect

The [actor](#actor) that must choose a direction and be answerable for what it costs.

### Artifact

Anything purba writes for a [reader](#reader), and never a file that a workflow uploads from a job. It is rewritten in place until its direction is agreed, and frozen after.

### Coder

The [actor](#actor) for which the direction is settled and the work is to build it correctly.

### Commit message

The [location](#location) that carries what must outlive the review that produced it. Its subject is the changelog entry.

### Gate

A check that refuses a merge. A rule nothing refuses is a convention rather than a gate.

### Handler

The [actor](#actor) that writes for an agent, which will act on what it wrote and on nothing else.

### Integrator

The [actor](#actor) that needs purba to tell a machine what happened, the same way every time.

### Issue

A unit of work, holding a question to settle or a task to do.

_Avoid_: ticket

### Liability

A statement about who is answerable for a change. Each one is written from the act that makes it true.

### Location

A place purba writes into. A [commit message](#commit-message), an [issue](#issue) and `docs/decisions/` are three of them.

### Maintainer

A person who holds write access to this repository. It is not an [actor](#actor), because it names who somebody is rather than what they read.

### Origin

The statement that a contributor has the right to submit a contribution. Only a person can make it, so no workflow writes one.

### Outside contribution

Work offered from a fork. It is applied to a branch in this repository rather than merged from the fork.

### Packager

The [actor](#actor) that must get purba into an environment it does not control, and prove it is allowed to ship it.

### Plugin author

The [actor](#actor) that builds on top of purba and needs the interfaces it depends on to keep their contract across releases.

### Published comment

A comment that purba renders to a [reader](#reader) outside the tree, through a documentation site, a type stub or a language's own help.

### Pull request

The [location](#location) that holds the discussion of how a change was implemented. That discussion stays here and never reaches the [commit message](#commit-message), because the review consumes it.

### Reader

An [actor](#actor) that a [location](#location) serves. A location inherits its readers from the path tree.

### Record

A Markdown file under `docs/decisions/`, titled with a proposition, that reads as current state. It is replaced rather than annotated.

### Register

How information is distributed within one [location](#location). A register belongs to one location and is never compared across two.

### Reviewer

The [actor](#actor) that is about to become answerable for work it did not write.

### Significance

The property that routes a change to the code owner. A change is architecturally significant if and only if it adds or changes a [record](#record).

### Stem

The [actor](#actor) that does not know the part of purba it has arrived at, and needs footing before it can act. An actor returns to the stem whenever it meets a part it does not know.

### Technical writer

The [actor](#actor) that writes for somebody outside the work, who must understand it without asking.

### Tester

The [actor](#actor) that needs purba to tell the truth about its own code.

### Toolsmith

The [actor](#actor) for which a mistake must be stopped before a person has to catch it.

### Unpublished comment

A comment that stays in the tree, read only by somebody who opens the file. It is written only when the information belongs in no other [location](#location).
