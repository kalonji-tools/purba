# A gate's job counts only by a token that GitHub signed for its run

## Context and Problem Statement

The `sign` job reads a gate's verdict from a job of the gate's file, on `pull_request`, on the branch of the pull request.
Each pull request from that branch makes a run of that file on the same head.
A pull request takes the file from its merge into its base.
So a pull request whose branch leaves the file alone runs the file of its base.

An account with write access can push a base whose file always passes.
No field of a workflow run names the base or the pull request that made it.
Its list of pull requests holds each open one that shares the head, and none once they close.

A probe in a sandbox measured the verdict that `check-run.sh` read before this record, and that a prototype of this record read after it.

| pull requests from one branch | before | after |
|---|---|---|
| A into `main`, `CI` red; then B into a base whose `CI` passes, and B finishes last | `success` | `failure` |
| C into that base, then its base moved to `main`, which starts no run | `success` | `none` |

## Considered Options

- **Count each job of the gate's file, on `pull_request`, on the branch.** A pull request into another base then stands for the gate.
- **Refuse a pull request whose branch another pull request shares.** That option closes only the case of a second pull request. A base that moves, or a fork at the same head, each needs a patch of its own.

## Decision Outcome

**Reach:** `.github/workflows/ci.yml` `.github/scripts/check-run.sh` `.github/scripts/write-acceptance-trailer.sh` `scripts/test/check-run.bats` `scripts/test/fixture.sh` `/.config/mise.toml` `/.config/mise.lock` `/pyproject.toml`

**Area:** workflow

**The `sign` job counts a gate's job only by a token that GitHub signed for its run.**
GitHub signs the token, so no workflow can forge it.
The token names the pull request and the base of its run.

The job `CI` asks GitHub for that token.
It uploads the token as the artifact `gate-token-<attempt>` of its run.
The `sign` job counts the job only when each line holds.

| part | must be |
|---|---|
| signature | RS256, by a key of GitHub's set that is RSA of 2048 bits with the exponent 65537 |
| `iss`, `aud` | the issuer of GitHub Actions, `purba-gate` |
| `repository`, `event_name` | this repository, `pull_request` |
| `ref` | `refs/pull/<n>/merge`, where `<n>` is the pull request that the `sign` job signs |
| `base_ref` | the base of that pull request |
| `run_id`, `run_attempt`, `check_run_id` | the run, the attempt and the job that the `sign` job counts |

The `sign` job reads no `exp`, because the token names one run and grants no access.
A check run of `purba-sign` belongs to no run, and still counts.

- **`bats` runs with OpenSSL 3 from mise, so each machine verifies a token with the same tool.** macOS ships LibreSSL instead. The `sign` job installs no mise, and uses the OpenSSL 3 of its runner.

[`main` requires only `Sign-off`, and accepts it only from `purba-sign`](a-required-check-of-main-counts-only-from-github-actions.md) says which jobs the `sign` job reads.
`id-token: write` is a permission that a job's own token may hold, like `contents: write`.
So [a workflow is granted an identity, not a power](a-workflow-is-granted-an-identity-not-a-power.md) asks for no App here.

**Downside:**

- **A run from before this record, or one whose key GitHub took out of its set, gives no verdict.** A push, or a close and a reopen, makes a new run.
- **A key of another shape gives no verdict.** GitHub may sign with a key of a shape that the table does not name. No gate then counts until `check-run.sh` builds that shape.
- **The token sits in an artifact that anyone can read.** Its audience is `purba-gate`. Its claims name one run, so the `sign` job counts it for that run alone.
- **The `sign` job trusts `token.actions.githubusercontent.com` at run time.** When that host does not answer, the `sign` job counts no gate. `write-acceptance-trailer.sh` then exits 2 before it writes anything, so the `sign` job fails. When `carry-verdicts.sh` cannot read the key set, it writes a failed `Carry`.
- **A `Sign-off` belongs to the head, not to the pull request.** A `sign` run for one pull request leaves a `Sign-off` that stands on each pull request at that head. A `sign` run from before a base moved leaves one that stands after the move. [A Sign-off that the sign job writes for one pull request stands on each pull request at the same head](https://github.com/kalonji-tools/purba/issues/949) holds it.

## Confirmation

`scripts/test/check-run.bats` gives `check-run.sh` a token for each claim that does not hold.
It also gives one for each signature that is not RS256 by a key of the set.
It gives one more that a key of the set signed, but that key is too short.
It expects no verdict from each.
`mise run test:shell` runs it.

[A pull request from the reviewed branch into another base makes a run that counts as a gate](https://github.com/kalonji-tools/purba/issues/899) holds the probe, on `df614c8`.
No probe measured a run on a pull request from Dependabot, so whether that run gets a token is unknown.
