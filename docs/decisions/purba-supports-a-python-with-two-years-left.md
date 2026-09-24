# purba supports a Python that has two years left

## Context and Problem Statement

purba's Python floor was 3.11 and no location held a reason for it.
A floor with no reason reads as a decision to the next person who finds it.

purba holds no product code, so the floor is a claim about a release the project has not reached.
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
- **The oldest version still supported at purba's first release.** Rejected. The release date is unknown, so the condition cannot decide. This map has refused an undecidable condition five times.
- **A fixed offset below the newest version.** Rejected. It computes the same floor today and states no reason, so it moves whenever the newest version moves and nobody can say whether that was intended.
- **The oldest version that still has a fixed span of life left.** Chosen. It is the reason written as a rule, so the floor recomputes without anybody choosing a version.

## Decision Outcome

purba's Python floor is the oldest CPython that still has two years of support left.

The rule computes the floor, and `pyproject.toml` declares what it computes today.
This record does not repeat that version, because it moves and the rule does not.

**The span is two years.**
It is long enough that a version cannot die in the months after purba first ships, and short enough that the floor does not outrun the versions people run.

**The floor is recomputed when the version below it crosses the span.**
That is an event in the release calendar rather than a date in this file.

**`requires-python` carries no upper bound.**
An abi3 wheel loads on a version that did not exist when it was built, so the claim above the floor is true without testing every version that satisfies it.
[Decide purba's release strategy](https://github.com/kalonji-tools/purba/issues/24) split the word that governs this: tested is what CI runs, and supported is what a wheel exists for.

**Three locations carry the floor and move together.**

| location | what it sets |
|---|---|
| `pyproject.toml` | the version a resolver refuses below |
| `Cargo.toml` | the abi3 feature, which sets the wheel tag |
| `mise.toml` | the interpreter a contributor and CI build against |

**Downside:**

- **A contributor whose machine runs the version below the floor cannot build purba.** The build refuses them, and the repair is to install another interpreter rather than to change a line.
- **Nothing reads the three locations together.** A person who moves one floor and leaves the others creates a disagreement that no check reports, and the first symptom is a wheel tagged for a version the metadata refuses.
  [Can the Python floor be declared in one location instead of three?](https://github.com/kalonji-tools/purba/issues/181) asks whether the three can become one.
- **The span is a judgement and this record freezes it.** No measurement chose two years, so a later reader can only find the reasoning above, never a number that settles it.
- **A free-threaded reader on 3.15 and later finds no wheel.** That interpreter refuses an abi3 wheel, `requires-python` still reads as yes, and the repair is the `abi3t` feature structure, which waits on the seam direction.

## Confirmation

A floor is proved by a wheel that builds on it and then loads on it.

[Does the wheel matrix carry free-threaded builds?](https://github.com/kalonji-tools/purba/issues/63) decided this floor and the arms that exercise it.
[Write the test workflow](https://github.com/kalonji-tools/purba/issues/42) owns the matrix that runs them, and that matrix is not wired.
