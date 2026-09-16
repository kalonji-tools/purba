# purba meets the next trait solver before it stabilizes

## Context and Problem Statement

purba builds on Rust nightly.
That was decided in conversation and never written down, so the tree carries the cost of a channel without the reason for it.

A reason is owed because "nightly because newer" cannot be shown wrong.
This map has refused that shape four times already, and a channel is the most expensive place to accept it.

Nothing in the tree needs nightly.
The crate compiles on stable today, and the dependency floor of 1.96 is cleared by any current release.

| measured | value |
|---|---|
| purba's own Rust, on 2026-09-16 | 19 lines |
| the pinned graph under the global solver on a stable compiler, on 2026-09-14 | 93 crates, exit 0 |

So the purchase is not a feature the code needs.
It is early contact with the compiler change that will arrive on stable anyway.

The next generation trait solver has been on by default on nightly since 2026-08-22.
There is no flag to write: the only value a project can set is the opt-out.
Stabilizing it is the sitting 2026 project goal, and it carries 115 open bug reports.

**The two solvers disagree, and the disagreement is asymmetric.**

| direction | breadth | fails on nightly | fails on stable |
|---|---|---|---|
| the new solver is stricter, and rejects what the old accepted | the dominant case, reported upstream as a non trivial amount of breakage | yes | no |
| the new solver is more complete, and accepts what the old rejected | two named patterns: recursive `impl Trait` calls, and associated types in higher ranked types | no | yes |

Only the second row can strand purba on nightly, and only through code purba writes itself.

A second question arrives with the channel, because the channel has to be named in a file.
[mise names every tool version](mise-names-every-tool-version.md) already decided which file that is, and the tree contradicted it: `rust-toolchain.toml` named a stable release under a comment describing an environment that had been removed.
Leaving both files in place is not a neutral duplication.
Three of rustup's five precedence ranks decide why.

| rustup precedence | rank |
|---|---|
| `cargo +toolchain` on the command line | 1 |
| `RUSTUP_TOOLCHAIN`, which mise sets | 2 |
| `rust-toolchain.toml` | 4 |

A toolchain file that disagrees with mise loses inside a mise shell and wins in any job that does not run through mise.
Nothing reports the difference.

## Considered Options

**What the channel buys.**

- **Newer `rustfmt` and `clippy`, or `-Z` flags in continuous integration.** Real, and not reasons. Both follow from any nightly, so neither could show this decision wrong, and a flag is spent out of a channel already bought. They are gains purba collects, and they could not have bought the channel on their own.
- **The next generation trait solver.** Chosen. It is a specific compiler change, on a published stabilization path, and the claim it supports can be tested on every pull request.

**How the return to stable stays open.**

- **Nothing checks it.** Rejected. The agreed escape hatch is a move back to stable as a last resort, and with no check nothing reports whether that hatch is still open. The drift is silent, it accumulates, and it surfaces as a pile of inference repairs on the day the hatch is needed.
- **A blocking stable build.** Rejected. It forbids the two patterns in the second row of the table above, outright and in advance, and those are the only gains the new solver offers a project that writes Rust.
- **A non blocking stable build.** Chosen. The tree holds no nightly only syntax, so a stable `cargo check` is an exact detector rather than an approximation, and it records the day the hatch closes without ever stopping the work.

## Decision Outcome

purba builds on Rust nightly to meet the next generation trait solver before it reaches stable.

**Which nightly.**

The channel is named once, in `mise.toml`, as a floating `nightly`.
`mise.lock` records the dated nightly that name resolves to.
A person reads and bumps the floating name, and a machine installs the date.
Naming a date in both files would state one fact twice and let the two copies disagree.

⚠️ **Neither file exists yet.**
[Write the mise substrate](https://github.com/kalonji-tools/purba/issues/39) writes them, and that ticket is blocked by this one.

**`rust-toolchain.toml` is deleted here.**
This follows from [mise names every tool version](mise-names-every-tool-version.md) rather than deciding anything new, and the deletion is what stops the tree contradicting that record.
Between this change and the substrate, purba names no toolchain anywhere, which costs nothing because nothing in the tree compiles Rust yet.

**A nightly only feature with no stable fallback is refused inside the product crate.**
Outside it the feature is allowed, and the ticket that uses it names its fallback.
This keeps the one thing the stable check watches free of anything stable cannot compile.

**This record expires when the solver stabilizes, and the channel does not expire with it.**
More nightly features can be taken up before that day, and each of them has its own life.

| when | what ends |
|---|---|
| the solver stabilizes | this record, because the reason it gives for the channel is discharged |
| every nightly feature in use is stable | the channel, and the stable control build with it |
| a feature stabilizes, or the channel goes | that feature's use, which takes the fallback its ticket named |

The control build outlives this record, because it detects drift from being on nightly at all rather than drift from the solver.

**When this record expires, the channel is unwarranted until another record names a reason to keep it.**
No single feature can extend the channel on its own, because every nightly only use names a fallback and can therefore be given up.
The warrant is always exactly one record, and never one record per feature.

**Downside:** the stable check is non blocking, and this project's own position is that a check which runs without blocking is a suggestion. It will sit red and ignored. That is accepted because its output is read on one day only, the day someone reaches for the hatch, and a blocking check would have bought that day by forbidding the solver's only two gains in advance. The channel also moves under purba without anyone choosing a moment: a nightly is a snapshot of a compiler whose new solver still carries the open bug reports counted above, and a bump can break the tree for reasons that are nobody's fault and still cost a day. `mise.lock` pins the toolchain by date and not by checksum, which [mise names every tool version](mise-names-every-tool-version.md) records in full.

## Confirmation

`cargo +<stable> check`, run on every pull request, reporting and never blocking.

It fails on the day purba first writes code the old solver rejects, which is the day the return to stable stops being one decision wide.

The `+` form is rank 1 in the table above and the toolchain mise exports is rank 2, so this runs inside the ordinary environment and needs no second one.

⚠️ **Nothing runs it today.**
[Write the quality workflow](https://github.com/kalonji-tools/purba/issues/36) wires it, and that ticket is blocked by this one.
Until it lands, the hatch is open on the evidence of the table in the Context above and on nothing newer.
