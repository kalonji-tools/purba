# purba supports a Python that has two years left

## Context and Problem Statement

purba's Python floor was 3.11 and no location held a reason for it.
A floor with no reason reads as a decision to the next person who finds it.

purba holds no product code, so the floor is a claim about a release that does not exist yet.
A version that ends soon after that release costs support and returns nothing.

PEP 693 states the lifetime: security fixes run for five years after a version's `.0` final.

| version | `.0` final | ends |
|---|---|---|
| 3.11 | 2022-10-24 | 2027-10 |
| 3.12 | 2023-10-02 | 2028-10 |
| 3.13 | 2024-10-07 | 2029-10 |
| 3.14 | 2025-10-07 | 2030-10 |

## Considered Options

- **A version chosen now.** Rejected. 3.11 was chosen that way, and because nothing recorded why, no reader could tell the decision from a placeholder.
- **The oldest version still supported at purba's first release.** Rejected. The release date is unknown, so the condition cannot decide. This map refused an undecidable condition five times.
- **A fixed offset below the newest version.** Rejected. It computes the same floor today and states no reason. It moves whenever the newest version moves, and nobody can say whether that was intended.
- **The oldest version that still has a fixed span of life left.** Chosen. It is the reason written as a rule, so the floor recomputes without anybody choosing a version.

## Decision Outcome

purba's Python floor is the oldest CPython that still has two years of support left.

The rule computes the floor, and `pyproject.toml` declares what it computes today.
This record does not repeat that version, because it moves and the rule does not.

**The span is two years.**
It is long enough that a version cannot die in the months after purba first ships.
It is short enough that the floor does not outrun the versions people run.

**The floor is recomputed when the version below it crosses the span.**
That is an event in the release calendar rather than a date in this file.

**A rise of the floor is a breaking change.**
Its commit is a `feat` that carries `!`, so `.config/cliff.toml` gives it the version a breaking feature earns.

**`requires-python` carries no upper bound.**
An abi3 wheel loads on a version that did not exist when it was built.
So the claim above the floor is true, and nothing tests every version that satisfies it.
[Decide purba's release strategy](https://github.com/kalonji-tools/purba/issues/24) split the word that governs this: tested is what CI runs, and supported is what a wheel exists for.

**Each location below carries the floor.**

| location | what it sets | when the floor rises without it |
|---|---|---|
| `pyproject.toml` | the version a resolver refuses below | `.github/workflows/publish.yml` refuses the next upload |
| `Cargo.toml` | the abi3 feature, which sets the wheel tag | `.github/workflows/publish.yml` refuses the next upload |
| `.config/mise.toml` | the interpreter `mise run` builds against | `mise run check` refuses, because pyo3 compares the abi3 feature with that interpreter |
| `.config/mise.lock` | the version that interpreter resolves to | mise refuses a version the lockfile does not hold |
| `.github/workflows/build.yml` | the interpreters the wheel matrix builds on | the arm below the floor fails, because pip refuses its wheel |

**Downside:**

- **Only the upload refuses a disagreement between `requires-python` and the abi3 feature.** A person who moves one and leaves the other passes every check before the merge. The next release then fails.
- **Nothing refuses a matrix without its floor arm.** No job then builds the wheel on the floor.
- **Nothing refuses a build on an interpreter below the floor.** `maturin build --interpreter` writes a version-specific wheel and a warning. `mise run build` uses the interpreter mise supplies, so only a build that names another interpreter meets this.
- **The span is a judgement and this record freezes it.** No measurement chose two years, so a later reader can only find the reasoning above, never a number that settles it.
- **A free-threaded reader finds no wheel.** Every free-threaded build refuses an abi3 wheel, and `requires-python` still reads as yes. The free-threaded arm of `build.yml` loads a wheel built for that interpreter alone, and no release ships that wheel. The repair is the `abi3t` feature structure, which waits on the seam direction.

## Confirmation

A floor is proved by a wheel that builds on it and then loads on it.

[Does the wheel matrix carry free-threaded builds?](https://github.com/kalonji-tools/purba/issues/63) decided this floor and the arms that exercise it.
`.github/workflows/build.yml` runs those arms. Its first arm is the floor.
`.github/workflows/publish.yml` refuses to upload a wheel unless its tag is abi3 at the floor that `requires-python` names.
`build.yml` reads no tag, so a wheel built without the abi3 feature passes it.
