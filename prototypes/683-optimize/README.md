# Keep the asserts of a test module under `-O`

Evidence for [What does purba print when Python strips every assert?](https://github.com/kalonji-tools/purba/issues/683).
This branch is never merged.

## The change

`crates/assertion/src/lib.rs` passes `optimize=0` to `compile()`, so the test module that purba loads keeps its asserts under `python -O` and `PYTHONOPTIMIZE`.
A module that the test imports follows the interpreter.

## Probes

`probes/` holds one test module for each shape.
`run.sh <purba of main> <purba with optimize=0>` runs each one plainly and under `PYTHONOPTIMIZE=1`, and `test_doc` under `PYTHONOPTIMIZE=2`.
`runs/probes.tsv` holds the output of two wheels that `mise run build` made in one worktree: one from `main` at f0c787f, and one with the change, on CPython 3.12.14.

`compile_optimize.py` shows what `compile(..., optimize=0)` does in plain Python under `-O`: the assert stays, and `__debug__` reads `True`.

## The suite

`runs/suite.txt` holds the counts of `mise run test:wheel` with the change, plainly and under `PYTHONOPTIMIZE=1`, and of `main` under `PYTHONOPTIMIZE=1`.
