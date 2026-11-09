# A finished gate runs `sign` with the scripts of `main`

## Context and Problem Statement

[Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md) starts the `sign` job when a gate finishes.
GitHub starts that run on `main`, with this repository's own token.
The job checks out the branch of the pull request.
It ran `write-acceptance-trailer.sh` from that checkout before it read any approval, and the script ran helpers from `scripts/` in it.

[A run can restore a cache that the default branch saved](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching#restrictions-for-accessing-a-cache), so each gate, the release and Publish restore a cache that a run on `main` saves.
So the code of a branch ran where a cache it saves reaches the wheel that Publish uploads.
[An outside contribution is applied, not merged](an-outside-contribution-is-applied-not-merged.md) carries outside work onto such a branch.

## Considered Options

- **The scripts of the branch, on all three events.** The code of a branch runs on `main` before anyone approves it.
- **No job restores a cache that `main` holds.** The code of a branch still runs on `main`.

## Decision Outcome

**Reach:** `.github/workflows/sign.yml` `.github/scripts/write-acceptance-trailer.sh` `.github/scripts/carry-verdicts.sh` `scripts/check-origin.sh` `scripts/check-replayable.sh` `scripts/test/write-acceptance-trailer.bats`

**Area:** tooling

**When a gate finishes, `sign` runs the scripts of `main`, and reads the branch only through git.**
No file of the branch runs on `main`.

- **A step copies `.github/scripts/` and `scripts/` of `main` outside the checkout.** The job runs that copy, and the working directory stays the branch.
- **Each script calls the scripts it needs beside itself.** A script that the working directory holds is a script of the branch.
- **On the pull request's own events, `sign` runs the scripts of the branch.** GitHub starts those runs on the merge ref of the pull request, so a cache they save belongs to that ref.

**Downside:**

- **A change to a script of `sign` does not run when its own gate finishes.** It runs on the other events of its pull request, and on the gate-completed event after it merges.

## Confirmation

`scripts/test/write-acceptance-trailer.bats` tests that the script runs the checks beside itself and not the ones in the working directory, and `mise run test:shell` runs those tests.
No test runs `sign.yml`.
[A gate finishing runs a branch's code where a cache it saves is restored by the gates and Publish](https://github.com/kalonji-tools/purba/issues/890) records the probe that checks it.
