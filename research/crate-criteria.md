# How projects decide that a crate may become a dependency

Research for [#958](https://github.com/kalonji-tools/purba/issues/958).
Fetched on 2026-11-13. This file states no criterion of its own.

## Summary

No surveyed source sets one threshold that a crate must reach.
The organisations that ship large Rust code bases (Chromium, Fuchsia, Firefox, Wasmtime, rustc)
gate a new crate on a **person's review**: an owner signs off, a board approves the licence,
or a trusted maintainer records an audit. The numbers and lists they write down cover weight
(binary size, checkout size), freshness and licences, not the crate’s quality.
The foundation guidance (OpenSSF) and the tools give numbers, but each window differs:
90 days (Scorecard), 12 months (OpenSSF guide), one idle year and 270 days without a reply
from the author (RustSec), 365 days (cargo-unmaintained), two years since a release (Chromium).
The evidence that a criterion predicts harm is thin. Two criteria have Rust-specific studies
behind them: `unsafe` code (memory-safety bugs need it) and known advisories.
The one study that tested security-practice scores against vulnerability counts
found they explain 9 % to 12 % of the variance, and that higher aggregate scores came
with *more* reported vulnerabilities (npm and PyPI, not crates.io).
Activity, maintainer count, popularity and licence clarity rest on argument or on incident
reports in the sources surveyed here. The sources openly disagree on whether low activity
disqualifies a crate, on whether one maintainer is a risk, and on how deep an `unsafe` review must go.

Five things the survey shows, each argued from more than one source:

1. **A human gate, not a score, admits a crate** in every organisation policy read (S15–S19).
   Scorecard itself says its aggregate "tells you nothing about what individual behaviors a repository is or is not doing" (S2).
2. **Maintenance windows do not agree**: 90 days, 12 months, 1 year, 365 days, 2 years (S1, S2, S6, S13, S16).
3. **Only `unsafe` and advisories have Rust-specific evidence** (S23–S26). Everything else is argument or incident.
4. **Tools flag; they do not decide.** cargo-geiger, Scorecard, cargo-deny and cargo-vet each say so in their own docs (S2, S8, S9, S12).
5. **Four of the five crates.io malware notices read describe a name that imitates a real crate**,
   and no surveyed tool checks names (S22, cargo-deny#421).

## How to read this file

- **Tiers.** [A] = fetched this session; the command is in [Appendix C](#appendix-c-commands).
  [B] = relayed by a fetched source, not re-verified. [C] = could not establish.
- **Sources** are numbered S1–S29 and pinned to a commit or tag in [Appendix B](#appendix-b-sources).
- **Liveness** (default branch's last commit + latest release) is in [Appendix A](#appendix-a-liveness).
- **Quotes** are verbatim. A quote that crossed a line break in the source is joined with one space.
  Inside a quote, a Markdown link keeps its text and drops its target.
- **Crate names.** This file names no library crate. Where a source names one, the text describes it instead.

## 1. The criteria

Which criteria do the sources apply before a crate joins? The table lists each criterion
and the sources that apply it. "Gate" means the source refuses or escalates on it;
"signal" means the source only asks the reviewer to weigh it.

| # | Criterion | Applied as a gate by | Applied as a signal by | Tier |
|---|---|---|---|---|
| C1 | Necessity: can the dependency be avoided | Chromium S16 (motivation and alternatives in the approval email) | OpenSSF guide S1, Fuchsia S17, Wasmtime S19 | [A] |
| C2 | Authenticity: right name, not a fork or typosquat | — | S1, S2C2F S4 (AUD-1), Rust Security Response WG S22 | [A] |
| C3 | No known vulnerability | cargo-audit S7, cargo-deny S8 (defaults fail), Microsoft S20 ("check-in gates") | S1, Scorecard S2, S4 (SCA-1) | [A] |
| C4 | Maintained / not abandoned | cargo-deny S8 (default fails on any `unmaintained` advisory), Chromium S16 (freshness) | S1, S2, S4 (SCA-3), RustSec S6, cargo-unmaintained S13, Fuchsia S17 | [A] |
| C5 | Recent release | Chromium S16 ("release within the last 2 years") | S1 (12 months) | [A] |
| C6 | More than one maintainer; several organisations | — | S1, S2 (Contributors), S17 (second-order) | [A] |
| C7 | Upstream reviews its own changes | — | S1, S2 (Code-Review) | [A] |
| C8 | Tests exist and pass | Firefox S18 ("reasonable tests" in the audit) | S1, S17 (first-order) | [A] |
| C9 | Licence on an allow list | rustc S15, Chromium S16 (gnrt), Fuchsia S17 (OSRB), Firefox S18, cargo-deny S8 | S1, S4 (SCA-2) | [A] |
| C10 | `unsafe` code is small, encapsulated, documented, reviewed | Chromium S16 (rule of 2), Google S10 (`ub-risk-N`), cargo-vet S9 (`safe-to-deploy`) | S17, S18, cargo-geiger S12 | [A] |
| C11 | Build-time code (`build.rs`, proc macros) | Chromium S16 (determinism check), Fuchsia S17 (`build.rs` not run) | cargo-deny S8 (`bans.build`), cargo-crev S11 (`CB` flag) | [A] |
| C12 | Not malicious; safe to run on a workstation | cargo-vet S9 (`safe-to-run`), Wasmtime S19, Firefox S18 | S1 (behaviour and sandbox testing) | [A] |
| C13 | Weight: transitive crates, binary and checkout size | Chromium S16 (16 kb, 20 MB prompts) | S1, S17 | [A] |
| C14 | Source and registry provenance | cargo-deny S8 (`sources`) | S4 (ING, ENF) | [A] |
| C15 | No duplicate versions | rustc S15 (runtime) | cargo-deny S8 (default warn) | [A] |
| C16 | Adoption / popularity | — | S1, S17 (reverse dependencies), S11 (downloads column) | [A] |
| C17 | Trust in the publisher | cargo-vet S9 (trusted entries), Wasmtime S19 | Chromium S16, S22, cargo-supply-chain S14 | [A] |
| C18 | Version and API stability (`0.x`, breaking changes) | — | S1 | [A] |
| C19 | Project security process (badge, policy, score) | — | S1, S2, Best Practices badge S5 | [A] |
| C20 | A named owner inside the consuming project | Chromium S16 (two committers), Fuchsia S17 (OWNERS +2) | S17 (asks whether the project would fork and maintain it) | [A] |

Two policies describe themselves as having no checklist.
Chromium's reviewers "explicitly avoid trying to codify what exactly to look for and how to do the reviews,
because ultimately we trust the reviewer’s judgement here" (S16) [A].
Firefox says: "Other than that, there is no formal sign-off procedure, but one may be added in the future." (S18) [A].

**The Rust project.** The only admission rule found from the Rust project is internal to rustc.
`tidy` keeps a licence allow list and a list of permitted rustc dependencies,
described as "a speed-bump to adding a new dependency to rustc. Please check with the compiler team before adding an entry."
(S15, `///` comment markers dropped) [A].
The Rust Security Response WG's one consumer-facing line is:
"In general, we recommend regularly auditing your dependencies, and only depending on crates whose author you trust." (S22, 2022) [A].
No Rust-project document that tells a *consumer* what a crate must meet was found
in the Cargo book or the Rust Forge (searches in Appendix C) [C].

## 2. The threshold

How does each source decide that a criterion is met? Four forms appear:
a **hard rule** (a list or a flag), a **score**, a **count or window**, and a **review by a person**.

| Source | Form | The numbers it states (verbatim where quoted) | Tier |
|---|---|---|---|
| S1 OpenSSF guide | Checklist, "Done" column; no pass mark | "within the previous 12 months" (activity); "the last release was issued within the previous 12 months"; "more than one maintainer" | [A] |
| S2 Scorecard `Maintained` | Score 0–10 | "If there is at least one commit per week during the previous 90 days, the project receives the highest score." Archived → lowest score. | [A] |
| S3 Scorecard `Maintained` code at v5.6.0 | Score | Proportional score of commits plus issue activity in 90 days against `activityPerWeek*lookBackDays/daysInOneWeek` (1 × 90 / 7, which is 12 in Go integer arithmetic). A repository created in the last 90 days gets the minimum score. | [A] |
| S2 Scorecard `Contributors` | Score | "at least 3 different companies in the last 30 commits; each of those contributors must have had at least 5 commits in the last 30 commits" | [A] |
| S2 Scorecard `Code-Review` | Score with deductions | "over the last ~30 commits"; "7 points are deducted if a single change is unreviewed, and another 3 are deducted if multiple changes are unreviewed"; 3 points for unreviewed bot changes | [A] |
| S2 Scorecard aggregate | Weighted mean | Critical 10, High 7.5, Medium 5, Low 2.5 | [A] |
| S4 S2C2F | Maturity levels L1–L4 | SCA-1 known vulnerabilities and SCA-2 licences at L1; SCA-3 end-of-life at L2; SCA-4 malware at L3 | [A] |
| S5 Best Practices badge | Self-certified form | "Projects that follow these best practices will be able to voluntarily self-certify" | [A] |
| S6 RustSec unmaintained | Rule + window + author's word | "Inactivity over a period of 1 year or more is the preferred threshold."; "Unresponsiveness by the author over a period of 270 days (or 60 days of unresponsiveness after being notified of a vulnerability) is the minimum" | [A] |
| S7 cargo-audit | Hard rule (advisory match) | Example config: `severity_threshold = "low"`; `deny = ["unmaintained"]` as an opt-in; database stale after 90 days | [A] |
| S8 cargo-deny | Hard rule per check, configurable level | Licence detection `confidence-threshold` "`0.0` - `1.0` (default `0.8`)"; advisory DB staleness "`P90D`, or 90 days" | [A] |
| S9 cargo-vet | Human audit against named criteria | Two built-in criteria; trusted-publisher entries "require a trust expiration date" | [A] |
| S10 Google `ub-risk-N` | Human audit, graded 0–4 | "Most projects that want to use this … should specify `ub-risk-2` as the policy criteria" (quoted in S9) | [A] |
| S11 cargo-crev | Count of trusted reviews | `--redundancy` "Number of reviews required [default: 1]"; thoroughness `low` "~2 minutes per file", `medium` "~15 minutes per file", `high` "hour or more per file" | [A] |
| S13 cargo-unmaintained | Heuristic + window | `--max-age` "[default: 365]" days | [A] |
| S15 rustc `tidy` | Hard rule (allow lists) | Licence list; permitted-dependency list; a new entry asks the author to "check with the compiler team" first | [A] |
| S16 Chromium | Review by people (TSC, Security, OWNERS, ATL) + freshness numbers | "If the expected APK increase is greater than 16kb, consider analyzing the library"; checkout "significant (e.g. >20MB)"; OWNERS file lists "two Chromium committers"; freshness: no newer minor/patch "available for 3+ months", no newer major "available for 1 year or more", "a release within the last 2 years"; per `unsafe` block, listed under "Not a goal": "Spending more than a minute or two per `unsafe` block" | [A] |
| S17 Fuchsia | Board approval (OSRB) + code review + OWNERS +2 | "Please allow about a week for a response."; "You must get a `Code Review Label +2` from one of the repository's owners." | [A] |
| S18 Firefox | Tool-enforced audit + licence check | "All vendored crates must be audited." | [A] |
| S19 Wasmtime | Tool-enforced audit by trusted people | "All dependencies are required to be "vetted" through the `cargo vet` tool"; "New vet entries are required to be made by trusted contributors" | [A] |
| S20 Microsoft | Tool in the check-in gate | Lists cargo-audit among tools that "should be used as part of check-in gates"; states no number | [A] |
| S21 Android | Not public | The public crate repository README describes tooling only; Chromium links Android's checklist as "[Google-internal]" | [C] |

Read as a whole, the organisations put their numbers on **cost** (size, freshness, review time),
and leave **quality** to a reviewer. The foundation and tool sources put numbers on **activity**.

## 3. The evidence

For each criterion: what evidence do the sources give that it predicts a dependency that harms a project?

| Criterion | Evidence found | Kind | Tier |
|---|---|---|---|
| C10 `unsafe` | Xu et al. surveyed 186 bug reports containing all Rust memory-safety CVEs to 2020-12-31: "Rust can keep its promise that all memory-safety bugs require unsafe code" (S25). Zheng et al. (433 vulnerabilities, 7 years): vulnerable code "contains statistically significantly more unsafe functions and blocks than the rest of the code", and "memory safety and concurrency issues account for nearly two thirds of the vulnerabilities" (S24). Evans et al.: `unsafe` appears in "less than 30% of Rust libraries, but more than half cannot be entirely statically checked … because of Unsafe Rust hidden somewhere in a library's call chain" (S26). | Empirical, Rust | [A] |
| C3 known vulnerabilities | Zheng et al.: "It takes over 2 years for the vulnerabilities to become publicly disclosed, and one third of the vulnerabilities have no fixes committed before their disclosure" (S24). This bounds what an advisory check can see. | Empirical, Rust | [A] |
| C4 maintenance | Zahan et al. (npm, PyPI): `Maintained`, Code Review, Branch Protection and Security Policy "were the most important practices influencing vulnerability count. However, we had low R^2 (ranging from 9% to 12%)" (S23). A Scorecard issue reports that, used as an abandonment detector, a `Maintained` score of 0 gave "roughly half to a third" false positives on hand-checked Java and JavaScript repositories (#5315). OpenSSF guide: "If it's unmaintained, it's also likely to be insecure." — no citation given (S1). | Empirical (not Rust); study relayed in an issue; argument | [A] study; [B] #5315's numbers; [A] that S1 gives no citation |
| C7 upstream code review | Same study: Code Review among the most important features, same low R^2 (S23). Depdive: updates of the top 1000 packages per registry, including crates.io, "are typically only partially code-reviewed (52.5% of the time)" and "only 9.0% of the packages had all their updates in our data set fully code-reviewed" (S27; LaTeX escapes removed). That measures prevalence, not harm. | Empirical (prevalence) | [A] |
| C19 aggregate score | "the number of reported vulnerabilities increased rather than reduced as the aggregate security score of the packages increased" (S23). | Empirical, not Rust | [A] |
| C16 popularity | Zheng et al.: "More popular packages tend to have more vulnerabilities, while the less popular packages suffer from vulnerabilities for more versions." The authors' reading: the larger community "tend to uncover more vulnerabilities" (S24). OpenSSF guide: widely used software means "more people will care about its security" (S1), no citation. | Empirical, Rust; argument | [A] |
| C2 authenticity / typosquatting | Five crates.io notices (2022–2026) were read. Four describe a crate that imitated a real crate's name, and four say the crate had no dependents on crates.io (S22). The crates.io team now says "In the vast majority of cases to date, these notifications have involved crates that have no evidence of real world usage" (S22, 2026-02-13). Backstabber's Knife Collection: 174 malicious packages, npm/PyPI/RubyGems only (S29). | Incident record, Rust; dataset not Rust | [A] |
| C11 build-time code | One notice: "The malicious code was executed at runtime … Notably, they did not execute any malicious code at build time." (S22). A build-script check alone would not have caught it. | Incident, Rust | [A] |
| C12 human audit | Cargo Scan: flagged effects cut the audit to "a median of 0.2% of lines of code when compared to auditing whole crates" (S28). That measures audit cost, not whether audits prevent harm. No study found that measures harm prevented by cargo-vet or cargo-crev audits. | Empirical (cost); none on effect | [A] cost; [C] effect |
| C6 maintainer count / diversity | None found. Scorecard rates `Contributors` risk "Low" (S2). OpenSSF guide links to an article that "many widely-used projects have one maintainer" (S1). | Argument | [A] no evidence in sources; [B] the linked article |
| C9 licence | OpenSSF guide: "Projects lacking clear licensing information frequently demonstrate deficiencies in other security best practices." No citation in the guide (S1). Licence is mostly a legal criterion in S4, S15–S18. | Argument | [A] claim; [C] support |
| C13 weight / dependency count | "Every new dependency increases the attack surface" (S1). Chromium cites "experience from Windows of the binary size impacting successful patch rate of updates" (S16). No study links count to harm. | Argument; one experience report | [A] |
| C5, C8, C18 recency, tests, `0.x` | Stated as checklist items with no supporting study (S1, S17). | Argument | [A] that none is cited; [C] support |
| C14 provenance | S2C2F maps incidents (dependency confusion, left-pad, account takeover) to practices (S4); the incidents are linked, not fetched. | Incident list | [B] |

The Rust-specific empirical record supports two criteria directly: `unsafe` code and known advisories.
The only study that tests practice scores against outcomes (S23) is on npm and PyPI,
and its own conclusion is that "additional factors may influence the package vulnerability count".
For the other criteria, the sources give argument, a checklist, or an incident.

## 4. What a tool decides

Which tool checks each criterion, under which name, and with what known false positives from its own tracker.
Issue states are as fetched on 2026-11-13. Issue text that names a crate is paraphrased.

| Criterion | Tool and check name | Default outcome | False positives / limits from the tool's own tracker or docs | Tier |
|---|---|---|---|---|
| C3 | cargo-audit `cargo audit` (RustSec DB) | Fails on vulnerabilities; config can ignore IDs | rustsec/rustsec#1119 (open): advisories for optional dependencies "pulled in by disabled features". #1184 (closed): a local crate with the same name as an advisory's crate is flagged. #21 (open, 2018): no call-graph filtering, so vulnerabilities in unused functions are reported. advisory-db#288 (closed): a consumer that uses the vulnerable API in a non-vulnerable way is still flagged. | [A] |
| C3 | cargo-deny `advisories` | Fails | cargo-deny#803 (closed): a "false-positive … edge, which triggers a security advisory". | [A] |
| C3 | Scorecard `Vulnerabilities` (OSV) | Score | Remediation offers an `osv-scanner.toml` ignore "If you believe the vulnerability does not affect your project" (S2). | [A] |
| C4 | cargo-deny `advisories.unmaintained` | "`all` (default) - Any crate that matches an unmaintained advisory will fail" | Inherits RustSec's definition (S6). | [A] |
| C4 | cargo-audit `informational_warnings` / `deny` | Example config makes `deny = ["unmaintained"]` an explicit choice | The default outcome is not stated in the fetched README or example. | [A] example; [C] default |
| C4 | cargo-unmaintained | Exit status 1 when found | README: a renamed package "is immediately considered unmaintained" (#441, open). #279 (open): flagged because an old version points at a repository that no longer exists. README counts 174 of 252 RustSec unmaintained advisories found (69 %) as of 2026-11-06, so 78 are missed; false negatives are documented. | [A] |
| C4 | Scorecard `Maintained` | Score 0–10 | ossf/scorecard#5315 (open): about a half to a third false positives when a 0 is read as "abandoned". The docs say "a lack of active maintenance is not necessarily always a problem" (S2). | [A] issue text; [B] its numbers |
| C6 | Scorecard `Contributors` | Score | ossf/scorecard#4175 (open): "a single maintainer who has joined many orgs can also pass the check". | [A] |
| C7 | Scorecard `Code-Review` | Score | ossf/scorecard#1826 (open): "on a 1-maintainer project, it shouldn't matter if changes are landed via a PR or not, because there's nobody to do code review"; the reporter warns of an incentive to make "a sock puppet account solely to code review all my changes". #1783 (open): review bypass via commits to local branches (title). | [A] |
| C9 | cargo-deny `licenses` | Licences not on `allow` "are denied by default" | #385, #181 (closed): licence file not found or clarification not applied (titles). | [A] titles |
| C9 | rustc `tidy` deps check | Fails | Not a public tool; exceptions "should be considered bugs" (S15). | [A] |
| C10 | cargo-geiger | Reports counts; no verdict | geiger#22 (open, 2018): `unsafe` reported in a small test case. geiger#11 (open, 2018): counts for a widely used parser crate exceed the real uses, per the reporter. #82 (open): a false negative (title). The README: "This tool is not meant to advise directly whether the code ultimately is truly insecure or not." | [A] |
| C10 | Google `ub-risk-0` automation | Reject on the string | "As a baseline, automation should reject any code containing the string `unsafe`." A person "**must** sign off" (S10). | [A] |
| C11 | cargo-deny `bans.build` | Configurable | Its own docs list what it "**WILL NOT DETECT**", including a crate that uploads SSH keys "using vanilla rust code" (S8). | [A] |
| C11 | cargo-crev `CB` flag | Column in `verify` | Flags presence only (S11). | [A] |
| C12, C17 | cargo-vet `cargo vet` | Fails until every crate is audited, trusted or exempted | `cargo vet init` exempts all existing dependencies by design (S9 FAQ). Chromium now calls its `cargo vet` items "Somewhat obsolete" (S16). No false-positive issue found by the search in Appendix C. | [A] |
| C12, C17 | cargo-crev `cargo crev verify` | `pass` only with enough trusted reviews | Default redundancy 1 (S11). No false-positive issue found by the search. | [A] |
| C14 | cargo-deny `sources` | `unknown-registry` and `unknown-git`: "`warn` (default)" | — | [A] |
| C15 | cargo-deny `bans.multiple-versions` | "`warn` (default)" | — | [A] |
| C2 | (none) | — | cargo-deny#421 (open since 2022-05-13) asks to "detect and warn/deny crates based on some heuristics" for typosquats. No surveyed tool has such a check. | [A] |
| C17 | cargo-supply-chain `publishers` | Lists; no verdict | — | [A] |

Every tool doc read here says, in its own words, that its output needs a person to judge it:
Scorecard ("The checks themselves are heuristics; there are false positives and false negatives."),
cargo-geiger (above), cargo-deny's build check (above), and cargo-vet's trusted entries
("Trust entries are fundamentally a heuristic.") [A].

## 5. Where the sources disagree

Each block shows the sources side by side. None is resolved here.

### D1. Does low activity disqualify a crate?

| Source | Position (verbatim) |
|---|---|
| S4 S2C2F SCA-3 | "For security purposes, no organization should take a dependency on software that is no longer receiving updates" |
| S1 OpenSSF guide | "Unmaintained software is a risk; most software needs continuous maintenance. If it's unmaintained, it's also likely to be insecure." |
| S2 Scorecard | "a lack of active maintenance is not necessarily always a problem. Some software, especially smaller utility functions, does not normally need to be maintained." |
| S6 RustSec | "Repository metrics like recent commits, open issues, latest release, etc are not reasons to go against the direct word of a crate author." |
| S17 Fuchsia | Activity is a second-order signal; "Missing second-order signals should not disqualify using a crate." |

### D2. How long a silence counts as unmaintained

| Source | Window |
|---|---|
| S2, S3 Scorecard | 90 days, about one activity per week for full score |
| S1 OpenSSF guide | 12 months for activity and for the last release |
| S13 cargo-unmaintained | 365 days (configurable), plus an outdated dependency or archived repository |
| S6 RustSec | 1 year inactive (preferred) **and** 270 days of no reply from the author |
| S16 Chromium | A release within 2 years; newer minor version not ignored for 3+ months |

### D3. Is a single maintainer a risk?

| Source | Position |
|---|---|
| S1 OpenSSF guide | "Verify the presence of more than one maintainer, ideally from different organizations" — and, in the same row, "note that many widely-used projects have one maintainer". |
| S2 Scorecard `Contributors` | Top score needs 3 companies; the docs add "Some projects cannot meet this requirement". |
| ossf/scorecard#1826 | "penalizing projects for something that's not in their control doesn't seem like it'll help security." |
| S17 Fuchsia | "Multiple maintainers" is a second-order signal that "should not disqualify". |

### D4. Is popularity a safety signal?

| Source | Position |
|---|---|
| S1 OpenSSF guide | "Widely-used software is more likely to offer useful information on how to use it securely and more people will care about its security." |
| S24 Zheng et al. | "More popular packages tend to have more vulnerabilities, while the less popular packages suffer from vulnerabilities for more versions." |
| S1 OpenSSF guide (same file) | "Don't choose it merely because it's used by large companies or because it's the latest fad." |

These two claims can both hold: more found and more fixed. The sources do not settle which effect dominates.

### D5. Trust the author, or audit the code?

| Source | Position |
|---|---|
| S22 Rust Security Response WG | "only depending on crates whose author you trust" |
| S9 cargo-vet | "it's usually impractical to require that every dependency was _developed_ by a trusted source, so the next best thing is to ensure that everything has been _audited_ by a trusted source." |
| S16 Chromium | "The review can be quite minimal or lightweight if Chromium already has a trust relationship with the crate authors." |
| S9 cargo-vet | "Trust entries are fundamentally a heuristic. The trusted publisher is not consulted and may or may not have personally authored or reviewed all the code." |

### D6. Is trust transitive?

| Source | Position |
|---|---|
| S11 cargo-crev | "`crev` recursively traverses the graph from the given root identity. This makes *trust* transitive." |
| S9 cargo-vet | "**No Web-of-Trust:** there is no notion of transitive trust." |

### D7. How deep must an `unsafe` review go?

| Source | Position |
|---|---|
| S16 Chromium checklist | Listed under "Not a goal": "Leaving a written trail that for each `unsafe` block explains reviewer's thinking why the safety requirements are met" and "Spending more than a minute or two per `unsafe` block" |
| S10 Google `ub-risk` (thorough) | "Explicitly justify the code in each unsafe block" and "Unless an unsafe Rust reviewer is very experienced, a group of two or more should perform thorough soundness reviews." |
| S9 cargo-vet `safe-to-deploy` | "they must review enough to fully reason about the behavior of all unsafe blocks and usage of powerful imports" |
| S12 cargo-geiger | "The use of unsafe is nuanced and necessary in some cases" |

S16 and S10 are both Google documents.

### D8. Can a number stand in for the judgement?

| Source | Position |
|---|---|
| S2 Scorecard goal | "Automate analysis and trust decisions on the security posture of open source projects." |
| S2 Scorecard non-goal | "Aggregate scores in particular tells you nothing about what individual behaviors a repository is or is not doing." |
| S5 Best Practices badge | Scorecard "focuses on what can be automatically measured (not necessarily what's important" |
| S17 Fuchsia | "No default outcome" — the principles apply to using a crate "**and** to the decision to implement ourselves". |
| S16 Chromium | "we trust the reviewer’s judgement here" |

### D9. Does a memory-safe crate need a security review?

| Source | Position |
|---|---|
| S16 Chromium | "Note that new Rust crates do not require security approval - the Rust third_party approval is sufficient." (other third-party code needs Chrome Security sign-off) |
| S17 Fuchsia | "On a project like Fuchsia, we cannot tolerate such risks and must review code that we use to ship our products." |
| S19 Wasmtime | A vet "is not a meticulous code review of a dependency for correctness but rather it is a statement that the crate does not contain malicious code" |

## 6. What could not be established [C]

- Android's crate admission criteria. The public repository has tooling only; the checklist is "[Google-internal]" (S16, S21).
- Whether Android's `main` branch is the active branch. Its last commit is 2025-03-26 (Appendix A).
- cargo-audit's default outcome for `unmaintained` advisories (S7 shows only an opt-in `deny`).
- Any study that measures harm prevented by human audits (cargo-vet, cargo-crev).
- Any evidence that maintainer count, organisation diversity, licence clarity, test presence or a `0.x` version predicts harm from a dependency.
- Any Rust-ecosystem study that tests Scorecard-style scores against outcomes (S23 covers npm and PyPI).
- A Rust-project document of consumer-facing admission criteria. The Cargo book and Rust Forge searches found none (Appendix C).
- A written Linux-kernel policy for admitting crates. The three `Documentation/rust/` files probed contain none.
- Whether Chromium's freshness rule ("a release within the last 2 years") is checked when a Rust crate is first imported, or only on updates.
- Which review Fuchsia means by "Code review is the most fundamental first-order signal": the upstream's, or Fuchsia's own.
- The underlying data of ossf/scorecard#5315 (a conference talk, not fetched).

## Appendix A: Liveness

Default branch's last commit and latest release, fetched 2026-11-13.
"Release" is a GitHub release when one exists, else the newest tag.

| Repository | Default branch @ last commit | Latest release |
|---|---|---|
| EmbarkStudios/cargo-deny | main @ 0fabb18da2cc 2026-11-12 | 0.20.2, 2026-07-09 |
| rustsec/rustsec (cargo-audit) | main @ 682f3a519d1d 2026-11-11 | cargo-audit/v0.23.0, 2026-11-10 |
| rustsec/advisory-db | main @ af7b3ec33460 2026-11-12 | no release, no tag |
| mozilla/cargo-vet | main @ cef4dd43a0b9 2026-10-22 | v0.10.3, 2026-10-22 |
| crev-dev/cargo-crev | main @ e0e71f9dc293 2026-11-06 | v0.27.1, 2026-04-12 |
| geiger-rs/cargo-geiger | master @ 118a9a281209 2026-10-12 | geiger-0.14.0, 2026-10-12 |
| trailofbits/cargo-unmaintained | main @ 41f8e2051cd3 2026-11-06 | v2.1.0, 2026-08-04 |
| ossf/scorecard | main @ 4ddc782c70eb 2026-11-12 | v5.6.0, 2026-10-12 |
| rust-secure-code/cargo-supply-chain | master @ ae1c240ace44 2026-10-15 | no GitHub release; tag v0.3.8, 2026-10-15 |
| ossf/wg-best-practices-os-developers | main @ c12573c64db7 2026-11-11 | no GitHub release; newest tag pyscg-v1.0.0, 2026-04-24 (not the guide's version) |
| ossf/s2c2f | main @ d0f0a7fbbc6c 2025-05-26 | v1.1, 2022-10-19 |
| coreinfrastructure/best-practices-badge | main @ 2406ace16ed6 2026-11-12 | sbom-production-20261106-100a704d, 2026-11-06 |
| google/rust-crate-audits | main @ 640a51eb32e2 2026-11-10 | no release, no tag |
| bytecodealliance/wasmtime | main @ 107a04c41ade 2026-11-13 | v50.0.1, 2026-10-29 |
| microsoft/rust-guidelines | main @ 9585e288d21d 2026-10-26 | no release, no tag |
| mozilla-firefox/firefox | main @ 2e185039d9be 2026-11-13 | Firefox releases are not GitHub releases [C] |
| rust-lang/rust | main @ 75bb7a374ff3 2026-11-13 | 1.100.0, 2026-11-12 |
| rust-lang/blog.rust-lang.org | main @ 875370d85901 2026-11-12 | no release, no tag |
| chromium/src (googlesource) | main @ 678035cfddd3 2026-11-13 | not applicable |
| fuchsia (googlesource) | main @ ed20606b2b9e 2026-11-13 | not applicable |
| platform/external/rust/android-crates-io | main @ 5a35dba89286 2025-03-26 | not applicable |

## Appendix B: Sources

Each link is pinned to the commit or tag that was fetched.

| ID | Source | Pinned link |
|---|---|---|
| S1 | OpenSSF, *Concise Guide for Evaluating Open Source Software* | [wg-best-practices-os-developers@c12573c](https://github.com/ossf/wg-best-practices-os-developers/blob/c12573c64db763bdba99f79017a8cb3da419fdbb/docs/Concise-Guide-for-Evaluating-Open-Source-Software.md) |
| S2 | OpenSSF Scorecard, check docs and README | [checks.md@4ddc782](https://github.com/ossf/scorecard/blob/4ddc782c70ebba1d65f0dd1b3470f24ed4ea18b1/docs/checks.md), [README.md@4ddc782](https://github.com/ossf/scorecard/blob/4ddc782c70ebba1d65f0dd1b3470f24ed4ea18b1/README.md) |
| S3 | Scorecard `Maintained` evaluation, tag v5.6.0 | [evaluation/maintained.go@v5.6.0](https://github.com/ossf/scorecard/blob/v5.6.0/checks/evaluation/maintained.go) |
| S4 | OpenSSF S2C2F framework | [framework.md@d0f0a7f](https://github.com/ossf/s2c2f/blob/d0f0a7fbbc6cc6cb6a248cbc9e98c3d8cf3b189a/specification/framework.md) |
| S5 | OpenSSF Best Practices badge criteria; badge vs. Scorecards | [criteria.md@2406ace](https://github.com/coreinfrastructure/best-practices-badge/blob/2406ace16ed6e36ef6aac6d3c894246ebda5a91c/docs/criteria.md), [badge-vs-scorecards.md@2406ace](https://github.com/coreinfrastructure/best-practices-badge/blob/2406ace16ed6e36ef6aac6d3c894246ebda5a91c/docs/badge-vs-scorecards.md) |
| S6 | RustSec unmaintained policy, contributing guide, maintainers' guide | [HOWTO_UNMAINTAINED.md@af7b3ec](https://github.com/rustsec/advisory-db/blob/af7b3ec33460b8e2de8d422e16d628518891a519/HOWTO_UNMAINTAINED.md), [CONTRIBUTING.md@af7b3ec](https://github.com/rustsec/advisory-db/blob/af7b3ec33460b8e2de8d422e16d628518891a519/CONTRIBUTING.md), [MAINTAINERS_GUIDE.md@af7b3ec](https://github.com/rustsec/advisory-db/blob/af7b3ec33460b8e2de8d422e16d628518891a519/MAINTAINERS_GUIDE.md) |
| S7 | cargo-audit README and example config | [README.md@682f3a5](https://github.com/rustsec/rustsec/blob/682f3a519d1d14ecd07b49f135e992fe39761545/cargo-audit/README.md), [audit.toml.example@682f3a5](https://github.com/rustsec/rustsec/blob/682f3a519d1d14ecd07b49f135e992fe39761545/cargo-audit/audit.toml.example) |
| S8 | cargo-deny check docs | [docs/src/checks@0fabb18](https://github.com/EmbarkStudios/cargo-deny/tree/0fabb18da2cc4407387382b6991e144edf4e48a8/docs/src/checks) |
| S9 | cargo-vet book and built-in criteria | [book/src@cef4dd4](https://github.com/mozilla/cargo-vet/tree/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src), [src/criteria@cef4dd4](https://github.com/mozilla/cargo-vet/tree/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/src/criteria) |
| S10 | Google, *Rust Crate Auditing Standards* | [auditing_standards.md@640a51e](https://github.com/google/rust-crate-audits/blob/640a51eb32e2f47d7ee8017a128485f7e1ce6780/auditing_standards.md) |
| S11 | cargo-crev docs | [trust.md@e0e71f9](https://github.com/crev-dev/cargo-crev/blob/e0e71f9dc2938c061ae0e99554caa8928d2299f1/cargo-crev/src/doc/trust.md), [getting_started.md@e0e71f9](https://github.com/crev-dev/cargo-crev/blob/e0e71f9dc2938c061ae0e99554caa8928d2299f1/cargo-crev/src/doc/getting_started.md), [editing-package-review.md@e0e71f9](https://github.com/crev-dev/cargo-crev/blob/e0e71f9dc2938c061ae0e99554caa8928d2299f1/crev-lib/rc/doc/editing-package-review.md) |
| S12 | cargo-geiger README | [README.md@118a9a2](https://github.com/geiger-rs/cargo-geiger/blob/118a9a2812098ad757c9174ad00be21deaea7e0d/README.md) |
| S13 | cargo-unmaintained README | [README.md@41f8e20](https://github.com/trailofbits/cargo-unmaintained/blob/41f8e2051cd3dca04cce58d49060688436be2b7f/README.md) |
| S14 | cargo-supply-chain README | [README.md@ae1c240](https://github.com/rust-secure-code/cargo-supply-chain/blob/ae1c240ace44b55ba01ac732f056eb632118e655/README.md) |
| S15 | rustc `tidy` dependency check, tag 1.100.0 | [deps.rs@1.100.0](https://github.com/rust-lang/rust/blob/1.100.0/src/tools/tidy/src/deps.rs) |
| S16 | Chromium third-party and Rust crate policy | [adding_to_third_party.md](https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/docs/adding_to_third_party.md), [README-importing-new-crates.md](https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/third_party/rust/README-importing-new-crates.md), [OWNERS-review-checklist.md](https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/third_party/rust/OWNERS-review-checklist.md), [third_party/rust/README.md](https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/third_party/rust/README.md), [rule-of-2.md](https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/docs/security/rule-of-2.md) |
| S17 | Fuchsia external crates and review process | [external_crates.md](https://fuchsia.googlesource.com/fuchsia/+/ed20606b2b9e597f4ce90fe6c70e128cfc9c5c18/docs/development/languages/rust/external_crates.md), [external_crates/review.md](https://fuchsia.googlesource.com/fuchsia/+/ed20606b2b9e597f4ce90fe6c70e128cfc9c5c18/docs/development/languages/rust/external_crates/review.md) |
| S18 | Firefox Rust build docs and `cargo vet` config | [build/docs/rust.md@2e18503](https://github.com/mozilla-firefox/firefox/blob/2e185039d9be31b6a60f9c002144e7ee0dd22762/build/docs/rust.md), [supply-chain/config.toml@2e18503](https://github.com/mozilla-firefox/firefox/blob/2e185039d9be31b6a60f9c002144e7ee0dd22762/supply-chain/config.toml) |
| S19 | Wasmtime coding guidelines, "Dependencies of Wasmtime" | [contributing-coding-guidelines.md@107a04c](https://github.com/bytecodealliance/wasmtime/blob/107a04c41adeedd3f20dc5dffcb783561c38846a/docs/contributing-coding-guidelines.md) |
| S20 | Microsoft Pragmatic Rust Guidelines, M-STATIC-VERIFICATION | [M-STATIC-VERIFICATION.md@9585e28](https://github.com/microsoft/rust-guidelines/blob/9585e288d21d2da994a0550c926a09b62d79f8bd/src/guidelines/universal/M-STATIC-VERIFICATION.md) |
| S21 | Android `android-crates-io` README | [README.md](https://android.googlesource.com/platform/external/rust/android-crates-io/+/5a35dba892863c34b659e63bef90f6fabf342edb/README.md) |
| S22 | Rust blog: five malicious-crate notices, 2022–2026 | [content@875370d](https://github.com/rust-lang/blog.rust-lang.org/tree/875370d859011a694efcd3111ce8da8ce57d72a9/content): `malicious-crate-rustdecimal.md`, `crates.io-malicious-crates-fasterlog-and-asyncprintln.md`, `crates.io-malicious-crates-evm-units-and-uniswap-utils.md`, `crates.io-malicious-crates-finch-rust-and-sha-rust.md`, `crates.io-malicious-crate-update.md` |
| S23 | Zahan et al., *Do Software Security Practices Yield Fewer Vulnerabilities?* (2022) | [arXiv:2210.14884](https://arxiv.org/abs/2210.14884) (abstract) |
| S24 | Zheng et al., *A Closer Look at the Security Risks in the Rust Ecosystem* (2023) | [arXiv:2308.15046v1](https://arxiv.org/abs/2308.15046v1) (abstract and full text) |
| S25 | Xu et al., *Memory-Safety Challenge Considered Solved? An In-Depth Study with All Rust CVEs* | [arXiv:2003.03296v6](https://arxiv.org/abs/2003.03296v6) (abstract) |
| S26 | Evans et al., *Is Rust Used Safely by Software Developers?* (2020) | [arXiv:2007.00752](https://arxiv.org/abs/2007.00752) (abstract) |
| S27 | Imtiaz and Williams, *Are your dependencies code reviewed?* (2022) | [arXiv:2206.09422](https://arxiv.org/abs/2206.09422) (abstract) |
| S28 | Zoghbi et al., *Auditing Rust Crates Effectively* (Cargo Scan, 2026) | [arXiv:2602.06466](https://arxiv.org/abs/2602.06466) (abstract) |
| S29 | Ohm et al., *Backstabber's Knife Collection* (2020) | [arXiv:2005.09535](https://arxiv.org/abs/2005.09535) (abstract) |

Tool issues cited in §3–§5 (fetched with the commands in Appendix C):
[ossf/scorecard#5315](https://github.com/ossf/scorecard/issues/5315),
[#1826](https://github.com/ossf/scorecard/issues/1826),
[#4175](https://github.com/ossf/scorecard/issues/4175),
[#1783](https://github.com/ossf/scorecard/issues/1783),
[rustsec/rustsec#1119](https://github.com/rustsec/rustsec/issues/1119),
[#1184](https://github.com/rustsec/rustsec/issues/1184),
[#21](https://github.com/rustsec/rustsec/issues/21),
[rustsec/advisory-db#288](https://github.com/rustsec/advisory-db/issues/288),
[EmbarkStudios/cargo-deny#421](https://github.com/EmbarkStudios/cargo-deny/issues/421),
[#803](https://github.com/EmbarkStudios/cargo-deny/issues/803),
[#385](https://github.com/EmbarkStudios/cargo-deny/issues/385),
[#181](https://github.com/EmbarkStudios/cargo-deny/issues/181),
[trailofbits/cargo-unmaintained#441](https://github.com/trailofbits/cargo-unmaintained/issues/441),
[#279](https://github.com/trailofbits/cargo-unmaintained/issues/279),
[geiger-rs/cargo-geiger#22](https://github.com/geiger-rs/cargo-geiger/issues/22),
[#11](https://github.com/geiger-rs/cargo-geiger/issues/11),
[#82](https://github.com/geiger-rs/cargo-geiger/issues/82).

## Appendix C: Commands

Downloads were saved outside the repository and read as data.
GitHub files were pinned to the default branch's head SHA, resolved with:

```sh
br=$(gh api "repos/$repo" --jq .default_branch)
sha=$(gh api "repos/$repo/commits/$br" --jq .sha)
curl -sSL -o <file> "https://raw.githubusercontent.com/$repo/$sha/$path"
```

Gitiles files were pinned the same way:

```sh
sha=$(curl -sSL "https://$host/$project/+/refs/heads/main?format=JSON" | tail -n +2 | jq -r .commit)
curl -sSL "https://$host/$project/+/$sha/$path?format=TEXT" | base64 -d
```

Liveness (Appendix A), per repository:

```sh
gh api "repos/$repo/commits/$br" --jq '.sha[:12]+" "+.commit.committer.date'
gh api "repos/$repo/releases/latest" --jq '.tag_name+" "+.published_at'
# when there is no release:
gh api "repos/$repo/tags?per_page=1" --jq '.[0].name'
```

The full text of S24 was converted with `nix-shell -p poppler-utils --run "pdftotext -layout 2308.15046.pdf 2308.15046.txt"`.

Searches whose empty result supports a [C] row:

```sh
gh search code "cargo vet" --repo rust-lang/cargo --limit 10 --json path      # no src/doc hit
gh search code "cargo-audit" --repo rust-lang/cargo --limit 10 --json path    # no src/doc hit
gh search code "supply chain" --repo rust-lang/cargo --limit 10 --json path   # no src/doc hit
gh search code "dependencies" --repo rust-lang/rust-forge --limit 20 --json path
for f in general-information.rst quick-start.rst index.rst; do
  curl -sSL "https://raw.githubusercontent.com/torvalds/linux/master/Documentation/rust/$f" \
    | grep -n -i -E 'crates.io|third.party|vendored|external crate|dependenc'
done                                                                            # no hit
```

Every fetch, in order:

```text
curl -sSL -o <file> https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0fabb18da2cc4407387382b6991e144edf4e48a8/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0fabb18da2cc4407387382b6991e144edf4e48a8/docs/src/checks/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0fabb18da2cc4407387382b6991e144edf4e48a8/docs/src/checks/advisories/cfg.md
curl -sSL -o <file> https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0fabb18da2cc4407387382b6991e144edf4e48a8/docs/src/checks/advisories/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0fabb18da2cc4407387382b6991e144edf4e48a8/docs/src/checks/bans/cfg.md
curl -sSL -o <file> https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0fabb18da2cc4407387382b6991e144edf4e48a8/docs/src/checks/bans/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0fabb18da2cc4407387382b6991e144edf4e48a8/docs/src/checks/licenses/cfg.md
curl -sSL -o <file> https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0fabb18da2cc4407387382b6991e144edf4e48a8/docs/src/checks/licenses/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0fabb18da2cc4407387382b6991e144edf4e48a8/docs/src/checks/sources/cfg.md
curl -sSL -o <file> https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/0fabb18da2cc4407387382b6991e144edf4e48a8/docs/src/checks/sources/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/rustsec/rustsec/682f3a519d1d14ecd07b49f135e992fe39761545/cargo-audit/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/rustsec/advisory-db/af7b3ec33460b8e2de8d422e16d628518891a519/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/rustsec/advisory-db/af7b3ec33460b8e2de8d422e16d628518891a519/CONTRIBUTING.md
curl -sSL -o <file> https://raw.githubusercontent.com/rustsec/advisory-db/af7b3ec33460b8e2de8d422e16d628518891a519/MAINTAINERS_GUIDE.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src/motivation.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src/audit-criteria.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src/built-in-criteria.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src/performing-audits.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src/trusting-publishers.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src/specifying-policies.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src/faq.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src/how-it-works.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src/importing-audits.md
curl -sSL -o <file> https://raw.githubusercontent.com/crev-dev/cargo-crev/e0e71f9dc2938c061ae0e99554caa8928d2299f1/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/crev-dev/cargo-crev/e0e71f9dc2938c061ae0e99554caa8928d2299f1/cargo-crev/src/doc/trust.md
curl -sSL -o <file> https://raw.githubusercontent.com/crev-dev/cargo-crev/e0e71f9dc2938c061ae0e99554caa8928d2299f1/cargo-crev/src/doc/getting_started.md
curl -sSL -o <file> https://raw.githubusercontent.com/crev-dev/cargo-crev/e0e71f9dc2938c061ae0e99554caa8928d2299f1/cargo-crev/src/doc/glossary.md
curl -sSL -o <file> https://raw.githubusercontent.com/crev-dev/cargo-crev/e0e71f9dc2938c061ae0e99554caa8928d2299f1/crev-lib/rc/doc/editing-package-review.md
curl -sSL -o <file> https://raw.githubusercontent.com/geiger-rs/cargo-geiger/118a9a2812098ad757c9174ad00be21deaea7e0d/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/geiger-rs/cargo-geiger/118a9a2812098ad757c9174ad00be21deaea7e0d/cargo-geiger/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/trailofbits/cargo-unmaintained/41f8e2051cd3dca04cce58d49060688436be2b7f/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/ossf/scorecard/4ddc782c70ebba1d65f0dd1b3470f24ed4ea18b1/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/ossf/scorecard/4ddc782c70ebba1d65f0dd1b3470f24ed4ea18b1/docs/checks.md
curl -sSL -o <file> https://raw.githubusercontent.com/ossf/scorecard/4ddc782c70ebba1d65f0dd1b3470f24ed4ea18b1/docs/faq.md
curl -sSL -o <file> https://raw.githubusercontent.com/rust-secure-code/cargo-supply-chain/ae1c240ace44b55ba01ac732f056eb632118e655/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/ossf/wg-best-practices-os-developers/c12573c64db763bdba99f79017a8cb3da419fdbb/docs/Concise-Guide-for-Evaluating-Open-Source-Software.md
curl -sSL -o <file> https://raw.githubusercontent.com/ossf/s2c2f/d0f0a7fbbc6cc6cb6a248cbc9e98c3d8cf3b189a/specification/framework.md
curl -sSL -o <file> https://raw.githubusercontent.com/ossf/s2c2f/d0f0a7fbbc6cc6cb6a248cbc9e98c3d8cf3b189a/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/coreinfrastructure/best-practices-badge/2406ace16ed6e36ef6aac6d3c894246ebda5a91c/docs/criteria.md
curl -sSL -o <file> https://raw.githubusercontent.com/coreinfrastructure/best-practices-badge/2406ace16ed6e36ef6aac6d3c894246ebda5a91c/docs/badge-vs-scorecards.md
curl -sSL -o <file> https://raw.githubusercontent.com/google/rust-crate-audits/640a51eb32e2f47d7ee8017a128485f7e1ce6780/README.md
curl -sSL -o <file> https://raw.githubusercontent.com/google/rust-crate-audits/640a51eb32e2f47d7ee8017a128485f7e1ce6780/auditing_standards.md
curl -sSL -o <file> https://raw.githubusercontent.com/bytecodealliance/wasmtime/107a04c41adeedd3f20dc5dffcb783561c38846a/docs/contributing-coding-guidelines.md
curl -sSL -o <file> https://raw.githubusercontent.com/bytecodealliance/wasmtime/107a04c41adeedd3f20dc5dffcb783561c38846a/docs/contributing-code-review.md
curl -sSL -o <file> https://raw.githubusercontent.com/bytecodealliance/wasmtime/107a04c41adeedd3f20dc5dffcb783561c38846a/docs/contributing-maintainer-guidelines.md
curl -sSL -o <file> https://raw.githubusercontent.com/microsoft/rust-guidelines/9585e288d21d2da994a0550c926a09b62d79f8bd/src/guidelines/universal/M-STATIC-VERIFICATION.md
curl -sSL -o <file> https://raw.githubusercontent.com/microsoft/rust-guidelines/9585e288d21d2da994a0550c926a09b62d79f8bd/src/guidelines/universal/M-SMALLER-CRATES.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/src/criteria/safe-to-run.txt
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla/cargo-vet/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/src/criteria/safe-to-deploy.txt
curl -sSL -o <file> https://raw.githubusercontent.com/rustsec/advisory-db/af7b3ec33460b8e2de8d422e16d628518891a519/HOWTO_UNMAINTAINED.md
curl -sSL 'https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/docs/adding_to_third_party.md?format=TEXT' | base64 -d
curl -sSL 'https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/third_party/rust/README.md?format=TEXT' | base64 -d
curl -sSL 'https://fuchsia.googlesource.com/fuchsia/+/ed20606b2b9e597f4ce90fe6c70e128cfc9c5c18/docs/development/languages/rust/external_crates.md?format=TEXT' | base64 -d
curl -sSL 'https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/third_party/rust/README-importing-new-crates.md?format=TEXT' | base64 -d
curl -sSL 'https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/docs/rust/README.md?format=TEXT' | base64 -d
curl -sSL 'https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/docs/rust/unsafe.md?format=TEXT' | base64 -d
curl -sSL 'https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/third_party/rust/OWNERS-review-checklist.md?format=TEXT' | base64 -d
curl -sSL 'https://chromium.googlesource.com/chromium/src/+/678035cfddd371a6a71c95859832e948cbcd7859/docs/security/rule-of-2.md?format=TEXT' | base64 -d
curl -sSL 'https://fuchsia.googlesource.com/fuchsia/+/ed20606b2b9e597f4ce90fe6c70e128cfc9c5c18/docs/development/languages/rust/external_crates/review.md?format=TEXT' | base64 -d
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla-firefox/firefox/2e185039d9be31b6a60f9c002144e7ee0dd22762/build/docs/rust.md
curl -sSL -o <file> https://raw.githubusercontent.com/mozilla-firefox/firefox/2e185039d9be31b6a60f9c002144e7ee0dd22762/supply-chain/config.toml
curl -sSL -o deps.rs https://raw.githubusercontent.com/rust-lang/rust/1.100.0/src/tools/tidy/src/deps.rs
curl -sSL 'https://android.googlesource.com/platform/external/rust/android-crates-io/+/5a35dba892863c34b659e63bef90f6fabf342edb/README.md?format=TEXT' | base64 -d
curl -sSL 'http://export.arxiv.org/api/query?id_list=2208.03412,2003.03296,2005.09535,2007.00752'
curl -sSL 'http://export.arxiv.org/api/query?id_list=2210.14884'
curl -sSL 'http://export.arxiv.org/api/query?search_query=abs:Rust+AND+abs:crates+AND+abs:vulnerabilit*&max_results=25'
curl -sSL 'http://export.arxiv.org/api/query?search_query=all:%22crates.io%22&max_results=40'
curl -sSL 'http://export.arxiv.org/api/query?id_list=2308.15046,2206.09422,2602.06466'
curl -sSL -o 2308.15046.pdf https://arxiv.org/pdf/2308.15046v1
curl -sSL -o <file> https://raw.githubusercontent.com/rust-lang/blog.rust-lang.org/875370d859011a694efcd3111ce8da8ce57d72a9/content/malicious-crate-rustdecimal.md
curl -sSL -o <file> https://raw.githubusercontent.com/rust-lang/blog.rust-lang.org/875370d859011a694efcd3111ce8da8ce57d72a9/content/crates.io-malicious-crates-fasterlog-and-asyncprintln.md
curl -sSL -o <file> https://raw.githubusercontent.com/rust-lang/blog.rust-lang.org/875370d859011a694efcd3111ce8da8ce57d72a9/content/crates.io-malicious-crates-finch-rust-and-sha-rust.md
curl -sSL -o <file> https://raw.githubusercontent.com/rust-lang/blog.rust-lang.org/875370d859011a694efcd3111ce8da8ce57d72a9/content/crates.io-malicious-crates-evm-units-and-uniswap-utils.md
curl -sSL -o <file> https://raw.githubusercontent.com/rust-lang/blog.rust-lang.org/875370d859011a694efcd3111ce8da8ce57d72a9/content/crates.io-malicious-crate-update.md
gh issue list --repo ossf/scorecard --state all --search 'false positive' --limit 60 --json number,title,state,createdAt,url
gh issue list --repo EmbarkStudios/cargo-deny --state all --search 'false positive' --limit 60 --json number,title,state,createdAt,url
gh issue list --repo trailofbits/cargo-unmaintained --state all --search 'false positive' --limit 60 --json number,title,state,createdAt,url
gh issue list --repo geiger-rs/cargo-geiger --state all --search 'false positive' --limit 60 --json number,title,state,createdAt,url
gh issue list --repo rustsec/rustsec --state all --search 'false positive' --limit 60 --json number,title,state,createdAt,url
gh issue list --repo mozilla/cargo-vet --state all --search 'false positive' --limit 60 --json number,title,state,createdAt,url
gh issue list --repo crev-dev/cargo-crev --state all --search 'false positive' --limit 60 --json number,title,state,createdAt,url
gh issue list --repo rustsec/advisory-db --state all --search 'false positive' --limit 60 --json number,title,state,createdAt,url
gh issue view 5315 --repo ossf/scorecard --json number,title,state,createdAt,body,url
gh issue view 1826 --repo ossf/scorecard --json number,title,state,createdAt,body,url
gh issue view 1907 --repo ossf/scorecard --json number,title,state,createdAt,body,url
gh issue view 4175 --repo ossf/scorecard --json number,title,state,createdAt,body,url
gh issue view 22 --repo geiger-rs/cargo-geiger --json number,title,state,createdAt,body,url
gh issue view 11 --repo geiger-rs/cargo-geiger --json number,title,state,createdAt,body,url
gh issue view 1119 --repo rustsec/rustsec --json number,title,state,createdAt,body,url
gh issue view 21 --repo rustsec/rustsec --json number,title,state,createdAt,body,url
gh issue view 1184 --repo rustsec/rustsec --json number,title,state,createdAt,body,url
gh issue view 279 --repo trailofbits/cargo-unmaintained --json number,title,state,createdAt,body,url
gh issue view 421 --repo EmbarkStudios/cargo-deny --json number,title,state,createdAt,body,url
gh issue view 288 --repo rustsec/advisory-db --json number,title,state,createdAt,body,url
curl -sSL -o evaluation_maintained.go https://raw.githubusercontent.com/ossf/scorecard/v5.6.0/checks/evaluation/maintained.go
curl -sSL -o raw_maintained.go https://raw.githubusercontent.com/ossf/scorecard/v5.6.0/checks/raw/maintained.go
curl -sSL -o <file> https://raw.githubusercontent.com/rust-lang/rust-forge/352432a501af75d9b35b072140ad381a456d4154/src/compiler/third-party-out-of-tree.md
curl -sSL -o <file> https://raw.githubusercontent.com/rust-lang/rust-forge/352432a501af75d9b35b072140ad381a456d4154/src/policies/crate-ownership.md
curl -sSL -o <file> https://raw.githubusercontent.com/rustsec/rustsec/682f3a519d1d14ecd07b49f135e992fe39761545/cargo-audit/audit.toml.example
```
