# Run a test critique

[A test critique reads a change's tests before its code is pushed](../decisions/a-test-critique-reads-a-changes-tests-before-its-code-is-pushed.md#decision-outcome) says when an agent owes a [test critique](../../CONTEXT.md#test-critique), and what it judges.

## The steps

Run the steps in a context that did not write the tests.

1. Take the head of the push of the tests.
2. Run `mise run list:records` on that head. Read the Decision Outcome and the Confirmation of each record that it prints on every pull request, and of each record that it prints as named by this change.
3. Read the design spec, and the test list of the implementation plan.
4. Read the tests in every [layer](../../CONTEXT.md#layer) where a test can check the behaviour.
5. Read the log of the run of the tests.
6. Judge the tests on each judgement and each practice that [the record of the test critique](../decisions/a-test-critique-reads-a-changes-tests-before-its-code-is-pushed.md#decision-outcome) names. Measure each judgement against its row.

## The comment

Post the critique as one comment on the pull request.
It carries these parts:

- the head that the critique read;
- the records on every pull request, and the records that the change names, as `mise run list:records` prints them;
- for each judgement, each finding, or "none";
- for each practice, each finding, or "none".

[A claim links what decides it](../decisions/a-claim-links-what-decides-it.md#decision-outcome) says what each finding links.

A new critique supersedes the one it replaces by the procedure in [an artifact is rewritten until its direction is agreed](../decisions/an-artifact-is-rewritten-until-its-direction-is-agreed.md#decision-outcome).

## What belongs in this file

This section is the register of this file, and [a register belongs to one location](../decisions/a-register-belongs-to-one-location.md#confirmation) names it.

A line belongs here when it says how to run a test critique, or what its comment carries.
This file links a rule that a record holds, and never states it.
