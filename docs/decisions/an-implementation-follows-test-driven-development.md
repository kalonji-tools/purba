# An implementation follows test-driven development

## Context and Problem Statement

purba's tests followed the code they test.
When purba took this decision, every `bats` test on `main` came after its script.
No file asked for another order.

A test written after the code passes against what the code does, not against what the change meant.
The reviewer reads such a test once it exists, and cannot steer it.

[A change is specified before it is built](a-change-is-specified-before-it-is-built.md) puts the implementation plan before the implementation.
It does not say how an author builds it.

## Considered Options

- **TDD for every implementation.** Rejected. A record or a prose change has no test that a runner executes, so no test can fail first.
- **TDD wherever a runner executes the change.** Chosen.

The proof of the order was weighed separately.

- **The order of the commits.** Rejected. It shows an order and no failure, and a reduction of the commits erases it.
- **A push that holds only the test, and fails.** Chosen. CI runs on the head of a push, so only a push of its own runs the test alone.

So was a gate.

- **A gate that reads the changed paths.** Rejected. It checks that a push came first, and not what that push showed.
- **No gate.** Chosen.

## Decision Outcome

**An implementation follows test-driven development (TDD) wherever a runner executes the change.**
That is shell under `scripts/` or `.github/scripts/`, which `bats` runs, and Rust, which `cargo test` runs.
A change to prose or to configuration owes no test of its own.
The test of a gate covers the configuration that the gate reads.
Kent Beck's [Canon TDD](https://newsletter.kentbeck.com/p/canon-tdd) defines the cycle.

**The [implementation plan](../../CONTEXT.md#implementation-plan) holds the test list, and says why each case fails today.**

| case | fails today because |
|---|---|
| refuses a package that two manifests declare | the script reads one manifest |
| names both files in the refusal | no refusal exists |

**The author pushes the test alone, and its run fails before the code follows.**
`Scripts` and `Build` cancel a run when a new push lands, and a cancelled run proves nothing.
The failed head outlives a reduction of the commits, because the pull request keeps each head that a force push replaced.

**A change with no new behaviour proves its coverage first.**
The plan names the test that covers the code.
Where no test does, the first push adds a [characterization test](https://michaelfeathers.silvrback.com/characterization-testing) that passes against the old code.
The change then keeps that test green.
The plan says whether its first push fails or passes.

**Downside:**

- **Nothing refuses a skipped test.** Only the reviewer reads the history, and a reviewer who does not look finds nothing.
- **A red run costs a wait.** The author waits for CI to report before the code is pushed.
- **Whether a change adds behaviour is a judgement.** A change the author calls a refactor owes a pass instead of a failure.
- **The proof stays on GitHub.** `main` carries the test and the code in one commit, so only the pull request shows the failed head.
- **Configuration that no gate reads owes no test.** A wrong value there waits for the change that first reads it.

## Confirmation

No command decides any of this, and none is planned.

| step | what shows it |
|---|---|
| the test list came first | the implementation plan in the body of the pull request |
| the test failed first | a head that holds only the test, with `Scripts` or `Build` at `FAILURE` |

A head still on the pull request shows its checks there.
This query reads a head that a force push replaced:

```
gh api graphql -f query='{repository(owner:"kalonji-tools",name:"purba"){pullRequest(number:<n>){timelineItems(first:50,itemTypes:[HEAD_REF_FORCE_PUSHED_EVENT]){nodes{... on HeadRefForcePushedEvent{beforeCommit{oid checkSuites(first:20){nodes{conclusion workflowRun{workflow{name}}}}}}}}}}}'
```

[`AGENTS.md`](../../AGENTS.md) carries this rule to an agent before it acts.
