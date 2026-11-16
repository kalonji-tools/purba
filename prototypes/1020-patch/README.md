# How a test patches a name, probed

A prototype for [May a test patch a name inside its act, or does a stand-in enter only as an input?](https://github.com/kalonji-tools/purba/issues/1020).
A throwaway branch: it never merges.

- `probes/` holds one module for each way a stand-in enters a test: an argument, `unittest.mock.patch` in a `with`, an assignment to a module's name, a decorator, pytest's `monkeypatch` and `MonkeyPatch.context()`, and the standard library's form of each `MonkeyPatch` method.
  `probes/lending.py` is the code under test.
  `runs/probes.txt` is what the wheel of `main` at 449b757 prints for each, on CPython 3.12.14 with pytest 9.2.0.
- `patches.py` reads every test function of the 9 pinned suites of [the cost of the rules](https://github.com/kalonji-tools/purba/tree/8e712dbffa95985777551b4d9d5c743083c8ba29/prototypes/1015-rules-cost) with `ast`, and counts each form of a patch.
  `runs/patches.txt` holds the counts.
- `runs/greps.txt` lists each `with …patch(` line of the 9 suites, counts each `Mock(…)` that a test file builds and how many give a spec on the same line, and counts the test files that import pytest or `unittest.mock`.

```sh
prototypes/1020-patch/run.sh <venv with the wheel of main and pytest> <folder of the 9 clones>
```

`run.sh` removes the folder of the prototype from each line of `runs/probes.txt`, so an `AttributeError` names `probes/lending.py` and no local path.
`patches.py` counts a test function once for each form it holds. It counts an assignment or a `setattr` to any name that the file imports, so a class that the file imports counts as a module.
The two `grep` counts read one line at a time, so a spec on the next line is not seen.
A probe of a target that does not exist names `lending.Librarian`, a real word: the repository's `typos` hook rewrote a misspelled name to `Library`, and the probe then passed.
