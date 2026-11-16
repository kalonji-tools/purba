# A check with no literal value

Evidence for [How does a test check a rule that has no literal value, such as a round trip or a property of every item?](https://github.com/kalonji-tools/purba/issues/1016).
This branch is never merged.

## Probes

`probes/` holds one test module for each shape of a check.
`run.sh` runs each one under the `purba` on `PATH`.
`runs/probes.txt` holds the output of the wheel that `mise run build` made from `main` at f0c787f.

## Count

`count/shapes.py` reads the 9 pinned suites of [the cost of the rules](https://github.com/kalonji-tools/purba/tree/8e712dbffa95985777551b4d9d5c743083c8ba29/prototypes/1015-rules-cost) with `ast`, and takes the same 4,404 test functions.
It writes `runs/per_suite.tsv`: one row for each shape, one column for each suite.
A row that starts with `T_` counts tests; any other row counts asserts.

Run it as `python -I count/shapes.py <corpus> <out>`.
The matchers are heuristics: a round trip in one statement needs a bare name on the expected side, and a round trip across statements follows only assignments of one name.
