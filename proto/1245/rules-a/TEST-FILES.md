# Which files are test files

A test file is a file whose name matches `test_*.py`. To choose other names, set `test-files` under `[tool.purba]` to a list of patterns from this closed list:

| pattern | a name matches when it |
|---|---|
| `test_*.py` | starts with `test_` and ends with `.py` |
| `*_test.py` | ends with `_test.py`, with at least one character before it |
| `tests.py` | is `tests.py` |
| `test.py` | is `test.py` |

```toml
[tool.purba]
test-files = ["*_test.py"]
```

The list replaces `test_*.py`. With the table above, `tests/b_test.py` runs and `tests/test_a.py` does not. To keep both, list both: `["test_*.py", "*_test.py"]`. A path you name must match the list too:

```text
purba: invocation error: tests/test_a.py names no test file: its name does not match *_test.py
```

`test-files` is only read from a `pyproject.toml` that holds `[tool.purba]`.

## Errors

purba names each error that it finds in `test-files` as a project error on the `pyproject.toml`, by its path from the working folder. It then runs no test, prints the count, and exits 4.

An item that is not on the list, with the nearest pattern that is:

```text
pyproject.toml project error: [tool.purba] test-files: test*.py is not a pattern purba takes; nearest: test_*.py
0 cases; 1 project error
```

From `tests/`, the same error names `../pyproject.toml`:

```text
../pyproject.toml project error: [tool.purba] test-files: test*.py is not a pattern purba takes; nearest: test_*.py
```

Several bad items give one line each. An item that is not a string is shown as you wrote it:

```text
pyproject.toml project error: [tool.purba] test-files: tset_*.py is not a pattern purba takes; nearest: test_*.py
pyproject.toml project error: [tool.purba] test-files: tests_*.py is not a pattern purba takes; nearest: test_*.py
pyproject.toml project error: [tool.purba] test-files: 7 is not a pattern purba takes; nearest: test.py
0 cases; 3 project errors
```

A value that is not a list, such as `test-files = "*_test.py"` or `test-files = 5`:

```text
pyproject.toml project error: [tool.purba] test-files: a string, not a list
pyproject.toml project error: [tool.purba] test-files: a number, not a list
```

An empty list, `test-files = []`:

```text
pyproject.toml project error: [tool.purba] test-files: an empty list
```
