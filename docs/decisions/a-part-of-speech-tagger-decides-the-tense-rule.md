# A part-of-speech tagger decides the tense rule

## Context and Problem Statement

The records gate refuses a perfect tense, a rule [purba borrows from Simplified Technical English](purba-borrows-from-simplified-technical-english-rather-than-adopting-it.md).
It found one with two word lists.
One list holds the adverbs that may stand between `has` and its participle.
The other holds the participles that do not end in `-ed`.

Each word a list lacked became a defect, and each defect added a word.
A list of adverbs is never complete, because English makes new adverbs freely.

Harper is an English grammar checker, and harper-core is its library.
Its dictionary records the forms of each verb, so it knows `rewritten` and `had` as participles.

Each option below was measured on the records and on prose that no rule shaped.
Only the lines the gate reads count.

| option | records: true / missed / false | 160 files of other prose: true / missed / false |
|---|---|---|
| the word-list walk, back to `has` over any word, which stops at a listed determiner | 1 / 1 / 0 | 84 / 5 / 2 |
| a Weir rule, run by `harper-cli` | 2 / 0 / 0 | 88 / 1 / 3 |
| the word-list walk or the Weir rule | 2 / 0 / 0 | 89 / 0 / 4 |
| harper-core as a library | 2 / 0 / 0 | 87 / 2 / 1 |

## Considered Options

- **The word-list walk.** Rejected. It passes a participle the list lacks, such as `has never had`, and each one costs a defect. A fix in bash of that list and of [The records gate reads a sentence wrapped across two lines as two sentences](https://github.com/kalonji-tools/purba/issues/311) was the alternative.
- **A Weir rule, run by `harper-cli`.** Rejected, and kept as the fallback. Weir reads the tag and never the dictionary. The tagger gives `enforced`, `pinned` and `rewritten` no tag, so the rule passes `has enforced`. `harper-cli` reads its user dictionary as bare words with no verb form, so purba cannot close that gap.
- **The word-list walk or the Weir rule.** Rejected. It refuses the most tenses, and also the most lines that hold none.
- **harper-core as a library.** Chosen. It reads the forms of a verb from the dictionary and the class of a word from the tagger.

## Decision Outcome

**A part-of-speech tagger decides the tense rule.**
The check walks back from a participle to `has`, `have`, `had` or `having`.
It stops at a determiner, a number, a noun, `to`, a form of `be` or a code span.
In a question that opens with a form of `have` or a question word, a determiner, a number and a noun do not stop it.
Between `have` and the participle, a determiner or a number with no noun or pronoun opens a noun phrase, so `What has a fixed span?` is a possession.

**The tagger does not catch every tense the word-list walk catches, and the trade is accepted.**

| against the word-list walk | lines |
|---|---|
| true tenses the tagger refuses and the walk passes | 6 |
| true tenses the walk refuses and the tagger passes | 2 |
| false lines the walk refuses and the tagger passes | 1 |

**The check is a cargo script in `scripts/`, and `scripts/check-records.sh` calls it.**
Its refusals print under the tense rule, so one rule still gives one message.
[A script that needs a Rust library is a cargo script](a-script-that-needs-a-rust-library-is-a-cargo-script.md) says how it is built.

| part | decided |
|---|---|
| the version | harper-core at a git tag |
| a participle the dictionary lacks | a list of purba's own beside the script, merged over Harper's dictionary |
| continuous integration | a run on `main` caches the build directory, keyed on the `Cargo.lock` of the script and on `.config/mise.toml` |
| when review refuses the Rust check | the Weir rule, run by `harper-cli`, and never beside the library |

**The gerund rule and the passive count keep their word lists.**
The tagger marks `reopening` as a noun and `shrinking` as an adjective.
So it passes a gerund the technical-noun list catches.
A tag does not separate `is refused` from `is closed`, so the passive count gains nothing.

| the gerund, on the same prose | lines |
|---|---|
| true gerunds the word lists refuse and the tagger passes | 14 |
| true gerunds the tagger refuses and the word lists pass | 10 |
| of those, gerunds after `as` or `because` | 9 |

**Only the tagger is adopted, and none of Harper's own lints.**
[Fix spelling mechanically in Markdown](https://github.com/kalonji-tools/purba/issues/219) refused those lints, and this record leaves that refusal as it stands.

**Downside:**

- **The tagger refuses a question about a possession.** `Has the record a fixed span?` is one. No line of the measured prose holds one.
- **The tagger refuses a question about a change to a possession.** `Which record has its scope narrowed?` is one. No line of the measured prose holds one.
- **The tagger passes a question whose subject is a determiner alone.** `Has each refused it?` is one. No line of the measured prose holds one.
- **A participle that Harper's dictionary lacks passes until purba's list holds it.** `grown` is the one such word the measurement found.
- **purba cannot fix a word the tagger reads wrong.** A trained model assigns the tag, and a word list took one line to fix.
- **A new Harper version can change a verdict on text that did not change.** The version moves only through a pull request that edits the tag, and the records gate runs on that pull request.

## Confirmation

**`mise run lint:records` runs the tagger.**
`scripts/check-records.sh` calls `scripts/check-prose/check-prose.rs`, and prints its hits under the tense rule.
The tests in `check-prose.rs` hold the sentences it refuses and the ones it passes.
`scripts/test/check-records.bats` holds its refusal.
That refusal reads the shipped participle list.
