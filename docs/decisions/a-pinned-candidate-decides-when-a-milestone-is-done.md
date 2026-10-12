# A pinned candidate decides when a milestone is done

## Context and Problem Statement

purba holds no product code yet.
Nothing says when a [milestone](../../CONTEXT.md#milestone) is done.
A feature list says what to build, and not whether purba runs real tests honestly.
A third-party suite can say both, if a commit pins its tests before the build starts.
The decision that a milestone is done must not rest on a verdict that cannot decide, or on a person who reads finished code.

| fact | source |
|---|---|
| the whole third-party invocation in oxitest's two corpora used 4 of 39 flags and 1 of 7 subcommands | [Inventory oxitest's feature surface](https://github.com/kalonji-tools/purba/issues/4#issuecomment-5560504787) |
| a clean migration cannot render a failure report, because nothing failed | [Distinguish a milestone from a feature](https://github.com/kalonji-tools/purba/issues/16#issuecomment-5561583582) |
| three oxitest gates rested on a predicate that cannot decide: they refused branches, found no defect, or gated nothing | [Define what "migrated cleanly" means](https://github.com/kalonji-tools/purba/issues/5#issuecomment-5570527130) |
| 770 of 859 merged oxitest pull requests drew no review event | [Decide the comprehension gates and branching model](https://github.com/kalonji-tools/purba/issues/8#issuecomment-5589479943) |
| oxitest's benchmark workflow measured nothing, and its run stayed green | [Decide the milestone shapes and their completion signals](https://github.com/kalonji-tools/purba/issues/19#issuecomment-5591870546) |
| the honest-outcome scenario compares the outcome of each test by name | [the landscape](../../LANDSCAPE.md#the-driver-scenarios) |
| the coverage mark and the harness that runs a candidate are out of scope until the landscape is drawn | [purba's landscape, drawn and ranked](https://github.com/kalonji-tools/purba/issues/463) |

## Considered Options

- **A verdict at the end that the migration is clean.** Rejected. "Clean" decides nothing, and a person then reads the finished code.
- **Total coverage, where a test may split or merge upstream tests.** Rejected. It needs a coverage mark that has no design.
  The honest-outcome scenario also compares by name.
- **A desired-state suite that the person approves before the build, and a signal that the shape of the milestone sets.** Chosen. The person reads one suite, and never the finished code.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `/CONTEXT.md`

**A milestone is done when the signal that its shape declares holds.**

| step | act |
|---|---|
| task one | pin the [candidate](../../CONTEXT.md#candidate) to a commit, and declare the shape |
| task two | commit the [desired-state suite](../../CONTEXT.md#desired-state-suite), and for a performance milestone also its plan. The person approves it before the build, and never against purba's code |
| then | build until the signal holds |

| shape | signal |
|---|---|
| additive | the desired-state suite passes |
| failure report | each failure of a deliberately broken candidate matches its agreed expected output |
| replacing | the suite passes, and the set of upstream names it holds does not change |
| performance | a measurement meets the plan that task two commits |

- **Each upstream name has a migrated test, an excluded test, or no test.** A migrated test keeps the upstream name.
  An excluded test keeps the name, and carries `skip` or `xfail` with a reason.
  The list of names with no test is computed, never written.
- **The candidate's product source stays as it was at the pin.** A migration that needs a source change shows a wrong candidate or a defect in purba.
- **A change after task two that widens or reshapes the suite is free.** A change that shrinks it goes back to the person for approval.
- **An issue of a milestone carries no candidate.** Its review and the approved suite gate it.
- **A change that purba needs to work as a command carries no candidate either.** Such a change adds `--version`, `--help` or the exit code.
  A candidate's tests reach purba through what a test can use, such as a fixture, and never through the command's flags or exit code.

**Downside:**

- **A milestone pays for its whole desired-state suite before any code.** A milestone that takes part of a candidate keeps that cost bearable.
- **No test splits or merges an upstream test.** A large upstream test stays one test.
- **purba never uses a GitHub milestone for a release.** The milestone object carries the candidate instead.
- **The flags and the exit code of the command have no candidate.** Review alone gates them.
- **A performance milestone cannot open without a plan.** The plan comes before any measurement.

## Confirmation

No command checks this yet, because no candidate is pinned.
A later map designs the harness that runs a candidate.
A gate can decide one rule: `git diff` between the pin and the last commit that holds the desired-state suite prints nothing for the product source.

[The landscape](../../LANDSCAPE.md#the-driver-scenarios) states the honest-outcome and legible-result scenarios.
A desired-state suite feeds each one.
[purba ranks an honest outcome first](purba-ranks-an-honest-outcome-first.md#decision-outcome) holds why.
The walking skeleton is the first to run a scenario.
