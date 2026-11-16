# The test form with checks of an iterable: throwaway prototype

This folder is a copy of `../1015-form/`, with the four changes that round 1 of [How does a test check an iterable: a property of each item, a product that is lazy, and an iterator that a check empties?](https://github.com/kalonji-tools/purba/issues/1025) agreed. It is never merged.
A blind porter read `FORM.md` alone and ported toolz tests into the form, and three blind raters scored the port beside pytest and oxitest. [`benchmark/README.md`](benchmark/README.md) holds the rating.
Everything else about the form and its runner is as `../1015-form/README.md` says.

## What changed since the form of #1015

1. `assert all(<check> for <target> in <reading>)` is a check: one `for`, no `if`, and a body that is one of the form's check shapes. A failure names every item that breaks it, with its index, and the check's reading of the item when it is not the item itself (`probes/test_1025_all.py`).
2. `assert any(…)` is a check of the same shape. A failure lists every item (`probes/test_1025_any.py`).
3. A check never reads a one-shot iterator: a name that the check reads, or the reading of its `all` or `any`, that holds an iterator (`iter(v) is v`) refuses the case (`probes/test_1025_iterators.py`).
4. A raise in the act or in a reading names the frame it came from: `raised inside the generator gen() at lazy.py:6` (`probes/test_1025_raise.py`).

`runs/probes_verify.txt` shows every probe matching its docstring. The fork tree's `chains/` was not run: its folder is not in this worktree.

## Round 1: what was measured before the build

- `round1/base/`: the probes on the wheel of `main` at 449b757 and at 680e274 (after "A failed expectation shows its reading" merged).
- `round1/proto/`: a standalone sketch of three shapes of a check of each item (`all()` rewritten, a trailing `for`, an `each()` proxy), and of the iterator refusal and the raise line. `proto.txt`, `any.txt` and `plain.txt` are their runs.
- `round1/count/`: counts over the 4,436 tests of the pinned suites: `shapes.txt` (each shape of a check of an iterable), `fileobj.txt` (tests whose check reads a name the test bound to a file-like object or an iterator).
- `round1/form-1015-baseline.txt`: the same shapes under the 1015 runner.

## After the rating: a variant of the iterator rule

`variant/` holds a patch of `run.py` (`run.py.diff`) and its run (`variant.txt`, against `base1025.txt` for this form). It lets `all()` and `any()` read an iterator that the check makes itself, as a call, and leaves a file or `StringIO` to the rule of an open file and to the rule that each later check is read alone. The rated form does not hold it.
