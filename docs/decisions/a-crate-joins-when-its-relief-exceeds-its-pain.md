# A crate joins when its relief exceeds its pain

## Context and Problem Statement

purba depends on crates that a manifest names.
Before this record, nothing said what a crate must meet to join.
No record named libc, proptest or pulldown-cmark.

| fact | source |
|---|---|
| no source sets one threshold; the organisations let a crate in after a person reviews it, and their numbers limit cost | [How do established projects and organisations decide that a crate may become a dependency, and what threshold must it reach?](https://github.com/kalonji-tools/purba/issues/958#issuecomment-6848867938) |
| only `unsafe` code and known advisories have studies in Rust that tie them to harm | same |
| the windows for "unmaintained" disagree, from 90 days to 2 years | same |
| the licence gate checks licences and nothing else about a crate | [`.cargo/deny.toml`](https://github.com/kalonji-tools/purba/blob/2790c1e/.cargo/deny.toml) |
| two records say why a crate joined: ruff's parser and harper's tagger | [purba parses Python with ruff's parser](purba-parses-python-with-ruff.md), [a part-of-speech tagger decides the tense rule](a-part-of-speech-tagger-decides-the-tense-rule.md) |
| a score from levels 1 to 3 already orders the design | [a design in depth waits for what it assumes](a-design-in-depth-waits-for-what-it-assumes.md#decision-outcome) |
| `cargo-deny` gives the advisory, licence and source checks; for a factor, a tool gives at most a number that a script must map to a level | [Which criteria of a crate can a tool decide, and which tool decides each?](https://github.com/kalonji-tools/purba/issues/975#issuecomment-6878233468) |

A person calibrated the formula on the crates of `2790c1e`.

| run | lowest pain | highest pain |
|---|:-:|:-:|
| first | 1.36 | 2.50 |
| after seven changes to the levels | 1.29 (pulldown-cmark) | 2.57 (harper) |

A probe measured upkeep and API churn again for each past week.

| weeks measured, to 2026-11-14 | a level moved | a verdict moved |
|:-:|:-:|:-:|
| 52 | 16 times | 0 times |

## Considered Options

- **No criteria: a crate joins with its own record, or with none, and only the licence gate checks it.** Nothing says what a crate must meet.
- **A person reviews each crate and decides, as the organisations do.** The architect chose a formula: "this will keep the ruling consistent".
- **A blast area that is too small also counts as pain.** A crate that reaches little also spares little, and relief already scores that.
- **A weekly workflow measures upkeep, maintainers, upstream review, tests and API churn again.** It opens an issue when a verdict would move or the owners change. It judges only the crates that purba already holds. [The comment that closed it](https://github.com/kalonji-tools/purba/issues/981#issuecomment-6883556794) gives the reason: [the map](https://github.com/kalonji-tools/purba/issues/957) looked for crates that can make purba better. In 52 weeks, no verdict moved.
- **A fixed interval: `lint:crates` refuses an entry whose `judged` date passes a fixed age.** Today's date becomes an input of the lint, and a pull request turns red on the date alone.
- **A change to the lock judges only what the change moves: `unsafe`, blast area and the checks.** `judged` no longer dates every level. Nobody reads `owners` again.

## Decision Outcome

**Reach:** `/CONTEXT.md` `.config/crates.toml` `.config/tasks.toml` `.cargo/deny.toml` `.cargo/check-prose.deny.toml` `scripts/crate-verdict.sh` `scripts/test/crate-verdict.bats`

**Area:** design, tooling

**A crate from outside purba that a manifest names joins only when it passes four checks and its [relief](../../CONTEXT.md#relief) exceeds its [pain](../../CONTEXT.md#pain).**
A task calculates the verdict, so each verdict is the same for the same levels.

The record judges each crate from outside purba that a manifest names.
Such a crate is a dependency or a dev dependency of a crate under `crates/`, or a dependency of a cargo script under `scripts/`.
The gates alone judge a crate that only arrives through another crate.
The record judges crates only.
Only the `unsafe` factor is specific to Rust, so a later record can apply this one to another kind of dependency.

**Four checks pass or fail.** One failure puts the crate out, unless a [waiver](../../CONTEXT.md#waiver) lets it in.

| check | passes when |
|---|---|
| known advisory | no RustSec advisory names the locked version |
| licence | the licence is on the allow list of `.cargo/deny.toml`, or that file has an exception for the crate |
| source | the crate comes from crates.io, not from a git repository |
| trust | a person confirms that the name, the repository and the publisher are those of the project that the pull request means to add, and crates.io shows no notice of malicious code |

`cargo-deny` checks the licence only of a crate that reaches the wheel, as [a script that needs a Rust library is a cargo script](a-script-that-needs-a-rust-library-is-a-cargo-script.md) decides.
A licence exception is not a waiver.

**Nine factors take a level from 1 to 3.** For each one, 3 is the worst for purba, except relief, where 3 is the most relief.

| factor | 1 | 2 | 3 |
|---|---|---|---|
| [blast area](../../CONTEXT.md#blast-area) | one part, and ≤ 10 crates it brings | two parts, or ≤ 30 crates | another part names the crate's types, or > 30 crates |
| upkeep | a release within 12 months | within 2 years | older |
| maintainers | ≥ 3 people can publish, or a team or an organisation account owns it | two | one |
| upstream review | ≥ 27 of the last 30 commits reviewed | ≥ 15 | < 15 |
| tests | CI runs the tests on each change | tests, and no CI | no tests |
| `unsafe`, in what purba compiles | `#![forbid(unsafe_code)]`, or no `unsafe` | each `unsafe` block has a `SAFETY` comment | an `unsafe` block without one |
| API churn | ≥ 1.0, and no major release in 12 months | 0.x, or a major release in 12 months | 0.x with a breaking release within 3 months (a 0.0.x bump is breaking), or ≥ 1.0 with a major release within 3 months |
| owner in purba | a bot proposes each new version | a record names when a person raises it | nobody |
| relief | the higher of: < 100 lines saved, or std would do; fast feedback, or no quality | 100 to 1,000 lines; a legible result, or a reviewable core | > 1,000 lines, or `unsafe` FFI; an honest outcome, or explicit |

Relief takes the higher of two levels: one for the code that the crate saves, and one for the quality it serves.
The qualities in the relief row are the five that [purba ranks an honest outcome first](purba-ranks-an-honest-outcome-first.md) ranks.
Each takes the level that [the order of design](a-design-in-depth-waits-for-what-it-assumes.md#decision-outcome) gives it as impact.

**The formula.** The other seven factors are each factor except blast area and relief: upkeep, maintainers, upstream review, tests, `unsafe`, API churn and owner in purba. Their weights are equal.

```math
\begin{aligned}
\text{pain} &= \frac{1}{2}\left(\text{blast area} + \frac{1}{7}\sum_{\text{the other seven factors}} \text{level}\right) \\
\text{in} &\iff \text{each check passes} \;\land\; \text{relief} > \text{pain}
\end{aligned}
```

| relief | joins when pain is |
|:-:|---|
| 1 | never |
| 2 | below 2 |
| 3 | below 3 |

**`.config/crates.toml` holds one entry for each crate.**

| field | holds |
|---|---|
| the nine factors | the level of each |
| `version` | the version that the levels judge |
| `owners` | the owners on crates.io when a person judged the crate: each login, and whether it is a user or a team |
| `judged` | the date a person judged the levels |
| `scored` | the record of the design that scored the relief and the trust of the crate, or the issue comment where a person scored them |
| `waiver.fails` | each failure that the waiver lets through: the id of an advisory, `"source"`, or `"score"` when relief does not exceed pain |
| `waiver.why` | why the crate joins against those failures |
| `waiver.issue` | the issue where a person decided the waiver |

The entry holds no result of a check.
A waiver names an advisory by its id, so a new advisory on the same crate fails the check again.
No waiver lets a crate past the trust check.

**`mise run lint:crates` calculates the verdict.** It runs the advisory and the source checks with `cargo-deny`, on the workspace and on each cargo script.
It refuses:

- a crate that a manifest names with no entry,
- an entry whose version differs from the version that the lock holds,
- an entry with no `scored`, or whose `scored` names a file that is not in `docs/decisions/`,
- an entry with a factor that has no level from 1 to 3,
- an entry whose verdict is out, with no waiver,
- a waiver that names a failure the crate does not have,
- an entry that fails something its waiver does not name,
- a waiver that names anything except an advisory id, `"source"` or `"score"`, or has no `why` or no `issue`,
- a finding of either check on a crate that no manifest names.

A crate that no manifest names passes the gates only through an ignore, with its reason, in the configuration of its graph.
`.cargo/deny.toml` serves the workspace, and `.cargo/<name>.deny.toml` serves the cargo script in `scripts/<name>/`.
`cargo-deny` refuses an unused ignore in each one.

**The formula comes first.** Before a waiver, the pull request tries one change that lets the crate in.

| failure | the change it tries |
|---|---|
| `"score"` | a level or a weight |
| an advisory id, or `"source"` | the wording of the check |

The change stands when it moves no verdict of a crate in `.config/crates.toml`, and no relief that a crate on [the shortlists](https://github.com/kalonji-tools/purba/issues/962#issuecomment-6876192563) needs to join.
Then the crate joins with no waiver.
Otherwise the waiver stands, and its issue names the change that the pull request tried.
`lint:crates` finds a moved verdict, because each moved verdict trips one of its refusals.
The reviewer checks that relief on the shortlists by hand, against [`research/future-crates.md`](https://github.com/kalonji-tools/purba/blob/8607015ff23c21b1c41fea9d17c6f693502f6679/research/future-crates.md).

**A design starts from its [shortlist](../../CONTEXT.md#shortlist).**

- The record of a part's design names each crate it picks, in its Decision Outcome. The pick needs no record of its own.
- Its Considered Options lists each crate of the shortlist that the design scored, with the relief it scored and why the crate lost. A crate the design did not score stays out of Considered Options. A design that picks no crate follows the same rule.
- A design may pick a crate that is not on the shortlist. It measures that crate by the same criteria.
- The shortlist stays as [Which crates go on the shortlist of each part not designed yet?](https://github.com/kalonji-tools/purba/issues/962#issuecomment-6876192563) froze it. No pick edits it.

**Each change judges again what it moves. Nothing judges an entry on a schedule.**

| what moves | the criteria it moves | who judges it again |
|---|---|---|
| each run | advisory, licence, source | `mise run lint`, on each pull request |
| a change to the lock | each factor, and the checks | the pull request that changes the lock, because `lint:crates` refuses a `version` that differs |
| a new part that names the crate | blast area, relief | the pull request that adds the part, as the reviewer judges |
| time passes | upkeep, API churn | the next pull request that changes the lock; otherwise nobody |
| upstream acts | maintainers, upstream review, tests, trust | the next pull request that changes the lock; otherwise nobody |

The pull request that changes the lock measures each factor again.
It updates `version`, `owners`, `judged` and `scored`.

**Four criteria stay out.**

| criterion | why |
|---|---|
| build-time code (`build.rs`, macros) | it flags every crate purba uses, so it decides nothing |
| duplicate versions | a duplicate arrives through other crates, and clippy allows it |
| popularity | "More popular packages tend to have more vulnerabilities" (Zheng et al., in [the research](https://github.com/kalonji-tools/purba/issues/958#issuecomment-6848867938)) |
| an aggregate security score | "the number of reported vulnerabilities increased rather than reduced as the aggregate security score of the packages increased" (Zahan et al., in [the research](https://github.com/kalonji-tools/purba/issues/958#issuecomment-6848867938)) |

**Downside:**

- **Relief and trust are a person's score.** A wrong score carries into the verdict until a pull request corrects it.
- **No tool measures a factor.** The author of a pull request measures each one by hand.
- **On the crates of `2790c1e`, the score drops none.** Only the checks drop a crate.
- **Equal weights are a judgement.** A waiver can show that one factor weighs more.
- **`unsafe` is a Rust factor.** A record that applies this one to another kind of dependency must replace it.
- **A licence exception holds no reason.** `.cargo/deny.toml` names the crate and the licence only.
- **The reviewer checks the shortlists by hand.** The relief each crate on them needs is in an issue comment, so `lint:crates` cannot read them.
- **Few crates calibrated the levels.** A change to a level or a weight can move none of them and still be wrong for the next crate.
- **A reworded check can need a person to judge it.** Then [a standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md) applies to it.
- **`lint:crates` cannot open an issue comment.** For an entry whose `scored` links a comment, it checks only that the field is there.
- **A new advisory turns a pull request red, even when it does not touch the crate.** The checks run on each pull request.
- **A level in `.config/crates.toml` can be out of date.** A crate that its authors leave has no new release. Its version in the lock never changes, so its upkeep stays at level 1. `judged` says when a person judged the levels.
- **`lint:crates` needs the network.** `cargo-deny` fetches the advisory database.

## Confirmation

`mise run lint:crates` refuses each case above.
`bats` tests each refusal.
No check judges relief or trust, because [a standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md) applies.
No check reads whether the pull request that changes the lock measured each factor again. The reviewer reads it.
