# The tense rule with a part-of-speech tagger

This prototype measures the questions
[#308](https://github.com/kalonji-tools/purba/issues/308) put to it on
2026-10-06. Records come from `main` at `4d5eb7b`. The walk comes from `8ba16680db9ee48bc3131f594d7c241243525f4b`,
the head of [#309](https://github.com/kalonji-tools/purba/pull/309). The tagger
comes from Harper `v2.3.0`.

This branch is never merged. Each script reproduces one table, and each file in
`out/` is the last run of a script.

No variant clears the bar #308 set: catch every true tense the walk catches, and
refuse no more false lines than the walk. The library with a noun stop, `lib-np`, comes
closest. It refuses 6 true lines that the walk passes, 1 of them in the
records. It refuses 1 false line where the walk refuses 2. But it misses 2 true
lines that the walk catches, so it fails the first half of the bar.

## The variants

| variant | what it is |
|---|---|
| `walk` | `scripts/check-records.sh` at `8ba16680db9ee48bc3131f594d7c241243525f4b`. It walks back from a participle over any word, and stops at a listed determiner, `to`, a code span or a form of `be` |
| `weir` | `weir/PerfectTense.weir`, run by `harper-cli`. Weir has no repetition, so the rule spells out each gap of up to 4 tokens. It reads only the tagger's tag |
| `union` | a line `walk` or `weir` refuses |
| `lib-det` | `libwalk/`, with harper-core as a library. A participle is a word that the dictionary marks as a past participle or a regular past. The walk stops at a word that the tagger marks as a determiner or a number, and at `to`, a form of `be` or a code span |
| `lib-np` | `lib-det`, and the walk also stops at a word that the tagger marks as a noun |
| `lib-det+v`, `lib-np+v` | each one, and a word that the tagger marks as a verb also counts as a participle |

## The score

From `score.sh`. Only the lines the bash gate reads are counted, so a table row
and a heading are left out. A true tense is a line some variant refuses that
`out/wild-false.keys` does not name. A tense that no variant refuses is not
counted.

| variant | records: true / missed / false | wild: true / missed / false | misses a line the walk catches | refuses a line the walk passes |
|---|---|---|---|---|
| walk | 1 / 1 / 0 | 84 / 5 / 2 | none | none |
| weir | 2 / 0 / 0 | 88 / 1 / 3 | issue-125.md:19 | issue-153.md:14 issue-228.md:77 |
| union | 2 / 0 / 0 | 89 / 0 / 4 | none | issue-153.md:14 issue-228.md:77 |
| lib-det | 2 / 0 / 0 | 87 / 2 / 2 | issue-125.md:19 issue-142.md:3 | issue-228.md:77 |
| lib-np | 2 / 0 / 0 | 87 / 2 / 1 | issue-125.md:19 issue-142.md:3 | none |
| lib-det+v | 2 / 0 / 0 | 88 / 1 / 5 | issue-125.md:19 | issue-153.md:14 issue-228.md:77 issue-33.md:13 issue-84.md:42 |
| lib-np+v | 2 / 0 / 0 | 88 / 1 / 3 | issue-125.md:19 | issue-153.md:14 issue-228.md:77 |

The two lines `lib-np` misses, and why:

| line | why |
|---|---|
| `issue-125.md:19` *"Have purba's own 92 issues been consistent?"* | the walk stops at the noun `issues` |
| `issue-142.md:3` *"now that it has grown one workflow"* | the dictionary gives `grown` no verb form |

The walk catches the first only because it does not stop at a noun. That same
choice makes it refuse `The gate has records refused by the person.`

## The probes

From `probes.sh`. The spec of
[#302](https://github.com/kalonji-tools/purba/issues/302) holds the walk's result for most of the same sentences.

See `out/probes.md`. Three results decide most of it:

- `weir` passes `has enforced`, `has pinned`, `has configured`, `has parsed` and
  `has rewritten`. Its tagger gives these words no tag, and Weir cannot read the
  dictionary.
- Every `lib` variant refuses all five, and refuses `has never had` and
  `have become`, which [#306](https://github.com/kalonji-tools/purba/issues/306) owns.
- Every variant passes `Has the gate refused the record?`, which
  [#307](https://github.com/kalonji-tools/purba/issues/307) owns.

## What the library costs

| | measured |
|---|---|
| a cold build of `libwalk` | 1 m 23 s |
| one run over the 36 records | 1.03 s, 145 MB |
| three runs over both corpora | byte-identical output |
| the source | a git tag. harper-core `2.3.0` is not on crates.io |

[purba writes its scripts in bash until purba can test them](https://github.com/kalonji-tools/purba/blob/main/docs/decisions/purba-writes-its-scripts-in-bash-until-purba-can-test-them.md)
stands against a gate written in Rust. This prototype does not answer that.

## The cargo script

From `script/time.sh`. `script/libwalk.rs` is `libwalk/` as one file, with its
manifest inside it. Each form starts from an empty build directory.

| form | cold build and first run | each later run |
|---|---|---|
| a binary crate, release | 82.0 s | 1.0 s |
| a cargo script, release | 89.3 s | 1.2 s |
| a cargo script, debug | 59.7 s | 25.7 s |

Two runs of the script differed by several seconds in each cold row, and by
less than a second in each later run. A debug build is quicker to make and
runs too slowly to gate. `resolver.lockfile-path` writes the lockfile of the
script where `script/time.sh` asks, and `--locked` then holds the build to it.

## A word the dictionary lacks

From `dictionary.sh`. The `+d` variants merge a dictionary of purba's own over
Harper's, and give `grown` its verb form. Only one line changes:
`issue-142.md:3`, *"now that it has grown one workflow"*, which `lib-np` passes
and `lib-np+d` refuses. Every other output in `out/` is the same with or
without it.

## The gerund

From `gerund.sh`. The library variant `ing-lead+u` keeps the scope of the bash
rule and drops both of its lists. An `-ing` word is a verb form when the tagger
marks it a verb, or when it has no tag and the dictionary marks it progressive.
It is refused after a form of `be`, or after a word the tagger marks as a
preposition or a subordinator, and the walk skips adverbs and particles.

Only the disagreements are labelled, in `out/g-labels.tsv`. `being` is counted
apart because the bash rule admits it on purpose, and a quotation is counted
apart because a record quotes it as it was written.

| corpus | refused by | true | `being` | quoted | false |
|---|---|---|---|---|---|
| records | bash only | 0 | 0 | 0 | 0 |
| records | library only | 3 | 4 | 1 | 1 |
| wild | bash only | 14 | 0 | 0 | 0 |
| wild | library only | 7 | 3 | 4 | 2 |

The library misses 14 true gerunds that bash refuses, such as
`without reopening`, `while recording` and `is shrinking`. The tagger marks
these words as a noun or an adjective. That is the noun-or-verb question that
the technical-noun list exists to answer, and a tag does not answer it here.

Of the 10 true gerunds the library adds, 9 follow `as` or `because`, the words
[#310](https://github.com/kalonji-tools/purba/issues/310) adds to the bash rule.

## The passive

A tag does not separate `is refused` from `is closed`, so the tagger decides the
passive no better than the count does. A scratch run with a Weir rule showed
this, and no script here reproduces it.

## Run it

```bash
./corpus.sh                                     # needs gh
mise exec harper-cli@2.3.0 -- ./keys.sh         # needs zip and cargo
./score.sh > out/score.md
mise exec harper-cli@2.3.0 -- ./probes.sh > out/probes.md
./gerund.sh > out/gerund.md
mise exec -- ./script/time.sh > out/script-times.md   # needs the nightly
./dictionary.sh > out/dictionary.md
```
