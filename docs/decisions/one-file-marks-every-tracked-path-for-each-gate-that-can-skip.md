# One file marks every tracked path for each gate that can skip

## Context and Problem Statement

[A path not marked to skip starts the gate](a-path-not-marked-to-skip-starts-the-gate.md).
A path added with no mark therefore starts the gate on every pull request that changes it.
Unless a rule reports that path, nobody decides whether the gate needs it.
Where the gate does not read the path, each of those runs is waste.

## Considered Options

Which rule keeps the marks complete?

- **No rule.** A path added with no mark starts the gate for good, and nobody decides it.
- **Derive the marks from the build.** A derivation sees what Cargo or `strace` records, and misses what mise, GitHub and the step of `ci.yml` that loads the wheel read. A derived mark then skips the gate on a path that the gate reads.
- **A list of only the paths that start the gate, or of only those that skip it.** A default decides a path on neither list, and no rule makes anyone decide that path.

Where do the marks live?

- **Each gate keeps its patterns in its own script.** No rule keeps a pattern list complete.
- **A file for each gate.** Each file names every tracked path again.

## Decision Outcome

**Reach:** `/.config/gates` `.github/scripts/changed.sh` `scripts/test/changed.bats` `scripts/check-gates.sh` `scripts/test/check-gates.bats` `/.config/tasks.toml` `.github/workflows/ci.yml` `/.github/CODEOWNERS`

**Area:** tooling

**`.config/gates` marks each tracked path for each attribute it names, and `mise run lint:gates` refuses a path left unmarked.**
The lint runs in Quality, which runs on every pull request.

**Each kind of work that `ci.yml` can skip takes one attribute in the file.**
The gitattributes format carries more than one attribute on a line.
A line such as `crates/**  build  -scripts` marks a path for two of them.

**Downside:**

- **Every new tracked path that no pattern matches needs a line.** `lint:gates` refuses the pull request that adds the path until `.config/gates` marks it for each attribute.
- **The lint proves that `.config/gates` marks a path, not that the mark is right.** [A path not marked to skip starts the gate](a-path-not-marked-to-skip-starts-the-gate.md) carries what a wrong mark costs.
- **A pattern that reaches the root marks a path before anyone decides it.** `*` or `*.rs` passes the lint for each path it matches. The header of `.config/gates` names that risk, and the code owner reads each pattern.

## Confirmation

`mise run lint:gates` refuses a tracked path that `.config/gates` leaves unmarked for an attribute it names.
Quality runs it on every pull request.
`scripts/test/check-gates.bats` tests `scripts/check-gates.sh`, and `mise run test:shell` runs those tests.
`.github/CODEOWNERS` names `/.config/`, so a change to a mark waits for the code owner.
