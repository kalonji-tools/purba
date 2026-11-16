# Must a test that expects a raise name its message as well as its type?

The numbers behind [#1021](https://github.com/kalonji-tools/purba/issues/1021).
This branch is never merged.

The corpus is the 9 pinned suites of [#1010](https://github.com/kalonji-tools/purba/issues/1010): attrs, click, dateutil, loguru, pyflakes, requests, schedule, six and toolz.
The unit is one item of a `with` whose call ends in `raises`, `warns`, `assertRaises`, `assertRaisesRegex`, `assertWarns` or `assertWarnsRegex`, inside a `def test*` of a file named `test*.py`.

## Counts by `ast`

- [`py5.py`](py5.py) → [`runs/py5.txt`](runs/py5.txt): how each expected raise or warning names more than its type.
- [`py5b.py`](py5b.py) → [`runs/py5b.txt`](runs/py5b.txt): what a check of `caught.value` reads, and how many `raise` statements of the same type name sit in the project's own code.
- [`fullmatch.py`](fullmatch.py) → [`runs/fullmatch.txt`](runs/fullmatch.txt): whether a literal `match=` covers the whole message that the baseline run caught, or a part of it.

## The mutation run

[`mutation/scripts`](mutation/scripts) copies each project, runs its tests once, and records the `raise` statement whose exception reached each `with`.
Then it replaces that one `raise` with `pass` and runs only that test.
A test that still passes caught another exception of the same type.
[`mutation/runs/summary.txt`](mutation/runs/summary.txt) holds the totals, and [`survivors.json`](mutation/runs/survivors.json) each test that passed.

The scripts hold the absolute paths of the session that ran them, under `/tmp`.

Bounds:

- one mutant per item, at the raise site of its first execution;
- only a `raise` statement in the project's own code that is not a test, and only the deepest frame;
- the outcome is the whole test's, not the item's;
- the `match=` that would have killed a survivor is a text check of the two messages, not a run;
- the call form `raises(T, fn, *args)` is not counted.
