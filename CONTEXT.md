# purba

purba is a test framework for Python, written in Rust. purba runs the tests of one module, through its [walking skeleton](#walking-skeleton).

This file is the source of truth for what a word means here. A decision record states what purba decided and links to this file for the words it uses.

## Language

### Acceptance

The statement that purba has taken a commit into `main`. A workflow derives it from the approval of a person whose account has write access, so it cannot disagree with the approval it reports.

### Actor

A role that reads purba. It names nobody, and one entity occupies several.

### Alias

A [query](#query) saved under a name in `[tool.purba.queries]`, which a query can name in its place.

### Architect

The [actor](#actor) that chooses purba's direction and is answerable for what that direction costs.

### Area

The kind of work a [record](#record) decides something in: product design, one [part](#part) or [concern](#concern), the pull-request workflow, writing, or repository tooling. A record names each area it decides something in.

### Arrange, act, assert

The layout of a [test](#test): the lines before its first [expectation](#expectation) arrange and act, and its expectations assert.

### Artifact

Anything purba writes for a [reader](#reader), and never a file that a workflow uploads from a job. It is rewritten in place until its direction is agreed, and frozen after.

### Blast area

How far a crate reaches into purba: the [parts](#part) that name it, and the crates it brings with it.

### Blind rating

A score of each [form](#form) of a choice, by raters who do not know which form is new.

### Blocked test

A selected test that an error kept from running, or that gave a [project error](#project-error) as it ran, such as a case with no [expectation](#expectation), so it has no [outcome](#outcome).

### Candidate

A third-party Python project, pinned to a commit, whose tests decide when a [milestone](#milestone) is done.

### Case

One call of a [test](#test), which a [run](#run) gives at most one [outcome](#outcome). A test with no [inputs](#input) is one case, under the test's own name.

### Catalyst

An [input](#input) that outlives one [test](#test), and whose type proves that no test can change it.

### Claim

A sentence that states a fact, which a command or a link can decide. A sentence that judges is not one.

### Coder

The [actor](#actor) that implements a settled decision in code.

### Commit message

The [location](#location) that carries what must outlive the review that produced it. Its subject is the changelog entry.

### Concern

A rule or mechanism that two or more [parts](#part) follow, and that no part owns. Like a part, each concern is designed in depth on its own.

### Configuration level

A place that sets a [dial](#dial): code, the command line, the environment, the project's file, the user's file or the system's file.

### Count

What a [run](#run) met, in numbers: each [case](#case) once, by its [outcome](#outcome) or as a [blocked](#blocked-test) or [unreached test](#unreached-test), and each error once, by its [kind](#kind). Reporting prints it as the last line on stdout, and renders the exit code from it.

### Debrief

A comment on a merged [pull request](#pull-request) that says where the change differs from its [implementation plan](#implementation-plan). It is owed only where the two differ.

### Design spec

A comment on an [issue](#issue) that states what a change must do and what was rejected. An issue is specified once its type is set, which declares that the person who decides agrees to the spec.

### Desired-state suite

A [candidate's](#candidate) tests, restated for purba before purba can run them. The person approves it before the build starts.

### Dial

A value of configuration that changes how purba behaves. The [part](#part) that reads it declares it, and a [configuration level](#configuration-level) sets it.

### Entity

A being that can take the role of an [actor](#actor). A person and an agent are each one.

### Expectation

An assert at the end of a [test's](#test) body, or at the end of the body of a `with` that comes just before such asserts, with only other expectations, or tuple asserts, after it, and that is no tuple assert itself. Selection asks that it compare two values by one operator, with a literal on one side, or take one of four forms: `not x`, `isinstance(x, T)`, `x.startswith(y)` or `x.endswith(y)`, or check each item of an `all()` or an `any()` by one of these. Selection refuses one in none of these. purba reads each one in each [case](#case), and gives the case its [outcome](#outcome) from those readings.

### Form

The text that a [tester](#tester) writes in a [test](#test), or reads in the report of a [run](#run), apart from what it means. Two forms can mean the same thing.

### Gate

A check that refuses a merge. A rule nothing refuses is a convention rather than a gate.

### Handler

The [actor](#actor) that writes the instructions an agent acts on.

### Implementation plan

What the author of a [pull request](#pull-request) writes to say how the change is built. The [reviewer](#reviewer) reads it.

### Input

A value that a [test](#test) or a fixture names by the fixture in the default of a parameter. A fixture that draws a [reagent](#reagent) takes its [stock](#stock) as its one parameter with no default.

### Integrator

The [actor](#actor) that wires purba's machine-readable output into another system.

### Internal error

An error that a [part](#part) sees in purba itself: a panic, or an error that purba never expects. A [tester](#tester) reports it as a bug in purba.

### Interrupt

A signal that asks a [run](#run) to stop: SIGINT, SIGTERM or SIGHUP. It is not an error, and purba charges no [test](#test) or module with a failure.

### Invariant

A rule that two or more [parts](#part) follow, written as an absence. Unlike a [concern](#concern), it needs no design of its own.

### Invocation error

An error that a [part](#part) sees in how a [run](#run) was started or where it runs: an argument, a path the [tester](#tester) typed, data purba cannot take, or a place purba cannot write. The tester fixes it in the command or where it runs.

### Issue

A unit of work, holding a question to settle or a task to do.

_Avoid_: ticket

### Kind

Whether an error is a [project error](#project-error), an [internal error](#internal-error) or an [invocation error](#invocation-error). The [part](#part) that first sees the error names its kind.

### Landscape

purba seen whole, in one file, at the level of its parts. It states purba's purpose, surroundings, division, [invariants](#invariant), cross-cutting [concerns](#concern) and [risks](#risk), and the order in which each part and concern is designed in depth.

### Lane

A set of lints that `Quality` runs when a [pull request](#pull-request) changes a path that one of them reads. The lane `always` runs on every pull request.

### Layer

One of the five places where a test of purba runs: Rust unit tests, the tests under `crates/*/tests`, the `bats` tests of its scripts, the tests inside a cargo script, and the end-to-end tests on the wheel.

### Liability

A statement about who is answerable for a change. Each one is written from the act that makes it true.

### Library author

The [actor](#actor) that builds a library against purba's [public surface](#public-surface), which a test imports by name.

### Location

A place purba writes into. A [commit message](#commit-message), an [issue](#issue) and `docs/decisions/` are three of them.

### Maintainer

A person who holds write access to this repository. It is not an [actor](#actor), because it names who somebody is rather than what they read.

### Milestone

A set of [issues](#issue) that a pinned [candidate](#candidate) decides is done. GitHub's milestone object carries it, so purba never uses a milestone for a release.

### Notice

A line that purba prints under the line of a [case](#case), after the lines of its [outcome](#outcome), that starts with `⚠`, names no [strict check](#strict-check), and refuses nothing. It changes no count and no exit code. A [private name](#private-name) that a [test's](#test) body reads gives one.

### Origin

The statement that a contributor has the right to submit a contribution. Only a person can make it, so no workflow writes one.

### Outcome

What a [run](#run) says about one [case](#case) that it reached and that no error blocked: passed, failed, raised or interrupted. A case in which no [expectation](#expectation) ran has no outcome, and gives a [project error](#project-error).

### Outside contribution

Work offered from a fork. It is applied to a branch in this repository rather than merged from the fork.

### Packager

The [actor](#actor) that builds purba's distributions and ships them into environments it does not control.

### Pain

What a crate costs purba: its [blast area](#blast-area), and the other factors of keeping it.

### Part

A division of purba with one responsibility and one interface. Each part is designed in depth on its own, and its code may be Rust, Python or both.

### Patch

A change that a test makes, itself or through a library, to a value outside the test, such as `lending.Library = Stub` or a call of `unittest.mock`'s `patch`. It puts a [stand-in](#stand-in) where the code under test reaches the real value.

### Post pass

A read of a branch, after the implementation, for what can be removed or improved, and for each breach of a [record](#record) whose [reach](#reach) the change touches. It leaves its findings as a comment on the [pull request](#pull-request).

### Practice tier

The [strict checks](#strict-check) whose violation a [suppression](#suppression) turns into a warning, so that the test runs.

### Private name

A name that starts with `_`, written as an attribute or imported from another module, or a name imported through a part of a module's path that starts with `_`. A name that starts and ends with `__`, such as `__len__`, is not one. purba reports each one that a [test's](#test) body reads, and refuses none.

### Probe

A comment that holds a command, the head it ran on, and its output, so that it decides a [claim](#claim).

### Project error

An error that a [part](#part) sees in the [tester's](#tester) code, other than a test's [outcome](#outcome). A module that does not parse or load, a test purba refuses to call, and an exception that no test's call raised are each one.

### Prose

The sentences of a [record](#record) outside its headings, its [reach](#reach), tables, fenced blocks and quotations.

### Public surface

What a [tester](#tester), a [library author](#library-author) or an [integrator](#integrator) can depend on: the names a test imports, the command's flags and exit code, the data purba writes, and the format of its configuration.

### Published comment

A comment that purba renders to a [reader](#reader) outside the tree, through a documentation site, a type stub or a language's own help.

### Pull request

The [location](#location) that holds the discussion of how a change was implemented. That discussion stays here and never reaches the [commit message](#commit-message), because the review consumes it.

### Query

The text that names the [tests](#test) of a [run](#run).

### Reach

The [locations](#location) a [record](#record) applies to. It names every location where a change makes the record apply, and a location with no path counts.

### Reader

An [actor](#actor) that a [location](#location) serves. A location inherits its readers from the path tree.

### Reading

One read of an [expectation](#expectation) in a [case](#case), and the value that it reads: the side of a comparison that is not the expected value, the `x` of one of the four forms, what the `for` of an `all()` or an `any()` in one of these forms loops over, or else its whole condition. The expected value is the side that is a literal, or the right side when both are.

### Reagent

An [input](#input) that fixtures builds for one [test](#test), and tears down after it.

### Record

A Markdown file under `docs/decisions/`, titled with a proposition, that reads as current state. It is replaced rather than annotated.

### Refusal

The [location](#location) a check writes into when it turns down a contributor's work. What a check writes when it cannot run is not a refusal.

### Register

How information is distributed within one [location](#location). A register belongs to one location and is never compared across two.

### Relief

What a crate spares purba: the code purba would write without it, or the quality it serves.

### Repair

A change that restores what a [record](#record), a [design spec](#design-spec) or a [gate](#gate) already requires. It carries no decision.

### Replay

A process that the [zygote](#zygote) forks to run a [test](#test) again, and to read one [expectation](#expectation) alone.

### Reviewer

The [actor](#actor) that reads and approves work it did not write.

### Risk

A failure scenario a test can check. It lands on one [part](#part) or [concern](#concern), and is weighed by how likely it is and by the quality it breaks.

### Run

One start of purba, from the command it reads to the exit code it sets.

### Shortlist

The crates that pass the checks for one need of a [part](#part) not built yet, each with the [relief](#relief) it needs to join. The design in depth of the part starts from it.

### Significance

The property that routes a change to the code owner. A change is architecturally significant if and only if it adds or changes a [record](#record).

### Stand-in

A value that a test gives the code under test in place of a real one, such as a mock or a stub.

### Stem

The [actor](#actor) that arrives at a [location](#location) it does not know yet. An actor returns to the stem at every location it does not know.

### Stock

An [input](#input) that outlives one [test](#test), and whose content can change. Only a fixture takes it, and that fixture hands each test a [reagent](#reagent) that it draws from the stock.

### Strict check

A rule that judges one of a [tester's](#tester) tests and gives a verdict of violation, clean or unknown. Its name is its breach, in kebab case, such as `computed-expected`. A [gate](#gate) judges a contributor's change instead.

### Suppression

A [tester's](#tester) comment that names one [strict check](#strict-check) and gives a reason, such as `# purba: suppress[computed-expected] <reason>`. It sits above a test or at the top of a module. purba shows each suppressed verdict as a warning.

### Tag

A word that a [test](#test) carries through `purba.tag`, so that a [query](#query) can pick it.

### Technical writer

The [actor](#actor) that writes purba's documentation for a [reader](#reader) outside the work.

### Test

A function at the top level of a [test file](#test-file), whose name starts with `test`, as import binds it, that carries only purba's own decorators, under purba's own names, and names the fixture of each of its [inputs](#input). It returns nothing, and is not a generator. A [suppression](#suppression) lets a function that breaks a check of the [practice tier](#practice-tier) run as a test. purba calls it to try to disprove a behaviour.

### Test critique

A read of a change's tests, by a context that did not write them, before the code is pushed. It looks for a behaviour that no test checks, a repeat, a test in the wrong [layer](#layer), and a test that fails for the wrong reason. It leaves its findings as a comment on the [pull request](#pull-request).

### Test file

A `.py` file whose name matches `test_*.py` or `*_test.py`.

### Tester

The [actor](#actor) that runs purba against its own code.

### Toolsmith

The [actor](#actor) that builds and configures purba's tooling.

### Triage

What an [issue](#issue) gets when it is filed: one label that says how it is resolved, and the issues that block it.

### Unpublished comment

A comment that stays in the tree, read only by somebody who opens the file. It is written only when the information belongs in no other [location](#location).

### Unreached test

A selected test that the [run](#run) never reached, because an [interrupt](#interrupt) stopped the run first. It has no [outcome](#outcome).

### Waiver

A decision that lets one crate in despite the failures it names, and says why.

### Walking skeleton

purba's first working code: a small version of every [part](#part), joined so that one [run](#run) goes through each of them. The later design of each part grows from this code, and does not replace it.

### Zygote

The process that purba forks for one module of a [run](#run). It loads the module once, and forks a process for each [test](#test) of that module. When that load leaves a Python thread alive, purba forks a second zygote that loads nothing.
