# PROTOTYPE 5: a stress test of prototype 4's form (#967). Never merge.

Throwaway. Prototype 5 changes nothing in the form: `FORM.md`, `cases.py` and `run.py` are prototype 4's, byte for byte ([`63e7735`](https://github.com/kalonji-tools/purba/tree/63e77357b64d8f6ad33cfcafd16ee74e57ecdbe7/prototypes/967-python-form-4)).
It ports three more loguru test files, each chosen to press the form on a different axis. The question is where the form holds, where it breaks, and whether #967 needs another iteration.

- [`LEARNINGS.md`](LEARNINGS.md): what held, what broke, what the strategy learns, and the case for closing or iterating.
- `stress/<axis>/pytest/tests/`: the loguru original at Delgan/loguru@28349be (MIT), with a trimmed copy of loguru's `conftest.py`.
- `stress/<axis>/prototype-5/`: the port, by an independent porter who got `FORM.md` and the rules of the brief only. `NOTES.md` is the porter's full record: the first attempt and what the runner said, each forced change, fidelity gaps, what the form cannot express, runner bugs, seeded mistakes, line counts. `probes/` holds each probe the notes cite.

No raters this round: the person asked for the pros, cons and limits, not a score.

| axis | loguru file | what it presses on |
|---|---|---|
| outside state and time | `test_filesink_delay.py` (10 items) | files in a temporary directory, a frozen clock in a `with`, an assert in the middle of a test, a check helper with a loop |
| errors and captured output | `test_add_option_catch.py` (11 items) | sinks that raise, captured stdout and stderr, a patched `sys.stderr`, a worker thread (`enqueue=True`), a `with` that swaps `threading.excepthook` |
| concurrency | `test_threading.py` (7 items) | threads that the test starts and joins, a barrier, sleeps, an expected value that is one of two orders |

Run each from its `tests/` directory with Python 3.12 on Linux, loguru 0.8.0 and freezegun 1.5.0:

```text
python ../../../../run.py .                      # the port
python -m pytest -q -p no:cacheprovider tests    # the original, from stress/<axis>/pytest/
```

| file | original under pytest | port under the form | first attempt, written as close to the original as FORM.md seemed to allow |
|---|---|---|---|
| `test_filesink_delay.py` | 10 passed | 10 passed, 20 checks | 0 passed, 10 refused |
| `test_add_option_catch.py` | 11 passed | 11 passed, 31 checks | 4 passed, 7 refused |
| `test_threading.py` | 7 passed | 7 passed, 14 checks | 0 passed, 7 refused |
