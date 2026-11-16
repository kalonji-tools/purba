# How a tester switches a test off, probed

A prototype for [How does a tester switch a test off with its reason, and what does the run show for it?](https://github.com/kalonji-tools/purba/issues/1019).
A throwaway branch: it never merges.

- `probes/` holds one module for each shape of a skip that a suite from pytest carries.
  `runs/probes.txt` is what the wheel of `main` at 449b757 prints for each, on CPython 3.12.14.
- `xfail/test_xf.py` runs a lax and a strict `xfail` under pytest 9.2.0, on a bug that is still there and on one that is fixed.
  `runs/xfail.txt` holds the summary and the exit code of each.
- `find_spec.py` shows what `importlib.util.find_spec` and `pytest.importorskip` do with a missing module, a broken install and a submodule of a missing package.
  `runs/find_spec.txt` holds it.
- `skips.py` reads every test file of the 9 pinned suites of [the cost of the rules](https://github.com/kalonji-tools/purba/tree/8e712dbffa95985777551b4d9d5c743083c8ba29/prototypes/1015-rules-cost) with `ast`, and splits each switch-off by its site, kind, condition and reason.
  `runs/skips.txt` holds the counts.

```sh
prototypes/1019-skip/run.sh <venv with the wheel of main and pytest> <folder of the 9 clones>
```

The condition column of `skips.py` matches words in the condition's source, so it is a rough split. A decorator that a module imports from elsewhere is counted only when its name holds `skip`, `needs`, `requires`, `only` or `xfail`.
