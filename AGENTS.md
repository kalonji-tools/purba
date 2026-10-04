# Instructions for an agent

Each line below is an instruction that no command enforces.
A rule that a command refuses is not here, because the command names the breach when it fires.
Each line links the file that states its rule, and the section of a record that holds it.

## Before you take an issue

- Do not implement an issue whose type is not set. Read it with `gh issue view <n> --json issueType`. ([a change is specified before it is built](docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome))
- Set the type only after the person who decides agrees to the spec. ([a change is specified before it is built](docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome))
- Read the spec against the tree, and narrow it where the tree moved. ([a change is specified before it is built](docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome))
- Post the answer to a question as its spec. The record is its implementation. ([a change is specified before it is built](docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome))

## When you write on an issue

- Rewrite an issue with no type in place. ([an artifact is rewritten until its direction is agreed](docs/decisions/an-artifact-is-rewritten-until-its-direction-is-agreed.md#decision-outcome))
- On an issue with a type, fix only a typo or a wrong filename. Replace anything else in this order. ([an artifact is rewritten until its direction is agreed](docs/decisions/an-artifact-is-rewritten-until-its-direction-is-agreed.md#decision-outcome))
  1. Post the replacement.
  2. Copy its URL.
  3. Mark the old text with that URL.
  4. Minimize the old comment when all of it is replaced.
- Narrow an issue, and never change the question it asks. Close a replaced question as `not_planned` with its answer, and open a new issue that links back to it. ([an artifact is rewritten until its direction is agreed](docs/decisions/an-artifact-is-rewritten-until-its-direction-is-agreed.md#decision-outcome))
- File a defect when you find it, and give it one label and its blockers. ([a change is specified before it is built](docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome), [triage](CONTEXT.md#triage))
- Land a rule somebody acts on in a tracked file before its issue closes. ([a change is specified before it is built](docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome))

## When you open a pull request

- Open it as a draft, and write the implementation plan before the implementation. ([a change is specified before it is built](docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome))
- Write the implementation plan in the body, under the headings of [`.github/PULL_REQUEST_TEMPLATE.md`](.github/PULL_REQUEST_TEMPLATE.md). ([a register belongs to one location](docs/decisions/a-register-belongs-to-one-location.md#decision-outcome))
- Where a runner executes the change, follow TDD: put the test list in the plan, and push the test alone until its run fails. ([an implementation follows test-driven development](docs/decisions/an-implementation-follows-test-driven-development.md#decision-outcome))
- Answer one issue with one pull request. ([a change is specified before it is built](docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome))
- Read your own branch after the implementation, and post what you find as a comment on the pull request. ([post pass](CONTEXT.md#post-pass))
- Find the commands you may run with `mise tasks ls`. ([`tasks.toml`](tasks.toml))

## During the review

- After you mark a pull request ready, watch it for review, and act on each round before anyone asks. ([the agent that marks a pull request ready watches it](docs/decisions/the-agent-that-marks-a-pull-request-ready-watches-it.md#decision-outcome))
- Make a change asked for in review as a new commit. Rewrite the branch only to follow `main`. ([a change is specified before it is built](docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome))
- Reduce the commits only once the reviewer is satisfied. ([a change is specified before it is built](docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome))

## In every commit

- Never write a `Signed-off-by:` trailer, and never run `mise run sign-off`. ([liability is recorded from the act that makes it true](docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md#decision-outcome), [`tasks.toml`](tasks.toml))
- Write an `Assisted-by: AGENT:MODEL` trailer, and no session address. ([a commit message outlives its review](docs/decisions/a-commit-outlives-its-review.md#decision-outcome))
- Write a body only for a sentence that must outlive the review and that no other location carries. ([a commit message outlives its review](docs/decisions/a-commit-outlives-its-review.md#decision-outcome))
- Never name the position of a commit in a planned series. ([a commit message outlives its review](docs/decisions/a-commit-outlives-its-review.md#decision-outcome))
- In a commit that rewrites a record, say what was wrong before, unless the diff shows it. ([a commit message outlives its review](docs/decisions/a-commit-outlives-its-review.md#decision-outcome))
- Check that the number in the subject names the issue the change answers. ([a commit message outlives its review](docs/decisions/a-commit-outlives-its-review.md#confirmation))

## When you write a record

- Put the answer to a new question in a new record. A question is new when no title on `main` answers it. ([a new question gets a new record](docs/decisions/a-new-question-gets-a-new-record.md#decision-outcome))
- Replace the text of a record. Never annotate it. ([a decision record is rewritten, not amended](docs/decisions/a-record-is-rewritten-not-amended.md#decision-outcome))
- Leave the Decision Outcome to the person. You may draft it. ([the person owns the Decision Outcome](docs/decisions/the-person-owns-the-decision-outcome.md#decision-outcome))
- Define no word in a record. A word that is specific to purba goes in `CONTEXT.md`. ([a term belongs to the glossary](docs/decisions/a-term-belongs-to-the-glossary.md#decision-outcome))
- Rewrite a record when an issue it waits on closes, and remove the link to that issue. ([a decision record is rewritten, not amended](docs/decisions/a-record-is-rewritten-not-amended.md#decision-outcome), [`.template.md`](docs/decisions/.template.md))

## In anything you write

- Write the minimum that conveys the point. Leave out what your session left behind. ([an artifact holds the minimum that conveys its point](docs/decisions/an-artifact-holds-the-minimum-that-conveys-its-point.md#decision-outcome))
- Write `issue`, never `ticket`. ([issue](CONTEXT.md#issue))
- Write a comment in the tree only for what no other location carries. Cite the primary source, never an issue number. ([an unpublished comment carries what no other location carries](docs/decisions/an-unpublished-comment-carries-what-no-other-location-carries.md#decision-outcome))

## After the merge

- Post a debrief on the pull request when the merged change differs from its implementation plan. Never edit the merged description. ([a change is specified before it is built](docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome), [an artifact is rewritten until its direction is agreed](docs/decisions/an-artifact-is-rewritten-until-its-direction-is-agreed.md#decision-outcome))

## What belongs in this file

This section is the register of this file, and [a register belongs to one location](docs/decisions/a-register-belongs-to-one-location.md#confirmation) names it.

A line belongs here when all three are true.

1. An agent acts on it.
2. A tracked file on `main` states it, and the line links that file, down to the section of a record.
3. No command refuses a breach of it.

A line leaves when a gate takes its rule, or when its source changes.
Nothing describes the repository here, because an overview does not help an agent act. ([Is there prior art for splitting agent-facing from human-facing artifacts?](https://github.com/kalonji-tools/purba/issues/70))
