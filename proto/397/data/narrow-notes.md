# Narrow reach notes

Written from the 34 records, `CONTEXT.md`, `tree.txt`, `wide-reaches.tsv` and `wide-reach-notes.md` only.

The narrow rule changed only three records, and it removed five entries. Most wide entries are tool configurations, gate scripts, carriers such as `AGENTS.md`, or tracker locations. A change to the content of each of those can break the record's rule. The removed entries share one shape: a change at that path makes the record *apply*, but the breach shows up somewhere else. That is either a routing file (`CODEOWNERS` and the ruleset) or the pull request's pushed heads.

Test applied to each entry: *can an edit to this location's content, or adding or removing a file there, make the rule false?* A location that says only *when* the rule applies (its antecedent) or *who* gets the change (its routing) was removed.

## Removed

| record | removed entry | quote from the record | why it routes rather than breaks |
|---|---|---|---|
| `an-implementation-follows-test-driven-development.md` | `scripts/**` | "wherever a runner executes the change. That is shell under `scripts/` or `.github/scripts/`, which `bats` runs" | The path is the rule's antecedent. It says when test-first is owed. The breach is "a head that holds only the test, with `Scripts` or `Build` at `FAILURE`" being missing, which shows only on the pull request. No script text can be test-first or not test-first. |
| `an-implementation-follows-test-driven-development.md` | `.github/scripts/**` | same sentence as above | Same reason: a trigger for the obligation, not content that can breach it. |
| `an-implementation-follows-test-driven-development.md` | `*.rs` | "and Rust, which `cargo test` runs" | Same reason. "The proof stays on GitHub. `main` carries the test and the code in one commit, so only the pull request shows the failed head." |
| `significance-is-declared-not-detected.md` | `docs/decisions/*.md` | "`CODEOWNERS` assigns `/docs/decisions/` to the human, and the ruleset requires that owner's review on a pull request touching it." / "Read the routing back from the ruleset, not from here" | A record change is what gets routed. No sentence in any record can make "significant if and only if it adds or changes a record" false. Only the routing in `CODEOWNERS` and the ruleset can break it. |
| `the-person-owns-the-decision-outcome.md` | `docs/decisions/*.md` | "`CODEOWNERS` routes the directory and the required approval pins it, so the rule is decidable and true." | Ownership is made true by routing and approval, not by record text. A record that is changed is the thing routed. See Hard calls for the one sentence that argues against this removal. |

## Kept despite looking like routing

| record | entry kept | the record sentence that makes its content breakable |
|---|---|---|
| `liability-is-recorded-from-the-act-that-makes-it-true.md` | `/CODEOWNERS` | "stewardship \| the code owner \| whenever ownership changes \| `CODEOWNERS`". `CODEOWNERS` *is* the stewardship statement. Also: "`CODEOWNERS` covers `/.github/` and `/scripts/` for this reason". Dropping a line breaks it. |
| `only-github-runs-what-lives-under-github.md` | `/CODEOWNERS` | "**`CODEOWNERS` covers both directories.** Placement decides which directory a script sits in, and never who approves it." The rule is about what `CODEOWNERS` says. |
| `significance-is-declared-not-detected.md`, `the-person-owns-the-decision-outcome.md` | `/CODEOWNERS`, `NOT-IN-GLOSSARY:ruleset` | "`CODEOWNERS` assigns `/docs/decisions/` to the person. The ruleset requires that owner's review". This routing *is* the rule, so its content breaks it. |
| `a-commit-outlives-its-review.md` | `/prek.toml` | "`prek.toml` carries the hooks that hold five of these rows." A hook removed there falsifies the Confirmation. The file defines the check. A change to it does not merely trigger the check. |
| `an-artifact-is-rewritten-until-its-direction-is-agreed.md` | `.github/workflows/sign.yml` | "the review thread `.github/workflows/sign.yml` posts \| the reviewer while they decide, blocking the merge until resolved \| live". It is a carrier of the agreement, like `AGENTS.md`. An edit that stops the thread breaks the "live" claim. |
| `an-unpublished-comment-carries-what-no-other-location-carries.md` | `/.gitattributes` | "offline \| every tracked file that `.gitattributes` does not mark generated". A false generated mark takes a hand-written file out of the gated half: "Link liveness and the paths a comment cites are decidable, so those two are gated". |
| `mise-names-every-tool-version.md`, `purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md` | `/.gitattributes` | "A lockfile is generated rather than authored, so `.gitattributes` marks it `linguist-generated` and a reviewer is not shown its diff." The mark hides a diff from review, which looks like routing. But the rule *is* the mark, and a missing mark breaks it. |
| `a-new-question-gets-a-new-record.md` | `docs/decisions/*.md` | "When an issue asks a question that no title on `main` answers, its answer goes in a new record." A record that absorbs a second decision breaks the rule in its own text. The Confirmation's "the reviewer of each change to `docs/decisions/`" is routing, but it is not the only reason the entry is there. |
| `a-term-belongs-to-the-glossary.md` | `docs/decisions/*.md` | "A term is defined once, in `CONTEXT.md`, and a record names it and does not hold it." A definition written into a record breaks it. |
| `the-crate-carries-an-rlib.md` | `*.rs` | "A fence on a bridge item fails to link, so it becomes prose or it moves to the core." and "A Rust example that nothing compiles is prose that looks like code." The fences live in `.rs` content. |
| `the-nightly-is-named-by-a-date.md` | `.github/workflows/bump.yml` | "A person moves the date." A `bump.yml` that merged or floated the date itself would break it. The file is not there because it triggers a run. |
| `a-change-is-specified-before-it-is-built.md` | `/tasks.toml` | "`tasks.toml` names the ones a contributor runs". It is a content claim about the file. |

## Empty reaches

None. Every narrowed record keeps at least one pattern or word:

- `an-implementation-follows-test-driven-development.md` keeps `/AGENTS.md`, `Implementation plan` and `Pull request`.
- `significance-is-declared-not-detected.md` keeps `/CODEOWNERS` and the ruleset.
- `the-person-owns-the-decision-outcome.md` keeps `/CODEOWNERS` and the ruleset.

## Missed by the wide reach (not added)

| record | location | the record sentence |
|---|---|---|
| `the-nightly-is-named-by-a-date.md` | `.github/workflows/*.yml` (only `build.yml` and `bump.yml` are in) | "each workflow names it [mise] and a developer machine does not" |
| `purba-meets-the-next-trait-solver-before-it-stabilizes.md` | `.github/workflows/*.yml`, `NOT-IN-GLOSSARY:ruleset` | "`cargo +<stable> check`, run on every pull request, reporting and never blocking". A workflow or required context that blocks on it breaks the rule. |
| `a-register-belongs-to-one-location.md` | `.github/workflows/*.yml` | Rejected options: "A required status check over the pull request body" and "A workflow on every issue, checking its headings". Adding either breaks the decision. |
| `liability-is-recorded-from-the-act-that-makes-it-true.md` | `.github/workflows/commits.yml` (probably; the record does not name the file) | "An advisory check reports the same rule on every push, and it is deliberately not a required one." |
| `an-unpublished-comment-carries-what-no-other-location-carries.md` | `scripts/test/check-links.bats` | "`scripts/test/check-links.bats` builds each shape the gate refuses offline." The test is named by the record, so the wide convention ("unless the record names the test") should have included it. |
| `the-crate-carries-an-rlib.md` | `NOT-IN-GLOSSARY:ruleset` | "the `Quality` job, whose name the ruleset holds as a required context" |
| `an-artifact-is-rewritten-until-its-direction-is-agreed.md` | `.github/scripts/post-record-thread.sh` (unnamed by the record; inferred from the tree only) | "the review thread `.github/workflows/sign.yml` posts". The thread's text may live in this script rather than in `sign.yml`. |
| `one-manifest-declares-each-package.md` | `rust-toolchain.toml` | `the-nightly-is-named-by-a-date` says its absence "follows from [one manifest declares each package]". This record's own text says only "A name in two manifests is a conflict", so the binding is indirect. |

## Hard calls

1. **The TDD record lost its three path patterns, and that may not narrow anything in practice.** The paths are the rule's antecedent ("wherever a runner executes the change"). No script or Rust text can breach test-first. But the reach keeps `Pull request`, and every pull request writes that location. So after the removal, the record is named on *every* pull request rather than on shell and Rust changes. The removal is right under the rule, but it trades a precise trigger for a match on every pull request. Replay should show the record named on prose-only pull requests that the wide reach did not name.
2. **`the-person-owns-the-decision-outcome.md`: one sentence argues for keeping `docs/decisions/*.md`.** The Context records that a *record* once stated a conflicting rule: "`significance-is-declared-not-detected` \| an agent may draft it where the person has shown, in review, that the points are read and understood". So record text did once contradict this rule. It was still removed, for two reasons. First, the record makes the rule true through routing ("`CODEOWNERS` routes the directory and the required approval pins it, so the rule is decidable and true"). Second, "another record could contradict this one" holds for every record, so it would put `docs/decisions/*.md` back into all 34 reaches. If replay misses a pull request that rewrote a record's Decision Outcome rule, this removal is the cause.
3. **`significance-is-declared-not-detected.md` now matches through `CODEOWNERS` alone.** The ruleset never matches in replay. So replay names this record only when a pull request edits `CODEOWNERS`. That is the honest narrow reach, but it means a pull request that *adds* a record no longer names the record that defines significance.
4. **A gate's own file was kept everywhere (`prek.toml`, `check-records.sh`, `check-links.sh`, `sign.yml` and its scripts, `quality.yml`).** One reading of "triggers a process" would remove them. They were kept because their content *is* the check: removing a hook or a refusal falsifies a Confirmation sentence. The change does not just set the check off. A stricter reading would remove these, and it would leave `a-commit-outlives-its-review.md` with only `Commit message`.
5. **The `.gitattributes` generated marks were kept in three records.** The mark decides which files a reviewer or a gate reads, which looks like routing. It was kept because each record states the mark itself as the rule ("`.gitattributes` marks it `linguist-generated`"), or ties the gated half to it.
