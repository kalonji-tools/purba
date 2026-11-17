# A test critique reads a change's tests before its code is pushed

## Context and Problem Statement

The author of a change pushes its tests alone, and the code follows once their run fails.
No step reads the tests while the author waits.
A behaviour that no test checks then reaches the test list only after the code.

| fact | source |
|---|---|
| the author pushes the test alone, and its run fails before the code follows | [an implementation follows test-driven development L47](https://github.com/kalonji-tools/purba/blob/b93bb6d66f89b6509c6bde60eb4fa8caed56ac0e/docs/decisions/an-implementation-follows-test-driven-development.md?plain=1#L47) |
| the TDD record and `AGENTS.md` name no step between the push of the test and the push of the code | the same line, and [`AGENTS.md` L31](https://github.com/kalonji-tools/purba/blob/b93bb6d66f89b6509c6bde60eb4fa8caed56ac0e/AGENTS.md?plain=1#L31) |
| the test "an `any()` that holds stops at its first true item" joined the test list after its code, in step 11 | [the implementation plan that checks each item with `all()` and `any()`](https://github.com/kalonji-tools/purba/pull/1047) |
| a test on the wheel caught a mutant of `whence.rs` that `cargo test` missed | [`research/test-review-tools.md` § 5](https://github.com/kalonji-tools/purba/blob/e4cc37cf0ca827df468f9ecc295627bce94a3ee8/research/test-review-tools.md#a-mutant-missed-by-cargo-test-caught-by-the-wheel) |
| two tests that reach the same lines can check different inputs | [the same file, § 3](https://github.com/kalonji-tools/purba/blob/e4cc37cf0ca827df468f9ecc295627bce94a3ee8/research/test-review-tools.md#the-probes-on-purba) |
| cargo-mutants `--in-diff` runs no mutant for a diff that changes only test code | [in-diff.md L22](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/in-diff.md?plain=1#L22) |
| P1, P2, P3 and P14, the strength of the evidence for each, and the contradiction of the sources on most other practices | `secondary`: [the survey of testing practices](https://github.com/kalonji-tools/purba/blob/bbc590c8ad4667f9880166b9d73ef6a6e46fbae4/research/testing-practices.md#answer), which cites the originator of each |
| no record says which tests are Rust unit tests, and which go under `crates/*/tests` | [What does an agent judge in a change's tests while its red run waits, and in which layers?](https://github.com/kalonji-tools/purba/issues/1059#issuecomment-6954555526), § 2 |
| 102 lines of Markdown use "review", in the sense of the reviewer's | [What is the review of a change's tests called?](https://github.com/kalonji-tools/purba/issues/1062#issuecomment-6954959500) |

## Considered Options

- **No step reads the tests while their run waits, as on `main`.** The test of the first true item of an `any()` reached the test list after the code.
- **The critique reads only the layers that the change touches.** It then reports a gap that a test in another layer closes. A test on the wheel caught a mutant that `cargo test` missed.
- **The author pushes the code before the critique posts.** A test that the author adds for a finding then follows the code, and passes against what the code does.
- **A tool makes a judgement.** cargo-mutants `--in-diff` runs no mutant for a diff that changes only tests. A coverage tool cannot find a repeat, because two tests that reach the same lines can check different inputs.

## Decision Outcome

**Reach:** [implementation plan](../../CONTEXT.md#implementation-plan) [pull request](../../CONTEXT.md#pull-request) `crates/**` `tests/**` `scripts/**` `docs/agents/test-critique.md` `/AGENTS.md` `/CONTEXT.md`

**Area:** workflow

**A [test critique](../../CONTEXT.md#test-critique) reads a change's tests before its code is pushed.**
A test that the author adds for a finding then fails before the code follows, as [an implementation follows test-driven development](an-implementation-follows-test-driven-development.md#decision-outcome) asks.

**The name:** a test critique, never a review, a pass or a run.
`CONTEXT.md` holds no `_Avoid_` line for "review", because [the gate](the-glossary-stays-open-and-a-gate-rewrites-each-word-it-avoids.md#decision-outcome) would rewrite each "review" of the reviewer.

**Who reads:** a context that did not write the tests.

**What it reads:**

- the design spec;
- the test list of the implementation plan;
- the tests in every [layer](../../CONTEXT.md#layer) where a test can check the behaviour, and not only the layers that the change touches;
- the log of the run of the tests;
- the records that `mise run list:records` names on the head of the tests.

No tool runs, because the code under test does not exist yet.

**What it judges:**

| judgement | measured against |
|---|---|
| a behaviour that no test checks | the behaviours that the spec names, each Confirmation line of a record that it reads, and the edge cases that it names itself: empty, one, many, an error, a boundary. Each edge case is a judgement |
| a repeat | another test in the same layer, with the same input and the same check. A test in another layer is not a repeat |
| a wrong layer | the layer rule of a record. Where no record holds one, the finding is a judgement |
| a wrong reason | the reason that the test list states, in meaning and not in its text. A build error counts only when the test list names it |

**The practices:** the critique also judges each test against these practices of the survey.

- **P1:** the test decides its own verdict.
- **P2:** the test gives the same result on the same code.
- **P3:** the test does not depend on another test.
- **P14:** nobody switches a test off.

The survey gives P1 to P3 the strongest evidence.
It disputes P14 only for a skip that states its reason.
The critique does not judge P4 to P13, and there purba departs from the survey.
The sources of the survey contradict each other on several of those practices.

**The comment:** the critique posts one comment on the pull request, even when it finds nothing.
Each finding is a [claim](../../CONTEXT.md#claim), so [a claim links what decides it](a-claim-links-what-decides-it.md#decision-outcome) says what it links.

**The wait:**

| first push of the tests | the code waits for |
|---|---|
| the run fails | the failed run and the comment |
| a characterization test, and the run passes | the passed run and the comment. The critique judges no wrong reason |
| no push, because the plan names a test that covers the code | nothing. Nobody owes a critique |

**What the author does with a finding:**

| finding | the author |
|---|---|
| a behaviour that no test checks | adds a row to the test list, and a test |
| a repeat | removes one of the two tests |
| a wrong layer | moves the test to the layer that the finding names |
| a wrong reason | rewrites the test, or the reason in its row |
| a finding that the author does not act on | writes it in the pull request body, under "What you must decide" |

A test that the author adds or moves goes in a push of the tests alone.
Its run fails before the code follows.

**A later push of tests:** it owes a new critique only when it adds a test that nobody asked for.
Only a finding of a critique, or the reviewer, asks for a test.
A new critique reads the whole change.
Its comment supersedes the comment of the earlier critique.

**Downside:**

- **The code waits for the critique too.** No change measured how long a critique takes.
- **Each critique costs a context.** No change measured its cost in tokens.
- **An edge case that the critique names is a judgement.** Two critiques can name different edge cases.
- **No record says which tests are Rust unit tests, and which go under `crates/*/tests`.** A wrong layer between those two is a judgement.

## Confirmation

No command checks this rule.
The reviewer reads the order of the pushes and the comment of the critique on each pull request.

[What does an agent judge in a change's tests while its red run waits, and in which layers?](https://github.com/kalonji-tools/purba/issues/1059), [What does the review of a change's tests post, and does the code wait for it?](https://github.com/kalonji-tools/purba/issues/1061) and [What is the review of a change's tests called?](https://github.com/kalonji-tools/purba/issues/1062) decided the rule.
