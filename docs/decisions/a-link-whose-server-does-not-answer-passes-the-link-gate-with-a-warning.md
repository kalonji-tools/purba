# A link whose server does not answer passes the link gate with a warning

## Context and Problem Statement

The online leg of `mise run lint:links` reads each web link in a file that a branch changes.
lychee prints a tag before each link it cannot reach.

| tag | what happened |
|---|---|
| `[404]` | the page is gone |
| `[TIMEOUT]` | the server did not answer in time |
| `[503]` | the server is unavailable |

The gate refused each tagged link.
So a slow server outside purba turned `CI` red.
`Sign-off` then waited for a rerun.

| where | server | what lychee printed | rerun |
|---|---|---|---|
| [the control arm of the CI prototype](https://github.com/kalonji-tools/purba/issues/926#issuecomment-6779485337), pushes 3 and 4 | `github.com` | `[503]` | none |
| [run 43061327780](https://github.com/kalonji-tools/purba/actions/runs/43061327780) | `pyo3.rs` | `[TIMEOUT]`, after 88 s of retries | passed |
| [run 43074911472](https://github.com/kalonji-tools/purba/actions/runs/43074911472) | `pyo3.rs` | `[TIMEOUT]` | passed |

## Considered Options

- **Refuse a link whose server does not answer, as a dead link.** A slow server then holds back the merge of each branch that touches a file with a link to it.

## Decision Outcome

**Reach:** `scripts/check-links.sh` `scripts/report.sh` `scripts/test/check-links.bats` `scripts/test/report.bats` `/pyproject.toml` `/CONTRIBUTING.md`

**Area:** writing

**A link whose server does not answer in time, or answers 503, passes the online leg of the link gate, and a warning names it.**
`check-links.sh` sends each `[TIMEOUT]` line and each `[503]` line to the warning, and every other line that lychee starts with `[` to the refusal.
The warning names each link under the file that holds it, as a refusal does.
[An unpublished comment carries what no other location carries](an-unpublished-comment-carries-what-no-other-location-carries.md) holds the rest of the link gate.

**Downside:**

- **A link to a server that never answers passes on every push.** Only the warning names it.
- **A page that always answers 503 passes too.**
- **A warning stops no job.** Nobody has to read it.

## Confirmation

`scripts/test/check-links.bats` gives the script fakes of lychee that print `[TIMEOUT]`, `[503]` and `[404]` lines.
It expects a warning for the first two tags and a refusal for the third, each link under the file that holds it.
`scripts/test/report.bats` expects a warning to write `::warning::` on a runner, and to leave the exit code at 0.
`mise run test:shell` runs both.
`Scripts` runs that task on a pull request that changes a path that starts the script tests.

[How does `lint:links` tell a dead link from a site that did not answer in time, so that a slow site outside purba does not hold back `Sign-off`?](https://github.com/kalonji-tools/purba/issues/938) holds the runs above.
