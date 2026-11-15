# A forecast finds the crates for each stated need

## Context and Problem Statement

[A crate joins when its relief exceeds its pain](a-crate-joins-when-its-relief-exceeds-its-pain.md) starts the design of a part from its [shortlist](../../CONTEXT.md#shortlist).
A shortlist answers one need of a part not designed yet.
The records, the landscape, the glossary and the issues state those needs in prose.
No script reads prose.

| fact | source |
|---|---|
| the tree and the issues stated 24 needs of parts not designed yet | [`research/future-crates.md` § 1](https://github.com/kalonji-tools/purba/blob/8607015ff23c21b1c41fea9d17c6f693502f6679/research/future-crates.md#L54) |
| a call to std or to a crate that purba holds met 12 of them | [How does a record state a need of a part not designed yet, so that a script reads it?](https://github.com/kalonji-tools/purba/issues/996#issuecomment-6891765982) |
| 0 of 1,317 issues sent through the API kept the structure of a form | [a register belongs to one location](a-register-belongs-to-one-location.md#context-and-problem-statement) |
| a query in a need's own words found at best 20 of 41 crates | [How do tools and projects find the crates that could meet a stated need?](https://github.com/kalonji-tools/purba/issues/994#issuecomment-6891284107) |
| a query by crates.io keywords found 14 of the 15 crates that no other query found, with words picked from the words of those crates | [How does a record state a need of a part not designed yet, so that a script reads it?](https://github.com/kalonji-tools/purba/issues/996#issuecomment-6891765982) |

## Considered Options

- **The need's sentence alone.** A query in a need's own words finds too few crates.
- **The sentence and the crates that meet it.** A list of crates finds no crate that is not on it.
- **A marker beside each line that states a need.** A marker scatters the needs over many files.
- **The body of an issue.** No gate reads an issue body.
- **A `met` field on a need that purba meets.** A met need has nothing to forecast, so it has no line.
- **A line that leaves when the design decides its need.** The design of a part decides all of its needs at once.

## Decision Outcome

**Reach:** `.config/needs.toml` `.config/shortlists.toml` `.config/tasks.toml` `.config/lanes` `.editorconfig` `scripts/check-needs.sh` `scripts/test/check-needs.bats` `/LANDSCAPE.md`

**Area:** design, tooling

**`.config/needs.toml` holds each stated need of a part not designed yet.**
Each line holds fields for a script, and a `need` sentence for a reader.

```toml
read-flags = { part = "command", need = "read the command's flags", words = ["args", "cli"], have = "pyo3 copies sys.argv, parsed by hand", stated = "https://github.com/kalonji-tools/purba/blob/2790c1e/CONTEXT.md#L181" }
```

| field | holds | required |
|---|---|---|
| the key | a name for the need, unique in the file, and the key of its entries in `.config/shortlists.toml` | yes |
| `part` | a part or concern of [the order of design](../../LANDSCAPE.md#the-order-of-design), as its line names it | yes |
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

**`mise run lint:needs` checks `.config/needs.toml`.**
It asks no server anything.
It refuses:

- a line whose `part` is not in the order of design,
- a line with no `words`, or an empty list of them,
- a line with no `stated`, or an empty one,
- a line with no `issue`, no `declined` and no entry in `.config/shortlists.toml`,
- an entry of `.config/shortlists.toml` whose need has no line.

**Downside:**

- **Whoever writes a line picks its words.** A query finds only the crates that carry one of those words. The measure in the Context is an upper bound, because its words came from the crates that it found.
- **`lint:needs` asks no server.** A `stated` link that no longer states the need passes. A keyword that crates.io does not hold passes too. The reviewer reads both.
- **No gate refuses a line with no `need`, or a line with `issue` beside `declined` or an entry.** The reviewer reads both.
- **No gate decides whether a call meets a need.** The reviewer judges whether a line belongs.

## Confirmation

`mise run lint:needs` refuses each breach that the list above names.

The forecast lands with [Build the forecast, mise run forecast](https://github.com/kalonji-tools/purba/issues/1003), which this record waits on.

[A crate joins when its relief exceeds its pain](a-crate-joins-when-its-relief-exceeds-its-pain.md) holds the rules of the shortlists and the formula.
