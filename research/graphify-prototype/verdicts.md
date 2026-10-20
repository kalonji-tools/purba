# The breaches each run reported

Read from the final report of each run.
Every run judged no line on the third list stale.

| pull request | arm | run | known breach | other breaches |
|---|---|---:|---|---|
| #597 | as it is today | 1 | missed | none |
| #597 | as it is today | 2 | found: `post-record-thread.sh:9-10`, `report-run-failure.sh:129-130`, `fixture.sh:104-105` | none |
| #597 | link map | 1 | missed: saw two of the comments, and ruled them admissible | none |
| #597 | link map | 2 | missed | none |
| #597 | concept graph | 1 | found: all three comments | none |
| #597 | concept graph | 2 | missed: saw the three comments, and ruled them admissible | none |
| #594 | as it is today | 1 | found: `check-placement.bats:246` | a judgement call: `check-placement.bats:223-225` repeats the script comment |
| #594 | as it is today | 2 | found | none; it ruled `check-placement.bats:223-225` out |
| #594 | link map | 1 | found | none; it ruled `check-placement.bats:223-225` out |
| #594 | link map | 2 | missed: did not flag line 246 | none |
| #594 | concept graph | 1 | found | a judgement call: `check-placement.bats:223-225` repeats the script comment |
| #594 | concept graph | 2 | found | a judgement call: pipe(7) is cited by name, not by link |
| #504 | as it is today | 1 | found: `build.yml:18`, `quality.yml:13` | none |
| #504 | as it is today | 2 | found | none |
| #504 | link map | 1 | found | false: "seventy-two" at `a-gate-owns-the-mechanical.md:90`, which the original pass ruled a frozen number |
| #504 | link map | 2 | missed: ruled that a bare URL is its own reason | none |
| #504 | concept graph | 1 | found | none |
| #504 | concept graph | 2 | found | none |
