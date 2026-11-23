# #1213 prototype: what each rule does on a real run

This branch is never merged. Each cell below comes from a run of an installed wheel, and names its run file.
No cell is predicted.

The three rules give different outcomes in five scenarios: S1, S2-project10, S3, S4 and S9.
In S1, `higher-overwrites` and `code-wins` let `--suppression-cap 50` undo the project's cap of 0, and `stricter-wins` keeps the cap.
In S3, only `code-wins` lets the test's own 6 s beat `--timeout 1`.
In S2 (project 10 s, user file 1 s), S4 (user 5 s, system 1 s) and S9 (user 0 s, project 1 s), `stricter-wins` lets a file outside the project decide the outcome.
In S9, the "strictest" value is 0, and 0 arms no timer, so `stricter-wins` turns the check off.
The fence refuses each of these files with exit 3, before any test runs.
`drop` removes the user file only when a `pyproject.toml` holds `[tool.purba]`.
In S2-dev, the project has a `pyproject.toml` with no `[tool.purba]`, so the user file still fails the test on the developer's machine and not in CI.

## Wheels

| wheel | commit | how it was built |
|---|---|---|
| baseline | `27e8f559` (main) | `mise run build` on a clean tree, installed with `pip install --no-index --find-links` into a scratch venv (as `test:wheel` does) |
| prototype | `fedc6c12` | the same, from the commit of the code, with a clean tree under `crates/`, `Cargo.*` and `pyproject.toml` |

Python 3.12.14, Linux. The harness is [`run.sh`](run.sh):

```sh
proto/1213/run.sh <venv>/bin/purba proto/1213/runs <work>             # prototype
proto/1213/run.sh <venv>/bin/purba proto/1213/runs/baseline <work> baseline
```

Each run file holds the command, the environment, every configuration file, the module, stdout, stderr, the exit code and the seconds the run took.
The harness replaces the scratch directory of the run with `<tmp>`.
Each run has its own `HOME`, `XDG_CONFIG_HOME` and `PURBA_PROTO_SYSTEM`, and starts from `env -i`.

## The grid

The module of every timeout scenario holds `test_slow`, which sleeps 2 s, and `test_quick`.
A cell reads: the resolved value ← its level · what the tester sees for `test_slow` · the exit code.
"raised (SIGALRM)" is purba's existing line `test_mod.py::test_slow raised: its process died by SIGALRM`.
A run that the alarm ended took 1.0 s, and a run that passed took 2.0 s (the `--- seconds` line).

| scenario | baseline (main) | `higher-overwrites` | `code-wins` | `stricter-wins` |
|---|---|---|---|---|
| **S0** no configuration | passed · 0 | no dial line · passed · 0 | no dial line · passed · 0 | no dial line · passed · 0 |
| **S1** project cap 0, 1 suppression, `--suppression-cap 50` | usage error · 3 | 50 ← command line · passed, 1 suppressed · 0 | 50 ← command line · passed, 1 suppressed · 0 | 0 ← project · module stopped: `1 suppressions, and the cap is 0`; test blocked · 4 |
| **S1-no-flag** project cap 0, 1 suppression | key ignored · passed, 1 suppressed · 0 | 0 ← project · stopped · 4 | 0 ← project · stopped · 4 | 0 ← project · stopped · 4 |
| **S2-dev** user 1 s, `pyproject.toml` without `[tool.purba]`, `merge` | passed · 0 | 1 ← user · raised (SIGALRM) · 4 | 1 ← user · raised (SIGALRM) · 4 | 1 ← user · raised (SIGALRM) · 4 |
| **S2-dev**, `drop` | (as above) | 1 ← user, not dropped · raised (SIGALRM) · 4 | 1 ← user · raised (SIGALRM) · 4 | 1 ← user · raised (SIGALRM) · 4 |
| **S2-ci** no user file | passed · 0 | no dial line · passed · 0 | no dial line · passed · 0 | no dial line · passed · 0 |
| **S2-project10** project 10 s, user 1 s, `merge` | passed · 0 | 10 ← project · passed · 0 | 10 ← project · passed · 0 | **1 ← user · raised (SIGALRM) · 4** |
| **S2-project10**, `drop` | (as above) | 10 ← project; "user file is dropped" · passed · 0 | 10 ← project · passed · 0 | 10 ← project · passed · 0 |
| **S3** project 4 s, code 6 s, `--timeout 1` | usage error · 3 | 1 ← command line · raised (SIGALRM) · 4 | **6 ← code (line 4) · passed · 0** | 1 ← command line · raised (SIGALRM) · 4 |
| **S3-no-flag** project 4 s, code 6 s | test refused: `line 4 is a purba comment that reads as no suppression…` · 4 | 6 ← code · passed · 0 | 6 ← code · passed · 0 | 4 ← project · passed · 0 |
| **S4** user 5 s, system 1 s, `open` | passed · 0 | 5 ← user · passed · 0 | 5 ← user · passed · 0 | **1 ← system · raised (SIGALRM) · 4** |
| **S4**, `fenced` | (as above) | invocation error: `the user file … sets timeout, and the fence closes this level to a dial that can change an outcome` · 3 | same · 3 | same · 3 |
| **S5** project `tmeout = 5` | key ignored · passed · 0 | invocation error: ``unknown key `tmeout`, did you mean `timeout`?`` · 3 | same · 3 | same · 3 |
| **S6** `PURBA_TIMEOUT=1`, project 10 s | passed · 0 | 1 ← environment · raised (SIGALRM) · 4 | 1 ← environment · raised (SIGALRM) · 4 | 1 ← environment · raised (SIGALRM) · 4 |
| **S7** project `retries = 2`, `open` | key ignored · passed · 0 | `retries = 2` ← project, printed only · passed · 0 | same · 0 | same · 0 |
| **S7**, `fenced` | (as above) | `retries = 2` ← project, printed only · passed · 0 | same · 0 | same · 0 |
| **S8** user `workers = 4`, `open` | passed · 0 | `workers = 4` ← user, printed only · passed · 0 | same · 0 | same · 0 |
| **S8**, `fenced` | (as above) | `workers = 4` ← user · passed · 0 | same · 0 | same · 0 |
| **S9** user 0 s, project 1 s | passed · 0 | 1 ← project · raised (SIGALRM) · 4 | 1 ← project · raised (SIGALRM) · 4 | **0 ← user · passed (no timer) · 0** |
| **S10** code `set[suppression-cap] 5`, `open` and `fenced` | test refused: comment reads as no suppression · 4 | invocation error: `code sets suppression-cap, and code may set only timeout` · 3 | same · 3 | same · 3 |

`test_quick` passed in every run that ran the timeout module's tests.
S1-no-flag, S2-ci, S3-no-flag, S9 and S10 are rows this prototype added.

Run files: `runs/<scenario>.<rule>[.<fence or user>].txt`, and `runs/baseline/<scenario>.baseline.txt`.
For example, the S3 `code-wins` cell is [`runs/S3.code-wins.txt`](runs/S3.code-wins.txt), and the S4 `fenced` cell under `stricter-wins` is [`runs/S4.stricter-wins.fenced.txt`](runs/S4.stricter-wins.fenced.txt).

## Provisional choices

The prototype made each of these choices. None of them is decided.

- Six levels: the command line, the environment (`PURBA_<DIAL>`), code (`# purba: set[timeout] <s>` above a `def`), the project (`[tool.purba]` of the first `pyproject.toml` that holds it, walking up from the module's directory, with no stop at the repository root), the user (`$XDG_CONFIG_HOME/purba/purba.toml`, else `$HOME/.config/purba/purba.toml`), and the system (`/etc/purba/purba.toml`, which `PURBA_PROTO_SYSTEM` replaces).
- Four dials: `timeout` (seconds), `suppression-cap`, `workers` and `retries`. `workers` and `retries` are resolved and printed only.
- Rules: `higher-overwrites` is command line > environment > code > project > user > system. `code-wins` puts code first. `stricter-wins` takes the smallest value at any level for `timeout` and `suppression-cap`. On a tie, it names the level that `higher-overwrites` ranks first.
- Under `stricter-wins`, 0 is the smallest timeout, and a timeout of 0 arms no timer.
- The fence `fenced` closes the user file and the system file to `timeout`, `suppression-cap` and `retries`. Code may set only `timeout` under both fences.
- `drop` ignores the user file only when a project file with `[tool.purba]` exists. A dropped file is not read, so its unknown keys and fenced keys are not checked.
- Errors: an unknown key, a closed level, a bad value, a bad switch and a `set[...]` comment above no `def` are each an invocation error with exit 3. The first error stops the read, so S4 `fenced` names the user file and never the system file.
- The nearest dial is offered when its edit distance is 2 or less.
- `suppression-cap` counts the module's suppressions without a fault: above a test or of the module. A count above the cap stops the module through the existing `stopped` path, with `<n> suppressions, and the cap is <m>`.
- `timeout` calls `libc::alarm` in each test's process at the fork. SIGALRM's default action ends the process. A replay's process and a process that loads the module itself also get the alarm, but no scenario reaches either.
- Selection's reader of `# purba:` comments skips `# purba: set[`, so that a code setting is no fault.
- stderr carries a header line, one line for each dial that a level sets, one line for each test that code sets a timeout on, and a line when `drop` ignores a user file. stdout is unchanged.
- A flag takes its value as the next argument (`--timeout 1`); `--timeout=1` is a usage error.
- The misspelling in S5 is `tmeout`. The `typos` hook rewrites the more common misspellings to `timeout` in place, also in run files, so S5 uses one that `typos` does not know.

## What surprised me

- **`stricter-wins` turns the check off in S9.** A timeout tightens as it gets lower, but 0 means "no timer". The smallest value at any level is then the loosest one. The rule needs a floor, or 0 needs another meaning.
- **`stricter-wins` gives the user file and the system file the last word on a tightening dial** (S2-project10, S4). The project's 10 s loses to a developer's 1 s, so the test fails on one machine only. `drop` or `fenced` stops it; the rule alone does not.
- **`stricter-wins` also stops a test from asking for more time.** In S3-no-flag the test sets 6 s in code, and it gets the project's 4 s.
- **`drop` keys on `[tool.purba]`, not on `pyproject.toml`.** In S2-dev, a project that never configured purba still lets the user file fail its test. The developer's run and CI differ under every rule and both user switches.
- **Today's purba refuses a `# purba: set[...]` comment.** On main, the comment is a project error and the test is blocked (S3-no-flag, S10 baseline). A level in code must change selection's reader.
- **purba's line for a timed-out test does not say "timeout".** It reads `raised: its process died by SIGALRM`, and the exit is 4, as for any raised test. It names neither the seconds nor the level.
- **The cap's stop says the module "did not load".** `test_round_trip blocked: test_mod.py did not load` follows the cap's error, because the `stopped` path writes it. The module did load. "1 suppressions" is also not singular.
- **`code-wins` and `higher-overwrites` differ only when a flag or a variable meets a code setting.** In this grid, that is S3 alone.
- **On main, every file is silent.** The baseline ignores `tmeout`, `retries` and every other key with exit 0.

## The cells most worth checking again

1. S9 `stricter-wins` (`runs/S9.stricter-wins.txt`): run it with `PURBA_PROTO_RULE=stricter-wins`, a user file with `timeout = 0` and a project file with `timeout = 1`. Expect 2.0 s and a pass. Then set the user file to 1, and expect 1.0 s and SIGALRM.
2. S3 `code-wins` (`runs/S3.code-wins.txt`): check that 6 s comes from line 4, and that `test_quick` gets 1 s. Change the sleep to 7 s and expect SIGALRM at about 6 s.
3. S2-dev `drop` (`runs/S2-dev.higher-overwrites.drop.txt`): add `[tool.purba]` with no key to the project's `pyproject.toml`. Expect the "dropped" line and a pass.
4. S1 `stricter-wins` (`runs/S1.stricter-wins.txt`): remove the suppression, and expect the cap to pass with `suppression-cap = 0`. Add a second suppression with `--suppression-cap 1`, and expect `2 suppressions, and the cap is 1` under `stricter-wins` only.
5. S4 `fenced` (`runs/S4.stricter-wins.fenced.txt`): remove the user file. Expect the error to name the system file. With neither file, expect no error.
