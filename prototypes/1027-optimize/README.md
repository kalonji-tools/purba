# Prototype for #1027: the warning under -O, and what tests it

A throwaway branch. It must never merge.

The wheel holds `compile(..., optimize=0)` in assertion's `run`, and three places for the warning.
`PURBA_PROTO_WARN` picks one place:

- `a`: `_main`, before command runs, on every run.
- `b`: command, once the module's text is read.
- `c`: command, once selection keeps a test that runs.

`PURBA_PROTO_MUT=interp` reverts `optimize=0`, as a mutation.
Arms `b` and `c` read `sys.flags` from command, so command takes `pyo3` as a dependency, where today it is a dev-dependency.

## Q1: when does the warning print?

`arms.sh` runs seven modules under `PYTHONOPTIMIZE=1` for each arm, and pytest 9.2.0 as the control.
[`runs/arms.txt`](runs/arms.txt) holds the output.

| module | `a` | `b` | `c` | pytest |
|---|---|---|---|---|
| no path | warns | no | no | n/a |
| `missing.py` | warns | no | no | warns |
| a syntax error | warns | warns | no | warns |
| no test | warns | warns | no | warns |
| each test refused by selection | warns | warns | no | warns |
| a helper that asserts | warns | warns | warns | warns |

pytest prints its warning in the summary at the end, and purba prints it first.

## Q2: which tests catch a revert?

`mutations.sh` runs `tests/optimize.bats` alone (arm `a`), and every bats file with `PYTHONOPTIMIZE=1` exported (arm `b`).
[`runs/mutations.txt`](runs/mutations.txt) holds the output.

| mutation | `optimize.bats` | every file under -O |
|---|---|---|
| none, the change whole | 0 of 5 red, 1 s | 82 of 114 red, 56 s |
| `optimize=0` reverted | 3 of 5 red | 88 of 114 red |
| the warning dropped | 4 of 5 red | 4 of 114 red, the 4 of `optimize.bats` |

With the change whole, 82 tests are red under -O, because the warning is a line on stderr, and those tests match stderr whole or as empty.
With the warning dropped, the 110 other tests pass under -O, so `optimize=0` breaks no other test.
A plain run of every file takes 59 s.
