# A forecast finds the crates for each stated need

## Context and Problem Statement

[A crate joins when its relief exceeds its pain](a-crate-joins-when-its-relief-exceeds-its-pain.md) starts the design of a part from its [shortlist](../../CONTEXT.md#shortlist).
A shortlist answers one need of a part not built yet.
The records, the landscape, the glossary and the issues state those needs in prose.
No script reads prose.

| fact | source |
|---|---|
| the tree and the issues stated 24 needs of parts not designed yet | [`research/future-crates.md` § 1](https://github.com/kalonji-tools/purba/blob/8607015ff23c21b1c41fea9d17c6f693502f6679/research/future-crates.md#L54) |
| a call to std or to a crate that purba holds met 12 of them | [How does a record state a need of a part not designed yet, so that a script reads it?](https://github.com/kalonji-tools/purba/issues/996#issuecomment-6891765982) |
| 0 of 1,317 issues sent through the API kept the structure of a form | [a register belongs to one location](a-register-belongs-to-one-location.md#context-and-problem-statement) |
| a query in a need's own words found at best 20 of 41 crates | [How do tools and projects find the crates that could meet a stated need?](https://github.com/kalonji-tools/purba/issues/994#issuecomment-6891284107) |
| a query by crates.io keywords found 14 of the 15 crates that no other query found, with words picked from the words of those crates | [How does a record state a need of a part not designed yet, so that a script reads it?](https://github.com/kalonji-tools/purba/issues/996#issuecomment-6891765982) |
| those keywords found 11 of the 15 in a top 3, 12 in a top 10, and 14 in a top 20 or a top 50 | [Wayfinder: forecast the crates that the parts not designed yet could use](https://github.com/kalonji-tools/purba/issues/999#issuecomment-6892647843) |
| a script gave the level a person gave in 323 of the 328 cells of 41 crates, in about 8.5 s for each crate | [Can a script measure the levels of a crate that purba does not lock, as research/future-crates.md measured them by hand?](https://github.com/kalonji-tools/purba/issues/995#issuecomment-6891407459) |

## Considered Options

- **The need's sentence alone.** A query in a need's own words finds too few crates.
- **The sentence and the crates that meet it.** A list of crates finds no crate that is not on it.
- **A marker beside each line that states a need.** A marker scatters the needs over many files.
- **The body of an issue.** No gate reads an issue body.
- **A `met` field on a need that purba meets.** A met need has nothing to forecast, so it has no line.
- **A line that leaves when the design decides its need.** The design of a part decides all of its needs at once.
- **The top 50 crates for each word.** It found no crate that the top 20 missed. It measures up to 50 crates for each word, not 20.

## Decision Outcome

**Reach:** `.config/needs.toml` `.config/shortlists.toml` `.config/tasks.toml` `.config/lanes` `.editorconfig` `scripts/check-needs.sh` `scripts/test/check-needs.bats` `scripts/forecast.sh` `scripts/forecast/**` `scripts/test/forecast.bats` `/LANDSCAPE.md`

**Area:** design, tooling

**`.config/needs.toml` holds each stated need of a part not built yet.**
Each line holds fields for a script, and a `need` sentence for a reader.

```toml
read-flags = { part = "command", need = "read the command's flags", words = ["args", "cli"], have = "pyo3 copies sys.argv, parsed by hand", stated = "https://github.com/kalonji-tools/purba/blob/2790c1e/CONTEXT.md#L181" }
```

| field | holds | required |
|---|---|---|
| the key | a name for the need, unique in the file, and the key of its entries in `.config/shortlists.toml` | yes |
| `part` | a part or concern of [the landscape](../../LANDSCAPE.md), as its line names it | yes |
| `need` | the need, in one sentence | yes |
| `words` | crates.io keywords, at least one | yes |
| `have` | what std or a crate that purba holds does, when it meets the need only in part | no |
| `stated` | a link to the line that states the need: a record, `LANDSCAPE.md`, `CONTEXT.md`, an issue or a comment | yes |
| `issue` | the open issue that vets the need's crates | no |
| `declined` | each crate that the issue declined | no |

**A line of `.config/needs.toml` has no length limit.** TOML 1.0 lets an inline table hold no newline.
[A normaliser's style is chosen for the reader of the code](a-normalisers-style-is-chosen-for-the-human-who-reads-the-code.md) names this exception.

**A need has no line when a call to std, or to a crate that purba holds, meets it.**

**A line moves through these states.**

| state | the line | what moves it on |
|---|---|---|
| new | no `issue`, no `declined`, no entry | the forecast runs in the pull request that adds the line, and an entity opens the issue |
| discussed | `issue` | the issue concludes |
| shortlisted | entries in `.config/shortlists.toml`, and no `issue` | the build of the part closes |
| declined | `declined`, and no `issue` | the forecast finds a crate not in `declined` |

- The issue vets each crate.
- When the issue concludes, the pull request that answers it writes an entry for each crate that the issue kept. When the issue kept no crate, that pull request names each crate in `declined`.
- A line whose forecast finds no crate gets `declined = []`.
- A line leaves when the build of its part closes. Its entries in `.config/shortlists.toml` leave with it.
- A need that the design of its part leaves open moves to the part that decides it, so its `part` changes.

**`mise run list:needs` finds and measures the crates of each line that no issue vets.**

| run | reads |
|---|---|
| `mise run list:needs` | each line with no `issue`, no `declined` and no entry in `.config/shortlists.toml` |
| `mise run list:needs --part <part>` | those lines of the part, and each of its lines with `declined` |
| `mise run list:needs --shortlisted` | each entry of `.config/shortlists.toml`, and no line |

- For each word of a line, it asks crates.io for the 20 crates with the most recent downloads that carry the keyword.
- It waits at least one second between two requests to crates.io. Its User-Agent names purba and no person.
- It drops each crate in `declined`.
- Without `--shortlisted`, it runs the advisory, licence and source checks of `.cargo/deny.toml` on each crate.
- It measures the eight levels from crates.io, a build of the crate and GitHub.
- A level that it cannot measure counts as 3. `unmeasured` names that level.
- For blast area, the parts of a crate are the parts of each line whose entry holds the crate or whose words find it.
- It computes pain and `needs` with the formula of [a crate joins when its relief exceeds its pain](a-crate-joins-when-its-relief-exceeds-its-pain.md).
- It prints the body of one issue for each line that finds a crate.
- A new line whose words find no crate gets no body. The forecast prints that the line needs `declined = []`.
- `--shortlisted` measures each entry at its `version`. It counts only the crates.io versions and the commits made before the `measured` day of the entry.
- `--shortlisted` prints each level, `unmeasured` and `needs` that moved.
- It writes neither `.config/needs.toml` nor `.config/shortlists.toml`.

**The forecast runs when a pull request can change what it finds, and when the design of a part starts.**

| event | the run |
|---|---|
| a pull request adds a line | `mise run list:needs`, in that pull request |
| a pull request changes the `words` of a declined line | `mise run list:needs --part <part>`, in that pull request |
| the design of a part starts | `mise run list:needs --part <part>` |
| a pull request changes a threshold of a level | `mise run list:needs --shortlisted`, in that pull request |

No workflow runs it, and nothing runs it on a schedule.

**`mise run lint:needs` checks `.config/needs.toml`.**
It asks no server anything.
It refuses:

- a line whose `part` is not a part or concern of the landscape,
- a line with no `words`, or an empty list of them,
- a line with no `stated`, or an empty one,
- a line with no `issue`, no `declined` and no entry in `.config/shortlists.toml`,
- an entry of `.config/shortlists.toml` whose need has no line.

**Downside:**

- **Whoever writes a line picks its words.** A query finds only the crates that carry one of those words. The keyword counts in the Context are an upper bound, because their words came from the crates that they found.
- **`lint:needs` asks no server.** A `stated` link that no longer states the need passes. A keyword that crates.io does not hold passes too. The reviewer reads both.
- **No gate refuses a line with no `need`, or a line with `issue` beside `declined` or an entry.** The reviewer reads both.
- **No gate decides whether a call meets a need.** The reviewer judges whether a line belongs.
- **The forecast builds each crate.** The build scripts and proc macros of the crate, and of what it brings, run on the machine that runs the forecast.
- **The forecast reads only GitHub and GitHub Actions.** For a crate on another forge, upstream review and tests are unmeasured. A crate whose tests run outside GitHub Actions, or behind a script that a workflow calls, gets tests level 2 where a person gives 1.
- **The forecast builds for `x86_64-unknown-linux-gnu` only.** The build needs the standard library of that target. An `unsafe` block that only macOS compiles goes uncounted.
- **`--shortlisted` reads the owners, the dependency graph and the CI runs as they are on the day it runs.** So a level of maintainers, blast area, `unsafe` or tests can move with no change to a threshold.
- **A day is the unit of `measured`.** `--shortlisted` drops a release made on the `measured` day before the forecast ran.
- **The forecast subtracts months from a date as text, for the windows of upkeep and API churn.** It takes three months before 2026-05-31 as `2026-02-31`. A release on 2026-03-01 then falls inside the window, though GNU `date` starts the window on 2026-03-03.
- **No gate checks that a pull request ran the forecast.** The reviewer reads the output of the forecast in the pull request.
- **No gate sees that a part's build closed.** The pull request that closes the build removes the part's lines, and the reviewer reads for it.

## Confirmation

`mise run lint:needs` refuses each breach that the list above names.

`bats` tests which lines the forecast reads, the `declined` filter, a line with no crate, `--part`, `--shortlisted` and the licence count. Each test reads recorded crates.io answers, and fakes the build and GitHub.

[A crate joins when its relief exceeds its pain](a-crate-joins-when-its-relief-exceeds-its-pain.md) holds the rules of the shortlists and the formula.
