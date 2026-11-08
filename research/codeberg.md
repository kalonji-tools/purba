# What a move from GitHub to Codeberg would lose, gain, restructure and cost

This file answers [What would purba lose, gain and restructure by moving from GitHub to Codeberg, and what would the move cost?](https://github.com/kalonji-tools/purba/issues/859)
It is never merged.

## Answer

**Codeberg's Terms of Use bar purba as it is built today.** § 2 (1) 7, voted in July 2026, says: "You must not share projects that mostly consist of code written by "generative AI"-tools (including services such as *Claude*, *OpenAI Codex*)."
An agent account writes purba's code. Whether purba falls under the clause is Codeberg's moderation call, and § 2 (2) sets "immediate removal of the content" for a breach.
Codeberg's hosted runners add a second rule: "No extensive pipelines to assist LLM-driven workflows".

Without the clause, CI is the largest gap. Codeberg hosts amd64 Linux runners only, with at most 4 CPUs and 10 minutes a job. purba builds and tests on macOS too.
PyPI's trusted publishing supports GitHub Actions, GitLab CI, Google Cloud and ActiveState, and not Forgejo or Codeberg.
Forgejo v17 has required checks with no admin bypass, issue forms, labels and issue dependencies. It has no issue types, no sub-issues, no GraphQL and no way to hide a comment, and purba's process uses each of them.

The gain is not clear. Codeberg posted trouble on 29 dates in 11 months, and on 2026-06-17 it kept signed-in users responsive at the cost of anonymous readers: the failure that started #858.

| question | short answer | tier |
|---|---|---|
| 1. What purba uses of GitHub | Actions run every check; the ruleset requires Sign-off, Quality and Build with no bypass; `Accepted-by:` comes from a GitHub review; the process rests on issue types, sub-issues and dependencies | [A] |
| 2. What Codeberg and Forgejo offer | most forge features; amd64 Linux CI only; no PyPI trusted publishing; no issue types, sub-issues or GraphQL | [A] |
| 3. What is lost | hosting itself, under § 2 (1) 7; macOS CI; trusted publishing; the issue features the process uses | [A] |
| 4. What is gained | no measured gain in reliability; Codeberg throttles anonymous readers too | [A] counts from Codeberg's posts |
| 5. What is restructured | every workflow, the signing flow, the process, the records' addresses | [A] inventory; [reasoning] for the work |
| 6. What it costs | see section 6; the clause makes the move impossible as written | [reasoning] |
| 7. Other projects | 10 of 16 cases left GitHub; 7 of 7 rewrote CI; 2 of 5 lost macOS or Windows CI; 5 of 6 importers lost data | [A] mostly |

## 1. What purba uses of GitHub

Measured from the tree at `e2ccac8` and from the API.

| use | how hard it is tied | what depends on it |
|---|---|---|
| 10 workflows in `.github/workflows/`, with Linux and macOS matrices | load-bearing | every check: Build, Quality, publish, release, sign |
| the ruleset `protect-main`: Sign-off, Quality, Build; rebase only; code owner; 0 bypass actors | load-bearing | each merge |
| `sign.yml` posts the check run `Sign-off` itself through the Checks API | load-bearing | the required check Sign-off |
| `Accepted-by:`, built from a GitHub review, a collaborator-permission call and the approver's numeric id | load-bearing | 471 of 495 commits on `main` carry it |
| a GitHub App token in `sign.yml` | load-bearing | the signing flow |
| PyPI trusted publishing (OIDC) in `publish.yml` | config | the first release; no PyPI project exists yet |
| issue types: Task marks a spec agreed | load-bearing | 272 of 501 issues; [a change is specified before it is built](https://github.com/kalonji-tools/purba/blob/e2ccac8/docs/decisions/a-change-is-specified-before-it-is-built.md#decision-outcome) |
| native issue dependencies | load-bearing | 231 links on 153 issues; the frontier |
| sub-issues | config | 256 links under 14 maps |
| comment minimizing, and GraphQL `userContentEdits` | load-bearing | the supersede procedure in `AGENTS.md`, and the post pass |
| `wayfinder:*` labels, issue and pull request templates, CODEOWNERS | config | triage, review |
| Dependabot, CodeQL default setup | config | action pins; code scanning |
| github.com addresses in tracked files | cosmetic to load-bearing | about 170 point at purba's own issues and pull requests (82 distinct issues), most in the records |
| `crates/purba/src/lib.rs:125` prints the github.com address of the defect template | load-bearing | the report of an internal error |
| mise downloads nearly each tool from github.com releases | config | `mise.lock` holds 90 download URLs |

Not used: wiki, Pages, Projects, Discussions, releases, tags, webhooks.

## 2. What Codeberg and Forgejo offer

Forgejo v17.0.1, which Codeberg runs since 2026-11-07.

| purba's need | Codeberg or Forgejo | equal? | source | tier |
|---|---|---|---|---|
| hosting a project mostly written by an AI tool | § 2 (1) 7 forbids it | none | [TermsOfUse.md L50](https://codeberg.org/Codeberg/org/src/branch/main/TermsOfUse.md) | [A] |
| Linux amd64 CI | `codeberg-tiny`, `-small`, `-medium`: 1 to 4 CPUs, 2 to 10 min; public repos only | partial | [actions/meta README](https://codeberg.org/actions/meta) | [A] |
| Linux arm64, macOS CI | none hosted; Forgejo's runner is supported only on Linux | none | the same; runner v13.2.1 README | [A] |
| GitHub Actions syntax | "not designed to be compatible"; `permissions:` ignored; matrix, reusable workflows, cache, artifacts exist | partial | Forgejo docs | [A] |
| PyPI trusted publishing | not a supported provider; a long-lived API token instead | none | [docs.pypi.org, adding a publisher](https://docs.pypi.org/trusted-publishers/adding-a-publisher/) | [A] |
| required checks, approvals, no admin bypass, signed commits | yes | equal | Forgejo v17 | [A] |
| rebase-only history | a merge-style setting | partial | Forgejo v17 | [A] |
| CODEOWNERS | read from the root or `docs/`, not `.github/`; patterns are regexps | partial | Forgejo v17 | [A] |
| issue forms, labels, issue dependencies with a REST API | yes | equal | Forgejo v17 | [A] |
| issue types, sub-issues, comment hiding, GraphQL, an edit-history API | not found in v17.0.1 | none | grep of the v17.0.1 source; a bound, not proof | [A] |
| Dependabot, CodeQL | Renovate, self-run; no CodeQL | partial, none | Forgejo docs | [A] |
| a GitHub App token | Authorized Integrations (short-lived OIDC tokens) | partial | Forgejo v17 | [A] |
| a GitHub mirror | push mirrors to GitHub; Codeberg forbids pull mirrors from other hosts | partial | Codeberg docs | [A] |

## 3. What is lost

| loss | what purba would do instead |
|---|---|
| hosting, under § 2 (1) 7 | stay on GitHub, or use another host |
| macOS builds and tests | a self-hosted Mac, which the Forgejo runner does not support; or no macOS CI |
| PyPI trusted publishing | a long-lived token, kept as a secret |
| issue types, which mark a spec agreed | a label |
| sub-issues, which hold each map | dependencies or a label |
| comment hiding, and GraphQL for the post pass | a new supersede procedure |
| CodeQL | none |
| the review behind `Accepted-by:`, and the Checks API behind Sign-off | a rewrite of the signing flow and its records |

## 4. What is gained

| measure | Codeberg | GitHub | tier |
|---|---|---|---|
| dates with posted trouble | 29 in 11 months, from its Mastodon status account | githubstatus.com counts each component, Copilot included, so its totals do not compare | [A] |
| anonymous readers | 2026-06-17: said it kept signed-in users responsive at the cost of anonymous readers | 2026-11-08: `504`, then `503`, for hours; incident opened at 10:36Z | [A] |
| reports from projects after a move | 3 of 3 that wrote about it reported slowness or outages | — | [A] |

No source measures a reliability gain from the move.

## 5. What is restructured

| part | change | tier |
|---|---|---|
| CI | each of the 10 workflows rewritten for Forgejo Actions; macOS on a machine purba runs | [A] for the gap; [reasoning] for the work |
| the merge gate | Sign-off, `Accepted-by:` and the GitHub App token rewritten; the liability records rewritten | [A] |
| the process | the issue-type, sub-issue and minimize rules in `AGENTS.md` and the records rewritten; `gh` replaced by `fj` v0.7.0 | [A] |
| addresses | about 170 links to purba's own issues and pull requests in the records; the defect address in the binary | [A] |
| history | old issues stay on GitHub or move through an importer | [A] survey |
| mirrors | a push mirror to GitHub, if any | [A] |

## 6. What the move costs

The Terms of Use clause makes the move impossible as written. The rest of this section assumes it did not apply.
The estimates are reasoning, not measurement.

| item | work | tier |
|---|---|---|
| rewrite 10 workflows for Forgejo Actions, and test them | large | [reasoning] |
| macOS builds and tests | a Mac to buy or rent and run, with an unsupported runner mode | [A] for the gap; [reasoning] for the cost |
| the signing flow and its records | large: it rests on GitHub's review and Checks APIs | [reasoning] |
| the process: issue types, sub-issues, minimize, GraphQL | large: it touches `AGENTS.md`, several records and the agent's tooling | [reasoning] |
| PyPI publishing with a token | small, with a weaker security story | [A] |
| issues: 501 issues, 231 dependency links, 256 sub-issue links | an importer loses data in 5 of 6 surveyed moves; or leave them on GitHub | [A] survey |
| addresses in records and in the binary | medium | [A] count; [reasoning] work |

## 7. Other projects

16 cases: 13 projects and 3 individuals.

| measure | count |
|---|---|
| left GitHub, for Codeberg or a forge they run | 10 of 16 |
| weighed a move and stayed on GitHub | 3 of 16: curl, Gitea, CPython |
| came back after moving | 2 of 16: one person to GitHub; postmarketOS from SourceHut to GitLab |
| rewrote CI | 7 of 7 with a known CI story |
| had a gap in pull request CI after the move | 3 of 7 |
| lost macOS or Windows CI | 2 of 5; Zig kept it only because it already ran its own machines |
| kept a live GitHub mirror | 2 of 10 leavers |
| moved issues with an importer | 6 of 8; 5 of those 6 lost something |
| lost PyPI trusted publishing | 1 of 1 Python leaver |
| fewer stars on Codeberg than on GitHub | 4 of 4 measured, 1.3 % to 21 % |
| commit authors fell after the move | 1 of 4 measured |

- Zig's repository on GitHub reads "Moved to Codeberg", and 2,658 of its issues are still open there [A].
- curl stays partly because GitHub pays for its CI [A].

## What this research could not verify

- Whether Codeberg's moderators would apply § 2 (1) 7 to purba [C].
- The cost of a Mac for CI, and whether Forgejo's host mode runs on macOS [C].
- Codeberg's outages before 2025-12, which makes its count a lower bound [C].
- Org rulesets and org issue fields of GitHub, which need the `admin:org` scope [C].
- Links inside issue and pull request text: only tracked files were counted [C].
