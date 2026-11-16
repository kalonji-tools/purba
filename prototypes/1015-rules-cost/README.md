# The cost of the rules on real tests (#1015) — throwaway, never merge

This prototype measures what the rules of `main` at f0c787f refuse in real test suites, and how often real tests touch nine practices.
It answers no question by itself. It is the tester's side of a cost-benefit for [#1015](https://github.com/kalonji-tools/purba/issues/1015).

## How it runs

`run.sh <folder>` clones the nine suites of `projects.txt` at their pinned commits, the same commits as the survey of #1010, and writes each count to `runs/`.
Nothing in the clones is imported or run.

- `crates/selection/examples/tally.rs` calls `purba_selection::select` of `main` on each test file, and prints one line for each test: selected, or refused with every reason (`runs/tally.tsv.gz`). It is stored gzipped, because the text gates of the repository rewrite the spelling of some test names; `zcat` reads it.
- `reasons.sh` counts the tests that each reason refuses (`runs/reasons.txt`).
- `origin.py` says where each bare name that `computes …` refuses comes from (`runs/origin.txt`).
- `shapes.py` sorts the asserts that `is not one comparison` refuses, and counts each private name (`runs/shapes.txt`).
- `runtime.py` gives an `ast` upper bound for the rules that execution applies while a case runs (`runs/runtime.txt`).
- `practices.py` counts, over every `def test…` at module level or in a class, the signals of nine practices (`runs/practices.txt`).

## What it cannot see

- Selection reads only the tests at module level, so pyflakes and schedule, whose tests sit in classes, give it none. 1,901 tests of 7 suites reach it. `practices.py` reads 4,404 tests of all 9.
- `runtime.py` counts a branch or a loop by syntax. Execution refuses one only when it went one way, or ran no round, so each count there is at most the true count.
- Each count of `practices.py` is a sign in the syntax, not a proof that the test breaks the practice. Most of the 231 "touches the network" tests of requests call a local test server, and `requests.Session()` opens no socket.
- The suites are pytest and `unittest` tests. A suite written for purba's form would look different, so a count is what a port starts from, not what it ends with.
