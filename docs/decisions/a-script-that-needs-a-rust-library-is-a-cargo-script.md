# A script that needs a Rust library is a cargo script

## Context and Problem Statement

[purba writes its scripts in bash until purba can test them](purba-writes-its-scripts-in-bash-until-purba-can-test-them.md), and lets a change of language be earned by what bash cannot do.
A rule can need a library that bash cannot call.
[A part-of-speech tagger decides the tense rule](a-part-of-speech-tagger-decides-the-tense-rule.md) needs harper-core, which is a Rust library.

The same check was built in each form, against harper-core and the records.

| form | cold build and first run | each later run, over the 36 records |
|---|---|---|
| a binary crate, release | 82 s | 1.0 s |
| a cargo script, release | 89 s | 1.2 s |
| a cargo script, debug | 60 s | 26 s |

## Considered Options

- **One named exception, like the Python inside `build.yml`.** Rejected. The next script that needs a library would need a record of its own.
- **A binary crate.** Rejected as the first form, and kept as the fallback. It builds a few seconds faster from cold, and a cache makes a cold build rare. It is a directory where a cargo script is one file.
- **A cargo script, under a rule that names the condition.** Chosen. The condition is what earns the change, so the next script that meets it needs no new record.

## Decision Outcome

**A script that needs a Rust library that bash cannot call is a cargo script.**
`cargo -Zscript` builds it in the release profile.
A debug build runs too slowly to gate.

**A binary crate with the same source is its fallback.**
It waits for the day `-Zscript` breaks on a nightly.

**Its lockfile sits in the tree.**
`resolver.lockfile-path` puts the `Cargo.lock` beside the script, and `--locked` holds every build to it.
[One manifest declares each package](one-manifest-declares-each-package.md) gives the crates of the script to the script's own manifest.

**`bats` tests it through the bash script that calls it.**
The library earns the change of language, and the runner does not change.

**Downside:**

- **It needs the nightly toolchain.** `-Zscript` exists only on nightly, and [purba meets the next trait solver before it stabilizes](purba-meets-the-next-trait-solver-before-it-stabilizes.md) allows it outside the product crate.
- **A cold build takes more than a minute.** A cache pays it once for each lockfile, and a machine without the cache pays it on its first run.
- **A test reaches the script only through its caller.** A defect inside the script shows as a wrong refusal of the bash script.
- **`lint:licences` reads the crate graph of purba only, so the crates of a cargo script carry any licence.** They never reach the wheel, like a tool that `mise.toml` declares.

## Confirmation

**No cargo script is in the tree yet, and no Rust formatter, linter or placement check reads one.**
`scripts/check-placement.sh` reads `*.sh` only.
[The records gate passes a perfect tense behind an adverb that neither ends in -ly nor is listed](https://github.com/kalonji-tools/purba/issues/302) adds the first cargo script, and wires all three with it.
