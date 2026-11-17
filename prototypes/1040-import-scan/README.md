# Where a test file imports pytest, scanned

A prototype for [Refuse a test module that imports pytest, as a strict check of the practice tier](https://github.com/kalonji-tools/purba/issues/1040).
A throwaway branch: it never merges.

It measures the cost of four questions of the spec on the test files of the 9 pinned suites of [the cost of the rules](https://github.com/kalonji-tools/purba/tree/8e712dbffa95985777551b4d9d5c743083c8ba29/prototypes/1015-rules-cost), at the commits of its `projects.txt`.

- Q3: how many lines of one file import pytest or `_pytest`.
- Q4: how many tests an import in a helper, or in a class, would reach alone.
- Q6: which calls of an importing function name pytest, matched through the imports or by the called name alone.
  The functions are the ones [the research of #1040](https://github.com/kalonji-tools/purba/blob/f62081253d3eab303cab37779ba9a6a2ea1dac01/research/unwanted-import.md) lists, and `find_spec` counts only for a dotted name.
- Q7: how many files import pytest only under `TYPE_CHECKING`.

`scan.py` reads each file with Python's `ast`, not with ruff's parser that purba uses.
`control/` holds modules whose answers are known; `runs/control.txt` is the scan of them.
`runs/scan.txt` is the scan of the 9 suites.

```sh
<venv>/bin/python -I prototypes/1040-import-scan/scan.py <folder of the 9 clones> > prototypes/1040-import-scan/runs/scan.txt
<venv>/bin/python -I prototypes/1040-import-scan/scan.py prototypes/1040-import-scan/control > prototypes/1040-import-scan/runs/control.txt
```

The scan ran on CPython 3.12.14.
