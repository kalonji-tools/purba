# A form that a tester meets is rated blind before it is decided

## Context and Problem Statement

An answer on an issue can pick the text that a tester writes in a test, or reads in the report of a run.
No record says when such an answer owes a blind rating.

| fact | source |
|---|---|
| two records on `main` cite a blind rating, and both come from the same issue | [the probe of the blind rating issue](https://github.com/kalonji-tools/purba/issues/1055#issuecomment-6954006519) |
| no record says when a blind rating is owed | the same |
| an answer changed a form with no rating, and its issue reopened | [the reuse issue](https://github.com/kalonji-tools/purba/issues/1010#issuecomment-6901755382) |
| its rating flipped two of its decisions | [the rating of the reuse issue](https://github.com/kalonji-tools/purba/issues/1010#issuecomment-6902837623) |
| the raters of one round scored pytest 0.28 lower, and oxitest 0.43 lower, than the raters of an earlier round | [prototype 2 of the experiment issue](https://github.com/kalonji-tools/purba/issues/967#issuecomment-6876537645) |

## Considered Options

- **No record says when an answer owes a rating, as on `main`.** The answer on [the reuse issue](https://github.com/kalonji-tools/purba/issues/1010#issuecomment-6901755382) had no rating.
- **Three raters rank written examples, and the ranking decides, as on [the input issue](https://github.com/kalonji-tools/purba/issues/700).** The raters never run the form. [The person owns the Decision Outcome](the-person-owns-the-decision-outcome.md#decision-outcome).
- **A record already on `main` holds the rule.** [A claim links what decides it](a-claim-links-what-decides-it.md#decision-outcome) says what a claim carries. This rule says when an answer owes a rating. [A change is specified before it is built](a-change-is-specified-before-it-is-built.md#decision-outcome) would grow by a whole procedure.

## Decision Outcome

**Reach:** [design spec](../../CONTEXT.md#design-spec) `crates/**` `tests/**` `/AGENTS.md` `/CONTEXT.md`

**Area:** workflow

**An answer that picks or changes a [form](../../CONTEXT.md#form) owes a [blind rating](../../CONTEXT.md#blind-rating) before the person decides.**
A later rating flipped decisions of an answer that changed a form with no rating.

**What owes a rating:**

- An answer that picks a form that a [tester](../../CONTEXT.md#tester) writes, such as the signature of a test, its cases or its fixtures.
- An answer that picks a form of the report of a [run](../../CONTEXT.md#run).
- An edit of a form that changes a line of the test that a rating ports.
- A form that an answer picked before this record, when an answer next changes it.

The rest of the [public surface](../../CONTEXT.md#public-surface), such as an exit code, owes no rating, because a rater cannot score it.

**The steps of a rating:**

1. A fresh context that reads only the written rules of the form ports the test. The port then checks those rules too.
2. A form that no runner executes gets a throwaway prototype first, so that the raters can run it.
3. The port covers loguru's [`test_add_option_format.py`](https://github.com/Delgan/loguru/blob/28349be3cd740867bc947eef741c69370ca45735/tests/test_add_option_format.py), and one more real test that shows the change.
4. Three raters rate, each in a fresh context. They do not see the issue, the design history or an earlier score.
5. The forms carry shuffled letters, so a rater does not know which form is new.
6. Each rater runs every form, and adds a case and a test to each. It writes the smallest test that passes in each, and seeds failures.
7. The raters score pytest and oxitest again in each round.

**What the answer reports:**

- A score for each form beside pytest and oxitest, and the sentence "A round compares only with itself."
- A link to each rater's scores, to the exact prompt, and to the folder that holds the ported tests, at a commit.

The report of a rating holds [claims](../../CONTEXT.md#claim), so [a claim links what decides it](a-claim-links-what-decides-it.md#decision-outcome) governs these links.
The person decides, and the rating does not. An answer that picks a form below the top score says why.

**Downside:**

- **Each answer that changes a form costs a port and three raters.** No answer measured its cost in tokens.
- **A score compares only with the scores of its own round.**
- **No command checks that an answer owed a rating, or that it got one.** Only the person reads each answer for it.

## Confirmation

No command checks this rule.
The person judges each answer that picks or changes a form, and then agrees to its spec or not.

[When does a choice between the forms of a design go to blind raters before it is decided?](https://github.com/kalonji-tools/purba/issues/1055) decided the rule.
