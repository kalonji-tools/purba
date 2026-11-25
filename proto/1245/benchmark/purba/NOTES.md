# Port notes

`PURBA_IMPLICIT_ROOT=1 purba` from `port/` gives `20 cases: 20 passed` (exit 0), the same count as pytest on the original. The inputs and expected values have not changed.

## Differences from the original

1. **Parametrize.** Each row is now its own test, and I chose the names. Rule: TESTS.md, "no fixtures and no parametrize"; "Each test is one case".
2. **Fixtures.** `writer` and `reset_logger` are now helpers. Each test calls `make_writer()`, which resets the logger. There is no teardown, because each test has its own process. Rule: TESTS.md, "a parameter that has no default".
3. **`pytest.raises`.** It is replaced by a helper, `raised(action)`, plus `isinstance(error, T)` and `re.search(pattern, str(error)) is not None` (`match=` also uses `re.search`). Reason: `[foreign-framework]` (see below).
4. **`raised` has no `return None`.** That line never ran, and purba refused the case: `[second-path] line 140 reaches line 41 of raised, which did not run`.

## Rules that are missing, wrong or unclear

- **`[foreign-framework]` is missing.** Importing pytest blocks every case in the module: `line 2 imports pytest, and a test module imports no other test framework`.
- **`[computed-expected]` is missing.** `assert type(error) is TypeError` gives `line 10 computes TypeError, and an expected value is a literal`. The rules never require a literal.
- **"One path" is unclear.** The rule names only `if`.
  - In the test body, an `if` is refused even when its body runs: `line 3 went one way only`.
  - A `try/except` and a `for` loop in the test body pass.
  - Inside a helper, only a line that did not run is refused. The message, "reaches line N of <helper>", is not documented.
- **Private names.** `reset_logger()` reads `loguru._logger`, and no warning appears. The warning is only for reads in the test body ("a test that reads"), so a helper can hide a read.
- **ROOT.md** gives no example from a subfolder. From `tests/`, purba prints `add it to ../pyproject.toml`.

## Surprises

- The root line shows the full `/tmp/...` path. Only paths under the home folder become `~`.
- A `test-files` project error goes to stdout. Only `purba:` lines go to stderr.
- `PURBA_IMPLICIT_ROOT=1` does not stop a bad `test-files` error: the exit code is still 4.
- `test-files = ["*_test.py"]` with no file that matches prints `0 cases`, and the exit code is 2.

Every other output matched the rules. The outputs are in `../scratch/out/`.
