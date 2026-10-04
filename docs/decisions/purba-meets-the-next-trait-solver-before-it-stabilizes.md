# purba meets the next trait solver before it stabilizes

## Context and Problem Statement

purba builds on Rust nightly.
That was decided in conversation and never written down, so the tree carries the cost of a channel without the reason for it.

A reason is owed because "nightly because newer" cannot be shown wrong.
This map refused that shape four times already, and a channel is the most expensive place to accept it.

Nothing in the tree needs nightly.
The crate compiles on stable today, and the dependency floor of 1.96 is cleared by any current release.

| measured | value |
|---|---|
| purba's own Rust, on 2026-09-16 | 19 lines |
| the pinned graph under the global solver on a stable compiler, on 2026-09-14 | 93 crates, exit 0 |

So the purchase is not a feature the code needs.
It is early contact with the compiler change that will arrive on stable anyway.

nightly turned the next generation trait solver on by default on 2026-08-22.
There is no flag to write: the only value a project can set is the opt-out.
Stabilizing it is the sitting 2026 project goal, and it carries 115 open bug reports.

**The two solvers disagree, and the disagreement is asymmetric.**

| direction | breadth | fails on nightly | fails on stable |
|---|---|---|---|
| the new solver is stricter, and rejects what the old accepted | the dominant case, reported upstream as a non trivial amount of breakage | yes | no |
| the new solver is more complete, and accepts what the old rejected | two named patterns: recursive `impl Trait` calls, and associated types in higher ranked types | no | yes |

Only the second row can strand purba on nightly, and only through code purba writes itself.

## Considered Options

**What the channel buys.**

- **Newer `rustfmt` and `clippy`, or `-Z` flags in continuous integration.** Real, and not reasons. Both follow from any nightly, so neither could show this decision wrong, and a flag is spent out of a channel already bought. They are gains purba collects, and neither could buy the channel on its own.
- **The next generation trait solver.** Chosen. It is a specific compiler change, on a published stabilization path, and the claim it supports can be tested on every pull request.

**How the return to stable stays open.**

- **Nothing checks it.** Rejected. The agreed escape hatch is a move back to stable as a last resort. With no check nothing reports whether that hatch is still open. The drift is silent, it accumulates, and it surfaces as a pile of inference repairs on the day the hatch is needed.
- **A blocking stable build.** Rejected. It forbids the two patterns in the second row of the table above, outright and in advance. Those are the only gains the new solver offers a project that writes Rust.
- **A non blocking stable build.** Chosen. The tree holds no nightly only syntax, so a stable `cargo check` is an exact detector rather than an approximation. It records the day the hatch closes and never stops the work.

## Decision Outcome

purba builds on Rust nightly to meet the next generation trait solver before it reaches stable.

[The nightly is named by a date](the-nightly-is-named-by-a-date.md) says which nightly.

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

**Downside:**

- **The stable check does not block, and this project's own position is that a check which runs without a block is a suggestion.** It will sit red and ignored. That is accepted because its output is read on one day only, the day someone reaches for the hatch. A blocking check would buy that day, and forbid the solver's only two gains in advance.
- **The channel moves under purba without anyone choosing a moment.** A nightly is a snapshot of a compiler whose new solver still carries the open bug reports counted above. A bump can break the tree for reasons that are nobody's fault, and still cost a day. `mise.lock` records a date for the toolchain and does not pin it, which [purba carries no tool manager beside mise, and no compiler of its own](purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md) records in full.

## Confirmation

`cargo +<stable> check`, run on every pull request, reporting and never blocking.

It fails on the day purba first writes code the old solver rejects.
That is the day the return to stable stops being one decision wide.

rustup ranks the `+` form above `RUSTUP_TOOLCHAIN`, which mise sets.
So this runs inside the ordinary environment and needs no second one.

⚠️ **Nothing runs it today.**
[Write the quality workflow](https://github.com/kalonji-tools/purba/issues/36) wires it, and that ticket is blocked by this one.
Until it lands, the hatch is open on the evidence of the table in the Context above and on nothing newer.
