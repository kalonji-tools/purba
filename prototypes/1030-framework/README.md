# A test module that imports pytest, probed

A prototype for [May a test module import another test framework, such as pytest?](https://github.com/kalonji-tools/purba/issues/1030).
A throwaway branch: it never merges.

- `probes/` holds one module for each way a test module reads pytest: the import alone, a `from` import that no test reads, a helper under `@pytest.fixture`, `MonkeyPatch.context()`, `raises`, `warns`, `skip()` in a test and at the module's level, `fail()`, `importorskip()`, and `mark.parametrize` with and without a suppression.
  Three more import `hypothesis`, `unittest` and `unittest.mock`.
  `probes/shop.py` is the code under test.
  `runs/probes.txt` is what the wheel of `main` at 88a2c9d prints for each, on CPython 3.12.14 with pytest 9.2.0 and hypothesis 6.170.15.
- `sidefx.py` imports one module and compares what the interpreter holds before and after: the import hooks, the warning filters, `showwarning`, `excepthook`, the handlers of `SIGINT` and `SIGALRM`, the threads, the `atexit` callbacks, the trace and profile functions and the modules loaded.
  `runs/sidefx.txt` holds it for `pytest`, `hypothesis`, `unittest` and `unittest.mock`.
- `counts.sh` counts the test files of the 9 pinned suites of [the cost of the rules](https://github.com/kalonji-tools/purba/tree/8e712dbffa95985777551b4d9d5c743083c8ba29/prototypes/1015-rules-cost) that import each test framework or library, and each pytest name that they read.
  `runs/counts.txt` holds the counts.
- `pertest.py` reads, with `ast`, each test function at the top level of a test file that imports pytest, and sorts it by whether it reads a name that the import binds, takes a parameter with no default, or does neither.
  `runs/pertest.txt` holds the counts.

```sh
PURBA=<venv>/bin/purba prototypes/1030-framework/run.sh > prototypes/1030-framework/runs/probes.txt
prototypes/1030-framework/counts.sh <folder of the 9 clones> > prototypes/1030-framework/runs/counts.txt
<venv>/bin/python -I prototypes/1030-framework/sidefx.py pytest
<venv>/bin/python -I prototypes/1030-framework/pertest.py <folder of the 9 clones>
```

A test file is a file named `test_*.py` or `*_test.py`.
[The probes of #1020](https://github.com/kalonji-tools/purba/blob/8b0af480c75100ac141a7abe7862042d5d092fcc/prototypes/1020-patch/runs/greps.txt) count 125 files that import pytest, since they read every `.py` file; five `conftest.py` files and `dateutil/tests/_common.py` are the 6 that `counts.sh` leaves out.
`counts.sh` reads one line at a time, so `importlib.import_module("pytest")` is not an import to it.
A test that `pertest.py` calls neither is not a test that passes: another check may refuse it.
